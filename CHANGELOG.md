# Changelog

All notable changes to `jev-seo` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Review-driven fixes from three real-product comments: basic auth (`--user`/`--password`) and custom request headers (`--header`, repeatable) applied to every target-site fetch so staging sites behind a login wall become auditable pre-launch; R58 broken-hreflang-cluster check spanning pages (noindexed or unreciprocated alternates), heuristic and advisory; R56 soft-404 verified against the "closed but still 200" pattern with a probe-gate test; Blocking→Advisory reclassification for R03 single-hop redirects and R54 templated metadata.
- Wave-1 additions: side-probability Score decisiveness, opening-after-H1 state, conditional and policy-page question filtering, budget reservation with missing-usage estimate, soft-404/temp-redirect/host-variant probes, crawl score caps, R54 templated metadata, R55 uncited claims, brief competitor exclusion, report trust footer, atomic ledger writes, registry invariant test.
- Test harness at 112 passing unit and integration tests with zero clippy warnings.
- Premortem hardening: scope-aware effective gates (template-owned tags warn on Markdown), JSON-before-gates contract, zero clippy under `-D warnings`, rank observation trail, link budget flags, crate include list, secret-scan CI.
- Wave 1 evidence foundations: finding provenance (`observed_at`, `source`, `RULE_SET_VERSION` in manifests), fact/heuristic truth tags on findings/CSV/actions/explain, `RULE-` gate truth split.
- Wave 1 CI invariants: `audit --forbid R31,R36` and `--max-critical 0` above the class gate.
- Wave 1 Jev depth: landing-match `query_fit` Noul, retry with backoff on 429/5xx, committed good/broken gate fixtures, example audit artifacts, question A/B log rows.
- Wave 1 reach: `docs/INTEGRATIONS.md` Python/Node/MCP quickstarts, Princeton trio in the brief prescription, llms.txt honesty wording, 15-bot registry with training/search split, rank observation provenance table.
- Blocking vs advisory CI gates: 20 deterministic rules fail the build, 33 judgment calls print as warnings; `audit --fail-on blocking|all`, gate class on every action line, CSV row, and `explain` card.
- `link` command: Jev Choice internal-link suggestions per audited page over pre-filtered destinations, with a `no_link` escape hatch.
- Jev contract hardening: `insufficient_context` escape on intent and gap Choices, `QUESTION_VERSION` pinned in spend lines and the ledger, every fan-out appended to `~/.jev-seo/eval.jsonl` for re-evaluation, runner-up intent printed on Flag verdicts, closed `keep`/`rewrite`/`missing` meta-description verdict in the page suite.
- GEO scoring fix: HTML inputs now score extracted body copy instead of head markup, one deduped state field, geo display gates on geo confidence only, resolved model build pinned in ledger and spend line.
- Security hardening: MCP file tools canonicalize paths and refuse dot-directories and system trees, safety classifier fails closed on check errors, SSRF fails closed on DNS failure with decimal/hex/octal and IPv4-mapped IPv6 coverage, bounded crawl reads, spend line on stderr for clean `--json`.
- Crawl truthfulness: text-based reader-upgrade gate, links always resolve from direct HTML, billed backends record even when direct copy wins, free-tier Jina excluded from paid lists, health blends measured areas only, orphans emit R25 with redirect-target inbound credit.
- Audit precision: order-insensitive meta/canonical parsing, all-three OG tags required, empty alt accepted as decorative, leading heading jumps flagged, inline HTML headings in Markdown counted, code-stripped word counts, broken JSON-LD fires R33, opening size fires R24/R41.
- Search honesty: unknown `--provider` values bail with the valid list, explicit Tavily warns like DFS, labels name the backend that actually served, rank prints the real top-N window, `brief --provider` passthrough, validated and capped `extract`.
- Jev sample path: homepage judges first, Markdown excerpts send prose without frontmatter, drops print a note.
- Test harness at 112 passing unit and integration tests with zero clippy warnings.
- Designed PDF deck from one manifest: cover, scorecard with impact list, findings by area, page inventory, narrative, method appendix (`audit --pdf`).
- PageSpeed vitals on `crawl --vitals`: keyless lab LCP/CLS/INP for the start URL with R51-R53 (slow LCP, layout shift, lab-only note); degrades cleanly offline.
- Eval rigor: second-judge protocol, wording A/B log, and human-label sample tables in `docs/EVAL.md`; question-registry snapshot test fails on silent Jev wording edits.
- Optional DataForSEO backend: `query --provider dfs` with `DATAFORSEO_USERNAME`/`DATAFORSEO_PASSWORD` (plus `DATAFORSEO_API_URL` override); explicit opt-in only, falls back to free scrape on error.
- Hardening from two review passes: display windows match rule windows, R19 needs density, SSRF recheck on Jev fetch and API overrides, byte caps on aux bodies, MCP guards on crawl/report tools, chain-exhaust reports R03, citation gates on CSV exports.
- Pair cannibalization: expand each multi-file stem into explicit A×B conflict pairs with word-count winner; print top pairs on audit, table in Markdown, `audit --pairs-csv`.
- MCP tools `seo_explain` and `seo_report` (score delta + actions + pairs); tools list is now 13.
- Action tracker export: `audit --actions-csv` (same ranked list as terminal; Excel opens CSV).
- `jev-seo explain R19|RULE-R19`: stable rule card (area, severity, effort band, fix); unknown ids fail closed.
- `jev-seo report --baseline`: diff two saved audit JSONs (score delta, rules cleared/new) with optional action-tracker CSV and `--json`.
- Optional `narrative.json` beside audit exports (`references/narrative.md`): required executive_summary/strengths/risks/plan keys, hard refusal of unknown action IDs, unverified-number and plan effort-band warnings; absent file embeds an automatic evidence-only summary labeled automatic.
- RunManifest contract (`src/manifest.rs`): frozen `run.json` + `ledger.json` with schema 1.0, citation gate on report writers, completeness banners on audit/crawl/llms scores.
- Max Jev fan-out: page suite on audit sample, site+GEO on crawl homepage, richer GEO/page/keyword/brief questions, MCP `seo_keywords`/`seo_geo` wired to the same suites; model alias `jev-latest`; usage tokens recorded after each call.
- Skill-hard thresholds: Choice/Score act at confidence 0.80; dedicated injection Noul pre-screen before quality suites; state pre-filter before truncate.
- `--manifest <DIR>` on `audit` and `crawl` to place companion artifacts.
- `--no-jev` and `--jev-budget <USD>` on audit, crawl, and geo (default $0.25 hard cap).
- `audit --pdf` and `audit --md` report writers: pure-Rust multi-page PDF and Markdown tables from the same audit data.
- `gsc` command: free Search Console query data via OAuth device flow with owner-only token storage.
- Optional paid search backend: `query --provider tavily` (or auto when `TAVILY_API_KEY` is set) with free DuckDuckGo fallback, hardened browser headers, and clean `--json` stdout. Thanks to @jerryrat for the API path and header research in PR #2.
- Paid `/extract` endpoint as `seo_extract` MCP tool, search depth/topic flags on `query`, Tavily-compatible URL override.
- Multi-backend page fetch: direct first, Jina and Firecrawl upgrade weak bodies only, quality math picks, per-page source labels, shared credit budget with `--max-credits`.
- Fifty-rule audit engine (`src/rules.rs`): stable R01-R50 registry across 9 areas with severity weights, reach factors, area scores, and a weighted overall blend.
- `crawl` scoring unified on the rules engine: findings, areas, and impact-ranked actions from one truth, stored in JSON for agents.
- `audit` findings from the same engine, CSV findings export for crawl and audit, and `audit --rescore` for offline rebuilds with backward-compatible JSON.
- Staged stderr progress with elapsed timers on audit and llms; stdout stays machine-clean.
- `SKILL.md` agent contract and `docs/EVAL.md` with measured repeatability, verification, timing, and cost figures.
- Project repository skeleton, PRD, architecture, design specifications, and agent contracts.
- XML sitemap inspection command `sitemap` validating `<loc>`, `<lastmod>`, canonical HTTPS links, parameter pollution, and 50,000 URL limits.
- International hreflang tag validation verifying language and country pairs and enforcing `x-default` fallbacks.
- Heading hierarchy skip-level validation in Markdown and HTML audits.
- Google Helpful Content and AI slop detection tracking em-dash frequency and scanning 17 synthetic writing markers.
- Internal link graph analysis detecting orphan pages with zero inbound links across audited directories.
- Keyword cannibalization radar identifying colliding pages targeting identical multi-word keyword stems.
- MCP `initialize` handshake, silent notifications, parse-error replies, and `isError` on tool failures.
- Shared guarded file reader for CLI and MCP tools plus a Jev safety classifier blocking secret-looking targets.
- SSRF block on remote fetches covering literal hosts, resolved DNS, and redirect landings.
- Confidence-gated Jev verdicts with per-command thresholds in `src/policy.rs`; strict response parsing and 8k state truncation.
- Composite five-dimension GEO score, intent routing hints, and per-command speculative fan-out.
- SERP relevance rerank with per-section confidence gating.
- `audit --min-pass` CI gate with a GitHub Actions workflow, GEO score trend history, and home-directory database.
- Test harness at 112 passing unit and integration tests with zero clippy warnings.
- README rewritten for launch: What's new table, full command and MCP matrices, v0.1.2 date, build-from-source path. Ships as v0.1.2 on 2026-10-22.

