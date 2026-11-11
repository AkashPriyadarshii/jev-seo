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

/// How `seo_cite_check` borrows a model. Live servers use [`StdioSampler`]
/// (MCP `sampling/createMessage` against the connected client); tests and the
/// pure [`handle_request`] path use [`NoSampler`], which always declines.
pub(crate) trait Sampler {
    fn sample(&mut self, prompt: &str) -> std::result::Result<String, String>;
}

/// Sampler for contexts with no client attached: every sampling call fails
/// clearly instead of hanging.
pub(crate) struct NoSampler;

impl Sampler for NoSampler {
    fn sample(&mut self, _prompt: &str) -> std::result::Result<String, String> {
        Err("no sampling client attached; run jev-seo as an MCP server inside an agent that supports sampling".into())
    }
}

/// Live sampler: sends `sampling/createMessage` to the connected client over
/// the same stdio pair and waits for the matching response id. Notifications
/// are skipped; responses for unknown ids keep waiting instead of dropping.
pub(crate) struct StdioSampler<'a> {
    rdr: std::io::StdinLock<'a>,
    stdout: &'a mut std::io::Stdout,
    next_id: u64,
    /// Set from the client's `initialize` params: no declared capability,
    /// no request sent (some clients silently drop it, which would hang).
    sampling_allowed: bool,
}

/// Next non-empty client line, or None on EOF/error. Reads with a byte cap:
/// an unbounded `read_line` lets a hostile client stream megabytes without a
/// newline and balloon RSS on a small-memory box. Over-cap chunks are
/// truncated; parse error handling downstream turns each into a -32700 reply.
const MCP_LINE_CAP: usize = 8 * 1024 * 1024;

fn capped_line(rdr: &mut std::io::StdinLock) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut over_cap = false;
    loop {
        let avail = rdr.fill_buf().ok()?;
        if avail.is_empty() {
            break; // EOF
        }
        let newl = avail.iter().position(|&b| b == b'\n');
        let take = newl.map(|i| i + 1).unwrap_or(avail.len());
        if !over_cap && buf.len() + take <= MCP_LINE_CAP {
            buf.extend_from_slice(&avail[..take]);
        } else {
            over_cap = true;
        }
        rdr.consume(take);
        if newl.is_some() {
            break;
        }
    }
    if buf.is_empty() && !over_cap {
        return None;
    }
    let s = String::from_utf8_lossy(&buf).into_owned();
    Some(s.trim_end_matches(['\r', '\n']).to_string())
}

impl<'a> StdioSampler<'a> {
    fn new(stdout: &'a mut std::io::Stdout, rdr: std::io::StdinLock<'a>) -> Self {
        Self {
            rdr,
            stdout,
            next_id: 0,
            sampling_allowed: false,
        }
    }

    /// Next non-empty client line, or None on EOF/error.
    fn next_line(&mut self) -> Option<String> {
        loop {
            match capped_line(&mut self.rdr) {
                None => return None,
                Some(t) if t.trim().is_empty() => continue,
                Some(t) => return Some(t),
            }
        }
    }

    fn reply(&mut self, text: &str) -> Result<()> {
        writeln!(self.stdout, "{}", text)?;
        self.stdout.flush()?;
        Ok(())
    }
}

impl Sampler for StdioSampler<'_> {
    fn sample(&mut self, prompt: &str) -> std::result::Result<String, String> {
        if !self.sampling_allowed {
            return Err("client did not declare the sampling capability".into());
        }
        let id = format!("jev-seo-sample-{}", self.next_id);
        self.next_id += 1;
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "sampling/createMessage",
            "params": {
                "messages": [{ "role": "user", "content": { "type": "text", "text": prompt } }],
                "maxTokens": 500,
            }
        });
        self.reply(&serde_json::to_string(&req).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        loop {
            let text = match self.next_line() {
                Some(t) => t,
                None => return Err("client closed the stream during sampling".into()),
            };
            let msg: serde_json::Value = serde_json::from_str(&text)
                .map_err(|_| "client sent a non-JSON sampling reply".to_string())?;
            if msg.get("id") != Some(&json!(id)) {
                continue; // notification or unrelated response: keep waiting
            }
            if let Some(err) = msg.get("error") {
                let detail = err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(format!("client declined sampling: {}", detail));
            }
            return sample_text(msg.get("result"))
                .ok_or_else(|| "client sent an unexpected sampling result shape".to_string());
        }
    }
}

