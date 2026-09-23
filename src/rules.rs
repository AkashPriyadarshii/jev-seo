//! Fifty-rule audit engine. Rules are data, findings are facts.
//! Severity weights and reach factors turn findings into area scores;
//! area scores blend into one overall grade. Deterministic, no model calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    Crawl,
    OnPage,
    Content,
    Links,
    Structured,
    AiAccess,
    Performance,
    Security,
    Canonical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: &'static str,
    pub area: Area,
    pub severity: Severity,
    pub effort: u8,
    pub title: &'static str,
    pub fix: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub area: Area,
    pub severity: Severity,
    pub scope: String,
    pub evidence: String,
    pub fix: String,
}

/// The fifty. Ids are stable API: reports, docs, and tests cite them.
pub const RULES: &[Rule] = &[
    // Crawl (8)
    Rule { id: "R01", area: Area::Crawl, severity: Severity::High, effort: 1, title: "Broken link", fix: "Restore the target or point the link at a live page." },
    Rule { id: "R02", area: Area::Crawl, severity: Severity::Critical, effort: 2, title: "Fetch failure", fix: "Fix DNS, TLS, or server errors blocking the crawl." },
    Rule { id: "R03", area: Area::Crawl, severity: Severity::Medium, effort: 2, title: "Redirect chain", fix: "Point the source at the final URL in one hop." },
    Rule { id: "R04", area: Area::Crawl, severity: Severity::Critical, effort: 2, title: "Redirect loop", fix: "Break the cycle so the chain terminates." },
    Rule { id: "R05", area: Area::Crawl, severity: Severity::Medium, effort: 1, title: "No sitemap seed", fix: "Publish /sitemap.xml so crawlers find every page." },
    Rule { id: "R06", area: Area::Crawl, severity: Severity::Medium, effort: 1, title: "No robots.txt", fix: "Serve robots.txt so crawler rules are discoverable." },
    Rule { id: "R07", area: Area::Crawl, severity: Severity::Low, effort: 1, title: "Crawl capped", fix: "Raise --max-pages for full coverage." },
    Rule { id: "R08", area: Area::Crawl, severity: Severity::Low, effort: 2, title: "Empty page body", fix: "Ship renderable content or noindex the shell." },
    // On-page (9)
    Rule { id: "R09", area: Area::OnPage, severity: Severity::High, effort: 1, title: "Missing title", fix: "Write a unique title under 60 characters." },
    Rule { id: "R10", area: Area::OnPage, severity: Severity::Medium, effort: 1, title: "Title length", fix: "Keep titles 30-60 characters so nothing truncates." },
    Rule { id: "R11", area: Area::OnPage, severity: Severity::High, effort: 1, title: "Missing meta description", fix: "Write a 120-160 character summary." },
    Rule { id: "R12", area: Area::OnPage, severity: Severity::Medium, effort: 1, title: "Meta description length", fix: "Trim or expand into the 120-160 window." },
    Rule { id: "R13", area: Area::OnPage, severity: Severity::High, effort: 1, title: "Missing H1", fix: "Give the page one H1 stating its topic." },
    Rule { id: "R14", area: Area::OnPage, severity: Severity::Medium, effort: 1, title: "Multiple H1", fix: "Keep one H1, demote the rest to H2." },
    Rule { id: "R15", area: Area::OnPage, severity: Severity::Medium, effort: 1, title: "Skipped heading level", fix: "Nest headings in order without jumps." },
    Rule { id: "R16", area: Area::OnPage, severity: Severity::Low, effort: 2, title: "Images missing alt", fix: "Describe every informative image." },
    Rule { id: "R17", area: Area::OnPage, severity: Severity::Low, effort: 1, title: "Missing favicon signal", fix: "Link an icon so tabs and bookmarks render." },
    // Content (7)
    Rule { id: "R18", area: Area::Content, severity: Severity::Medium, effort: 2, title: "Thin page", fix: "Expand past 300 words or merge into a parent." },
    Rule { id: "R19", area: Area::Content, severity: Severity::Medium, effort: 2, title: "AI slop markers", fix: "Rewrite flagged boilerplate in plain words." },
    Rule { id: "R20", area: Area::Content, severity: Severity::Low, effort: 2, title: "Em-dash density", fix: "Hold em-dashes under 2 per 500 words." },
    Rule { id: "R21", area: Area::Content, severity: Severity::Medium, effort: 2, title: "Duplicate titles", fix: "Give every page a unique title." },
    Rule { id: "R22", area: Area::Content, severity: Severity::Medium, effort: 2, title: "Duplicate descriptions", fix: "Give every page a unique summary." },
    Rule { id: "R23", area: Area::Content, severity: Severity::Medium, effort: 2, title: "Keyword cannibalization", fix: "One winner per stem; merge or re-target the rest." },
    Rule { id: "R24", area: Area::Content, severity: Severity::Low, effort: 2, title: "No direct answer opening", fix: "Open with the answer before the background." },
    // Links (6)
    Rule { id: "R25", area: Area::Links, severity: Severity::Medium, effort: 2, title: "Orphan page", fix: "Link inward from a related page." },
    Rule { id: "R26", area: Area::Links, severity: Severity::Low, effort: 1, title: "No outbound links", fix: "Cite at least one source or next step." },
    Rule { id: "R27", area: Area::Links, severity: Severity::Medium, effort: 1, title: "HTTP outbound link", fix: "Point outbound links at HTTPS targets." },
    Rule { id: "R28", area: Area::Links, severity: Severity::Low, effort: 1, title: "Empty anchor text", fix: "Write anchors that describe the target." },
    Rule { id: "R29", area: Area::Links, severity: Severity::Low, effort: 2, title: "Nofollow-heavy page", fix: "Let internal links pass value; save nofollow for untrusted." },
    Rule { id: "R30", area: Area::Links, severity: Severity::Low, effort: 1, title: "Fragment-only links", fix: "Point navigation at real pages, not anchors." },
    // Structured (6)
    Rule { id: "R31", area: Area::Structured, severity: Severity::Medium, effort: 2, title: "No JSON-LD", fix: "Add Schema.org markup matching the page type." },
    Rule { id: "R32", area: Area::Structured, severity: Severity::Medium, effort: 1, title: "Missing rich-result props", fix: "Fill the properties search requires for rich results." },
    Rule { id: "R33", area: Area::Structured, severity: Severity::High, effort: 1, title: "Invalid JSON-LD", fix: "Fix the syntax so parsers accept the block." },
    Rule { id: "R34", area: Area::Structured, severity: Severity::Medium, effort: 1, title: "Deprecated schema type", fix: "Migrate to the current type." },
    Rule { id: "R35", area: Area::Structured, severity: Severity::Low, effort: 1, title: "Missing Open Graph tags", fix: "Add og:title, description, and image." },
    Rule { id: "R36", area: Area::Canonical, severity: Severity::Low, effort: 1, title: "Missing canonical", fix: "Point every page at its canonical URL." },
    // AI access (5)
    Rule { id: "R37", area: Area::AiAccess, severity: Severity::Medium, effort: 1, title: "No llms.txt", fix: "Ship /llms.txt so answer engines can read the site." },
    Rule { id: "R38", area: Area::AiAccess, severity: Severity::Low, effort: 1, title: "AI crawlers unnamed", fix: "Name AI crawlers explicitly in robots.txt." },
    Rule { id: "R39", area: Area::AiAccess, severity: Severity::High, effort: 1, title: "AI training blocked", fix: "Allow the crawlers whose citations are wanted." },
    Rule { id: "R40", area: Area::AiAccess, severity: Severity::Low, effort: 1, title: "Sitemap not advertised", fix: "Add the Sitemap line to robots.txt." },
    Rule { id: "R41", area: Area::AiAccess, severity: Severity::Low, effort: 2, title: "Low citation density", fix: "Raise facts per paragraph with numbers and definitions." },
    // Performance (5)
    Rule { id: "R42", area: Area::Performance, severity: Severity::Medium, effort: 2, title: "Slow page over 800ms", fix: "Cut server time and payload below the budget." },
    Rule { id: "R43", area: Area::Performance, severity: Severity::Low, effort: 2, title: "Oversize page body", fix: "Trim pages past 2MB or paginate." },
    Rule { id: "R44", area: Area::Performance, severity: Severity::Low, effort: 1, title: "No compression signal", fix: "Serve gzip or brotli with a content-encoding header." },
    Rule { id: "R45", area: Area::Performance, severity: Severity::Low, effort: 2, title: "Render-blocking weight", fix: "Defer non-critical scripts and styles." },
    Rule { id: "R46", area: Area::Performance, severity: Severity::Low, effort: 1, title: "Images without dimensions", fix: "Set width and height to stop layout shift." },
    // Security (2)
    Rule { id: "R47", area: Area::Security, severity: Severity::High, effort: 1, title: "Serves over HTTP", fix: "Redirect everything to HTTPS with HSTS." },
    Rule { id: "R48", area: Area::Security, severity: Severity::Medium, effort: 1, title: "Mixed content", fix: "Upgrade HTTP subresources to HTTPS." },
    // Canonical (2)
    Rule { id: "R49", area: Area::Canonical, severity: Severity::Medium, effort: 1, title: "Canonical variant duplicates", fix: "One URL per page: pick slash policy and enforce it." },
    Rule { id: "R50", area: Area::Canonical, severity: Severity::Low, effort: 1, title: "Tracking-param URLs indexed", fix: "Strip marketing params from canonicals and sitemaps." },
];

