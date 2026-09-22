//! One place for every Jev confidence threshold. Tune numbers here, nowhere else.

/// Verdict for a semantic answer set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verdict {
    /// Confident. Print scores as facts.
    Act,
    /// Shaky. Print scores marked unverified.
    Flag,
    /// Unsure. Print local-only output, no Jev numbers.
    Drop,
}

pub struct Thresholds {
    pub act: f64,
    pub flag: f64,
}

/// Per-command thresholds. Higher stakes, higher bar.
pub fn thresholds(command: &str) -> Thresholds {
    match command {
        "geo" | "brief" => Thresholds { act: 0.75, flag: 0.5 },
        "query" | "keywords" => Thresholds { act: 0.6, flag: 0.4 },
        _ => Thresholds { act: 0.7, flag: 0.45 },
    }
}

pub fn gate(command: &str, confidence: f64) -> Verdict {
    let t = thresholds(command);
    if confidence >= t.act {
        Verdict::Act
    } else if confidence >= t.flag {
        Verdict::Flag
    } else {
        Verdict::Drop
    }
}

/// Short marker appended to Jev-derived lines when gated.
pub fn marker(v: Verdict) -> &'static str {
    match v {
        Verdict::Act => "",
        Verdict::Flag => " [verify]",
        Verdict::Drop => " [jev unsure]",
    }
}

/// Composite GEO dimensions and code-owned weights. Sums to 1.0.
pub const GEO_WEIGHTS: &[(&str, f64)] = &[
    ("geo_structure", 0.25),
    ("geo_density", 0.25),
    ("geo_directness", 0.20),
    ("geo_statistics", 0.20),
    ("geo_freshness", 0.10),
];

/// Five atomic Score questions (5 levels each, score 0-4) for one request.
pub fn geo_questions() -> serde_json::Value {
    serde_json::json!({
        "geo_structure": { "type": "score", "instructions": "How well is the content structured for citation (headings, lists, definitions)?",
            "criteria": ["Unstructured wall of text", "Some headings but unclear hierarchy", "Clear headings and sections", "Definitions, lists, and code blocks where needed", "Reference-grade structure throughout"] },
        "geo_density": { "type": "score", "instructions": "How fact-dense is the content versus filler?",
            "criteria": ["Mostly fluff and buzzwords", "Thin facts padded with prose", "Balanced facts and explanation", "Dense facts, minimal filler", "Every sentence carries citable information"] },
        "geo_directness": { "type": "score", "instructions": "Does the opening give a direct answer?",
            "criteria": ["No answer, pure preamble", "Answer buried deep", "Answer present but wordy", "Concise answer near the top", "Self-contained direct answer in the opening"] },
        "geo_statistics": { "type": "score", "instructions": "How well is the content backed by numbers, benchmarks, or proof?",
            "criteria": ["No evidence at all", "Vague claims without numbers", "Some numbers but no sources", "Benchmarks with context", "Empirical proof with sources throughout"] },
        "geo_freshness": { "type": "score", "instructions": "How current are the examples and references?",
            "criteria": ["Obsolete or dead references", "Dated examples", "Mostly current", "Current with versions noted", "Cutting-edge and verified current"] }
    })
}

/// Weighted 1-10 composite from extra answers. None when a dimension is missing.
pub fn composite_geo(extra: &serde_json::Map<String, serde_json::Value>) -> Option<(u32, f64)> {
    let mut total = 0.0;
    let mut conf_total = 0.0;
    for (id, weight) in GEO_WEIGHTS {
        let a = extra.get(*id)?;
        let score = a.get("score")?.as_f64()?;
        total += (score / 4.0) * weight;
        conf_total += a.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0) * weight;
    }
    Some((((total * 9.0) + 1.0).round().clamp(1.0, 10.0) as u32, conf_total))
}

/// Question ids in `extra` whose confidence sits below the act bar.
/// Code prints these as needs-review instead of silently trusting them.
/// A missing confidence also surfaces: unknown certainty is review-worthy.
pub fn needs_review(extra: &serde_json::Map<String, serde_json::Value>, command: &str) -> Vec<String> {
    let act = thresholds(command).act;
    let mut ids: Vec<String> = extra
        .iter()
        .filter(|(_, a)| {
            a.get("confidence")
                .and_then(|c| c.as_f64())
                .map(|c| c < act)
                .unwrap_or(true)
        })
        .map(|(k, _)| k.clone())
        .collect();
    ids.sort();
    ids
}

/// Next command hint from intent. Pure routing, no inference.
pub fn route_for_intent(intent: &str) -> &'static str {
    match intent {
        "transactional" | "commercial" => "rank / brief",
        "navigational" => "rank",
        _ => "geo / audit",
    }
}
