# Project State: jev-seo

## Current Phase: v0.3.0 maxen — half done (local, not pushed)

You shipped `v0.1.2` on 26 September 2026. Automation bumped `0.1.3` after it, which was wrong. You corrected the map: the next version is `v0.3.0 maxen`, not `0.1.3` or `0.2`. `Cargo.toml` reads `0.3.0` locally. This repo stays local until you say go.

### Shipped in v0.3.0 half (local, not pushed — former 0.1.3 merged in)
- [x] `jev-seo serve` plugin bridge: `POST /mcp` Streamable HTTP + `GET /plugin.json` + `extensions/`
- [x] `capabilities --json` single source (`58 rules / 15 tools / commands`)
- [x] `watch --once/--every` sqlite baseline `watch-<slug>` with delta and drift
- [x] Deterministic `fix-plan audit.json --json`: `rule->patch->verify` per Finding, `fact/heuristic`, heuristic capped 0.65, blocking first
- [x] Crawl sitemap seeding via `robots.txt Sitemap:` + `/sitemap.xml` fallback, `sitemapindex` 1-level expand, `--sitemap <url>` override, start-URL `*.xml/*sitemap*` detection, `warn: no sitemap seed` (closes #3) + `seo_crawl {sitemap}`
- [x] `133 tests` green, `clippy -D warnings` clean (added `test_robots_sitemaps_and_sitemap_detection`)

### Shipped in v0.1.2 (26 September 2026, pushed)
- [x] W6-W8 + Phase A/B + Wave 1: PDF deck, vitals R51-R53, eval rigor, DFS opt-in, security, truthfulness, provenance
- [x] Live audits: `docs/audits/akashpriyadarshi-vercel-app-2026-09-24.md`, `docs/audits/typesafe-ai-2026-09-24.md`
- [x] Review-driven P1 + PH-comment fixes: R03 only on 2+ hops, `seo_extract` SSRF per-URL, `--no-jev` audit gate, R54 advisory/heuristic; R56 soft-404 probe
- [x] Basic auth / custom headers: `--user`/`--password` and repeatable `--header` on all target-site fetches
- [x] R58 broken-hreflang-cluster: cross-page graph flags noindexed or unreciprocated alternates, heuristic + advisory
- [x] `RULE_SET_VERSION 2026.09b`; registry `R01-R58`
- [x] 15-tool MCP: `seo_cite_check` (answer/sampling/engine/manual loop), `seo_gap` + `gsc gap`, keyless GEO, paid engine answers, drift alerts, citation bot count, llms.txt shape grade
- [x] Agentic loop kit: `drift baseline/compare/history`, `audit --digest`, `bundle`, pair judging, post-H1 excerpts, gate numbers in `docs/EVAL.md`

### Shipped on master (local + pushed docs)
- [x] Phase 0-4 skeleton, modules, crawl, llms, doctor, HTML reports (v0.1.1 on crates.io)
- [x] Rule engine R01-R57, CSV export, rescore, multi-backend fetch, gsc, SKILL.md, EVAL.md
- [x] W1 RunManifest (`run.json` + `ledger.json`), citation gate, completeness banners
- [x] W1 Max Jev fan-out, `jev-latest`, act 0.80, injection pre-screen, state pre-filter
- [x] W2 `--jev-budget`, `--no-jev`, Score legend 0-1, EVAL W2 protocol
- [x] W3 action tracker CSV, `explain`, `report --baseline`, narrative.json, hardened .gitignore
- [x] W4 pair cannibalization A×B, MCP `seo_explain`/`seo_report` (13 tools), README matrices
- [x] W5 single PDF builder, drop SpreadsheetML, drop launch DESIGN.md, reword session commits
- [x] README launch rewrite (What's new, full matrices, 22 Oct date, build-from-source)
- [x] 107 tests green, clippy clean

### Review fixes (two subagent passes, same session)
- [x] Aligned display windows to rule windows (title 30-60, desc 120-160)
- [x] R19 gated on density (no single-hit noise)
- [x] SSRF recheck on Jev homepage fetch + custom API endpoint overrides
- [x] Byte caps on aux bodies (Jina, robots, sitemap, llms, site fetch)
- [x] MCP `safety_gate` on `seo_crawl`, guarded reads on `seo_report`
- [x] Chain-exhaust reports R03, provider label per backend, rescore flag note
- [x] Citation gate on action/pair CSV exports, NaN guards on vitals
- [x] 85 tests green, clippy `-D warnings` clean
- [ ] Deferred: thin `main.rs` to `view.rs`, trait-gate HTTP/Jev, explicit Tavily opt-in

### Release v0.1.2 (2026-09-26)
- [x] Bumped `Cargo.toml` 0.1.1 → 0.1.2
- [x] Renamed CHANGELOG `[Unreleased]` → `[0.1.2] - 2026-09-26`
- [x] Tagged `v0.1.2`, pushed master + site + tag; GitHub Release binaries via workflow
- [ ] `cargo publish` to crates.io (needs registry token)
- [x] `npm publish` as `@akashpriyadarshii/jev-seo` (unscoped name blocked by registry)

## v0.3.0 maxen — remaining half (not yet coded — ask before you start)
- [ ] Rule breadth finish: R56 hardened + JS-only shell flag (`readable_text <300w && script>2×`) + rich-result required props per `@type` + R03 loop hardened
- [ ] JS-rendered verification: optional `--features js` (`chromiumoxide`) headless, default warns `unrendered shell?`; no dep until you opt in
- [ ] Memory & trends: per-engine GEO `geo_history` sqlite, `watch --diff-only`, `<svg>` sparkline trends in HTML/PDF, offline `--rescore` for every command
- [ ] Evidence layer for `jev-rewrite`: `CrawlReport`/`RunManifest` as `COMPATIBILITY REPORT` artifact, clean-room guards already shipped

## v0.4.0 (next, after maxen ships)
- [ ] Full `sitemap-index` recursion (>1 level), multi-host crawl projects, screenshot/social-preview checks
- [ ] Perf-budget CI gates (`--max-lcp/--max-cls`), SARIF output, `plugin check` custom rules
- [ ] Flagship demos `strace-rs` + `iproute2-rs` (ss) published via jev-seo case studies — per `jev.md` strategy

## Structural Backlog (from arch review, not versioned)
- [ ] Thin `src/main.rs`: extract printing to `src/view.rs`
- [ ] One central fetch helper with body caps and redirect revalidation
- [ ] Abstract HTTP and Jev behind traits for hermetic tests

## Post-v0.3.0 Backlog (carried)

- [ ] Robots bot list refresh: Claude-User, Claude-SearchBot, Perplexity-User, Meta-ExternalAgent, Applebot-Extended, DuckAssistBot, YouBot, Cohere-ai, MistralAI-User, amazonbot, Grok; wildcard and longest-match evaluation
- [ ] Sitemap index support, 50MB limit check, malformed-XML errors
- [ ] Schema array `@context`/`@type` handling, HowTo deprecation correction
- [ ] Doctor reports all keys (Tavily, DataForSEO, Jina, Firecrawl, GSC)
- [ ] Rank DB hardening: corrupt recovery, migration, busy timeout, first-run vs dropped-out
- [ ] GSC actionable errors (expired grant hints), INP rule, R03/R49 dedup
- [ ] Audit/crawl scoring parity, MCP schema bounds and error framing, brief outline upgrades
- [ ] Eval-log sampler command, `link` MCP tool
- [ ] Topical share (Phase C feature, Semrush Topical Gravity model): topic file holds query sets + brand domains + competitors; `jev-seo topic --file topics.json` runs existing SERP checks per query and aggregates brand share per topic vs competitors; free DDG default, DFS LLM-mention endpoints opt-in only, never claim local ChatGPT/Gemini measurement; trends over runs from rank DB; adjacent-topic suggestions via `query_fit`; MCP `seo_topic` tool; share metric carries repeat-run drift note in EVAL.md, no rank prediction claims
