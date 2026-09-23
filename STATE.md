# Project State: jev-seo

## Current Phase: Unreleased → v0.1.2 (2026-10-22)

### Shipped on master (local + pushed docs)
- [x] Phase 0-4 skeleton, modules, crawl, llms, doctor, HTML reports (v0.1.1 on crates.io)
- [x] Fifty-rule engine, CSV export, rescore, multi-backend fetch, gsc, SKILL.md, EVAL.md
- [x] W1 RunManifest (`run.json` + `ledger.json`), citation gate, completeness banners
- [x] W1 Max Jev fan-out, `jev-latest`, act 0.80, injection pre-screen, state pre-filter
- [x] W2 `--jev-budget`, `--no-jev`, Score legend 0-1, EVAL W2 protocol
- [x] W3 action tracker CSV, `explain`, `report --baseline`, narrative.json, hardened .gitignore
- [x] W4 pair cannibalization A×B, MCP `seo_explain`/`seo_report` (13 tools), README matrices
- [x] W5 single PDF builder, drop SpreadsheetML, drop launch DESIGN.md, reword session commits
- [x] README launch rewrite (What's new, full matrices, 22 Oct date, build-from-source)
- [x] 72 tests green, clippy `-D warnings` clean

### Before release (22 Oct)
- [ ] Bump `Cargo.toml` 0.1.1 → 0.1.2
- [ ] Rename CHANGELOG `[Unreleased]` → `[0.1.2] - 2026-10-22`
- [ ] Tag `v0.1.2`, `cargo publish`, GitHub Release binaries
- [x] Push master code + docs (`254e7ac`, `[skip ci]`, no release today)

## v0.2.0 Structural Backlog (from independent arch review)
- [ ] Thin `src/main.rs`: extract printing to `src/view.rs`
- [ ] One central fetch helper with body caps and redirect revalidation
- [ ] Abstract HTTP and Jev behind traits for hermetic tests
