use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;

static SLOP_RE: LazyLock<Regex> = LazyLock::new(|| {
    let mut alts: Vec<String> = AI_SLOP_PHRASES.iter().map(|p| regex::escape(p)).collect();
    alts.sort_by_key(|a| std::cmp::Reverse(a.len()));
    Regex::new(&format!("(?i)\\b(?:{})\\b", alts.join("|"))).expect("slop regex")
});
static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<(script|style|head)\b[^>]*>.*?</(script|style|head)\s*>").expect("tag regex"));
static MD_HTML_H_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)<h([1-6])\b[^>]*>"#).expect("md html heading regex"));
static MD_FENCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```.*?```").expect("fence regex"));
static MD_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]+\)|\[([^\]]*)\]\([^)]+\)").expect("md link regex"));

/// Attribute value from a single HTML tag, any attribute order.
fn tag_attr(tag: &str, attr: &str) -> Option<String> {
    let pat = format!(r#"(?i)\b{}\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)"#, regex::escape(attr));
    let re = Regex::new(&pat).ok()?;
    re.captures(tag).and_then(|c| {
        c.get(1).map(|m| {
            m.as_str()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .to_string()
        })
    })
}

/// Distinct AI-slop markers with word boundaries: "robust" no longer fires
/// inside "robustness", but multi-word tells like "in summary" still match.
fn slop_hits(text: &str) -> Vec<String> {
    let mut found: Vec<String> = SLOP_RE
        .find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    found.sort();
    found
}

/// Markdown body stripped of fences, code spans, and link markup for honest
/// word counts: syntax characters are not prose.
pub fn md_plain_text(body: &str) -> String {
    let no_fence = MD_FENCE_RE.replace_all(body, " ");
    let no_links = MD_LINK_RE.replace_all(&no_fence, "$1$2");
    let no_code: String = no_links
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, s)| s)
        .collect::<Vec<_>>()
        .join(" ");
    no_code
        .split_whitespace()
        .filter(|w| {
            let t = w.trim_matches(|c: char| c == '#' || c == '*' || c == '_' || c == '-' || c == '>' || c == '|');
            !t.is_empty()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn md_prose_words(body: &str) -> usize {
    md_plain_text(body).split_whitespace().count()
}

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
    /// False only when an ld+json block failed to parse. Defaults true so
    /// reports saved before validation existed do not newly fail R33.
    #[serde(default = "schema_valid_default")]
    pub schema_json_valid: bool,
    pub checks: Vec<CheckItem>,
}

fn schema_valid_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

/// Display windows match the R10/R12 rule windows exactly: a page never
/// passes the badge while firing the rule, or vice versa.
pub const MIN_TITLE_CHARS: usize = 30;
pub const MAX_TITLE_CHARS: usize = 60;
pub const MIN_DESC_CHARS: usize = 120;
pub const MAX_DESC_CHARS: usize = 160;
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
    // Inline HTML in markdown bodies counts: layout tags the author wrote
    // by hand satisfy canonical/schema/OG the same as frontmatter keys.
    let mut schema_json_valid = true;
    {
        let link_tag_re = Regex::new(r#"(?is)<link\b[^>]*>"#)?;
        for cap in link_tag_re.captures_iter(&body) {
            if tag_attr(&cap[0], "rel").is_some_and(|v| v.eq_ignore_ascii_case("canonical")) {
                canonical_found = true;
            }
        }
        let script_re = Regex::new(r#"(?is)<script\b[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#)?;
        for cap in script_re.captures_iter(&body) {
            schema_found = true;
            if serde_json::from_str::<serde_json::Value>(&cap[1]).is_err() {
                schema_json_valid = false;
            }
        }
        let meta_tag_re = Regex::new(r#"(?is)<meta\b[^>]*>"#)?;
        let mut og_seen = std::collections::BTreeSet::new();
        for cap in meta_tag_re.captures_iter(&body) {
            if let Some(prop) = tag_attr(&cap[0], "property") {
                let p = prop.to_ascii_lowercase();
                if p == "og:title" || p == "og:description" || p == "og:image" {
                    og_seen.insert(p);
                }
            }
        }
        if og_seen.len() == 3 {
            og_tags_found = true;
        }
    }
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
        // Inline HTML headings count too: docs often mix <h2> into markdown.
        let mut html_heading: Option<usize> = None;
        if !trimmed.starts_with('#') {
            if let Some(cap) = MD_HTML_H_RE.captures(trimmed) {
                if let Some(lvl) = cap[1].parse::<usize>().ok() {
                    html_heading = Some(lvl);
                    match lvl {
                        1 => h1_count += 1,
                        2 => h2_count += 1,
                        3 => h3_count += 1,
                        _ => {}
                    }
                    past_first_heading = true;
                }
            }
        }
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
            html_heading
        };

        if let Some(lvl) = heading_lvl {
            if let Some(prev) = prev_heading_level {
                if lvl > prev + 1 {
                    heading_skipped_levels.push(format!("H{} -> H{}", prev, lvl));
                }
            } else if lvl > 2 {
                // Leading jump with no prior heading: H3-first skips H1+H2.
                // H2-first is covered by the missing-H1 check instead.
                heading_skipped_levels.push(format!("H1 -> H{}", lvl));
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

    let word_count = md_prose_words(&body);

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let em_dash_count = content.chars().filter(|&c| c == '\u{2014}').count();
    let ai_slop_words_found = slop_hits(&body);

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
        schema_json_valid,
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
    let tag_re = Regex::new(r#"(?is)<(meta|link)\b[^>]*>"#)?;
    let img_re = Regex::new(r#"(?is)<img\b([^>]*)>"#)?;
    let alt_attr_re = Regex::new(r#"(?is)\balt\s*="#)?;
    let a_re = Regex::new(r#"(?is)<a\b[^>]*href=["']([^"']*)["']"#)?;
    let schema_block_re = Regex::new(r#"(?is)<script\b[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#)?;
    let strip_html = Regex::new(r#"<[^>]+>"#)?;

    let title = title_re.captures(content).map(|c| c[1].trim().to_string());
    // Attribute order must not matter: find the tag by one attribute,
    // read the value from the other.
    let mut description: Option<String> = None;
    let mut canonical_found = false;
    let mut og_seen = std::collections::BTreeSet::new();
    for cap in tag_re.captures_iter(content) {
        let tag = &cap[0];
        let is_meta = cap[1].eq_ignore_ascii_case("meta");
        if is_meta {
            if let Some(name) = tag_attr(tag, "name") {
                if name.eq_ignore_ascii_case("description") && description.is_none() {
                    description = tag_attr(tag, "content").map(|s| s.trim().to_string());
                }
            }
            if let Some(prop) = tag_attr(tag, "property") {
                let p = prop.to_ascii_lowercase();
                if p == "og:title" || p == "og:description" || p == "og:image" {
                    og_seen.insert(p);
                }
            }
        } else if tag_attr(tag, "rel").is_some_and(|v| v.eq_ignore_ascii_case("canonical")) {
            canonical_found = true;
        }
    }
    // All three OG tags required: one of three passing hid partial markup.
    let og_tags_found = og_seen.len() == 3;

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
                } else if lvl > 2 {
                    heading_skipped_levels.push(format!("H1 -> H{}", lvl));
                }
                prev_heading_level = Some(lvl);
            }
        }
    }

    let mut schema_found = false;
    let mut schema_json_valid = true;
    for cap in schema_block_re.captures_iter(content) {
        schema_found = true;
        if serde_json::from_str::<serde_json::Value>(&cap[1]).is_err() {
            schema_json_valid = false;
        }
    }

    let mut image_count = 0;
    let mut images_missing_alt = 0;
    for cap in img_re.captures_iter(content) {
        image_count += 1;
        // Missing means no alt attribute at all. Empty alt is valid:
        // it marks decorative images screen readers should skip.
        if !alt_attr_re.is_match(&cap[1]) {
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

    // Word count runs on body copy only: head, scripts, and styles are not
    // prose, and counting them inflated depth passes.
    let copy = TAG_RE.replace_all(content, " ");
    let plain_text = strip_html.replace_all(&copy, " ");
    let words: Vec<&str> = plain_text.split_whitespace().collect();
    let word_count = words.len();

    // Opening passage for the GEO window: words before the first H2, which
    // approximates the lede. The old 30%-of-total metric failed every real
    // page by construction.
    let h2_at = Regex::new(r#"(?is)<h2\b"#)
        .ok()
        .and_then(|re| re.find(content))
        .map(|m| m.start())
        .unwrap_or(content.len());
    let lede_clean = TAG_RE.replace_all(&content[..h2_at], " ");
    let lede_text = strip_html.replace_all(&lede_clean, " ");
    let opening_words = lede_text.split_whitespace().count();

    let title_len = title.as_ref().map(|s| s.chars().count()).unwrap_or(0);
    let description_len = description.as_ref().map(|s| s.chars().count()).unwrap_or(0);

    let em_dash_count = content.chars().filter(|&c| c == '\u{2014}').count();
    let ai_slop_words_found = slop_hits(&plain_text);

    let checks = vec![
        check_title_length(title_len),
        check_meta_description(description_len),
        check_h1_uniqueness(h1_count),
        check_heading_hierarchy(&heading_skipped_levels),
        check_content_depth(word_count),
        check_image_alt_tags(image_count, images_missing_alt),
        check_geo_citation_density(opening_words),
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
        schema_json_valid,
        canonical_found,
        og_tags_found,
        geo_opening_words: opening_words,
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

/// Two pages fighting for one stem. Winner = more words, then lower path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannibalizationPair {
    pub keyword_stem: String,
    pub a: String,
    pub b: String,
    pub winner: String,
    pub a_words: usize,
    pub b_words: usize,
}

/// Expand every multi-file stem into explicit A×B pairs for editors.
pub fn cannibalization_pairs(rep: &DirectoryAuditReport) -> Vec<CannibalizationPair> {
    let words_for = |path: &str| -> usize {
        rep.reports
            .iter()
            .find(|r| r.file_path == path)
            .map(|r| r.word_count)
            .unwrap_or(0)
    };
    let mut out = Vec::new();
    for item in &rep.keyword_cannibalization {
        let files = &item.colliding_files;
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let (a, b) = (files[i].clone(), files[j].clone());
                let (aw, bw) = (words_for(&a), words_for(&b));
                let winner = if aw > bw || (aw == bw && a < b) { a.clone() } else { b.clone() };
                out.push(CannibalizationPair {
                    keyword_stem: item.keyword_stem.clone(),
                    a,
                    b,
                    winner,
                    a_words: aw,
                    b_words: bw,
                });
            }
        }
    }
    out.sort_by(|x, y| {
        x.keyword_stem
            .cmp(&y.keyword_stem)
            .then(x.a.cmp(&y.a))
            .then(x.b.cmp(&y.b))
    });
    out
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
    #[serde(default)]
    pub findings: Vec<crate::rules::Finding>,
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

        return Ok(with_findings(DirectoryAuditReport {
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
            findings: Vec::new(),
        }))
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
        findings: Vec::new(),
    })
}

/// Fill rule findings after construction. Shared by live audits and --rescore.
pub fn with_findings(mut rep: DirectoryAuditReport) -> DirectoryAuditReport {
    rep.findings = crate::rules::check_audit(&rep);
    rep
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

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// ASCII-fold a line for PDF standard fonts (WinAnsi only).
fn pdf_text(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '—' | '–' => '-',
            '“' | '”' => '"',
            '‘' | '’' => '\'',
            '…' => '.',
            '▲' => '^',
            '▼' => 'v',
            '✖' | '⚠' => '!',
            c if c.is_ascii() => c,
            _ => '?',
        })
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Minimal multi-page PDF 1.4 writer, std only. Helvetica, A4, 11pt.
fn pdf_lines(title: &str, lines: &[String]) -> Vec<u8> {
    const PER_PAGE: usize = 48;
    let mut pages: Vec<&[String]> = Vec::new();
    let mut i = 0;
    while i < lines.len().max(1) {
        pages.push(&lines[i..(i + PER_PAGE).min(lines.len())]);
        i += PER_PAGE;
        if lines.is_empty() {
            break;
        }
    }
    // Objects: 1 catalog, 2 pages, 3 font, then per page (page, content).
    let mut objs: Vec<Vec<u8>> = Vec::new();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    let kids: Vec<String> = (0..pages.len()).map(|p| format!("{} 0 R", 4 + p * 2)).collect();
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()).as_bytes().to_vec());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    for (pi, chunk) in pages.iter().enumerate() {
        let content_obj = 5 + pi * 2;
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
                content_obj
            )
            .as_bytes()
            .to_vec(),
        );
        let mut stream = String::from("BT /F1 11 Tf 50 790 Td 14 TL ");
        if pi == 0 {
            stream.push_str(&format!("({}) Tj T* ", pdf_text(title)));
        }
        for line in chunk.iter() {
            stream.push_str(&format!("({}) Tj T* ", pdf_text(line)));
        }
        stream.push_str("ET");
        let bytes = stream.as_bytes();
        let mut obj = format!("<< /Length {} >>\nstream\n", bytes.len()).as_bytes().to_vec();
        obj.extend_from_slice(bytes);
        obj.extend_from_slice(b"\nendstream");
        objs.push(obj);
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (n, body) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", n + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objs.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF", objs.len() + 1, xref_at).as_bytes(),
    );
    out
}

/// Markdown twin of to_html: scorecard, pages, findings, method.
pub fn to_markdown(rep: &DirectoryAuditReport) -> String {
    let score = rep.pass_rate.round().clamp(0.0, 100.0) as u32;
    let mut m = String::new();
    m.push_str(&format!(
        "# SEO audit: {}\n\nScore **{}/100 ({})** across {} files and {} words, pass rate {:.1}%.\n",
        rep.dir_path, score, crate::actions::grade(score), rep.total_files, rep.total_words, rep.pass_rate
    ));
    m.push_str("\n## Pages\n\n| Page | Words | Title | Checks |\n|---|---|---|---|\n");
    for r in &rep.reports {
        let passed = r.checks.iter().filter(|c| c.passed).count();
        m.push_str(&format!(
            "| {} | {} | {} | {}/{} |\n",
            r.file_path,
            r.word_count,
            r.title.as_deref().unwrap_or(""),
            passed,
            r.checks.len()
        ));
    }
    if !rep.duplicate_titles.is_empty() {
        m.push_str(&format!("\n## Duplicate titles ({})\n\n", rep.duplicate_titles.len()));
        let mut titles: Vec<&String> = rep.duplicate_titles.keys().collect();
        titles.sort();
        for t in titles {
            m.push_str(&format!("- {} ({} files)\n", t, rep.duplicate_titles[t].len()));
        }
    }
    if !rep.orphan_pages.is_empty() {
        m.push_str(&format!("\n## Orphan pages ({})\n\n", rep.orphan_pages.len()));
        for f in &rep.orphan_pages {
            m.push_str(&format!("- {}\n", f));
        }
    }
    if !rep.thin_pages.is_empty() {
        m.push_str(&format!("\n## Thin pages ({})\n\n", rep.thin_pages.len()));
        for (f, wc) in &rep.thin_pages {
            m.push_str(&format!("- {} ({} words)\n", f, wc));
        }
    }
    if !rep.keyword_cannibalization.is_empty() {
        m.push_str(&format!("\n## Keyword cannibalization ({})\n\n", rep.keyword_cannibalization.len()));
        for item in &rep.keyword_cannibalization {
            m.push_str(&format!("- {} ({} pages)\n", item.keyword_stem, item.colliding_files.len()));
        }
        let pairs = cannibalization_pairs(rep);
        if !pairs.is_empty() {
            m.push_str(&format!("\n### Conflict pairs ({})\n\n", pairs.len()));
            m.push_str("| Stem | A | B | A words | B words | Keep |\n|---|---|---|---:|---:|---|\n");
            for p in pairs.iter().take(20) {
                m.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    p.keyword_stem, p.a, p.b, p.a_words, p.b_words, p.winner
                ));
            }
        }
    }
    m.push_str("\n## Method\n\nOn-page checks per file, duplicate titles, orphan link graph, thin-page and cannibalization radar. Scores rank work; they never predict rankings or traffic.\n");
    m.push_str("\n## Completeness\n\nLocal file walk only. Jev and live crawl are not part of this report unless run separately. Companion artifacts: `run.json`, `ledger.json`.\n");
    m
}

