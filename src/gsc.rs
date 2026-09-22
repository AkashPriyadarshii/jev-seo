//! Google Search Console: free first-party query data for your own sites.
//! OAuth device flow, no browser embedding. Client id comes from env because
//! every user registers their own Desktop OAuth client once:
//! GOOGLE_CLIENT_ID + GOOGLE_CLIENT_SECRET. Tokens live at 0600 perms.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const SCOPE: &str = "https://www.googleapis.com/auth/webmasters.readonly";

fn config_path() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(std::path::PathBuf::from(format!("{}/.config/jev-seo/gsc.json", home)))
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenStore {
    refresh_token: String,
}

fn save_refresh(token: &str) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Owner-only file from the first byte: no world-readable window.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        use std::io::Write;
        f.write_all(serde_json::to_string_pretty(&TokenStore { refresh_token: token.to_string() })?.as_bytes())?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&path, serde_json::to_string_pretty(&TokenStore { refresh_token: token.to_string() })?)?;
        Ok(())
    }
}

fn load_refresh() -> Result<String> {
    let raw = std::fs::read_to_string(config_path()?)?;
    Ok(serde_json::from_str::<TokenStore>(&raw)?.refresh_token)
}

fn client_pair() -> Result<(String, String)> {
    Ok((
        std::env::var("GOOGLE_CLIENT_ID").context("GOOGLE_CLIENT_ID not set")?,
        std::env::var("GOOGLE_CLIENT_SECRET").context("GOOGLE_CLIENT_SECRET not set")?,
    ))
}

fn access_token() -> Result<String> {
    let (id, secret) = client_pair()?;
    let refresh = load_refresh()?;
    let body: serde_json::Value = ureq::post("https://oauth2.googleapis.com/token")
        .timeout(Duration::from_secs(15))
        .send_json(serde_json::json!({
            "client_id": id,
            "client_secret": secret,
            "refresh_token": refresh,
            "grant_type": "refresh_token",
        }))
        .context("token refresh failed")?
        .into_json()?;
    body.get("access_token")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .context("no access_token in refresh response")
}

/// Step 1: print the URL + code. Run `gsc auth --code <device_code>` after.
pub fn auth_start() -> Result<String> {
    let (id, _secret) = client_pair()?;
    let body: serde_json::Value = ureq::post("https://oauth2.googleapis.com/device/code")
        .timeout(Duration::from_secs(15))
        .send_json(serde_json::json!({ "client_id": id, "scope": SCOPE }))
        .context("device flow start failed")?
        .into_json()?;
    let url = body.get("verification_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let code = body.get("user_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let device = body.get("device_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    println!("Open:   {}", url);
    println!("Enter:  {}", code);
    println!("Then:   jev-seo gsc auth --code {}", device);
    Ok(device)
}

/// Step 2: poll with the device code until approved, store refresh token.
/// Google answers pending polls as HTTP 400 with an error body, so both
/// the Ok and the 400 paths feed the same error check.
pub fn auth_poll(device_code: &str) -> Result<()> {
    let (id, secret) = client_pair()?;
    for _ in 0..30 {
        let body: serde_json::Value = match ureq::post("https://oauth2.googleapis.com/token")
            .timeout(Duration::from_secs(15))
            .send_json(serde_json::json!({
                "client_id": id,
                "client_secret": secret,
                "device_code": device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
            })) {
            Ok(r) => r.into_json()?,
            Err(ureq::Error::Status(400, r)) => r.into_json().unwrap_or(serde_json::Value::Null),
            Err(e) => anyhow::bail!("token poll failed: {}", e),
        };
        if let Some(refresh) = body.get("refresh_token").and_then(|v| v.as_str()) {
            save_refresh(refresh)?;
            println!("Search Console linked. Token stored with owner-only permissions.");
            return Ok(());
        }
        let err = body.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
        if err != "authorization_pending" && err != "slow_down" {
            anyhow::bail!("auth failed: {}", err);
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    anyhow::bail!("approval timed out, rerun auth")
}

fn api_get(path: &str, token: &str) -> Result<serde_json::Value> {
    Ok(ureq::get(&format!("https://www.googleapis.com{}", path))
        .timeout(Duration::from_secs(15))
        .set("Authorization", &format!("Bearer {}", token))
        .call()?
        .into_json()?)
}

/// Verified sites on this account.
pub fn sites() -> Result<Vec<String>> {
    let token = access_token()?;
    let body = api_get("/webmasters/v3/sites", &token)?;
    Ok(body
        .get("siteEntry")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("siteUrl").and_then(|u| u.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GscRow {
    pub query: String,
    pub clicks: f64,
    pub impressions: f64,
    pub ctr: f64,
    pub position: f64,
}

/// Top queries by clicks, last 28 days, for one verified site.
pub fn top_queries(site: &str, limit: usize) -> Result<Vec<GscRow>> {
    let token = access_token()?;
    let end = chrono_now_days_ago(0);
    let start = chrono_now_days_ago(28);
    let url = format!("/webmasters/v3/sites/{}/searchAnalytics/query", urlencoding(site));
    let body: serde_json::Value = ureq::post(&format!("https://www.googleapis.com{}", url))
        .timeout(Duration::from_secs(20))
        .set("Authorization", &format!("Bearer {}", token))
        .send_json(serde_json::json!({
            "startDate": start,
            "endDate": end,
            "dimensions": ["query"],
            "rowLimit": limit.clamp(1, 100),
            "orderBy": [{ "fieldName": "clicks", "sortOrder": "DESCENDING" }],
        }))
        .context("searchAnalytics query failed")?
        .into_json()?;
    Ok(body
        .get("rows")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .map(|r| GscRow {
                    query: r.get("keys").and_then(|k| k.get(0)).and_then(|q| q.as_str()).unwrap_or("").to_string(),
                    clicks: r.get("clicks").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    impressions: r.get("impressions").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    ctr: r.get("ctr").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    position: r.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
                .collect()
        })
        .unwrap_or_default())
}

pub(crate) fn urlencoding(s: &str) -> String {
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

pub(crate) fn chrono_now_days_ago(days: i64) -> String {
    // Days since epoch without a date crate: 86400s per day.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let target = now.saturating_sub(days as u64 * 86400);
    let days_total = target / 86400;
    // Howard Hinnant's civil-from-days algorithm.
    let z = days_total as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", y + if m <= 2 { 1 } else { 0 }, m, d)
}
