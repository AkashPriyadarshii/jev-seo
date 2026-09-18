# Developer Handoff: jev-seo

## 1. Quick Setup
```bash
cd C:/Users/saves/Desktop/jev-seo
cargo check
```

## 2. Environment Variables
- `TYPESAFE_API_KEY`: Required for semantic Jev evaluations (`keywords`, `query`, `geo`, `audit --semantic`).
- When missing, `jev-seo` gracefully falls back to local AST checks and raw SERP tables without semantic Jev ratings.

## 3. Pre-Commit Check
Always run:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
git jev check
```
