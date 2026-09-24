//! RunManifest: one frozen contract every writer and agent reads.
//!
//! `run.json` + `ledger.json` land next to exports. Score surfaces print a
//! completeness banner. Text gates refuse unknown RULE-*/action IDs before a
//! report is written. Deterministic; no model calls.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

pub const SCHEMA_VERSION: &str = "1.0";
/// TypeSafe list price, USD per million input tokens (docs.typesafe.ai).
pub const JEV_USD_PER_MTOK: f64 = 0.042;
/// Default hard spend cap when the CLI does not pass `--jev-budget`.
pub const DEFAULT_JEV_BUDGET_USD: f64 = 0.25;

pub static JEV_REQUESTS: AtomicU32 = AtomicU32::new(0);
pub static JEV_INPUT_TOKENS: AtomicU64 = AtomicU64::new(0);
pub static JEV_FAILED: AtomicU32 = AtomicU32::new(0);
/// Budget in micro-USD (u64) so we can compare without floats in the hot path.
static JEV_BUDGET_MICROUSD: AtomicU64 = AtomicU64::new((DEFAULT_JEV_BUDGET_USD * 1_000_000.0) as u64);
static JEV_SKIPPED_BUDGET: AtomicU32 = AtomicU32::new(0);

pub fn set_jev_budget_usd(usd: f64) {
    let micro = (usd.max(0.0) * 1_000_000.0) as u64;
    JEV_BUDGET_MICROUSD.store(micro, Ordering::Relaxed);
}

pub fn jev_budget_usd() -> f64 {
    JEV_BUDGET_MICROUSD.load(Ordering::Relaxed) as f64 / 1_000_000.0
}

/// True when posting `est_tokens` more would exceed the hard USD cap.
/// Spent tokens so far + reservation estimate, at list price.
pub fn jev_budget_exhausted(est_tokens: u64) -> bool {
    let spent = JEV_INPUT_TOKENS.load(Ordering::Relaxed) + est_tokens;
    let micro = (spent as f64 / 1_000_000.0 * JEV_USD_PER_MTOK * 1_000_000.0).ceil() as u64;
    micro > JEV_BUDGET_MICROUSD.load(Ordering::Relaxed)
}

pub fn note_budget_skip() {
    JEV_SKIPPED_BUDGET.fetch_add(1, Ordering::Relaxed);
}

/// What the score could and could not see. Always print with a score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Completeness {
    /// True only when no known gap applies.
    pub full: bool,
    /// Human lines: sample caps, missing keys, robots, seed source.
    pub notes: Vec<String>,
}

impl Completeness {
    pub fn banner(&self) -> String {
        if self.full && self.notes.is_empty() {
            "Completeness:  full".to_string()
        } else {
            let head = if self.full { "full" } else { "partial" };
            format!("Completeness:  {} ({})", head, self.notes.join("; "))
        }
    }
}

/// Spend and access for one run. Free paths stay at zero.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ledger {
    pub schema_version: String,
    pub command: String,
    pub target: String,
    /// TypeSafe key present for this run (value never stored).
    pub jev_key_present: bool,
    pub jev_requests: u32,
    pub jev_input_tokens: u64,
    pub jev_failed: u32,
    pub jev_cost_usd: f64,
    pub jev_budget_usd: f64,
    pub jev_skipped_budget: u32,
    /// Resolved model build that served (jev-latest is an alias; thresholds
    /// couple to the versioned build). Unknown when no Jev call ran.
    #[serde(default)]
    pub jev_model: String,
    /// Question-set build that produced the scores (policy::QUESTION_VERSION).
    #[serde(default)]
    pub jev_question_version: String,
    pub fetch_credits_spent: u32,
    pub fetch_credit_cap: u32,
    pub paid_backends_used: Vec<String>,
    pub notes: Vec<String>,
}

