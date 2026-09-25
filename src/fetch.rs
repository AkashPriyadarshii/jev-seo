//! Fetch backends raced for speed, picked by extracted-text word count.
//! Default stays free: direct fetch wins on real copy, Jina reader then
//! Firecrawl escalate on thin bodies with a key set. Every pick records source and cost.

use anyhow::{Context, Result};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub const JINA_ENDPOINT: &str = "https://r.jina.ai/";

/// Extra request headers applied to target-site fetches (crawl, robots,
/// sitemap, llms, site-scoring). Filled once from CLI flags early in main;
/// staging sites behind basic auth become auditable before launch.
static EXTRA_HEADERS: OnceLock<Vec<(String, String)>> = OnceLock::new();

/// Register extra headers from CLI flags. Called once at startup; later
/// callers (CLI and MCP) share the same set for the run.
pub fn set_extra_headers(user: Option<&str>, password: Option<&str>, raw: &[String]) {
    let mut headers: Vec<(String, String)> = Vec::new();
    if let (Some(u), Some(p)) = (user, password) {
        let token = crate::serp::base64_basic(&format!("{}:{}", u, p));
        headers.push(("Authorization".to_string(), format!("Basic {}", token)));
    }
    for r in raw {
        if let Some((k, v)) = r.split_once(':') {
            let k = k.trim();
            let v = v.trim();
            if !k.is_empty() && !v.is_empty() {
                headers.push((k.to_string(), v.to_string()));
            }
        }
    }
    let _ = EXTRA_HEADERS.set(headers);
}

/// Apply the run's extra headers to a request builder. No-op when none set.
pub fn with_extra_headers(mut req: ureq::Request) -> ureq::Request {
    if let Some(headers) = EXTRA_HEADERS.get() {
        for (k, v) in headers {
            req = req.set(k, v);
        }
    }
    req
}

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

/// Largest aux body kept in memory (robots, sitemap, reader upgrades).
/// Crawl page bodies use the bigger MAX_BODY_BYTES in crawl.rs.
pub const MAX_AUX_BYTES: usize = 512_000;

/// Read a response body with a hard byte cap. Truncates at a char boundary;
/// oversized bodies shrink instead of OOMing the run.
pub fn capped_string(resp: ureq::Response, cap: usize) -> anyhow::Result<String> {
    use std::io::Read;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(cap as u64 + 1)
        .read_to_end(&mut buf)?;
    let s = String::from_utf8_lossy(&buf);
    if s.len() <= cap {
        return Ok(s.into_owned());
    }
    let end = s.floor_char_boundary(cap);
    Ok(s[..end].to_string())
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

/// Opening passage: collapsed text right after the first H1, capped at
/// `limit` chars. Answer-first and next-step judgments must read what follows
/// the heading, not nav and banner copy above it. Falls back to plain
/// readable text when no H1 exists.
pub fn opening_after_h1(html: &str, limit: usize) -> String {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("</h1>").map(|i| i + "</h1>".len());
    let src = match start {
        Some(i) => &html[i.min(html.len())..],
        None => return readable_text(html, limit),
    };
    let mut text = String::with_capacity(src.len().min(limit * 2));
    let mut in_tag = false;
    let mut script = false;
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' {
            let tag: String = chars[i..].iter().take_while(|&&x| x != '>').collect();
            let tl = tag.to_ascii_lowercase();
            if tl.starts_with("<script") || tl.starts_with("<style") {
                script = true;
            } else if tl.starts_with("</script") || tl.starts_with("</style") {
                script = false;
            }
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            text.push(' ');
        } else if !in_tag && !script {
            text.push(c);
        }
        i += 1;
    }
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(limit)
        .collect()
}

/// Visible body copy for Jev state. Head-first raw markup scored markup, not
/// copy, so this strips scripts/styles, drops tags, and returns collapsed
/// body text capped at `limit` chars. Non-HTML input passes through to the cap.
pub fn readable_text(html: &str, limit: usize) -> String {
    let lower = html.to_ascii_lowercase();
    let mut src = html;
    if let Some(start) = lower.find("<body") {
        let from = start + "<body".len();
        if let Some(end_tag) = html[from..].find('>') {
            src = &html[from + end_tag + 1..];
        }
    }
    if let Some(end) = src.to_ascii_lowercase().find("</body>") {
        src = &src[..end];
    }
    let mut out = src.to_string();
    for tag in ["script", "style", "svg", "noscript"] {
        loop {
            let low = out.to_ascii_lowercase();
            let open = match low.find(&format!("<{tag}")) {
                Some(i) => i,
                None => break,
            };
            let close_tag = format!("</{tag}>");
            let rest = &low[open..];
            let cut = match rest.find(&close_tag) {
                Some(i) => open + i + close_tag.len(),
                None => out.len(),
            };
            out.replace_range(open..cut.min(out.len()), " ");
        }
    }
    let mut text = String::with_capacity(out.len());
    let mut in_tag = false;
    for c in out.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(limit).collect()
}

fn jina_key() -> Option<String> {
    std::env::var("JINA_API_KEY").ok().filter(|k| !k.trim().is_empty())
}

fn firecrawl_key() -> Option<String> {
    std::env::var("FIRECRAWL_API_KEY").ok().filter(|k| !k.trim().is_empty())
}

fn firecrawl_endpoint() -> anyhow::Result<String> {
    match std::env::var("FIRECRAWL_API_URL") {
        Ok(b) => {
            let b = b.trim_end_matches('/').to_string();
            let full = if b.ends_with("/scrape") { b } else { format!("{}/scrape", b) };
            crate::paths::reject_api_endpoint(&full, "FIRECRAWL_API_URL")
        }
        Err(_) => Ok("https://api.firecrawl.dev/v1/scrape".to_string()),
    }
}

/// Jina reader: one GET, markdown back, no key at base tier.
pub fn jina_fetch(url: &str) -> Result<FetchResult> {
    crate::paths::reject_private_url(url)?;
    let t0 = Instant::now();
    let target = format!("{}{}", JINA_ENDPOINT, url);
    let mut req = ureq::get(&target)
        .timeout(Duration::from_secs(15))
        .set("User-Agent", concat!("jev-seo/", env!("CARGO_PKG_VERSION")));
    if let Some(key) = jina_key() {
        req = req.set("Authorization", &format!("Bearer {}", key));
    }
    let body = capped_string(req.call()?, MAX_AUX_BYTES)?;
    Ok(FetchResult { body, source: "jina", elapsed_ms: t0.elapsed().as_millis(), cost: 0 })
}

/// Firecrawl scrape: JS-rendered markdown. Paid, key-gated.
/// No budget inside: callers debit before and refund on empty.
pub fn firecrawl_fetch(url: &str) -> Result<FetchResult> {
    crate::paths::reject_private_url(url)?;
    let key = firecrawl_key().context("FIRECRAWL_API_KEY not set")?;
    let t0 = Instant::now();
    let payload = serde_json::json!({ "url": url, "formats": ["markdown"] });
    let resp = ureq::post(&firecrawl_endpoint()?)
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
