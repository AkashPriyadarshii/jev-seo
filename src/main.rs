use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::json;

mod audit;
mod actions;
mod brief;
mod crawl;
mod engine;
mod fetch;
mod gsc;
mod llms;
mod mcp;
mod paths;
mod policy;
mod rank;
mod robots;
mod rules;
mod schema;
mod serp;
mod sitemap;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(name = "jev-seo")]
#[command(author = "Akash Priyadarshi")]
#[command(version)]
#[command(about = "FOSS zero-cost, agent-first SEO & GEO search radar powered by TypeSafe Jev", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Autocomplete keyword discovery & intent classification
    Keywords {
        /// Target root query
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Scrape live SERP competitors and analyze winning content gaps
    Query {
        /// Search keyword to inspect
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Search backend: auto, ddg, or tavily (paid, needs TAVILY_API_KEY)
        #[arg(long, default_value = "auto")]
        provider: String,
        /// Paid search depth: basic, fast, ultra-fast, advanced
        #[arg(long, default_value = "advanced")]
        depth: String,
        /// Paid search topic: general, news, finance
        #[arg(long, default_value = "general")]
        topic: String,
        #[arg(long)]
        json: bool,
    },
    /// Audit a local file or directory for on-page SEO issues, duplicate titles, and thin pages
    Audit {
        /// File path or directory path to inspect
        path: String,
        #[arg(long)]
        target_query: Option<String>,
        #[arg(long)]
        json: bool,
        /// Exit nonzero when pass rate falls below this percent (CI gate)
        #[arg(long)]
        min_pass: Option<f64>,
        /// Write a single-file HTML report to this path
        #[arg(long, value_name = "PATH")]
        html: Option<String>,
        /// Write a PDF report to this path
        #[arg(long, value_name = "PATH")]
        pdf: Option<String>,
        /// Write a Markdown report to this path
        #[arg(long, value_name = "PATH")]
        md: Option<String>,
        /// Write a CSV findings export to this path
        #[arg(long, value_name = "PATH")]
        csv: Option<String>,
        /// Rebuild findings and actions from a saved audit JSON, no work
        #[arg(long, value_name = "PATH")]
        rescore: Option<String>,
    },
    /// Generative Engine Optimization (GEO) citation scoring via Jev
    Geo {
        /// File path or URL to score
        target: String,
        #[arg(long)]
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Validate Schema.org JSON-LD markup against active 2026 search specifications
    Schema {
        /// File path, HTML, or raw JSON string
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Inspect robots.txt on a live domain for AI crawler permissions and sitemaps
    Robots {
        /// Domain or URL to inspect
        domain: String,
        #[arg(long)]
        json: bool,
    },
    /// Synthesize live SERP competitor results into a ready-to-write content brief
    Brief {
        /// Topic or search query
        topic: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        markdown: bool,
        #[arg(long)]
        json: bool,
    },
    /// Track domain rankings in local SQLite (.jev-seo.db)
    Rank {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        query: String,
    },
    /// Inspect and validate XML sitemaps for protocols, URL limits, and hreflang
    Sitemap {
        /// URL or local file path to sitemap.xml
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Crawl a live site for broken links, redirect chains, and orphan pages
    Crawl {
        /// Start URL (seeds from /sitemap.xml when present)
        #[arg(value_name = "URL")]
        url: Option<String>,
        /// Maximum pages to fetch from this host
        #[arg(long, default_value_t = crate::crawl::DEFAULT_MAX_PAGES)]
        max_pages: usize,
        /// Fetch backends: auto, direct, jina, firecrawl (paid, needs key)
        #[arg(long, default_value = "auto")]
        fetch: String,
        /// Max paid fetch credits per run. 0 parks paid backends.
        #[arg(long, default_value_t = 0)]
        max_credits: u32,
        #[arg(long)]
        json: bool,
        /// Compare against the last stored snapshot in SQLite
        #[arg(long)]
        diff: bool,
        /// Write a CSV findings export to this path
        #[arg(long, value_name = "PATH")]
        csv: Option<String>,
        /// Rebuild score and actions from a saved crawl JSON, no network
        #[arg(long, value_name = "PATH")]
        rescore: Option<String>,
    },
    /// Check llms.txt presence and AI crawler permissions for answer-engine readiness
    Llms {
        /// Domain or URL to inspect
        domain: String,
        #[arg(long)]
        json: bool,
    },
    /// Check environment: version, API key presence, database, platform
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Google Search Console: free first-party query data (auth, sites, query)
    Gsc {
        /// Action: auth, sites, or query
        op: String,
        /// Device code for `auth --code`, site URL for `query`
        #[arg(long)]
        site: Option<String>,
        /// Device code value for auth step 2
        #[arg(long)]
        code: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Start native stdio JSON-RPC 2.0 Agent MCP Server
    Mcp,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keywords { query, json } => {
            let suggestions = serp::get_autocomplete(&query)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&suggestions)?);
                return Ok(());
            }

            println!("{}", format!("Keyword Autocomplete for \"{}\":", query).cyan().bold());
            for (idx, item) in suggestions.iter().enumerate() {
                println!("  {}. {}", idx + 1, item);
            }

            let state = json!({
                "root_query": query,
                "suggestions": suggestions
            });
            if let Some((eval, v)) = gated_eval("keywords", state) {
                println!("\n{}", "Intent & Semantic Classification:".cyan().bold());
                println!("  Primary Intent: {} (confidence: {:.2}){}", eval.intent.green(), eval.intent_confidence, policy::marker(v));
                println!("  Content Gap:    {}", eval.content_gap.yellow());
                println!("  Suggested Next: jev-seo {}", policy::route_for_intent(&eval.intent));
            }
        }
        Commands::Query { query, limit, provider, depth, topic, json } => {
            let backend = match provider.as_str() {
                "ddg" => serp::Provider::Ddg,
                "tavily" => serp::Provider::Tavily,
                _ => serp::Provider::Auto,
            };
            let opts = serp::SearchOpts { depth, topic, answer: false };
            eprintln!("{}", format!("Scraping live SERP for \"{}\" (limit: {})...", query, limit).dimmed());
            let items = serp::scrape_serp_opts(&query, limit, backend, &opts)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&items)?);
                return Ok(());
            }

            println!("\n{}", "Top Competitors on DuckDuckGo:".cyan().bold());
            for item in &items {
                println!("  #{:<2} {} - {}", item.position.to_string().green().bold(), item.title, item.url.dimmed());
                if !item.snippet.is_empty() {
                    println!("      {}", item.snippet);
                }
            }

            // Rerank by Jev relevance: one Score per result in the same request.
            let mut rel_questions = serde_json::Map::new();
            for (i, _) in items.iter().enumerate() {
                rel_questions.insert(
                    format!("rel_{}", i),
                    json!({
                        "type": "score",
                        "instructions": format!("How relevant is result {} to the query in `target_query`?", i),
                        "criteria": ["Irrelevant or spam", "Tangential mention", "Relevant to the query", "Highly relevant, directly answers", "Exact best match"]
                    }),
                );
            }
            let state = json!({
                "target_query": query,
                "competitor_serp": items
            });
            match engine::JevClient::new().map(|c| c.fanout_eval_with(state, serde_json::Value::Object(rel_questions))) {
                Some(Ok(eval)) => {
                    // Rerank gated on its own answers, not the gap verdict.
                    let rel_conf: f64 = (0..items.len())
                        .map(|i| {
                            eval.extra
                                .get(&format!("rel_{}", i))
                                .and_then(|a| a.get("confidence"))
                                .and_then(|c| c.as_f64())
                                .unwrap_or(0.0)
                        })
                        .sum::<f64>()
                        / items.len().max(1) as f64;
                    if rel_conf >= policy::thresholds("query").flag {
                        let mut ranked: Vec<(usize, f64)> = items
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let s = eval
                                    .extra
                                    .get(&format!("rel_{}", i))
                                    .and_then(|a| a.get("score"))
                                    .and_then(|s| s.as_f64())
                                    .unwrap_or(0.0);
                                (i, s)
                            })
                            .collect();
                        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        println!("\n{}", "Relevance Ranking (TypeSafe Jev):".cyan().bold());
                        for (rank, (i, score)) in ranked.iter().enumerate() {
                            let item = &items[*i];
                            println!("  #{:<2} (rel {:.1}) {} - {}", rank + 1, score, item.title, item.url.dimmed());
                        }
                    }
                    match policy::gate("query", eval.confidence()) {
                        policy::Verdict::Drop => eprintln!("{}", format!("Note: Jev unsure on gap analysis (confidence {:.2}), local results above stand.", eval.confidence()).yellow()),
                        v => {
                            println!("\n{}", "Competitive Gap Analysis (TypeSafe Jev):".cyan().bold());
                            println!("  Intent Class:    {}", eval.intent.green());
                            println!("  AI Citations:    {}/10 GEO Score{}", eval.geo_score, policy::marker(v));
                            println!("  Primary Gap:     {}", eval.content_gap.yellow());
                        }
                    }
                }
                Some(Err(e)) => eprintln!("{}", format!("Warning: Jev scoring failed ({e:#}), showing local-only output.").yellow()),
                None => eprintln!("{}", "Note: TYPESAFE_API_KEY not set, showing local-only output.".yellow()),
            }
        }
        Commands::Audit { path, target_query, json, min_pass, html, pdf, md, csv, rescore } => {
            let t0 = std::time::Instant::now();
            let dir_report = match rescore {
                Some(path) => {
                    let raw = std::fs::read_to_string(&path)?;
                    let saved: audit::DirectoryAuditReport = serde_json::from_str(&raw)?;
                    audit::with_findings(saved)
                }
                None => audit::with_findings(audit::audit_path(&path)?),
            };
            eprintln!(
                "[jev-seo {:>02}:{:>02}] audited {} files in {:.1}s",
                t0.elapsed().as_secs() / 60,
                t0.elapsed().as_secs() % 60,
                dir_report.total_files,
                t0.elapsed().as_secs_f64()
            );

            if let Some(out) = &html {
                std::fs::write(out, audit::to_html(&dir_report))?;
                println!("HTML report written to {}", out.dimmed());
            }
            if let Some(out) = &pdf {
                std::fs::write(out, audit::to_pdf(&dir_report))?;
                println!("PDF report written to {}", out.dimmed());
            }
            if let Some(out) = &md {
                std::fs::write(out, audit::to_markdown(&dir_report))?;
                println!("Markdown report written to {}", out.dimmed());
            }
            if let Some(out) = &csv {
                std::fs::write(out, crate::rules::to_csv(&dir_report.findings))?;
                println!("CSV findings written to {}", out.dimmed());
            }

            if let Some(floor) = min_pass {
                if dir_report.pass_rate < floor {
                    anyhow::bail!(
                        "pass rate {:.1}% below gate {:.1}%",
                        dir_report.pass_rate,
                        floor
                    );
                }
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&dir_report)?);
                return Ok(());
            }

            if dir_report.total_files > 1 {
                println!("\n{} {}", "Batch Directory SEO Audit:".cyan().bold(), dir_report.dir_path);
                println!("  Total Files Audited: {}", dir_report.total_files);
                println!("  Total Word Count:    {}", dir_report.total_words);
                println!("  Avg Words per File:  {}", dir_report.avg_words_per_file);
                println!("  Overall Check Pass:  {:.1}%", dir_report.pass_rate);

                if !dir_report.duplicate_titles.is_empty() {
                    println!("\n{}", "Duplicate Titles Detected:".red().bold());
                    for (title, files) in &dir_report.duplicate_titles {
                        println!("  - \"{}\" in {} files:", title, files.len());
                        for f in files {
                            println!("      {}", f.dimmed());
                        }
                    }
                } else {
                    println!("  Title Collisions:    {}", "0 (Clean)".green());
                }

                if !dir_report.keyword_cannibalization.is_empty() {
                    println!("\n{}", "Keyword Cannibalization Detected:".yellow().bold());
                    for item in &dir_report.keyword_cannibalization {
                        println!("  - Target Stem: \"{}\" in {} pages:", item.keyword_stem.cyan(), item.colliding_files.len());
                        for f in &item.colliding_files {
                            println!("      {}", f.dimmed());
                        }
                    }
                } else {
                    println!("  Cannibalization:     {}", "0 (Unique Intent)".green());
                }

                if !dir_report.orphan_pages.is_empty() {
                    println!("\n{}", format!("Orphan Pages (0 incoming internal links, {} files):", dir_report.orphan_pages.len()).yellow().bold());
                    for f in dir_report.orphan_pages.iter().take(5) {
                        println!("  - {}", f.yellow());
                    }
                    if dir_report.orphan_pages.len() > 5 {
                        println!("    ... and {} more", dir_report.orphan_pages.len() - 5);
                    }
                } else {
                    println!("  Orphan Pages:        {}", "0 (Fully Interlinked)".green());
                }

                if !dir_report.thin_pages.is_empty() {
                    println!("\n{}", format!("Thin Pages (<300 words, {} files):", dir_report.thin_pages.len()).yellow().bold());
                    for (f, wc) in dir_report.thin_pages.iter().take(5) {
                        println!("  - {} ({} words)", f, wc);
                    }
                    if dir_report.thin_pages.len() > 5 {
                        println!("    ... and {} more", dir_report.thin_pages.len() - 5);
                    }
                }

                if !dir_report.missing_canonicals.is_empty() {
                    println!("  Missing Canonicals:  {} files", dir_report.missing_canonicals.len().to_string().yellow());
                }
                println!("\n{}", "Summary Status: Audit complete across directory.".green());

                let actions = crate::rules::actions_for(&dir_report.findings);
                println!("  Rule findings: {}", dir_report.findings.len());
                if actions.is_empty() {
                    println!("  Actions:       {}", "none, directory is clean".green());
                } else {
                    println!("\n{}", "Top Actions:".cyan().bold());
                    for a in crate::actions::top(&actions, 5) {
                        println!("  [P{}|e{}|i{:>3}] {} {} {} - {}", a.priority, a.effort, a.impact, if a.quick_win { "QUICK".green().bold().to_string() } else { String::new() }, a.id.bold(), a.title, a.evidence.dimmed());
                    }
                }
            } else if let Some(report) = dir_report.reports.first() {
                println!("\n{} {}", "On-Page SEO Audit:".cyan().bold(), report.file_path);
                println!("  Title:       {}", report.title.as_deref().unwrap_or("N/A"));
                println!("  Description: {}", report.description.as_deref().unwrap_or("N/A"));
                println!("  Headings:    H1: {}, H2: {}, H3: {}", report.h1_count, report.h2_count, report.h3_count);
                if !report.heading_skipped_levels.is_empty() {
                    println!("  Skipped H*:  {}", report.heading_skipped_levels.join(", ").yellow());
                }
                println!("  Word Count:  {}", report.word_count);
                if report.em_dash_count > 0 || !report.ai_slop_words_found.is_empty() {
                    println!("  AI Tells:    {} em-dashes, words: [{}]", report.em_dash_count, report.ai_slop_words_found.join(", "));
                }
                println!("  Links:       Internal: {}, External: {}", report.internal_links, report.external_links);

                println!("\n{}", "Check Results:".bold());
                for check in &report.checks {
                    let badge = if check.passed { "PASS".green().bold() } else { "WARN".yellow().bold() };
                    println!("  [{}] {:<20} - {}", badge, check.name, check.message);
                }
                let file_findings: Vec<crate::rules::Finding> = dir_report
                    .findings
                    .iter()
                    .filter(|f| f.scope == report.file_path)
                    .cloned()
                    .collect();
                let failed = crate::rules::actions_for(&file_findings);
                if !failed.is_empty() {
                    println!("\n{}", "Top Actions:".cyan().bold());
                    for a in crate::actions::top(&failed, 5) {
                        println!("  [P{}|e{}|i{:>3}] {} {} {} - {}", a.priority, a.effort, a.impact, if a.quick_win { "QUICK".green().bold().to_string() } else { String::new() }, a.id.bold(), a.title, a.evidence.dimmed());
                    }
                }

                if let Some(query) = target_query {
                    let state = json!({
                        "target_query": query,
                        "page_title": report.title,
                        "description": report.description,
                        "checks": report.checks
                    });
                    if let Some((eval, v)) = gated_eval("audit", state) {
                        println!("\n{}", "Semantic Gap Evaluation (TypeSafe Jev):".cyan().bold());
                        println!("  GEO Score:       {}/10{}", eval.geo_score, policy::marker(v));
                        println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                        println!("  Content Gap:     {}", eval.content_gap.yellow());
                    }
                }
            }
        }
        Commands::Geo { target, query, json } => {
            let content = match crate::paths::read_user_file(&target, &["md", "mdx", "markdown", "html", "htm", "txt"]) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", format!("Error: {e:#}").red());
                    return Ok(());
                }
            };
            if let Some(client) = engine::JevClient::new() {
                let state = json!({
                    "query": query,
                    "content": content
                });
                match client.fanout_eval_with(state, policy::geo_questions()) {
                    Ok(eval) => {
                        let v = policy::gate("geo", eval.confidence());
                        if v == policy::Verdict::Drop {
                            eprintln!("{}", format!("Jev unsure (confidence {:.2}), no score.", eval.confidence()).yellow());
                            return Ok(());
                        }
                        let m = policy::marker(v);
                        if json {
                            println!("{}", serde_json::to_string_pretty(&eval)?);
                        } else {
                            println!("\n{}", "Generative Engine Optimization (GEO) Report:".cyan().bold());
                            println!("  Target Query:    {}", query);
                            println!("  GEO Score:       {}/10{}", eval.geo_score, m);
                            if let Some((composite, cconf)) = policy::composite_geo(&eval.extra) {
                                println!("  Composite:       {}/10 (confidence: {:.2})", composite, cconf);
                            }
                            let review = policy::needs_review(&eval.extra, "geo");
                            if !review.is_empty() {
                                println!("  Needs review:    {} [{}]", review.len().to_string().yellow(), review.join(", ").dimmed());
                            }
                            println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                            println!("  Primary Gap:     {}", eval.content_gap.yellow());
                            if let Ok(db) = rank::DbStore::open() {
                                match db.record_geo(&target, &query, eval.geo_score) {
                                    Ok(Some(prev)) if prev != eval.geo_score => {
                                        let arrow = if eval.geo_score > prev { "▲".green() } else { "▼".red() };
                                        println!("  Since Last:      {} {} → {}", arrow, prev, eval.geo_score);
                                    }
                                    Ok(Some(prev)) => println!("  Since Last:      {} (no change)", prev),
                                    _ => println!("  Since Last:      first recorded check"),
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("{}", format!("Error: Jev scoring failed ({e:#}).").red()),
                }
            } else {
                eprintln!("{}", "Error: TYPESAFE_API_KEY environment variable not set.".red());
            }
        }
        Commands::Schema { target, json } => {
            let report = schema::validate_target(&target)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "Schema.org Structured Data Audit:".cyan().bold(), report.target);
            println!("  Schemas Found:       {}", report.schemas_found);
            println!("  Detected Types:      {}", if report.types.is_empty() { "None".to_string() } else { report.types.join(", ") });
            println!("  Completeness Score:  {}/100", report.completeness_score);
            println!("  Validation Status:   {}", if report.is_valid { "VALID".green().bold() } else { "INVALID".red().bold() });

            if !report.details.is_empty() {
                println!("\n{}", "Schema Details:".bold());
                for d in &report.details {
                    let status = if d.is_valid { "PASS".green() } else { "FAIL".red() };
                    println!("  [{}] @type: {}", status, d.schema_type.bold());
                    if !d.missing_required.is_empty() {
                        println!("      Missing required: {}", d.missing_required.join(", ").red());
                    }
                    if !d.missing_recommended.is_empty() {
                        println!("      Missing recommended: {}", d.missing_recommended.join(", ").yellow());
                    }
                    if let Some(dep) = &d.deprecation_notice {
                        println!("      DEPRECATION: {}", dep.yellow());
                    }
                }
            }

            if !report.errors.is_empty() {
                println!("\n{}", "Errors:".red().bold());
                for e in &report.errors {
                    println!("  ✖ {}", e);
                }
            }
            if !report.warnings.is_empty() {
                println!("\n{}", "Warnings:".yellow().bold());
                for w in &report.warnings {
                    println!("  ⚠ {}", w);
                }
            }
        }
        Commands::Robots { domain, json } => {
            let report = robots::inspect_robots(&domain)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "robots.txt & AI Crawler Audit:".cyan().bold(), report.domain);
            println!("  Robots URL:     {}", report.robots_url.dimmed());
            println!("  HTTP Status:    {}", report.status_code);
            println!("  Has robots.txt: {}", if report.has_robots { "YES".green() } else { "NO".red() });

            if !report.sitemaps.is_empty() {
                println!("\n{}", "Sitemaps Discovered:".cyan().bold());
                for s in &report.sitemaps {
                    println!("  - {}", s);
                }
            }

            println!("\n{}", "AI Crawler Permissions:".bold());
            for rule in &report.ai_bot_rules {
                let badge = match rule.status {
                    robots::BotStatus::Allowed => "ALLOW".green().bold(),
                    robots::BotStatus::Disallowed => "BLOCK".red().bold(),
                    robots::BotStatus::DefaultStar => "DEFAULT(*)".yellow(),
                };
                println!("  [{:<10}] {:<16} ({})", badge, rule.bot_name.bold(), rule.purpose.dimmed());
                println!("               {}", rule.rule_snippet.dimmed());
            }
        }
        Commands::Brief { topic, limit, markdown, json } => {
            let brief = brief::generate_brief(&topic, limit)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&brief)?);
                return Ok(());
            }

            if markdown {
                println!("{}", brief.to_markdown());
                return Ok(());
            }

            println!("\n{} \"{}\"", "Content Brief Blueprint:".cyan().bold(), brief.topic);
            println!("  Suggested Title: {}", brief.suggested_title.green().bold());
            println!("  Target Length:   {}", brief.target_word_count);
            println!("  Search Intent:   {}", brief.search_intent);
            println!("  Audience:        {}", brief.target_audience);
            println!("  Differentiator:  {}", brief.winning_angle.yellow());

            println!("\n{}", "GEO 150-Word Opening Prescription:".cyan().bold());
            println!("  {}", brief.geo_opening_prescription);

            println!("\n{}", "Recommended Heading Outline (H2):".bold());
            for h2 in &brief.recommended_h2_outline {
                println!("  {}", h2);
            }

            println!("\n{}", "Top Competitors Analyzed:".bold());
            for comp in &brief.competitor_benchmarks {
                println!("  #{} {} ({})", comp.rank, comp.title, comp.url.dimmed());
            }
        }
        Commands::Rank { domain, query } => {
            println!("{}", format!("Searching DuckDuckGo rank for domain: \"{}\" on query: \"{}\"...", domain, query).dimmed());
            let items = serp::scrape_serp(&query, 30)?;
            let position = items.iter().position(|i| paths::url_matches_domain(&i.url, &domain)).map(|p| p + 1);
            let target_url = position.and_then(|p| items.get(p - 1)).map(|i| i.url.as_str());

            let mut db = rank::DbStore::open()?;
            let delta = db.track_keyword(&domain, &query, position, target_url)?;

            println!("\n{}", "Rank Tracking Result:".cyan().bold());
            println!("  Domain:   {}", delta.domain);
            println!("  Keyword:  {}", delta.term);

            let rank_str = match delta.curr_rank {
                Some(r) => format!("#{}", r).green().bold().to_string(),
                None => "Not in top 30".red().to_string(),
            };
            println!("  Current:  {}", rank_str);

            if let Some(prev) = delta.prev_rank {
                println!("  Previous: #{}", prev);
                if let Some(curr) = delta.curr_rank {
                    if curr < prev {
                        println!("  Change:   {}", format!("▲{}", prev - curr).green().bold());
                    } else if curr > prev {
                        println!("  Change:   {}", format!("▼{}", curr - prev).red().bold());
                    } else {
                        println!("  Change:   {}", "No change".dimmed());
                    }
                }
            } else {
                println!("  Previous: First recorded check");
            }
        }
        Commands::Sitemap { target, json } => {
            let report = sitemap::audit_sitemap(&target)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "XML Sitemap & Hreflang Audit:".cyan().bold(), report.target);
            println!("  Valid Sitemap:   {}", if report.is_valid { "YES".green().bold() } else { "NO".red().bold() });
            println!("  Total URLs:      {}", report.total_urls);
            println!("  HTTPS URLs:      {}/{} ({:.1}%)", 
                report.https_urls, 
                report.total_urls,
                if report.total_urls > 0 { (report.https_urls as f64 / report.total_urls as f64) * 100.0 } else { 0.0 }
            );
            if report.insecure_http_urls > 0 {
                println!("  Insecure HTTP:   {}", format!("{} URLs", report.insecure_http_urls).red().bold());
            }
            if report.urls_with_params > 0 {
                println!("  Query Params:    {}", format!("{} URLs with '?'", report.urls_with_params).yellow());
            }
            println!("  With <lastmod>:  {}/{}", report.urls_with_lastmod, report.total_urls);
            println!("  Hreflang Tags:   {}", report.hreflang_count);

            if !report.sample_urls.is_empty() {
                println!("\n{}", "Sample URLs:".bold());
                for u in &report.sample_urls {
                    println!("  - {}", u.dimmed());
                }
            }

            if !report.errors.is_empty() {
                println!("\n{}", "Errors:".red().bold());
                for e in &report.errors {
                    println!("  ✖ {}", e);
                }
            }
            if !report.warnings.is_empty() {
                println!("\n{}", "Warnings:".yellow().bold());
                for w in &report.warnings {
                    println!("  ⚠ {}", w);
                }
            }
        }
        Commands::Crawl { url, max_pages, fetch, max_credits, json, diff, csv, rescore } => {
            if let Some(path) = rescore {
                let raw = std::fs::read_to_string(&path)?;
                let saved: crawl::CrawlReport = serde_json::from_str(&raw)?;
                let report = crawl::finish_report(crawl::ReportParts {
                    start_url: saved.start_url,
                    pages: saved.pages,
                    redirects: saved.redirects,
                    inbound: saved.inbound,
                    errors: saved.errors,
                    seeded_from_sitemap: saved.seeded_from_sitemap,
                    capped: saved.capped,
                    robots_honored: saved.robots_honored,
                });
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("  Rescored:      {}/100 ({})", report.score, crate::actions::grade(report.score));
                }
                return Ok(());
            }
            let url = url.unwrap_or_else(|| {
                eprintln!("{}", "Error: provide a start URL or --rescore PATH.".red());
                std::process::exit(2);
            });
            eprintln!("{}", format!("Crawling {} (max {} pages)...", url, max_pages).dimmed());
            let mode = match fetch.as_str() {
                "direct" => crate::fetch::FetchMode::Direct,
                "jina" => crate::fetch::FetchMode::Jina,
                "firecrawl" => crate::fetch::FetchMode::Firecrawl,
                _ => crate::fetch::FetchMode::Auto,
            };
            let mut budget = crate::fetch::Budget { max_credits, spent: 0 };
            let report = crawl::crawl_site(&url, max_pages, mode, &mut budget)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "Live Site Crawl:".cyan().bold(), report.start_url);
            println!("  Pages crawled: {}", report.pages_crawled);
            println!("  Health:        {}/100 ({})", report.score, crate::actions::grade(report.score).green().bold());
            let upgraded = report.pages.iter().filter(|p| p.source != "direct").count();
            if upgraded > 0 {
                println!("  Upgraded:      {} pages via alt backends", upgraded.to_string().yellow());
            }
            if budget.spent > 0 {
                println!("  Paid spend:    {} fetch credits", budget.spent.to_string().yellow());
            }
            println!(
                "  Areas:         {}",
                report
                    .areas
                    .iter()
                    .map(|a| format!("{} {}", crate::rules::label(&a.area), a.score))
                    .collect::<Vec<_>>()
                    .join(", ")
                    .dimmed()
            );
            if report.broken.is_empty() {
                println!("  Broken links:  {}", "0 (Clean)".green());
            } else {
                println!("  Broken links:  {}", report.broken.len().to_string().red().bold());
                for p in report.broken.iter().take(5) {
                    println!("    ✖ [{}] {}", p.status, p.url.dimmed());
                }
                if report.broken.len() > 5 {
                    println!("      ... and {} more", report.broken.len() - 5);
                }
            }
            let slow: Vec<&crawl::PageRecord> = report
                .pages
                .iter()
                .filter(|p| p.status == 200 && p.elapsed_ms > crawl::SLOW_PAGE_MS)
                .collect();
            if slow.is_empty() {
                println!("  Slow pages:    {}", "0".green());
            } else {
                println!("  Slow pages:    {}", format!("{} over {}ms", slow.len(), crawl::SLOW_PAGE_MS).yellow());
                for p in slow.iter().take(5) {
                    println!("      {}ms {}", p.elapsed_ms, p.url.dimmed());
                }
            }
            if report.redirects.is_empty() {
                println!("  Redirects:     {}", "0".green());
            } else {
                println!("  Redirects:     {}", report.redirects.len().to_string().yellow());
                for (from, to) in report.redirects.iter().take(3) {
                    println!("      {} -> {}", from.dimmed(), to.dimmed());
                }
            }
            if report.orphans.is_empty() {
                println!("  Orphans:       {}", "0".green());
            } else {
                println!("  Orphans:       {}", report.orphans.len().to_string().yellow());
                for f in report.orphans.iter().take(5) {
                    println!("      {}", f.yellow());
                }
            }
            if !report.errors.is_empty() {
                println!("  Fetch errors:  {}", report.errors.len().to_string().yellow());
                for e in report.errors.iter().take(3) {
                    println!("      {}", e.dimmed());
                }
            }
            if report.actions.is_empty() {
                println!("  Actions:       {}", "none, site is clean".green());
            } else {
                println!("\n{}", "Top Actions:".cyan().bold());
                for a in crate::actions::top(&report.actions, 5) {
                    println!("  [P{}|e{}|i{:>3}] {} {} {} - {}", a.priority, a.effort, a.impact, if a.quick_win { "QUICK".green().bold().to_string() } else { String::new() }, a.id.bold(), a.title, a.evidence.dimmed());
                }
            }
            println!(
                "  Completeness:  {} seeded, robots {}, {}{}",
                if report.seeded_from_sitemap { "sitemap" } else { "start-URL" },
                if report.robots_honored { "honored" } else { "missing" },
                report.pages_crawled,
                if report.capped { " (capped, raise --max-pages)" } else { " (full)" }
            );
            if diff {
                match rank::DbStore::open()?.record_crawl_snapshot(
                    &report.start_url,
                    report.pages_crawled as i64,
                    report.broken.len() as i64,
                )? {
                    Some((prev_pages, prev_broken)) => {
                        println!(
                            "  Since last:    {} pages (was {}), {} broken (was {})",
                            report.pages_crawled, prev_pages, report.broken.len(), prev_broken
                        );
                    }
                    None => println!("  Since last:    first recorded snapshot"),
                }
            }
            if let Some(out) = &csv {
                std::fs::write(out, crate::rules::to_csv(&report.findings))?;
                println!("CSV findings written to {}", out.dimmed());
            }
        }
        Commands::Llms { domain, json } => {
            let t0 = std::time::Instant::now();
            let report = llms::check_llms(&domain)?;
            eprintln!(
                "[jev-seo {:>02}:{:>02}] readiness checked in {:.1}s",
                t0.elapsed().as_secs() / 60,
                t0.elapsed().as_secs() % 60,
                t0.elapsed().as_secs_f64()
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "Agent Readiness:".cyan().bold(), report.domain);
            println!("  llms.txt:      {}", if report.info.present { "YES".green() } else { "NO".red() });
            if report.info.present {
                println!("  Sections:      {}", report.info.sections.len());
            }
            println!("  AI explicit:   {}", if report.ai_allowed.is_empty() { "none".yellow().to_string() } else { report.ai_allowed.join(", ").green().to_string() });
            println!("  AI default:    {} bots inherit allow (*)", report.ai_default.len());
            if !report.ai_blocked.is_empty() {
                println!("  AI blocked:    {}", report.ai_blocked.join(", ").red());
            }
            println!("  Score:         {}/100", report.score);
            if report.actions.is_empty() {
                println!("  Actions:       {}", "none, ready".green());
            } else {
                println!("\n{}", "Top Actions:".cyan().bold());
                for a in crate::actions::top(&report.actions, 5) {
                    println!("  [P{}|e{}|i{:>3}] {} {} {} - {}", a.priority, a.effort, a.impact, if a.quick_win { "QUICK".green().bold().to_string() } else { String::new() }, a.id.bold(), a.title, a.evidence.dimmed());
                }
            }
            println!("\n{}", "Checks:".bold());
            for c in &report.checks {
                let badge = if c.passed { "PASS".green().bold() } else { "WARN".yellow().bold() };
                println!("  [{}] {:<22} - {}", badge, c.name, c.message);
            }
        }
        Commands::Doctor { json } => {
            let key_set = std::env::var("TYPESAFE_API_KEY").map(|k| !k.trim().is_empty()).unwrap_or(false);
            let db_ok = rank::DbStore::open().is_ok();
            let report = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "os": std::env::consts::OS,
                "typesafe_key": if key_set { "set" } else { "missing" },
                "database": if db_ok { "writable" } else { "error" },
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }
            println!("\n{}", "Doctor:".cyan().bold());
            println!("  Version:       {}", env!("CARGO_PKG_VERSION"));
            println!("  Platform:      {}", std::env::consts::OS);
            println!(
                "  Jev API key:   {}",
                if key_set { "set (value hidden)".green().to_string() } else { "missing, Jev scores will skip".yellow().to_string() }
            );
            println!("  Database:      {}", if db_ok { "writable".green().to_string() } else { "ERROR".red().to_string() });
        }
        Commands::Gsc { op, site, code, limit, json } => {
            match op.as_str() {
                "auth" => {
                    if let Some(device) = code {
                        gsc::auth_poll(&device)?;
                    } else {
                        gsc::auth_start()?;
                    }
                }
                "sites" => {
                    let sites = gsc::sites()?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&sites)?);
                    } else {
                        println!("\n{}", "Verified sites:".cyan().bold());
                        for s in &sites {
                            println!("  - {}", s);
                        }
                    }
                }
                "query" => {
                    let site = site.unwrap_or_else(|| {
                        eprintln!("{}", "Error: query needs --site <verified URL>.".red());
                        std::process::exit(2);
                    });
                    let rows = gsc::top_queries(&site, limit)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&rows)?);
                    } else {
                        println!("\n{} {}", "Top queries:".cyan().bold(), site.dimmed());
                        for r in &rows {
                            println!(
                                "  {:<40} clicks {:>6.0} pos {:>5.1}",
                                r.query.chars().take(40).collect::<String>(),
                                r.clicks,
                                r.position
                            );
                        }
                    }
                }
                other => {
                    eprintln!("{}", format!("Error: unknown gsc action '{}', use auth, sites, or query.", other).red());
                    std::process::exit(2);
                }
            }
        }
        Commands::Mcp => {
            mcp::run_stdio_server()?;
        }
    }

    Ok(())
}

/// Run a Jev eval gated by policy. Returns None on low confidence or API
/// failure, after telling the user the output is local-only. Never silent.
fn gated_eval(command: &str, state: serde_json::Value) -> Option<(engine::AnalysisResult, policy::Verdict)> {
    gated_eval_with(command, state, serde_json::json!({}))
}

fn gated_eval_with(
    command: &str,
    state: serde_json::Value,
    extra: serde_json::Value,
) -> Option<(engine::AnalysisResult, policy::Verdict)> {
    let client = engine::JevClient::new()?;
    match client.fanout_eval_with(state, extra) {
        Ok(eval) => {
            let v = policy::gate(command, eval.confidence());
            if v == policy::Verdict::Drop {
                eprintln!("{}", format!("Note: Jev unsure (confidence {:.2}), showing local-only output.", eval.confidence()).yellow());
                return None;
            }
            Some((eval, v))
        }
        Err(e) => {
            eprintln!("{}", format!("Warning: Jev scoring failed ({e:#}), showing local-only output.").yellow());
            None
        }
    }
}