/// Pull model text out of a `sampling/createMessage` result, accepting the
/// single-object and array content shapes.
fn sample_text(result: Option<&serde_json::Value>) -> Option<String> {
    let content = result?.get("content")?;
    let first = if let Some(arr) = content.as_array() {
        arr.first()?
    } else {
        content
    };
    if first.get("type")?.as_str()? != "text" {
        return None;
    }
    first.get("text")?.as_str().map(str::to_string)
}

pub fn run_stdio_server() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut sampler = StdioSampler::new(&mut stdout, stdin.lock());

    while let Some(text) = sampler.next_line() {
        let req: RpcRequest = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(_) => {
                let err = json!({ "jsonrpc": "2.0", "id": null,
                    "error": { "code": -32700, "message": "Parse error" } });
                sampler.reply(&err.to_string())?;
                continue;
            }
        };

        // Notifications carry no id and get no reply.
        if req.id.is_none() {
            continue;
        }

        // Capability gate: only clients that declare `sampling` ever get a
        // `sampling/createMessage` request; the rest take the manual loop.
        if req.method == "initialize" {
            sampler.sampling_allowed = req
                .params
                .as_ref()
                .and_then(|p| p.get("capabilities"))
                .and_then(|c| c.get("sampling"))
                .is_some();
        }

        let resp = handle_request_with(&req, &mut sampler);
        let resp_str = serde_json::to_string(&resp)?;
        sampler.reply(&resp_str)?;
    }

    Ok(())
}

/// Pure entry point (no sampling client): used by tests and embedders.
#[allow(dead_code)]
pub(crate) fn handle_request(req: &RpcRequest) -> RpcResponse {
    handle_request_with(req, &mut NoSampler)
}

/// Test seam: citation struct without ledger side effects.
#[cfg(test)]
pub(crate) fn cite_detail_for_test(
    target: &str,
    answer: &str,
    citations: &[String],
) -> serde_json::Value {
    cite_detail(target, answer, citations)
}

