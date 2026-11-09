# Project Rules & Agent Instructions: jev-seo

## Overview
`jev-seo` is a 100% Free and Open Source (FOSS), agent-first SEO & Generative Engine Optimization (GEO) suite powered by TypeSafe AI Jev (System One) and local zero-cost scraping. An open-source, subscription-free alternative to Semrush, Ahrefs, and OpenSEO.
- Registry: **64 stable rules (R01-R64)** across 9 areas; **142 tests** green; rule-set build `2026.09b`.
- Free path costs ₹0 (DuckDuckGo + local files). Jev semantic calls are hard-capped (`--jev-budget`, default $0.25/run; `--no-jev` is fully offline). Paid backends (Tavily, DataForSEO, Jina, Firecrawl) are explicit opt-in only: flag + key, never silent.
- **v0.4.0 unreleased** (Search CI: `check` + `verify`, `.jev-seo/policy.toml`, regional SERP, GSC inspect/winners/losers, Bing sites/queries, SARIF/JUnit, R59-R64, R37 informational, citation struct, indexability table, 4 new MCP tools, `jev://` resources, evidence snapshots, portable manifest). `v0.3.0` shipped 2 October 2026 (64 rules, 142 tests). `v0.1.2` shipped 26 September 2026. Former unreleased `0.1.3` was merged into `v0.3.0`.

## Core Directives
1. **FOSS & Zero-Cost Free Path**: MIT licensed. DuckDuckGo + local files + SQLite drift are free forever. No paid API is *required*; DFS/Tavily/Jina/Firecrawl exist only behind opt-in flags.
2. **TypeSafe Jev System One First**: Use deterministic primitives (`Choice`, `Score`, `Noul`) for intent classification, content gap analysis, and GEO citation scoring. Never use Jev for arithmetic, character counts, or generative prose. Choice/Score act at confidence 0.80; anything under prints `[verify]` or a withheld number, never a confident fake.
3. **Speculative Fan-out**: Ingest state once, batch independent questions into a single request.
4. **Hardware & Resource Limits**: Optimize for low-RAM (Windows 11, 8GB total memory). Fast cold starts, streaming output, zero heavy headless browser dependencies by default. (Raw-vs-rendered DOM diff is scoped as its own future version, not bolted on.)
5. **Humanizer & Stop-Slop**: All public docs, READMEs, and commit messages must strictly avoid AI vocabulary, passive voice, and boilerplate.
6. **Gates over guesses**: 22 rules are Blocking (fail CI via `--fail-on`), 36 are Advisory warnings. `fact`/`heuristic` truth tags on every finding; heuristic rules never fail a build silently.
7. **Commit locally, push on "go"**: never push `master` or `site` without an explicit go. `[skip ci]` on docs/asset-only commits.

## Ecosystem Footer Standard
Every marketing page or documentation footer must reference:
- **Ecosystem**: `jev-superpowers`, `jev-curate`, `tdlib-android`, `kharcha`
- **Author**: Akash Priyadarshi (Patna, Bihar, India)
- **Links**: Portfolio (https://akashpriyadarshi.is-a.dev), GitHub (https://github.com/AkashPriyadarshii)

## Live Site Deploy
- Site files live in **`master/site/`** (vendored, `linguist-vendored`). GitHub Pages is off; Vercel serves `site/` as canonical. Edit `site/` on `master`; push on go.
- GitHub Pages: https://akashpriyadarshii.github.io/jev-seo/ auto-deploys on push to `site`.
- Vercel: https://jevseo.vercel.app (user-managed link, independent of Pages).
- Edit flow: edit `site/` on `master`, commit, push on go. `master` stays pure Rust crate plus vendored site.
