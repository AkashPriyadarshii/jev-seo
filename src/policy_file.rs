//! `.jev-seo/policy.toml` — the repo's search contract. Read-only in v0.4.0:
//! declare gates, locales, engines, budgets. `jev-seo check` enforces it.
//! Missing file = defaults (blocking fail_on, min_health 0, US/en).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Policy {
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub target: Target,
    #[serde(default)]
    pub gates: Gates,
    #[serde(default)]
    pub search: Search,
    #[serde(default)]
    pub ai: Ai,
    #[serde(default)]
    pub budget: Budget,
    #[serde(default)]
    pub paths: Paths,
    #[serde(default)]
    pub providers: Providers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    #[serde(default)]
    pub framework: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default = "default_device")]
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gates {
    #[serde(default = "default_fail_on")]
    pub fail_on: String,
    #[serde(default)]
    pub max_critical: usize,
    #[serde(default)]
    pub min_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Search {
    #[serde(default = "default_engines")]
    pub engines: Vec<String>,
    #[serde(default = "default_locations")]
    pub locations: Vec<String>,
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ai {
    #[serde(default = "default_ai_engines")]
    pub engines: Vec<String>,
    #[serde(default = "default_confidence")]
    pub min_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    #[serde(default = "default_jev")]
    pub jev_usd: f64,
    #[serde(default = "default_external")]
    pub external_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Paths {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Providers {
    #[serde(default = "default_mode")]
    pub serp: String,
    #[serde(default = "default_mode")]
    pub renderer: String,
    #[serde(default = "default_mode")]
    pub ai: String,
    #[serde(default = "default_mode")]
    pub backlink: String,
}

fn default_name() -> String {
    "site".into()
}
fn default_locale() -> String {
    "en-IN".into()
}
fn default_device() -> String {
    "mobile".into()
}
fn default_fail_on() -> String {
    "blocking".into()
}
fn default_engines() -> Vec<String> {
    vec!["google".into(), "bing".into()]
}
fn default_locations() -> Vec<String> {
    vec!["IN".into(), "US".into()]
}
fn default_languages() -> Vec<String> {
    vec!["en".into()]
}
fn default_ai_engines() -> Vec<String> {
    vec!["chatgpt".into(), "perplexity".into()]
}
fn default_confidence() -> f64 {
    0.80
}
fn default_jev() -> f64 {
    0.25
}
fn default_external() -> f64 {
    1.00
}
fn default_mode() -> String {
    "auto".into()
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: default_name(),
            base_url: String::new(),
        }
    }
}
impl Default for Target {
    fn default() -> Self {
        Self {
            framework: String::new(),
            locale: default_locale(),
            device: default_device(),
        }
    }
}
impl Default for Gates {
    fn default() -> Self {
        Self {
            fail_on: default_fail_on(),
            max_critical: 0,
            min_health: 0.0,
        }
    }
}
impl Default for Search {
    fn default() -> Self {
        Self {
            engines: default_engines(),
            locations: default_locations(),
            languages: default_languages(),
        }
    }
}
impl Default for Ai {
    fn default() -> Self {
        Self {
            engines: default_ai_engines(),
            min_confidence: default_confidence(),
        }
    }
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            jev_usd: default_jev(),
            external_usd: default_external(),
        }
    }
}

/// Load `.jev-seo/policy.toml` from cwd. Missing file = defaults, no error.
pub fn load() -> Policy {
    load_at(".jev-seo/policy.toml")
}

pub fn load_at(path: &str) -> Policy {
    let src = std::fs::read_to_string(path).unwrap_or_default();
    if src.trim().is_empty() {
        return Policy::default();
    }
    toml::from_str(&src).unwrap_or_default()
}
