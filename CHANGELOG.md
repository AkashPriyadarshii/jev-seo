# Changelog

All notable changes to `jev-seo` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Optional paid search backend: `query --provider tavily` (or auto when `TAVILY_API_KEY` is set) with free DuckDuckGo fallback, hardened browser headers, and clean `--json` stdout. Thanks to @jerryrat for the API path and header research in PR #2.
- Paid `/extract` endpoint as `seo_extract` MCP tool, search depth/topic flags on `query`, Tavily-compatible URL override.
- Multi-backend page fetch: direct first, Jina and Firecrawl upgrade weak bodies only, quality math picks, per-page source labels, shared credit budget with `--max-credits`.
- `gsc` command: free Search Console query data via OAuth device flow with owner-only token storage.
- Fifty-rule audit engine (`src/rules.rs`): stable R01-R50 registry across 9 areas with severity weights, reach factors, area scores, and a weighted overall blend.
- `crawl` scoring unified on the rules engine: findings, areas, and impact-ranked actions from one truth, stored in JSON for agents.
- `audit` findings from the same engine, CSV findings export for crawl and audit, and `audit --rescore` for offline rebuilds with backward-compatible JSON.
- Staged stderr progress with elapsed timers on audit and llms; stdout stays machine-clean.
- `SKILL.md` agent contract and `docs/EVAL.md` with measured repeatability, verification, timing, and cost figures.
- Project repository skeleton, PRD, architecture, design specifications, and agent contracts.
- XML sitemap inspection command `sitemap` validating `<loc>`, `<lastmod>`, canonical HTTPS links, parameter pollution, and 50,000 URL limits.
- International hreflang tag validation verifying language and country pairs and enforcing `x-default` fallbacks.
- Heading hierarchy skip-level validation in Markdown and HTML audits.
- Google Helpful Content and AI slop detection tracking em-dash frequency and scanning 17 synthetic writing markers.
- Internal link graph analysis detecting orphan pages with zero inbound links across audited directories.
- Keyword cannibalization radar identifying colliding pages targeting identical multi-word keyword stems.
- MCP `initialize` handshake, silent notifications, parse-error replies, and `isError` on tool failures.
- Shared guarded file reader for CLI and MCP tools plus a Jev safety classifier blocking secret-looking targets.
- SSRF block on remote fetches covering literal hosts, resolved DNS, and redirect landings.
- Confidence-gated Jev verdicts with per-command thresholds in `src/policy.rs`; strict response parsing and 8k state truncation.
- Composite five-dimension GEO score, intent routing hints, and per-command speculative fan-out.
- SERP relevance rerank with per-section confidence gating.
- `audit --min-pass` CI gate with a GitHub Actions workflow, GEO score trend history, and home-directory database.
- Expanded test harness to 27 passing unit and integration tests.

## [0.1.1] - 2026-09-22

### Added
- Live-site `crawl` command: parallel 8-worker BFS over one host seeded from sitemap.xml, robots.txt honored, reporting broken links, redirect chains with hop detail, orphan pages, and per-page timing with slow-page flags; `--diff` compares against the last SQLite snapshot; `--rescore` rebuilds score and actions offline from saved JSON.
- URL canonicalization for crawl dedup: lowercase host, tracking-param stripping, `/index.html` folding, single trailing-slash policy.
- Crawl health score with letter grades, per-area scores (links, redirects, performance), and impact-ranked actions with quick-win flags.
- `llms` command scoring answer-engine readiness: llms.txt presence, explicit AI crawler allows, sitemap advertisement, robots.txt presence, with ranked actions.
- `audit --html PATH` writing a designed single-file HTML report with grade scorecard and method appendix.
- `doctor` command reporting version, API key presence, database state, and platform.
- Jev needs-review surfacing: `policy::needs_review` lists low-confidence question ids instead of silently trusting them, wired into `geo` output.
- `seo_crawl` and `seo_llms` MCP tools, expanding the server to 10 tools.
- Staged stderr progress with elapsed timers on crawls; stdout stays machine-clean JSON under `--json`.
- Binary version and user-agent strings now read from `Cargo.toml`, no hardcoded versions.
- README screenshots from real runs, CI badge, personal paths scrubbed from docs.

## [0.1.0] - 2026-09-21

### Added
- First release: full CLI suite, MCP server, JSON-LD/robots/sitemap/hreflang/SEO audits, GEO radar.
- CI (`cargo check` + `cargo test`) and release workflow: 5-target matrix binaries with SHA256.
