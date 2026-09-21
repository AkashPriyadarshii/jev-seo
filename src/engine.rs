use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct JevClient {
    api_key: String,
    endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub intent: String,
    pub intent_confidence: f64,
    pub geo_score: u32,
    pub geo_confidence: f64,
    pub direct_answer: bool,
    pub direct_answer_p: f64,
    pub content_gap: String,
    pub gap_confidence: f64,
    /// Answers to command-specific questions, keyed by question id.
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl AnalysisResult {
    /// Gate on intent + geo only. Gap options often split probability across
    /// near-winners, which must not veto the headline score.
    pub fn confidence(&self) -> f64 {
        self.intent_confidence.min(self.geo_confidence)
    }
}

impl JevClient {
    pub fn new() -> Option<Self> {
        let key = std::env::var("TYPESAFE_API_KEY").ok()?;
        if key.trim().is_empty() {
            return None;
        }
        Some(Self {
            api_key: key,
            endpoint: "https://api.typesafe.ai/v1/systemone".to_string(),
        })
    }

    pub fn fanout_eval(&self, state: serde_json::Value) -> Result<AnalysisResult> {
        self.fanout_eval_with(state, serde_json::json!({}))
    }

    /// Pre-execution safety classifier for agent-driven file/URL tools.
    /// True = target looks like a secret, credential, or system path, block it.
    /// API failure fails open: the static path guard already ran.
    pub fn safety_block(&self, tool: &str, target: &str) -> bool {
        let payload = json!({
            "model": "jev-1.13.0",
            "state": { "tool": tool, "target": target },
            "questions": {
                "unsafe_target": {
                    "type": "noul",
                    "instructions": "Does `target` name a secret, credential, private key, token, password, system directory, or dot-file that the tool must not read?",
                }
            }
        });
        let body: serde_json::Value = match self
            .post(payload)
            .and_then(|r| r.into_json().map_err(anyhow::Error::from))
        {
            Ok(b) => b,
            Err(_) => return false,
        };
        body.get("answers")
            .and_then(|a| a.get("unsafe_target"))
            .and_then(|u| u.get("noul"))
            .and_then(|n| n.as_f64())
            .map(|p| p >= 0.7)
            .unwrap_or(false)
    }

    fn post(&self, payload: serde_json::Value) -> Result<ureq::Response> {
        ureq::post(&self.endpoint)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(12))
            .send_json(payload)
            .context("Failed to communicate with TypeSafe Jev API")
    }
    /// Same as fanout_eval plus command-specific questions merged into the one
    /// request. Answers land in `extra` for code to consume.
    pub fn fanout_eval_with(
        &self,
        state: serde_json::Value,
        extra_questions: serde_json::Value,
    ) -> Result<AnalysisResult> {
        let state = truncate_state(state);
        let mut questions = json!({
                "intent": {
                    "type": "choice",
                    "instructions": "Select the primary search intent.",
                    "criteria": {
                        "informational": "How-to, tutorial, explanation, documentation, research",
                        "commercial": "Product reviews, pricing comparisons, buying evaluation",
                        "transactional": "Immediate download, sign-up, purchase, command execution",
                        "navigational": "Specific brand, GitHub repo, or homepage search"
                    }
                },
                "geo_score": {
                    "type": "score",
                    "instructions": "Rate citation likelihood for generative search engines (Perplexity, SearchGPT, Gemini).",
                    "criteria": [
                        "Very low: promotional fluff, lacks concrete documentation or technical specifics",
                        "Low: shallow overview, missing practical code examples or proof",
                        "Moderate: helpful technical details but lacks authoritative benchmark or structured layout",
                        "High: clear, structured, copy-pasteable commands and direct factual definitions",
                        "Exceptional: comprehensive authoritative reference, zero fluff, perfect citation density"
                    ]
                },
                "direct_answer": {
                    "type": "noul",
                    "instructions": "Does the content provide a direct, concise factual answer or code example in the opening section?",
                    "criteria": {
                        "true": "Content begins with a direct definition, quickstart command, or concise answer",
                        "false": "Content rambles, buries the solution, or lacks concrete code"
                    }
                },
                "content_gap": {
                    "type": "choice",
                    "instructions": "What is the primary weakness or missing angle?",
                    "criteria": {
                        "missing_statistics": "Lacks empirical benchmarks, numbers, or proof",
                        "generic_prose": "AI-slop, superficial fluff, empty buzzwords",
                        "no_step_by_step": "Missing practical reproduction steps or code blocks",
                        "outdated_examples": "Obsolete APIs or dead references",
                        "none": "Satisfies user query with high information density"
                    }
                }
        });
        if let Some(map) = questions.as_object_mut() {
            if let Some(extra) = extra_questions.as_object() {
                for (k, v) in extra {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
        let payload = json!({
            "model": "jev-1.13.0",
            "state": state,
            "questions": questions
        });

        let resp = self.post(payload)?;

        let body: serde_json::Value = resp.into_json()?;
        let answers = body.get("answers").context("Invalid Jev response schema")?;

        let intent_obj = &answers["intent"];
        let intent = intent_obj["choice"]
            .as_str()
            .context("Jev response missing answers.intent.choice")?
            .to_string();
        let intent_confidence = intent_obj["confidence"]
            .as_f64()
            .context("Jev response missing answers.intent.confidence")?;

        let geo_obj = &answers["geo_score"];
        let geo_val = geo_obj["score"]
            .as_f64()
            .context("Jev response missing answers.geo_score.score")?;
        let geo_confidence = geo_obj["confidence"].as_f64().unwrap_or(0.0);
        let geo_score = ((geo_val / 4.0 * 9.0) + 1.0).round().clamp(1.0, 10.0) as u32;

        let direct_obj = &answers["direct_answer"];
        let direct_answer_p = direct_obj["noul"]
            .as_f64()
            .or_else(|| direct_obj["probability"].as_f64())
            .context("Jev response missing answers.direct_answer.noul")?;
        let direct_answer = direct_answer_p >= 0.5;

        let gap_obj = &answers["content_gap"];
        let content_gap = gap_obj["choice"]
            .as_str()
            .context("Jev response missing answers.content_gap.choice")?
            .to_string();
        let gap_confidence = gap_obj["confidence"].as_f64().unwrap_or(0.0);

        Ok(AnalysisResult {
            intent,
            intent_confidence,
            geo_score,
            geo_confidence,
            direct_answer,
            direct_answer_p,
            content_gap,
            gap_confidence,
            extra: answers
                .as_object()
                .map(|m| {
                    m.iter()
                        .filter(|(k, _)| {
                            !["intent", "geo_score", "direct_answer", "content_gap"]
                                .contains(&k.as_str())
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

/// Cap every string in the state so oversized pages never 400 the API.
fn truncate_state(value: serde_json::Value) -> serde_json::Value {
    const LIMIT: usize = 8000;
    match value {
        serde_json::Value::String(s) => {
            if s.len() > LIMIT {
                let cut = s.floor_char_boundary(LIMIT);
                serde_json::Value::String(format!("{}…[truncated]", &s[..cut]))
            } else {
                serde_json::Value::String(s)
            }
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(truncate_state).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter().map(|(k, v)| (k, truncate_state(v))).collect(),
        ),
        other => other,
    }
}
