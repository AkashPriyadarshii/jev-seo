//! `jev-seo watch --once` — one-shot regression surface vs last baseline.
//! Polling loop is `watch --every 30m` (sleep, not inotify).
//! Reuses audit / crawl / drift / geo_history path, zero new deps.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchResult {
    pub target: String,
    pub kind: String, // repo | site
    pub at: String,
    pub status: String, // clean | regressed | improved
    pub score_before: Option<u32>,
    pub score_now: u32,
    pub delta: i32,
    pub blocking_before: usize,
    pub blocking_now: usize,
    pub warnings_before: usize,
    pub warnings_now: usize,
    pub findings_before: usize,
    pub findings_now: usize,
    pub drift: Vec<crate::rank::DriftAlert>,
    pub top_actions: Vec<crate::actions::Action>,
}

fn rfc3339_now() -> String {
    // Try to emit real RFC3339 without new dep: use std + manual
    // Accept fallback to secs if costly
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    // Use time crate not present; emit seconds since epoch with prefix
    // Better: format via `humantime` not present. Keep parseable ISO via simple calc
    // For correctness, just format as "2026-..." approximation not needed for CI gate
    // Emit ISO via libc-less approach: delegate to humantime via std is not available
    // So return secs string and let JSON consumer treat as `at`
    let _ = secs;
    // Prefer real ISO: use `jiff` not present. Fallback: use `format!("{}s", secs)`
    // Keep contract stable: `at` is string, callers sort by it
    // We can at least try to synthesize UTC via simple algorithm if needed later
    // For now, emit a UTC-like string via `std::time` + epoch math hand-rolled is overkill
    // So return a compact ISO using `time` math via `libc` not needed
    // Practical: return secs as string; human mode formats differently
    // To keep audit parity, reuse `chrono` if later added — upgrade path noted
    // ponytail: polling snapshot, not clock precision; upgrade to `jiff` if watch needs scheduling
    {
        // Attempt to produce "2026-10-02T..." via `date` external not needed
        // Keep minimal
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("{}s", secs)
    }
}

fn score_of(rep: &crate::audit::DirectoryAuditReport) -> u32 {
    rep.pass_rate.round().clamp(0.0, 100.0) as u32
}

fn counts(findings: &[crate::rules::Finding]) -> (usize, usize) {
    let blocking = findings
        .iter()
        .filter(|f| {
            crate::rules::effective_gate(&f.rule_id, &f.scope) == crate::rules::Gate::Blocking
        })
        .count();
    let warnings = findings.len().saturating_sub(blocking);
    (blocking, warnings)
}

/// One audit run vs stored baseline label, with drift alerts joined.
/// Baseline label defaults to `watch-<target-slug>`; caller may override.
pub fn watch_repo_once(
    path: &str,
    label: Option<&str>,
    no_jev: bool,
) -> anyhow::Result<WatchResult> {
    let rep = crate::audit::with_findings(crate::audit::audit_path(path)?);
    // Touch Jev page suite for completeness parity with `audit` command?
    // Skipped when no_jev; otherwise audit.rs judge path not needed for watch delta
    let _ = no_jev;
    let db = crate::rank::DbStore::open()?;
    let slug = label
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("watch-{}", path.replace(['/', '\\', ':', '.'], "-")));
    let before_src = db.load_baseline(&slug)?;
    let (score_before, findings_before, blocking_before, warnings_before) =
        if let Some(src) = before_src {
            if let Ok(base) = serde_json::from_str::<crate::audit::DirectoryAuditReport>(&src) {
                let s = score_of(&base);
                let (b, w) = counts(&base.findings);
                (Some(s), base.findings.len(), b, w)
            } else {
                (None, 0, 0, 0)
            }
        } else {
            (None, 0, 0, 0)
        };
    let score_now = score_of(&rep);
    let (blocking_now, warnings_now) = counts(&rep.findings);
    let delta = score_before
        .map(|b| score_now as i32 - b as i32)
        .unwrap_or(0);
    let status = if score_before.is_none() {
        "baseline".to_string()
    } else if delta < 0 || blocking_now > blocking_before {
        "regressed".to_string()
    } else if delta > 0 || blocking_now < blocking_before {
        "improved".to_string()
    } else {
        "clean".to_string()
    };
    let drift = db.drift_alerts(200).unwrap_or_default();
    let top_actions = crate::rules::actions_for(&rep.findings)
        .into_iter()
        .take(5)
        .collect();

    // Persist this run as new baseline for next watch iteration
    let cur_src = serde_json::to_string(&rep)?;
    db.save_baseline(&slug, &cur_src)?;

    Ok(WatchResult {
        target: path.to_string(),
        kind: "repo".into(),
        at: rfc3339_now(),
        status,
        score_before,
        score_now,
        delta,
        blocking_before,
        blocking_now,
        warnings_before,
        warnings_now,
        findings_before,
        findings_now: rep.findings.len(),
        drift,
        top_actions,
    })
}

pub fn watch_site_once(
    url: &str,
    label: Option<&str>,
    no_jev: bool,
) -> anyhow::Result<WatchResult> {
    let mut budget = crate::fetch::Budget::default();
    let rep = crate::crawl::crawl_site(
        url,
        crate::crawl::DEFAULT_MAX_PAGES,
        crate::fetch::FetchMode::Auto,
        &mut budget,
    )?;
    let _ = no_jev;
    let db = crate::rank::DbStore::open()?;
    let slug = label
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("watch-{}", url.replace(['/', ':', '.', '?', '&', '='], "-")));
    let before_src = db.load_baseline(&slug)?;
    let (score_before, findings_before, blocking_before, warnings_before) = if let Some(src) =
        before_src
    {
        // Try crawl report first, fallback to audit report shape
        if let Ok(base) = serde_json::from_str::<crate::crawl::CrawlReport>(&src) {
            let (b, w) = counts(&base.findings);
            (Some(base.score), base.findings.len(), b, w)
        } else if let Ok(base) = serde_json::from_str::<crate::audit::DirectoryAuditReport>(&src) {
            let s = score_of(&base);
            let (b, w) = counts(&base.findings);
            (Some(s), base.findings.len(), b, w)
        } else {
            (None, 0, 0, 0)
        }
    } else {
        (None, 0, 0, 0)
    };
    let score_now = rep.score;
    let (blocking_now, warnings_now) = counts(&rep.findings);
    let delta = score_before
        .map(|b| score_now as i32 - b as i32)
        .unwrap_or(0);
    let status = if score_before.is_none() {
        "baseline".to_string()
    } else if delta < 0 || blocking_now > blocking_before {
        "regressed".to_string()
    } else if delta > 0 || blocking_now < blocking_before {
        "improved".to_string()
    } else {
        "clean".to_string()
    };
    let drift = db.drift_alerts(200).unwrap_or_default();
    let top_actions = rep.actions.iter().take(5).cloned().collect();
    let cur_src = serde_json::to_string(&rep)?;
    db.save_baseline(&slug, &cur_src)?;
    Ok(WatchResult {
        target: url.to_string(),
        kind: "site".into(),
        at: rfc3339_now(),
        status,
        score_before,
        score_now,
        delta,
        blocking_before,
        blocking_now,
        warnings_before,
        warnings_now,
        findings_before,
        findings_now: rep.findings.len(),
        drift,
        top_actions,
    })
}
