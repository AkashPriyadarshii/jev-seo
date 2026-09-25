# Project Rules & Agent Instructions: jev-seo

## Overview
`jev-seo` is a 100% Free and Open Source (FOSS), agent-first SEO & Generative Engine Optimization (GEO) suite powered by TypeSafe AI Jev (System One) and local zero-cost scraping. An open-source, subscription-free alternative to Semrush, Ahrefs, and OpenSEO.
- Registry: **58 stable rules (R01-R58)** across 9 areas; **112 tests** green; rule-set build `2026.09b`.
- Free path costs ₹0 (DuckDuckGo + local files). Jev semantic calls are hard-capped (`--jev-budget`, default $0.25/run; `--no-jev` is fully offline). Paid backends (Tavily, DataForSEO, Jina, Firecrawl) are explicit opt-in only: flag + key, never silent.
- **v0.1.2 releases 22 October 2026.** No crates publish, no GitHub Release, no public push without explicit "go".

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
- **Links**: Portfolio (https://akashpriyadarshi.vercel.app), GitHub (https://github.com/AkashPriyadarshii)

## Live Site Deploy (site branch)
- Site files live on the **`site` branch** (never `master`).
- GitHub Pages: https://akashpriyadarshii.github.io/jev-seo/ auto-deploys on push to `site`.
- Vercel: https://jevseo.vercel.app (user-managed link, independent of Pages).
- Edit flow: `git checkout site` → edit → commit → push origin site. `master` stays pure Rust crate.
