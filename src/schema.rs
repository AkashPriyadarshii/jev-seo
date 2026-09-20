use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationReport {
    pub target: String,
    pub schemas_found: usize,
    pub types: Vec<String>,
    pub is_valid: bool,
    pub completeness_score: u32,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub details: Vec<SchemaDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDetail {
    pub schema_type: String,
    pub is_valid: bool,
    pub missing_required: Vec<String>,
    pub missing_recommended: Vec<String>,
    pub is_deprecated: bool,
    pub deprecation_notice: Option<String>,
}

pub fn validate_target(target_str: &str) -> Result<SchemaValidationReport> {
    let content = if std::path::Path::new(target_str).exists() {
        std::fs::read_to_string(target_str)
            .with_context(|| format!("Failed to read target file: {}", target_str))?
    } else {
        target_str.to_string()
    };

    validate_content(target_str, &content)
}

pub fn validate_content(target_label: &str, content: &str) -> Result<SchemaValidationReport> {
    let mut raw_schemas = Vec::new();

    // 1. Try extracting YAML frontmatter schema
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);
    if let Some(data) = parsed.data {
        if let Ok(val) = data.as_hashmap() {
            if let Some(s) = val.get("schema").or_else(|| val.get("json_ld")) {
                let v = pod_to_json(s);
                if v.is_object() || v.is_array() {
                    raw_schemas.push(v);
                }
            }
        }
    }

    // 2. Try extracting <script type="application/ld+json">...</script>
    let re = Regex::new(r#"(?is)<script[^>]*type=["']application/ld\+json["'][^>]*>(.*?)</script>"#)?;
    for cap in re.captures_iter(content) {
        if let Some(mat) = cap.get(1) {
            let body = mat.as_str().trim();
            if let Ok(v) = serde_json::from_str::<Value>(body) {
                raw_schemas.push(v);
            }
        }
    }

    // 3. Try parsing raw JSON if passed directly
    if raw_schemas.is_empty() {
        if let Ok(v) = serde_json::from_str::<Value>(content.trim()) {
            if v.is_object() || v.is_array() {
                raw_schemas.push(v);
            }
        }
    }

    if raw_schemas.is_empty() {
        return Ok(SchemaValidationReport {
            target: target_label.to_string(),
            schemas_found: 0,
            types: Vec::new(),
            is_valid: false,
            completeness_score: 0,
            warnings: vec!["No JSON-LD schema or frontmatter markup found in target.".into()],
            errors: vec!["Missing application/ld+json schema markup.".into()],
            details: Vec::new(),
        });
    }

    // Flatten any @graph or arrays
    let mut items = Vec::new();
    for s in raw_schemas {
        if let Some(arr) = s.as_array() {
            items.extend(arr.clone());
        } else if let Some(graph) = s.get("@graph").and_then(|g| g.as_array()) {
            items.extend(graph.clone());
        } else {
            items.push(s);
        }
    }

    let mut types = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut details = Vec::new();
    let mut total_score_acc = 0u32;

    for item in &items {
        let (detail, errs, warns, score) = audit_schema_item(item);
        types.push(detail.schema_type.clone());
        errors.extend(errs);
        warnings.extend(warns);
        total_score_acc += score;
        details.push(detail);
    }

    let avg_score = if !items.is_empty() {
        total_score_acc / items.len() as u32
    } else {
        0
    };

    let is_valid = errors.is_empty() && !items.is_empty();

    Ok(SchemaValidationReport {
        target: target_label.to_string(),
        schemas_found: items.len(),
        types,
        is_valid,
        completeness_score: avg_score,
        warnings,
        errors,
        details,
    })
}

const INITIAL_SCHEMA_SCORE: u32 = 100;
const MISSING_CONTEXT_PENALTY: u32 = 30;
const MISSING_TYPE_PENALTY: u32 = 40;
const DEPRECATED_SCHEMA_PENALTY: u32 = 25;

fn audit_schema_item(item: &Value) -> (SchemaDetail, Vec<String>, Vec<String>, u32) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut score = INITIAL_SCHEMA_SCORE;

    // Verify @context
    let context_str = item.get("@context").and_then(|c| c.as_str()).unwrap_or("");
    if !context_str.contains("schema.org") {
        errors.push("Missing or invalid @context: must contain 'https://schema.org'".into());
        score = score.saturating_sub(MISSING_CONTEXT_PENALTY);
    }

    // Verify @type
    let schema_type = item.get("@type").and_then(|t| t.as_str()).unwrap_or("Unknown").to_string();
    if schema_type == "Unknown" {
        errors.push("Missing @type definition in schema object".into());
        score = score.saturating_sub(MISSING_TYPE_PENALTY);
    }

    let mut missing_required = Vec::new();
    let mut missing_recommended = Vec::new();
    let mut is_deprecated = false;
    let mut deprecation_notice = None;

    match schema_type.as_str() {
        "SoftwareApplication" => {
            validate_software_app(item, &mut missing_required, &mut missing_recommended, &mut score);
        }
        "Article" | "TechArticle" | "BlogPosting" | "NewsArticle" => {
            validate_article(item, &mut missing_required, &mut missing_recommended, &mut score);
        }
        "Organization" | "Corporation" => {
            validate_organization(item, &mut missing_required, &mut missing_recommended, &mut score);
        }
        "WebSite" => {
            validate_website(item, &mut missing_required, &mut score);
        }
        "Product" => {
            validate_product(item, &mut missing_required, &mut missing_recommended, &mut score);
        }
        "FAQPage" => {
            check_field(item, "mainEntity", true, &mut missing_required, &mut score, 40);
            warnings.push("Google restricted FAQPage rich results primarily to authoritative health and government domains (Sept 2023)".into());
        }
        "HowTo" => {
            is_deprecated = true;
            let msg = "Google deprecated HowTo rich snippets worldwide in Sept 2023. This schema no longer triggers rich results.".to_string();
            deprecation_notice = Some(msg.clone());
            warnings.push(msg);
            score = score.saturating_sub(DEPRECATED_SCHEMA_PENALTY);
        }
        "SpecialAnnouncement" => {
            is_deprecated = true;
            let msg = "SpecialAnnouncement schema is deprecated by Google Search post-COVID.".to_string();
            deprecation_notice = Some(msg.clone());
            warnings.push(msg);
            score = score.saturating_sub(DEPRECATED_SCHEMA_PENALTY);
        }
        _ => {
            check_field(item, "name", false, &mut missing_recommended, &mut score, 10);
        }
    }

    if !missing_required.is_empty() {
        errors.push(format!("{}: missing required properties [{}]", schema_type, missing_required.join(", ")));
    }
    if !missing_recommended.is_empty() {
        warnings.push(format!("{}: missing recommended properties [{}]", schema_type, missing_recommended.join(", ")));
    }

    let detail = SchemaDetail {
        schema_type,
        is_valid: missing_required.is_empty() && errors.is_empty(),
        missing_required,
        missing_recommended,
        is_deprecated,
        deprecation_notice,
    };

    (detail, errors, warnings, score)
}

