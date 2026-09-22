//! Ranked actions: every command ends with the same shape.
//! P1 fixes first, P3 last; ties break toward lower effort. Code-owned,
//! deterministic, no model calls.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Stable id, e.g. CRAWL-001.
    pub id: String,
    /// 1 = fix now, 2 = fix this week, 3 = polish.
    pub priority: u8,
    /// 1 = config or copy tweak, 2 = small code change, 3 = project.
    pub effort: u8,
    /// 0-100 impact score, derived from priority and effort.
    pub impact: u32,
    /// High impact at minimal effort. Do these first.
    pub quick_win: bool,
    pub title: String,
    pub evidence: String,
}

impl Action {
    pub fn new(id: &str, priority: u8, effort: u8, title: &str, evidence: String) -> Self {
        let impact = ((4 - priority.min(3)) as u32)
            .saturating_mul(30)
            .saturating_sub((effort.saturating_sub(1)) as u32 * 5);
        Self {
            id: id.to_string(),
            priority,
            effort,
            impact,
            quick_win: impact >= 55 && effort == 1,
            title: title.to_string(),
            evidence,
        }
    }
}

/// Letter grade on a 0-100 score.
pub fn grade(score: u32) -> &'static str {
    if score >= 90 {
        "A"
    } else if score >= 75 {
        "B"
    } else if score >= 60 {
        "C"
    } else if score >= 40 {
        "D"
    } else {
        "F"
    }
}

pub fn rank(mut actions: Vec<Action>) -> Vec<Action> {
    actions.sort_by(|a, b| {
        b.impact
            .cmp(&a.impact)
            .then(a.effort.cmp(&b.effort))
            .then(a.id.cmp(&b.id))
    });
    actions
}

pub fn top(actions: &[Action], n: usize) -> &[Action] {
    &actions[..actions.len().min(n)]
}
