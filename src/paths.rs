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
    // Canonicalize: resolves symlinks, `..`, and relative prefixes so the
    // checks below run on the real location, not the spelled one.
    let canon = path
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("file not found: {}", trimmed))?;
    if !canon.is_file() {
        anyhow::bail!("not a file: {}", trimmed);
    }
    // Deny dot-files AND dot-directories anywhere in the real path, so
    // `~/.config/x.json` or `sub/.git/hooks/x.html` cannot pass on extension.
    // Scratch files directly under the system temp root are exempt.
    if has_dot_component(&canon) {
        anyhow::bail!("refusing dot-file or dot-directory: {}", trimmed);
    }
    // Deny system trees even when reached without dots (symlink, /etc/hosts).
    // Deny system trees even when reached without dots (symlink, /etc/hosts).
    let canon_str = canon.to_string_lossy().to_ascii_lowercase();
    for prefix in ["/etc/", "/proc/", "/sys/", "/dev/"] {
        if canon_str == prefix.trim_end_matches('/') || canon_str.starts_with(prefix) {
            anyhow::bail!("refusing system path: {}", trimmed);
        }
    }
    if let Some(name) = canon.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            anyhow::bail!("refusing dot-file: {}", trimmed);
        }
    }
    let ext_ok = canon
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| allowed.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false);
    if !ext_ok {
        anyhow::bail!("refusing non-content file: {}", trimmed);
    }
    Ok(std::fs::read_to_string(&canon)?)
}

/// Guard for directory-scoped tools (audit path). Same canonicalization and
/// denials as read_user_file, minus the file-only and extension checks, so
/// agents cannot point audits at home, dot-dirs, or system trees.
pub fn check_audit_path(target: &str) -> Result<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty path");
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        anyhow::bail!("pass a local directory path, not a URL");
    }
    let path = std::path::Path::new(trimmed);
    if !path.exists() {
        anyhow::bail!("path not found: {}", trimmed);
    }
    let canon = path
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("path not found: {}", trimmed))?;
    if has_dot_component(&canon) {
        anyhow::bail!("refusing dot-file or dot-directory: {}", trimmed);
    }
    let canon_str = canon.to_string_lossy().to_ascii_lowercase();
    for prefix in ["/etc/", "/proc/", "/sys/", "/dev/", "/root/"] {
        if canon_str == prefix.trim_end_matches('/') || canon_str.starts_with(prefix) {
            anyhow::bail!("refusing system path: {}", trimmed);
        }
    }
    Ok(canon.to_string_lossy().into_owned())
}

/// True when any path component starts with a dot. The system temp root is
/// special: session leaves (tempdir() nests dot-dirs there on some platforms)
/// are exempt, but anything nested below the leaf is still checked, so
/// `<tmp>/.session/.git/x` stays denied while staged scratch files pass.
fn has_dot_component(canon: &std::path::Path) -> bool {
    use std::path::Component;
    let dotted = |c: &Component| {
        c.as_os_str()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    };
    match std::env::temp_dir().canonicalize() {
        Ok(root) => match canon.strip_prefix(&root) {
            // Skip the session leaf itself, check the rest.
            Ok(rel) => rel.components().skip(1).any(|c| dotted(&c)),
            Err(_) => canon.components().any(|c| dotted(&c)),
        },
        Err(_) => canon.components().any(|c| dotted(&c)),
    }
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
    let host = host.trim_end_matches('.');
    if host == "localhost" || host == "::1" || host.ends_with(".localhost") {
        anyhow::bail!("refusing private fetch target: {}", host);
    }
    if let Some(ip) = normalize_host_ip(host) {
        if is_private_ip(ip) {
            anyhow::bail!("refusing private fetch target: {}", host);
        }
        return Ok(url);
    }
    // Hostname: resolve and check every addr (blocks DNS rebinding at request time).
    // Note: std DNS has no timeout knob; callers already bound total time
    // with per-request timeouts, so a slow resolver stalls one call, not the run.
    let port = url.port_or_known_default().unwrap_or(443);
    match (host, port).to_socket_addrs() {
        Ok(addrs) => {
            for addr in addrs {
                if is_private_ip(addr.ip()) {
                    anyhow::bail!("refusing private fetch target: {} resolves privately", host);
                }
            }
            Ok(url)
        }
        // Fail closed: an unverified host is not fetchable. A DNS failure for
        // the checker can still resolve for a third-party fetcher, so passing
        // here would punch a hole through Jina/Firecrawl delegation.
        Err(_) => anyhow::bail!("refusing unverified fetch target (DNS failed): {}", host),
    }
}

/// Parse dotted, decimal, octal, and hex IPv4 forms (`2130706433`,
/// `0x7f.0.0.1`, `0177.0.0.1`) the way resolvers do, so non-dotted literals
/// cannot slip past the private-IP check.
fn normalize_host_ip(host: &str) -> Option<std::net::IpAddr> {
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return Some(ip);
    }
    fn part(s: &str) -> Option<u32> {
        if s.len() > 2 && (s.starts_with("0x") || s.starts_with("0X")) {
            u32::from_str_radix(&s[2..], 16).ok()
        } else if s.len() > 1 && s.starts_with('0') {
            u32::from_str_radix(&s[1..], 8).ok()
        } else {
            s.parse::<u32>().ok()
        }
    }
    let parts: Vec<Option<u32>> = host.split('.').map(part).collect();
    if parts.iter().any(|p| p.is_none()) {
        return None;
    }
    let p: Vec<u32> = parts.into_iter().flatten().collect();
    let n: u32 = match p.len() {
        1 => p[0],
        2 if p[0] <= 0xff && p[1] <= 0xff_ffff => (p[0] << 24) | p[1],
        3 if p[0] <= 0xff && p[1] <= 0xff && p[2] <= 0xffff => (p[0] << 24) | (p[1] << 16) | p[2],
        4 if p.iter().all(|x| *x <= 0xff) => (p[0] << 24) | (p[1] << 16) | (p[2] << 8) | p[3],
        _ => return None,
    };
    Some(std::net::IpAddr::V4(std::net::Ipv4Addr::from(n)))
}

fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || (o[0] == 100 && o[1] >= 64 && o[1] < 128) // CGNAT 100.64/10
                || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // benchmark 198.18/15
        }
        std::net::IpAddr::V6(v6) => {
            let s = v6.segments();
            // IPv4-mapped IPv6 (::ffff:127.0.0.1) must face the IPv4 checks.
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_private_ip(std::net::IpAddr::V4(mapped));
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation 2001:db8::/32
                || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (s[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        }
    }
}

/// Validate an operator-supplied API endpoint override. Fail closed: a
/// misconfigured override that points at private space must error loudly,
/// never silently fall back to a different host.
pub fn reject_api_endpoint(raw: &str, name: &str) -> anyhow::Result<String> {
    reject_private_url(raw)
        .map(|u| u.to_string())
        .map_err(|e| anyhow::anyhow!("{} override refused: {}", name, e))
}

/// Re-check the final URL after fetches that follow redirects.
pub fn reject_redirect_target(final_url: &str) -> anyhow::Result<()> {
    reject_private_url(final_url).map(|_| ())
}
