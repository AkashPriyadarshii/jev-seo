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
    pub heading_skipped_levels: Vec<String>,
    pub em_dash_count: usize,
    pub ai_slop_words_found: Vec<String>,
    pub internal_link_targets: Vec<String>,
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

pub const AI_SLOP_PHRASES: &[&str] = &[
    "delve",
    "leverage",
    "testament",
    "bolster",
    "foster",
    "seamless",
    "seamlessly",
    "crucial",
    "robust",
    "landscape",
    "furthermore",
    "moreover",
    "underscores",
    "unveil",
    "tapestry",
    "beacon",
    "in summary",
    "in today's",
];

pub fn check_heading_hierarchy(skipped: &[String]) -> CheckItem {
    let passed = skipped.is_empty();
    CheckItem {
        name: "Heading Hierarchy".into(),
        passed,
        message: if passed {
            "Logical heading progression (no skipped levels)".into()
        } else {
            format!("Skipped levels detected: [{}]", skipped.join(", "))
        },
    }
}

pub fn check_ai_slop(em_dash_count: usize, slop_words: &[String], word_count: usize) -> CheckItem {
    let em_dash_density = if word_count > 0 {
        (em_dash_count as f64 / word_count as f64) * 500.0
    } else {
        0.0
    };
    let is_excessive_dashes = em_dash_density > 2.0 && em_dash_count >= 2;
    let is_excessive_words = slop_words.len() >= 3;
    let passed = !is_excessive_dashes && !is_excessive_words;

    let message = if passed {
        format!("Natural writing tone ({} em-dashes, {} AI crutches)", em_dash_count, slop_words.len())
    } else {
        format!(
            "Helpful Content risk: {} em-dashes ({:.1}/500w) and {} AI tells [{}]",
            em_dash_count,
            em_dash_density,
            slop_words.len(),
            slop_words.join(", ")
        )
    };

    CheckItem {
        name: "Helpful Content (AI Slop)".into(),
        passed,
        message,
    }
}

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
    let mut internal_link_targets = Vec::new();
    let mut prev_heading_level: Option<usize> = None;
    let mut heading_skipped_levels = Vec::new();

    let mut first_section_words = 0;
    let mut past_first_heading = false;

    let link_re = Regex::new(r#"\[([^\]]*)\]\(([^)]+)\)"#)?;

    for line in body.lines() {
        let trimmed = line.trim();
        let heading_lvl = if let Some(stripped) = trimmed.strip_prefix("# ") {
            h1_count += 1;
            past_first_heading = true;
            if title.is_none() {
                title = Some(stripped.trim().to_string());
            }
            Some(1)
        } else if trimmed.starts_with("## ") {
            h2_count += 1;
            past_first_heading = true;
            Some(2)
        } else if trimmed.starts_with("### ") {
            h3_count += 1;
            Some(3)
        } else if trimmed.starts_with("#### ") {
            Some(4)
        } else if trimmed.starts_with("##### ") {
            Some(5)
        } else if trimmed.starts_with("###### ") {
            Some(6)
        } else {
            None
        };

        if let Some(lvl) = heading_lvl {
            if let Some(prev) = prev_heading_level {
                if lvl > prev + 1 {
                    heading_skipped_levels.push(format!("H{} -> H{}", prev, lvl));
                }
            }
            prev_heading_level = Some(lvl);
        } else if !past_first_heading || h2_count == 0 {
            first_section_words += trimmed.split_whitespace().count();
        }

        if trimmed.contains("![") {
            image_count += 1;
            if trimmed.contains("![](") {
                images_missing_alt += 1;
            }
        }

        for cap in link_re.captures_iter(trimmed) {
            let target = cap[2].trim().split('#').next().unwrap_or("").trim();
            if target.starts_with("http://") || target.starts_with("https://") {
                external_links += 1;
            } else if !target.is_empty() && !target.starts_with('#') && !target.starts_with("mailto:") {
                internal_links += 1;
                internal_link_targets.push(target.to_string());
            }
        }
    }

    let words: Vec<&str> = body.split_whitespace().collect();
    let word_count = words.len();

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let em_dash_count = content.chars().filter(|&c| c == '\u{2014}').count();
    let lower_body = body.to_lowercase();
    let mut ai_slop_words_found = Vec::new();
    for &phrase in AI_SLOP_PHRASES {
        if lower_body.contains(phrase) {
            ai_slop_words_found.push(phrase.to_string());
        }
    }

    let checks = vec![
        check_title_length(title_len),
        check_meta_description(description_len),
        check_h1_uniqueness(h1_count),
        check_heading_hierarchy(&heading_skipped_levels),
        check_content_depth(word_count),
        check_image_alt_tags(image_count, images_missing_alt),
        check_geo_citation_density(first_section_words),
        check_ai_slop(em_dash_count, &ai_slop_words_found, word_count),
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
        heading_skipped_levels,
        em_dash_count,
        ai_slop_words_found,
        internal_link_targets,
        checks,
    })
}