/// PDF twin of to_html: scorecard, pages, findings, optional narrative, method.
#[allow(dead_code)] // bin always embeds a narrative; tests call the plain path
pub fn to_pdf(rep: &DirectoryAuditReport) -> Vec<u8> {
    to_pdf_opt(rep, None)
}

/// PDF with an embedded narrative block when present.
pub fn to_pdf_with_narrative(rep: &DirectoryAuditReport, n: &crate::narrative::Narrative) -> Vec<u8> {
    to_pdf_opt(rep, Some(n))
}

/// Deck section 1: cover with score and grade.
pub fn pdf_cover(rep: &DirectoryAuditReport) -> Vec<String> {
    let score = rep.pass_rate.round().clamp(0.0, 100.0) as u32;
    vec![
        "jev-seo audit report".to_string(),
        format!("Target: {}", rep.dir_path),
        format!("Score: {}/100 ({})", score, crate::actions::grade(score)),
        format!(
            "Files: {}  Words: {}  Pass rate: {:.1}%",
            rep.total_files, rep.total_words, rep.pass_rate
        ),
    ]
}

/// Deck section 2: scorecard totals plus ranked impact list.
pub fn pdf_scorecard(rep: &DirectoryAuditReport) -> Vec<String> {
    let mut lines = vec!["Scorecard:".to_string()];
    lines.push(format!(
        "Files walked: {}  Words: {}  Avg words/file: {}",
        rep.total_files, rep.total_words, rep.avg_words_per_file
    ));
    lines.push(format!(
        "Findings recorded: {}  Duplicate titles: {}  Orphans: {}  Thin pages: {}",
        rep.findings.len(),
        rep.duplicate_titles.len(),
        rep.orphan_pages.len(),
        rep.thin_pages.len()
    ));
    lines.push(String::new());
    lines.push("Top actions by impact:".to_string());
    let actions = crate::rules::actions_for(&rep.findings);
    if actions.is_empty() {
        lines.push("- none, directory is clean".to_string());
    }
    for a in actions.iter().take(8) {
        lines.push(format!(
            "- [P{}] {}  impact {}  effort {}{}",
            a.priority,
            a.id,
            a.impact,
            a.effort,
            if a.quick_win { "  quick-win" } else { "" }
        ));
    }
    lines
}

