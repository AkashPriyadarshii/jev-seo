# Project State: jev-seo

## Current Phase: Unreleased → v0.1.2 (2026-10-22)

### Shipped on master (local, unpushed)
- [x] W6-W8 + Phase A/B + Wave 1: PDF deck, vitals R51-R53, eval rigor, DFS opt-in, security, truthfulness, provenance
- [x] Live audits recorded: `docs/audits/akashpriyadarshi-vercel-app-2026-09-24.md`, `docs/audits/typesafe-ai-2026-09-24.md`
- [x] Review-driven P1 + PH-comment fixes: R03 only on 2+ hops, `seo_extract` SSRF per-URL, `--no-jev` audit gate, R54 advisory/heuristic; R56 soft-404 verified with probe-gate test
- [x] Basic auth / custom headers: `--user`/`--password` (Basic) and repeatable `--header` applied to all target-site fetches (crawl/robots/sitemap/llms/site-scoring)
- [x] R58 broken-hreflang-cluster: cross-page graph flags noindexed or unreciprocated alternates, heuristic + advisory
- [x] RULE_SET_VERSION 2026.09b; registry R01-R58
- [x] 112 tests green, clippy clean
- [x] 15-tool MCP: `seo_cite_check` (answer/sampling/engine/manual loop), `seo_gap` + `gsc gap`, keyless GEO score, paid engine answers, drift alerts, citation bot count, llms.txt shape grade
- [x] Agentic loop kit: `drift baseline/compare/history`, `audit --digest`, `bundle`, pair judging, post-H1 excerpts, gate numbers in `docs/EVAL.md`
- [x] 132 tests green, clippy `-D warnings` clean, descriptions refreshed with TypeSafe Jev naming

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

### Before release (22 Oct)
- [ ] Bump `Cargo.toml` 0.1.1 → 0.1.2
- [ ] Rename CHANGELOG `[Unreleased]` → `[0.1.2] - 2026-10-22`
- [ ] Tag `v0.1.2`, `cargo publish`, GitHub Release binaries
- [x] Push master code + docs (`254e7ac`, `[skip ci]`, no release today)

## v0.2.0 Structural Backlog (from independent arch review)
- [ ] Thin `src/main.rs`: extract printing to `src/view.rs`
- [ ] One central fetch helper with body caps and redirect revalidation
- [ ] Abstract HTTP and Jev behind traits for hermetic tests

## Post-22-Oct Backlog (Phase C polish, after v0.1.2 ships)
- [ ] Robots bot list refresh: Claude-User, Claude-SearchBot, Perplexity-User, Meta-ExternalAgent, Applebot-Extended, DuckAssistBot, YouBot, Cohere-ai, MistralAI-User, amazonbot, Grok; wildcard and longest-match evaluation
- [ ] Sitemap index support, 50MB limit check, malformed-XML errors
- [ ] Schema array `@context`/`@type` handling, HowTo deprecation correction
- [ ] Doctor reports all keys (Tavily, DataForSEO, Jina, Firecrawl, GSC)
- [ ] Rank DB hardening: corrupt recovery, migration, busy timeout, first-run vs dropped-out
- [ ] GSC actionable errors (expired grant hints), INP rule, R03/R49 dedup
- [ ] Audit/crawl scoring parity, MCP schema bounds and error framing, brief outline upgrades
- [ ] Eval-log sampler command, `link` MCP tool
- [ ] Topical share (Phase C feature, Semrush Topical Gravity model): topic file holds query sets + brand domains + competitors; `jev-seo topic --file topics.json` runs existing SERP checks per query and aggregates brand share per topic vs competitors; free DDG default, DFS LLM-mention endpoints opt-in only, never claim local ChatGPT/Gemini measurement; trends over runs from rank DB; adjacent-topic suggestions via `query_fit`; MCP `seo_topic` tool; share metric carries repeat-run drift note in EVAL.md, no rank prediction claims
