use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::json;
use std::collections::BTreeSet;

mod audit;
mod actions;
mod brief;
mod crawl;
mod engine;
mod fetch;
mod gsc;
mod llms;
mod manifest;
mod narrative;
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
        /// Write ranked action-tracker CSV (id, priority, effort, impact)
        #[arg(long, value_name = "PATH")]
        actions_csv: Option<String>,
        /// Write action-tracker SpreadsheetML (.xls) Excel opens without conversion
        #[arg(long, value_name = "PATH")]
        actions_xls: Option<String>,
        /// Write stem×URL conflict pairs CSV (cannibalization)
        #[arg(long, value_name = "PATH")]
        pairs_csv: Option<String>,
        /// Rebuild findings and actions from a saved audit JSON, no work
        #[arg(long, value_name = "PATH")]
        rescore: Option<String>,
        /// Write run.json + ledger.json to this directory (default: beside first export)
        #[arg(long, value_name = "DIR")]
        manifest: Option<String>,
        /// Skip all Jev semantic calls (rules and local audit only)
        #[arg(long)]
        no_jev: bool,
        /// Hard Jev spend cap in USD for this run
        #[arg(long, default_value_t = 0.25, value_name = "USD")]
        jev_budget: f64,
    },
    /// Generative Engine Optimization (GEO) citation scoring via Jev
    Geo {
        /// File path or URL to score
        target: String,
        #[arg(long)]
        query: String,
        #[arg(long)]
        json: bool,
        /// Hard Jev spend cap in USD for this run
        #[arg(long, default_value_t = 0.25, value_name = "USD")]
        jev_budget: f64,
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
        /// Write run.json + ledger.json to this directory (default: beside first export)
        #[arg(long, value_name = "DIR")]
        manifest: Option<String>,
        /// Skip the homepage Jev site+GEO judgment
        #[arg(long)]
        no_jev: bool,
        /// Hard Jev spend cap in USD for this run
        #[arg(long, default_value_t = 0.25, value_name = "USD")]
        jev_budget: f64,
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
    /// Explain a stable rule id (R19 or RULE-R19): area, severity, fix
    Explain {
        /// Rule id, e.g. R19 or RULE-R19
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Diff two saved audit JSON reports (current vs baseline)
    Report {
        /// Current audit JSON (from `audit --json` or audit path written)
        path: String,
        /// Baseline audit JSON to compare against
        #[arg(long, value_name = "PATH")]
        baseline: String,
        /// Write action-tracker CSV for the current report
        #[arg(long, value_name = "PATH")]
        actions_csv: Option<String>,
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

/// Push CLI `--jev-budget` into the process-wide spend cap (USD).
fn apply_jev_budget_from(cli: &Cli) {
    let usd = match &cli.command {
        Commands::Audit { jev_budget, .. }
        | Commands::Geo { jev_budget, .. }
        | Commands::Crawl { jev_budget, .. } => *jev_budget,
        _ => manifest::DEFAULT_JEV_BUDGET_USD,
    };
    manifest::set_jev_budget_usd(usd);
}

/// Write an action-tracker CSV beside optional export path (or given path).
fn write_actions_csv(path: &str, actions: &[actions::Action]) -> Result<()> {
    std::fs::write(path, actions::to_csv(actions))?;
    println!("Action tracker CSV written to {}", path.dimmed());
    Ok(())
}

/// Diff two DirectoryAuditReport JSONs: score delta, rule set changes.
fn diff_audit_reports(current: &audit::DirectoryAuditReport, base: &audit::DirectoryAuditReport) -> serde_json::Value {
    use std::collections::BTreeSet;
    let score = |r: &audit::DirectoryAuditReport| r.pass_rate.round().clamp(0.0, 100.0) as u32;
    let rules = |r: &audit::DirectoryAuditReport| -> BTreeSet<String> {
        r.findings.iter().map(|f| f.rule_id.clone()).collect()
    };
    let cur = rules(current);
    let old = rules(base);
    let fixed: Vec<String> = old.difference(&cloned_set(&cur)).cloned().collect();
    let new: Vec<String> = cur.difference(&cloned_set(&old)).cloned().collect();
    let sc_now = score(current);
    let sc_old = score(base);
    json!({
        "baseline_score": sc_old,
        "current_score": sc_now,
        "delta": sc_now as i32 - sc_old as i32,
        "grade": crate::actions::grade(sc_now),
        "rules_fixed": fixed,
        "rules_new": new,
        "findings_baseline": base.findings.len(),
        "findings_current": current.findings.len(),
        "files_baseline": base.total_files,
        "files_current": current.total_files,
    })
}

fn cloned_set(s: &std::collections::BTreeSet<String>) -> std::collections::BTreeSet<String> {
    s.clone()
}

/// Shared by CLI `report` and MCP `seo_report`.
pub(crate) fn main_report_diff(
    current: &audit::DirectoryAuditReport,
    base: &audit::DirectoryAuditReport,
) -> serde_json::Value {
    diff_audit_reports(current, base)
}

/// Test seam for report diff (binary crate cannot use crate::main helpers from tests otherwise).
#[cfg(test)]
pub(crate) fn main_shim_diff(
    current: &audit::DirectoryAuditReport,
    base: &audit::DirectoryAuditReport,
) -> serde_json::Value {
    diff_audit_reports(current, base)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    apply_jev_budget_from(&cli);

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
            let extras = policy::keyword_value_extras(&suggestions, 10);
            if let Some((eval, v)) = gated_eval_with("keywords", state, extras) {
                println!("\n{}", "Intent & Semantic Classification:".cyan().bold());
                println!("  Primary Intent: {} (confidence: {:.2}){}", eval.intent.green(), eval.intent_confidence, policy::marker(v));
                println!("  Content Gap:    {}", eval.content_gap.yellow());
                print_keyword_values(&eval.extra, &suggestions);
                println!("  Suggested Next: jev-seo {}", policy::route_for_intent(&eval.intent));
                print_jev_spend_line();
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
                    print_jev_spend_line();
                }
                Some(Err(e)) => eprintln!("{}", format!("Warning: Jev scoring failed ({e:#}), showing local-only output.").yellow()),
                None => eprintln!("{}", "Note: TYPESAFE_API_KEY not set, showing local-only output.".yellow()),
            }
        }
        Commands::Audit { path, target_query, json, min_pass, html, pdf, md, csv, actions_csv, actions_xls, pairs_csv, rescore, manifest, no_jev, jev_budget: _ } => {
            let t0 = std::time::Instant::now();
            let is_rescore = rescore.is_some();
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

            let actions = crate::rules::actions_for(&dir_report.findings);
            // Max Jev: speculative page suite on the largest pages BEFORE any ledger snapshot.
            let jev_pages = if !is_rescore && !no_jev {
                judge_audit_sample(&dir_report, target_query.as_deref(), 12)
            } else {
                Vec::new()
            };
            let jev_used = !jev_pages.is_empty() || (target_query.is_some() && manifest::jev_key_present());
            let mut completeness = manifest::completeness_audit(dir_report.total_files, false, !jev_pages.is_empty() || target_query.is_some());
            if !jev_pages.is_empty() {
                completeness.notes.push(format!("Jev judged {} pages (sample)", jev_pages.len()));
                completeness.full = false;
            }

            // Optional narrative.json beside exports (or --manifest dir); else auto summary.
            let mut export_dirs = manifest::auto_dirs(&[
                html.as_deref(),
                pdf.as_deref(),
                md.as_deref(),
                csv.as_deref(),
            ]);
            if let Some(dir) = &manifest {
                export_dirs = vec![std::path::PathBuf::from(dir)];
            } else if !export_dirs.iter().any(|d| d.join("narrative.json").is_file()) {
                let cwd = std::path::PathBuf::from(".");
                if cwd.join("narrative.json").is_file() {
                    export_dirs.push(cwd);
                }
            }
            let mut n_story: Option<narrative::Narrative> = None;
            for d in export_dirs.iter() {
                match narrative::load(d, &dir_report.findings, &actions, dir_report.total_files) {
                    Ok(Some(n)) => {
                        n_story = Some(n);
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => anyhow::bail!("{e:#}"),
                }
            }
            if n_story.is_none() {
                n_story = Some(narrative::auto(
                    &dir_report.findings,
                    &actions,
                    dir_report.total_files,
                    dir_report.pass_rate,
                ));
            }
            let n = n_story.as_ref().expect("narrative always set");
            if !n.unverified_numbers.is_empty() {
                eprintln!(
                    "{} narrative numbers not in audit: {}",
                    "warn".yellow(),
                    n.unverified_numbers.join(", ")
                );
            }
            if !n.effort_mismatches.is_empty() {
                eprintln!(
                    "{} narrative effort mismatches: {}",
                    "warn".yellow(),
                    n.effort_mismatches.join("; ")
                );
            }
            let n_md = narrative::to_markdown(n);
            let n_html = narrative::to_html(n);

            // Build report bodies once, gate every citation, then write.
            let body_html = html.as_ref().map(|_| audit::to_html(&dir_report) + &n_html);
            let body_pdf = pdf.as_ref().map(|_| audit::to_pdf_with_narrative(&dir_report, n));
            let body_md = md.as_ref().map(|_| audit::to_markdown(&dir_report) + &n_md);
            let body_csv = csv.as_ref().map(|_| crate::rules::to_csv(&dir_report.findings));
            if let Some(body) = &body_html {
                manifest::gate_report(body, &dir_report.findings, &actions)?;
            }
            if let Some(body) = &body_pdf {
                manifest::gate_report(&String::from_utf8_lossy(body), &dir_report.findings, &actions)?;
            }
            if let Some(body) = &body_md {
                manifest::gate_report(body, &dir_report.findings, &actions)?;
            }
            if let Some(body) = &body_csv {
                manifest::gate_report(body, &dir_report.findings, &actions)?;
            }
            if let Some(out) = &html {
                std::fs::write(out, body_html.as_deref().unwrap_or_default())?;
                println!("HTML report written to {}", out.dimmed());
            }
            if let Some(out) = &pdf {
                std::fs::write(out, body_pdf.as_deref().unwrap_or_default())?;
                println!("PDF report written to {}", out.dimmed());
            }
            if let Some(out) = &md {
                std::fs::write(out, body_md.as_deref().unwrap_or_default())?;
                println!("Markdown report written to {}", out.dimmed());
            }
            if let Some(out) = &csv {
                std::fs::write(out, body_csv.as_deref().unwrap_or_default())?;
                println!("CSV findings written to {}", out.dimmed());
            }
            if let Some(out) = &actions_csv {
                write_actions_csv(out, &actions)?;
            }
            if let Some(out) = &actions_xls {
                std::fs::write(out, actions::to_spreadsheet_xml(&actions))?;
                println!("Action tracker SpreadsheetML written to {}", out.dimmed());
            }

            let texts_owned: Vec<String> = body_html
                .iter()
                .chain(body_md.iter())
                .chain(body_csv.iter())
                .cloned()
                .collect();
            let texts: Vec<&str> = texts_owned.iter().map(|s| s.as_str()).collect();

            // Ledger LAST so Jev tokens from judge_audit_sample are included.
            let mut ledger = manifest::Ledger::new("audit", &path);
            if !jev_pages.is_empty() {
                ledger.note(format!("Jev page suite: {} pages judged", jev_pages.len()));
            } else if target_query.is_some() {
                ledger.note(if jev_used {
                    "Jev target-query scoring available"
                } else {
                    "Jev target-query requested but TYPESAFE_API_KEY missing"
                });
            }
            let score = manifest::ScoreCard {
                score: dir_report.pass_rate.round().clamp(0.0, 100.0) as u32,
                grade: crate::actions::grade(dir_report.pass_rate.round().clamp(0.0, 100.0) as u32).to_string(),
                kind: "pass-rate".into(),
            };
            let run_manifest = manifest::ManifestInput {
                command: "audit",
                target: &path,
                findings: &dir_report.findings,
                actions: &actions,
                texts: &texts,
                score: Some(score),
                completeness: completeness.clone(),
                ledger,
            }
            .build();
            if !run_manifest.validation.ok {
                anyhow::bail!(
                    "run manifest citation gate failed: {:?}",
                    run_manifest.validation
                );
            }
            let force = manifest.is_some();
            let mut wrote_manifest = false;
            for d in &export_dirs {
                let (run_p, led_p) = manifest::write_pair(d, &run_manifest)?;
                println!("Run manifest written to {}", run_p.display().to_string().dimmed());
                println!("Spend ledger written to {}", led_p.display().to_string().dimmed());
                wrote_manifest = true;
            }
            if !wrote_manifest && (force || !json) {
                let (run_p, led_p) = manifest::write_pair(std::path::Path::new("."), &run_manifest)?;
                println!("Run manifest written to {}", run_p.display().to_string().dimmed());
                println!("Spend ledger written to {}", led_p.display().to_string().dimmed());
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
                manifest::print_banner(&completeness);
                print_jev_page_suite(&jev_pages);

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
                    let pairs = audit::cannibalization_pairs(&dir_report);
                    if !pairs.is_empty() {
                        println!("\n{}", format!("Conflict pairs ({}):", pairs.len()).yellow().bold());
                        for p in pairs.iter().take(10) {
                            println!(
                                "  \"{}\"  {} ({}w) × {} ({}w)  → keep {}",
                                p.keyword_stem.cyan(),
                                p.a,
                                p.a_words,
                                p.b,
                                p.b_words,
                                p.winner.green()
                            );
                        }
                        if pairs.len() > 10 {
                            println!("    ... and {} more pairs", pairs.len() - 10);
                        }
                        if let Some(out) = &pairs_csv {
                            let mut s = String::from("stem,a,b,a_words,b_words,winner\n");
                            let cell = |v: &str| format!("\"{}\"", v.replace('"', "\"\""));
                            for p in &pairs {
                                s.push_str(&format!(
                                    "{},{},{},{},{},{}\n",
                                    cell(&p.keyword_stem),
                                    cell(&p.a),
                                    cell(&p.b),
                                    p.a_words,
                                    p.b_words,
                                    cell(&p.winner)
                                ));
                            }
                            std::fs::write(out, s)?;
                            println!("Cannibalization pairs CSV written to {}", out.dimmed());
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
                manifest::print_banner(&completeness);
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
                        "checks": report.checks,
                        "page": {
                            "title": report.title,
                            "description": report.description,
                            "text": excerpt_local(&report.file_path),
                            "word_count": report.word_count
                        }
                    });
                    if let Some((eval, v)) = gated_eval_with("audit", state, merge_extras(policy::geo_questions(), policy::page_audit_extras())) {
                        println!("\n{}", "Semantic Gap Evaluation (TypeSafe Jev):".cyan().bold());
                        println!("  GEO Score:       {}/10{}", eval.geo_score, policy::marker(v));
                        if let Some((comp, cconf)) = policy::composite_geo(&eval.extra) {
                            println!("  Composite GEO:   {}/10 (confidence {:.2})", comp, cconf);
                        }
                        println!("  Direct Answer:   {} (p={:.2})", if eval.direct_answer { "YES".green() } else { "NO".red() }, eval.direct_answer_p);
                        println!("  Content Gap:     {}", eval.content_gap.yellow());
                        print_page_extras(&eval.extra);
                    }
                } else if !jev_pages.is_empty() {
                    // Single-file run still got the suite from the sample path.
                }
            }
        }
        Commands::Geo { target, query, json, jev_budget: _ } => {
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
                    "content": content,
                    "page": { "text": content.chars().take(8000).collect::<String>(), "title": target }
                });
                match client.judge_page(state) {
                    Ok(eval) => {
                        if policy::injection_blocked(&eval.extra) {
                            eprintln!("{}", "Jev blocked: injection risk in content (preflight); no score.".yellow());
                            print_jev_spend_line();
                            return Ok(());
                        }
                        let v = policy::gate("geo", eval.confidence());
                        if v == policy::Verdict::Drop {
                            eprintln!("{}", format!("Jev unsure (confidence {:.2}), no score.", eval.confidence()).yellow());
                            print_jev_spend_line();
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
                            print_page_extras(&eval.extra);
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
                        print_jev_spend_line();
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
        Commands::Crawl { url, max_pages, fetch, max_credits, json, diff, csv, rescore, manifest, no_jev, jev_budget: _ } => {
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
                let completeness = manifest::completeness_crawl(&report, max_pages);
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("  Rescored:      {}/100 ({})", report.score, crate::actions::grade(report.score));
                    manifest::print_banner(&completeness);
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
            // Max Jev: one site+GEO fan-out on the homepage before any ledger snapshot.
            let site_jev = if no_jev { None } else { judge_crawl_site(&report.start_url) };
            let mut completeness_pre = manifest::completeness_crawl(&report, max_pages);
            if site_jev.is_some() {
                completeness_pre
                    .notes
                    .push("Jev site+GEO judged on homepage".into());
            }
            if json {
                // Still freeze the contract for agents piping --json.
                let mut ledger = manifest::Ledger::new("crawl", &report.start_url);
                let backends: Vec<String> = report
                    .pages
                    .iter()
                    .filter(|p| p.source != "direct")
                    .map(|p| p.source.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                ledger.with_fetch(budget.spent, budget.max_credits, backends);
                if site_jev.is_some() {
                    ledger.note("Jev site+GEO fan-out on homepage");
                }
                let run_manifest = manifest::ManifestInput {
                    command: "crawl",
                    target: &report.start_url,
                    findings: &report.findings,
                    actions: &report.actions,
                    texts: &[],
                    score: Some(manifest::ScoreCard {
                        score: report.score,
                        grade: report.grade.clone(),
                        kind: "health".into(),
                    }),
                    completeness: completeness_pre,
                    ledger,
                }
                .build();
                if !run_manifest.validation.ok {
                    anyhow::bail!("run manifest citation gate failed: {:?}", run_manifest.validation);
                }
                let dirs = match &manifest {
                    Some(dir) => vec![std::path::PathBuf::from(dir)],
                    None => manifest::auto_dirs(&[csv.as_deref()]),
                };
                let dirs = if dirs.is_empty() {
                    vec![std::path::PathBuf::from(".")]
                } else {
                    dirs
                };
                for d in &dirs {
                    let (run_p, led_p) = manifest::write_pair(d, &run_manifest)?;
                    eprintln!("Run manifest written to {}", run_p.display());
                    eprintln!("Spend ledger written to {}", led_p.display());
                }
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
                let body = crate::rules::to_csv(&report.findings);
                manifest::gate_report(&body, &report.findings, &report.actions)?;
                std::fs::write(out, body)?;
                println!("CSV findings written to {}", out.dimmed());
            }

            let mut completeness = manifest::completeness_crawl(&report, max_pages);
            if site_jev.is_some() {
                completeness
                    .notes
                    .push("Jev site+GEO judged on homepage".into());
            }
            manifest::print_banner(&completeness);
            if let Some(eval) = &site_jev {
                let v = policy::gate("crawl", eval.confidence());
                println!("\n{}", "Jev site view (TypeSafe):".cyan().bold());
                println!(
                    "  GEO:            {}/10{}",
                    eval.geo_score,
                    policy::marker(v)
                );
                if let Some((c, cf)) = policy::composite_geo(&eval.extra) {
                    println!("  Composite GEO:   {}/10 (confidence {:.2})", c, cf);
                }
                println!(
                    "  Intent:         {} (confidence {:.2})",
                    eval.intent.green(),
                    eval.intent_confidence
                );
                println!("  Value prop:     {}", eval.content_gap.yellow());
                print_page_extras(&eval.extra);
                print_jev_spend_line();
            }
            let mut ledger = manifest::Ledger::new("crawl", &report.start_url);
            if site_jev.is_some() {
                ledger.note("Jev site+GEO fan-out on homepage");
            }
            let backends: Vec<String> = report
                .pages
                .iter()
                .filter(|p| p.source != "direct")
                .map(|p| p.source.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            ledger.with_fetch(budget.spent, budget.max_credits, backends);
            if budget.spent > 0 {
                ledger.note(format!("{} paid fetch credits spent (cap {})", budget.spent, budget.max_credits));
            }
            let run_manifest = manifest::ManifestInput {
                command: "crawl",
                target: &report.start_url,
                findings: &report.findings,
                actions: &report.actions,
                texts: &[],
                score: Some(manifest::ScoreCard {
                    score: report.score,
                    grade: report.grade.clone(),
                    kind: "health".into(),
                }),
                completeness: completeness.clone(),
                ledger,
            }
            .build();
            if !run_manifest.validation.ok {
                anyhow::bail!("run manifest citation gate failed: {:?}", run_manifest.validation);
            }
            let mut export_dirs = manifest::auto_dirs(&[csv.as_deref()]);
            if let Some(dir) = &manifest {
                export_dirs = vec![std::path::PathBuf::from(dir)];
            }
            let force = manifest.is_some();
            let mut wrote_manifest = false;
            for d in &export_dirs {
                let (run_p, led_p) = manifest::write_pair(d, &run_manifest)?;
                println!("Run manifest written to {}", run_p.display().to_string().dimmed());
                println!("Spend ledger written to {}", led_p.display().to_string().dimmed());
                wrote_manifest = true;
            }
            if !wrote_manifest && (force || !json) {
                let (run_p, led_p) = manifest::write_pair(std::path::Path::new("."), &run_manifest)?;
                println!("Run manifest written to {}", run_p.display().to_string().dimmed());
                println!("Spend ledger written to {}", led_p.display().to_string().dimmed());
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
            let completeness = manifest::completeness_llms(&report);
            manifest::print_banner(&completeness);
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
        Commands::Explain { id, json } => {
            let text = rules::explain(&id)
                .ok_or_else(|| anyhow::anyhow!("unknown rule id: {id} (try R01..R50 or RULE-R01..RULE-R50)"))?;
            if json {
                let bare = id.strip_prefix("RULE-").unwrap_or(&id).to_string();
                let r = rules::rule(&bare).expect("explain found it");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "id": r.id,
                        "area": rules::label(&r.area),
                        "severity": format!("{:?}", r.severity),
                        "effort": r.effort,
                        "title": r.title,
                        "fix": r.fix,
                    }))?
                );
                return Ok(());
            }
            println!("{}", text.cyan());
        }
        Commands::Report { path, baseline, actions_csv, json } => {
            let cur: audit::DirectoryAuditReport = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
            let base: audit::DirectoryAuditReport = serde_json::from_str(&std::fs::read_to_string(&baseline)?)?;
            let diff = diff_audit_reports(&cur, &base);
            let actions = crate::rules::actions_for(&cur.findings);
            if let Some(out) = &actions_csv {
                write_actions_csv(out, &actions)?;
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&json!({
                    "diff": diff,
                    "actions": actions,
                }))?);
                return Ok(());
            }
            let delta = diff["delta"].as_i64().unwrap_or(0);
            let sign = if delta > 0 { "+" } else { "" };
            println!("\n{}", "Report diff (baseline → current):".cyan().bold());
            println!(
                "  Score:  {} → {} ({sign}{delta}) grade {}",
                diff["baseline_score"],
                diff["current_score"],
                diff["grade"]
            );
            println!(
                "  Findings: {} → {}",
                diff["findings_baseline"], diff["findings_current"]
            );
            let fixed: Vec<String> = serde_json::from_value(diff["rules_fixed"].clone())?;
            let added: Vec<String> = serde_json::from_value(diff["rules_new"].clone())?;
            if fixed.is_empty() && added.is_empty() {
                println!("  Rules: no new or cleared rule ids");
            } else {
                if !fixed.is_empty() {
                    println!("  Cleared: {}", fixed.join(", ").green());
                }
                if !added.is_empty() {
                    println!("  New:     {}", added.join(", ").yellow());
                }
            }
            if !actions.is_empty() {
                println!("\n{}", "Top actions:".cyan().bold());
                for a in actions::top(&actions, 5) {
                    println!(
                        "  [P{}] {}  {}  impact {}{}",
                        a.priority,
                        a.id.bold(),
                        a.title,
                        a.impact,
                        if a.quick_win { "  quick-win".green().to_string() } else { String::new() }
                    );
                }
            }
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
#[allow(dead_code)]
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
                print_jev_spend_line();
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

fn merge_extras(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    let mut out = a;
    if let (Some(dst), Some(src)) = (out.as_object_mut(), b.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    out
}

fn excerpt_local(path: &str) -> String {
    std::fs::read_to_string(path)
        .map(|s| {
            let cleaned: String = s
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .collect();
            cleaned.chars().take(8000).collect()
        })
        .unwrap_or_default()
}

/// Speculative Jev page suite on the largest audit files. One fan-out per page.
/// Returns (path, eval, verdict) for pages that passed the confidence gate.
fn judge_audit_sample(
    dir_report: &audit::DirectoryAuditReport,
    target_query: Option<&str>,
    limit: usize,
) -> Vec<(String, engine::AnalysisResult, policy::Verdict)> {
    let client = match engine::JevClient::new() {
        Some(c) => c,
        None => return Vec::new(),
    };
    let mut pages: Vec<&audit::AuditReport> = dir_report.reports.iter().collect();
    pages.sort_by_key(|b| std::cmp::Reverse(b.word_count));
    let mut out = Vec::new();
    for report in pages.into_iter().take(limit) {
        let content = excerpt_local(&report.file_path);
        if content.trim().is_empty() {
            continue;
        }
        let mut state = serde_json::json!({
            "target_query": target_query.unwrap_or(""),
            "page": {
                "title": report.title,
                "description": report.description,
                "text": content,
                "word_count": report.word_count,
                "checks": report.checks,
            }
        });
        if let Some(obj) = state.as_object_mut() {
            obj.insert("content".into(), serde_json::Value::String(content.clone()));
            if let Some(q) = target_query {
                obj.insert("query".into(), serde_json::Value::String(q.to_string()));
            }
        }
        match client.judge_page(state) {
            Ok(eval) => {
                if policy::injection_blocked(&eval.extra) {
                    eprintln!(
                        "{}",
                        format!(
                            "Blocked: injection risk in {} (dedicated Noul pre-screen); quality suite not run.",
                            report.file_path
                        )
                        .yellow()
                    );
                    continue;
                }
                let v = policy::gate("audit", eval.confidence());
                if v != policy::Verdict::Drop {
                    out.push((report.file_path.clone(), eval, v));
                }
            }
            Err(e) => {
                eprintln!("{}", format!("Jev page judge failed for {}: {e:#}", report.file_path).yellow());
            }
        }
    }
    out
}

fn print_jev_page_suite(pages: &[(String, engine::AnalysisResult, policy::Verdict)]) {
    if pages.is_empty() {
        return;
    }
    println!("\n{}", format!("Jev page suite ({} pages, full fan-out):", pages.len()).cyan().bold());
    for (path, eval, v) in pages {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        let review = policy::needs_review(&eval.extra, "audit");
        println!(
            "  {} GEO {}/10 intent {} gap {}{}",
            name,
            eval.geo_score,
            eval.intent,
            eval.content_gap,
            policy::marker(*v)
        );
        print_page_extras(&eval.extra);
        if !review.is_empty() {
            println!("      needs review: {}", review.join(", ").dimmed());
        }
    }
    print_jev_spend_line();
}

fn print_page_extras(extra: &serde_json::Map<String, serde_json::Value>) {
    let keys = [
        "page_helpfulness",
        "page_trust",
        "page_specificity",
        "title_fit",
        "meta_fit",
        "importance",
        "value_prop",
        "geo_statistics",
        "geo_directness",
        "answer_first",
        "clear_next_step",
        "entity_clarity",
        "injection_risk",
    ];
    let mut parts = Vec::new();
    for k in keys {
        if let Some(a) = extra.get(k) {
            if let Some(s) = a.get("score").and_then(|x| x.as_f64()) {
                let c = a.get("confidence").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let norm = normalize_score(a).unwrap_or(s);
                parts.push(format!("{k}={norm:.2}({c:.2})"));
            } else if let Some(n) = a.get("noul").or_else(|| a.get("probability")).and_then(|x| x.as_f64()) {
                parts.push(format!("{k}=P{n:.2}"));
            } else if let Some(ch) = a.get("choice").and_then(|x| x.as_str()) {
                parts.push(format!("{k}={ch}"));
            }
        }
    }
    if !parts.is_empty() {
        println!("      {}", parts.join(" · ").dimmed());
    }
}

/// Score answers are positions on ordered levels (0..N). Map to 0..1 using the
/// legend keys so display matches confidence scale (skill: score is not probability,
/// but comparing levels as a fraction is what the rubric means).
fn normalize_score(a: &serde_json::Value) -> Option<f64> {
    let s = a.get("score")?.as_f64()?;
    let legend = a.get("legend")?;
    let top = legend
        .as_object()?
        .keys()
        .filter_map(|k| k.parse::<f64>().ok())
        .fold(0.0f64, f64::max);
    if top <= 0.0 {
        return Some(s.clamp(0.0, 1.0));
    }
    Some((s / top).clamp(0.0, 1.0))
}

fn print_keyword_values(extra: &serde_json::Map<String, serde_json::Value>, suggestions: &[String]) {
    let mut rows = Vec::new();
    for (i, s) in suggestions.iter().take(10).enumerate() {
        if let Some(a) = extra.get(format!("kw_value_{i}").as_str()) {
            if let Some(sc) = a.get("score").and_then(|x| x.as_f64()) {
                rows.push(format!("{sc:.1} {}", s));
            }
        }
    }
    if !rows.is_empty() {
        println!("  Query value:    {}", rows.join(" · "));
    }
}

fn print_jev_spend_line() {
    let reqs = manifest::JEV_REQUESTS.load(std::sync::atomic::Ordering::Relaxed);
    let toks = manifest::JEV_INPUT_TOKENS.load(std::sync::atomic::Ordering::Relaxed);
    let cost = manifest::jev_cost_usd(toks);
    if reqs > 0 {
        println!(
            "  Jev spend:      {} request(s), {} input tokens, ${:.6}",
            reqs, toks, cost
        );
    }
}

/// Fetch homepage HTML and run one site+GEO fan-out for crawl runs.
fn judge_crawl_site(start_url: &str) -> Option<engine::AnalysisResult> {
    let client = engine::JevClient::new()?;
    let body = ureq::get(start_url)
        .timeout(std::time::Duration::from_secs(15))
        .set("User-Agent", crawl::CRAWL_UA)
        .call()
        .ok()?
        .into_string()
        .ok()?;
    if body.trim().is_empty() {
        return None;
    }
    let tmp = std::env::temp_dir().join(format!("jev-seo-site-{}.html", std::process::id()));
    std::fs::write(&tmp, &body).ok()?;
    let report = audit::audit_file(tmp.to_str()?).ok();
    let _ = std::fs::remove_file(&tmp);
    let text: String = body.chars().take(8000).collect();
    let state = serde_json::json!({
        "query": start_url,
        "content": text,
        "page": {
            "title": report.as_ref().and_then(|r| r.title.clone()),
            "description": report.as_ref().and_then(|r| r.description.clone()),
            "text": text,
            "word_count": report.as_ref().map(|r| r.word_count).unwrap_or(0),
        }
    });
    match client.judge_site(state) {
        Ok(eval) => Some(eval),
        Err(e) => {
            eprintln!("{}", format!("Jev site judge failed: {e:#}").yellow());
            None
        }
    }
}
