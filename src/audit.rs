use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub file_path: String,
    pub title: Option<String>,
    pub title_len: usize,
    pub description: Option<String>,
    pub description_len: usize,
    pub h1_count: usize,
    pub h2_count: usize,
    pub h3_count: usize,
    pub word_count: usize,
    pub image_count: usize,
    pub images_missing_alt: usize,
    pub internal_links: usize,
    pub external_links: usize,
    pub schema_found: bool,
    pub canonical_found: bool,
    pub og_tags_found: bool,
    pub geo_opening_words: usize,
    pub checks: Vec<CheckItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub fn audit_file(path_str: &str) -> Result<AuditReport> {
    let path = Path::new(path_str);
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path_str))?;

    let is_markdown = path.extension().map_or(false, |ext| {
        ext == "md" || ext == "mdx" || ext == "markdown"
    });

    if is_markdown {
        audit_markdown(path_str, &content)
    } else {
        audit_html(path_str, &content)
    }
}

fn audit_markdown(path_str: &str, content: &str) -> Result<AuditReport> {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);

    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut canonical_found = false;
    let mut schema_found = false;
    let mut og_tags_found = false;

    if let Some(data) = parsed.data {
        if let Ok(val) = data.as_hashmap() {
            if let Some(t) = val.get("title") {
                title = t.as_string().ok();
            }
            if let Some(d) = val.get("description") {
                description = d.as_string().ok();
            }
            if val.contains_key("canonical") || val.contains_key("canonical_url") {
                canonical_found = true;
            }
            if val.contains_key("schema") || val.contains_key("json_ld") {
                schema_found = true;
            }
            if val.contains_key("og_image") || val.contains_key("image") {
                og_tags_found = true;
            }
        }
    }

    let body = parsed.content;
    let mut h1_count = 0;
    let mut h2_count = 0;
    let mut h3_count = 0;
    let mut image_count = 0;
    let mut images_missing_alt = 0;
    let mut internal_links = 0;
    let mut external_links = 0;

    let mut first_section_words = 0;
    let mut past_first_heading = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            h1_count += 1;
            past_first_heading = true;
            if title.is_none() {
                title = Some(trimmed[2..].trim().to_string());
            }
        } else if trimmed.starts_with("## ") {
            h2_count += 1;
            past_first_heading = true;
        } else if trimmed.starts_with("### ") {
            h3_count += 1;
        } else if !past_first_heading || h2_count == 0 {
            first_section_words += trimmed.split_whitespace().count();
        }

        if trimmed.contains("![") {
            image_count += 1;
            if trimmed.contains("![](") {
                images_missing_alt += 1;
            }
        }

        if trimmed.contains("http://") || trimmed.contains("https://") {
            external_links += 1;
        } else if trimmed.contains("](") {
            internal_links += 1;
        }
    }

    let words: Vec<&str> = body.split_whitespace().collect();
    let word_count = words.len();

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let mut checks = Vec::new();
    checks.push(CheckItem {
        name: "Title Tag Length".into(),
        passed: title_len >= 30 && title_len <= 65,
        message: format!("Length: {} chars (Optimal: 40-60 chars)", title_len),
    });

    checks.push(CheckItem {
        name: "Meta Description".into(),
        passed: description_len >= 80 && description_len <= 165,
        message: if description_len == 0 {
            "Missing description frontmatter".into()
        } else {
            format!("Length: {} chars (Optimal: 120-160 chars)", description_len)
        },
    });

    checks.push(CheckItem {
        name: "H1 Uniqueness".into(),
        passed: h1_count == 1,
        message: format!("Found {} H1 headings (Expected: exactly 1)", h1_count),
    });

    checks.push(CheckItem {
        name: "Content Depth".into(),
        passed: word_count >= 300,
        message: format!("Word count: {} (Recommended min: 300 words)", word_count),
    });

    checks.push(CheckItem {
        name: "Image Alt Tags".into(),
        passed: images_missing_alt == 0,
        message: format!("Images: {}, Missing Alt: {}", image_count, images_missing_alt),
    });

    // 2026 GEO Passage Length Check (134-167 words in opening 30%)
    let geo_passed = first_section_words >= 100 && first_section_words <= 200;
    checks.push(CheckItem {
        name: "GEO Citation Density".into(),
        passed: geo_passed,
        message: format!("Opening passage: {} words (Optimal AI citation block: 134-167 words)", first_section_words),
    });

    // Structured Data Check
    checks.push(CheckItem {
        name: "Schema Markup".into(),
        passed: schema_found,
        message: if schema_found { "Structured data present".into() } else { "No JSON-LD/schema frontmatter defined".into() },
    });

    // Canonical Tag Check
    checks.push(CheckItem {
        name: "Canonical Reference".into(),
        passed: canonical_found,
        message: if canonical_found { "Canonical tag configured".into() } else { "Missing canonical URL frontmatter".into() },
    });

    Ok(AuditReport {
        file_path: path_str.to_string(),
        title,
        title_len,
        description,
        description_len,
        h1_count,
        h2_count,
        h3_count,
        word_count,
        image_count,
        images_missing_alt,
        internal_links,
        external_links,
        schema_found,
        canonical_found,
        og_tags_found,
        geo_opening_words: first_section_words,
        checks,
    })
}