pub(crate) fn handle_request_with(req: &RpcRequest, sampler: &mut dyn Sampler) -> RpcResponse {
    match req.method.as_str() {
        "initialize" => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "jev-seo", "version": env!("CARGO_PKG_VERSION") },
                "instructions": "Free SEO and GEO audits: audit local docs, crawl live sites, score AI citation likelihood, track rank drift. Crawl before judging; quote the completeness line with any score."
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
                        "name": "seo_cite_check",
                        "title": "Citation check",
                        "description": "Check whether an AI answer names the target domain for a buyer query. Pass \"answer\" to score directly (works everywhere); omit it to borrow the client's model via sampling, or get the prompt back to answer yourself",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "Domain or URL to look for, e.g. example.com" },
                                "query": { "type": "string", "description": "Buyer query to check" },
                                "answer": { "type": "string", "description": "AI answer text to score; omit to use sampling or receive the prompt" }
                            },
                            "required": ["target", "query"]
                        },
                        "outputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string" },
                                "query": { "type": "string" },
                                "cited": { "type": "boolean" },
                                "since_last": {},
                                "source": { "type": "string" },
                                "excerpt": { "type": "string" }
                            },
                            "required": ["target", "query", "cited", "source"]
                        },
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
                    },
                    {
                        "name": "seo_gap",
                        "title": "Gap queue",
                        "description": "Ranked gap queue for a verified Search Console site: high-impression queries in weak positions joined with citation history",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "site": { "type": "string", "description": "Verified site URL, e.g. https://example.com/" },
                                "limit": { "type": "integer", "description": "Top queries to rank (default 10)" }
                            },
                            "required": ["site"]
                        }
                    },
                    {
                        "name": "seo_keywords",
                        "title": "Keyword suggestions",
                        "description": "Fetch autocomplete keyword suggestions for zero cost",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "SERP inspect",
                        "description": "Search live SERP competitors (search API, DuckDuckGo fallback) and analyze content gaps",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Audit path",
                        "description": "Audit a local file or an entire directory for on-page SEO issues, duplicate titles, and thin pages",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
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
                        "title": "GEO score",
                        "description": "Evaluate Generative Engine Optimization (GEO) citation likelihood (1-10) and direct answer presence using TypeSafe Jev",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Validate schema",
                        "description": "Validate Schema.org JSON-LD markup against active 2026 search specifications",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
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
                        "title": "Inspect robots",
                        "description": "Inspect robots.txt on a live domain for AI crawler permissions (GPTBot, ClaudeBot, PerplexityBot, Google-Extended) and sitemaps",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Content brief",
                        "description": "Synthesize live SERP competitor results into a ready-to-write content brief with H2 outlines and 150-word direct answer guidance",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Audit sitemap",
                        "description": "Inspect and validate XML sitemaps for protocols, 50k URL limits, and international hreflang tags",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Crawl site",
                        "description": "Crawl a live site for broken links, redirect chains, and orphan pages",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "url": { "type": "string", "description": "Start URL" },
                                "max_pages": { "type": "integer", "description": "Maximum pages to fetch (default: 50)" },
                                "sitemap": { "type": "string", "description": "Sitemap URL override (else robots.txt Sitemap: + /sitemap.xml)" }
                            },
                            "required": ["url"]
                        }
                    },
                    {
                        "name": "seo_llms",
                        "title": "Check llms.txt",
                        "description": "Check llms.txt presence and AI crawler permissions for answer-engine readiness",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Extract markdown",
                        "description": "Extract clean markdown from URLs via paid API (needs TAVILY_API_KEY), else error",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
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
                        "title": "Explain rule",
                        "description": "Explain a stable rule id (R19 or RULE-R19): area, severity, effort, title, fix",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
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
                        "title": "Diff reports",
                        "description": "Diff two saved audit JSON reports: score delta, rules cleared/new, ranked actions",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string", "description": "Current audit JSON path" },
                                "baseline": { "type": "string", "description": "Baseline audit JSON path" },
                                "baseline_label": { "type": "string", "description": "Stored baseline label instead of a path (see drift baseline)" }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "seo_check",
                        "title": "Search CI check",
                        "description": "One-command search CI: audit a path under policy gates, report PASS/FAIL with blocking counts",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string", "description": "File or directory to check" }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "seo_visibility",
                        "title": "Citation visibility",
                        "description": "Rich citation verdict: mentioned vs cited vs recommended, position, sentiment (alias of seo_cite_check struct)",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string" },
                                "query": { "type": "string" },
                                "answer": { "type": "string", "description": "AI answer text to score" }
                            },
                            "required": ["target", "query"]
                        }
                    },
                    {
                        "name": "seo_index",
                        "title": "Indexability truth",
                        "description": "Indexability truth table for a URL: HTTP, robots, meta, canonical, sitemap, inlinks to one verdict",
                        "annotations": { "readOnlyHint": true, "openWorldHint": true, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "url": { "type": "string", "description": "URL to evaluate" }
                            },
                            "required": ["url"]
                        }
                    },
                    {
                        "name": "seo_verify",
                        "title": "Verify fixes",
                        "description": "Re-check rule ids against a path: PASS when the finding is gone",
                        "annotations": { "readOnlyHint": true, "openWorldHint": false, "destructiveHint": false },
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "rules": { "type": "string", "description": "Comma rule ids, e.g. R09,R13" },
                                "path": { "type": "string" }
                            },
                            "required": ["rules", "path"]
                        }
                    }
                ]
            })),
            error: None,
        },
        "resources/list" => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id.clone(),
            result: Some(json!({
                "resources": [
                    { "uri": "jev://project/config", "name": "Project search contract", "mimeType": "application/json" },
                    { "uri": "jev://project/health", "name": "Current search health", "mimeType": "application/json" },
                    { "uri": "jev://project/findings", "name": "Latest audit findings", "mimeType": "application/json" },
                ]
            })),
            error: None,
        },
        "resources/read" => {
            let uri = req
                .params
                .as_ref()
                .and_then(|p| p.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let text = match uri {
                "jev://project/config" => {
                    let p = crate::policy_file::load();
                    serde_json::to_string_pretty(&p).unwrap_or_default()
                }
                "jev://project/health" => {
                    serde_json::to_string_pretty(&crate::health::empty_coverage())
                        .unwrap_or_default()
                }
                "jev://project/findings" => "[]".into(),
                _ => String::new(),
            };
            if text.is_empty() {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(json!({ "code": -32002, "message": "Unknown resource" })),
                }
            } else {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id.clone(),
                    result: Some(
                        json!({ "contents": [{ "uri": uri, "mimeType": "application/json", "text": text }] }),
                    ),
                    error: None,
                }
            }
        }
        "tools/call" => {
            let params = req.params.as_ref();
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let args = params
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or(json!({}));

            let result_content = execute_tool_with(tool_name, &args, sampler);
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
            Some(true) => Some(format!(
                "Error: blocked unsafe target for {}: {}",
                tool, target
            )),
            Some(false) => None,
            None => Some(format!(
                "Error: safety check unreachable for {}: {}",
                tool, target
            )),
        },
    }
}

