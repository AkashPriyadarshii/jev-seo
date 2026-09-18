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
        let conn = Connection::open(".jev-seo.db")?;
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
             );",
        )?;
        Ok(Self { conn })
    }

    pub fn track_keyword(
        &mut self,
        domain: &str,
        term: &str,
        curr_rank: Option<usize>,
        serp_url: Option<&str>,
    ) -> Result<RankDelta> {
        self.conn.execute(
            "INSERT OR IGNORE INTO keywords (domain, term) VALUES (?1, ?2)",
            params![domain, term],
        )?;

        let keyword_id: i64 = self.conn.query_row(
            "SELECT id FROM keywords WHERE domain = ?1 AND term = ?2",
            params![domain, term],
            |row| row.get(0),
        )?;

        let prev_rank: Option<usize> = self
            .conn
            .query_row(
                "SELECT position FROM rank_history WHERE keyword_id = ?1 ORDER BY checked_at DESC LIMIT 1",
                params![keyword_id],
                |row| {
                    let pos: Option<i64> = row.get(0)?;
                    Ok(pos.map(|p| p as usize))
                },
            )
            .unwrap_or(None);

        let curr_pos_i64 = curr_rank.map(|p| p as i64);
        self.conn.execute(
            "INSERT INTO rank_history (keyword_id, position, serp_url) VALUES (?1, ?2, ?3)",
            params![keyword_id, curr_pos_i64, serp_url],
        )?;

        Ok(RankDelta {
            domain: domain.to_string(),
            term: term.to_string(),
            prev_rank,
            curr_rank,
            serp_url: serp_url.map(|s| s.to_string()),
        })
    }
}
