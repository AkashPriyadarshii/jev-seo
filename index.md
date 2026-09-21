# jev-seo

FOSS zero-cost SEO & GEO search radar for developers and AI agents. Rust CLI + MCP. No subscriptions, no credit card.

## Install

```sh
cargo install jev-seo
export TYPESAFE_API_KEY=your_key_here
```

## Commands

- `jev-seo query "offline expense tracker android"` — live SERP competitor radar + Jev relevance rerank + gap analysis
- `jev-seo audit docs/` — batch on-page audit: title collisions, thin pages, AI slop, orphan links. Add `--min-pass 40` to fail CI below the floor.
- `jev-seo geo README.md --query "agentic skills"` — GEO citation probability 1-10 plus a 5-dimension composite, vs Perplexity, SearchGPT, Gemini Overviews
- `jev-seo schema index.html` — JSON-LD validator: SoftwareApplication, Article, Organization, Product + deprecation flags
- `jev-seo robots example.com` — GPTBot, ClaudeBot, PerplexityBot, Google-Extended, Bytespider permissions
- `jev-seo brief "fast local sqlite tui"` — competitor SERP synthesis to H2 outline + 150-word GEO direct answer
- `jev-seo sitemap sitemap.xml` — 50,000 URL limit, HTTPS canonical, hreflang check
- `jev-seo keywords "scraper"` — autocomplete discovery + Jev intent classification
- `jev-seo rank --domain crates.io --query "rust grep"` — rank drift tracked in local SQLite over time
- `jev-seo mcp` — stdio MCP server with the 8 tools above, straight into your coding agent

## Why

Semrush and Ahrefs cost $130/month. jev-seo scrapes free DuckDuckGo endpoints, audits local files with a native Rust binary under 30MB RAM, and scores AI citation likelihood through TypeSafe Jev typed decisions. One binary, one local SQLite file, no telemetry.

## Links

- Repo: https://github.com/AkashPriyadarshii/jev-seo
- Docs: https://jevseo.vercel.app/llms.txt
- Sitemap: https://jevseo.vercel.app/sitemap.xml
