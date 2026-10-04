---
name: jev-audit
description: Audit any site or docs folder for SEO and GEO with jev-seo. Use when the user asks to audit SEO, check rankings, score AI visibility, validate schemas, inspect robots.txt, crawl for broken links, or gate a deploy on search health.
---

Audit with the jev-seo binary. Prefer JSON output when chaining commands. Never invent scores; every number comes from a command.

## Fast paths

- Full site picture: `jev-seo crawl <url> --max-pages 50` then `jev-seo llms <domain>`.
- Docs folder gate: `jev-seo audit docs/ --min-pass 60 --html report.html --md report.md`.
- One question, one answer: `jev-seo geo <file> --query "<query>"`, `jev-seo robots <domain>`, `jev-seo schema <file>`, `jev-seo sitemap <url>`.
- MCP over stdio: `jev-seo mcp`. Remote: `https://jev-seo.de.deplexo.com/mcp`.

## Reading output

- `Health`, `Score`, and `Grade` lines are deterministic 0-100 values with A-F grades.
- `Top Actions` are ranked by impact with effort bands; ids like `RULE-R01` map to the rule registry (R01-R64).
- `Completeness` says what was skipped (caps, missing sitemap, missing robots). A capped crawl is a sample, not a verdict.
- `--json` keeps stdout machine-clean; progress goes to stderr.

## Rules for agents

1. Crawl before judging. Never score pages the crawl has not fetched.
2. Quote the completeness line alongside any score.
3. Gate deploys with `audit --min-pass` or crawl health deltas, not vibes.
4. Keep crawls polite: default 50 pages, robots.txt honored.
