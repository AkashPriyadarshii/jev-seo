//! Fetch backends raced for speed, picked by quality math.
//! Default stays free: direct fetch races Jina reader, Firecrawl only
//! escalates on failure with a key set. Every pick records source and cost.

use anyhow::{Context, Result};
use std::time::{Duration, Instant};

pub const JINA_ENDPOINT: &str = "https://r.jina.ai/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    /// Race direct + Jina, escalate to Firecrawl on failure.
    Auto,
    Direct,
    Jina,
    Firecrawl,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub body: String,
    pub source: &'static str,
    pub elapsed_ms: u128,
    /// Paid credits this fetch cost. 0 for free backends.
    pub cost: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Budget {
    /// Max paid credits per run. 0 = paid backends stay parked.
    pub max_credits: u32,
    pub spent: u32,
}

impl Budget {
    pub fn allow(&mut self, cost: u32) -> bool {
        if self.spent + cost <= self.max_credits {
            self.spent += cost;
            true
        } else {
            false
        }
    }

    pub fn refund(&mut self, cost: u32) {
        self.spent = self.spent.saturating_sub(cost);
    }
}

/// Quality 0.0-1.0: has body, markdown density, heading presence.
/// Empty scores 0 so upgrades always fire on missing content.
pub fn quality(body: &str) -> f64 {
    if body.trim().is_empty() {
        return 0.0;
    }
    let mut q = 0.2;
    let lines: Vec<&str> = body.lines().collect();
    let md_lines = lines.iter().filter(|l| {
        let t = l.trim_start();
        t.starts_with('#') || t.starts_with('-') || t.starts_with("* ") || t.starts_with("1.")
    }).count();
    q += 0.5 * (md_lines as f64 / lines.len().max(1) as f64).min(1.0);
    if body.contains("# ") {
        q += 0.3;
    }
    q.min(1.0)
}

fn jina_key() -> Option<String> {
    std::env::var("JINA_API_KEY").ok().filter(|k| !k.trim().is_empty())
}

fn firecrawl_key() -> Option<String> {
    std::env::var("FIRECRAWL_API_KEY").ok().filter(|k| !k.trim().is_empty())
}

fn firecrawl_endpoint() -> String {
    std::env::var("FIRECRAWL_API_URL")
        .map(|b| {
            let b = b.trim_end_matches('/').to_string();
            if b.ends_with("/scrape") { b } else { format!("{}/scrape", b) }
        })
        .unwrap_or_else(|_| "https://api.firecrawl.dev/v1/scrape".to_string())
}

/// Jina reader: one GET, markdown back, no key at base tier.
pub fn jina_fetch(url: &str) -> Result<FetchResult> {
    let t0 = Instant::now();
    let target = format!("{}{}", JINA_ENDPOINT, url);
    let mut req = ureq::get(&target)
        .timeout(Duration::from_secs(15))
        .set("User-Agent", concat!("jev-seo/", env!("CARGO_PKG_VERSION")));
    if let Some(key) = jina_key() {
        req = req.set("Authorization", &format!("Bearer {}", key));
    }
    let body = req.call()?.into_string()?;
    Ok(FetchResult { body, source: "jina", elapsed_ms: t0.elapsed().as_millis(), cost: 0 })
}

/// Firecrawl scrape: JS-rendered markdown. Paid, key-gated.
/// No budget inside: callers debit before and refund on empty.
pub fn firecrawl_fetch(url: &str) -> Result<FetchResult> {
    let key = firecrawl_key().context("FIRECRAWL_API_KEY not set")?;
    let t0 = Instant::now();
    let payload = serde_json::json!({ "url": url, "formats": ["markdown"] });
    let resp = ureq::post(&firecrawl_endpoint())
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", key))
        .timeout(Duration::from_secs(30))
        .send_string(&payload.to_string())
        .context("firecrawl request failed")?;
    let body: serde_json::Value = resp.into_json()?;
    let md = body
        .get("data")
        .and_then(|d| d.get("markdown"))
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    if md.trim().is_empty() {
        anyhow::bail!("firecrawl returned no markdown");
    }
    Ok(FetchResult { body: md, source: "firecrawl", elapsed_ms: t0.elapsed().as_millis(), cost: 1 })
}
