use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsReport {
    pub domain: String,
    pub robots_url: String,
    pub status_code: u16,
    pub has_robots: bool,
    pub ai_bot_rules: Vec<AiBotRule>,
    pub sitemaps: Vec<String>,
    pub disallow_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBotRule {
    pub bot_name: String,
    pub purpose: String,
    pub status: BotStatus,
    pub rule_snippet: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotStatus {
    Allowed,
    Disallowed,
    DefaultStar,
}

pub fn inspect_robots(target: &str) -> Result<RobotsReport> {
    let base_url = if target.starts_with("http://") || target.starts_with("https://") {
        Url::parse(target)?
    } else {
        Url::parse(&format!("https://{}", target))?
    };

    let domain = base_url.host_str().unwrap_or(target).to_string();
    let robots_url = format!("{}://{}/robots.txt", base_url.scheme(), domain);

    let resp = ureq::get(&robots_url)
        .timeout(Duration::from_secs(8))
        .set("User-Agent", "jev-seo/0.1.0 (TypeSafe Jev Search Radar; +https://github.com/AkashPriyadarshii/jev-seo)")
        .call();

    match resp {
        Ok(response) => {
            let status_code = response.status();
            let body = response.into_string().unwrap_or_default();
            parse_robots_txt(&domain, &robots_url, status_code, &body)
        }
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            parse_robots_txt(&domain, &robots_url, code, &body)
        }
        Err(_) => {
            Ok(RobotsReport {
                domain,
                robots_url,
                status_code: 0,
                has_robots: false,
                ai_bot_rules: Vec::new(),
                sitemaps: Vec::new(),
                disallow_all: false,
            })
        }
    }
}

pub const TRACKED_AI_BOTS: &[(&str, &str)] = &[
    ("GPTBot", "OpenAI model training foundation data"),
    ("ChatGPT-User", "ChatGPT real-time browsing"),
    ("ClaudeBot", "Anthropic Claude model training"),
    ("anthropic-ai", "Anthropic search & web indexing"),
    ("PerplexityBot", "Perplexity generative search citation indexer"),
    ("Google-Extended", "Google Gemini & Vertex AI training data"),
    ("Bytespider", "ByteDance AI & TikTok search crawler"),
    ("CCBot", "Common Crawl open foundation training set"),
];

#[derive(Debug, Clone, Default)]
struct AgentSection {
    agents: Vec<String>,
    disallows: Vec<String>,
    allows: Vec<String>,
}

pub fn parse_robots_txt(domain: &str, robots_url: &str, status_code: u16, body: &str) -> Result<RobotsReport> {
    if status_code != 200 || body.trim().is_empty() {
        return Ok(RobotsReport {
            domain: domain.to_string(),
            robots_url: robots_url.to_string(),
            status_code,
            has_robots: false,
            ai_bot_rules: Vec::new(),
            sitemaps: Vec::new(),
            disallow_all: false,
        });
    }

    let mut sitemaps = Vec::new();
    let mut sections: Vec<AgentSection> = Vec::new();
    let mut current_section = AgentSection::default();

    for line in body.lines() {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.is_empty() {
            continue;
        }

        if let Some(pos) = clean.find(':') {
            let key = clean[..pos].trim().to_lowercase();
            let val = clean[pos + 1..].trim();

            if key == "user-agent" {
                if !current_section.disallows.is_empty() || !current_section.allows.is_empty() {
                    sections.push(current_section);
                    current_section = AgentSection::default();
                }
                current_section.agents.push(val.to_lowercase());
            } else if key == "disallow" {
                current_section.disallows.push(val.to_string());
            } else if key == "allow" {
                current_section.allows.push(val.to_string());
            } else if key == "sitemap" {
                sitemaps.push(val.to_string());
            }
        }
    }

    if !current_section.agents.is_empty() {
        sections.push(current_section);
    }

    let star_disallows: Vec<String> = sections
        .iter()
        .filter(|sec| sec.agents.iter().any(|a| a == "*"))
        .flat_map(|sec| sec.disallows.clone())
        .collect();

    let disallow_all = star_disallows.iter().any(|d| d == "/");

    let ai_bot_rules = TRACKED_AI_BOTS
        .iter()
        .map(|(bot, purpose)| evaluate_bot_rule(bot, purpose, &sections, disallow_all))
        .collect();

    Ok(RobotsReport {
        domain: domain.to_string(),
        robots_url: robots_url.to_string(),
        status_code,
        has_robots: true,
        ai_bot_rules,
        sitemaps,
        disallow_all,
    })
}

fn evaluate_bot_rule(
    bot: &str,
    purpose: &str,
    sections: &[AgentSection],
    disallow_all: bool,
) -> AiBotRule {
    let bot_lower = bot.to_lowercase();
    let explicit_rule = sections
        .iter()
        .find(|sec| sec.agents.iter().any(|a| a == &bot_lower));

    if let Some(sec) = explicit_rule {
        if sec.disallows.iter().any(|d| d == "/") {
            AiBotRule {
                bot_name: bot.to_string(),
                purpose: purpose.to_string(),
                status: BotStatus::Disallowed,
                rule_snippet: format!("User-agent: {} -> Disallow: /", bot),
            }
        } else if sec.disallows.is_empty() || (sec.disallows.len() == 1 && sec.disallows[0].is_empty()) {
            AiBotRule {
                bot_name: bot.to_string(),
                purpose: purpose.to_string(),
                status: BotStatus::Allowed,
                rule_snippet: format!("User-agent: {} -> Disallow: (none)", bot),
            }
        } else {
            let mut parts = Vec::new();
            if !sec.disallows.is_empty() {
                parts.push(format!("Disallow: [{}]", sec.disallows.join(", ")));
            }
            if !sec.allows.is_empty() {
                parts.push(format!("Allow: [{}]", sec.allows.join(", ")));
            }
            AiBotRule {
                bot_name: bot.to_string(),
                purpose: purpose.to_string(),
                status: BotStatus::Allowed,
                rule_snippet: format!("User-agent: {} -> Selective: {}", bot, parts.join(", ")),
            }
        }
    } else if disallow_all {
        AiBotRule {
            bot_name: bot.to_string(),
            purpose: purpose.to_string(),
            status: BotStatus::Disallowed,
            rule_snippet: "Inherited from: User-agent: * -> Disallow: /".to_string(),
        }
    } else {
        AiBotRule {
            bot_name: bot.to_string(),
            purpose: purpose.to_string(),
            status: BotStatus::DefaultStar,
            rule_snippet: "Inherits default allow (*) policy".to_string(),
        }
    }
}
