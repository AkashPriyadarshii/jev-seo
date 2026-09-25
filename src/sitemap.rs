use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapReport {
    pub target: String,
    pub is_valid: bool,
    pub total_urls: usize,
    pub https_urls: usize,
    pub insecure_http_urls: usize,
    pub urls_with_params: usize,
    pub urls_with_lastmod: usize,
    pub hreflang_count: usize,
    pub invalid_hreflang_codes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub sample_urls: Vec<String>,
}

pub const MAX_SITEMAP_URLS: usize = 50_000;

pub fn audit_sitemap(target: &str) -> Result<SitemapReport> {
    let xml_content = if target.starts_with("http://") || target.starts_with("https://") {
        crate::paths::reject_private_url(target)?;
        let resp = crate::fetch::with_extra_headers(
            ureq::get(target)
                .timeout(Duration::from_secs(10))
                .set("User-Agent", concat!("jev-seo/", env!("CARGO_PKG_VERSION"), " (TypeSafe Jev XML Sitemap Inspector)")),
        )
        .call()
        .with_context(|| format!("Failed to fetch remote sitemap: {}", target))?;
        crate::paths::reject_redirect_target(resp.get_url())?;
        crate::fetch::capped_string(resp, crate::fetch::MAX_AUX_BYTES)
            .with_context(|| format!("Failed to read sitemap body from {}", target))?
    } else {
        crate::paths::read_user_file(target, &["xml"])
            .with_context(|| format!("Failed to read local sitemap file: {}", target))?
    };

    parse_sitemap_xml(target, &xml_content)
}

pub fn parse_sitemap_xml(target: &str, xml: &str) -> Result<SitemapReport> {
    let loc_re = Regex::new(r#"(?is)<loc>(.*?)</loc>"#)?;
    let lastmod_re = Regex::new(r#"(?is)<lastmod>(.*?)</lastmod>"#)?;
    let hreflang_re = Regex::new(r#"(?is)<xhtml:link\b[^>]*hreflang=["']([^"']+)["'][^>]*>"#)?;

    let mut total_urls = 0;
    let mut https_urls = 0;
    let mut insecure_http_urls = 0;
    let mut urls_with_params = 0;
    let mut sample_urls = Vec::new();

    for cap in loc_re.captures_iter(xml) {
        total_urls += 1;
        let url = cap[1].trim();

        if total_urls <= 5 {
            sample_urls.push(url.to_string());
        }

        if url.starts_with("https://") {
            https_urls += 1;
        } else if url.starts_with("http://") {
            insecure_http_urls += 1;
        }

        if url.contains('?') {
            urls_with_params += 1;
        }
    }

    let urls_with_lastmod = lastmod_re.find_iter(xml).count();

    let mut hreflang_count = 0;
    let mut invalid_hreflang_codes = Vec::new();
    let mut has_x_default = false;

    for cap in hreflang_re.captures_iter(xml) {
        hreflang_count += 1;
        let code = cap[1].trim();
        if code.eq_ignore_ascii_case("x-default") {
            has_x_default = true;
        } else if code.eq_ignore_ascii_case("en-uk") {
            invalid_hreflang_codes.push("en-UK (invalid: use en-GB instead)".into());
        } else if !is_valid_hreflang(code) {
            invalid_hreflang_codes.push(format!("{} (malformed language/region code)", code));
        }
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if total_urls == 0 {
        errors.push("No <loc> URLs found in sitemap XML.".into());
    }

    if total_urls > MAX_SITEMAP_URLS {
        errors.push(format!(
            "Sitemap contains {} URLs, exceeding Google's maximum of 50,000 URLs per file.",
            total_urls
        ));
    }

    if insecure_http_urls > 0 {
        warnings.push(format!(
            "Found {} Insecure HTTP URLs. Google requires all sitemap URLs to be canonical HTTPS.",
            insecure_http_urls
        ));
    }

    if urls_with_params > 0 {
        warnings.push(format!(
            "Found {} URLs with Query Parameters. Sitemaps should only reference clean canonical URLs.",
            urls_with_params
        ));
    }

    if hreflang_count > 0 && !has_x_default {
        warnings.push("Multilingual hreflang annotations found but missing required 'x-default' fallback entry.".into());
    }

    if !invalid_hreflang_codes.is_empty() {
        warnings.push(format!(
            "Invalid hreflang codes detected: [{}]",
            invalid_hreflang_codes.join(", ")
        ));
    }

    let is_valid = errors.is_empty() && total_urls > 0;

    Ok(SitemapReport {
        target: target.to_string(),
        is_valid,
        total_urls,
        https_urls,
        insecure_http_urls,
        urls_with_params,
        urls_with_lastmod,
        hreflang_count,
        invalid_hreflang_codes,
        warnings,
        errors,
        sample_urls,
    })
}

fn is_valid_hreflang(code: &str) -> bool {
    let parts: Vec<&str> = code.split('-').collect();
    if parts.is_empty() || parts.len() > 2 {
        return false;
    }

    let lang = parts[0];
    if lang.len() != 2 && lang.len() != 3 {
        return false;
    }

    if parts.len() == 2 {
        let region = parts[1];
        if region.len() != 2 && region.len() != 3 {
            return false;
        }
    }

    true
}
