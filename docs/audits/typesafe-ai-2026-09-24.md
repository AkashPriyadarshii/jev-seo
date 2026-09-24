# Site audit: typesafe.ai (2026-09-24)

Full jev-seo sweep on the vendor's own site. Binary at wave-1-add head, all keys active. Total Jev spend under $0.001.

## Crawl (16 pages, complete)

- Health 98-99/100 (A). Areas: crawl 100, links 96, performance 95, security 100, canonical 100.
- Broken links: 0. Redirects: 0.
- 3 orphan pages (R25): /blog, /blog/antibenchmaxxing, /blog/diogo-almeida---founders-you-should-know.
- 2 slow pages over 800ms in one pass (R42): /data-processing 3.0s, one long blog post 0.9s; second pass clean.
- No compression signal on all 16 pages (R44). Enable gzip or brotli.

## Agent readiness: 30/100 — the weak spot

- llms.txt missing: LLMS-001 (P1, effort 1) "Ship an llms.txt - answer engines cannot see the site".
- 0 of 15 tracked bots explicitly allowed; all inherit default `*` policy: LLMS-002 (P2) "Name AI crawlers in robots.txt".
- Sitemap advertised in robots, robots.txt present.

## Sitemap: valid

- 14 URLs, all HTTPS. 0 of 14 carry lastmod; add it everywhere. No hreflang.

## GEO: 3/10 [verify], composite 5/10 at 0.38

- Query "typesafe ai system one typed decisions". Primary gap generic_prose; 12 signals need review.
- Direct answer first: NO (p=0.16). page_trust 0.23, title_fit 0.22 drag the score.
- importance 0.98: searchers want this page, but it does not answer first.

## Reading

- Technically strong site (99 health, zero broken links) but invisible to answer engines: no llms.txt, no explicit AI crawler grants, thin GEO opening. Ship llms.txt + bot grants first; those are the P1/P2 quick wins.