pub fn rule(id: &str) -> Option<&'static Rule> {
    let bare = id.strip_prefix("RULE-").unwrap_or(id);
    RULES.iter().find(|r| r.id.eq_ignore_ascii_case(bare))
}

/// One-line explain for `jev-seo explain RULE-R19`.
pub fn explain(id: &str) -> Option<String> {
    let r = rule(id)?;
    Some(format!(
        "{} | {} | {:?} | effort {} ({})\n  title: {}\n  fix:   {}",
        r.id,
        label(&r.area),
        r.severity,
        r.effort,
        match r.effort {
            1 => "hours",
            2 => "about a day",
            3 => "several days",
            _ => "a project",
        },
        r.title,
        r.fix
    ))
}

fn deduct(sev: Severity) -> f64 {
    match sev {
        Severity::Critical => 25.0,
        Severity::High => 12.0,
        Severity::Medium => 6.0,
        Severity::Low => 2.0,
    }
}

/// Area weights for the overall blend. Sums to 100.
pub const AREA_WEIGHTS: &[(Area, f64)] = &[
    (Area::Crawl, 15.0),
    (Area::OnPage, 15.0),
    (Area::Content, 15.0),
    (Area::Links, 10.0),
    (Area::Structured, 10.0),
    (Area::AiAccess, 10.0),
    (Area::Performance, 10.0),
    (Area::Security, 10.0),
    (Area::Canonical, 5.0),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaScore {
    pub area: Area,
    pub score: u32,
    pub findings: usize,
}

/// Score areas from findings. Reach = affected scopes / total scopes per area.
pub fn score_areas(findings: &[Finding], totals: &HashMap<Area, usize>) -> Vec<AreaScore> {
    let mut by_area: HashMap<Area, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_area.entry(f.area).or_default().push(f);
    }
    let mut out = Vec::new();
    for (area, _) in AREA_WEIGHTS {
        let empty: Vec<&Finding> = Vec::new();
        let list = by_area.get(area).unwrap_or(&empty);
        let total = totals.get(area).copied().unwrap_or(1).max(1) as f64;
        // Distinct scopes per rule so one bad page counts once per rule.
        let mut hits: HashMap<&str, std::collections::HashSet<&str>> = HashMap::new();
        for f in list {
            hits.entry(f.rule_id.as_str()).or_default().insert(f.scope.as_str());
        }
        let mut penalty = 0.0;
        for (rule_id, scopes) in &hits {
            let sev = list
                .iter()
                .find(|f| &f.rule_id == rule_id)
                .map(|f| f.severity)
                .unwrap_or(Severity::Low);
            let reach = (scopes.len() as f64 / total).min(1.0);
            penalty += deduct(sev) * (0.5 + 0.5 * reach);
        }
        out.push(AreaScore { area: *area, score: (100.0 - penalty).max(0.0).round() as u32, findings: list.len() });
    }
    out
}

