# System Architecture: jev-seo

## 1. System Overview

```
                          ┌───────────────────────────┐
                          │       CLI / Stdio MCP     │
                          └─────────────┬─────────────┘
                                        │
                 ┌──────────────────────┼──────────────────────┐
                 ▼                      ▼                      ▼
          ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
          │  DDG Scraper│        │ AST Crawler │        │SQLite Store │
          │ (HTTP/Regex)│        │ (Local MD)  │        │(.jev-seo.db)│
          └──────┬──────┘        └──────┬──────┘        └──────┬──────┘
                 │                      │                      │
                 └──────────────┬───────┘                      │
                                ▼                              │
                     ┌──────────────────────┐                  │
                     │   TypeSafe Jev       │                  │
                     │  (Speculative Fanout)│                  │
                     └──────────┬───────────┘                  │
                                ▼                              ▼
                     ┌─────────────────────────────────────────┐
                     │          Output Formatter               │
                     │ (ANSI Terminal / JSON / Agent Responses)│
                     └─────────────────────────────────────────┘
```

## 2. Core Modules

### `serp.rs` (Zero-Cost SERP & Autocomplete)
- Built on `ureq` for synchronous, sub-150ms HTTP calls without large async runtime overhead.
- Autocomplete: `GET https://duckduckgo.com/ac/?q={}&type=list` (zero-auth JSON suggestions).
- SERP Scraping: `POST https://html.duckduckgo.com/html/` with form-encoded `q={query}` and anti-detection desktop User-Agent headers.
- Parses results using `fast-html-parser` CSS selectors (`.result__title a`, `.result__snippet`).
- Zero Chromium, zero Playwright, <15MB RAM active.

### `engine.rs` (TypeSafe Jev System One Client)
- Direct endpoint: `POST https://api.typesafe.ai/v1/systemone`
- Authenticated via `$TYPESAFE_API_KEY`.
- Single-payload speculative fan-outs combining `Choice`, `Score`, and `Noul` primitives.
- Evaluates search intent, SERP competitive gaps, and direct answer placement.

### `audit.rs` (Static Markdown & HTML Auditor)
- Walks directories or targets single `.md`, `.mdx`, and `.html` files.
- Uses `gray-matter-rs` for frontmatter extraction (YAML/TOML title, description, canonical).
- Uses `fast-html-parser` for DOM inspection.
- Executes deterministic SEO checks: title length (30-65 chars), meta description length (80-165 chars), H1 presence/uniqueness, heading hierarchy progression without skip-levels, image alt tag completeness, canonical presence, and OpenGraph tags.
- Google Helpful Content and AI Slop Detector: Scans em-dash density per 500 words and 17 synthetic writing crutch terms.
- Internal Link Graph: Extracts link targets across the directory to identify orphan pages with zero inbound links.
- Keyword Cannibalization Radar: Normalizes titles and groups colliding pages targeting identical multi-word keyword stems.

### `sitemap.rs` (XML Sitemap & Hreflang Auditor)
- Fetches remote sitemaps via `ureq` or reads local XML files.
- Parses `<loc>`, `<lastmod>`, and `<xhtml:link>` elements using streaming regex matching.
- Enforces Google webmaster limits: flags files exceeding 50,000 URLs.
- Protocol Security: Flags insecure `http://` entries and enforces canonical `https://`.
- Parameter Pollution: Detects query strings (`?`) in sitemap paths.
- International Hreflang: Validates two-letter ISO 639-1 language codes and optional ISO 3166-1 alpha-2 region codes. Flags legacy errors (e.g. `en-UK` instead of `en-GB`) and enforces the `x-default` fallback tag.

### `geo.rs` (Generative Engine Optimization)
- Implements AI discovery heuristics (YellowFrogio AI-Discovery framework).
- Single fan-out payload to Jev:
  - `geo_score` (`Score` 1-10): Probability of being cited by Perplexity, SearchGPT, and Gemini Overviews.
  - `direct_answer` (`Noul`): Is a direct factual answer present in the first 150 words?
  - `content_gap` (`Choice`): Identifies primary gap (`missing_statistics`, `generic_prose`, `no_step_by_step`, `outdated_examples`, `none`).

