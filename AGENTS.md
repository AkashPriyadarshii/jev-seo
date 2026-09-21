# Project Rules & Agent Instructions: jev-seo

## Overview
`jev-seo` is a 100% Free and Open Source (FOSS), zero-cost, agent-first SEO & Generative Engine Optimization (GEO) suite powered by TypeSafe AI Jev (System One) and local zero-cost scraping. An open-source, subscription-free alternative to Semrush, Ahrefs, and OpenSEO.

## Core Directives
1. **FOSS & Zero-Cost**: MIT licensed. No paid APIs required (no DataForSEO, no Search1API requirement). Uses free DuckDuckGo endpoints, local AST parsers, and TypeSafe Jev.
2. **TypeSafe Jev System One First**: Use deterministic primitives (`Choice`, `Score`, `Noul`) for intent classification, content gap analysis, and GEO citation scoring. Never use Jev for arithmetic, character counts, or generative prose.
3. **Speculative Fan-out**: Ingest state once, batch independent questions into a single request.
4. **Hardware & Resource Limits**: Optimize for low-RAM (Windows 11, 8GB total memory). Fast cold starts, streaming output, zero heavy headless browser dependencies by default.
5. **Humanizer & Stop-Slop**: All public docs, READMEs, and commit messages must strictly avoid AI vocabulary, passive voice, and boilerplate.

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
