# Project State: jev-seo

## Current Phase: 0 - Initial MD Skeleton & Architecture
- [x] Initial MD Skeleton created on Desktop (`AGENTS.md`, `CLAUDE.md`, `STATE.md`, `CHANGELOG.md`, `README.md`, `docs/*`)
- [x] Integrate scouted building blocks (`ureq` direct DDG, `gray-matter-rs`, `fast-html-parser`, `YellowFrogio` GEO heuristics, `rusqlite` WAL) into PRD & ARCHITECTURE
- [x] Initialize Cargo workspace and project layout (`Cargo.toml`, `src/`)
- [x] Module 1: Keyword Research (`serp.rs` - DDG autocomplete suggest + Jev intent classification)
- [x] Module 2: SERP Competitor Radar (`serp.rs` - DDG HTML scraper + Jev gap analysis)
- [x] Module 3: Local Site & Markdown Auditor (`audit.rs` - AST/frontmatter crawler for meta, headings, link health)
- [x] Module 4: GEO & AI Visibility Scorer (`geo.rs` / `engine.rs` - Perplexity/SearchGPT citation probability via Jev Score/Noul)
- [x] Module 5: Local Rank Tracker (`rank.rs` - SQLite persistence & rank drift comparison)
- [x] Module 6: Native Agent MCP Server (`mcp.rs` - stdio JSON-RPC 2.0 server for Claude Code/Gemini CLI)
- [x] Unit & Integration Tests with mock fixtures (4/4 passed)
- [x] Built and verified release binary (`target/release/jev-seo.exe`)

## Phase 1: Content Quality & Hierarchy Audits (Completed)
- [x] Heading hierarchy skip-level detection (`H1 -> H3` without `H2`) in Markdown and HTML
- [x] Google Helpful Content and AI slop detection (em-dash density per 500 words plus 17 synthetic writing tells)

## Phase 2: Internal Linking & Keyword Cannibalization (Completed)
- [x] Internal link extraction across Markdown and HTML files
- [x] In-memory link graph with orphan page detection (pages receiving 0 inbound links)
- [x] Keyword cannibalization radar grouping colliding pages targeting identical keyword stems

## Phase 3: XML Sitemap & International Hreflang (Completed)
- [x] XML sitemap inspection (`<loc>`, `<lastmod>`, canonical HTTPS enforcement, 50,000 URL limit, query param detection)
- [x] International hreflang tag validation (flags malformed `en-UK`, verifies `x-default` fallback)
- [x] Native MCP server expansion (`seo_sitemap` tool added, 8 tools active)
- [x] Test suite expanded to 21 passing tests with zero clippy warnings
- [x] Production binary built and verified (`cargo build --release`)

## Phase 4: Live Crawl & Agent Readiness (Shipped in v0.1.1)
- [x] Live-site `crawl` command (broken links, redirect chains, orphans, timing, `--diff`, `--rescore`)
- [x] URL canonicalization for crawl dedup
- [x] 8-worker parallel fetch pool (std threads, no new dependencies)
- [x] Crawl health score with grades, area scores, impact-ranked actions, completeness notes
- [x] `llms` answer-engine readiness scorer with ranked actions
- [x] `audit --html` designed single-file report with grade scorecard
- [x] `doctor` environment command
- [x] Jev needs-review surfacing in `geo` output
- [x] MCP server at 10 tools (`seo_crawl`, `seo_llms`)
- [x] 37 passing tests, zero clippy warnings on binary target
- [x] README screenshots from real runs, personal paths scrubbed
- [ ] Commit, tag v0.1.1, and cut release binaries

## v0.2.0 Structural Backlog (from independent arch review)
- [ ] Thin `src/main.rs`: extract printing to `src/view.rs`
- [ ] One central fetch helper with body caps and redirect revalidation
- [ ] Abstract HTTP and Jev behind traits for hermetic tests
