---
title: "jev-seo: FOSS Zero-Cost SEO & GEO Search Radar"
description: "Open-source, subscription-free alternative to Semrush and OpenSEO. Powered by TypeSafe AI Jev and local DuckDuckGo scraping."
canonical: "https://github.com/AkashPriyadarshii/jev-seo"
keywords:
  - seo
  - geo
  - generative-engine-optimization
  - typesafe-ai
  - jev
  - rust
  - mcp
  - search-radar
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
    <a href="#non-goals">Non-Goals</a> •
    <a href="#ecosystem">Ecosystem</a>
  </p>
</div>

---

## Why jev-seo?

Semrush and Ahrefs cost upwards of $130 per month. OpenSEO still requires paid DataForSEO credit cards. Most SEO suites are bloated web dashboards filled with vanity charts.

- **Zero subscriptions (₹0)**: Scrapes DuckDuckGo HTML and suggest endpoints directly. No credit cards, no paid API keys.
- **TypeSafe Jev System One**: Semantic intent classification, competitive gap detection, and AI visibility (GEO) scored deterministically without conversational LLM hallucinations.
- **Batch Directory Auditing**: Audits hundreds of markdown/HTML files in <1 second, detecting title collisions, canonical mismatches, and thin pages.
- **2026 Schema.org Validator**: Deeply inspects JSON-LD schemas (`SoftwareApplication`, `Article`, `Organization`, `Product`) and flags deprecated schemas.
- **Robots.txt & AI Crawler Radar**: Evaluates permissions for AI bots (`GPTBot`, `ClaudeBot`, `PerplexityBot`, `Google-Extended`, `Bytespider`).
- **SERP Content Briefs**: Synthesizes competitor SERP snippets into an actionable content blueprint with H2 outlines and optimal 134-167 word GEO blocks.
- **Agent native (MCP)**: Native stdio JSON-RPC 2.0 MCP server directly feeds 7 SEO tools into Claude Code, Gemini CLI, and Antigravity.
- **Local & private**: All rank tracking and audit histories persist in a single local SQLite database (`.jev-seo.db`).
- **Low RAM footprint**: Fast native Rust binary that runs in under 30MB RAM on resource-constrained machines.

---

## Quickstart

```bash
# Install via Cargo (crates.io)
cargo install jev-seo

# Set your free TypeSafe key for semantic oracle scoring
export TYPESAFE_API_KEY=your_key_here

# Run a live SERP competitive radar check
jev-seo query "offline expense tracker android"

# Audit a single file or an entire documentation directory
jev-seo audit docs/

# Validate JSON-LD Schema.org markup
jev-seo schema index.html

# Check AI crawler permissions on a live domain
jev-seo robots github.com

# Generate a SERP-driven content brief
jev-seo brief "agentic skills" --limit 5

# Track domain rankings over time (saved to local SQLite)
jev-seo rank --domain "crates.io" --query "rust grep"

# Start the Native Agent MCP Server
jev-seo mcp
```

---

## Workflows

### 1. Batch Directory & File On-Page Audit
```bash
jev-seo audit content/posts/
```
Recursively scans all markdown and HTML files, identifies title collisions, detects thin content (<300 words), verifies canonical links, and flags image alt deficiencies.

### 2. JSON-LD Schema.org Validation
```bash
jev-seo schema content/guide.md
```
Validates required properties across `SoftwareApplication`, `Article`, `Organization`, and `Product`. Flags deprecated schema types (e.g. `HowTo` rich results).

### 3. Robots.txt & AI Crawler Inspection
```bash
jev-seo robots example.com
```
Audits `robots.txt` directives specifically for modern generative AI indexers:
* `GPTBot` (OpenAI model training)
* `ClaudeBot` & `anthropic-ai` (Anthropic)
* `PerplexityBot` (Perplexity citation engine)
* `Google-Extended` (Gemini training)
* `Bytespider` (ByteDance)

### 4. SERP-Driven Content Brief Generator
```bash
jev-seo brief "fast local sqlite tui" --markdown
```
Pulls live DuckDuckGo competitors, calculates optimal word count, and uses Jev System One fan-out to generate an actionable heading outline with a 150-word GEO direct-answer prescription.

