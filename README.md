---
title: "jev-seo: FOSS Zero-Cost SEO & GEO Search Radar"
description: "Open-source, subscription-free alternative to Semrush and OpenSEO. Powered by TypeSafe AI Jev and local DuckDuckGo scraping."
---

<!--
Title: jev-seo - FOSS Zero-Cost SEO & GEO Audit Suite
Description: Open-source, subscription-free alternative to Semrush and OpenSEO. Powered by TypeSafe AI Jev and local scraping.
Keywords: seo, geo, generative engine optimization, typesafe ai, jev, foss, rust, cli, mcp, semrush alternative
-->

<div align="center">
  <h1>jev-seo</h1>
  <p><strong>FOSS, zero-cost SEO & GEO search radar for developers and coding agents</strong></p>
  <p>
    <a href="https://github.com/AkashPriyadarshii/jev-seo/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-0055ff.svg?style=flat-square" alt="MIT License" /></a>
    <a href="https://crates.io"><img src="https://img.shields.io/badge/language-Rust-0055ff.svg?style=flat-square" alt="Rust" /></a>
    <a href="https://typesafe.ai"><img src="https://img.shields.io/badge/oracle-TypeSafe%20Jev-0055ff.svg?style=flat-square" alt="TypeSafe Jev" /></a>
  </p>
  <p>By <strong>Akash Priyadarshi</strong></p>
  <p>
    <a href="#why-jev-seo">Why</a> •
    <a href="#quickstart">Quickstart</a> •
    <a href="#workflows">Workflows</a> •
    <a href="#architecture">Architecture</a> •
    <a href="#non-goals">Non-Goals</a>
  </p>
</div>

---

## Why jev-seo?

Semrush and Ahrefs cost upwards of $130 per month. OpenSEO still requires paid DataForSEO credit cards. Most SEO suites are bloated web dashboards filled with vanity charts.

- **Zero subscriptions (₹0)**: Scrapes DuckDuckGo HTML and suggest endpoints directly. No credit cards, no paid API keys.
- **TypeSafe Jev System One**: Semantic intent classification, competitive gap detection, and AI visibility (GEO) scored deterministically without conversational LLM hallucinations.
- **Agent native (MCP)**: Native stdio MCP server directly feeds keyword gaps and page audit scores into Claude Code, Gemini CLI, and Antigravity.
- **Local & private**: All rank tracking and audit histories persist in a single local SQLite database (`.jev-seo.db`).
- **Low RAM footprint**: Fast native Rust binary that runs in under 30MB RAM on resource-constrained machines.

## Quickstart

```bash
# Install via Cargo (crates.io)
cargo install jev-seo

# Set your TypeSafe key for semantic oracle scoring
export TYPESAFE_API_KEY=your_key_here

# Run a live SERP competitive radar check
jev-seo query "offline expense tracker android"

# Audit a markdown blog post or local HTML file
jev-seo audit content/posts/my-post.md --keyword "offline expense tracker"

# Track domain rankings over time (saved to local SQLite)
jev-seo rank track --domain mysite.com --keywords "rust cli, seo tools"

# Start the Agent MCP Server
jev-seo mcp
```

## Workflows

### 1. Keyword Research & Search Intent
```bash
jev-seo keywords "rust tui"
```
Fetches autocomplete branches and uses Jev `Choice` to categorize intent into Informational, Commercial, Navigational, or Transactional.

### 2. Live SERP Radar & Content Gaps
```bash
jev-seo query "best local markdown editor"
```
Pulls top 10 live SERP results, analyzes winning patterns, and highlights structural content gaps.

### 3. Generative Engine Optimization (GEO)
```bash
jev-seo geo https://mysite.com/features
```
Calculates citation probability for Perplexity, SearchGPT, and Gemini Overviews using Jev `Score` (1-10) and `Noul`.

### 4. Git Pre-Commit SEO Screener
```bash
git diff HEAD~1 | jev-seo diff
```
Screens changed markdown and HTML files for missing meta tags, broken link references, and AI-slop vocabulary.

## Architecture

```
jev-seo
├── src
│   ├── main.rs         CLI entrypoint & command dispatch
│   ├── cli.rs          Clap command definitions
│   ├── engine.rs       TypeSafe Jev client & speculative fan-out
│   ├── serp.rs         DuckDuckGo HTML & suggest scraper
│   ├── audit.rs        Markdown & HTML parser and meta verifier
│   ├── rank.rs         Local SQLite rank drift tracker
│   └── mcp.rs          Stdio JSON-RPC 2.0 MCP server
└── Cargo.toml
```

## Non-Goals

- No bloated electron or cloud web dashboards.
- No paid third-party API dependencies (no forced DataForSEO subscriptions).
- No generative AI prose synthesis (Jev is used strictly as a deterministic evaluation oracle).

---

## Ecosystem

Built alongside:
- [design-genius](https://github.com/AkashPriyadarshii/design-genius)
- [akash-design-engineering](https://github.com/AkashPriyadarshii/akash-design-engineering)
- [tdlib-android](https://github.com/AkashPriyadarshii/tdlib-android)
- [kharcha](https://github.com/AkashPriyadarshii/kharcha)

**Author:** Akash Priyadarshi (Patna, Bihar, India)  
GitHub: [@AkashPriyadarshii](https://github.com/AkashPriyadarshii) • Portfolio: [akashpriyadarshi.vercel.app](https://akashpriyadarshi.vercel.app) • LinkedIn: [in/akash-priyadarshi-1aa51b37a](https://linkedin.com/in/akash-priyadarshi-1aa51b37a)
