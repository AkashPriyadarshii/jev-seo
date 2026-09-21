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
    let form_data = format!("q={}&b=&kl=us-en", urlencoding::encode(query));
    let resp = ureq::post("https://html.duckduckgo.com/html/")
        .set("User-Agent", USER_AGENT)
        .set("Content-Type", "application/x-www-form-urlencoded")
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
