# Project State: jev-seo

## Current Phase: Unreleased → v0.1.2 (2026-10-22)

### Shipped on master (local, unpushed)
- [x] W6 designed PDF deck: cover, scorecard, findings by area, inventory, narrative, method
- [x] W7 PageSpeed vitals: `crawl --vitals`, R51-R53, offline-safe degrade
- [x] W8 eval rigor: second-judge protocol, wording A/B log, human-label table, registry snapshot test
- [x] DFS opt-in: `query --provider dfs`, key-gated, free fallback
- [x] GEO state fix (`b577aa2`): body copy scoring, deduped state, geo-only gate, model pin
- [x] Phase A security (`d6dc533`): MCP path guards, fail-closed safety/SSRF, bounded reads, stderr spend
- [x] Phase A-addition (`6f12e54`): escapes, question versioning, eval log, runner-up flags, `link`, meta verdict
- [x] Phase B truthfulness (`98c7bfb`): upgrade math, spend truth, rank labels, audit precision, provider honesty, R25/R33/R24/R41
- [x] Blocking vs advisory gates: 20 rules fail CI, 33 warn; `--fail-on`, tagged actions/CSV/explain
- [x] Wave 1 (`wave-1`): provenance, truth tags, llms honesty, 15-bot registry, rank provenance, forbid/max-critical, query_fit, retry, fixtures, examples, integrations doc, Princeton trio
- [x] Wave-1 additions: side-probability gating, opening state, reservation, probes, R54-R57, brief upgrades, durability, secret-scan CI, effective Markdown gates
- [x] Site audit recorded: `docs/audits/akashpriyadarshi-vercel-app-2026-09-24.md`
- [x] 107 tests green, clippy clean

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
