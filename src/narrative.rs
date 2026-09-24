//! Narrative contract: optional `narrative.json` beside the report.
//!
//! Pattern: free text may cite action IDs and numbers; the loader refuses
//! unknown IDs, flags numbers that match nothing in the audit, and checks
//! plan effort bands against the action table. Absent file => automatic
//! evidence-only summary (labeled automatic).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

pub const REQUIRED_KEYS: &[&str] = &["executive_summary", "strengths", "risks", "plan"];

/// Action effort 1..4 as shown on plan lines (must match actions::Action.effort).
pub fn effort_band(effort: u8) -> &'static str {
    crate::actions::effort_label(effort)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBlock {
    pub horizon: String,
    #[serde(default)]
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narrative {
    pub executive_summary: Vec<String>,
    pub strengths: Vec<String>,
    pub risks: Vec<String>,
    pub plan: Vec<PlanBlock>,
    #[serde(default)]
    pub closing: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub automatic: bool,
    #[serde(default)]
    pub unknown_ids: Vec<String>,
    #[serde(default)]
    pub unverified_numbers: Vec<String>,
    #[serde(default)]
    pub effort_mismatches: Vec<String>,
}

/// Extract `RULE-Rxx` / `RULE-Rxxx` and bare action-like tokens we accept in prose.
pub fn extract_action_ids(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_uppercase() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_uppercase() || bytes[i].is_ascii_digit() || bytes[i] == '-') {
            i += 1;
        }
        let tok: String = bytes[start..i].iter().collect();
        let is_rule = tok.starts_with("RULE-R") && tok.len() >= 8;
        let is_pref = tok.ends_with(|c: char| c.is_ascii_digit())
            && tok.contains('-')
            && tok.len() >= 5
            && matches!(tok.split('-').next().unwrap_or(""), "LLMS" | "CRAWL" | "ACT" | "SEO");
        if is_rule || is_pref {
            out.insert(tok);
        }
    }
    out
}

fn known_numbers(findings: &[crate::rules::Finding], actions: &[crate::actions::Action], n_files: usize) -> BTreeSet<u64> {
    // Store rounded micro-units so we avoid f64 Ord (not implemented).
    let mut known: BTreeSet<u64> = BTreeSet::new();
    let mut add = |v: f64| {
        known.insert((v * 1000.0).round() as u64);
        known.insert(v.round() as u64);
        known.insert((v * 100.0).round() as u64);
    };
    add(n_files as f64);
    add(findings.len() as f64);
    add(actions.len() as f64);
    for f in findings {
        for tok in f.evidence.split(|c: char| !c.is_ascii_digit() && c != '.') {
            if let Ok(v) = tok.parse::<f64>() {
                add(v);
            }
        }
    }
    for a in actions {
        add(a.impact as f64);
        add(a.effort as f64);
        add(a.priority as f64);
        for tok in a.evidence.split(|c: char| !c.is_ascii_digit() && c != '.') {
            if let Ok(v) = tok.parse::<f64>() {
                add(v);
            }
        }
    }
    known
}

/// Numbers in narrative text that match nothing citable in the audit.
/// Skips tiny integers (1..=10 without a decimal) and URL/date-like runs.
pub fn unverified_numbers(text: &str, findings: &[crate::rules::Finding], actions: &[crate::actions::Action], n_files: usize) -> Vec<String> {
    let cleaned = {
        let mut s = text.to_string();
        for id in extract_action_ids(text) {
            s = s.replace(&id, " ");
        }
        // strip URLs and paths
        let re_url = regex::Regex::new(r"(https?://|/|@)[\w@./%-]*").unwrap();
        s = re_url.replace_all(&s, " ").into_owned();
        let re_date = regex::Regex::new(r"\d{4}-\d{2}-\d{2}(?:T[\d:.+-]+)?").unwrap();
        re_date.replace_all(&s, " ").into_owned()
    };
    let known = known_numbers(findings, actions, n_files);
    // Rust regex crate: no look-around. Tokenize numbers, drop those glued to word/dot/comma.
    let re_num = regex::Regex::new(r"\d{1,3}(?:,\d{3})+(?:\.\d+)?|\d+(?:\.\d+)?").unwrap();
    let chars: Vec<char> = cleaned.chars().collect();
    let mut bad = Vec::new();
    for cap in re_num.captures_iter(&cleaned) {
        let m = cap.get(0).unwrap();
        let start = m.start();
        let tok = m.as_str().to_string();
        if start > 0 {
            let prev = chars[start - 1];
            if prev.is_alphanumeric() || prev == '.' || prev == ',' || prev == '%' {
                continue;
            }
        }
        let v: f64 = match tok.replace(',', "").parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v <= 10.0 && !tok.contains('.') {
            continue;
        }
        let keys = [
            (v * 1000.0).round() as u64,
            v.round() as u64,
            (v * 100.0).round() as u64,
        ];
        if !keys.iter().any(|k| known.contains(k)) && !bad.contains(&tok) {
            bad.push(tok);
        }
    }
    bad
}

