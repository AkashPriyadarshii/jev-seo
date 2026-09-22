# Developer Handoff: jev-seo

## 1. Quick Setup
```bash
cd jev-seo
cargo check
```

## 2. Environment Variables
- `TYPESAFE_API_KEY`: Required for semantic Jev evaluations (`keywords`, `query`, `geo`, `audit --semantic`).
- When missing, `jev-seo` gracefully falls back to local AST checks and raw SERP tables without semantic Jev ratings.

## 3. Pre-Commit Check
Always run:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
git jev check
```

## 4. Test Harness & Diagnostics
The test harness in `src/tests.rs` contains 21 unit and integration tests verifying:
- Markdown and HTML on-page SEO parsers
- Heading hierarchy skip-level detection (`H1 -> H3`)
- Google Helpful Content and AI slop detection (em-dash density and 17 AI tells)
- Internal link graph analysis and orphan page detection
- Keyword cannibalization radar and stem collision grouping
- XML sitemap protocol enforcement (HTTPS, query parameter detection, 50,000 URL limit)
- International hreflang tag validation and fallback verification (`x-default`)
- Native stdio JSON-RPC 2.0 MCP server dispatch across all 8 tools

To test a live sitemap:
```bash
jev-seo sitemap https://example.com/sitemap.xml
```