impl Ledger {
    pub fn new(command: &str, target: &str) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            command: command.into(),
            target: target.into(),
            jev_key_present: jev_key_present(),
            jev_requests: JEV_REQUESTS.load(Ordering::Relaxed),
            jev_input_tokens: JEV_INPUT_TOKENS.load(Ordering::Relaxed),
            jev_failed: JEV_FAILED.load(Ordering::Relaxed),
            jev_cost_usd: jev_cost_usd(JEV_INPUT_TOKENS.load(Ordering::Relaxed)),
            jev_budget_usd: jev_budget_usd(),
            jev_skipped_budget: jev_budget_skip_count(),
            jev_model: crate::engine::jev_model().to_string(),
            jev_question_version: crate::policy::QUESTION_VERSION.to_string(),
            fetch_credits_spent: 0,
            fetch_credit_cap: 0,
            paid_backends_used: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_fetch(&mut self, spent: u32, cap: u32, backends: Vec<String>) -> &mut Self {
        self.fetch_credits_spent = spent;
        self.fetch_credit_cap = cap;
        self.paid_backends_used = backends;
        self
    }

    pub fn note(&mut self, s: impl Into<String>) -> &mut Self {
        self.notes.push(s.into());
        self
    }
}

/// IDs and facts that exist in this run. Writers and narratives cite only these.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CitationSet {
    pub action_ids: BTreeSet<String>,
    pub rule_ids: BTreeSet<String>,
    pub scores: BTreeSet<i64>,
    pub counts: BTreeSet<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: String,
    #[serde(default)]
    pub rule_set_version: String,
    pub tool: ToolMeta,
    pub run: RunMeta,
    pub target: String,
    pub command: String,
    pub score: Option<ScoreCard>,
    pub completeness: Completeness,
    pub citations: CitationsOut,
    pub ledger: Ledger,
    pub validation: ValidationReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMeta {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMeta {
    pub finished_at_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCard {
    /// 0-100 as presented to the user (pass rate, health, or readiness).
    pub score: u32,
    pub grade: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsOut {
    pub action_ids: Vec<String>,
    pub rule_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub ok: bool,
    pub unknown_action_ids: Vec<String>,
    pub unknown_rule_ids: Vec<String>,
    pub unknown_citations_in_text: Vec<String>,
    pub checked_texts: usize,
}

pub fn jev_key_present() -> bool {
    std::env::var("TYPESAFE_API_KEY").map(|k| !k.trim().is_empty()).unwrap_or(false)
}

pub fn jev_cost_usd(input_tokens: u64) -> f64 {
    (input_tokens as f64 / 1_000_000.0) * JEV_USD_PER_MTOK
}

/// Ledger field for budget skips this process (filled at snapshot time).
pub fn jev_budget_skip_count() -> u32 {
    JEV_SKIPPED_BUDGET.load(Ordering::Relaxed)
}

fn now_unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Every action id produced by the rule engine for these findings.
#[allow(dead_code)]
pub fn action_ids_for(findings: &[crate::rules::Finding]) -> Vec<String> {
    crate::rules::actions_for(findings).into_iter().map(|a| a.id).collect()
}

/// Well-formed action ids only (RULE-R01, LLMS-001, CRAWL-001, ...).
pub fn action_id_shape_ok(id: &str) -> bool {
    if id.is_empty() || id.len() > 40 {
        return false;
    }
    // Pattern: ALNUM segments joined by '-', last segment has a digit.
    let mut parts = id.split('-').peekable();
    if parts.peek().is_none() {
        return false;
    }
    let mut saw_digit_tail = false;
    for (i, part) in id.split('-').enumerate() {
        if part.is_empty() {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
        if i == 0 && !part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return false;
        }
        if part.chars().any(|c| c.is_ascii_digit()) && i > 0 {
            saw_digit_tail = true;
        }
    }
    saw_digit_tail || id.starts_with("RULE-")
}

pub fn known_rule_id(id: &str) -> bool {
    crate::rules::rule(id).is_some()
}

#[allow(dead_code)]
pub fn known_action_id(id: &str, actions: &[crate::actions::Action]) -> bool {
    actions.iter().any(|a| a.id == id)
}

/// Scan free text for citation tokens: RULE-Rxx, bare R01-R53, PREFIX-NNN.
pub fn extract_citations(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '-') {
            i += 1;
        }
        let token: String = bytes[start..i].iter().collect();
        let trimmed = token.trim_matches('-');
        if trimmed.is_empty() {
            continue;
        }
        // RULE-R01 style or PREFIX-001 / PREFIX-01 with digits at end.
        let looks = (trimmed.starts_with("RULE-R") && trimmed.len() == 8)
            || (trimmed.len() >= 4
                && trimmed.contains('-')
                && trimmed
                    .rsplit('-')
                    .next()
                    .map(|tail| tail.len() >= 2 && tail.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
                && trimmed.split('-').next().map(|h| h.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())).unwrap_or(false))
            || (trimmed.len() == 3
                && trimmed.starts_with('R')
                && trimmed[1..].chars().all(|c| c.is_ascii_digit()));
        if looks {
            out.insert(trimmed.to_string());
        }
    }
    out
}

fn normalize_citation(tok: &str) -> String {
    // RULE-R01 -> R01 for rule lookup; keep full for action lookup.
    if let Some(rest) = tok.strip_prefix("RULE-") {
        return rest.to_string();
    }
    tok.to_string()
}

/// Validate findings, actions, and any report text against the known sets.
pub fn validate(
    findings: &[crate::rules::Finding],
    actions: &[crate::actions::Action],
    texts: &[&str],
) -> Result<ValidationReport> {
    let report = validate_core(findings, actions, texts);
    if !report.ok {
        bail!(
            "citation gate failed: unknown rules {:?}, unknown actions {:?}, unknown in text {:?}",
            report.unknown_rule_ids,
            report.unknown_action_ids,
            report.unknown_citations_in_text
        );
    }
    Ok(report)
}

fn validate_core(
    findings: &[crate::rules::Finding],
    actions: &[crate::actions::Action],
    texts: &[&str],
) -> ValidationReport {
    let mut unknown_rules = Vec::new();
    let mut unknown_actions = Vec::new();
    let mut unknown_in_text = Vec::new();

    for f in findings {
        if !known_rule_id(&f.rule_id) && !unknown_rules.contains(&f.rule_id) {
            unknown_rules.push(f.rule_id.clone());
        }
    }
    for a in actions {
        let bad_shape = !action_id_shape_ok(&a.id);
        let bad_rule =
            a.id.starts_with("RULE-") && !known_rule_id(a.id.trim_start_matches("RULE-"));
        if (bad_shape || bad_rule) && !unknown_actions.contains(&a.id) {
            if bad_shape {
                unknown_actions.push(format!("{} (bad shape)", a.id));
            } else {
                unknown_actions.push(a.id.clone());
            }
        }
    }

    let action_set: BTreeSet<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    let rule_set: BTreeSet<&str> = findings.iter().map(|f| f.rule_id.as_str()).collect();

    for text in texts {
        for tok in extract_citations(text) {
            let norm = normalize_citation(&tok);
            let bad = if tok.starts_with("RULE-") {
                !action_set.contains(tok.as_str())
            } else if norm.len() == 3 && norm.starts_with('R') {
                !known_rule_id(&norm) && !rule_set.contains(norm.as_str())
            } else if action_id_shape_ok(&tok) {
                let head = tok.split('-').next().unwrap_or("");
                matches!(head, "LLMS" | "CRAWL" | "ACT" | "JEV" | "SEO")
                    && !action_set.contains(tok.as_str())
            } else {
                false
            };
            if bad && !unknown_in_text.contains(&tok) {
                unknown_in_text.push(tok);
            }
        }
    }

    ValidationReport {
        ok: unknown_rules.is_empty() && unknown_actions.is_empty() && unknown_in_text.is_empty(),
        unknown_action_ids: unknown_actions,
        unknown_rule_ids: unknown_rules,
        unknown_citations_in_text: unknown_in_text,
        checked_texts: texts.len(),
    }
}

/// Soft variant: returns the report without bailing (for embed-in-manifest).
pub fn validate_report(
    findings: &[crate::rules::Finding],
    actions: &[crate::actions::Action],
    texts: &[&str],
) -> ValidationReport {
    validate_core(findings, actions, texts)
}

fn citations(findings: &[crate::rules::Finding], actions: &[crate::actions::Action]) -> CitationsOut {
    let rules: BTreeSet<String> = findings.iter().map(|f| f.rule_id.clone()).collect();
    let acts: BTreeSet<String> = actions.iter().map(|a| a.id.clone()).collect();
    CitationsOut {
        action_ids: acts.into_iter().collect(),
        rule_ids: rules.into_iter().collect(),
    }
}

pub struct ManifestInput<'a> {
    pub command: &'a str,
    pub target: &'a str,
    pub findings: &'a [crate::rules::Finding],
    pub actions: &'a [crate::actions::Action],
    pub texts: &'a [&'a str],
    pub score: Option<ScoreCard>,
    pub completeness: Completeness,
    pub ledger: Ledger,
}

