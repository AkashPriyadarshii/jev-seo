//! Portable evidence snapshots: `jev-evidence/` JSONL per run. Every row
//! carries provider/engine/location/timestamp/hash/ruleset/cost/confidence.
//! Files, not SQLite: no migration risk to existing drift baselines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Observation {
    pub kind: String,
    pub provider: String,
    pub engine: String,
    pub country: String,
    pub language: String,
    pub device: String,
    pub observed_at: u64,
    pub input_hash: String,
    pub ruleset: String,
    pub cost_usd: f64,
    pub confidence: f64,
    pub payload: serde_json::Value,
}

/// Stable u64 hash of an input string (FNV-1a, stdlib only).
#[allow(dead_code)]
pub fn input_hash(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

/// Append one observation line to `jev-evidence/<kind>.jsonl` under dir.
#[allow(dead_code)]
pub fn append(dir: &str, kind: &str, obs: &Observation) -> anyhow::Result<()> {
    std::fs::create_dir_all(format!("{dir}/jev-evidence"))?;
    let line = serde_json::to_string(obs)?;
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/jev-evidence/{kind}.jsonl"))?;
    writeln!(f, "{line}")?;
    Ok(())
}
