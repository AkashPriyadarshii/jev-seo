//! Live-site crawler: BFS over one host, seeded from sitemap.xml when present.
//! Reports broken links, redirect chains, and orphan pages. Polite by design:
//! one sequential request stream, robots.txt honored, private hosts refused.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{mpsc, LazyLock};
use std::time::{Duration, Instant};
use url::Url;

pub const CRAWL_UA: &str = concat!("jev-seo/", env!("CARGO_PKG_VERSION"), " (TypeSafe Jev Search Radar Crawler)");
pub const DEFAULT_MAX_PAGES: usize = 50;
pub const SLOW_PAGE_MS: u128 = 800;
/// Largest page body kept in memory. Bigger pages truncate, never OOM.
pub const MAX_BODY_BYTES: usize = 2_000_000;

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<a\b[^>]*\bhref\s*=\s*["']([^"']+?)["']"#).expect("link regex"));
static LOC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<loc>(.*?)</loc>"#).expect("loc regex"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRecord {
    pub url: String,
    pub status: u16,
    pub final_url: String,
    pub outlinks: usize,
    pub elapsed_ms: u128,
    pub bytes: usize,
    pub hops: Vec<(u16, String)>,
    pub encoding: Option<String>,
    #[serde(default = "direct_source")]
    pub source: String,
    #[serde(default)]
    pub fetch_cost: u32,
}

fn direct_source() -> String {
    "direct".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlReport {
    pub start_url: String,
    pub pages_crawled: usize,
    pub score: u32,
    pub grade: String,
    pub areas: Vec<crate::rules::AreaScore>,
    pub actions: Vec<crate::actions::Action>,
    #[serde(default)]
    pub findings: Vec<crate::rules::Finding>,
    pub broken: Vec<PageRecord>,
    pub redirects: Vec<(String, String)>,
    pub orphans: Vec<String>,
    pub errors: Vec<String>,
    pub pages: Vec<PageRecord>,
    #[serde(default)]
    pub inbound: HashMap<String, usize>,
    pub seeded_from_sitemap: bool,
    pub capped: bool,
    pub robots_honored: bool,
}

/// Same-host anchor hrefs resolved to absolute canonical URLs.
pub fn extract_links(html: &str, base: &Url) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for cap in LINK_RE.captures_iter(html) {
        let raw = cap[1].trim();
        if raw.is_empty()
            || raw.starts_with("mailto:")
            || raw.starts_with("javascript:")
            || raw.starts_with("tel:")
        {
            continue;
        }
        let joined = match base.join(raw) {
            Ok(u) => u,
            Err(_) => continue,
        };
        if joined.scheme() != "http" && joined.scheme() != "https" {
            continue;
        }
        if joined.host_str() != base.host_str() {
            continue;
        }
        let mut clean = joined.to_string();
        if let Some(pos) = clean.find('#') {
            clean.truncate(pos);
        }
        let canon = canonicalize(&clean);
        if seen.insert(canon.clone()) {
            out.push(canon);
        }
    }
    out
}

/// Canonical form for dedup: lowercase host, no fragment, no tracking
/// params, no /index.html tail, single trailing-slash policy.
pub fn canonicalize(raw: &str) -> String {
    let mut u = match Url::parse(raw) {
        Ok(u) => u,
        Err(_) => return raw.to_string(),
    };
    u.set_fragment(None);
    let kept: Vec<(String, String)> = u
        .query_pairs()
        .filter(|(k, _)| {
            let k = k.to_lowercase();
            !(k == "amp"
                || k.starts_with("utm_")
                || k == "fbclid"
                || k == "gclid"
                || k == "msclkid"
                || k == "yclid")
        })
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    if kept.is_empty() {
        u.set_query(None);
    } else {
        let mut qp = u.query_pairs_mut();
        qp.clear();
        for (k, v) in &kept {
            qp.append_pair(k, v);
        }
    }
    let mut path = u.path().to_string();
    for tail in ["/index.html", "/index.htm"] {
        if path.ends_with(tail) {
            path.truncate(path.len() - tail.len());
            break;
        }
    }
    if path.len() > 1 {
        path = path.trim_end_matches('/').to_string();
    }
    if path.is_empty() {
        path.push('/');
    }
    u.set_path(&path);
    u.to_string()
}

