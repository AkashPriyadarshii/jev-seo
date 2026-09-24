---
name: jev-seo
description: Free SEO and GEO audits for any site or docs folder. Use when the user asks to audit SEO, check rankings, score AI visibility, validate schemas, inspect robots.txt, crawl for broken links, or gate a deploy on search health. Runs a single Rust binary, no paid APIs.
---

# jev-seo skill

Run the binary. Prefer JSON output when chaining commands. Never invent scores; every number below comes from a command.

## Fast paths

- Full site picture: `jev-seo crawl <url> --max-pages 50` then `jev-seo llms <domain>`.
- Own-site truth: `jev-seo gsc auth`, then `jev-seo gsc query --site <url>` for free Search Console clicks and positions.
- Docs folder gate: `jev-seo audit docs/ --min-pass 60 --html report.html --pdf report.pdf --md report.md`.
- One question, one answer: `jev-seo geo <file> --query "<query>"`, `jev-seo robots <domain>`, `jev-seo schema <file>`, `jev-seo sitemap <url>`.
- Internal links: `jev-seo link docs/ --limit 10` suggests one contextual target per page, or `no_link`.
- Environment check first on a new machine: `jev-seo doctor`.

## Reading output

- `Health`, `Score`, and `Grade` lines are deterministic 0-100 values with A-F grades.
- `Top Actions` are ranked by impact with effort bands; ids like `RULE-R01` map to the rule registry (R01-R57) in `src/rules.rs`.
- New since v0.1.1: `explain` any rule id, `report --baseline` diffs, `crawl --vitals` PageSpeed checks, `query --provider dfs` opt-in DataForSEO.
- Jev contract: Choices offer `insufficient_context`, Flags print the runner-up, every fan-out appends to `~/.jev-seo/eval.jsonl`. Question version rides the spend line.
- `Completeness` says what was skipped (caps, missing sitemap, missing robots). A capped crawl is a sample, not a verdict.
- `Needs review` marks Jev answers below the confidence bar. Treat those lines as unverified.
- `--json` keeps stdout machine-clean; progress goes to stderr.

## Rules for agents

1. Crawl before judging. Never score pages the crawl has not fetched.
2. Quote the completeness line alongside any score.
3. Gate deploys with `audit --min-pass` or crawl health deltas via `--diff`, not vibes. `--fail-on blocking` (default) fails only on deterministic rules; judgment calls warn. `--forbid` names non-negotiables.
4. Export findings with `--csv` for spreadsheets; rebuild past runs with `--rescore`, no spend.
4. Jev needs `TYPESAFE_API_KEY`. Without it the tool still runs; semantic sections report local-only output.
5. Keep crawls polite: default 50 pages, 8 workers, robots.txt honored. Raise `--max-pages` deliberately.
