//! Single source of truth for capability counts.
//! Never hand-edit counts elsewhere; import from here or query `jev-seo capabilities --json`.

use crate::rules::{RULE_SET_VERSION, RULES};

pub const MCP_TOOLS: usize = 15;

/// Ordered tool names matching mcp.rs tools/list.
pub const MCP_TOOL_NAMES: &[&str] = &[
    "seo_cite_check",
    "seo_gap",
    "seo_keywords",
    "seo_serp_inspect",
    "seo_audit",
    "seo_geo",
    "seo_schema",
    "seo_robots",
    "seo_brief",
    "seo_sitemap",
    "seo_crawl",
    "seo_llms",
    "seo_extract",
    "seo_explain",
    "seo_report",
];

/// Canonical command list.
pub const COMMANDS: &[&str] = &[
    "keywords",
    "query",
    "link",
    "audit",
    "geo",
    "schema",
    "robots",
    "brief",
    "rank",
    "sitemap",
    "crawl",
    "llms",
    "doctor",
    "explain",
    "report",
    "bundle",
    "drift",
    "gsc",
    "mcp",
    "serve",
    "capabilities",
    "watch",
];

pub fn as_json() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "rule_set": RULE_SET_VERSION,
        "rules": RULES.len(),
        "mcp_tools": MCP_TOOLS,
        "mcp_tool_names": MCP_TOOL_NAMES,
        "commands": COMMANDS,
    })
}
