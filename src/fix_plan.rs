//! `jev-seo fix-plan audit.json` — deterministic patch hints from findings.
//! No LLM, no prose generation. Jev is evaluator not writer.
//! One FixItem per finding, grouped by rule truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItem {
    pub rule: String,
    pub target: String,
    pub action: String,
    pub kind: String, // fact | heuristic
    pub confidence: f64,
    pub reason: String,
    pub verification: String,
    pub patch: PatchHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchHint {
    pub file: String,
    pub field: String,
    pub hint: String,
}

fn hint_for(rule_id: &str) -> (&'static str, &'static str, f64) {
    let bare = rule_id.strip_prefix("RULE-").unwrap_or(rule_id);
    match bare {
        "R09" => ("frontmatter.title", "add title: under 60 chars", 0.98),
        "R10" => ("frontmatter.title", "trim/expand title to 30-60 chars", 0.90),
        "R11" => ("frontmatter.description", "add description 120-160 chars", 0.98),
        "R12" => ("frontmatter.description", "trim/expand description to 120-160", 0.90),
        "R13" => ("body.h1", "add one H1", 0.98),
        "R14" => ("body.headings", "keep one H1, demote rest to H2", 0.90),
        "R15" => ("body.headings", "nest headings without jumps", 0.90),
        "R16" => ("body.images.alt", "add alt text", 0.98),
        "R18" => ("body.content", "expand past 300 words or merge", 0.60),
        "R19" => ("body.prose", "rewrite slop markers in plain words", 0.60),
        "R20" => ("body.prose", "hold em-dashes under 2/500w", 0.60),
        "R21" => ("frontmatter.title", "unique title per page", 0.85),
        "R22" => ("frontmatter.description", "unique description per page", 0.85),
        "R23" => ("frontmatter.title", "one winner per stem; merge/retarget", 0.60),
        "R24" => ("body.opening", "open with answer before background", 0.60),
        "R25" => ("body.links", "add inbound link from related page", 0.75),
        "R31" => ("body.schema", "add JSON-LD matching page type", 0.90),
        "R33" => ("body.schema", "fix JSON-LD syntax", 0.98),
        "R35" => ("head.opengraph", "add og:title/description/image", 0.90),
        "R36" => ("head.canonical", "add canonical URL", 0.98),
        "R58" => ("head.hreflang", "reciprocal hreflang, no noindex target", 0.75),
        "R47" => ("site.tls", "redirect to HTTPS + HSTS", 0.98),
        "R49" => ("site.canonical", "pick slash policy, enforce one URL", 0.85),
        "R50" => ("site.canonical", "strip utm_/tracking params", 0.85),
        _ => ("body", "see fix in rule registry", 0.70),
    }
}

pub fn from_findings(findings: &[crate::rules::Finding]) -> Vec<FixItem> {
    let mut out = Vec::new();
    for f in findings {
        let (field, hint, conf) = hint_for(&f.rule_id);
        let kind = crate::rules::truth_kind(&f.rule_id).to_string();
        let confidence = if kind == "heuristic" { conf.min(0.65) } else { conf };
        let rule = crate::rules::rule(&f.rule_id);
        let reason = rule
            .map(|r| format!("{} ({} {:?})", r.title, f.rule_id, r.severity))
            .unwrap_or_else(|| f.evidence.clone());
        let verification = format!("jev-seo audit {} --json | jq '.findings | no {}'", f.scope, f.rule_id);
        out.push(FixItem {
            rule: f.rule_id.clone(),
            target: f.scope.clone(),
            action: field.to_string(),
            kind,
            confidence,
            reason,
            verification,
            patch: PatchHint {
                file: f.scope.clone(),
                field: field.to_string(),
                hint: hint.to_string(),
            },
        });
    }
    // Blocking first, then confidence desc, then rule asc — deterministic
    out.sort_by(|a, b| {
        let ga = crate::rules::gate(&a.rule) == crate::rules::Gate::Blocking;
        let gb = crate::rules::gate(&b.rule) == crate::rules::Gate::Blocking;
        gb.cmp(&ga)
            .then(b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.rule.cmp(&b.rule))
            .then(a.target.cmp(&b.target))
    });
    out
}
