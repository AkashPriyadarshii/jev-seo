use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::json;

mod audit;
mod engine;
mod mcp;
mod rank;
mod serp;

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
    /// Audit local markdown (.md/.mdx) or HTML file for on-page SEO issues
    Audit {
        /// File path to inspect
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
    /// Track domain rankings in local SQLite (.jev-seo.db)
    Rank {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        query: String,
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

            if let Some(client) = engine::JevClient::new() {
                let state = json!({
                    "root_query": query,
                    "suggestions": suggestions
                });
                if let Ok(eval) = client.fanout_eval(state) {
                    println!("\n{}", "Intent & Semantic Classification:".cyan().bold());
                    println!("  Primary Intent: {} (confidence: {:.2})", eval.intent.green(), eval.intent_confidence);
                    println!("  Content Gap:    {}", eval.content_gap.yellow());
                }
            }
        }
        Commands::Query { query, limit, json } => {
            println!("{}", format!("Scraping live SERP for \"{}\" (limit: {})...", query, limit).dimmed());
            let items = serp::scrape_serp(&query, limit)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&items)?);
                return Ok(());
            }

            println!("\n{}", format!("Top Competitors on DuckDuckGo:").cyan().bold());
            for item in &items {
                println!("  #{:<2} {} - {}", item.position.to_string().green().bold(), item.title, item.url.dimmed());
                if !item.snippet.is_empty() {
                    println!("      {}", item.snippet);
                }
            }

            if let Some(client) = engine::JevClient::new() {
                let state = json!({
                    "target_query": query,
                    "competitor_serp": items
                });
                if let Ok(eval) = client.fanout_eval(state) {
                    println!("\n{}", "Competitive Gap Analysis (TypeSafe Jev):".cyan().bold());
                    println!("  Intent Class:    {}", eval.intent.green());
                    println!("  AI Citations:    {}/10 GEO Score", eval.geo_score);
                    println!("  Primary Gap:     {}", eval.content_gap.yellow());
                }
            }
        }
        Commands::Audit { path, target_query, json } => {
            let report = audit::audit_file(&path)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }

            println!("\n{} {}", "On-Page SEO Audit:".cyan().bold(), report.file_path);
            println!("  Title:       {}", report.title.as_deref().unwrap_or("N/A"));
            println!("  Description: {}", report.description.as_deref().unwrap_or("N/A"));
            println!("  Headings:    H1: {}, H2: {}, H3: {}", report.h1_count, report.h2_count, report.h3_count);
            println!("  Word Count:  {}", report.word_count);
            println!("  Links:       Internal: {}, External: {}", report.internal_links, report.external_links);

            println!("\n{}", "Check Results:".bold());
            for check in &report.checks {
                let badge = if check.passed { "PASS".green().bold() } else { "WARN".yellow().bold() };
                println!("  [{}] {:<20} - {}", badge, check.name, check.message);
            }

            if let Some(query) = target_query {
                if let Some(client) = engine::JevClient::new() {
                    let state = json!({
                        "target_query": query,
                        "page_title": report.title,
                        "description": report.description,
                        "checks": report.checks
                    });
                    if let Ok(eval) = client.fanout_eval(state) {
                        println!("\n{}", "Semantic Gap Evaluation (TypeSafe Jev):".cyan().bold());
                        println!("  GEO Score:       {}/10", eval.geo_score);
                        println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                        println!("  Content Gap:     {}", eval.content_gap.yellow());
                    }
                }
            }
        }
        Commands::Geo { target, query, json } => {
            let content = std::fs::read_to_string(&target).unwrap_or_else(|_| target.clone());
            if let Some(client) = engine::JevClient::new() {
                let state = json!({
                    "query": query,
                    "content": content
                });
                let eval = client.fanout_eval(state)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&eval)?);
                } else {
                    println!("\n{}", "Generative Engine Optimization (GEO) Report:".cyan().bold());
                    println!("  Target Query:    {}", query);
                    println!("  GEO Score:       {}/10", eval.geo_score);
                    println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                    println!("  Primary Gap:     {}", eval.content_gap.yellow());
                }
            } else {
                eprintln!("{}", "Error: TYPESAFE_API_KEY environment variable not set.".red());
            }
        }
        Commands::Rank { domain, query } => {
            println!("{}", format!("Searching DuckDuckGo rank for domain: \"{}\" on query: \"{}\"...", domain, query).dimmed());
            let items = serp::scrape_serp(&query, 30)?;
            let position = items.iter().position(|i| i.url.contains(&domain)).map(|p| p + 1);
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
        Commands::Mcp => {
            mcp::run_stdio_server()?;
        }
    }

    Ok(())
}