/// Deck section 3: findings grouped by area with severity.
pub fn pdf_findings_by_area(rep: &DirectoryAuditReport) -> Vec<String> {
    use std::collections::BTreeMap;
    let mut lines = vec!["Findings by area:".to_string()];
    if rep.findings.is_empty() {
        lines.push("- none".to_string());
        return lines;
    }
    let mut by_area: BTreeMap<String, Vec<&crate::rules::Finding>> = BTreeMap::new();
    for f in &rep.findings {
        by_area
            .entry(crate::rules::label(&f.area).to_string())
            .or_default()
            .push(f);
    }
    for (area, list) in &by_area {
        lines.push(format!("{} ({}):", area, list.len()));
        for f in list.iter().take(8) {
            lines.push(format!("- {} {:?} {}", f.rule_id, f.severity, f.scope));
        }
    }
    lines
}

/// Deck section 4: page inventory with per-page check counts.
pub fn pdf_inventory(rep: &DirectoryAuditReport) -> Vec<String> {
    let mut lines = vec!["Page inventory (path | words | title | checks):".to_string()];
    for r in &rep.reports {
        let passed = r.checks.iter().filter(|c| c.passed).count();
        lines.push(format!(
            "- {} | {}w | {} | {}/{}",
            r.file_path,
            r.word_count,
            r.title.as_deref().unwrap_or(""),
            passed,
            r.checks.len()
        ));
    }
    if !rep.duplicate_titles.is_empty() {
        lines.push(String::new());
        lines.push(format!("Duplicate titles ({}):", rep.duplicate_titles.len()));
        let mut titles: Vec<&String> = rep.duplicate_titles.keys().collect();
        titles.sort();
        for t in titles.iter().take(10) {
            lines.push(format!("- {} ({} files)", t, rep.duplicate_titles[*t].len()));
        }
    }
    if !rep.orphan_pages.is_empty() {
        lines.push(String::new());
        lines.push(format!("Orphan pages ({}):", rep.orphan_pages.len()));
        for f in rep.orphan_pages.iter().take(10) {
            lines.push(format!("- {}", f));
        }
    }
    if !rep.thin_pages.is_empty() {
        lines.push(String::new());
        lines.push(format!("Thin pages ({}):", rep.thin_pages.len()));
        for (f, wc) in rep.thin_pages.iter().take(10) {
            lines.push(format!("- {} ({} words)", f, wc));
        }
    }
    lines
}

