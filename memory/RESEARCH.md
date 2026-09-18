# Memory & Research: jev-seo

## Scouted References & Building Blocks
- **DuckDuckGo Autocomplete**: `GET https://duckduckgo.com/ac/?q={}&type=list` returns JSON array `[query, [suggestions]]`. Spec validated via `theabbie/suggest`.
- **DuckDuckGo HTML SERP**: `POST https://html.duckduckgo.com/html/` with body `q={query}&b=&kl=us-en`. No JavaScript required, zero bot blocks with browser user agent.
- **AST Parsing**:
  - `gray-matter-rs` for clean split of YAML/TOML frontmatter from Markdown body.
  - `fast-html-parser` for SIMD-accelerated DOM CSS selector querying.
- **GEO Discovery Framework**: Heuristics from `YellowFrogio/AI-Discovery-SEO-Framework-2026`:
  - Information gain vs competitor baseline.
  - Heading-to-answer proximity (<150 words).
  - Entity citation density.
- **Persistence**: `rusqlite` bundled in WAL mode.

## TypeSafe Jev Endpoints & Payload Schema
- Endpoint: `POST https://api.typesafe.ai/v1/systemone`
- Model: `jev-latest`
- Speculative Fan-out Schema:
  ```json
  {
    "model": "jev-latest",
    "state": {
      "target_query": "offline expense tracker android",
      "competitor_snippets": "...",
      "page_excerpt": "..."
    },
    "questions": {
      "intent": {
        "type": "choice",
        "instructions": "Classify the primary search intent for this query.",
        "criteria": {
          "informational": "Seeking explanations, how-tos, or documentation",
          "commercial": "Comparing software products or services before buying",
          "navigational": "Looking for a specific website, brand, or repository",
          "transactional": "Looking to download, purchase, or sign up immediately"
        }
      },
      "geo_score": {
        "type": "score",
        "instructions": "Rate how likely AI engines (Perplexity, SearchGPT, Gemini) are to cite this page content (1-10).",
        "min": 1,
        "max": 10
      },
      "direct_answer": {
        "type": "noul",
        "instructions": "Does the page present a direct factual answer or code example in the opening section?"
      },
      "content_gap": {
        "type": "choice",
        "instructions": "What is the primary content gap relative to the target search query?",
        "criteria": {
          "missing_statistics": "Lacks benchmarks, real numbers, or empirical proof",
          "generic_prose": "Vague marketing language and AI-slop boilerplate",
          "no_step_by_step": "Missing concrete setup instructions or code snippets",
          "outdated_examples": "Refers to obsolete library versions or dead links",
          "none": "Content is comprehensive and satisfies intent"
        }
      }
    }
  }
  ```
