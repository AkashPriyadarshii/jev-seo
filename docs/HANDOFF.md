# Developer Handoff: jev-seo

## 1. Quick Setup
```bash
cd jev-seo
cargo check
```

## 2. Environment Variables
- `TYPESAFE_API_KEY`: Required for semantic Jev evaluations (`keywords`, `query`, `geo`, `audit --semantic`).
- `TAVILY_API_KEY`: Optional paid search backend for `query --provider tavily` (or auto when set). Without it everything runs free.
- `TAVILY_API_URL`: Optional override pointing at any Tavily-compatible endpoint or self-hosted proxy.
- `JINA_API_KEY`: Optional. Jina reader works keyless at base tier; a key raises limits for fetch upgrades.
- `FIRECRAWL_API_KEY`: Optional paid JS-rendered fetch for weak pages. Parked at zero spend unless `crawl --max-credits` allows it.
- `FIRECRAWL_API_URL`: Optional override for self-hosted Firecrawl.
- `GOOGLE_CLIENT_ID` + `GOOGLE_CLIENT_SECRET`: Optional. Desktop OAuth client for `gsc` Search Console data.
- `DATAFORSEO_USERNAME` + `DATAFORSEO_PASSWORD`: Optional paid DataForSEO live SERP for `query --provider dfs`. Parked unless explicitly requested; falls back to free scrape on error.
- `DATAFORSEO_API_URL`: Optional override for self-hosted DataForSEO-compatible endpoint.
- PageSpeed vitals need no key (`crawl --vitals`); keyless quota degrades to a skipped note.
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
The test harness in `src/tests.rs` contains 81 unit and integration tests verifying:
- Markdown and HTML on-page SEO parsers
- Heading hierarchy skip-level detection (`H1 -> H3`)
- Google Helpful Content and AI slop detection (em-dash density and 17 AI tells)
- Internal link graph analysis and orphan page detection
- Keyword cannibalization radar, stem collision grouping, and A×B conflict pairs
- XML sitemap protocol enforcement (HTTPS, query parameter detection, 50,000 URL limit)
- International hreflang tag validation and fallback verification (`x-default`)
- RunManifest citation gates, Jev budget cap, injection pre-screen
- Narrative contract (unknown action IDs, unverified numbers, effort bands)
- `explain` rule cards and `report --baseline` score diffs
- Native stdio JSON-RPC 2.0 MCP server dispatch across all 13 tools

Always run before commit:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

To test a live sitemap:
```bash
jev-seo sitemap https://example.com/sitemap.xml
```

## 5. Release checklist (v0.1.2 on 2026-10-22)
1. Bump `Cargo.toml` to `0.1.2`
2. Rename CHANGELOG `[Unreleased]` → `[0.1.2] - 2026-10-22`
3. Tag `v0.1.2` and `cargo publish`
4. Cut GitHub Release binaries from the release workflow