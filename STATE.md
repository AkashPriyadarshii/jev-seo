# Project State: jev-seo

## Current Phase: 0 - Initial MD Skeleton & Architecture
- [x] Initial MD Skeleton created on Desktop (`AGENTS.md`, `CLAUDE.md`, `STATE.md`, `CHANGELOG.md`, `README.md`, `docs/*`)
- [x] Integrate scouted building blocks (`ureq` direct DDG, `gray-matter-rs`, `fast-html-parser`, `YellowFrogio` GEO heuristics, `rusqlite` WAL) into PRD & ARCHITECTURE
- [x] Initialize Cargo workspace and project layout (`Cargo.toml`, `src/`)
- [x] Module 1: Keyword Research (`serp.rs` - DDG autocomplete suggest + Jev intent classification)
- [x] Module 2: SERP Competitor Radar (`serp.rs` - DDG HTML scraper + Jev gap analysis)
- [x] Module 3: Local Site & Markdown Auditor (`audit.rs` - AST/frontmatter crawler for meta, headings, link health)
- [x] Module 4: GEO & AI Visibility Scorer (`geo.rs` / `engine.rs` - Perplexity/SearchGPT citation probability via Jev Score/Noul)
- [x] Module 5: Local Rank Tracker (`rank.rs` - SQLite persistence & rank drift comparison)
- [x] Module 6: Native Agent MCP Server (`mcp.rs` - stdio JSON-RPC 2.0 server for Claude Code/Gemini CLI)
- [x] Unit & Integration Tests with mock fixtures (4/4 passed)
- [x] Built and verified release binary (`target/release/jev-seo.exe`)