/// Longest-match allow check against one robots.txt body. Empty Disallow
/// allows everything; on a length tie Allow beats Disallow. Unknown or empty
/// body allows. Pure function, no network.
pub fn robots_allows(body: &str, path: &str) -> bool {
    let mut groups: Vec<(Vec<String>, Vec<String>, Vec<String>)> = Vec::new();
    let mut agents: Vec<String> = Vec::new();
    let mut disallows: Vec<String> = Vec::new();
    let mut allows: Vec<String> = Vec::new();
    for line in body.lines() {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }
        if let Some(pos) = clean.find(':') {
            let key = clean[..pos].trim().to_lowercase();
            let val = clean[pos + 1..].trim().to_string();
            if key == "user-agent" {
                if !disallows.is_empty() || !allows.is_empty() {
                    groups.push((std::mem::take(&mut agents), std::mem::take(&mut disallows), std::mem::take(&mut allows)));
                }
                agents.push(val.to_lowercase());
            } else if key == "disallow" {
                disallows.push(val);
            } else if key == "allow" {
                allows.push(val);
            }
        }
    }
    if !agents.is_empty() {
        groups.push((agents, disallows, allows));
    }
    let mut best_dis = 0usize;
    let mut best_allow = 0usize;
    let mut matched = false;
    for (agents, disallows, allows) in &groups {
        if !agents.iter().any(|a| a == "*" || a.contains("jev-seo")) {
            continue;
        }
        matched = true;
        for d in disallows {
            if !d.is_empty() && path.starts_with(d.as_str()) {
                best_dis = best_dis.max(d.len());
            }
        }
        for a in allows {
            if !a.is_empty() && path.starts_with(a.as_str()) {
                best_allow = best_allow.max(a.len());
            }
        }
    }
    if !matched {
        return true;
    }
    // Longest match wins; on a length tie Allow beats Disallow.
    best_allow >= best_dis
}

/// Seed URLs from a sitemap XML body. Same loc pattern as the auditor.
pub fn sitemap_seed_urls(xml: &str) -> Vec<String> {
    LOC_RE
        .captures_iter(xml)
        .map(|c| c[1].trim().to_string())
        .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
        .collect()
}

struct Fetch {
    status: u16,
    final_url: String,
    body: String,
    elapsed_ms: u128,
    bytes: usize,
    hops: Vec<(u16, String)>,
    encoding: Option<String>,
}

fn fetch_page(url: &str) -> Fetch {
    let agent: ureq::Agent = ureq::AgentBuilder::new().redirects(0).build();
    let base_host = Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_default();
    let mut current = url.to_string();
    let mut hops = Vec::new();
    let start = Instant::now();
    let done = |status: u16, final_url: String, body: String, bytes: usize, hops: Vec<(u16, String)>, encoding: Option<String>| {
        Fetch { status, final_url, body, elapsed_ms: start.elapsed().as_millis(), bytes, hops, encoding }
    };
    for _ in 0..6 {
        let resp = agent
            .get(&current)
            .timeout(Duration::from_secs(10))
            .set("User-Agent", CRAWL_UA)
            .call();
        match resp {
            Ok(r) => {
                let final_url = r.get_url().to_string();
                let is_html = r
                    .header("content-type")
                    .map(|c| c.contains("html"))
                    .unwrap_or(true);
                let encoding = r.header("content-encoding").map(str::to_string);
                let declared: Option<usize> = r.header("content-length").and_then(|v| v.parse().ok());
                let mut body = if is_html { r.into_string().unwrap_or_default() } else { String::new() };
                if body.len() > MAX_BODY_BYTES {
                    body.truncate(MAX_BODY_BYTES);
                }
                let bytes = declared.unwrap_or(body.len());
                return done(200, final_url, body, bytes, hops, encoding);
            }
            Err(ureq::Error::Status(code, r)) if (300..400).contains(&code) => {
                let loc = r.header("location").unwrap_or("").to_string();
                hops.push((code, current.clone()));
                let next = Url::parse(&current).and_then(|b| b.join(&loc));
                match next {
                    Ok(n)
                        if n.host_str() == Some(base_host.as_str())
                            && crate::paths::reject_private_url(n.as_str()).is_ok() =>
                    {
                        current = n.to_string()
                    }
                    _ => return done(code, current, String::new(), 0, hops, None),
                }
            }
            Err(ureq::Error::Status(code, r)) => {
                let final_url = r.get_url().to_string();
                return done(code, final_url, String::new(), 0, hops, None);
            }
            Err(_) => return done(0, current, String::new(), 0, hops, None),
        }
    }
    done(0, current, String::new(), 0, hops, None)
}