fn audit_html(path_str: &str, content: &str) -> Result<AuditReport> {
    let title_re = Regex::new(r#"(?is)<title[^>]*>(.*?)</title>"#)?;
    let meta_desc_re = Regex::new(r#"(?is)<meta[^>]*name=["']description["'][^>]*content=["'](.*?)["']"#)?;
    let img_re = Regex::new(r#"(?is)<img\b([^>]*)>"#)?;
    let alt_re = Regex::new(r#"(?is)alt=["']([^"']+)["']"#)?;
    let a_re = Regex::new(r#"(?is)<a\b[^>]*href=["']([^"']*)["']"#)?;
    let schema_re = Regex::new(r#"(?is)<script[^>]*type=["']application/ld\+json["'][^>]*>.*?</script>"#)?;
    let canonical_re = Regex::new(r#"(?is)<link[^>]*rel=["']canonical["'][^>]*href=["'](.*?)["']"#)?;
    let og_re = Regex::new(r#"(?is)<meta[^>]*property=["']og:(title|description|image)["']"#)?;
    let strip_html = Regex::new(r#"<[^>]+>"#)?;

    let title = title_re.captures(content).map(|c| c[1].trim().to_string());
    let description = meta_desc_re.captures(content).map(|c| c[1].trim().to_string());

    let h1_count = Regex::new(r#"(?is)<h1\b[^>]*>.*?</h1>"#)?.find_iter(content).count();
    let h2_count = Regex::new(r#"(?is)<h2\b[^>]*>.*?</h2>"#)?.find_iter(content).count();
    let h3_count = Regex::new(r#"(?is)<h3\b[^>]*>.*?</h3>"#)?.find_iter(content).count();

    let heading_seq_re = Regex::new(r#"(?is)<(h[1-6])\b[^>]*>"#)?;
    let mut prev_heading_level: Option<usize> = None;
    let mut heading_skipped_levels = Vec::new();
    for cap in heading_seq_re.captures_iter(content) {
        let tag = &cap[1];
        if let Some(lvl_char) = tag.chars().nth(1) {
            if let Some(lvl) = lvl_char.to_digit(10).map(|d| d as usize) {
                if let Some(prev) = prev_heading_level {
                    if lvl > prev + 1 {
                        heading_skipped_levels.push(format!("H{} -> H{}", prev, lvl));
                    }
                }
                prev_heading_level = Some(lvl);
            }
        }
    }

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
    let mut internal_link_targets = Vec::new();
    for cap in a_re.captures_iter(content) {
        let href = cap[1].trim();
        let target = href.split('#').next().unwrap_or("").trim();
        if target.starts_with("http://") || target.starts_with("https://") {
            external_links += 1;
        } else if !target.is_empty() && !target.starts_with('#') && !target.starts_with("mailto:") {
            internal_links += 1;
            internal_link_targets.push(target.to_string());
        }
    }

    let plain_text = strip_html.replace_all(content, " ");
    let words: Vec<&str> = plain_text.split_whitespace().collect();
    let word_count = words.len();

    let first_30_pct_words = (word_count as f64 * 0.30).round() as usize;

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let em_dash_count = content.chars().filter(|&c| c == '\u{2014}').count();
    let lower_content = plain_text.to_lowercase();
    let mut ai_slop_words_found = Vec::new();
    for &phrase in AI_SLOP_PHRASES {
        if lower_content.contains(phrase) {
            ai_slop_words_found.push(phrase.to_string());
        }
    }

    let checks = vec![
        check_title_length(title_len),
        check_meta_description(description_len),
        check_h1_uniqueness(h1_count),
        check_heading_hierarchy(&heading_skipped_levels),
        check_content_depth(word_count),
        check_image_alt_tags(image_count, images_missing_alt),
        check_geo_citation_density(first_30_pct_words),
        check_ai_slop(em_dash_count, &ai_slop_words_found, word_count),
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
        heading_skipped_levels,
        em_dash_count,
        ai_slop_words_found,
        internal_link_targets,
        checks,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannibalizationItem {
    pub keyword_stem: String,
    pub colliding_files: Vec<String>,
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
    pub orphan_pages: Vec<String>,
    pub keyword_cannibalization: Vec<CannibalizationItem>,
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
            orphan_pages: Vec::new(),
            keyword_cannibalization: Vec::new(),
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

    // Internal Link Graph & Orphan Page Detection
    let mut inbound_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for rep in &reports {
        let rep_norm = rep.file_path.replace('\\', "/");
        let rep_stem = Path::new(&rep.file_path).file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

        for raw_target in &rep.internal_link_targets {
            let clean = raw_target.trim().trim_start_matches("./").replace('\\', "/");
            let clean_stem = clean.split('/').next_back().unwrap_or("").to_string();

            for f in &files {
                let f_norm = f.to_string_lossy().replace('\\', "/");
                let f_name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let f_stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                if (f_norm.ends_with(&clean) || f_name == clean_stem || f_stem == clean_stem)
                    && f_norm != rep_norm
                    && f_name != rep_stem
                {
                    *inbound_map.entry(f_norm.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut orphan_pages = Vec::new();
    if files.len() > 1 {
        for f in &files {
            let f_norm = f.to_string_lossy().replace('\\', "/");
            let f_name = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            // Skip root index / readme as landing entrypoint
            if f_name.starts_with("readme") || f_name.starts_with("index") {
                continue;
            }
            if inbound_map.get(&f_norm).copied().unwrap_or(0) == 0 {
                orphan_pages.push(f.to_string_lossy().to_string());
            }
        }
    }

    // Keyword Cannibalization Radar
    let stop_words: std::collections::HashSet<&str> = [
        "a", "an", "the", "in", "on", "at", "for", "to", "of", "and", "or", "with", "by", "is", "vs", "how"
    ].into_iter().collect();

    let mut stem_to_files: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for rep in &reports {
        if let Some(title) = &rep.title {
            let words: Vec<String> = title
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty() && !stop_words.contains(w) && w.len() > 2)
                .map(|s| s.to_string())
                .collect();

            if words.len() >= 2 {
                let stem = words[..words.len().min(3)].join(" ");
                stem_to_files.entry(stem).or_default().push(rep.file_path.clone());
            }
        }
    }

    let keyword_cannibalization: Vec<CannibalizationItem> = stem_to_files
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(keyword_stem, colliding_files)| CannibalizationItem { keyword_stem, colliding_files })
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
        orphan_pages,
        keyword_cannibalization,
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
