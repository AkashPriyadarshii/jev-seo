# Site audit: akashpriyadarshi.vercel.app (2026-09-24)

Full jev-seo sweep on the portfolio site. Binary at wave-1-add head, all keys active. Total Jev spend under $0.001.

## Crawl (30 pages, capped)

- Health 97/100 (A). Areas: crawl 98, links 95, performance 93, security 100, canonical 100.
- Broken links: 0. Redirects: 0.
- 19 orphan pages (R25): uses, privacy, most blog posts. Link inward from index and related posts.
- 25 of 30 pages slower than 800ms (worst /contact 4.1s), no compression signal anywhere (R44 x30). Enable gzip or brotli.
- 4 blog pages return empty bodies (R08, blocking): building-patna, building-paperleaks, building-formkaro, +1.
- Rerun with a higher `--max-pages` for the full-site verdict.

## Agent readiness: 100/100

- llms.txt present, 7 sections. 5 of 15 bots named explicitly.
- Name OAI-SearchBot, ChatGPT-User, and Claude-SearchBot to reach 8 of 15.

## Sitemap: valid

- 61 URLs, all HTTPS. Only 27 carry lastmod; add it everywhere.

## GEO: 5/10 [verify], composite 7/10 at 0.58

- query_fit 0.95, helpfulness 0.93. Weak: title_fit 0.18, no direct-answer opening, statistics 0.67, directness 0.77 at zero confidence.
- Fix: one fact-dense definition block near the top of the homepage; tighten the title for the searcher's words.

## Brand search

- Portfolio ranks #1 for the name query (Tavily); tracker records #4 with provenance.
- Entity collision: RocketReach and Peerlist attribute a different Akash Priyadarshi (CGI engineer, Bengaluru) to this name.
- Fix: strengthen sameAs links to GitHub and X in the Person block; get the bio quoted verbatim in third-party profiles.