/// Normalize a target domain or URL to the bare host the model's answer is
/// scanned for: lowercase, no scheme, no leading www., no path.
/// The exact buyer-query prompt the agent answers, whether over sampling or
/// with its own model in the manual loop.
fn cite_prompt(query: &str) -> String {
    format!(
        "Answer this search query briefly, naming the specific websites or brands you recommend: {query}"
    )
}

/// Citation struct: mentioned vs cited vs recommended, with position and
/// sentiment. Deterministic text heuristics, no model. Keeps bool `cited`
/// for compat. Negation ("would not recommend X") counts as mentioned only.
fn cite_detail(target: &str, answer: &str, citations: &[String]) -> serde_json::Value {
    let host = cite_host(target);
    let lower = answer.to_lowercase();
    let in_text = lower.contains(&host);
    let in_cites = citations.iter().any(|c| c.to_lowercase().contains(&host));
    let mentioned = in_text;
    // Answer text naming the domain IS the citation (the model said it).
    // Engine citation list is the second path. Negation revokes both.
    let cited = (in_text || in_cites) && !is_negated(&lower, &host);
    let negated = is_negated(&lower, &host);
    let recommended = cited && !negated && has_recommend_signal(&lower, &host);
    let position = mention_position(&lower, &host);
    json!({
        "mentioned": mentioned,
        "cited": cited && !negated,
        "recommended": recommended,
        "position": position,
        "negated": negated,
        "sentiment": if negated { "negative" } else if recommended { "positive" } else { "neutral" },
    })
}

#[allow(dead_code)]
fn has_citation_signal(lower: &str, host: &str) -> bool {
    let i = lower.find(host).unwrap_or(usize::MAX);
    let window = &lower[i.saturating_sub(120)..(i + host.len() + 120).min(lower.len())];
    window.contains("http")
        || window.contains("www.")
        || window.contains("source")
        || window.contains("cite")
        || window.contains("link")
}

fn is_negated(lower: &str, host: &str) -> bool {
    let i = match lower.find(host) {
        Some(i) => i,
        None => return false,
    };
    let window = &lower[i.saturating_sub(80)..i];
    window.contains("not recommend")
        || window.contains("would not")
        || window.contains("avoid")
        || window.contains("don't")
        || window.contains("do not")
}

fn has_recommend_signal(lower: &str, host: &str) -> bool {
    let i = match lower.find(host) {
        Some(i) => i,
        None => return false,
    };
    let window = &lower[i.saturating_sub(160)..(i + host.len() + 60).min(lower.len())];
    window.contains("recommend")
        || window.contains("best")
        || window.contains("top")
        || window.contains("try")
        || window.contains("use")
}

fn mention_position(lower: &str, host: &str) -> Option<usize> {
    let i = lower.find(host)?;
    Some(lower[..i].chars().filter(|&c| c == '.').count() + 1)
}