fn validate_software_app(item: &Value, required: &mut Vec<String>, recommended: &mut Vec<String>, score: &mut u32) {
    check_field(item, "name", true, required, score, 20);
    if item.get("operatingSystem").is_none() && item.get("applicationCategory").is_none() {
        required.push("operatingSystem or applicationCategory".into());
        *score = score.saturating_sub(15);
    }
    check_field(item, "offers", false, recommended, score, 10);
    check_field(item, "description", false, recommended, score, 10);
}

fn validate_article(item: &Value, required: &mut Vec<String>, recommended: &mut Vec<String>, score: &mut u32) {
    check_field(item, "headline", true, required, score, 20);
    check_field(item, "author", true, required, score, 20);
    check_field(item, "datePublished", true, required, score, 15);
    check_field(item, "publisher", false, recommended, score, 10);
    check_field(item, "image", false, recommended, score, 10);
}

fn validate_organization(item: &Value, required: &mut Vec<String>, recommended: &mut Vec<String>, score: &mut u32) {
    check_field(item, "name", true, required, score, 30);
    check_field(item, "url", true, required, score, 25);
    check_field(item, "logo", false, recommended, score, 15);
}

fn validate_website(item: &Value, required: &mut Vec<String>, score: &mut u32) {
    check_field(item, "name", true, required, score, 35);
    check_field(item, "url", true, required, score, 35);
}

fn validate_product(item: &Value, required: &mut Vec<String>, recommended: &mut Vec<String>, score: &mut u32) {
    check_field(item, "name", true, required, score, 25);
    check_field(item, "offers", true, required, score, 25);
    check_field(item, "description", false, recommended, score, 10);
}

fn check_field(
    item: &Value,
    field: &str,
    is_required: bool,
    missing_list: &mut Vec<String>,
    score: &mut u32,
    deduct: u32,
) {
    if item.get(field).is_none() {
        missing_list.push(field.to_string());
        if is_required {
            *score = score.saturating_sub(deduct);
        } else {
            *score = score.saturating_sub(deduct / 2);
        }
    }
}

fn pod_to_json(pod: &gray_matter::Pod) -> Value {
    match pod {
        gray_matter::Pod::String(s) => Value::String(s.clone()),
        gray_matter::Pod::Integer(i) => Value::Number((*i).into()),
        gray_matter::Pod::Float(f) => serde_json::Number::from_f64(*f).map(Value::Number).unwrap_or(Value::Null),
        gray_matter::Pod::Boolean(b) => Value::Bool(*b),
        gray_matter::Pod::Array(arr) => Value::Array(arr.iter().map(pod_to_json).collect()),
        gray_matter::Pod::Hash(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), pod_to_json(v));
            }
            Value::Object(obj)
        }
        gray_matter::Pod::Null => Value::Null,
    }
}

