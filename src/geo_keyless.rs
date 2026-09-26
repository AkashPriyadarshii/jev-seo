//! Deterministic keyless GEO score: what the free tier answers when no Jev key
//! is configured. Documented formula, no model, no network:
//! term coverage 40 + answer-first 20 + structure 20 + depth 20 = 100.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeylessGeo {
    pub score_10: u32,
    pub score_100: u32,
    pub signals: Vec<String>,
}

/// Query words that carry meaning: lowercase, 3+ chars, deduped.
fn query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for w in query.split(|c: char| !c.is_alphanumeric()) {
        let t = w.to_lowercase();
        if t.chars().count() >= 3 && !terms.contains(&t) {
            terms.push(t);
        }
    }
    terms
}

fn coverage(terms: &[String], text: &str) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }
    let lower = text.to_lowercase();
    let hit = terms.iter().filter(|t| lower.contains(t.as_str())).count();
    hit as f64 / terms.len() as f64
}

pub fn score(text: &str, query: &str) -> KeylessGeo {
    let terms = query_terms(query);
    let words = text.split_whitespace().count();
    let opening: String = text.chars().take(400).collect();

    let cover = coverage(&terms, text);
    let first = coverage(&terms, &opening);
    let cover_pts = (cover * 40.0).round() as u32;
    let first_pts = (first * 20.0).round() as u32;

    let mut struct_pts = 0u32;
    let mut signals = vec![format!("term coverage {:.0}%", cover * 100.0)];
    if text.contains("\n# ") || text.contains("\n## ") || text.contains("<h2") || text.contains("<h3") {
        struct_pts += 7;
        signals.push("headings present".into());
    }
    if text.contains("\n- ") || text.contains("\n* ") || text.contains("\n1. ") || text.contains("<li") {
        struct_pts += 7;
        signals.push("list structure present".into());
    }
    if text.contains('|') && text.contains('\n') || text.contains("<table") {
        struct_pts += 6;
        signals.push("table structure present".into());
    }
    let depth_pts = if words >= 600 {
        20
    } else if words >= 300 {
        12
    } else if words >= 100 {
        6
    } else {
        0
    };
    signals.push(format!("{} words", words));

    let total = (cover_pts + first_pts + struct_pts + depth_pts).min(100);
    KeylessGeo { score_10: total / 10, score_100: total, signals }
}
