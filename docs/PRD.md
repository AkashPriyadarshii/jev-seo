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
- **FR-7: Agent MCP Server (`mcp.rs`)**: Native stdio JSON-RPC 2.0 protocol for Claude Code and Gemini CLI exposing 13 tools (`seo_keywords`, `seo_serp_inspect`, `seo_audit`, `seo_geo`, `seo_schema`, `seo_robots`, `seo_brief`, `seo_sitemap`, `seo_crawl`, `seo_llms`, `seo_extract`, `seo_explain`, `seo_report`).
- **FR-8: XML Sitemap & International Hreflang Auditor (`sitemap.rs`)**: Fetch and parse XML sitemaps to verify HTTPS enforcement, 50,000 URL boundaries, query parameter avoidance, and ISO hreflang tag correctness with required `x-default` fallbacks.
- **FR-9: Content Quality & Hierarchy Guard (`audit.rs`)**: Enforce strict heading progressions without skip-levels and scan for excessive em-dashes and synthetic AI writing terms.
- **FR-10: Link Graph & Cannibalization Radar (`audit.rs`)**: Map in-memory directory link graphs to detect orphan pages and group competing pages targeting identical keyword stems.
- **FR-11: Live-Site Crawler (`crawl.rs`)**: Parallel 8-worker BFS over one host seeded from sitemap.xml with robots.txt honored. Canonical dedup, per-page timing, redirect hop chains, health score with grades, area scores, and impact-ranked actions.
- **FR-12: Answer-Engine Readiness (`llms.rs`)**: Score llms.txt presence plus explicit AI crawler allows with ranked readiness actions.
- **FR-13: Ranked Actions & Grades (`actions.rs`)**: Shared P1-P3 priorities, effort bands, 0-100 impact, quick-win flags, and A-F grades on every command that scores.
- **FR-14: Multi-Format Reports (`audit.rs`)**: Designed single-file HTML with grade scorecard, pure-Rust multi-page PDF, and Markdown tables from the same audit data.
- **FR-15: Environment Doctor**: Version, API key presence (never printed), database state, and platform in one command.

## 4. Non-Functional Requirements
- **Performance**: Cold start < 60ms. Memory consumption < 30MB RAM.
- **Privacy**: Local-first. SERP queries and ranking records never leave the local machine.
- **Cost**: ₹0 for SERP data. Fractions of a cent per Jev evaluation ($0.042/Mtok).

## 5. Release History & Future Versions
- **v0.1.0**: Full CLI suite, MCP server, JSON-LD, robots, sitemap, hreflang, GEO radar, local audits.
- **v0.1.1**: Live crawl, canonical dedup, parallel fetch, llms readiness, ranked actions with grades, HTML/PDF/Markdown reports, doctor, needs-review surfacing, 10 MCP tools.
- **Unreleased backlog, ships as v0.1.2 on 22 October**: rule engine R01-R57, unified rules scoring, CSV export, rescore, RunManifest + citation gates, Jev spend budget, action tracker CSV, `explain`, `report --baseline`, narrative.json, pair cannibalization A×B, MCP `seo_explain`/`seo_report` (13 tools), skill-hard Jev thresholds, designed PDF deck, PageSpeed vitals, opt-in DataForSEO, eval second-judge protocol.
- **v0.2.0 (rule breadth)**: 50+ live-page rules across crawl, on-page, content, links, structured data, AI access, performance, and security areas with severity weights and reach factors; soft-404 detection; JavaScript-only page flagging; required rich-result property checks; redirect loop detection from recorded hop chains.
- **v0.3.0 (JS-rendered verification)**: raw-HTML vs rendered-DOM diff so "GPTBot allowed" is distinguishable from "GPTBot allowed into an empty shell". Requires a headless browser dependency (chromiumoxide or similar): adds a native runtime dependency, changes the execution model (spawn + render lifecycle, longer wall time per page), and needs its own resource budget for an 8GB host. Scoped as its own version deliberately — not bolted onto v0.1.2. R56 soft-404, llms, and robots checks keep working on raw HTML until then; the report calls out "unrendered" when a shell is suspected.
- **v0.3.0 (memory & trends)**: GEO prompt library with per-engine tracking in SQLite; free Search Console sync for real query data; `watch` mode reporting only what changed; trend charts in HTML reports; offline rescore for every command from saved JSON.
- **v1.0 (proof & scale)**: Measured evaluation note with repeat-run drift numbers and rule verification rates; multi-host crawl projects; sitemap-index traversal; screenshot and social-preview checks; performance budgets with CI failure gates; SARIF output for code scanning dashboards; plugin check interface for custom team rules.