## [0.1.1] - 2026-09-22

### Added
- Live-site `crawl` command: parallel 8-worker BFS over one host seeded from sitemap.xml, robots.txt honored, reporting broken links, redirect chains with hop detail, orphan pages, and per-page timing with slow-page flags; `--diff` compares against the last SQLite snapshot; `--rescore` rebuilds score and actions offline from saved JSON.
- URL canonicalization for crawl dedup: lowercase host, tracking-param stripping, `/index.html` folding, single trailing-slash policy.
- Crawl health score with letter grades, per-area scores (links, redirects, performance), and impact-ranked actions with quick-win flags.
- `llms` command scoring answer-engine readiness: llms.txt presence, explicit AI crawler allows, sitemap advertisement, robots.txt presence, with ranked actions.
- `audit --html PATH` writing a designed single-file HTML report with grade scorecard and method appendix.
- `doctor` command reporting version, API key presence, database state, and platform.
- Jev needs-review surfacing: `policy::needs_review` lists low-confidence question ids instead of silently trusting them, wired into `geo` output.
- `seo_crawl` and `seo_llms` MCP tools, expanding the server to 10 tools.
- Staged stderr progress with elapsed timers on crawls; stdout stays machine-clean JSON under `--json`.
- Binary version and user-agent strings now read from `Cargo.toml`, no hardcoded versions.
- README screenshots from real runs, CI badge, personal paths scrubbed from docs.

## [0.1.0] - 2026-09-21

### Added
- First release: full CLI suite, MCP server, JSON-LD/robots/sitemap/hreflang/SEO audits, GEO radar.
- CI (`cargo check` + `cargo test`) and release workflow: 5-target matrix binaries with SHA256.
