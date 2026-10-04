//! Multi-dimensional Search Health + Data Coverage. Overall 0-100 stays for
//! compat; nine dims expose where the number comes from. Coverage says what
//! evidence exists, so free-lane limits print instead of hiding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Health {
    pub overall: f64,
    pub dims: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Coverage {
    pub crawl_pages: usize,
    pub gsc_days: usize,
    pub serp_queries: usize,
    pub ai_checks: usize,
    pub citation_confidence: f64,
    pub freshest_note: String,
}

/// Map rule areas to the nine public dims. Technical is crawl health;
/// indexability comes from seo_index; authority stays honest (no fake index).
#[allow(dead_code)]
pub fn from_areas(areas: &HashMap<String, f64>, overall: f64) -> Health {
    let get = |k: &str| areas.get(k).copied().unwrap_or(0.0);
    let dims = HashMap::from([
        ("technical".into(), get("crawl")),
        ("content_evidence".into(), get("content")),
        ("performance".into(), get("performance")),
        ("authority".into(), get("links")),
        ("indexability".into(), get("canonical")),
        ("search_visibility".into(), get("onpage")),
        ("ai_visibility".into(), get("ai_access")),
        ("structured".into(), get("structured")),
        ("security".into(), get("security")),
    ]);
    Health { overall, dims }
}

#[allow(dead_code)]
pub fn empty_coverage() -> Coverage {
    Coverage {
        crawl_pages: 0,
        gsc_days: 0,
        serp_queries: 0,
        ai_checks: 0,
        citation_confidence: 0.0,
        freshest_note: "no observations this run".into(),
    }
}
