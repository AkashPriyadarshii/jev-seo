use anyhow::Result;
use std::net::ToSocketAddrs;

/// Read a user-named file with guard rails shared by CLI and MCP tools.
/// Non-path input (no separators, not an existing path) is returned as-is so
/// inline snippets keep working. URLs are never fetched here.
pub fn read_user_file(target: &str, allowed: &[&str]) -> Result<String> {
    let trimmed = target.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        anyhow::bail!("pass a local file path or an inline snippet, not a URL");
    }
    let path = std::path::Path::new(trimmed);
    // Inline snippets (JSON-LD, HTML) are content, never paths.
    if trimmed.starts_with('{') || trimmed.starts_with('<') || trimmed.contains('\n') {
        return Ok(trimmed.to_string());
    }
    let looks_like_path = path.exists()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || (trimmed.starts_with('.') && trimmed.len() > 1);
    if !looks_like_path {
        return Ok(trimmed.to_string());
    }
    if !path.exists() {
        anyhow::bail!("file not found: {}", trimmed);
    }
    if !path.is_file() {
        anyhow::bail!("not a file: {}", trimmed);
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            anyhow::bail!("refusing dot-file: {}", trimmed);
        }
    }
    let ext_ok = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| allowed.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false);
    if !ext_ok {
        anyhow::bail!("refusing non-content file: {}", trimmed);
    }
    Ok(std::fs::read_to_string(path)?)
}

/// True when a SERP result URL belongs to the tracked domain (exact host or
/// subdomain), so evilcrates.io never counts for crates.io.
pub fn url_matches_domain(result_url: &str, domain: &str) -> bool {
    let domain = domain.trim().trim_start_matches("https://").trim_start_matches("http://");
    let domain = domain.split('/').next().unwrap_or(domain);
    let host = url::Url::parse(result_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()));
    match host {
        Some(h) => h == domain.to_ascii_lowercase() || h.ends_with(&format!(".{}", domain.to_ascii_lowercase())),
        None => false,
    }
}

/// Refuse non-public fetch targets. Blocks SSRF to loopback, LAN, link-local
/// (cloud metadata), and non-http schemes.
pub fn reject_private_url(raw: &str) -> anyhow::Result<url::Url> {
    let url = url::Url::parse(raw).map_err(|_| anyhow::anyhow!("not a valid URL: {}", raw))?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("refusing non-http URL: {}", raw);
    }
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    if host == "localhost" || host == "::1" || host.ends_with(".localhost") {
        anyhow::bail!("refusing private fetch target: {}", host);
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_private_ip(ip) {
            anyhow::bail!("refusing private fetch target: {}", host);
        }
        return Ok(url);
    }
    // Hostname: resolve and check every addr (blocks DNS rebinding at request time).
    let port = url.port_or_known_default().unwrap_or(443);
    match (host.as_str(), port).to_socket_addrs() {
        Ok(addrs) => {
            for addr in addrs {
                if is_private_ip(addr.ip()) {
                    anyhow::bail!("refusing private fetch target: {} resolves privately", host);
                }
            }
            Ok(url)
        }
        Err(_) => Ok(url),
    }
}

fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified()
        }
        std::net::IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

/// Re-check the final URL after fetches that follow redirects.
pub fn reject_redirect_target(final_url: &str) -> anyhow::Result<()> {
    reject_private_url(final_url).map(|_| ())
}
