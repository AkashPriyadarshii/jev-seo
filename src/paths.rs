use anyhow::Result;

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