/// Score one answer for a target citation and log the drift verdict.
/// Best-effort ledger: a stuck database never fails the check.
fn cite_verdict(
    target: &str,
    query: &str,
    answer: &str,
    source: &str,
    citations: &[String],
) -> String {
    let detail = cite_detail(target, answer, citations);
    let cited = detail
        .get("cited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let prev = crate::rank::DbStore::open()
        .ok()
        .and_then(|db| db.record_cite(target, query, cited).ok())
        .flatten();
    let excerpt: String = answer.chars().take(300).collect();
    let mut out = serde_json::json!({
        "target": target,
        "query": query,
        "cited": cited,
        "since_last": prev,
        "source": source,
        "engine_citations": citations,
        "excerpt": excerpt,
    });
    if let Some(map) = out.as_object_mut() {
        if let Some(d) = detail.as_object() {
            for (k, v) in d {
                map.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::to_string_pretty(&out).unwrap_or_default()
}

fn cite_host(target: &str) -> String {
    let t = target.trim().to_lowercase();
    let t = t.split("://").last().unwrap_or(&t);
    let t = t.strip_prefix("www.").unwrap_or(t);
    t.split(['/', '?', '#']).next().unwrap_or(t).to_string()
}

fn execute_tool_with(name: &str, args: &serde_json::Value, sampler: &mut dyn Sampler) -> String {
    match name {
        "seo_gap" => {
            let site = args.get("site").and_then(|v| v.as_str()).unwrap_or("");
            if site.trim().is_empty() {
                return "Error: seo_gap needs \"site\" (verified Search Console URL).".into();
            }
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            match crate::gsc::gap(site, limit) {
                Ok(rows) => serde_json::to_string_pretty(&rows).unwrap_or_default(),
                Err(e) => format!("Error: {e:#}"),
            }
        }
        "seo_cite_check" => {
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if target.is_empty() || query.is_empty() {
                return "Error: seo_cite_check needs both \"target\" (domain or URL) and \"query\".".into();
            }
            if let Some(err) = safety_gate("seo_cite_check", target) {
                return err;
            }
            // Universal path: the agent answers with its own model, no client
            // capability needed. Works on every MCP client ever made.
            if let Some(answer) = args.get("answer").and_then(|v| v.as_str()) {
                if !answer.trim().is_empty() {
                    return cite_verdict(target, query, answer, "answer", &[]);
                }
            }
            // Fast path: borrow the client's model over sampling.
            let prompt = cite_prompt(query);
            match sampler.sample(&prompt) {
                Ok(answer) => cite_verdict(target, query, &answer, "sampling", &[]),
                Err(se) => match crate::llm::ask(&prompt) {
                    // Paid path: configured engine answers server-side.
                    Ok((answer, cites)) => cite_verdict(target, query, &answer, "engine", &cites),
                    // Manual loop fallback: tell the agent exactly how to finish.
                    Err(le) => serde_json::to_string_pretty(&json!({
                        "target": target,
                        "query": query,
                        "needs_answer": true,
                        "prompt": prompt,
                        "note": format!("sampling unavailable ({se}); engine unavailable ({le}); answer the prompt with your own model, then re-call seo_cite_check with \"answer\""),
                    }))
                    .unwrap_or_default(),
                },
            }
        }
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
            let content = match crate::paths::read_user_file(
                target,
                &["md", "mdx", "markdown", "html", "htm", "txt"],
            ) {
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
                let state = crate::engine::page_state(
                    query,
                    Some(target.to_string()),
                    None,
                    text,
                    wc,
                    opening,
                );
                match client.judge_page(state) {
                    Ok(eval) => {
                        if crate::policy::injection_blocked(&eval.extra) {
                            return "Error: blocked: injection risk in content (Jev pre-screen)."
                                .into();
                        }
                        if crate::policy::gate("geo", eval.geo_confidence)
                            == crate::policy::Verdict::Drop
                        {
                            return "Error: Jev unsure (low confidence), no score.".into();
                        }
                        serde_json::to_string_pretty(&eval).unwrap_or_default()
                    }
                    Err(e) => format!("Error evaluating GEO via Jev: {}", e),
                }
            } else {
                // Free tier: deterministic keyless score, no model involved.
                let out = json!({
                    "target": target,
                    "query": query,
                    "keyless": crate::geo_keyless::score(&content, query),
                    "note": "keyless deterministic score; set TYPESAFE_API_KEY for Jev semantic judgment",
                });
                serde_json::to_string_pretty(&out).unwrap_or_default()
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
                            if crate::policy::gate("keywords", eval.confidence())
                                != crate::policy::Verdict::Drop
                            {
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
            let max_pages =
                args.get("max_pages")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(crate::crawl::DEFAULT_MAX_PAGES as u64) as usize;
            let sitemap = args
                .get("sitemap")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty());
            if let Some(s) = sitemap {
                if let Some(err) = safety_gate("seo_crawl", s) {
                    return err;
                }
            }
            let mut budget = crate::fetch::Budget::default();
            match crate::crawl::crawl_site_with_sitemap(
                url,
                max_pages,
                crate::fetch::FetchMode::Auto,
                &mut budget,
                sitemap,
            ) {
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
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
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
            let baseline_label = args
                .get("baseline_label")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(err) = safety_gate("seo_report", path) {
                return err;
            }
            let base_src = if !baseline_label.trim().is_empty() {
                match crate::rank::DbStore::open() {
                    Ok(db) => match db.load_baseline(baseline_label.trim()) {
                        Ok(Some(src)) => src,
                        Ok(None) => {
                            return format!(
                                "Error: no baseline '{}' stored.",
                                baseline_label.trim()
                            )
                        }
                        Err(e) => return format!("Error: read baseline: {e:#}"),
                    },
                    Err(e) => return format!("Error: open drift store: {e:#}"),
                }
            } else {
                if let Some(err) = safety_gate("seo_report", baseline) {
                    return err;
                }
                match crate::paths::read_user_file(baseline, &["json"]) {
                    Ok(src) => src,
                    Err(e) => return format!("Error: read audit JSON: {e}"),
                }
            };
            let base: crate::audit::DirectoryAuditReport = match serde_json::from_str(&base_src) {
                Ok(b) => b,
                Err(e) => return format!("Error: parse baseline JSON: {}", e),
            };
            match crate::paths::read_user_file(path, &["json"]) {
                Ok(c) => match serde_json::from_str::<crate::audit::DirectoryAuditReport>(&c) {
                    Ok(cur) => {
                        let actions = crate::rules::actions_for(&cur.findings);
                        // Drift is best-effort: a missing ledger yields no alerts.
                        let drift = crate::rank::DbStore::open()
                            .ok()
                            .and_then(|db| db.drift_alerts(200).ok())
                            .unwrap_or_default();
                        let out = json!({
                            "diff": crate::main_report_diff(&cur, &base),
                            "actions": actions,
                            "pairs": crate::audit::cannibalization_pairs(&cur),
                            "drift": drift,
                        });
                        serde_json::to_string_pretty(&out).unwrap_or_default()
                    }
                    Err(e) => format!("Error: parse audit JSON: {}", e),
                },
                Err(e) => format!("Error: read audit JSON: {}", e),
            }
        }
        "seo_check" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            if let Some(err) = safety_gate("seo_check", path) {
                return err;
            }
            let checked = match crate::paths::check_audit_path(path) {
                Ok(p) => p,
                Err(e) => return format!("Error: {e:#}"),
            };
            match crate::audit::audit_path(&checked) {
                Ok(rep) => {
                    let blocking = crate::rules::blocking_findings(&rep.findings).len();
                    serde_json::to_string_pretty(&json!({
                        "path": path, "pass_rate": rep.pass_rate,
                        "blocking": blocking,
                        "verdict": if blocking > 0 { "FAIL" } else { "PASS" },
                    }))
                    .unwrap_or_default()
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        "seo_visibility" => {
            // Rich struct alias: same engine as seo_cite_check, struct output.
            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if target.is_empty() || query.is_empty() {
                return "Error: seo_visibility needs both \"target\" and \"query\".".into();
            }
            if let Some(answer) = args.get("answer").and_then(|v| v.as_str()) {
                if !answer.trim().is_empty() {
                    return cite_verdict(target, query, answer, "answer", &[]);
                }
            }
            cite_verdict(target, query, &cite_prompt(query), "prompt", &[])
        }
        "seo_index" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if url.trim().is_empty() {
                return "Error: seo_index needs \"url\".".into();
            }
            serde_json::to_string_pretty(&crate::indexability::truth(url)).unwrap_or_default()
        }
        "seo_verify" => {
            let rules = args.get("rules").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            if let Some(err) = safety_gate("seo_verify", path) {
                return err;
            }
            let checked = match crate::paths::check_audit_path(path) {
                Ok(p) => p,
                Err(e) => return format!("Error: {e:#}"),
            };
            match crate::audit::audit_path(&checked) {
                Ok(rep) => {
                    let rows: Vec<serde_json::Value> = rules
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|id| {
                            let bare = id.strip_prefix("RULE-").unwrap_or(id);
                            let still = rep
                                .findings
                                .iter()
                                .any(|f| f.rule_id == bare || f.rule_id == id);
                            json!({"rule": bare, "pass": !still})
                        })
                        .collect();
                    serde_json::to_string_pretty(&rows).unwrap_or_default()
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        _ => format!("Error: unknown tool: {}", name),
    }
}