### `rank.rs` (Local SQLite Persistence)
- Embedded `rusqlite` (bundled SQLite 3) with WAL mode enabled.
- Database file: `.jev-seo.db` (auto-gitignored).
- Schema:
  - `keywords (id INTEGER PRIMARY KEY AUTOINCREMENT, domain TEXT NOT NULL, term TEXT NOT NULL, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, UNIQUE(domain, term))`
  - `rank_history (id INTEGER PRIMARY KEY AUTOINCREMENT, keyword_id INTEGER NOT NULL REFERENCES keywords(id), position INTEGER, serp_url TEXT, checked_at DATETIME DEFAULT CURRENT_TIMESTAMP)`
- Computes rank delta across runs: `(prev: #5 -> now: #3 [▲2])`.

### `mcp.rs` (Stdio Model Context Protocol)
- JSON-RPC 2.0 loop over stdin/stdout.
- Tools (13):
  - `seo_keywords`: Autocomplete variants and search intent classification.
  - `seo_serp_inspect`: Live SERP competitor titles, snippets, URLs.
  - `seo_audit`: Local file/directory audit under the 57-rule engine.
  - `seo_geo`: GEO citation score with confidence gates.
  - `seo_schema`: JSON-LD structural and deprecation checks.
  - `seo_robots`: Robots.txt and AI crawler permissions.
  - `seo_brief`: SERP-driven content brief with outline.
  - `seo_sitemap`: XML sitemap and hreflang validation.
  - `seo_crawl`: Live-site BFS crawl with health score.
  - `seo_llms`: llms.txt and AI crawler readiness.
  - `seo_extract`: URL to markdown (key-gated paid path).
  - `seo_explain`: Rule card for stable R01-R57 ids.
  - `seo_report`: Diff two saved audit JSONs (score, rules, actions, pairs).

### `manifest.rs` (RunManifest & spend ledger)
- Writes `run.json` (schema 1.0) and `ledger.json` beside exports or `--manifest <DIR>`.
- Citation gate refuses report text that cites unknown action or rule ids.
- Completeness banners on audit/crawl/llms scores (what ran, what was skipped).
- Process-wide `--jev-budget` hard cap checked before each Jev POST.

### `narrative.rs` (Optional owner narrative)
- Loads `narrative.json` beside exports: required executive_summary, strengths, risks, plan.
- Hard-fails on unknown action IDs; warns on unverified numbers and effort-band mismatches.
- Absent file embeds an automatic evidence-only summary, labeled automatic.

### `crawl.rs` (Live-Site Crawler)
- Level-batched parallel fetch over 8 std threads, no async runtime.
- Seeds from `/sitemap.xml`, honors robots.txt, SSRF-guarded redirects with same-host pins and hop chains.
- Canonical dedup (tracking params, `/index.html`, slash policy), per-page timing, 2MB body cap.
- Weak bodies upgrade to Jina or Firecrawl backends under a shared credit budget.

### `fetch.rs` (Fetch Backends)
- Direct (free), Jina reader (keyless base tier), Firecrawl scrape (paid, key-gated).
- Quality math picks the best body; budgets cap paid spend with refunds only when no call happened.

### `rules.rs` (Rule Engine)
- Stable R01-R57 registry across nine areas with severity weights and reach factors. R51-R53 read homepage PageSpeed vitals.
- One scoring truth for crawl and audit output: area scores, weighted overall, impact-ranked actions.

### `actions.rs` (Ranked Actions)
- Shared P1-P3 priorities, effort bands, 0-100 impact, quick-win flags, A-F grades.

### `llms.rs` (Answer-Engine Readiness)
- Scores llms.txt presence plus explicit AI crawler allows with ranked readiness actions.

### `vitals.rs` (PageSpeed vitals)
- Keyless `pagespeedonline/v5` fetch for the crawl start URL with SSRF guard. Lab LCP/CLS/INP plus field-data presence; None on failure so crawls stay offline-safe. R51-R53 fire from it.

### `gsc.rs` (Search Console)
- Free first-party query data via OAuth device flow; refresh tokens stored with owner-only permissions.

### `policy.rs` (Confidence Gates)
- Per-command Jev thresholds plus needs-review surfacing for low-confidence answers.
