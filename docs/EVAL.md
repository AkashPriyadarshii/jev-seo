# Evaluation: measured numbers, not claims

All figures below were measured on 2026-09-22 against live targets and local
fixtures with the unreleased working tree. Rerun any of them; commands included.

## Protocol (W2): how to re-measure

Run these after a rules or scoring change; record date, git SHA, and machine.

1. **Rule fact-check (local)**  
   Pick one finding per fired rule id from `audit --csv` output. Independently
   recompute with shell tools (`wc -w`, title length, `grep` for duplicates).
   Pass = evidence text matches the independent check. Failures go in this file.

2. **Crawl ×3 drift**  
   `jev-seo crawl <url> --max-pages 10 --json` three times, same flags.  
   Compare `score` (expect equal or ±1 when R42 timing fires) and finding id sets.
   Timing rules (R42) may differ; non-timing ids must match.

3. **Jev repeatability**  
   Same page file, `jev-seo geo <file> --query "<q>"` twice with the same
   `--jev-budget`. Compare `geo_score` and decisive band. Record input tokens
   from the spend line; budget skips must be 0.

4. **Second-judge (optional)**  
   Export one request’s state/questions; score answers with a different model
   or human label. Agreement ≠ accuracy; report both.

5. **Budget guard**  
   `jev-seo audit <dir> --jev-budget 0` must run rules-only with zero Jev
   requests (`ledger.json` `jev_requests` or skip counter ≥ 1 if suite attempted).

Update the tables below with new dates when you re-run.

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
| Full test suite | see `cargo test` |

## Cost

Local rules, crawl, and reports spend $0. Semantic paths call TypeSafe Jev at
$0.042 per million input tokens under `--jev-budget` (default 0.25 USD).
Each command prints a Jev spend line; `ledger.json` records requests, tokens,
cost, budget, and budget skips.

## Known limits

- Timing rules (R42) vary with the server; thresholds are engineering picks,
  not tuned against human labels.
- `--rescore` rebuilds findings, scores, and actions from saved pages, but
  keeps the saved pass rate and counts. Recrawl when the engine changed.
- Default crawl scope is 50 pages and 8 workers; larger sites are sampled
  and the report says so.
- Jev confidence bands are calibrated by the model vendor, not by us.
- Second-judge agreement is not human ground truth.
