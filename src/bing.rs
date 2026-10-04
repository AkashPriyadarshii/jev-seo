//! Bing Webmaster REST first-party data. Key-owned, explicit opt-in:
//! BING_WEBMASTER_KEY. v0.4.0 ships sites + query performance.
//! Crawl/keywords/AI endpoints reuse this client in v0.5.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const BASE: &str = "https://ssl.bing.com/webmaster/api.svc/json";

fn key() -> Result<String> {
    std::env::var("BING_WEBMASTER_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .context("BING_WEBMASTER_KEY not set")
}

/// Key gate: true only with a non-empty key. Checked before any network.
#[allow(dead_code)]
pub fn enabled() -> bool {
    key().is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub url: String,
}

/// Verified sites on this key.
pub fn sites() -> Result<Vec<Site>> {
    let k = key()?;
    let body: serde_json::Value = ureq::get(&format!("{BASE}/GetUserSites?apikey={k}"))
        .timeout(std::time::Duration::from_secs(20))
        .call()
        .context("bing GetUserSites failed")?
        .into_json()?;
    let sites = body
        .get("d")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    e.get("Url")
                        .and_then(|u| u.as_str())
                        .map(|u| Site { url: u.into() })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(sites)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRow {
    pub query: String,
    pub clicks: f64,
    pub impressions: f64,
}

/// Query performance for one site. Thin slice: rank/traffic endpoint.
pub fn query_stats(site: &str) -> Result<Vec<QueryRow>> {
    let k = key()?;
    let body: serde_json::Value = ureq::get(&format!(
        "{BASE}/GetQueryStats?apikey={k}&siteUrl={}",
        urlencoding(site)
    ))
    .timeout(std::time::Duration::from_secs(20))
    .call()
    .context("bing GetQueryStats failed")?
    .into_json()?;
    let rows = body
        .get("d")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .map(|r| QueryRow {
                    query: r
                        .get("Query")
                        .and_then(|q| q.as_str())
                        .unwrap_or("")
                        .to_string(),
                    clicks: r.get("Clicks").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    impressions: r.get("Impressions").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(rows)
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}