pub fn crawl_site(
    start_url: &str,
    max_pages: usize,
    mode: crate::fetch::FetchMode,
    budget: &mut crate::fetch::Budget,
) -> Result<CrawlReport> {
    let start = crate::paths::reject_private_url(start_url)?;
    let start_clean = canonicalize(start.as_str());
    let t0 = Instant::now();
    let stamp = || {
        let s = t0.elapsed().as_secs();
        format!("[jev-seo {:>02}:{:>02}]", s / 60, s % 60)
    };
    let host = start.host_str().unwrap_or("").to_string();
    let robots_body = ureq::get(&format!("{}://{}/robots.txt", start.scheme(), host))
        .timeout(Duration::from_secs(8))
        .set("User-Agent", CRAWL_UA)
        .call()
        .ok()
        .filter(|r| crate::paths::reject_redirect_target(r.get_url()).is_ok())
        .and_then(|r| r.into_string().ok())
        .unwrap_or_default();
    eprintln!("{} robots.txt {}", stamp(), if robots_body.is_empty() { "missing" } else { "ok" });

    let sitemap_urls: Vec<String> = ureq::get(&format!("{}://{}/sitemap.xml", start.scheme(), host))
        .timeout(Duration::from_secs(10))
        .set("User-Agent", CRAWL_UA)
        .call()
        .ok()
        .filter(|r| crate::paths::reject_redirect_target(r.get_url()).is_ok())
        .and_then(|r| r.into_string().ok())
        .map(|xml| sitemap_seed_urls(&xml))
        .unwrap_or_default();
    let seeded = !sitemap_urls.is_empty();
    let mut seeds: Vec<String> = sitemap_urls
        .into_iter()
        .map(|u| canonicalize(&u))
        .take(max_pages.max(1))
        .collect();
    if seeds.is_empty() {
        seeds.push(start_clean.clone());
    }
    eprintln!("{} {} seed pages{}", stamp(), seeds.len(), if seeded { " (sitemap)" } else { " (start URL)" });

    let mut queue: VecDeque<String> = seeds.into_iter().collect();
    let mut visited: HashSet<String> = HashSet::new();
    let mut pages: Vec<PageRecord> = Vec::new();
    let mut inbound: HashMap<String, usize> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut redirects: Vec<(String, String)> = Vec::new();

    struct Done {
        url: String,
        fetch: Fetch,
        source: &'static str,
        cost: u32,
    }

    // Level-by-level parallel fetch: 8 workers per batch, cheap parse and
    // robots checks stay single-threaded. std only, no runtime dependency.
    // One shared budget across batches so paid backends cannot overspend.
    const WORKERS: usize = 8;
    let cap = max_pages.max(1);
    let budget_arc = std::sync::Arc::new(std::sync::Mutex::new(budget.clone()));
    while pages.len() < cap {
        let mut batch: Vec<String> = Vec::new();
        while batch.len() < WORKERS && pages.len() + batch.len() < cap {
            match queue.pop_front() {
                Some(u) => {
                    if !visited.insert(u.clone()) {
                        continue;
                    }
                    let parsed = match Url::parse(&u) {
                        Ok(p) => p,
                        Err(e) => {
                            errors.push(format!("{}: {}", u, e));
                            continue;
                        }
                    };
                    if parsed.host_str() != start.host_str() {
                        continue;
                    }
                    if !robots_allows(&robots_body, parsed.path()) {
                        continue;
                    }
                    batch.push(u);
                }
                None => break,
            }
        }
        if batch.is_empty() {
            break;
        }
        let (tx, rx) = mpsc::channel::<Done>();
        std::thread::scope(|s| {
            for url in &batch {
                let tx = tx.clone();
                let budget_arc = budget_arc.clone();
                s.spawn(move || {
                    let mut fetch = fetch_page(url);
                    let mut source = "direct";
                    let mut cost = 0u32;
                    // Upgrade weak bodies only. Direct wins outright on good pages.
                    if !matches!(mode, crate::fetch::FetchMode::Direct)
                        && crate::fetch::quality(&fetch.body) < 0.4
                    {
                        if matches!(mode, crate::fetch::FetchMode::Auto | crate::fetch::FetchMode::Jina) {
                            if let Ok(j) = crate::fetch::jina_fetch(url) {
                                if crate::fetch::quality(&j.body) > crate::fetch::quality(&fetch.body) {
                                    fetch.bytes = j.body.len();
                                    fetch.body = j.body;
                                    fetch.elapsed_ms += j.elapsed_ms;
                                    source = j.source;
                                    cost = j.cost;
                                }
                            }
                        }
                        if matches!(mode, crate::fetch::FetchMode::Auto | crate::fetch::FetchMode::Firecrawl) {
                            // Debit under a short lock, fetch outside it.
                            // Refund only when no call happened; billed calls stay billed.
                            let debited = budget_arc
                                .lock()
                                .map(|mut b| b.allow(1))
                                .unwrap_or(false);
                            if debited {
                                match crate::fetch::firecrawl_fetch(url) {
                                    Ok(f) => {
                                        if !f.body.trim().is_empty()
                                            && crate::fetch::quality(&f.body) > crate::fetch::quality(&fetch.body)
                                        {
                                            fetch.bytes = f.body.len();
                                            fetch.body = f.body;
                                            fetch.elapsed_ms += f.elapsed_ms;
                                            source = f.source;
                                            cost = f.cost;
                                        }
                                    }
                                    Err(_) => {
                                        if let Ok(mut b) = budget_arc.lock() {
                                            b.refund(1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let _ = tx.send(Done { url: url.clone(), fetch, source, cost });
                });
            }
        });
        drop(tx);
        for d in rx {
            let f = d.fetch;
            if f.status == 0 {
                errors.push(format!("{}: fetch failed", d.url));
            }
            // Raw compare modulo trailing slash: canonical-only rewrites
            // (tracking params, index.html) must not count as redirects.
            if f.final_url.trim_end_matches('/') != d.url.trim_end_matches('/') && f.status != 0 {
                redirects.push((d.url.clone(), f.final_url.clone()));
            }
            let outlinks = if f.status == 200 && !f.body.is_empty() {
                match Url::parse(&f.final_url) {
                    Ok(base) => {
                        let links = extract_links(&f.body, &base);
                        for l in &links {
                            *inbound.entry(l.clone()).or_insert(0) += 1;
                            if !visited.contains(l) {
                                queue.push_back(l.clone());
                            }
                        }
                        links.len()
                    }
                    Err(_) => 0,
                }
            } else {
                0
            };
            pages.push(PageRecord {
                url: d.url,
                status: f.status,
                final_url: f.final_url,
                outlinks,
                elapsed_ms: f.elapsed_ms,
                bytes: f.bytes,
                hops: f.hops,
                encoding: f.encoding,
                source: d.source.to_string(),
                fetch_cost: d.cost,
            });
        }
        eprintln!("{} crawled {} pages", stamp(), pages.len());
    }

    inbound.entry(start_clean.clone()).or_insert(0);
    let capped = pages.len() >= cap && !queue.is_empty();
    if let Ok(b) = budget_arc.lock() {
        *budget = b.clone();
    }
    Ok(finish_report(ReportParts {
        start_url: start_clean,
        pages,
        redirects,
        inbound,
        errors,
        seeded_from_sitemap: seeded,
        capped,
        robots_honored: !robots_body.is_empty(),
    }))
}

/// Assemble score, grade, areas, and actions from crawl facts.
/// Shared by live crawls and offline --rescore runs.
pub struct ReportParts {
    pub start_url: String,
    pub pages: Vec<PageRecord>,
    pub redirects: Vec<(String, String)>,
    pub inbound: HashMap<String, usize>,
    pub errors: Vec<String>,
    pub seeded_from_sitemap: bool,
    pub capped: bool,
    pub robots_honored: bool,
}

pub fn finish_report(parts: ReportParts) -> CrawlReport {
    let ReportParts {
        start_url,
        pages,
        redirects,
        inbound,
        errors,
        seeded_from_sitemap,
        capped,
        robots_honored,
    } = parts;
    let broken: Vec<PageRecord> = pages.iter().filter(|p| p.status >= 400 || p.status == 0).cloned().collect();
    let orphans: Vec<String> = pages
        .iter()
        .filter(|p| p.status == 200 && p.url != start_url && inbound.get(&p.url).copied().unwrap_or(0) == 0)
        .map(|p| p.url.clone())
        .collect();
    let mut rep = CrawlReport {
        start_url,
        pages_crawled: pages.len(),
        score: 0,
        grade: String::new(),
        areas: Vec::new(),
        actions: Vec::new(),
        findings: Vec::new(),
        broken,
        redirects,
        orphans,
        errors,
        pages,
        inbound,
        seeded_from_sitemap,
        capped,
        robots_honored,
    };
    // One scoring truth: the fifty-rule engine. Totals scope reach per area.
    let findings = crate::rules::check_crawl(&rep);
    let mut totals = HashMap::new();
    for area in [
        crate::rules::Area::Crawl,
        crate::rules::Area::Performance,
        crate::rules::Area::Security,
        crate::rules::Area::Canonical,
    ] {
        totals.insert(area, rep.pages_crawled);
    }
    let areas = crate::rules::score_areas(&findings, &totals);
    let score = crate::rules::overall(&areas);
    rep.score = score;
    rep.grade = crate::actions::grade(score).to_string();
    rep.areas = areas;
    rep.actions = crate::rules::actions_for(&findings);
    rep.findings = findings;
    rep
}
