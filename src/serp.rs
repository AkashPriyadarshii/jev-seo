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

/// Search through a real search API (Tavily). Results already carry the page's
/// extracted text, so `brief` can summarise competitors without a second fetch.
///
/// The endpoint and key come from the environment so the key never lands in source:
///   TAVILY_API_KEY  (required to enable this path)
///   TAVILY_API_URL  (optional; defaults to Tavily's public endpoint, and can point
///                    at any Tavily-compatible service, including a self-hosted proxy)
fn tavily_search(query: &str, limit: usize) -> Result<Vec<SerpItem>> {
    let key = std::env::var("TAVILY_API_KEY")
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
        .timeout(std::time::Duration::from_secs(45))
        .send_string(&payload.to_string())
        .context("Failed to reach the search API endpoint")?;

    let v: serde_json::Value = resp.into_json().context("search API returned non-JSON")?;
    let arr = v
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
        // char-safe truncation: String::truncate panics on a non-UTF8 boundary
        let snippet: String = r
            .get("content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .chars()
            .take(400)
            .collect();
        let position = items.len() + 1;
        items.push(SerpItem { position, title, url, snippet });
    }
    if items.is_empty() {
        anyhow::bail!("search API returned zero results");
    }
    Ok(items)
}

pub fn scrape_serp(query: &str, limit: usize) -> Result<Vec<SerpItem>> {
    // DuckDuckGo's HTML endpoint answers non-browser clients with an image CAPTCHA
    // ("select all squares containing a duck") served by their `anomaly` system.
    // The decision is probabilistic and mixes header completeness, request rate and
    // client fingerprint, so it cannot be made reliable. When a real search API is
    // configured, use it; keep the scrape only as a fallback.
    if std::env::var("TAVILY_API_KEY").map(|k| !k.trim().is_empty()).unwrap_or(false) {
        match tavily_search(query, limit) {
            Ok(items) => return Ok(items),
            Err(e) => eprintln!("[jev-seo] search API failed ({}); falling back to DuckDuckGo", e),
        }
    }

    let form_data = format!("q={}&b=&kl=us-en", urlencoding::encode(query));
    // Bare UA+Content-Type scores as a bot; a coherent browser header set clears the
    // challenge when the request is not rate-limited (verified: bare = blocked,
    // +Accept = 10 results). Necessary but not sufficient — hence the API path above.
    let resp = ureq::post("https://html.duckduckgo.com/html/")
        .set("User-Agent", USER_AGENT)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
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
