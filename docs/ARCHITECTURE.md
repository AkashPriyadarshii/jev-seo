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
- Executes 12 deterministic checks: title length (40-60 chars), meta description length (120-160 chars), H1 presence/uniqueness, heading hierarchy order, image alt tag completeness, and broken local anchors.

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
  - `seo_keywords`: Autocomplete + search intent classification.
  - `seo_serp_inspect`: Live SERP competitors and winning angles.
  - `seo_audit`: Local file or directory on-page SEO scan.
  - `seo_geo`: Generative Engine Optimization citation evaluation.
  - `seo_rank_track`: Domain ranking verification and historical logging.