/// Deck section 5: narrative block when present.
pub fn pdf_narrative(n: &crate::narrative::Narrative) -> Vec<String> {
    let mut lines = vec!["Narrative:".to_string()];
    if n.automatic {
        lines.push("(automatic evidence-only summary)".to_string());
    }
    for p in &n.executive_summary {
        lines.push(p.clone());
    }
    if !n.risks.is_empty() {
        lines.push("Risks:".to_string());
        for r in &n.risks {
            lines.push(format!("- {}", r));
        }
    }
    if !n.unverified_numbers.is_empty() {
        lines.push(format!(
            "Warning: numbers not in audit: {}",
            n.unverified_numbers.join(", ")
        ));
    }
    lines
}

/// Deck section 6: method appendix, same facts as the Markdown twin.
pub fn pdf_method() -> Vec<String> {
    vec![
        "Method:".to_string(),
        "On-page checks per file, duplicate titles, orphan link graph, thin-page and cannibalization radar.".to_string(),
        "Scores rank work; they never predict rankings or traffic.".to_string(),
        "Completeness: local file walk; Jev and live crawl not in this report.".to_string(),
        "Companion files: run.json, ledger.json.".to_string(),
    ]
}

fn to_pdf_opt(rep: &DirectoryAuditReport, n: Option<&crate::narrative::Narrative>) -> Vec<u8> {
    let mut lines = pdf_cover(rep);
    lines.push(String::new());
    lines.extend(pdf_scorecard(rep));
    lines.push(String::new());
    lines.extend(pdf_findings_by_area(rep));
    lines.push(String::new());
    lines.extend(pdf_inventory(rep));
    if let Some(n) = n {
        lines.push(String::new());
        lines.extend(pdf_narrative(n));
    }
    lines.push(String::new());
    lines.extend(pdf_method());
    pdf_lines(&format!("jev-seo audit report: {}", rep.dir_path), &lines)
}