impl<'a> ManifestInput<'a> {
    pub fn build(self) -> RunManifest {
        let validation = validate_report(self.findings, self.actions, self.texts);
        RunManifest {
            schema_version: SCHEMA_VERSION.into(),
            rule_set_version: crate::rules::RULE_SET_VERSION.into(),
            tool: ToolMeta {
                name: "jev-seo".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            run: RunMeta {
                finished_at_unix_ms: now_unix_ms(),
            },
            target: self.target.into(),
            command: self.command.into(),
            score: self.score,
            completeness: self.completeness,
            citations: citations(self.findings, self.actions),
            ledger: self.ledger,
            validation,
        }
    }
}

/// Hard gate: refuse to write a report whose citations are unknown.
pub fn gate_report(text: &str, findings: &[crate::rules::Finding], actions: &[crate::actions::Action]) -> Result<()> {
    validate(findings, actions, &[text]).map(|_| ())
}

/// Directory that should hold run.json / ledger.json for an export path.
fn export_dir(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Write run.json and ledger.json next to an export (or into `dir`).
pub fn write_pair(dir: &Path, manifest: &RunManifest) -> Result<(PathBuf, PathBuf)> {
    std::fs::create_dir_all(dir)?;
    let run_path = dir.join("run.json");
    let ledger_path = dir.join("ledger.json");
    let run_json = serde_json::to_string_pretty(manifest).context("serialize run.json")?;
    let ledger_json = serde_json::to_string_pretty(&manifest.ledger).context("serialize ledger.json")?;
    std::fs::write(&run_path, run_json)?;
    std::fs::write(&ledger_path, ledger_json)?;
    Ok((run_path, ledger_path))
}

/// Derive companion manifest paths from the first export the user asked for.
pub fn auto_dirs(exports: &[Option<&str>]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    for p in exports.iter().flatten() {
        let d = export_dir(Path::new(p));
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    dirs
}

pub fn completeness_audit(files: usize, capped_hint: bool, jev: bool) -> Completeness {
    let mut notes = Vec::new();
    if capped_hint {
        notes.push("local tree sampled by walker".into());
    }
    if !jev {
        notes.push("Jev not assessed (no TYPESAFE_API_KEY or not requested)".into());
    }
    notes.push(format!("{} files walked", files));
    Completeness {
        full: !capped_hint && jev,
        notes,
    }
}

pub fn completeness_crawl(rep: &crate::crawl::CrawlReport, max_pages: usize) -> Completeness {
    let mut notes = Vec::new();
    notes.push(format!(
        "seed {}",
        if rep.seeded_from_sitemap { "sitemap" } else { "start-URL" }
    ));
    notes.push(format!(
        "robots {}",
        if rep.robots_honored { "honored" } else { "missing" }
    ));
    notes.push(format!("{} pages", rep.pages_crawled));
    if rep.capped {
        notes.push(format!("capped at {} (raise --max-pages)", max_pages.min(rep.pages_crawled.max(max_pages))));
        notes.push("sample, not full-site verdict".into());
    }
    if !rep.errors.is_empty() {
        notes.push(format!("{} fetch errors", rep.errors.len()));
    }
    let full = !rep.capped && rep.errors.is_empty() && rep.robots_honored && rep.seeded_from_sitemap;
    Completeness { full, notes }
}

pub fn completeness_llms(rep: &crate::llms::LlmsReport) -> Completeness {
    let mut notes = Vec::new();
    notes.push(format!("llms.txt {}", if rep.info.present { "present" } else { "missing" }));
    notes.push(format!(
        "robots {}",
        if rep.robots_present { "present" } else { "missing" }
    ));
    notes.push("live single-domain check".into());
    Completeness {
        full: rep.info.present && rep.robots_present,
        notes,
    }
}

/// Directory holding the local DB; the eval log lives beside it.
fn jev_home_dir() -> Option<std::path::PathBuf> {
    if let Ok(db) = std::env::var("JEV_SEO_DB") {
        return std::path::Path::new(&db)
            .parent()
            .map(|p| p.to_path_buf())
            .filter(|p| !p.as_os_str().is_empty());
    }
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".jev-seo")
    })
}

