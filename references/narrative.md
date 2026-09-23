# narrative.json contract

Optional file in the audit export folder (beside `run.json`). The lead agent
writes it after reading the findings. If absent, the renderer embeds an
automatic evidence-only summary and labels it automatic.

```json
{
  "executive_summary": [
    "Verdict in plain language with the pass rate and what drives it.",
    "One or two issues that matter, citing action IDs such as RULE-R10."
  ],
  "strengths": ["Three to five things the tree does well, each tied to evidence."],
  "risks": ["Three to five things holding it back, each citing an action ID."],
  "plan": [
    {"horizon": "This week", "items": ["RULE-R10 Shorten titles (hours)"]},
    {"horizon": "This month", "items": ["..."]},
    {"horizon": "This quarter", "items": ["..."]}
  ],
  "closing": "Optional: what to measure after the fixes, or what the audit could not see.",
  "author": "optional name"
}
```

Required keys: `executive_summary`, `strengths`, `risks`, `plan`.

## Gate (enforced at load)

- Every `RULE-Rxx` / `LLMS-###` / `CRAWL-###` token in the file must exist
  in this run’s action list. Unknown IDs: **hard fail**, no report embed.
- Numbers larger than 10 (not dates/URLs/IDs) that match nothing in the
  findings/actions are listed as `unverified_numbers` (warning, not fail).
- Plan lines ending in `(hours|about a day|several days|a project)` must
  match the effort band of every action they cite (`effort_mismatches`).

## Writing rules

- Write for the file owner, not an SEO specialist. Name the consequence, then the fix.
- Every claim comes from the audit. No traffic, revenue, or ranking predictions.
- Separate rule facts from Jev judgments; mark Jev items “to verify” when flagged.
- Plan items start with the action ID and include the effort band.
- Plain words. No em dashes. No hype.
- Absolute words (“every”, “all”) only when the whole set was checked.
- Heuristic rules stay labeled heuristic in the narrative.