/// Plan lines that state an effort band different from every action they cite.
pub fn effort_mismatches(n: &Narrative, actions: &[crate::actions::Action]) -> Vec<String> {
    let mut out = Vec::new();
    for block in &n.plan {
        for item in &block.items {
            let ids = extract_action_ids(item);
            if ids.is_empty() {
                continue;
            }
            let lower = item.to_lowercase();
            let stated = ["hours", "about a day", "several days", "a project"]
                .iter()
                .find(|band| lower.contains(&format!("({})", band)))
                .copied();
            if let Some(stated) = stated {
                let bands: BTreeSet<&str> = ids
                    .iter()
                    .filter_map(|id| actions.iter().find(|a| &a.id == id))
                    .map(|a| effort_band(a.effort))
                    .collect();
                if !bands.is_empty() && !bands.contains(stated) {
                    out.push(format!(
                        "{} says '({})' but the action table says '{}'",
                        ids.iter().cloned().collect::<Vec<_>>().join("/"),
                        stated,
                        bands.iter().cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }
    }
    out
}

pub fn unknown_ids(text: &str, actions: &[crate::actions::Action]) -> Vec<String> {
    let known: BTreeSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    // Also allow bare RULE-Rxx form when actions use RULE-Rxx already.
    extract_action_ids(text)
        .into_iter()
        .filter(|id| !known.contains(id.as_str()))
        .collect()
}

/// Load `narrative.json` from `dir` if present. Hard-fail on unknown action IDs.
pub fn load(
    dir: &Path,
    findings: &[crate::rules::Finding],
    actions: &[crate::actions::Action],
    n_files: usize,
) -> Result<Option<Narrative>> {
    let path = dir.join("narrative.json");
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut n: Narrative = serde_json::from_str(&raw).context("parse narrative.json")?;
    for k in REQUIRED_KEYS {
        if *k == "plan" && n.plan.is_empty() {
            bail!("narrative.json missing required key: plan");
        }
        if *k != "plan" {
            let arr = match *k {
                "executive_summary" => &n.executive_summary,
                "strengths" => &n.strengths,
                "risks" => &n.risks,
                _ => continue,
            };
            if arr.is_empty() {
                bail!("narrative.json missing or empty required key: {k}");
            }
        }
    }
    let text = serde_json::to_string(&n)?;
    n.unknown_ids = unknown_ids(&text, actions);
    if !n.unknown_ids.is_empty() {
        bail!(
            "narrative.json cites action IDs that do not exist: {:?}",
            n.unknown_ids
        );
    }
    n.unverified_numbers = unverified_numbers(&text, findings, actions, n_files);
    n.effort_mismatches = effort_mismatches(&n, actions);
    n.automatic = false;
    if n.author.is_none() {
        n.author = Some("jev-seo, from the audit evidence".into());
    }
    Ok(Some(n))
}

/// Evidence-only summary when narrative.json is absent. Always labeled automatic.
pub fn auto(
    findings: &[crate::rules::Finding],
    actions: &[crate::actions::Action],
    n_files: usize,
    pass_rate: f64,
) -> Narrative {
    let top: Vec<String> = actions
        .iter()
        .take(3)
        .map(|a| format!("{} {} (impact {}, effort {})", a.id, a.title, a.impact, effort_band(a.effort)))
        .collect();
    let strengths: Vec<String> = vec![format!(
        "{n_files} files walked; pass rate {pass_rate:.1}%; {} findings recorded",
        findings.len()
    )];
    let risks: Vec<String> = if top.is_empty() {
        vec!["No rule findings in this run.".into()]
    } else {
        top.to_vec()
    };
    Narrative {
        executive_summary: vec![
            format!("Overall pass rate {pass_rate:.1}% across {n_files} files. Scores rank work; they do not predict rankings or traffic."),
            if actions.is_empty() {
                "No prioritized actions from the rule engine.".into()
            } else {
                format!("Top work: {}.", top.join("; "))
            },
        ],
        strengths,
        risks,
        plan: vec![PlanBlock {
            horizon: "This week".into(),
            items: top.into_iter().take(3).collect(),
        }],
        closing: Some("Local file walk only unless crawl and Jev were run in the same session.".into()),
        author: Some("jev-seo automatic summary".into()),
        automatic: true,
        unknown_ids: vec![],
        unverified_numbers: vec![],
        effort_mismatches: vec![],
    }
}

/// Markdown block for embedding in reports.
pub fn to_markdown(n: &Narrative) -> String {
    let mut m = String::from("\n## Narrative\n\n");
    if n.automatic {
        m.push_str("_Automatic evidence-only summary (no narrative.json)._\n\n");
    } else if let Some(a) = &n.author {
        m.push_str(&format!("_Narrative by {a}._\n\n"));
    }
    for p in &n.executive_summary {
        m.push_str(p);
        m.push_str("\n\n");
    }
    if !n.strengths.is_empty() {
        m.push_str("### Strengths\n\n");
        for s in &n.strengths {
            m.push_str(&format!("- {s}\n"));
        }
        m.push('\n');
    }
    if !n.risks.is_empty() {
        m.push_str("### Risks\n\n");
        for s in &n.risks {
            m.push_str(&format!("- {s}\n"));
        }
        m.push('\n');
    }
    if !n.plan.is_empty() {
        m.push_str("### Plan\n\n");
        for b in &n.plan {
            m.push_str(&format!("**{}**\n\n", b.horizon));
            for item in &b.items {
                m.push_str(&format!("- {item}\n"));
            }
            m.push('\n');
        }
    }
    if let Some(c) = &n.closing {
        m.push_str(c);
        m.push('\n');
    }
    if !n.unverified_numbers.is_empty() {
        m.push_str(&format!(
            "\n_Warnings: numbers not found in the audit: {}._\n",
            n.unverified_numbers.join(", ")
        ));
    }
    if !n.effort_mismatches.is_empty() {
        m.push_str(&format!(
            "\n_Warnings: effort mismatches: {}._\n",
            n.effort_mismatches.join("; ")
        ));
    }
    m
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// HTML narrative section for the single-file report.
pub fn to_html(n: &Narrative) -> String {
    let mut h = String::from("<h2>Narrative</h2>");
    if n.automatic {
        h.push_str("<p class=\"meta\">Automatic evidence-only summary (no narrative.json).</p>");
    }
    for p in &n.executive_summary {
        h.push_str(&format!("<p>{}</p>", esc(p)));
    }
    if !n.strengths.is_empty() {
        h.push_str("<h3>Strengths</h3><ul>");
        for s in &n.strengths {
            h.push_str(&format!("<li>{}</li>", esc(s)));
        }
        h.push_str("</ul>");
    }
    if !n.risks.is_empty() {
        h.push_str("<h3>Risks</h3><ul>");
        for s in &n.risks {
            h.push_str(&format!("<li>{}</li>", esc(s)));
        }
        h.push_str("</ul>");
    }
    if !n.plan.is_empty() {
        h.push_str("<h3>Plan</h3>");
        for b in &n.plan {
            h.push_str(&format!("<p><strong>{}</strong></p><ul>", esc(&b.horizon)));
            for item in &b.items {
                h.push_str(&format!("<li>{}</li>", esc(item)));
            }
            h.push_str("</ul>");
        }
    }
    if !n.unverified_numbers.is_empty() || !n.effort_mismatches.is_empty() {
        h.push_str("<p class=\"meta\">Warnings: ");
        if !n.unverified_numbers.is_empty() {
            h.push_str(&format!(
                "numbers not in audit: {}. ",
                esc(&n.unverified_numbers.join(", "))
            ));
        }
        if !n.effort_mismatches.is_empty() {
            h.push_str(&format!("effort mismatches: {}.", esc(&n.effort_mismatches.join("; "))));
        }
        h.push_str("</p>");
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Area, Finding, Severity};

    fn act(id: &str, effort: u8) -> crate::actions::Action {
        crate::actions::Action::new(id, 2, effort, "title", "e".into())
    }

    fn find() -> Finding {
        Finding {
            rule_id: "R10".into(),
            area: Area::OnPage,
            severity: Severity::Low,
            scope: "s".into(),
            evidence: "12 hits".into(),
            fix: "f".into(),
            kind: "fact".into(),
            observed_at: 1,
            source: "t".into(),
        }
    }

    #[test]
    fn unknown_ids_rejected() {
        let actions = vec![act("RULE-R10", 1)];
        assert_eq!(unknown_ids("see RULE-R99 now", &actions), vec!["RULE-R99"]);
        assert!(unknown_ids("see RULE-R10 now", &actions).is_empty());
    }

    #[test]
    fn unverified_flags_large_unknown_number() {
        let findings = vec![];
        let actions = vec![act("RULE-R10", 1)];
        let bad = unverified_numbers("Pass rate was 79.2% with 5000 pages.", &findings, &actions, 2);
        assert!(bad.iter().any(|t| t.starts_with("5000") || t == "5000"), "{bad:?}");
    }

    #[test]
    fn unverified_allows_small_ints_and_known_counts() {
        let findings = vec![find()];
        let actions = vec![];
        let bad = unverified_numbers("2 files and 12 hits passed.", &findings, &actions, 2);
        assert!(bad.is_empty(), "{bad:?}");
    }

    #[test]
    fn effort_band_mismatch_detected() {
        let actions = vec![act("RULE-R10", 1)];
        let n = Narrative {
            executive_summary: vec!["x".into()],
            strengths: vec!["s".into()],
            risks: vec!["r".into()],
            plan: vec![PlanBlock {
                horizon: "week".into(),
                items: vec!["RULE-R10 Fix title (a project)".into()],
            }],
            closing: None,
            author: None,
            automatic: false,
            unknown_ids: vec![],
            unverified_numbers: vec![],
            effort_mismatches: vec![],
        };
        let m = effort_mismatches(&n, &actions);
        assert_eq!(m.len(), 1);
        assert!(m[0].contains("hours") || m[0].contains("a project"));
    }

    #[test]
    fn auto_summary_labeled() {
        let n = auto(&[], &[], 4, 80.0);
        assert!(n.automatic);
        assert!(!n.executive_summary.is_empty());
    }
}
