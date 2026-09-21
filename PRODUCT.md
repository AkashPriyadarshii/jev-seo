# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Static hand-built HTML + CSS, zero framework, zero build step, zero JS frameworks; single-page marketing site. Chosen for a resource-constrained machine and a project whose product is itself a zero-dependency Rust binary. (Inference: user's brief said "max best frontend", terse; taste-skill maps the developer-tool audience to native CSS over frameworks, and the previous session pinned no framework. Confirmed by no objection after delivery.)

## Users

Technical founders, indie hackers, and AI coding agents who distrust paid SEO suites. They run CLIs, read logs, audit markdown, and want tools that are free, local, and fast. Secondary audience: coding agents consuming the MCP server tool.

## Product Purpose

jev-seo is a FOSS, zero-cost (₹0) SEO and GEO search radar: audit content, validate Schema.org, check AI crawler permissions, generate SERP content briefs, and score generative-engine visibility. It replaces $130/month semantics suites with a native Rust CLI and an MCP server, using TypeSafe AI Jev System One as a deterministic oracle and local DuckDuckGo scraping for data.

## Positioning

The mechanism a neighboring product cannot copy: a full SEO/GEO suite with zero subscriptions and zero hallucination risk, run from a terminal with no dashboard, backed by TypeSafe Jev System One typed judgments instead of conversational LLM outputs. Local-first (SQLite rank tracking, audits in <1s for hundreds of files, <30MB RAM).

## Capabilities

- `keywords`: autocomplete discovery + Jev intent classification
- `query`: live SERP competitor scraping + gap analysis (DuckDuckGo)
- `audit`: batch on-page audit, orphan pages, AI-slop detection (em-dash density, 17 AI boilerplate tells), heading hierarchy, keyword cannibalization
- `geo`: generative engine optimization citation scoring (1-10) for Perplexity, SearchGPT, Gemini Overviews
- `schema`: JSON-LD Schema.org validator (SoftwareApplication, Article, Organization, Product), deprecation flags
- `robots`: robots.txt AI crawler radar (GPTBot, ClaudeBot, PerplexityBot, Google-Extended, Bytespider)
- `brief`: competitor SERP synthesis into heading outline + 150-word GEO direct answer
- `sitemap`: XML sitemap + hreflang auditor (50k URL limit, HTTPS canonical, x-default)
- `rank`: SQLite rank drift tracker
- `mcp`: stdio JSON-RPC 2.0 server exposing the tools to Claude Code, Gemini CLI, Antigravity

## Evidence

- MIT license, Rust, no paid APIs, DuckDuckGo HTML + suggest scraping, local SQLite (`.jev-seo.db`)
- 21 passing tests, zero clippy warnings; release binary installed to `C:\Users\saves\.local\bin\jev-seo.exe`
- Badge color language already established: #0055ff accent blue
- Author: Akash Priyadarshi (Patna, Bihar, India)