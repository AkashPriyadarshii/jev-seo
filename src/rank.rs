use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankDelta {
    pub domain: String,
    pub term: String,
    pub prev_rank: Option<usize>,
    pub curr_rank: Option<usize>,
    pub serp_url: Option<String>,
}

pub struct DbStore {
    conn: Connection,
}

impl DbStore {
    pub fn open() -> Result<Self> {
        let path = std::env::var("JEV_SEO_DB").ok().unwrap_or_else(|| {
            std::env::var("HOME")
                .map(|h| format!("{}/.jev-seo/jev-seo.db", h))
                .unwrap_or_else(|_| ".jev-seo.db".to_string())
        });
        Self::open_at(&path)
    }

    pub fn open_at(path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS keywords (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 domain TEXT NOT NULL,
                 term TEXT NOT NULL,
                 created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                 UNIQUE(domain, term)
             );
              CREATE TABLE IF NOT EXISTS rank_history (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  keyword_id INTEGER NOT NULL REFERENCES keywords(id),
                  position INTEGER,
                  serp_url TEXT,
                  checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
              );
               CREATE TABLE IF NOT EXISTS geo_history (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   target TEXT NOT NULL,
                   term TEXT NOT NULL,
                   score INTEGER NOT NULL,
                   checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
               );
                CREATE TABLE IF NOT EXISTS cite_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    target TEXT NOT NULL,
                    term TEXT NOT NULL,
                    cited INTEGER NOT NULL,
                    checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS crawl_snapshots (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   start_url TEXT NOT NULL,
                   pages INTEGER NOT NULL,
                   broken INTEGER NOT NULL,
                   checked_at DATETIME DEFAULT CURRENT_TIMESTAMP
               );
                -- Provenance per observation: a Tavily top-20 and a DDG top-30
                -- must never merge into one bare number.
                CREATE TABLE IF NOT EXISTS rank_observations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    keyword_id INTEGER NOT NULL REFERENCES keywords(id),
                    position INTEGER,
                    serp_url TEXT,
                    provider TEXT NOT NULL DEFAULT 'ddg',
                    engine TEXT NOT NULL DEFAULT 'duckduckgo-html',
                    observed_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX IF NOT EXISTS idx_rank_history_keyword ON rank_history(keyword_id);
                CREATE INDEX IF NOT EXISTS idx_geo_history_target ON geo_history(target, term);
                CREATE INDEX IF NOT EXISTS idx_cite_history_target ON cite_history(target, term);
                CREATE INDEX IF NOT EXISTS idx_crawl_snapshots_url ON crawl_snapshots(start_url);
                CREATE INDEX IF NOT EXISTS idx_rank_obs_keyword ON rank_observations(keyword_id);",
        )?;
        Ok(Self { conn })
    }

    pub fn track_keyword(
        &mut self,
        domain: &str,
        term: &str,
        curr_rank: Option<usize>,
        serp_url: Option<&str>,
        provider: &str,
        engine: &str,
    ) -> Result<RankDelta> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO keywords (domain, term) VALUES (?1, ?2)",
            params![domain, term],
        )?;

        let keyword_id: i64 = tx.query_row(
            "SELECT id FROM keywords WHERE domain = ?1 AND term = ?2",
            params![domain, term],
            |row| row.get(0),
        )?;

        let prev_rank: Option<usize> = tx
            .query_row(
                "SELECT position FROM rank_history WHERE keyword_id = ?1 ORDER BY id DESC LIMIT 1",
                params![keyword_id],
                |row| {
                    let pos: Option<i64> = row.get(0)?;
                    Ok(pos.map(|p| p as usize))
                },
            )
            .unwrap_or(None);

        let curr_pos_i64 = curr_rank.map(|p| p as i64);
        tx.execute(
            "INSERT INTO rank_history (keyword_id, position, serp_url) VALUES (?1, ?2, ?3)",
            params![keyword_id, curr_pos_i64, serp_url],
        )?;
        tx.execute(
            "INSERT INTO rank_observations (keyword_id, position, serp_url, provider, engine) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![keyword_id, curr_pos_i64, serp_url, provider, engine],
        )?;
        tx.commit()?;

        Ok(RankDelta {
            domain: domain.to_string(),
            term: term.to_string(),
            prev_rank,
            curr_rank,
            serp_url: serp_url.map(|s| s.to_string()),
        })
    }

    /// Provenance trail for a tracked term, newest first: position, provider,
    /// engine. Powers per-engine drift views without touching rank_history.
    pub fn observation_trail(&self, domain: &str, term: &str) -> Result<Vec<(Option<usize>, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT o.position, o.provider, o.engine FROM rank_observations o
             JOIN keywords k ON k.id = o.keyword_id
             WHERE k.domain = ?1 AND k.term = ?2 ORDER BY o.id DESC",
        )?;
        let rows = stmt.query_map(params![domain, term], |row| {
            let pos: Option<i64> = row.get(0)?;
            let provider: String = row.get(1)?;
            let engine: String = row.get(2)?;
            Ok((pos.map(|p| p as usize), provider, engine))
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(anyhow::Error::from)
    }

    /// Record a GEO score, returning the previous score for the delta line.
    pub fn record_geo(&self, target: &str, term: &str, score: u32) -> Result<Option<u32>> {
        let prev: Option<u32> = self
            .conn
            .query_row(
                "SELECT score FROM geo_history WHERE target = ?1 AND term = ?2 ORDER BY id DESC LIMIT 1",
                params![target, term],
                |row| row.get(0),
            )
            .unwrap_or(None);
        self.conn.execute(
            "INSERT INTO geo_history (target, term, score) VALUES (?1, ?2, ?3)",
            params![target, term, score as i64],
        )?;
        Ok(prev)
    }

    /// Record an MCP sampling citation check, returning the previous verdict.
    pub fn record_cite(&self, target: &str, term: &str, cited: bool) -> Result<Option<bool>> {
        let prev: Option<bool> = self
            .conn
            .query_row(
                "SELECT cited FROM cite_history WHERE target = ?1 AND term = ?2 ORDER BY id DESC LIMIT 1",
                params![target, term],
                |row| {
                    let v: i64 = row.get(0)?;
                    Ok(v != 0)
                },
            )
            .ok();
        self.conn.execute(
            "INSERT INTO cite_history (target, term, cited) VALUES (?1, ?2, ?3)",
            params![target, term, cited as i64],
        )?;
        Ok(prev)
    }

    /// Store a crawl snapshot, returning the previous (pages, broken) pair for --diff.
    pub fn record_crawl_snapshot(
        &self,
        start_url: &str,
        pages: i64,
        broken: i64,
    ) -> Result<Option<(i64, i64)>> {
        let prev: Option<(i64, i64)> = self
            .conn
            .query_row(
                "SELECT pages, broken FROM crawl_snapshots WHERE start_url = ?1 ORDER BY id DESC LIMIT 1",
                params![start_url],
                |row| {
                    let pages: i64 = row.get(0)?;
                    let broken: i64 = row.get(1)?;
                    Ok((pages, broken))
                },
            )
            .ok();
        self.conn.execute(
            "INSERT INTO crawl_snapshots (start_url, pages, broken) VALUES (?1, ?2, ?3)",
            params![start_url, pages, broken],
        )?;
        Ok(prev)
    }
}
