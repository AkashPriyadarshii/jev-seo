use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::engine::JevClient;
use crate::serp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBrief {
    pub topic: String,
    pub suggested_title: String,
    pub target_word_count: String,
    pub search_intent: String,
    pub target_audience: String,
    pub winning_angle: String,
    pub recommended_h2_outline: Vec<String>,
    pub competitor_benchmarks: Vec<CompetitorBenchmark>,
    pub geo_opening_prescription: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorBenchmark {
    pub rank: usize,
    pub title: String,
    pub url: String,
    pub snippet_summary: String,
}

pub fn generate_brief(topic: &str, limit: usize) -> Result<ContentBrief> {
    let competitors = serp::scrape_serp(topic, limit)?;

    let competitor_benchmarks: Vec<CompetitorBenchmark> = competitors
        .iter()
        .map(|c| CompetitorBenchmark {
            rank: c.position,
            title: c.title.clone(),
            url: c.url.clone(),
            snippet_summary: c.snippet.clone(),
        })
        .collect();

    let mut search_intent = "informational".to_string();
    let target_audience = "practitioner / systems engineer".to_string();
    let mut winning_angle = "empirical benchmarks & copy-pasteable implementation".to_string();

    if let Some(client) = JevClient::new() {
        let state = json!({
            "target_topic": topic,
            "top_competitors": competitor_benchmarks.iter().take(5).collect::<Vec<_>>()
        });

        match client.fanout_eval(state) {
            Ok(eval) => {
                if crate::policy::gate("brief", eval.confidence()) != crate::policy::Verdict::Drop {
                    search_intent = eval.intent;
                    if eval.content_gap != "none" {
                        winning_angle = format!("Address competitor gap: {}", eval.content_gap);
                    }
                }
            }
            Err(e) => eprintln!("Warning: Jev brief scoring failed ({e:#}), using defaults."),
        }
    }

    let clean_topic = topic.trim();
    let suggested_title = format!("{}: Complete Engineering Guide & Benchmarks", clean_topic);

    let recommended_h2_outline = vec![
        format!("1. What is {} (Direct 150-Word Definition)", clean_topic),
        "2. Why Existing Solutions Fail (Pain Points & Trade-offs)".to_string(),
        "3. Empirical Performance Benchmarks & Architecture".to_string(),
        "4. Step-by-Step Implementation & Working Code".to_string(),
        "5. Edge Cases, Failure Modes & Production Checklist".to_string(),
        "6. Comparison Matrix vs Key Alternatives".to_string(),
    ];

    let geo_opening_prescription = format!(
        "Draft the opening 134-167 words as a self-contained, fact-dense direct answer defining '{}', stating its primary utility, and giving a 1-line copy-paste quickstart. Do not use filler throat-clearers ('In today's fast-paced world...').",
        clean_topic
    );

    Ok(ContentBrief {
        topic: clean_topic.to_string(),
        suggested_title,
        target_word_count: "1,400 – 2,200 words".to_string(),
        search_intent,
        target_audience,
        winning_angle,
        recommended_h2_outline,
        competitor_benchmarks,
        geo_opening_prescription,
    })
}

impl ContentBrief {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Content Brief: {}\n\n", self.topic));
        md.push_str(&format!("- **Suggested Title**: {}\n", self.suggested_title));
        md.push_str(&format!("- **Target Length**: {}\n", self.target_word_count));
        md.push_str(&format!("- **Search Intent**: {}\n", self.search_intent));
        md.push_str(&format!("- **Audience**: {}\n", self.target_audience));
        md.push_str(&format!("- **Primary Differentiator**: {}\n\n", self.winning_angle));

        md.push_str("## GEO Opening Prescription (134-167 Words)\n\n");
        md.push_str(&format!("> {}\n\n", self.geo_opening_prescription));

        md.push_str("## Recommended Heading Outline (H2)\n\n");
        for h2 in &self.recommended_h2_outline {
            md.push_str(&format!("- **{}**\n", h2));
        }
        md.push('\n');

        md.push_str("## Top SERP Competitors Benchmarked\n\n");
        for comp in &self.competitor_benchmarks {
            md.push_str(&format!(
                "- **#{} [{}]({})**\n  _{}_\n",
                comp.rank, comp.title, comp.url, comp.snippet_summary
            ));
        }

        md
    }
}
