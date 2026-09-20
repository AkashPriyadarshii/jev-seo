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
- Tools:
  - `seo_keywords`: Autocomplete variants and search intent classification.
  - `seo_serp_inspect`: Live SERP competitors and winning angles.
  - `seo_audit`: Local file or directory on-page SEO scan with orphan detection.
  - `seo_geo`: Generative Engine Optimization citation evaluation.
  - `seo_schema`: Schema.org JSON-LD structural and deprecation validator.
  - `seo_robots`: Live robots.txt and AI crawler permission checker.
  - `seo_brief`: SERP-driven heading outline and content brief generator.
  - `seo_sitemap`: XML sitemap, URL limits, HTTPS, and hreflang validator.