/// Append-only eval trace: one JSON line per Jev fan-out (question version,
/// model, input size, full answers). Powers re-evaluation and Act sampling.
/// Best-effort: never fails a run. Rotates past 5 MB.
pub fn append_eval_log(entry: serde_json::Value) {
    const CAP_BYTES: u64 = 5 * 1024 * 1024;
    let Some(dir) = jev_home_dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("eval.jsonl");
    if path.metadata().map(|m| m.len() > CAP_BYTES).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", entry);
    }
}

/// Terminal banner line (stderr-safe to print on stdout after scores).
pub fn print_banner(c: &Completeness) {
    println!("  {}", c.banner());
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    use crate::rules::{Area, Finding, Severity};

    fn finding(rule: &str) -> Finding {
        let r = crate::rules::rule(rule).expect("rule");
        Finding {
            rule_id: rule.into(),
            area: r.area,
            severity: r.severity,
            scope: "s".into(),
            evidence: "e".into(),
            fix: r.fix.into(),
            kind: "fact".into(),
            observed_at: 1,
            source: "t".into(),
        }
    }

    #[test]
    fn schema_version_frozen() {
        assert_eq!(SCHEMA_VERSION, "1.0");
    }

    #[test]
    fn action_id_shape() {
        assert!(action_id_shape_ok("RULE-R01"));
        assert!(action_id_shape_ok("LLMS-001"));
        assert!(!action_id_shape_ok(""));
        assert!(!action_id_shape_ok("RULE-"));
        assert!(!action_id_shape_ok("no-digits"));
    }

    #[test]
    fn known_rules() {
        assert!(known_rule_id("R01"));
        assert!(!known_rule_id("R99"));
    }

    #[test]
    fn extract_citations_finds_tokens() {
        let t = "Fix RULE-R01 and R42; ignore LLMS-001 here.";
        let c = extract_citations(t);
        assert!(c.contains("RULE-R01") || c.contains("R01"));
        assert!(c.contains("R42"));
        assert!(c.contains("LLMS-001"));
    }

    #[test]
    fn validate_accepts_real_findings() {
        let findings = vec![finding("R01")];
        let actions = crate::rules::actions_for(&findings);
        let text = format!("Top action {} targets the broken link.", actions[0].id);
        let r = validate(&findings, &actions, &[&text]).expect("should pass");
        assert!(r.ok);
        assert_eq!(r.unknown_rule_ids.len(), 0);
    }

    #[test]
    fn validate_rejects_unknown_action_in_text() {
        let findings = vec![finding("R01")];
        let actions = crate::rules::actions_for(&findings);
        let bad = "Narrative cites RULE-R99 which does not exist.";
        let err = validate(&findings, &actions, &[bad]).expect_err("must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("citation gate failed"), "{msg}");
    }

    #[test]
    fn validate_rejects_unknown_rule_finding() {
        let findings = vec![Finding {
            rule_id: "R99".into(),
            area: Area::Crawl,
            severity: Severity::Low,
            scope: "x".into(),
            evidence: "y".into(),
            fix: "z".into(),
            kind: "fact".into(),
            observed_at: 1,
            source: "t".into(),
        }];
        let actions = vec![];
        assert!(validate(&findings, &actions, &[]).is_err());
    }

    #[test]
    fn ledger_cost_math() {
        assert_eq!(jev_cost_usd(0), 0.0);
        let c = jev_cost_usd(1_000_000);
        assert!((c - JEV_USD_PER_MTOK).abs() < 1e-12);
    }

    #[test]
    fn manifest_builds_and_serializes() {
        let findings = vec![finding("R05"), finding("R06")];
        let actions = crate::rules::actions_for(&findings);
        let text = format!("See {} for sitemap.", actions[0].id);
        let ledger = Ledger::new("crawl", "https://x.test/");
        let m = ManifestInput {
            command: "crawl",
            target: "https://x.test/",
            findings: &findings,
            actions: &actions,
            texts: &[&text],
            score: Some(ScoreCard {
                score: 88,
                grade: "B".into(),
                kind: "health".into(),
            }),
            completeness: Completeness {
                full: false,
                notes: vec!["capped".into()],
            },
            ledger,
        }
        .build();
        assert_eq!(m.schema_version, "1.0");
        assert!(m.validation.ok);
        assert!(!m.citations.action_ids.is_empty());
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"schema_version\":\"1.0\"") || json.contains("\"schema_version\": \"1.0\""));
        assert!(json.contains("\"ledger\""));
    }

    #[test]
    fn gate_blocks_unknown_citation() {
        let findings = vec![finding("R01")];
        let actions = crate::rules::actions_for(&findings);
        assert!(gate_report("ok with RULE-R01", &findings, &actions).is_ok());
        assert!(gate_report("bad JEV-999", &findings, &actions).is_err());
    }

    #[test]
    fn completeness_banners() {
        let full = Completeness { full: true, notes: vec![] };
        assert_eq!(full.banner(), "Completeness:  full");
        let part = Completeness {
            full: false,
            notes: vec!["capped at 50".into()],
        };
        assert!(part.banner().contains("partial"));
        assert!(part.banner().contains("capped at 50"));
    }

    #[test]
    fn write_pair_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let findings = vec![finding("R01")];
        let actions = crate::rules::actions_for(&findings);
        let m = ManifestInput {
            command: "audit",
            target: "docs/",
            findings: &findings,
            actions: &actions,
            texts: &[],
            score: None,
            completeness: Completeness { full: true, notes: vec![] },
            ledger: Ledger::new("audit", "docs/"),
        }
        .build();
        let (run_p, led_p) = write_pair(dir.path(), &m).unwrap();
        assert!(run_p.exists() && led_p.exists());
        let back: RunManifest = serde_json::from_str(&std::fs::read_to_string(&run_p).unwrap()).unwrap();
        assert_eq!(back.schema_version, SCHEMA_VERSION);
        let led: Ledger = serde_json::from_str(&std::fs::read_to_string(&led_p).unwrap()).unwrap();
        assert_eq!(led.command, "audit");
    }
}
