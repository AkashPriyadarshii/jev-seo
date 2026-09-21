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
            Err(_) => continue,
        };

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
                        "description": "Scrape live DuckDuckGo SERP competitors and analyze gaps",
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
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": result_content
                        }
                    ]
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

fn read_geo_target(target: &str) -> anyhow::Result<String> {
    const ALLOWED: &[&str] = &["md", "mdx", "markdown", "html", "htm", "txt"];
    let trimmed = target.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        anyhow::bail!("geo does not fetch URLs — pass a local content file path or an inline snippet");
    }
    let path = std::path::Path::new(trimmed);
    let looks_like_path = path.exists()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || (trimmed.starts_with('.') && trimmed.len() > 1);
    if !looks_like_path {
        return Ok(trimmed.to_string());
    }
    if !path.exists() {
        anyhow::bail!("file not found: {}", trimmed);
    }
    if !path.is_file() {
        anyhow::bail!("not a file: {}", trimmed);
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            anyhow::bail!("refusing dot-file: {}", trimmed);
        }
    }
    let ext_ok = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false);
    if !ext_ok {
        anyhow::bail!(
            "refusing non-content file (allowed: .md .mdx .markdown .html .htm .txt): {}",
            trimmed
        );
    }
    Ok(std::fs::read_to_string(path)?)
}

fn execute_tool(name: &str, args: &serde_json::Value) -> String {
    match name {
        "seo_keywords" => {
            let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match crate::serp::get_autocomplete(q) {
                Ok(items) => serde_json::to_string_pretty(&items).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_serp_inspect" => {
            let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            match crate::serp::scrape_serp(q, limit) {
                Ok(items) => serde_json::to_string_pretty(&items).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_audit" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            match crate::audit::audit_path(path) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_geo" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let content = match read_geo_target(target) {
                Ok(c) => c,
                Err(e) => return format!("Error: {}", e),
            };
            if let Some(client) = crate::engine::JevClient::new() {
                let state = json!({ "query": query, "content": content });
                match client.fanout_eval(state) {
                    Ok(eval) => serde_json::to_string_pretty(&eval).unwrap_or_default(),
                    Err(e) => format!("Error evaluating GEO via Jev: {}", e),
                }
            } else {
                "Error: TYPESAFE_API_KEY environment variable not configured.".into()
            }
        }
        "seo_schema" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
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
            match crate::sitemap::audit_sitemap(target) {
                Ok(rep) => serde_json::to_string_pretty(&rep).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}