/// Weighted mean over scored areas only.
pub fn overall(areas: &[AreaScore]) -> u32 {
    let mut total = 0.0;
    let mut weight = 0.0;
    for a in areas {
        let w = AREA_WEIGHTS.iter().find(|(ar, _)| ar == &a.area).map(|(_, w)| w).copied().unwrap_or(0.0);
        total += a.score as f64 * w;
        weight += w;
    }
    if weight == 0.0 {
        return 100;
    }
    (total / weight).round() as u32
}

/// Findings grouped into ranked actions. One action per rule, evidence capped.
pub fn actions_for(findings: &[Finding]) -> Vec<crate::actions::Action> {    let mut by_rule: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_rule.entry(f.rule_id.as_str()).or_default().push(f);
    }
    let mut ids: Vec<&str> = by_rule.keys().copied().collect();
    ids.sort();
    let mut out = Vec::new();
    for id in ids {
        let list = &by_rule[id];
        let first = list[0];
        let priority = match first.severity {
            Severity::Critical | Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
        };
        let effort = rule(id).map(|r| r.effort).unwrap_or(2);
        let title = rule(id).map(|r| r.title).unwrap_or("Fix flagged issue");
        let mut ev: Vec<String> = list.iter().take(3).map(|f| f.scope.clone()).collect();
        if list.len() > 3 {
            ev.push(format!("+{} more", list.len() - 3));
        }
        out.push(crate::actions::Action::new(
            &format!("RULE-{}", id),
            priority,
            effort,
            &format!("{} ({} hits)", title, list.len()),
            ev.join(", "),
        ));
    }
    crate::actions::rank(out)
}

