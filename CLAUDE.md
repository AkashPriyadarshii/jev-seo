# CLAUDE.md for jev-seo

## Live Site (deploy model)
- Site source lives on the **`site` branch**, not `master`. Multi-file static site: `index.html`, `styles.css`, `script.js`, `about.html`, `contact.html`, `privacy.html`, plus `vercel.json` / `.vercelignore`.
- **GitHub Pages**: https://akashpriyadarshii.github.io/jev-seo/ (source = `site` branch, root `/`). Auto-deploys on push to `site`.
- **Vercel**: https://jevseo.vercel.app (managed by the user, keeps its own link).
- To change the site: checkout/commit on `site` branch only. Never touch `site` from `master`.
- Local: `git checkout site && python -m http.server` to preview.

## Development Commands
- Build: `cargo build --release`
- Test: `cargo test`
- Run: `cargo run -- <subcommand>`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt --check`

## Tooling & Dependencies
- Rust 2021 edition
- HTTP & Transport: `ureq` (sync, lightweight, no Tokio bloat). Target-site fetches (crawl/robots/sitemap/llms/site-scoring) accept `--user`/`--password` (Basic) and repeatable `--header`; API providers never see them.
- SERP Provider: DuckDuckGo HTML free by default; optional Tavily (`--provider tavily`) and DataForSEO (`--provider dfs`) behind explicit opt-in flags + keys, free fallback on error.
- System One engine: TypeSafe AI Jev via `https://api.typesafe.ai/v1/systemone` ($TYPESAFE_API_KEY). `--no-jev` is a complete offline path (zero network); `--jev-budget` hard-caps USD before every POST (default 0.25).
- Rule engine: `src/rules.rs`, 58 stable ids R01-R58, `RULE_SET_VERSION` bumped on any registry change (current `2026.09b`). 22 Blocking / 36 Advisory via `gate()`; `truth_kind()` labels every finding fact/heuristic.
- AST & Markup: `gray-matter-rs` (Markdown frontmatter) + `fast-html-parser` (SIMD HTML parsing)
- Sitemap & Hreflang: `sitemap.rs` sitemap audit (loc/lastmod/HTTPS/50k limit, hreflang codes + x-default); R58 cross-page cluster check (noindexed or unreciprocated alternates) in `rules.rs`.
- Local Cache / History: `rusqlite` (bundled SQLite with WAL mode in `.jev-seo.db`)
- CLI Framework: `clap` with derive features
- MCP Server: Stdio JSON-RPC 2.0 protocol for agent loops (15 native tools: cite check, gap queue, plus 13 originals). Agent wiring lives in `SKILL.md`.

## Style & Constraints
- Maximum performance, zero runtime allocations on hot paths.
- Error handling with `thiserror` / `anyhow`.
- Pre-commit gates: `git jev check` or `jev-axi diff --staged`.
- Scores rank work; they never predict rankings or traffic. Low-confidence Jev verdicts print `[verify]`.
- No crates publish, no GitHub Release, no push without an explicit "go". v0.1.2 released 26 Sept 2026.
- Brag/demo output lives in `brag-output/` and is gitignored; never commit render artifacts.

- Profile: release-order touch 2026-09-22
