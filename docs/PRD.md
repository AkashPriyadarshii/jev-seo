---
title: "Product Requirements Document (PRD): jev-seo"
description: "Zero-cost, agent-first SEO and GEO search radar specification replacing commercial suites with local scraping and TypeSafe AI System One."
---

# Product Requirements Document (PRD): jev-seo

## 1. Problem Statement
Commercial SEO tools (Semrush, Ahrefs) cost $120-$300+/month and are designed for marketing agencies, not developers or autonomous coding agents. OpenSEO provides an open-source web UI but still requires paid DataForSEO API credits. Developers writing technical blogs, landing pages, and documentation need an instant, ₹0, local CLI that audits search intent and AI visibility (GEO) without monthly bills or account lock-in.

## 2. Target Persona
- **Developer / Founder**: Needs to audit markdown posts, verify meta tags, and check competitor rankings directly from the terminal or Git hooks.
- **Autonomous Coding Agents (Claude Code, Gemini CLI, Cursor)**: Need structured, deterministic JSON/MCP outputs to self-correct SEO and content gaps during build and refactoring loops.

## 3. Key Functional Requirements
- **FR-1: Keyword Intent Radar (`serp.rs`)**: Retrieve autocomplete queries without API keys via DDG suggest endpoint (`https://duckduckgo.com/ac/?q={}&type=list`) and classify intent (Informational, Commercial, Navigational, Transactional) via Jev System One.
- **FR-2: Zero-Cost SERP Scraper (`serp.rs`)**: Direct `ureq` POST to `https://html.duckduckgo.com/html/` (`q={query}`) with anti-bot headers (<150ms round-trip, zero headless browser/node_modules).
- **FR-3: Semantic Competitive Gap Engine (`engine.rs`)**: Single-call speculative fan-out evaluating top 10 competitor snippets for content gaps and winning structure.
- **FR-4: Local Site & Markdown Auditor (`audit.rs`)**: Extract YAML/TOML frontmatter via `gray-matter-rs` and HTML AST via `fast-html-parser`. Run 12 deterministic SEO checks (title/meta length, heading hierarchy, image alts, canonical tags).
- **FR-5: Generative Engine Optimization (GEO) (`geo.rs` / `engine.rs`)**: Scored via Jev fan-out based on AI discovery heuristics (YellowFrogio AI-Discovery framework):
  - `Score` (1–10): Citation likelihood for AI synthesis engines (Perplexity, SearchGPT, Gemini Overviews).
  - `Noul`: Is a direct factual answer present in the first 150 words?
  - `Choice`: Primary content gap (`missing_statistics`, `generic_prose`, `no_step_by_step`, `outdated_examples`, `none`).
- **FR-6: Local SQLite Rank Tracker (`rank.rs`)**: Embedded `rusqlite` (WAL mode) tracking `keywords` and `rank_history` (domain, position, serp_url, timestamp) in `.jev-seo.db`.
- **FR-7: Agent MCP Server (`mcp.rs`)**: Native stdio JSON-RPC 2.0 protocol for Claude Code and Gemini CLI exposing 15 tools (`seo_keywords`, `seo_serp_inspect`, `seo_audit`, `seo_geo`, `seo_schema`, `seo_robots`, `seo_brief`, `seo_sitemap`, `seo_crawl`, `seo_llms`, `seo_extract`, `seo_explain`, `seo_report`, `seo_cite_check`, `seo_gap`).
- **FR-8: XML Sitemap & International Hreflang Auditor (`sitemap.rs`)**: Fetch and parse XML sitemaps to verify HTTPS enforcement, 50,000 URL boundaries, query parameter avoidance, and ISO hreflang tag correctness with required `x-default` fallbacks.
- **FR-9: Content Quality & Hierarchy Guard (`audit.rs`)**: Enforce strict heading progressions without skip-levels and scan for excessive em-dashes and synthetic AI writing terms.
- **FR-10: Link Graph & Cannibalization Radar (`audit.rs`)**: Map in-memory directory link graphs to detect orphan pages and group competing pages targeting identical keyword stems.
- **FR-11: Live-Site Crawler (`crawl.rs`)**: Parallel 8-worker BFS over one host seeded from sitemap.xml with robots.txt honored. Canonical dedup, per-page timing, redirect hop chains, health score with grades, area scores, and impact-ranked actions.
- **FR-12: Answer-Engine Readiness (`llms.rs`)**: Score llms.txt presence plus explicit AI crawler allows with ranked readiness actions.
- **FR-13: Ranked Actions & Grades (`actions.rs`)**: Shared P1-P3 priorities, effort bands, 0-100 impact, quick-win flags, and A-F grades on every command that scores.
- **FR-14: Multi-Format Reports (`audit.rs`)**: Designed single-file HTML with grade scorecard, pure-Rust multi-page PDF, and Markdown tables from the same audit data.
- **FR-15: Environment Doctor**: Version, API key presence (never printed), database state, and platform in one command.
- **FR-16: Capabilities (`capabilities.rs`)**: Single source `jev-seo capabilities --json` reports `{version, rule_set, rules, mcp_tools, commands}` for agents/docs.
- **FR-17: Plugin Bridge (`plugin_bridge.rs`)**: `jev-seo serve` HTTP bridge (`POST /mcp` Streamable HTTP, `GET /plugin.json`, `/extensions/*`) for ChatGPT Plugin Extensions.

