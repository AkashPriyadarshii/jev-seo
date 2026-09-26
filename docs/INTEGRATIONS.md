# Integrations: drive jev-seo from Python, Node, and agents

jev-seo is a Rust binary with machine-clean `--json` on every command and a
15-tool MCP server. Any language drives it through stdout or stdio. No SDK
needed; parse the JSON with your standard library.

## Stable JSON shapes

- Audit: `DirectoryAuditReport` — `pass_rate`, `reports[]` (per-file
  `AuditReport` with `title`, `checks[]`, `word_count`), `findings[]`
  (`rule_id`, `area`, `severity`, `kind` fact/heuristic, `observed_at`,
  `source`, `scope`, `evidence`, `fix`).
- Crawl: `CrawlReport` — `score`, `grade`, `areas[]`, `actions[]`
  (`id`, `priority`, `effort`, `impact`, `quick_win`), `broken[]`,
  `redirects[]`, `orphans[]`, `pages[]` (`source`, `fetch_cost`, `upgraded`).
- Rank/geo/brief/llms/schema/robots/sitemap: flat structs, same field names
  as the terminal labels. `run.json` adds `rule_set_version`, ledger, and
  validation; it is the portable snapshot agents should keep.
- Spend lines and progress go to stderr, never stdout.

## Python

```python
import json, subprocess

def audit(path, min_pass=0):
    p = subprocess.run(
        ["jev-seo", "audit", path, "--no-jev", "--json"],
        capture_output=True, text=True, check=False,
    )
    report = json.loads(p.stdout)  # stderr holds progress only
    blocking = [f for f in report["findings"]
                if f["rule_id"] in {"R01","R02","R09","R11","R13","R31","R33","R36"}]
    if report["pass_rate"] < min_pass or blocking:
        raise SystemExit(f"gate failed: {len(blocking)} blocking findings")
    return report
```

## Node

```js
import { execFileSync } from "node:child_process";

const out = execFileSync("jev-seo", ["crawl", process.argv[2], "--max-pages", "50", "--json"], { encoding: "utf8" });
const report = JSON.parse(out); // progress went to stderr
const paid = report.pages.filter(p => p.source !== "direct");
console.log(`health ${report.score} (${report.grade}), paid backends: ${new Set(paid.map(p => p.source)).size}`);
```

## Agents (Claude Code, Cursor, Gemini CLI)

Prefer the MCP server over shelling out: `jev-seo mcp` speaks stdio
JSON-RPC 2.0 with 15 tools (`seo_cite_check`, `seo_gap`, `seo_audit`,
`seo_geo`, `seo_crawl`, `seo_llms`, `seo_brief`, `seo_extract`,
`seo_explain`, `seo_report`, ...). Point the client at the binary; no API keys required for the
deterministic surface, `TYPESAFE_API_KEY` unlocks the semantic one.
`SKILL.md` in the repo root is the agent contract: fast paths, output
reading rules, and deploy-gating policy.