/// Spreadsheet-ready export. Opens in Excel as the action tracker.
pub fn to_csv(findings: &[Finding]) -> String {
    let mut s = String::from("rule,area,severity,scope,evidence\n");
    for f in findings {
        let cell = |v: &str| format!("\"{}\"", v.replace('"', "\"\""));
        s.push_str(&format!(
            "{},{},{},{},{}\n",
            f.rule_id,
            label(&f.area),
            format!("{:?}", f.severity).to_lowercase(),
            cell(&f.scope),
            cell(&f.evidence)
        ));
    }
    s
}

/// Short display label per area.
pub fn label(area: &Area) -> &'static str {
    match area {
        Area::Crawl => "crawl",
        Area::OnPage => "on-page",
        Area::Content => "content",
        Area::Links => "links",
        Area::Structured => "structured",
        Area::AiAccess => "ai-access",
        Area::Performance => "performance",
        Area::Security => "security",
        Area::Canonical => "canonical",
    }
}

fn track_param(k: &str) -> bool {
    let k = k.to_lowercase();
    k == "amp" || k.starts_with("utm_") || k == "fbclid" || k == "gclid" || k == "msclkid" || k == "yclid"
}

fn has_tracking(url: &str) -> bool {
    url::Url::parse(url)
        .map(|u| u.query_pairs().any(|(k, _)| track_param(&k)))
        .unwrap_or(false)
}

fn mk(rule_id: &'static str, scope: String, evidence: String) -> Finding {
    let r = rule(rule_id).expect("rule id");
    Finding {
        rule_id: rule_id.to_string(),
        area: r.area,
        severity: r.severity,
        scope,
        evidence,
        fix: r.fix.to_string(),
    }
}

/// Live findings from a crawl report.
pub fn check_crawl(rep: &crate::crawl::CrawlReport) -> Vec<Finding> {
    use crate::crawl::{MAX_BODY_BYTES, SLOW_PAGE_MS};
    let mut out = Vec::new();
    for p in &rep.pages {
        if p.status == 0 {
            out.push(mk("R02", p.url.clone(), "fetch failed".into()));
        } else if p.status >= 400 {
            out.push(mk("R01", p.url.clone(), format!("HTTP {}", p.status)));
        }
        if p.status == 200 && p.bytes == 0 {
            out.push(mk("R08", p.url.clone(), "empty body".into()));
        }
        if p.status == 200 && p.elapsed_ms > SLOW_PAGE_MS {
            out.push(mk("R42", p.url.clone(), format!("{}ms", p.elapsed_ms)));
        }
        if p.bytes > MAX_BODY_BYTES {
            out.push(mk("R43", p.url.clone(), format!("{} bytes", p.bytes)));
        }
        if p.status == 200
            && p.encoding
                .as_deref()
                .map(|e| !e.contains("gzip") && !e.contains("br") && !e.contains("zstd"))
                .unwrap_or(true)
        {
            out.push(mk("R44", p.url.clone(), "no content-encoding".into()));
        }
        if p.url.starts_with("http://") {
            out.push(mk("R47", p.url.clone(), "plain HTTP".into()));
        }
        if p.hops.len() >= 5 || p.hops.iter().any(|(_, u)| u == &p.final_url) {
            out.push(mk("R04", p.url.clone(), format!("{} hops", p.hops.len())));
        }
        if has_tracking(&p.final_url) {
            out.push(mk("R50", p.url.clone(), "tracking params served".into()));
        }
        if p.final_url.trim_end_matches('/') != p.url.trim_end_matches('/')
            && crate::crawl::canonicalize(&p.final_url) == p.url
        {
            out.push(mk("R49", p.url.clone(), format!("serves variant {}", p.final_url)));
        }
    }
    for (from, to) in &rep.redirects {
        out.push(mk("R03", from.clone(), format!("-> {}", to)));
    }
    if !rep.seeded_from_sitemap {
        out.push(mk("R05", rep.start_url.clone(), "no /sitemap.xml".into()));
    }
    if !rep.robots_honored {
        out.push(mk("R06", rep.start_url.clone(), "no robots.txt".into()));
    }
    if rep.capped {
        out.push(mk("R07", rep.start_url.clone(), format!("capped at {} pages", rep.pages_crawled)));
    }
    out
}

