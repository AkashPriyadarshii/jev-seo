# CLAUDE.md for jev-seo

## Development Commands
- Build: `cargo build --release`
- Test: `cargo test`
- Run: `cargo run -- <subcommand>`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt --check`

## Tooling & Dependencies
- Rust 2021 edition
- HTTP & Transport: `ureq` (sync, lightweight, no Tokio bloat)
- SERP Provider: DuckDuckGo HTML (`html.duckduckgo.com/html/`) & Suggest API (`duckduckgo.com/ac/`) (₹0 cost, zero API keys)
- System One engine: TypeSafe AI Jev via `https://api.typesafe.ai/v1/systemone` ($TYPESAFE_API_KEY)
- AST & Markup: `gray-matter-rs` (Markdown frontmatter) + `fast-html-parser` (SIMD HTML parsing)
- Sitemap & Hreflang: `sitemap.rs` streaming XML parser with ISO region and HTTPS checks
- Local Cache / History: `rusqlite` (bundled SQLite with WAL mode in `.jev-seo.db`)
- CLI Framework: `clap` with derive features
- MCP Server: Stdio JSON-RPC 2.0 protocol for agent loops (8 native tools)

## Style & Constraints
- Maximum performance, zero runtime allocations on hot paths.
- Error handling with `thiserror` / `anyhow`.
- Pre-commit gates: `git jev check` or `jev-axi diff --staged`.
