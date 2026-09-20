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

pub const MIN_TITLE_CHARS: usize = 30;
pub const MAX_TITLE_CHARS: usize = 65;
pub const MIN_DESC_CHARS: usize = 80;
pub const MAX_DESC_CHARS: usize = 165;
pub const MIN_CONTENT_WORDS: usize = 300;
pub const GEO_MIN_WORDS: usize = 100;
pub const GEO_MAX_WORDS: usize = 200;

pub fn check_title_length(title_len: usize) -> CheckItem {
    let passed = (MIN_TITLE_CHARS..=MAX_TITLE_CHARS).contains(&title_len);
    CheckItem {
        name: "Title Tag Length".into(),
        passed,
        message: format!("Length: {} chars (Optimal: {}-{} chars)", title_len, MIN_TITLE_CHARS, MAX_TITLE_CHARS),
    }
}

pub fn check_meta_description(description_len: usize) -> CheckItem {
    let passed = (MIN_DESC_CHARS..=MAX_DESC_CHARS).contains(&description_len);
    CheckItem {
        name: "Meta Description".into(),
        passed,
        message: if description_len == 0 {
            "Missing description metadata".into()
        } else {
            format!("Length: {} chars (Optimal: {}-{} chars)", description_len, MIN_DESC_CHARS, MAX_DESC_CHARS)
        },
    }
}

pub fn check_h1_uniqueness(h1_count: usize) -> CheckItem {
    CheckItem {
        name: "H1 Uniqueness".into(),
        passed: h1_count == 1,
        message: format!("Found {} H1 headings (Expected: exactly 1)", h1_count),
    }
}

pub fn check_content_depth(word_count: usize) -> CheckItem {
    CheckItem {
        name: "Content Depth".into(),
        passed: word_count >= MIN_CONTENT_WORDS,
        message: format!("Word count: {} (Recommended min: {} words)", word_count, MIN_CONTENT_WORDS),
    }
}

pub fn check_image_alt_tags(image_count: usize, images_missing_alt: usize) -> CheckItem {
    CheckItem {
        name: "Image Alt Tags".into(),
        passed: images_missing_alt == 0,
        message: format!("Images: {}, Missing Alt: {}", image_count, images_missing_alt),
    }
}

pub fn check_geo_citation_density(words: usize) -> CheckItem {
    let passed = (GEO_MIN_WORDS..=GEO_MAX_WORDS).contains(&words);
    CheckItem {
        name: "GEO Citation Density".into(),
        passed,
        message: format!("Opening passage: {} words (Optimal AI citation block: 134-167 words)", words),
    }
}

pub fn check_schema_markup(schema_found: bool) -> CheckItem {
    CheckItem {
        name: "Schema Markup".into(),
        passed: schema_found,
        message: if schema_found { "Structured data present".into() } else { "No JSON-LD/schema markup defined".into() },
    }
}

pub fn check_canonical_reference(canonical_found: bool) -> CheckItem {
    CheckItem {
        name: "Canonical Reference".into(),
        passed: canonical_found,
        message: if canonical_found { "Canonical tag configured".into() } else { "Missing canonical URL definition".into() },
    }
}

pub fn check_opengraph_metadata(og_found: bool) -> CheckItem {
    CheckItem {
        name: "OpenGraph Metadata".into(),
        passed: og_found,
        message: if og_found { "OpenGraph meta tags found".into() } else { "Missing og:title, og:description, or og:image tags".into() },
    }
}

