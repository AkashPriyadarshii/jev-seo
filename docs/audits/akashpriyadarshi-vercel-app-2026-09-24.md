# Site audit: akashpriyadarshi.vercel.app (2026-09-24)

Full jev-seo sweep on the portfolio site. Binary at wave-1-add head, all keys active. Total Jev spend under $0.001.

## Crawl (50 pages, capped)

- Broken links: 0. Redirects: 0.
- 38 orphan pages (R25): uses, privacy, most blog posts. Link inward from index and related posts.
- 11 project pages return empty bodies (R08, blocking).
- 42 of 50 pages slower than 800ms, no compression signal anywhere. Enable gzip or brotli.
- Sitemap lists 60 URLs; rerun above 50 for the full picture.

## Agent readiness: 100/100

- llms.txt present, 6 sections. 5 of 15 bots named explicitly.
- Name OAI-SearchBot, ChatGPT-User, and Claude-SearchBot to reach 8 of 15.

## Sitemap: valid

- 60 URLs, all HTTPS. Only 26 carry lastmod; add it everywhere.

## Schema: valid, 100/100

- Person plus WebSite. No action.

## GEO: 5/10 [verify], composite 7/10 at 0.59

- query_fit 0.95, helpfulness 0.93. Weak: directness 0.69 at zero confidence, statistics 0.65, no direct-answer opening.
- Fix: one fact-dense definition block near the top of the homepage.

## Brand search

- Portfolio ranks #1 for the name query (Tavily); tracker records #4 with provenance.
- Entity collision: RocketReach and Peerlist attribute a different Akash Priyadarshi (CGI engineer, Bengaluru) to this name.
- Fix: strengthen sameAs links to GitHub and X in the Person block; get the bio quoted verbatim in third-party profiles.