/// Single-file HTML report for humans. Same numbers as the JSON output.
pub fn to_html(rep: &DirectoryAuditReport) -> String {
    let score = rep.pass_rate.round().clamp(0.0, 100.0) as u32;
    let grade = crate::actions::grade(score);
    let mut h = String::new();
    h.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    h.push_str("<title>jev-seo audit report</title>");
    h.push_str("<style>:root{--pink:#f386a1;--ink:#111;--mut:#666;--line:#e2e2e2}body{font-family:system-ui,-apple-system,sans-serif;color:var(--ink);max-width:960px;margin:0 auto;padding:2rem 1rem;line-height:1.55}.hero{background:#111;color:#fff;border-radius:10px;padding:2rem;display:flex;gap:2rem;align-items:center;flex-wrap:wrap}.grade{font-size:4rem;font-weight:800;background:var(--pink);color:#111;border-radius:10px;min-width:6rem;text-align:center}.meta{font-size:.9rem;color:#bbb}table{border-collapse:collapse;width:100%;margin-top:1rem}th,td{border:1px solid var(--line);padding:.45rem .65rem;text-align:left;font-size:.9rem}th{background:#f6f6f6}h2{margin-top:2.2rem;font-size:1.15rem;border-bottom:2px solid var(--pink);display:inline-block}ul{padding-left:1.2rem}.foot{margin-top:2.5rem;font-size:.8rem;color:var(--mut);border-top:1px solid var(--line);padding-top:1rem}code{background:#f1f1f1;padding:.1rem .35rem;border-radius:4px;font-size:.85em}</style>");
    h.push_str("</head><body>");
    h.push_str(&format!(
        "<div class=\"hero\"><div class=\"grade\">{}</div><div><h1 style=\"margin:0;font-size:1.5rem\">SEO audit: {}</h1><p class=\"meta\">{} files | {} words | pass rate {:.1}% | score {}/100</p></div></div>",
        grade,
        esc(&rep.dir_path),
        rep.total_files,
        rep.total_words,
        rep.pass_rate,
        score
    ));
    h.push_str("<h2>Pages</h2><table><tr><th>Page</th><th>Words</th><th>Title</th><th>Checks passed</th></tr>");
    for r in &rep.reports {
        let passed = r.checks.iter().filter(|c| c.passed).count();
        h.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}/{}</td></tr>",
            esc(&r.file_path),
            r.word_count,
            esc(r.title.as_deref().unwrap_or("")),
            passed,
            r.checks.len()
        ));
    }
    h.push_str("</table>");
    if !rep.duplicate_titles.is_empty() {
        h.push_str("<h2>Duplicate titles</h2><ul>");
        let mut titles: Vec<&String> = rep.duplicate_titles.keys().collect();
        titles.sort();
        for t in titles {
            h.push_str(&format!("<li>{} ({} files)</li>", esc(t), rep.duplicate_titles[t].len()));
        }
        h.push_str("</ul>");
    }
    if !rep.orphan_pages.is_empty() {
        h.push_str(&format!("<h2>Orphan pages ({})</h2><ul>", rep.orphan_pages.len()));
        for f in &rep.orphan_pages {
            h.push_str(&format!("<li>{}</li>", esc(f)));
        }
        h.push_str("</ul>");
    }
    if !rep.thin_pages.is_empty() {
        h.push_str(&format!("<h2>Thin pages ({})</h2><ul>", rep.thin_pages.len()));
        for (f, wc) in &rep.thin_pages {
            h.push_str(&format!("<li>{} ({} words)</li>", esc(f), wc));
        }
        h.push_str("</ul>");
    }
    if !rep.keyword_cannibalization.is_empty() {
        h.push_str("<h2>Keyword cannibalization</h2><ul>");
        for item in &rep.keyword_cannibalization {
            h.push_str(&format!(
                "<li>{} ({} pages)</li>",
                esc(&item.keyword_stem),
                item.colliding_files.len()
            ));
        }
        h.push_str("</ul>");
    }
    h.push_str("<div class=\"foot\">Generated locally by <code>jev-seo audit --html</code>. Scores rank work; they never predict rankings or traffic. Method: on-page checks per file, duplicate titles, orphan detection via internal link graph, thin-page and cannibalization radar. Completeness: local file walk; Jev and live crawl not part of this report unless run separately. Companion files: <code>run.json</code>, <code>ledger.json</code>.</div>");
    h.push_str("</body></html>");
    h
}
