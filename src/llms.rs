//! Agent-readiness check: llms.txt presence plus AI crawler permissions.
//! Scores a domain on what an answer engine needs before it can cite it.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmsInfo {
    pub present: bool,
    pub status: u16,
    pub bytes: usize,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmsReport {
    pub domain: String,
    pub llms_url: String,
    pub info: LlmsInfo,
    pub ai_allowed: Vec<String>,
    pub ai_default: Vec<String>,
    pub ai_blocked: Vec<String>,
    pub robots_sitemaps: Vec<String>,
    pub robots_present: bool,
    pub score: u32,
    pub checks: Vec<crate::audit::CheckItem>,
    pub actions: Vec<crate::actions::Action>,
}

/// Headings are lines starting with '#'. Pure function, no network.
pub fn parse_llms_txt(body: &str) -> (usize, Vec<String>) {
    let sections: Vec<String> = body
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .filter(|l| !l.is_empty())
        .take(20)
        .collect();
    (body.len(), sections)
}

fn check(name: &str, passed: bool, message: &str) -> crate::audit::CheckItem {
    crate::audit::CheckItem {
        name: name.to_string(),
        passed,
        message: message.to_string(),
    }
}

pub fn check_llms(target: &str) -> Result<LlmsReport> {
    let base = if target.starts_with("http://") || target.starts_with("https://") {
        Url::parse(target)?
    } else {
        Url::parse(&format!("https://{}", target))?
    };
    crate::paths::reject_private_url(base.as_str())?;
    let domain = base.host_str().unwrap_or(target).to_string();
    let llms_url = format!("{}://{}/llms.txt", base.scheme(), domain);

    let (status, body) = match crate::fetch::with_extra_headers(
        ureq::get(&llms_url)
            .timeout(Duration::from_secs(8))
            .set("User-Agent", concat!("jev-seo/", env!("CARGO_PKG_VERSION"), " (TypeSafe Jev Agent Readiness Check)")),
    )
    .call()
    {
        Ok(r) => (r.status(), crate::fetch::capped_string(r, crate::fetch::MAX_AUX_BYTES).unwrap_or_default()),
        Err(ureq::Error::Status(code, r)) => (code, crate::fetch::capped_string(r, crate::fetch::MAX_AUX_BYTES).unwrap_or_default()),
        Err(_) => (0, String::new()),
    };
    let present = status == 200 && !body.trim().is_empty();
    let (bytes, sections) = if present { parse_llms_txt(&body) } else { (0, Vec::new()) };

    let robots = crate::robots::inspect_robots(&domain).unwrap_or_else(|_| crate::robots::RobotsReport {
        domain: domain.clone(),
        robots_url: String::new(),
        status_code: 0,
        has_robots: false,
        ai_bot_rules: Vec::new(),
        sitemaps: Vec::new(),
        disallow_all: false,
    });

    let mut ai_allowed = Vec::new();
    let mut ai_default = Vec::new();
    let mut ai_blocked = Vec::new();
    for rule in &robots.ai_bot_rules {
        match rule.status {
            crate::robots::BotStatus::Allowed => ai_allowed.push(rule.bot_name.clone()),
            crate::robots::BotStatus::DefaultStar => ai_default.push(rule.bot_name.clone()),
            crate::robots::BotStatus::Disallowed => ai_blocked.push(rule.bot_name.clone()),
        }
    }

    let mut score = 0u32;
    let mut checks = Vec::new();
    if present {
        score += 40;
        checks.push(check("llms.txt present", true, &format!("{} bytes, {} sections", bytes, sections.len())));
    } else {
        checks.push(check("llms.txt present", false, "No usable llms.txt at /llms.txt"));
    }
    let explicit_allowed = robots
        .ai_bot_rules
        .iter()
        .filter(|r| r.status == crate::robots::BotStatus::Allowed)
        .count();
    let bot_points = (explicit_allowed.min(3) as u32) * 10;
    score += bot_points;
    checks.push(check(
        "AI crawlers allowed",
        explicit_allowed > 0,
        &format!("{} of {} tracked bots explicitly allowed", explicit_allowed, robots.ai_bot_rules.len()),
    ));
    if !robots.sitemaps.is_empty() {
        score += 15;
        checks.push(check("Sitemap advertised", true, &format!("{} sitemap(s) in robots.txt", robots.sitemaps.len())));
    } else {
        checks.push(check("Sitemap advertised", false, "No Sitemap line in robots.txt"));
    }
    if robots.has_robots {
        score += 15;
        checks.push(check("robots.txt present", true, "Crawler rules discoverable"));
    } else {
        checks.push(check("robots.txt present", false, "No robots.txt served"));
    }

    let has_sitemap = !robots.sitemaps.is_empty();
    Ok(LlmsReport {
        domain,
        llms_url,
        info: LlmsInfo { present, status, bytes, sections },
        ai_allowed,
        ai_default,
        ai_blocked,
        robots_sitemaps: robots.sitemaps,
        robots_present: robots.has_robots,
        score: score.min(100),
        actions: llms_actions(present, explicit_allowed, robots.has_robots, has_sitemap),
        checks,
    })
}

fn llms_actions(present: bool, explicit_allowed: usize, robots_present: bool, sitemap: bool) -> Vec<crate::actions::Action> {
    let mut actions = Vec::new();
    let mut n = 1;
    let mut push = |priority: u8, effort: u8, title: String, evidence: String| {
        actions.push(crate::actions::Action::new(&format!("LLMS-{:03}", n), priority, effort, &title, evidence));
        n += 1;
    };
    if !present {
        push(1, 1, "Ship an llms.txt".into(), "answer engines cannot see the site".into());
    }
    if explicit_allowed == 0 {
        push(2, 1, "Name AI crawlers in robots.txt".into(), "all bots on default policy".into());
    }
    if !robots_present {
        push(2, 1, "Serve a robots.txt".into(), "crawler rules undiscoverable".into());
    }
    if !sitemap {
        push(3, 1, "Advertise the sitemap in robots.txt".into(), "no Sitemap line found".into());
    }
    crate::actions::rank(actions)
}