fn audit_html(path_str: &str, content: &str) -> Result<AuditReport> {
    let title_re = Regex::new(r#"(?is)<title[^>]*>(.*?)</title>"#)?;
    let meta_desc_re = Regex::new(r#"(?is)<meta[^>]*name=["']description["'][^>]*content=["'](.*?)["']"#)?;
    let h1_re = Regex::new(r#"(?is)<h1[^>]*>.*?</h1>"#)?;
    let h2_re = Regex::new(r#"(?is)<h2[^>]*>.*?</h2>"#)?;
    let h3_re = Regex::new(r#"(?is)<h3[^>]*>.*?</h3>"#)?;
    let img_re = Regex::new(r#"(?is)<img\b([^>]*)>"#)?;
    let alt_re = Regex::new(r#"(?is)alt=["']([^"']+)["']"#)?;
    let a_re = Regex::new(r#"(?is)<a\b[^>]*href=["']([^"']*)["']"#)?;
    let schema_re = Regex::new(r#"(?is)<script[^>]*type=["']application/ld\+json["'][^>]*>.*?</script>"#)?;
    let canonical_re = Regex::new(r#"(?is)<link[^>]*rel=["']canonical["'][^>]*href=["'](.*?)["']"#)?;
    let og_re = Regex::new(r#"(?is)<meta[^>]*property=["']og:(title|description|image)["']"#)?;
    let strip_html = Regex::new(r#"<[^>]+>"#)?;

    let title = title_re.captures(content).map(|c| c[1].trim().to_string());
    let description = meta_desc_re.captures(content).map(|c| c[1].trim().to_string());

    let h1_count = h1_re.find_iter(content).count();
    let h2_count = h2_re.find_iter(content).count();
    let h3_count = h3_re.find_iter(content).count();

    let schema_found = schema_re.is_match(content);
    let canonical_found = canonical_re.is_match(content);
    let og_tags_found = og_re.is_match(content);

    let mut image_count = 0;
    let mut images_missing_alt = 0;
    for cap in img_re.captures_iter(content) {
        image_count += 1;
        let attrs = &cap[1];
        if !alt_re.is_match(attrs) {
            images_missing_alt += 1;
        }
    }

    let mut internal_links = 0;
    let mut external_links = 0;
    for cap in a_re.captures_iter(content) {
        let href = &cap[1];
        if href.starts_with("http://") || href.starts_with("https://") {
            external_links += 1;
        } else if !href.starts_with('#') {
            internal_links += 1;
        }
    }

    let plain_text = strip_html.replace_all(content, " ");
    let words: Vec<&str> = plain_text.split_whitespace().collect();
    let word_count = words.len();

    let first_30_pct_words = (word_count as f64 * 0.30).round() as usize;

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let mut checks = Vec::new();
    checks.push(CheckItem {
        name: "Title Tag Length".into(),
        passed: title_len >= 30 && title_len <= 65,
        message: format!("Length: {} chars", title_len),
    });

    checks.push(CheckItem {
        name: "Meta Description".into(),
        passed: description_len >= 80 && description_len <= 165,
        message: format!("Length: {} chars", description_len),
    });

    checks.push(CheckItem {
        name: "H1 Presence".into(),
        passed: h1_count == 1,
        message: format!("Found {} H1 headings", h1_count),
    });

    checks.push(CheckItem {
        name: "Schema (JSON-LD)".into(),
        passed: schema_found,
        message: if schema_found { "JSON-LD schema markup present".into() } else { "Missing <script type=\"application/ld+json\">".into() },
    });

    checks.push(CheckItem {
        name: "Canonical URL".into(),
        passed: canonical_found,
        message: if canonical_found { "Canonical tag found".into() } else { "Missing <link rel=\"canonical\"> tag".into() },
    });

    checks.push(CheckItem {
        name: "OpenGraph Metadata".into(),
        passed: og_tags_found,
        message: if og_tags_found { "OpenGraph meta tags found".into() } else { "Missing og:title or og:image tags".into() },
    });

    Ok(AuditReport {
        file_path: path_str.to_string(),
        title,
        title_len,
        description,
        description_len,
        h1_count,
        h2_count,
        h3_count,
        word_count,
        image_count,
        images_missing_alt,
        internal_links,
        external_links,
        schema_found,
        canonical_found,
        og_tags_found,
        geo_opening_words: first_30_pct_words,
        checks,
    })
}
