use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::json;

mod audit;
mod brief;
mod engine;
mod mcp;
mod paths;
mod policy;
mod rank;
mod robots;
mod schema;
mod serp;
mod sitemap;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(name = "jev-seo")]
#[command(author = "Akash Priyadarshi")]
#[command(version = "0.1.0")]
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
        Commands::Query { query, limit, json } => {
            println!("{}", format!("Scraping live SERP for \"{}\" (limit: {})...", query, limit).dimmed());
            let items = serp::scrape_serp(&query, limit)?;

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

            let state = json!({
                "target_query": query,
                "competitor_serp": items
            });
            if let Some((eval, v)) = gated_eval("query", state) {
                println!("\n{}", "Competitive Gap Analysis (TypeSafe Jev):".cyan().bold());
                println!("  Intent Class:    {}", eval.intent.green());
                println!("  AI Citations:    {}/10 GEO Score{}", eval.geo_score, policy::marker(v));
                println!("  Primary Gap:     {}", eval.content_gap.yellow());
            }
        }
        Commands::Audit { path, target_query, json } => {
            let dir_report = audit::audit_path(&path)?;

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
            let content = match geo_target_content(&target) {
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
                            println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                            println!("  Primary Gap:     {}", eval.content_gap.yellow());
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
        Commands::Mcp => {
            mcp::run_stdio_server()?;
        }
    }

    Ok(())
}

fn geo_target_content(target: &str) -> Result<String> {
    crate::paths::read_user_file(target, &["md", "mdx", "markdown", "html", "htm", "txt"])
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
