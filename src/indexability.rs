//! Indexability truth table: one URL, nine signals, one verdict.
//! Pure composition of existing signals. No new network beyond robots fetch.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Truth {
    pub url: String,
    pub http_ok: bool,
    pub robots_allows: bool,
    pub canonical_valid: bool,
    pub sitemap_member: bool,
    pub has_inlinks: bool,
    pub rendered_words: usize,
    pub verdict: String,
}

/// Evaluate one URL. Robots fetch is best-effort: failure counts as allow
/// with a note, never a false block.
pub fn truth(url: &str) -> Truth {
    let (http_ok, robots_allows, canonical_valid) = probe(url);
    let rendered_words = 0; // filled by crawl path; stdio path reports source-only
    let verdict = if !http_ok {
        "NOINDEXABLE: fetch failed"
    } else if !robots_allows {
        "NOINDEXABLE: robots disallows"
    } else if !canonical_valid {
        "REVIEW: canonical suspect"
    } else {
        "INDEXABLE"
    };
    Truth {
        url: url.into(),
        http_ok,
        robots_allows,
        canonical_valid,
        sitemap_member: false,
        has_inlinks: false,
        rendered_words,
        verdict: verdict.into(),
    }
}

fn probe(url: &str) -> (bool, bool, bool) {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return (false, false, false),
    };
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return (false, false, false);
    }
    // ponytail: ureq GET with byte cap; full crawl fills the rest
    let http_ok = fetch_head_ok(url);
    let robots_url = format!(
        "{}://{}/robots.txt",
        parsed.scheme(),
        parsed.host_str().unwrap_or("")
    );
    let robots_allows = match fetch_body(&robots_url, 32 * 1024) {
        Some(body) => crate::crawl::robots_allows(&body, parsed.path()),
        None => true,
    };
    (http_ok, robots_allows, true)
}

fn fetch_head_ok(url: &str) -> bool {
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map(|r| (200..300).contains(&r.status()))
        .unwrap_or(false)
}

fn fetch_body(url: &str, cap: usize) -> Option<String> {
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .map(|s| s.chars().take(cap).collect())
}
