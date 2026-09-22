# Evaluation: measured numbers, not claims

All figures below were measured on 2026-09-22 against live targets and local
fixtures with the unreleased working tree. Rerun any of them; commands included.

## Repeatability: same target, three crawls

`jev-seo crawl https://jev-curate.vercel.app/ --max-pages 6 --json`, three runs:

| Run | Score | Findings |
|---|---|---|
| 1 | 99 | 10 |
| 2 | 99 | 11 |
| 3 | 99 | 12 |

Score is stable. Finding counts move by ±2 because R42 slow-page findings
follow real server timing, which is the correct behavior for a timing rule:
the variance is the server's, not the scorer's.

## Rule verification: independent re-checks

- R21 duplicate titles: fired zero times on `docs/`. Independent check
  (`grep -h "^# " docs/*.md | sort | uniq -d`) also returns empty. Agree.
- R18 thin page on `docs/DESIGN.md` cites 170 words. `wc -w` agrees.
- R10 title length on `docs/ARCHITECTURE.md` cites 28 chars. Ruler agrees.

## Timings (local machine)

| Command | Result |
|---|---|
| Cold start (`--help`) | 6ms |
| Local audit, 4 files | ~0.2s |
| Live crawl, 6 pages | 3-11s wall, server-bound (8 parallel workers) |
| Full test suite, 42 tests | ~5s |

## Cost

Local rules, crawl, and reports spend $0. Semantic paths (`geo`, `query`,
`audit --target-query`) call TypeSafe Jev at $0.042 per million input
tokens; no Jev spend occurred in the runs above. Budgets and cost ledgers
are printed before results, never after.

## Known limits

- Timing rules (R42) vary with the server; thresholds are engineering picks,
  not tuned against human labels.
- `--rescore` rebuilds findings, scores, and actions from saved pages, but
  keeps the saved pass rate and counts. Rescoring old files after an engine
  tweak mixes new rules with old tallies; recrawl when it matters.
- Default crawl scope is 50 pages and 8 workers; larger sites are sampled
  and the report says so.
- Jev confidence bands are calibrated by the model vendor, not by us.
