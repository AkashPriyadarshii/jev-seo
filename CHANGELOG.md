# Changelog

All notable changes to `jev-seo` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project repository skeleton, PRD, architecture, design specifications, and agent contracts.
- XML sitemap inspection command `sitemap` validating `<loc>`, `<lastmod>`, canonical HTTPS links, parameter pollution, and 50,000 URL limits.
- International hreflang tag validation verifying language and country pairs and enforcing `x-default` fallbacks.
- Heading hierarchy skip-level validation in Markdown and HTML audits.
- Google Helpful Content and AI slop detection tracking em-dash frequency and scanning 17 synthetic writing markers.
- Internal link graph analysis detecting orphan pages with zero inbound links across audited directories.
- Keyword cannibalization radar identifying colliding pages targeting identical multi-word keyword stems.
- New `seo_sitemap` tool added to the native MCP server, expanding the exposed toolset to 8 tools.
- Expanded test harness to 21 passing unit and integration tests.
