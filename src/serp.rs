use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerpItem {
    pub position: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub fn get_autocomplete(query: &str) -> Result<Vec<String>> {
    let url = format!("https://duckduckgo.com/ac/?q={}&type=list", urlencoding::encode(query));
    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .context("Failed to query autocomplete")?;

    let json: serde_json::Value = resp.into_json()?;
    if let Some(arr) = json.as_array() {
        if arr.len() > 1 {
            if let Some(suggestions) = arr[1].as_array() {
                let list: Vec<String> = suggestions
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                return Ok(list);
            }
        }
    }
    Ok(vec![])
}

pub fn scrape_serp(query: &str, limit: usize) -> Result<Vec<SerpItem>> {
    scrape_serp_with(query, limit, Provider::Auto)
}

/// Search backend selection. Default stays free forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Tavily-compatible API when `TAVILY_API_KEY` is set, else DuckDuckGo.
    Auto,
    /// Free scrape, always available.
    Ddg,
    /// Paid API, explicit opt-in only.
    Tavily,
}

pub fn select_provider(want: Provider) -> Provider {
    match want {
        Provider::Auto => {
            if std::env::var("TAVILY_API_KEY").map(|k| !k.trim().is_empty()).unwrap_or(false) {
                Provider::Tavily
            } else {
                Provider::Ddg
            }
        }
        other => other,
    }
}

pub fn scrape_serp_with(query: &str, limit: usize, want: Provider) -> Result<Vec<SerpItem>> {
    match select_provider(want) {
        Provider::Tavily => match tavily_search(query, limit) {
            Ok(items) => Ok(items),
            Err(e) => {
                eprintln!("[jev-seo] paid search failed ({}); falling back to free scrape", e);
                scrape_ddg(query, limit)
            }
        },
        Provider::Ddg | Provider::Auto => scrape_ddg(query, limit),
    }
}

/// Tavily-compatible search API. Opt-in via env, never the default path:
/// TAVILY_API_KEY (enables it), TAVILY_API_URL (optional proxy override).
pub(crate) fn tavily_search(query: &str, limit: usize) -> Result<Vec<SerpItem>> {    let key = std::env::var("TAVILY_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .context("TAVILY_API_KEY not set")?;
    let endpoint = std::env::var("TAVILY_API_URL")
        .unwrap_or_else(|_| "https://api.tavily.com/search".to_string());
    let payload = serde_json::json!({
        "query": query,
        "max_results": limit.clamp(1, 20),
        "search_depth": "advanced",
        "include_answer": false,
    });
    let resp = ureq::post(&endpoint)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", key))
        .timeout(std::time::Duration::from_secs(15))
        .send_string(&payload.to_string())
        .context("search API request failed")?;
    let body: serde_json::Value = resp.into_json()?;
    let arr = body
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    let mut items = Vec::new();
    for r in arr.iter() {
        let url = r.get("url").and_then(|x| x.as_str()).unwrap_or("").to_string();
        if url.is_empty() {
            continue;
        }
        let title = r.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let snippet: String = r
            .get("content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .chars()
            .take(400)
            .collect();
        items.push(SerpItem { position: items.len() + 1, title, url, snippet });
    }
    if items.is_empty() {
        anyhow::bail!("search API returned zero results");
    }
    Ok(items)
}

fn scrape_ddg(query: &str, limit: usize) -> Result<Vec<SerpItem>> {
    let form_data = format!("q={}&b=&kl=us-en", urlencoding::encode(query));
    let resp = ureq::post("https://html.duckduckgo.com/html/")
        .set("User-Agent", USER_AGENT)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .set("Accept-Language", "en-US,en;q=0.9")
        .set("Referer", "https://html.duckduckgo.com/")
        .set("Origin", "https://html.duckduckgo.com")
        .set("Sec-Fetch-Dest", "document")
        .set("Sec-Fetch-Mode", "navigate")
        .set("Sec-Fetch-Site", "same-origin")
        .set("Sec-Fetch-User", "?1")
        .set("Upgrade-Insecure-Requests", "1")
        .timeout(std::time::Duration::from_secs(8))
        .send_string(&form_data)
        .context("Failed to request DuckDuckGo HTML SERP")?;

    let html = resp.into_string()?;
    let lower = html.to_lowercase();
    if lower.contains("anomaly") || lower.contains("captcha") || lower.contains("challenge-form") {
        anyhow::bail!("search blocked (bot challenge page) — no rank recorded");
    }
    let mut items = Vec::new();

    let result_re = Regex::new(r#"(?s)<div[^>]*class="[^"]*result\b[^"]*"[^>]*>(.*?)</div>\s*</div>"#)?;
    let title_re = Regex::new(r#"(?s)<a[^>]*class="[^"]*result__url[^"]*"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)?;
    let title_fallback_re = Regex::new(r#"(?s)<a[^>]*class="[^"]*result__snippet[^"]*"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)?;
    let link_re = Regex::new(r#"(?s)<a[^>]*class="[^"]*result__title[^"]*"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)?;
    let snippet_re = Regex::new(r#"(?s)<a[^>]*class="[^"]*result__snippet[^"]*"[^>]*>(.*?)</a>"#)?;
    let strip_html = Regex::new(r#"<[^>]+>"#)?;

    for cap in result_re.captures_iter(&html) {
        if items.len() >= limit {
            break;
        }

        let block = &cap[1];
        let link_match = link_re.captures(block)
            .or_else(|| title_re.captures(block))
            .or_else(|| title_fallback_re.captures(block));

        if let Some(lcap) = link_match {
            let raw_url = &lcap[1];
            let raw_title = &lcap[2];
            let clean_title = strip_html.replace_all(raw_title, "").trim().to_string();
            let clean_url = extract_actual_url(raw_url);

            let snippet = if let Some(scap) = snippet_re.captures(block) {
                strip_html.replace_all(&scap[1], "").trim().to_string()
            } else {
                String::new()
            };

            if !clean_title.is_empty() && !clean_url.is_empty() {
                items.push(SerpItem {
                    position: items.len() + 1,
                    title: clean_title,
                    url: clean_url,
                    snippet,
                });
            }
        }
    }

    if items.is_empty() {
        anyhow::bail!("no results parsed — search may have been blocked; no rank recorded");
    }
    Ok(items)
}

fn extract_actual_url(raw: &str) -> String {
    if let Some(pos) = raw.find("uddg=") {
        let remainder = &raw[pos + 5..];
        let end_pos = remainder.find('&').unwrap_or(remainder.len());
        let encoded = &remainder[..end_pos];
        return urlencoding::decode(encoded).unwrap_or_else(|_| encoded.into()).to_string();
    }
    raw.to_string()
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
    pub fn decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
        let decoded: Vec<u8> = url::form_urlencoded::parse(s.as_bytes())
            .into_owned()
            .flat_map(|(k, _)| k.into_bytes())
            .collect();
        String::from_utf8(decoded)
    }
}
