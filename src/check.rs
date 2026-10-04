//! `jev-seo check` — one-command search CI. Policy + audit scope +
//! baseline compare + exit code. SARIF/JUnit writers for GitHub.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub path: String,
    pub pass_rate: f64,
    pub blocking: usize,
    pub critical: usize,
    pub verdict: String,
    pub findings: Vec<CheckFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckFinding {
    pub rule: String,
    pub target: String,
    pub severity: String,
    pub gate: String,
}

/// SARIF 2.1.0 minimal: one run, one driver, results carry ruleId + level.
pub fn to_sarif(tool: &str, version: &str, findings: &[CheckFinding]) -> String {
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "ruleId": f.rule,
                "level": if f.gate == "blocking" { "error" } else { "warning" },
                "message": { "text": format!("{} on {}", f.rule, f.target) },
                "locations": [{ "physicalLocation": {
                    "artifactLocation": { "uri": f.target },
                }}],
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{ "tool": { "driver": { "name": tool, "version": version } },
            "results": results }],
    }))
    .unwrap_or_default()
}

/// JUnit minimal: one testsuite, blocking findings are failures.
pub fn to_junit(suite: &str, findings: &[CheckFinding]) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\">\n",
        xml_escape(suite),
        findings.len(),
        findings.iter().filter(|f| f.gate == "blocking").count(),
    );
    for f in findings {
        out.push_str(&format!(
            "  <testcase classname=\"search\" name=\"{}-{}\">\n",
            xml_escape(&f.rule),
            xml_escape(&f.target)
        ));
        if f.gate == "blocking" {
            out.push_str(&format!(
                "    <failure message=\"{} ({})\"/>\n",
                xml_escape(&f.rule),
                xml_escape(&f.severity)
            ));
        }
        out.push_str("  </testcase>\n");
    }
    out.push_str("</testsuite>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sarif_shape() {
        let f = vec![CheckFinding {
            rule: "R09".into(),
            target: "docs/a.md".into(),
            severity: "High".into(),
            gate: "blocking".into(),
        }];
        let s: serde_json::Value = serde_json::from_str(&to_sarif("jev-seo", "0.4.0", &f)).unwrap();
        assert_eq!(s["version"], "2.1.0");
        assert_eq!(s["runs"][0]["results"][0]["level"], "error");
    }
}
