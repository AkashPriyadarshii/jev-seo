# Visual & CLI Design Guidelines: jev-seo

## 1. Terminal Output Philosophy
- Practical brutalism. Dense, high-information-density tables.
- No decorative emojis, no metronomic rule-of-three formatting.
- Strict ANSI color mapping:
  - Cyan (`\x1b[36m`): Commands and URLs
  - Green (`\x1b[32m`): Winning metrics / Passing checks
  - Yellow (`\x1b[33m`): Warnings / Intermediate scores
  - Red (`\x1b[31m`): Critical gaps / Missing tags
  - Dim/Gray (`\x1b[90m`): Metadata, timestamps, latency

## 2. Table Formats

### Competitor SERP Output
```
Position  Domain              Intent           GEO Score  Gap Warning
─────────────────────────────────────────────────────────────────────────────
#1        docs.rs             Informational    9/10       Strong technical code
#2        github.com          Informational    8/10       Active issue threads
#3        medium.com          Commercial       4/10       AI-slop / thin content
```

### Page Audit Output
```
File: content/posts/my-post.md
Target Keyword: "offline expense tracker"

Check                 Status    Details
─────────────────────────────────────────────────────────────────────────────
Title Tag             PASS      48 chars (optimal 40-60)
Meta Description      WARN      Missing frontmatter description
H1 Match              PASS      Contains target keyword
Heading Hierarchy     PASS      1x H1, 4x H2, 8x H3
GEO Citation Score    8/10      High code & factual density (Jev Score)
Intent Alignment      94%       Strongly satisfies query (Jev Noul)
```