## 4. Non-Functional Requirements
- **Performance**: Cold start < 60ms. Memory consumption < 30MB RAM.
- **Privacy**: Local-first. SERP queries and ranking records never leave the local machine.
- **Cost**: ₹0 for SERP data. Fractions of a cent per Jev evaluation ($0.042/Mtok).

## 5. Release History & Future Versions
- **v0.1.0**: Full CLI suite, MCP server, JSON-LD, robots, sitemap, hreflang, GEO radar, local audits.
- **v0.1.1**: Live crawl, canonical dedup, parallel fetch, llms readiness, ranked actions with grades, HTML/PDF/Markdown reports, doctor, needs-review surfacing, 10 MCP tools.
- **v0.1.2 on 26 September 2026**: rule engine R01-R58, unified rules scoring, CSV export, rescore, RunManifest + citation gates, Jev spend budget, action tracker CSV, `explain`, `report --baseline`, narrative.json, pair cannibalization A×B, 15-tool MCP (`seo_cite_check`, `seo_gap`, keyless GEO, drift alerts), skill-hard Jev thresholds, designed PDF deck, PageSpeed vitals, opt-in DataForSEO, eval second-judge protocol.
- **v0.3.0 maxen (current, merged — half done)**: single version that absorbs former 0.1.3 unreleased plus PRD 0.2.0 + both 0.3.0 scopes plus `jev.md` rewrite-engine evidence role. Shipped half: plugin bridge `jev-seo serve`, `capabilities --json`, `watch --once/--every`, deterministic `fix-plan`, crawl seeding via `robots.txt Sitemap:` + `sitemapindex` 1-level expand + `--sitemap <url>` + single-page warn (closes #3). Remaining half in code: (A) rule breadth — R56 hardened + JS-only shell flag + rich-result required props per @type + R03 loop; (B) JS-rendered verification — optional `--features js` behind `chromiumoxide`, default warns `unrendered shell?` via `readable_text <300w && script>2× text`, no dep until you opt in; (C) memory & trends — per-engine GEO sqlite `geo_history`, `watch --diff-only`, `<svg>` sparkline trends in HTML/PDF, `offline --rescore` for every command; (D) evidence layer for `jev-rewrite` — `CrawlReport`/`RunManifest` is the `COMPATIBILITY REPORT {cli,stdout,exit,fuzz,perf}` artifact, clean-room guards (`reject_private_url` + dot-file deny + same-host `Sitemap:`). Ponytail order: reuse `rules.rs`/`rank.rs`/`manifest.rs`, then stdlib svg, then native headless last.
- **v0.4.0 (next, scoped after maxen)**: full `sitemap-index` recursion (>1 level), multi-host crawl projects, screenshot/social-preview checks, perf-budget CI gates (`--max-lcp/--max-cls`), SARIF output, `plugin check` custom rules, `jev-rewrite` flagship demos `strace-rs` + `iproute2-rs` (ss) published via jev-seo case studies — per `jev.md` strategy.
- **v1.0 (proof & scale)**: measured eval note with repeat-run drift, rule verification rates, ecosystem identity `I build test-proven Rust replacements for Linux foundations`.
