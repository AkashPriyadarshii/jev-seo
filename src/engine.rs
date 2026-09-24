use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::Ordering;

/// Alias from docs.typesafe.ai; resolves to the active production model.
pub const MODEL: &str = "jev-latest";

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

/// Single deduped page state: one text field only. The old `content` +
/// `page.text` duplication sent the same head markup twice and halved useful
/// context; callers build state through this and add only small extras.
pub fn page_state(
    query: &str,
    title: Option<String>,
    description: Option<String>,
    text: String,
    word_count: usize,
) -> serde_json::Value {
    json!({
        "query": query,
        "page": {
            "title": title,
            "description": description,
            "text": text,
            "word_count": word_count
        }
    })
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

    #[allow(dead_code)]
    pub fn fanout_eval(&self, state: serde_json::Value) -> Result<AnalysisResult> {
        self.fanout_eval_with(state, serde_json::json!({}))
    }

    /// Pre-execution safety classifier for agent-driven file/URL tools.
    /// True = target looks like a secret, credential, or system path, block it.
    /// None = the check itself failed (transport error, bad schema): callers
    /// must fail closed, because an unreachable guard is not a clean bill.
    pub fn safety_block(&self, tool: &str, target: &str) -> Option<bool> {
        let payload = json!({
            "model": MODEL,
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
            Err(_) => return None,
        };
        record_usage(&body);
        body.get("answers")
            .and_then(|a| a.get("unsafe_target"))
            .and_then(|u| u.get("noul"))
            .and_then(|n| n.as_f64())
            .map(|p| p >= 0.7)
    }

    fn post(&self, payload: serde_json::Value) -> Result<ureq::Response> {
        let body = payload.to_string();
        // Hard spend cap before dispatch: estimate tokens from request size.
        let est_tokens = (body.len() / 3) as u64;
        if crate::manifest::jev_budget_exhausted(est_tokens) {
            crate::manifest::note_budget_skip();
            return Err(anyhow::anyhow!(
                "Jev budget cap reached (${:.4}); raise --jev-budget to continue",
                crate::manifest::jev_budget_usd()
            ));
        }
        // Transient 429/5xx get two more tries with backoff. Every attempt
        // counts as a request: the ledger bills attempts, not wishes.
        let waits = [500u64, 1500u64];
        for attempt in 0..=waits.len() {
            let raw = ureq::post(&self.endpoint)
                .set("Authorization", &format!("Bearer {}", self.api_key))
                .set("Content-Type", "application/json")
                .timeout(std::time::Duration::from_secs(12))
                .send_string(&body);
            let retryable = matches!(&raw, Err(ureq::Error::Status(code, _)) if *code == 429 || (500..=599).contains(code));
            let resp = raw.context("Failed to communicate with TypeSafe Jev API");
            match &resp {
                Ok(_) => {
                    crate::manifest::JEV_REQUESTS.fetch_add(1, Ordering::Relaxed);
                    return resp;
                }
                Err(_) => {
                    crate::manifest::JEV_REQUESTS.fetch_add(1, Ordering::Relaxed);
                    crate::manifest::JEV_FAILED.fetch_add(1, Ordering::Relaxed);
                }
            }
            if retryable {
                if let Some(ms) = waits.get(attempt) {
                    std::thread::sleep(std::time::Duration::from_millis(*ms));
                    continue;
                }
            }
            return resp;
        }
        unreachable!("retry loop always returns")
    }
    /// Same as fanout_eval plus command-specific questions merged into the one
    /// request. Answers land in `extra` for code to consume.
    /// State is pre-filtered (skill: strip junk) then truncated.
    pub fn fanout_eval_with(
        &self,
        state: serde_json::Value,
        extra_questions: serde_json::Value,
    ) -> Result<AnalysisResult> {
        let state = truncate_state(prefilter_state(state));
        let input_chars = state.to_string().len();
        let mut questions = json!({
                "intent": {
                    "type": "choice",
                    "instructions": "Select the primary search intent.",
                    "criteria": {
                        "informational": "How-to, tutorial, explanation, documentation, research",
                        "commercial": "Product reviews, pricing comparisons, buying evaluation",
                        "transactional": "Immediate download, sign-up, purchase, command execution",
                        "navigational": "Specific brand, GitHub repo, or homepage search",
                        "insufficient_context": "Supplied evidence is too thin to choose safely"
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
                        "none": "Satisfies user query with high information density",
                        "insufficient_context": "Too little content to judge a weakness"
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
            "model": MODEL,
            "state": state,
            "questions": questions
        });

        let resp = self.post(payload)?;

        let body: serde_json::Value = resp.into_json()?;
        record_usage(&body);
        let answers = body.get("answers").context("Invalid Jev response schema")?;
        let ts_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        crate::manifest::append_eval_log(serde_json::json!({
            "ts_ms": ts_ms,
            "qver": crate::policy::QUESTION_VERSION,
            "model": MODEL,
            "resolved_model": body.get("model").and_then(|m| m.as_str()),
            "input_chars": input_chars,
            "answers": answers
        }));

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
        // Score/Choice always carry confidence per the API; a missing value is
        // unknown, not zero. 0.5 lands in Flag so it prints [verify] and
        // needs_review surfaces it, instead of forcing a Drop.
        let geo_confidence = geo_obj["confidence"].as_f64().unwrap_or(0.5);
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
        let gap_confidence = gap_obj["confidence"].as_f64().unwrap_or(0.5);

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
                    let mut out: serde_json::Map<String, serde_json::Value> = m
                        .iter()
                        .filter(|(k, _)| {
                            !["intent", "geo_score", "direct_answer", "content_gap"]
                                .contains(&k.as_str())
                        })
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    // Keep the intent distribution: reviewers see the runner-up
                    // on Flag verdicts instead of a bare low confidence.
                    if let Some(probs) = answers
                        .get("intent")
                        .and_then(|i| i.get("probabilities"))
                        .and_then(|p| p.as_object())
                    {
                        out.insert(
                            "intent_probs".into(),
                            serde_json::Value::Object(probs.clone()),
                        );
                    }
                    out
                })
                .unwrap_or_default(),
        })
    }

    /// Dedicated injection pre-screen (skill jaggedness #6): one Noul before the
    /// full suite. True = block; do not trust further semantic answers on this state.
    pub fn injection_preflight(&self, state: &serde_json::Value) -> Result<bool> {
        let payload = json!({
            "model": MODEL,
            "state": truncate_state(prefilter_state(state.clone())),
            "questions": crate::policy::injection_question()
        });
        let body: serde_json::Value = self.post(payload)?.into_json()?;
        record_usage(&body);
        let p = body
            .pointer("/answers/injection_risk/noul")
            .or_else(|| body.pointer("/answers/injection_risk/probability"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        Ok(p >= 0.70)
    }

    /// Judge one page (or snippet) with the full speculative suite in a single
    /// request: base intent/GEO/gap + geo dimensions + page quality.
    /// Runs injection preflight first; blocked content returns a Drop-tier result
    /// without asking quality questions.
    pub fn judge_page(
        &self,
        state: serde_json::Value,
    ) -> Result<AnalysisResult> {
        if self.injection_preflight(&state)? {
            return Ok(AnalysisResult {
                intent: "unclear".into(),
                intent_confidence: 0.0,
                geo_score: 1,
                geo_confidence: 0.0,
                direct_answer: false,
                direct_answer_p: 0.0,
                content_gap: "generic_prose".into(),
                gap_confidence: 0.0,
                extra: serde_json::json!({
                    "injection_risk": { "type": "noul", "value": 1.0, "band": "yes", "blocked": true }
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            });
        }
        let mut extras = crate::policy::geo_questions();
        let page = crate::policy::page_audit_extras();
        if let (Some(dst), Some(src)) = (extras.as_object_mut(), page.as_object()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        self.fanout_eval_with(state, extras)
    }

    /// Site/homepage judgment: base + GEO dims + value prop / entity / model.
    pub fn judge_site(&self, state: serde_json::Value) -> Result<AnalysisResult> {
        if self.injection_preflight(&state)? {
            return Ok(AnalysisResult {
                intent: "unclear".into(),
                intent_confidence: 0.0,
                geo_score: 1,
                geo_confidence: 0.0,
                direct_answer: false,
                direct_answer_p: 0.0,
                content_gap: "generic_prose".into(),
                gap_confidence: 0.0,
                extra: serde_json::json!({
                    "injection_risk": { "type": "noul", "value": 1.0, "band": "yes", "blocked": true }
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            });
        }
        let mut extras = crate::policy::geo_questions();
        let site = crate::policy::site_extras();
        if let (Some(dst), Some(src)) = (extras.as_object_mut(), site.as_object()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        self.fanout_eval_with(state, extras)
    }
}

/// Drop noise fields before truncate so Jev sees decisive evidence only.
fn prefilter_state(value: serde_json::Value) -> serde_json::Value {
    const DROP_KEYS: &[&str] = &[
        "raw_html",
        "html",
        "scripts",
        "styles",
        "css",
        "inline_script",
        "dom_snapshot",
        "history",
        "chat_transcript",
        "entire_repo",
        "conversation",
    ];
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .filter(|(k, v)| {
                    let kl = k.to_ascii_lowercase();
                    if DROP_KEYS.iter().any(|d| kl == *d) {
                        return false;
                    }
                    // Drop empty containers and pure-whitespace strings.
                    match v {
                        serde_json::Value::Array(a) => !a.is_empty(),
                        serde_json::Value::Object(o) => !o.is_empty(),
                        serde_json::Value::String(s) => !s.trim().is_empty(),
                        _ => true,
                    }
                })
                .map(|(k, v)| (k, prefilter_state(v)))
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(prefilter_state).collect())
        }
        other => other,
    }
}

/// Fold API usage into the run ledger (input tokens drive cost at list price).
/// Also pins the resolved model version: thresholds couple to one model's
/// distribution, so the ledger records which build actually served.
pub static JEV_MODEL: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn jev_model() -> &'static str {
    JEV_MODEL.get().map(String::as_str).unwrap_or("unknown")
}

fn record_usage(body: &serde_json::Value) {
    let usage = match body.get("usage") {
        Some(u) if u.is_object() => u.clone(),
        _ => return,
    };
    if let Some(t) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
        crate::manifest::JEV_INPUT_TOKENS.fetch_add(t, Ordering::Relaxed);
    }
    if let Some(m) = body.get("model").and_then(|m| m.as_str()) {
        let _ = JEV_MODEL.set(m.to_string());
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
