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
- **FR-7: Agent MCP Server (`mcp.rs`)**: Native stdio JSON-RPC 2.0 protocol for Claude Code and Gemini CLI exposing `seo_keywords`, `seo_serp_inspect`, `seo_audit`, and `seo_geo`.

## 4. Non-Functional Requirements
- **Performance**: Cold start < 60ms. Memory consumption < 30MB RAM.
- **Privacy**: Local-first. SERP queries and ranking records never leave the local machine.
- **Cost**: ₹0 for SERP data. Fractions of a cent per Jev evaluation ($0.042/Mtok).
