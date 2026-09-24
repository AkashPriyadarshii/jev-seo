use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RpcRequest {
    pub(crate) jsonrpc: String,
    pub(crate) id: Option<serde_json::Value>,
    pub(crate) method: String,
    pub(crate) params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RpcResponse {
    pub(crate) jsonrpc: String,
    pub(crate) id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<serde_json::Value>,
}

pub fn run_stdio_server() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let text = match line {
            Ok(t) => t,
            Err(_) => break,
        };

        if text.trim().is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(_) => {
                let err = json!({ "jsonrpc": "2.0", "id": null,
                    "error": { "code": -32700, "message": "Parse error" } });
                writeln!(stdout, "{}", err)?;
                stdout.flush()?;
                continue;
            }
        };

        // Notifications carry no id and get no reply.
        if req.id.is_none() {
            continue;
        }

        let resp = handle_request(&req);
        let resp_str = serde_json::to_string(&resp)?;
        writeln!(stdout, "{}", resp_str)?;
        stdout.flush()?;
    }

    Ok(())
}

pub(crate) fn handle_request(req: &RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        "initialize" => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "jev-seo", "version": env!("CARGO_PKG_VERSION") }
            })),
            error: None,
        },
        "notifications/initialized" | "notifications/cancelled" => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(json!({
                "tools": [
                    {
                        "name": "seo_keywords",
                        "description": "Fetch autocomplete keyword suggestions for zero cost",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "seo_serp_inspect",
                        "description": "Search live SERP competitors (search API, DuckDuckGo fallback) and analyze content gaps",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" },
                                "limit": { "type": "integer" }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "seo_audit",
                        "description": "Audit a local file or an entire directory for on-page SEO issues, duplicate titles, and thin pages",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "seo_geo",
                        "description": "Evaluate Generative Engine Optimization (GEO) citation likelihood (1-10) and direct answer presence using TypeSafe Jev",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "File path or content snippet to score" },
                                "query": { "type": "string", "description": "Target search query" }
                            },
                            "required": ["target", "query"]
                        }
                    },
                    {
                        "name": "seo_schema",
                        "description": "Validate Schema.org JSON-LD markup against active 2026 search specifications",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "File path, HTML snippet, or JSON string" }
                            },
                            "required": ["target"]
                        }
                    },
                    {
                        "name": "seo_robots",
                        "description": "Inspect robots.txt on a live domain for AI crawler permissions (GPTBot, ClaudeBot, PerplexityBot, Google-Extended) and sitemaps",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "domain": { "type": "string", "description": "Domain or URL to inspect" }
                            },
                            "required": ["domain"]
                        }
                    },
                    {
                        "name": "seo_brief",
                        "description": "Synthesize live SERP competitor results into a ready-to-write content brief with H2 outlines and 150-word direct answer guidance",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "topic": { "type": "string" },
                                "limit": { "type": "integer", "description": "Number of SERP competitors to scrape (default: 5)" }
                            },
                            "required": ["topic"]
                        }
                    },
                    {
                        "name": "seo_sitemap",
                        "description": "Inspect and validate XML sitemaps for protocols, 50k URL limits, and international hreflang tags",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "URL or local file path to sitemap.xml" }
                            },
                            "required": ["target"]
                        }
                    },
                    {
                        "name": "seo_crawl",
                        "description": "Crawl a live site for broken links, redirect chains, and orphan pages",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "url": { "type": "string", "description": "Start URL" },
                                "max_pages": { "type": "integer", "description": "Maximum pages to fetch (default: 50)" }
                            },
                            "required": ["url"]
                        }
                    },
                    {
                        "name": "seo_llms",
                        "description": "Check llms.txt presence and AI crawler permissions for answer-engine readiness",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "domain": { "type": "string", "description": "Domain or URL to inspect" }
                            },
                            "required": ["domain"]
                        }
                    },
                    {
                        "name": "seo_extract",
                        "description": "Extract clean markdown from URLs via paid API (needs TAVILY_API_KEY), else error",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "urls": { "type": "array", "items": { "type": "string" } },
                                "query": { "type": "string", "description": "Rerank intent" }
                            },
                            "required": ["urls", "query"]
                        }
                    },
                    {
                        "name": "seo_explain",
                        "description": "Explain a stable rule id (R19 or RULE-R19): area, severity, effort, title, fix",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string", "description": "Rule id, e.g. R19 or RULE-R19" }
                            },
                            "required": ["id"]
                        }
                    },
                    {
                        "name": "seo_report",
                        "description": "Diff two saved audit JSON reports: score delta, rules cleared/new, ranked actions",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string", "description": "Current audit JSON path" },
                                "baseline": { "type": "string", "description": "Baseline audit JSON path" }
                            },
                            "required": ["path", "baseline"]
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            let params = req.params.as_ref();
            let tool_name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
            let args = params.and_then(|p| p.get("arguments")).cloned().unwrap_or(json!({}));

            let result_content = execute_tool(tool_name, &args);
            let is_error = result_content.starts_with("Error:");
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": result_content,
                            "isError": is_error
                        }
                    ],
                    "isError": is_error
                })),
                error: None,
            }
        }
        _ => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: None,
            error: Some(json!({ "code": -32601, "message": "Method not found" })),
        },
    }
}

/// Second-layer guard for agent-chosen targets. Static path rules already ran;
/// this asks Jev whether the target smells like a secret. No key configured
/// means this layer is off (static guards still apply); a failed check while
/// configured blocks, because an unreachable guard is not a clean bill.
fn safety_gate(tool: &str, target: &str) -> Option<String> {
    match crate::engine::JevClient::new() {
        None => None,
        Some(client) => match client.safety_block(tool, target) {
            Some(true) => Some(format!("Error: blocked unsafe target for {}: {}", tool, target)),
            Some(false) => None,
            None => Some(format!("Error: safety check unreachable for {}: {}", tool, target)),
        },
    }
}

fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "seo_serp_inspect" => {
            let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            match crate::serp::scrape_serp(q, limit) {
                Ok((items, _)) => serde_json::to_string_pretty(&items).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_audit" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_audit", path) {
                return err;
            }
            let checked = match crate::paths::check_audit_path(path) {
                Ok(p) => p,
                Err(e) => return format!("Error: {e:#}"),
            };
            match crate::audit::audit_path(&checked) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_geo" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_geo", target) {
                return err;
            }
            let content = match crate::paths::read_user_file(target, &["md", "mdx", "markdown", "html", "htm", "txt"]) {
                Ok(c) => c,
                Err(e) => return format!("Error: {}", e),
            };
            if let Some(client) = crate::engine::JevClient::new() {
                let lower = target.to_ascii_lowercase();
                let is_html = lower.ends_with(".html") || lower.ends_with(".htm");
                let text = if is_html {
                    crate::fetch::readable_text(&content, 6000)
                } else {
                    content.chars().take(6000).collect::<String>()
                };
                let opening = is_html.then(|| crate::fetch::opening_after_h1(&content, 500));
                let wc = text.split_whitespace().count();
                let state = crate::engine::page_state(query, Some(target.to_string()), None, text, wc, opening);
                match client.judge_page(state) {
                    Ok(eval) => {
                        if crate::policy::injection_blocked(&eval.extra) {
                            return "Error: blocked: injection risk in content (Jev pre-screen).".into();
                        }
                        if crate::policy::gate("geo", eval.geo_confidence) == crate::policy::Verdict::Drop {
                            return "Error: Jev unsure (low confidence), no score.".into();
                        }
                        serde_json::to_string_pretty(&eval).unwrap_or_default()
                    }
                    Err(e) => format!("Error evaluating GEO via Jev: {}", e),
                }
            } else {
                "Error: TYPESAFE_API_KEY environment variable not configured.".into()
            }
        }
        "seo_keywords" => {
            let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match crate::serp::get_autocomplete(q) {
                Ok(items) => {
                    if let Some(client) = crate::engine::JevClient::new() {
                        let state = json!({ "root_query": q, "suggestions": items });
                        let extras = crate::policy::keyword_value_extras(&items, 10);
                        if let Ok(eval) = client.fanout_eval_with(state, extras) {
                            if crate::policy::gate("keywords", eval.confidence()) != crate::policy::Verdict::Drop {
                                let out = json!({
                                    "suggestions": items,
                                    "intent": eval.intent,
                                    "intent_confidence": eval.intent_confidence,
                                    "content_gap": eval.content_gap,
                                    "keyword_values": eval.extra
                                });
                                return serde_json::to_string_pretty(&out).unwrap_or_default();
                            }
                        }
                    }
                    serde_json::to_string_pretty(&items).unwrap_or_default()
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_schema" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_schema", target) {
                return err;
            }
            match crate::schema::validate_target(target) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_robots" => {
            let domain = args.get("domain").and_then(|v| v.as_str()).unwrap_or("");
            match crate::robots::inspect_robots(domain) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_brief" => {
            let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            match crate::brief::generate_brief(topic, limit) {
                Ok(brief) => serde_json::to_string_pretty(&brief).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_sitemap" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_sitemap", target) {
                return err;
            }
            match crate::sitemap::audit_sitemap(target) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_crawl" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_crawl", url) {
                return err;
            }
            let max_pages = args.get("max_pages").and_then(|v| v.as_u64()).unwrap_or(crate::crawl::DEFAULT_MAX_PAGES as u64) as usize;
            let mut budget = crate::fetch::Budget::default();
            match crate::crawl::crawl_site(url, max_pages, crate::fetch::FetchMode::Auto, &mut budget) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_llms" => {
            let domain = args.get("domain").and_then(|v| v.as_str()).unwrap_or("");
            match crate::llms::check_llms(domain) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_extract" => {
            let urls: Vec<String> = args
                .get("urls")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match crate::serp::tavily_extract(&urls, query) {
                Ok(md) => md,
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_explain" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
            match crate::rules::explain(id) {
                Some(text) => text,
                None => format!("Error: unknown rule id: {}", id),
            }
        }
        "seo_report" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let baseline = args.get("baseline").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(err) = safety_gate("seo_report", path) {
                return err;
            }
            if let Some(err) = safety_gate("seo_report", baseline) {
                return err;
            }
            match (
                crate::paths::read_user_file(path, &["json"]),
                crate::paths::read_user_file(baseline, &["json"]),
            ) {
                (Ok(c), Ok(b)) => match (
                    serde_json::from_str::<crate::audit::DirectoryAuditReport>(&c),
                    serde_json::from_str::<crate::audit::DirectoryAuditReport>(&b),
                ) {
                    (Ok(cur), Ok(base)) => {
                        let actions = crate::rules::actions_for(&cur.findings);
                        let out = json!({
                            "diff": crate::main_report_diff(&cur, &base),
                            "actions": actions,
                            "pairs": crate::audit::cannibalization_pairs(&cur),
                        });
                        serde_json::to_string_pretty(&out).unwrap_or_default()
                    }
                    (Err(e), _) | (_, Err(e)) => format!("Error: parse audit JSON: {}", e),
                },
                (Err(e), _) | (_, Err(e)) => format!("Error: read audit JSON: {}", e),
            }
        }
        _ => format!("Error: unknown tool: {}", name),
    }
}