/// Local findings from a directory audit report.
pub fn check_audit(rep: &crate::audit::DirectoryAuditReport) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in &rep.reports {
        let scope = r.file_path.clone();
        match &r.title {
            None => out.push(mk("R09", scope.clone(), "no title".into())),
            Some(t) => {
                if r.title_len < 30 || r.title_len > 60 {
                    out.push(mk("R10", scope.clone(), format!("{} chars: {}", r.title_len, t)));
                }
            }
        }
        match &r.description {
            None => out.push(mk("R11", scope.clone(), "no description".into())),
            Some(_) => {
                if r.description_len < 120 || r.description_len > 160 {
                    out.push(mk("R12", scope.clone(), format!("{} chars", r.description_len)));
                }
            }
        }
        if r.h1_count == 0 {
            out.push(mk("R13", scope.clone(), "no H1".into()));
        } else if r.h1_count > 1 {
            out.push(mk("R14", scope.clone(), format!("{} H1", r.h1_count)));
        }
        if !r.heading_skipped_levels.is_empty() {
            out.push(mk("R15", scope.clone(), r.heading_skipped_levels.join(", ")));
        }
        if r.images_missing_alt > 0 {
            out.push(mk("R16", scope.clone(), format!("{} missing alt", r.images_missing_alt)));
        }
        if r.word_count < 300 {
            out.push(mk("R18", scope.clone(), format!("{} words", r.word_count)));
        }
        if r.em_dash_count > 0 || !r.ai_slop_words_found.is_empty() {
            out.push(mk(
                "R19",
                scope.clone(),
                format!("{} em-dashes, {}", r.em_dash_count, r.ai_slop_words_found.join(", ")),
            ));
        }
        if r.word_count > 0 && r.em_dash_count * 500 / r.word_count.max(1) > 2 {
            out.push(mk("R20", scope.clone(), format!("{} em-dashes", r.em_dash_count)));
        }
        if r.internal_links == 0 && r.external_links == 0 {
            out.push(mk("R26", scope.clone(), "no outbound links".into()));
        }
        if !r.schema_found {
            out.push(mk("R31", scope.clone(), "no JSON-LD".into()));
        }
        if !r.canonical_found {
            out.push(mk("R36", scope.clone(), "no canonical".into()));
        }
        if !r.og_tags_found {
            out.push(mk("R35", scope.clone(), "no Open Graph tags".into()));
        }
    }
    for (title, files) in &rep.duplicate_titles {
        out.push(mk("R21", files.first().cloned().unwrap_or_default(), format!("shared title: {}", title)));
    }
    let mut by_desc: HashMap<&str, Vec<&str>> = HashMap::new();
    for r in &rep.reports {
        if let Some(d) = &r.description {
            by_desc.entry(d.as_str()).or_default().push(r.file_path.as_str());
        }
    }
    for (desc, files) in &by_desc {
        if files.len() > 1 {
            out.push(mk("R22", files[0].to_string(), format!("shared description: {}", desc)));
        }
    }
    for item in &rep.keyword_cannibalization {
        out.push(mk(
            "R23",
            item.colliding_files.first().cloned().unwrap_or_default(),
            format!("stem: {}", item.keyword_stem),
        ));
    }
    for f in &rep.orphan_pages {
        out.push(mk("R25", f.clone(), "0 inbound links".into()));
    }
    out
}
