//! Opt-in paid engine answers over any OpenAI-compatible chat endpoint.
//! One code path covers OpenRouter, Perplexity, Ollama, vLLM, gateways:
//! JEV_SEO_LLM_KEY + JEV_SEO_LLM_MODEL, optional JEV_SEO_LLM_URL
//! (defaults to OpenRouter). No key, no calls, never silent.

use anyhow::{Context, Result};
use std::time::Duration;

fn endpoint() -> String {
    std::env::var("JEV_SEO_LLM_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".into())
}

/// Ask the configured engine. Returns answer text plus citation URLs when the
/// provider supplies them (Perplexity-style top-level `citations` array).
pub fn ask(prompt: &str) -> Result<(String, Vec<String>)> {
    let key = std::env::var("JEV_SEO_LLM_KEY").context("JEV_SEO_LLM_KEY not set")?;
    let model = std::env::var("JEV_SEO_LLM_MODEL").context("JEV_SEO_LLM_MODEL not set")?;
    let url = format!("{}/chat/completions", endpoint().trim_end_matches('/'));
    let body: serde_json::Value = ureq::post(&url)
        .timeout(Duration::from_secs(30))
        .set("Authorization", &format!("Bearer {}", key.trim()))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": model.trim(),
            "messages": [{ "role": "user", "content": prompt }],
            "max_tokens": 500,
        }))
        .context("engine request failed")?
        .into_json()?;
    let text = body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .context("engine returned no choices text")?
        .to_string();
    let cites = body
        .get("citations")
        .and_then(|c| c.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    Ok((text, cites))
}