pub fn audit_file(path_str: &str) -> Result<AuditReport> {
    let path = Path::new(path_str);
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path_str))?;

    let is_markdown = path.extension().is_some_and(|ext| {
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
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            h1_count += 1;
            past_first_heading = true;
            if title.is_none() {
                title = Some(stripped.trim().to_string());
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

    let checks = vec![
        check_title_length(title_len),
        check_meta_description(description_len),
        check_h1_uniqueness(h1_count),
        check_content_depth(word_count),
        check_image_alt_tags(image_count, images_missing_alt),
        check_geo_citation_density(first_section_words),
        check_schema_markup(schema_found),
        check_canonical_reference(canonical_found),
    ];

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

    let checks = vec![
        check_title_length(title_len),
        check_meta_description(description_len),
        check_h1_uniqueness(h1_count),
        check_content_depth(word_count),
        check_image_alt_tags(image_count, images_missing_alt),
        check_geo_citation_density(first_30_pct_words),
        check_schema_markup(schema_found),
        check_canonical_reference(canonical_found),
        check_opengraph_metadata(og_tags_found),
    ];

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryAuditReport {
    pub dir_path: String,
    pub total_files: usize,
    pub total_words: usize,
    pub avg_words_per_file: usize,
    pub pass_rate: f64,
    pub reports: Vec<AuditReport>,
    pub duplicate_titles: std::collections::HashMap<String, Vec<String>>,
    pub thin_pages: Vec<(String, usize)>,
    pub missing_canonicals: Vec<String>,
    pub missing_descriptions: Vec<String>,
}

pub fn audit_path(path_str: &str) -> Result<DirectoryAuditReport> {
    let path = Path::new(path_str);
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path_str);
    }

    if path.is_file() {
        let single = audit_file(path_str)?;
        let total_words = single.word_count;
        let mut missing_canonicals = Vec::new();
        if !single.canonical_found {
            missing_canonicals.push(path_str.to_string());
        }
        let mut missing_descriptions = Vec::new();
        if single.description.is_none() {
            missing_descriptions.push(path_str.to_string());
        }
        let mut thin_pages = Vec::new();
        if single.word_count < MIN_CONTENT_WORDS {
            thin_pages.push((path_str.to_string(), single.word_count));
        }
        let passed = single.checks.iter().filter(|c| c.passed).count();
        let pass_rate = if !single.checks.is_empty() {
            (passed as f64 / single.checks.len() as f64) * 100.0
        } else {
            100.0
        };

        return Ok(DirectoryAuditReport {
            dir_path: path_str.to_string(),
            total_files: 1,
            total_words,
            avg_words_per_file: total_words,
            pass_rate,
            reports: vec![single],
            duplicate_titles: std::collections::HashMap::new(),
            thin_pages,
            missing_canonicals,
            missing_descriptions,
        });
    }

    let mut files = Vec::new();
    collect_audit_files(path, &mut files)?;
    files.sort();

    if files.is_empty() {
        anyhow::bail!("No markdown (.md/.mdx) or HTML files found in directory: {}", path_str);
    }

    let mut reports = Vec::new();
    let mut total_words = 0;
    let mut total_checks = 0;
    let mut total_passed = 0;
    let mut title_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut thin_pages = Vec::new();
    let mut missing_canonicals = Vec::new();
    let mut missing_descriptions = Vec::new();

    for f in &files {
        let p_str = f.to_string_lossy().to_string();
        if let Ok(rep) = audit_file(&p_str) {
            total_words += rep.word_count;
            for c in &rep.checks {
                total_checks += 1;
                if c.passed {
                    total_passed += 1;
                }
            }

            if let Some(t) = &rep.title {
                let clean_t = t.trim().to_string();
                if !clean_t.is_empty() {
                    title_map.entry(clean_t).or_default().push(p_str.clone());
                }
            }

            if rep.word_count < MIN_CONTENT_WORDS {
                thin_pages.push((p_str.clone(), rep.word_count));
            }
            if !rep.canonical_found {
                missing_canonicals.push(p_str.clone());
            }
            if rep.description.is_none() {
                missing_descriptions.push(p_str.clone());
            }

            reports.push(rep);
        }
    }

    let duplicate_titles: std::collections::HashMap<String, Vec<String>> = title_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    let total_files = reports.len();
    let avg_words_per_file = total_words.checked_div(total_files).unwrap_or(0);
    let pass_rate = if total_checks > 0 {
        (total_passed as f64 / total_checks as f64) * 100.0
    } else {
        100.0
    };

    Ok(DirectoryAuditReport {
        dir_path: path_str.to_string(),
        total_files,
        total_words,
        avg_words_per_file,
        pass_rate,
        reports,
        duplicate_titles,
        thin_pages,
        missing_canonicals,
        missing_descriptions,
    })
}

fn collect_audit_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" || file_name == "dist" || file_name == "build" {
                continue;
            }
            if path.is_dir() {
                collect_audit_files(&path, files)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "md" || ext == "mdx" || ext == "markdown" || ext == "html" || ext == "htm" {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}