### 5. Generative Engine Optimization (GEO)
```bash
jev-seo geo README.md --query "agentic skills framework for coding agents"
```
Calculates citation probability for Perplexity, SearchGPT, and Gemini Overviews using Jev `Score` (1-10) and `Noul`.

### 6. XML Sitemap & International Hreflang Auditor
```bash
jev-seo sitemap sitemap.xml
jev-seo sitemap https://example.com/sitemap.xml
```
Validates sitemaps against Google webmaster standards: enforces canonical HTTPS protocols, flags parameter pollution, verifies the 50,000 URL limit, and audits international `hreflang` codes (flagging malformed codes like `en-UK` vs `en-GB` and enforcing `x-default`).

### 7. Google Helpful Content & AI Slop Radar
Integrated into `jev-seo audit`:
* **Em-dash density scanner**: Flags excessive em-dash saturation (>2 per 500 words).
* **AI Vocabulary Detector**: Flags 17 pervasive AI boilerplate crutches (`delve`, `leverage`, `testament`, `foster`, `seamless`, `crucial`, `robust`, `landscape`, etc.).
* **Heading Hierarchy Auditor**: Flags illegal skip-levels (e.g. `H1 -> H3` without `H2`).
* **Internal Link Graph**: Pinpoints orphan pages with 0 inbound internal links across directories.
* **Keyword Cannibalization Radar**: Identifies competing articles targeting identical keyword stems.

---

## Architecture

```
jev-seo
├── src
│   ├── main.rs         CLI entrypoint & command dispatch
│   ├── engine.rs       TypeSafe Jev client & speculative fan-out
│   ├── serp.rs         DuckDuckGo HTML & suggest scraper
│   ├── audit.rs        Batch directory & file on-page meta auditor
│   ├── schema.rs       JSON-LD Schema.org structural & semantic validator
│   ├── robots.rs       Robots.txt & AI crawler permission analyzer
│   ├── brief.rs        SERP-driven content brief generator
│   ├── rank.rs         Local SQLite rank drift tracker (.jev-seo.db)
│   ├── sitemap.rs      XML sitemap & international hreflang auditor
│   ├── mcp.rs          Native stdio JSON-RPC 2.0 MCP server (8 tools)
│   └── tests.rs        Unit & integration test harness (21 tests passing)
├── Cargo.toml
└── README.md
```

---

## Non-Goals

- No bloated electron or cloud web dashboards.
- No paid third-party API dependencies (no forced DataForSEO subscriptions).
- No generative AI prose synthesis (Jev is used strictly as a deterministic evaluation oracle).

---

## Ecosystem

* [design-genius](https://github.com/AkashPriyadarshii/design-genius) — Autonomous design system intelligence for web applications
* [akash-design-engineering](https://github.com/AkashPriyadarshii/akash-design-engineering) — High-performance editorial brutalist design tokens and cookbook
* [tdlib-android](https://github.com/AkashPriyadarshii/tdlib-android) — Precompiled TDLib native binaries for all 4 Android ABIs
* [kharcha](https://github.com/AkashPriyadarshii/kharcha) — India-first offline-first UPI expense tracker for Android

---

## Author

**Akash Priyadarshi**  
Patna, Bihar, India  
* GitHub: [@AkashPriyadarshii](https://github.com/AkashPriyadarshii)  
* Portfolio: [akashpriyadarshi.vercel.app](https://akashpriyadarshi.vercel.app)  
* LinkedIn: [Akash Priyadarshi](https://linkedin.com/in/akash-priyadarshi-1aa51b37a)  
* Resume: [akashpriyadarshii.github.io/Resume](https://akashpriyadarshii.github.io/Resume/)  

**Social:** [X / Twitter](https://x.com/Akash__ydv001) • [Threads](https://www.threads.net/@akash.priyadarshii) • [Instagram](https://www.instagram.com/akash.priyadarshii/) • [Reddit](https://reddit.com/user/DragonfruitWeak2801)

---

*Zero-cost, agent-first SEO radar built with TypeSafe AI System One.*
