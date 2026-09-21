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
    pub direct_answer: bool,
    pub direct_answer_p: f64,
    pub content_gap: String,
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
        let payload = json!({
            "model": "jev-1.13.0",
            "state": state,
            "questions": {
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
            }
        });

        let resp = ureq::post(&self.endpoint)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(12))
            .send_json(payload)
            .context("Failed to communicate with TypeSafe Jev API")?;

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
        let geo_score = ((geo_val + 1.0) * 2.0).round().clamp(1.0, 10.0) as u32;

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

        Ok(AnalysisResult {
            intent,
            intent_confidence,
            geo_score,
            direct_answer,
            direct_answer_p,
            content_gap,
        })
    }
}
