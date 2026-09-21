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
