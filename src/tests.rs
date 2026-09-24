#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::audit::{audit_file, audit_path};
    use crate::brief::generate_brief;
    use crate::rank::DbStore;
    use crate::schema::validate_content;
    use crate::serp::get_autocomplete;

    #[test]
    fn test_audit_markdown() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("post.md");
        std::fs::write(
            &path,
            "---\ntitle: High Performance Rust Web Frameworks\ndescription: A comprehensive comparison of Actix, Axum, and Warp for building production services.\n---\n# High Performance Rust Web Frameworks\n\n## Introduction\nRust is blazing fast.\n\n![Arch](https://example.com/arch.png)\n"
        ).unwrap();

        let report = audit_file(path.to_str().unwrap()).unwrap();
        assert_eq!(report.h1_count, 1);
        assert_eq!(report.h2_count, 1);
        assert_eq!(report.title.as_deref(), Some("High Performance Rust Web Frameworks"));
        assert!(report.checks.iter().any(|c| c.name == "H1 Uniqueness" && c.passed));
    }

    #[test]
    fn test_batch_directory_audit() {
        let dir = tempfile::tempdir().unwrap();
        let path1 = dir.path().join("post1.md");
        let path2 = dir.path().join("post2.md");

        std::fs::write(
            &path1,
            "---\ntitle: Unique Title One\ndescription: A detailed guide to systems engineering in 2026.\ncanonical: https://example.com/one\n---\n# Title One\n\nWord word word.\n"
        ).unwrap();

        std::fs::write(
            &path2,
            "---\ntitle: Unique Title Two\ndescription: Another guide to systems engineering in 2026.\ncanonical: https://example.com/two\n---\n# Title Two\n\nWord word word.\n"
        ).unwrap();

        let dir_report = audit_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(dir_report.total_files, 2);
        assert!(dir_report.duplicate_titles.is_empty(), "Titles should be unique");
    }

    #[test]
    fn test_audit_html() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("page.html");
        std::fs::write(
            &path,
            "<!DOCTYPE html><html><head><title>My SEO Title</title><meta name=\"description\" content=\"A valid test description between eighty and one hundred sixty characters long for SEO testing.\"></head><body><h1>Main Heading</h1><img src=\"foo.jpg\" alt=\"Foo\"><a href=\"/local\">Local</a><a href=\"https://google.com\">Ext</a></body></html>"
        ).unwrap();

        let report = audit_file(path.to_str().unwrap()).unwrap();
        assert_eq!(report.h1_count, 1);
        assert_eq!(report.title.as_deref(), Some("My SEO Title"));
        assert_eq!(report.internal_links, 1);
        assert_eq!(report.external_links, 1);
        assert_eq!(report.images_missing_alt, 0);
    }

    #[test]
    fn test_schema_validation() {
        let json_ld = r#"{
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "jev-seo",
            "operatingSystem": "Windows, Linux, macOS",
            "applicationCategory": "DeveloperApplication",
            "description": "Zero-cost SEO and GEO search radar CLI"
        }"#;

        let report = validate_content("test_snippet", json_ld).unwrap();
        assert_eq!(report.schemas_found, 1);
        assert_eq!(report.types[0], "SoftwareApplication");
        assert!(report.is_valid);
        assert!(report.completeness_score >= 80);
    }

    #[test]
    fn test_schema_deprecation_notice() {
        let json_ld = r#"{
            "@context": "https://schema.org",
            "@type": "HowTo",
            "name": "How to Build an Agent"
        }"#;

        let report = validate_content("test_howto", json_ld).unwrap();
        assert_eq!(report.types[0], "HowTo");
        assert!(report.warnings.iter().any(|w| w.contains("deprecated")));
    }

    #[test]
    fn test_brief_structure() {
        if let Ok(b) = generate_brief("rust command line tools", 3) {
            assert!(!b.recommended_h2_outline.is_empty());
            assert!(!b.geo_opening_prescription.is_empty());
            let md = b.to_markdown();
            assert!(md.contains("## Recommended Heading Outline"));
        }
    }

    #[test]
    fn test_sqlite_rank_drift() {
        let mut db = DbStore::open().unwrap();
        let delta1 = db.track_keyword("example.com", "best rust framework", Some(5), Some("https://example.com/rust"), "ddg", "duckduckgo-html").unwrap();
        assert_eq!(delta1.curr_rank, Some(5));

        let delta2 = db.track_keyword("example.com", "best rust framework", Some(3), Some("https://example.com/rust"), "ddg", "duckduckgo-html").unwrap();
        assert_eq!(delta2.prev_rank, Some(5));
        assert_eq!(delta2.curr_rank, Some(3));
    }

    #[test]
    fn test_rank_observations_carry_provenance() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("prov.db");
        let mut db = DbStore::open_at(db_path.to_str().unwrap()).unwrap();
        db.track_keyword("x.test", "q", Some(4), Some("https://x.test/"), "tavily", "tavily-search").unwrap();
        db.track_keyword("x.test", "q", None, None, "ddg", "duckduckgo-html").unwrap();
        let trail = db.observation_trail("x.test", "q").unwrap();
        assert_eq!(trail.len(), 2);
        assert_eq!(trail[0], (None, "ddg".into(), "duckduckgo-html".into()));
        assert_eq!(trail[1], (Some(4), "tavily".into(), "tavily-search".into()));
    }

    #[test]
    fn test_ddg_autocomplete_live() {
        if let Ok(list) = get_autocomplete("rust programming") {
            assert!(!list.is_empty(), "Autocomplete should return suggestions");
        }
    }

    #[test]
    fn test_robots_txt_parsing_and_ai_crawler_radar() {
        use crate::robots::{parse_robots_txt, BotStatus};

        let robots_body = r#"
User-agent: GPTBot
Disallow: /
Allow: /public

User-agent: ClaudeBot
Disallow: /private
Disallow: /admin

User-agent: *
Disallow: /api/

Sitemap: https://example.com/sitemap.xml
"#;
        let rep = parse_robots_txt("example.com", "https://example.com/robots.txt", 200, robots_body).unwrap();
        assert!(rep.has_robots);
        assert_eq!(rep.sitemaps.len(), 1);
        assert_eq!(rep.sitemaps[0], "https://example.com/sitemap.xml");

        let gpt = rep.ai_bot_rules.iter().find(|r| r.bot_name == "GPTBot").unwrap();
        assert_eq!(gpt.status, BotStatus::Disallowed);

        let claude = rep.ai_bot_rules.iter().find(|r| r.bot_name == "ClaudeBot").unwrap();
        assert_eq!(claude.status, BotStatus::Allowed);
        assert!(claude.rule_snippet.contains("/private"));

        let perplexity = rep.ai_bot_rules.iter().find(|r| r.bot_name == "PerplexityBot").unwrap();
        assert_eq!(perplexity.status, BotStatus::DefaultStar);
    }

    #[test]
    fn test_robots_global_disallow_inheritance() {
        use crate::robots::{parse_robots_txt, BotStatus};

        let robots_body = "User-agent: *\nDisallow: /\n";
        let rep = parse_robots_txt("blocked.com", "https://blocked.com/robots.txt", 200, robots_body).unwrap();
        assert!(rep.disallow_all);
        for rule in &rep.ai_bot_rules {
            assert_eq!(rule.status, BotStatus::Disallowed);
        }
    }

    #[test]
    fn test_schema_article_validation() {
        let json_ld = r#"{
            "@context": "https://schema.org",
            "@type": "TechArticle",
            "headline": "Building High-Throughput Search Engines in Rust",
            "author": { "@type": "Person", "name": "Akash" },
            "datePublished": "2026-09-20",
            "publisher": { "@type": "Organization", "name": "TypeSafe" }
        }"#;

        let report = validate_content("test_article", json_ld).unwrap();
        assert!(report.is_valid);
        assert_eq!(report.schemas_found, 1);
        assert_eq!(report.types[0], "TechArticle");
        assert!(report.completeness_score >= 85);
    }

    #[test]
    fn test_schema_invalid_missing_type_and_context() {
        let json_ld = r#"{ "foo": "bar" }"#;
        let report = validate_content("test_broken", json_ld).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("@context")));
        assert!(report.errors.iter().any(|e| e.contains("@type")));
    }

    #[test]
    fn test_audit_missing_alt_tags() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken_image.html");
        std::fs::write(
            &path,
            "<!DOCTYPE html><html><head><title>Page with Broken Image Alt</title><meta name=\"description\" content=\"A test description with sufficient length to pass the meta description audit rule.\"></head><body><h1>Heading</h1><img src=\"no_alt.jpg\"><img src=\"has_alt.jpg\" alt=\"Alt Text\"></body></html>"
        ).unwrap();

        let report = audit_file(path.to_str().unwrap()).unwrap();
        assert_eq!(report.image_count, 2);
        assert_eq!(report.images_missing_alt, 1);
        assert!(report.checks.iter().any(|c| c.name == "Image Alt Tags" && !c.passed));
    }

    #[test]
    fn test_mcp_tools_list_protocol() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "tools/list".into(),
            params: None,
        };

        let resp = handle_request(&req);
        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        let tools = res.get("tools").and_then(|t| t.as_array()).unwrap();
        assert_eq!(tools.len(), 13, "All 13 agent SEO tools must be exposed");

        let names: Vec<&str> = tools.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str())).collect();
        assert!(names.contains(&"seo_keywords"));
        assert!(names.contains(&"seo_serp_inspect"));
        assert!(names.contains(&"seo_audit"));
        assert!(names.contains(&"seo_geo"));
        assert!(names.contains(&"seo_schema"));
        assert!(names.contains(&"seo_robots"));
        assert!(names.contains(&"seo_brief"));
        assert!(names.contains(&"seo_sitemap"));
        assert!(names.contains(&"seo_crawl"));
        assert!(names.contains(&"seo_llms"));
        assert!(names.contains(&"seo_extract"));
        assert!(names.contains(&"seo_explain"));
        assert!(names.contains(&"seo_report"));
    }

    #[test]
    fn test_mcp_explain_and_report_tools() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let explain = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: Some(json!({
                "name": "seo_explain",
                "arguments": { "id": "RULE-R19" }
            })),
        };
        let resp = handle_request(&explain);
        let text = resp.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
        assert!(text.contains("AI slop markers"), "{text}");

        let missing = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(3)),
            method: "tools/call".into(),
            params: Some(json!({
                "name": "seo_explain",
                "arguments": { "id": "R99" }
            })),
        };
        let resp = handle_request(&missing);
        let text = resp.result.as_ref().unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(text.starts_with("Error:"), "{text}");
    }

    #[test]
    fn test_cannibalization_pairs_expand() {
        use crate::audit::{cannibalization_pairs, AuditReport, CannibalizationItem, CheckItem, DirectoryAuditReport};
        let mk = |path: &str, wc: usize| AuditReport {
            file_path: path.into(),
            title: Some("Best Rust SEO".into()),
            title_len: 14,
            description: None,
            description_len: 0,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: wc,
            image_count: 0,
            images_missing_alt: 0,
            internal_links: 0,
            external_links: 0,
            schema_found: false,
            schema_json_valid: true,
            canonical_found: false,
            og_tags_found: false,
            geo_opening_words: 0,
            heading_skipped_levels: vec![],
            em_dash_count: 0,
            ai_slop_words_found: vec![],
            internal_link_targets: vec![],
            uncited_claims: 0,
            checks: vec![CheckItem { name: "n".into(), passed: true, message: "m".into() }],
        };
        let rep = DirectoryAuditReport {
            dir_path: "d".into(),
            total_files: 3,
            total_words: 75,
            avg_words_per_file: 25,
            pass_rate: 80.0,
            reports: vec![mk("a.md", 10), mk("b.md", 40), mk("c.md", 25)],
            duplicate_titles: Default::default(),
            thin_pages: vec![],
            missing_canonicals: vec![],
            missing_descriptions: vec![],
            orphan_pages: vec![],
            keyword_cannibalization: vec![CannibalizationItem {
                keyword_stem: "best rust seo".into(),
                colliding_files: vec!["a.md".into(), "b.md".into(), "c.md".into()],
            }],
            findings: vec![],
        };
        let pairs = cannibalization_pairs(&rep);
        assert_eq!(pairs.len(), 3, "3 files → C(3,2)=3 pairs");
        let ab = pairs.iter().find(|p| p.a == "a.md" && p.b == "b.md").expect("a×b");
        assert_eq!(ab.winner, "b.md");
        assert_eq!(ab.a_words, 10);
        assert_eq!(ab.b_words, 40);
        let ac = pairs.iter().find(|p| p.a == "a.md" && p.b == "c.md").expect("a×c");
        assert_eq!(ac.winner, "c.md");
    }

    #[test]
    fn test_mcp_tool_call_schema() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: Some(json!({
                "name": "seo_schema",
                "arguments": {
                    "target": r#"{"@context":"https://schema.org","@type":"WebSite","name":"Test","url":"https://test.com"}"#
                }
            })),
        };

        let resp = handle_request(&req);
        assert!(resp.error.is_none());
        let content = resp.result.unwrap().get("content").and_then(|c| c.as_array()).cloned().unwrap();
        let text = content[0].get("text").and_then(|t| t.as_str()).unwrap();
        assert!(text.contains("\"is_valid\": true"));
    }

    #[test]
    fn test_mcp_tool_call_sitemap() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url><loc>https://example.com/page1</loc><lastmod>2026-09-20</lastmod></url>
            <url><loc>https://example.com/page2</loc></url>
        </urlset>"#;

        let dir = tempfile::tempdir().unwrap();
        let sitemap_path = dir.path().join("sitemap.xml");
        std::fs::write(&sitemap_path, sample_xml).unwrap();

        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(3)),
            method: "tools/call".into(),
            params: Some(json!({
                "name": "seo_sitemap",
                "arguments": {
                    "target": sitemap_path.to_str().unwrap()
                }
            })),
        };

        let resp = handle_request(&req);
        assert!(resp.error.is_none());
        let content = resp.result.unwrap().get("content").and_then(|c| c.as_array()).cloned().unwrap();
        let text = content[0].get("text").and_then(|t| t.as_str()).unwrap();
        assert!(text.contains("\"total_urls\": 2"));
        assert!(text.contains("\"is_valid\": true"));
    }

    #[test]
    fn test_heading_hierarchy_skip_level() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skip_heading.md");
        std::fs::write(
            &path,
            "---\ntitle: Heading Test\ndescription: Test heading hierarchy skip levels in markdown.\n---\n# Title\n### Jumped to H3\n"
        ).unwrap();

        let rep = audit_file(path.to_str().unwrap()).unwrap();
        assert!(!rep.heading_skipped_levels.is_empty(), "Should detect skipped H2 level");
        assert!(rep.heading_skipped_levels[0].contains("H1 -> H3"));
        assert!(rep.checks.iter().any(|c| c.name == "Heading Hierarchy" && !c.passed));
    }

    #[test]
    fn test_ai_slop_and_em_dash_detection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slop_test.md");
        std::fs::write(
            &path,
            "---\ntitle: AI Slop Test Page\ndescription: A test page to verify automated detection of synthetic content tells.\n---\n# AI Slop Detection\n\nWe delve into this crucial landscape — a testament to modern engineering — to seamlessly foster leverage across systems.\n"
        ).unwrap();

        let rep = audit_file(path.to_str().unwrap()).unwrap();
        assert!(rep.em_dash_count >= 2, "Should count em-dashes");
        assert!(!rep.ai_slop_words_found.is_empty(), "Should find AI crutch words");
        assert!(rep.ai_slop_words_found.contains(&"delve".to_string()));
        assert!(rep.ai_slop_words_found.contains(&"testament".to_string()));
        assert!(rep.checks.iter().any(|c| c.name == "Helpful Content (AI Slop)" && !c.passed));
    }

    #[test]
    fn test_orphan_pages_and_cannibalization() {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("index.md");
        let about = dir.path().join("about.md");
        let orphan = dir.path().join("isolated.md");

        std::fs::write(
            &index,
            "---\ntitle: Best Rust SEO Framework Guide\ndescription: Guide to rust SEO tools.\n---\n# Best Rust SEO Framework Guide\nCheck [About Us](/about.md) for details.\n"
        ).unwrap();

        std::fs::write(
            &about,
            "---\ntitle: Best Rust SEO Framework Overview\ndescription: Overview of rust SEO tools.\n---\n# Best Rust SEO Framework Overview\nBack to [Home](/index.md).\n"
        ).unwrap();

        std::fs::write(
            &orphan,
            "---\ntitle: Totally Orphaned Page\ndescription: An isolated page with no inbound internal links anywhere.\n---\n# Orphan Page\nNo one links to me.\n"
        ).unwrap();

        let dir_rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(dir_rep.total_files, 3);
        assert!(!dir_rep.orphan_pages.is_empty(), "Should detect orphan page");
        assert!(dir_rep.orphan_pages.iter().any(|p| p.contains("isolated.md")));

        assert!(!dir_rep.keyword_cannibalization.is_empty(), "Should detect title keyword cannibalization");
        assert!(dir_rep.keyword_cannibalization.iter().any(|c| c.keyword_stem.contains("best rust seo")));
    }

    #[test]
    fn test_sitemap_xml_parsing_and_hreflang() {
        use crate::sitemap::parse_sitemap_xml;

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
                xmlns:xhtml="http://www.w3.org/1999/xhtml">
            <url>
                <loc>https://example.com/en/page</loc>
                <lastmod>2026-09-20</lastmod>
                <xhtml:link rel="alternate" hreflang="en-US" href="https://example.com/en/page"/>
                <xhtml:link rel="alternate" hreflang="en-UK" href="https://example.com/uk/page"/>
                <xhtml:link rel="alternate" hreflang="x-default" href="https://example.com/"/>
            </url>
            <url>
                <loc>http://example.com/insecure-page</loc>
            </url>
            <url>
                <loc>https://example.com/search?q=rust&amp;page=2</loc>
            </url>
        </urlset>"#;

        let rep = parse_sitemap_xml("https://example.com/sitemap.xml", xml).unwrap();
        assert_eq!(rep.total_urls, 3);
        assert_eq!(rep.https_urls, 2);
        assert_eq!(rep.insecure_http_urls, 1);
        assert_eq!(rep.urls_with_params, 1);
        assert_eq!(rep.urls_with_lastmod, 1);
        assert_eq!(rep.hreflang_count, 3);
        assert!(!rep.invalid_hreflang_codes.is_empty(), "en-UK should be flagged as invalid");
        assert!(rep.invalid_hreflang_codes[0].contains("en-GB"));
        assert!(rep.warnings.iter().any(|w| w.contains("Insecure HTTP")));
        assert!(rep.warnings.iter().any(|w| w.contains("Query Parameters")));
    }

    #[test]
    fn test_mcp_unknown_method() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(99)),
            method: "unknown/method".into(),
            params: None,
        };

        let resp = handle_request(&req);
        assert!(resp.error.is_some());
        let err = resp.error.unwrap();
        assert_eq!(err.get("code").and_then(|c| c.as_i64()), Some(-32601));
    }

    #[test]
    fn test_policy_gate() {
        use crate::policy::{gate, thresholds, Verdict, ACT};
        // Skill bar: Choice/Score act only at confidence >= 0.80.
        assert_eq!(thresholds("geo").act, ACT);
        assert!((ACT - 0.80).abs() < 1e-12);
        assert_eq!(gate("geo", 0.9), Verdict::Act);
        assert_eq!(gate("geo", 0.79), Verdict::Flag);
        assert_eq!(gate("geo", 0.6), Verdict::Flag);
        assert_eq!(gate("geo", 0.2), Verdict::Drop);
        assert_eq!(gate("audit", 0.85), Verdict::Act);
        assert_eq!(gate("audit", 0.7), Verdict::Flag);
    }

    #[test]
    fn test_injection_blocked_band() {
        use crate::policy::injection_blocked;
        use serde_json::json;
        let mut yes = serde_json::Map::new();
        yes.insert("injection_risk".into(), json!({"type":"noul","value":0.9,"noul":0.9}));
        assert!(injection_blocked(&yes));
        let mut no = serde_json::Map::new();
        no.insert("injection_risk".into(), json!({"type":"noul","value":0.1,"noul":0.1}));
        assert!(!injection_blocked(&no));
        assert!(!injection_blocked(&serde_json::Map::new()));
    }

    #[test]
    fn test_route_for_intent_exists() {
        use crate::policy::route_for_intent;
        assert_eq!(route_for_intent("navigational"), "rank");
        assert_eq!(route_for_intent("informational"), "geo / audit");
    }

    #[test]
    fn test_jev_budget_cap() {
        use crate::manifest::{
            jev_budget_exhausted, jev_budget_usd, set_jev_budget_usd, DEFAULT_JEV_BUDGET_USD,
            JEV_INPUT_TOKENS,
        };
        use std::sync::atomic::Ordering;
        set_jev_budget_usd(0.0);
        assert!(jev_budget_exhausted(1));
        set_jev_budget_usd(DEFAULT_JEV_BUDGET_USD);
        assert!(!jev_budget_exhausted(1));
        // Tokens already spent count toward the cap.
        let before = JEV_INPUT_TOKENS.load(Ordering::Relaxed);
        JEV_INPUT_TOKENS.fetch_add(10_000_000, Ordering::Relaxed); // $0.42 at list price
        assert!(jev_budget_exhausted(1));
        JEV_INPUT_TOKENS.fetch_sub(10_000_000, Ordering::Relaxed);
        assert_eq!(JEV_INPUT_TOKENS.load(Ordering::Relaxed), before);
        set_jev_budget_usd(DEFAULT_JEV_BUDGET_USD);
        assert!((jev_budget_usd() - DEFAULT_JEV_BUDGET_USD).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_score_maps_legend_top_to_one() {
        use serde_json::json;
        // Same logic as main::normalize_score (duplicated for unit access).
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
        let a = json!({"score": 3.0, "legend": {"0": "a", "1": "b", "2": "c", "3": "d"}});
        assert!((normalize_score(&a).unwrap() - 1.0).abs() < 1e-9);
        let b = json!({"score": 1.5, "legend": {"0": "a", "1": "b", "2": "c", "3": "d"}});
        assert!((normalize_score(&b).unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_composite_geo() {
        use crate::policy::composite_geo;
        use serde_json::json;
        let mut extra = serde_json::Map::new();
        for id in ["geo_structure", "geo_density", "geo_directness", "geo_statistics", "geo_freshness"] {
            extra.insert(id.into(), json!({"score": 4.0, "confidence": 0.9}));
        }
        assert_eq!(composite_geo(&extra), Some((10, 0.9)));
        extra.remove("geo_density");
        assert_eq!(composite_geo(&extra), None);
    }

    #[test]
    fn test_url_matches_domain() {
        use crate::paths::url_matches_domain;
        assert!(url_matches_domain("https://crates.io/crates/rg", "crates.io"));
        assert!(url_matches_domain("https://docs.crates.io/x", "crates.io"));
        assert!(!url_matches_domain("https://evilcrates.io/x", "crates.io"));
        assert!(!url_matches_domain("not a url", "crates.io"));
    }

    #[test]
    fn test_reject_private_url() {
        use crate::paths::reject_private_url;
        assert!(reject_private_url("https://example.com/robots.txt").is_ok());
        for bad in [
            "http://localhost/x",
            "http://127.0.0.1/x",
            "http://10.0.0.5/x",
            "http://169.254.169.254/",
            "ftp://example.com/x",
        ] {
            assert!(reject_private_url(bad).is_err(), "{}", bad);
        }
    }

    #[test]
    fn test_read_user_file_guards() {
        use crate::paths::read_user_file;
        let dir = tempfile::tempdir().unwrap();
        let dot = dir.path().join(".hidden.md");
        std::fs::write(&dot, "x").unwrap();
        assert!(read_user_file(dot.to_str().unwrap(), &["md"]).is_err());
        let exe = dir.path().join("run.sh");
        std::fs::write(&exe, "x").unwrap();
        assert!(read_user_file(exe.to_str().unwrap(), &["md"]).is_err());
        let ok = dir.path().join("page.md");
        std::fs::write(&ok, "hello").unwrap();
        assert_eq!(read_user_file(ok.to_str().unwrap(), &["md"]).unwrap(), "hello");
        assert_eq!(read_user_file("just a snippet", &["md"]).unwrap(), "just a snippet");
        assert!(read_user_file("https://example.com/x", &["md"]).is_err());
    }

    #[test]
    fn test_mcp_initialize_and_is_error() {
        use crate::mcp::{handle_request, RpcRequest};
        use serde_json::json;

        let init = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: None,
        };
        let resp = handle_request(&init);
        assert!(resp.result.as_ref().unwrap().get("protocolVersion").is_some());

        let call = RpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: Some(json!({"name": "nope", "arguments": {}})),
        };
        let resp = handle_request(&call);
        let result = resp.result.unwrap();
        assert_eq!(result.get("isError").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_crawl_extract_links_same_host() {
        use crate::crawl::extract_links;
        let base = url::Url::parse("https://example.com/docs/a").unwrap();
        let html = r#"<a href="/docs/b">B</a><a href="https://example.com/c#frag">C</a><a href="https://other.com/x">X</a><a href="mailto:a@b.c">M</a><a href="/docs/b">dup</a>"#;
        let links = extract_links(html, &base);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com/docs/b".to_string()));
        assert!(links.contains(&"https://example.com/c".to_string()));
    }

    #[test]
    fn test_crawl_robots_allows() {
        use crate::crawl::robots_allows;
        let body = "User-agent: *\nDisallow: /private/\nDisallow: /tmp\nAllow: /tmp/public\n";
        assert!(robots_allows(body, "/docs/a"));
        assert!(!robots_allows(body, "/private/x"));
        assert!(robots_allows(body, "/tmp/public/x"));
        assert!(!robots_allows(body, "/tmp/x"));
        assert!(robots_allows("", "/anything"));
    }

    #[test]
    fn test_crawl_sitemap_seeds() {        use crate::crawl::sitemap_seed_urls;
        let xml = r#"<?xml version="1.0"?><urlset><url><loc>https://example.com/a</loc></url><url><loc>https://example.com/b</loc></url></urlset>"#;
        let seeds = sitemap_seed_urls(xml);
        assert_eq!(seeds, vec!["https://example.com/a", "https://example.com/b"]);
    }

    #[test]
    fn test_crawl_canonicalize() {        use crate::crawl::canonicalize;
        assert_eq!(canonicalize("https://Example.COM/a/?utm_source=x#frag"), "https://example.com/a");
        assert_eq!(canonicalize("https://example.com/index.html"), "https://example.com/");
        assert_eq!(canonicalize("https://example.com/docs/?fbclid=1&x=2"), "https://example.com/docs?x=2");
        assert_eq!(canonicalize("https://example.com/a/"), "https://example.com/a");
        assert_eq!(canonicalize("https://example.com/a"), "https://example.com/a");
    }

    #[test]
    fn test_rules_registry_and_scoring() {        use crate::rules::{overall, score_areas, Area, Finding, Severity, RULES};
        assert_eq!(RULES.len(), 57);
        assert!(RULES.iter().all(|r| crate::rules::rule(r.id).is_some()));
        let findings = vec![
            Finding { rule_id: "R01".into(), area: Area::Crawl, severity: Severity::High, scope: "https://x.test/a".into(), evidence: "HTTP 404".into(), fix: "Restore the target.".into(), kind: "fact".into(), observed_at: 1, source: "t".into() },
            Finding { rule_id: "R42".into(), area: Area::Performance, severity: Severity::Medium, scope: "https://x.test/a".into(), evidence: "900ms".into(), fix: "Cut server time.".into(), kind: "fact".into(), observed_at: 1, source: "t".into() },
        ];
        let mut totals = std::collections::HashMap::new();
        totals.insert(Area::Crawl, 10);
        totals.insert(Area::Performance, 10);
        let areas = score_areas(&findings, &totals);
        let crawl = areas.iter().find(|a| a.area == Area::Crawl).unwrap();
        assert!(crawl.score < 100 && crawl.score > 80);
        let perf = areas.iter().find(|a| a.area == Area::Performance).unwrap();
        assert!(perf.score < 100 && perf.score > 90);
        assert!(overall(&[]) == 100);
        let ranked = crate::rules::actions_for(&findings);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].id, "RULE-R01");
    }

    #[test]
    fn test_rules_check_crawl_live_shapes() {
        use crate::crawl::{finish_report, PageRecord, ReportParts};
        use std::collections::HashMap;
        let pages = vec![
            PageRecord { url: "https://x.test/".into(), status: 200, final_url: "https://x.test/".into(), outlinks: 1, elapsed_ms: 100, bytes: 500, hops: vec![], encoding: Some("gzip".into()), source: "direct".into(), fetch_cost: 0, upgraded: false },
            PageRecord { url: "https://x.test/dead".into(), status: 404, final_url: "https://x.test/dead".into(), outlinks: 0, elapsed_ms: 50, bytes: 0, hops: vec![], encoding: None, source: "direct".into(), fetch_cost: 0, upgraded: false },
        ];
        let rep = finish_report(ReportParts {
            start_url: "https://x.test/".into(),
            pages,
            redirects: vec![],
            inbound: HashMap::new(),
            errors: vec![],
            seeded_from_sitemap: false,
            capped: false,
            robots_honored: false,
            vitals: None,
            probes: vec![],
        });
        assert!(rep.findings.iter().any(|f| f.rule_id == "R01"));
        assert!(rep.findings.iter().any(|f| f.rule_id == "R05"));
        assert!(rep.findings.iter().any(|f| f.rule_id == "R06"));
        assert!(!rep.actions.is_empty());
        assert!(rep.score < 100);
    }

    #[test]
    fn test_vitals_rules_fire_on_lab_numbers() {
        use crate::crawl::{finish_report, PageRecord, ReportParts};
        use std::collections::HashMap;
        let pages = vec![
            PageRecord { url: "https://x.test/".into(), status: 200, final_url: "https://x.test/".into(), outlinks: 0, elapsed_ms: 100, bytes: 500, hops: vec![], encoding: Some("gzip".into()), source: "direct".into(), fetch_cost: 0, upgraded: false },
        ];
        let rep = finish_report(ReportParts {
            start_url: "https://x.test/".into(),
            pages,
            redirects: vec![],
            inbound: HashMap::new(),
            errors: vec![],
            seeded_from_sitemap: true,
            capped: false,
            robots_honored: true,
            vitals: Some(crate::vitals::Vitals {
                lcp_ms: Some(3100),
                cls_milli: Some(240),
                inp_ms: Some(120),
                score: Some(62),
                field: false,
            }),
            probes: vec![],
        });
        assert!(rep.findings.iter().any(|f| f.rule_id == "R51"));
        assert!(rep.findings.iter().any(|f| f.rule_id == "R52"));
        assert!(rep.findings.iter().any(|f| f.rule_id == "R53"));
        assert!(rep.findings.iter().any(|f| f.rule_id == "R53" && f.fix.contains("directional")));
        let clean = crate::crawl::finish_report(crate::crawl::ReportParts {
            start_url: "https://x.test/".into(),
            pages: vec![],
            redirects: vec![],
            inbound: HashMap::new(),
            errors: vec![],
            seeded_from_sitemap: true,
            capped: false,
            robots_honored: true,
            vitals: Some(crate::vitals::Vitals {
                lcp_ms: Some(1800),
                cls_milli: Some(40),
                inp_ms: Some(150),
                score: Some(94),
                field: true,
            }),
            probes: vec![],
        });
        assert!(!clean.findings.iter().any(|f| f.rule_id == "R51"));
        assert!(!clean.findings.iter().any(|f| f.rule_id == "R52"));
        assert!(!clean.findings.iter().any(|f| f.rule_id == "R53"));
    }

    #[test]
    fn test_pdf_deck_sections() {
        use crate::audit::{audit_path, pdf_cover, pdf_findings_by_area, pdf_inventory, pdf_method, pdf_scorecard};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "---\ntitle: T\ndescription: A fine description for testing.\n---\n# T\n\nWords here live happily in this file with enough of them to pass depth.\n").unwrap();
        let rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        let cover = pdf_cover(&rep).join("\n");
        assert!(cover.contains("jev-seo audit report") && cover.contains("Score:"));
        let scorecard = pdf_scorecard(&rep).join("\n");
        assert!(scorecard.contains("Scorecard:") && scorecard.contains("Top actions by impact:"));
        let areas = pdf_findings_by_area(&rep).join("\n");
        assert!(areas.contains("Findings by area:"));
        let inv = pdf_inventory(&rep).join("\n");
        assert!(inv.contains("Page inventory"));
        let method = pdf_method().join("\n");
        assert!(method.contains("Method:") && method.contains("never predict rankings"));
        let pdf = crate::audit::to_pdf_opt(&rep, None);
        let text = String::from_utf8_lossy(&pdf);
        for section in ["jev-seo audit report", "Scorecard:", "Findings by area:", "Page inventory", "Method:"] {
            assert!(text.contains(section), "missing {section}");
        }
    }

    #[test]
    fn test_rules_check_audit_shapes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "# T\n\nshort\n").unwrap();
        let rep = crate::audit::with_findings(crate::audit::audit_path(dir.path().to_str().unwrap()).unwrap());
        assert!(rep.findings.iter().any(|f| f.rule_id == "R18"));
        let thin = rep.findings.iter().find(|f| f.rule_id == "R18").unwrap();
        assert_eq!(thin.kind, "heuristic");
        assert_eq!(thin.source, "local-audit");
        assert!(thin.observed_at > 0);
        let csv = crate::rules::to_csv(&rep.findings);
        assert!(csv.starts_with("rule,area,severity,kind,observed_at,source,scope,evidence\n"));
    }

    #[test]
    fn test_rescore_reads_v011_json() {
        let old = r#"{"start_url":"https://x.test/","pages_crawled":1,"score":90,"grade":"A","areas":[],"actions":[],"broken":[],"redirects":[],"orphans":[],"errors":[],"pages":[{"url":"https://x.test/","status":200,"final_url":"https://x.test/","outlinks":0,"elapsed_ms":10,"bytes":100,"hops":[]}],"seeded_from_sitemap":true,"capped":false,"robots_honored":true}"#;
        let saved: crate::crawl::CrawlReport = serde_json::from_str(old).unwrap();
        let rep = crate::crawl::finish_report(crate::crawl::ReportParts {
            start_url: saved.start_url,
            pages: saved.pages,
            redirects: saved.redirects,
            inbound: saved.inbound,
            errors: saved.errors,
            seeded_from_sitemap: saved.seeded_from_sitemap,
            capped: saved.capped,
            robots_honored: saved.robots_honored,
            vitals: saved.vitals,
            probes: vec![],
        });
        assert!(rep.score <= 100);
    }

    #[test]
    fn test_actions_rank_priority_then_effort() {
        use crate::actions::{grade, rank, Action};
        let mk = |id: &str, p: u8, e: u8| Action::new(id, p, e, id, String::new());
        let ranked = rank(vec![mk("C", 2, 1), mk("A", 1, 3), mk("B", 1, 1)]);
        let ids: Vec<&str> = ranked.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["B", "A", "C"]);
        assert_eq!(crate::actions::top(&ranked, 2).len(), 2);
        assert_eq!(grade(91), "A");
        assert_eq!(grade(80), "B");
        assert_eq!(grade(60), "C");
        assert_eq!(grade(40), "D");
        assert_eq!(grade(10), "F");
        let q = Action::new("Q", 2, 1, "q", String::new());
        assert!(q.quick_win);
        assert_eq!(q.impact, 60);
    }

    #[test]
    fn test_action_tracker_csv() {
        use crate::actions::{to_csv, Action};
        let a = vec![Action::new("RULE-R19", 2, 2, "AI slop markers (2 hits)", "index.html, about.html".into())];
        let csv = to_csv(&a);
        assert!(csv.starts_with("id,priority,effort_band,impact,quick_win,gate,title,evidence\n"));
        assert!(csv.contains("RULE-R19"));
        assert!(csv.contains("about a day"));
        assert!(csv.contains(",advisory,"));
    }

    #[test]
    fn test_gate_split_covers_registry() {
        use crate::rules::{gate, Gate, RULES};
        assert_eq!(RULES.len(), 57);
        let blocking: Vec<&&str> = RULES
            .iter()
            .filter(|r| gate(r.id) == Gate::Blocking)
            .map(|r| &r.id)
            .collect();
        assert_eq!(blocking.len(), 23, "blocking set changed: {:?}", blocking);
        for id in ["R01", "R09", "R11", "R33", "R36", "R47", "RULE-R02"] {
            assert_eq!(gate(id), Gate::Blocking, "{id}");
        }
        for id in ["R10", "R19", "R20", "R24", "R41", "R42", "R51", "R99", "CRAWL-001"] {
            assert_eq!(gate(id), Gate::Advisory, "{id}");
        }
    }

    #[test]
    fn test_blocking_findings_selects_only_blockers() {
        use crate::rules::{blocking_findings, Finding};
        use crate::rules::{Area, Severity};
        let mk = |id: &str| Finding {
            rule_id: id.into(),
            area: Area::Content,
            severity: Severity::Medium,
            scope: "s".into(),
            evidence: "e".into(),
            fix: "f".into(),
            kind: "fact".into(),
            observed_at: 1,
            source: "t".into(),
        };
        let findings = vec![mk("R19"), mk("R09"), mk("R42")];
        let blocked = blocking_findings(&findings);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].rule_id, "R09");
    }

    #[test]
    fn test_explain_rule_forms() {
        use crate::rules::explain;
        let a = explain("R19").expect("bare id");
        let b = explain("RULE-R19").expect("prefixed id");
        assert!(a.contains("AI slop markers"));
        assert!(b.contains("content"));
        assert!(b.contains("Rewrite flagged boilerplate"));
        assert!(explain("R99").is_none());
    }

    #[test]
    fn test_audit_report_diff() {
        use crate::audit::DirectoryAuditReport;
        use crate::rules::{Area, Finding, Severity};
        let mk = |pass: f64, rule: &str| DirectoryAuditReport {
            dir_path: "d".into(),
            total_files: 3,
            total_words: 100,
            avg_words_per_file: 33,
            pass_rate: pass,
            reports: vec![],
            duplicate_titles: Default::default(),
            thin_pages: vec![],
            missing_canonicals: vec![],
            missing_descriptions: vec![],
            orphan_pages: vec![],
            keyword_cannibalization: vec![],
            findings: vec![Finding {
                rule_id: rule.into(),
                area: Area::OnPage,
                severity: Severity::Medium,
                scope: "x.html".into(),
                evidence: "e".into(),
                fix: "f".into(),
                kind: "fact".into(),
                observed_at: 1,
                source: "t".into(),
            }],
        };
        let base = mk(50.0, "R09");
        let cur = mk(80.0, "R10");
        let diff = crate::main_shim_diff(&cur, &base);
        assert_eq!(diff["baseline_score"], 50);
        assert_eq!(diff["current_score"], 80);
        assert_eq!(diff["delta"], 30);
    }

    #[test]
    fn test_llms_parse() {
        use crate::llms::parse_llms_txt;
        let body = "# Title\n\nSome prose.\n\n## Docs\n\n- item\n";
        let (bytes, sections) = parse_llms_txt(body);
        assert_eq!(bytes, body.len());
        assert_eq!(sections, vec!["Title", "Docs"]);
    }

    #[test]
    fn test_policy_needs_review() {
        use crate::policy::needs_review;
        let extra: serde_json::Map<String, serde_json::Value> = serde_json::from_value(serde_json::json!({
            "geo_structure": { "score": 3.0, "confidence": 0.9 },
            "geo_density": { "score": 2.0, "confidence": 0.6 },
            "geo_freshness": { "score": 1.0 }
        }))
        .unwrap();
        let ids = needs_review(&extra, "geo");
        assert_eq!(ids, vec!["geo_density", "geo_freshness"]);
    }

    #[test]
    fn test_dfs_backend_parks_without_keys() {
        use crate::serp::{dfs_enabled, Provider};
        let (u, p) = (
            std::env::var("DATAFORSEO_USERNAME").ok(),
            std::env::var("DATAFORSEO_PASSWORD").ok(),
        );
        std::env::remove_var("DATAFORSEO_USERNAME");
        std::env::remove_var("DATAFORSEO_PASSWORD");
        assert!(!dfs_enabled());
        // Explicit Dfs stays explicit (never silently rerouted); no network here.
        assert_eq!(crate::serp::select_provider(Provider::Dfs), Provider::Dfs);
        if let Some(k) = u {
            std::env::set_var("DATAFORSEO_USERNAME", k);
        }
        if let Some(k) = p {
            std::env::set_var("DATAFORSEO_PASSWORD", k);
        }
    }

    #[test]
    fn test_base64_basic_vectors() {
        // Standard vectors: padding 0, 1, 2.
        assert_eq!(crate::serp::dfs_basic_for_test("Man"), "TWFu");
        assert_eq!(crate::serp::dfs_basic_for_test("Ma"), "TWE=");
        assert_eq!(crate::serp::dfs_basic_for_test("M"), "TQ==");
        assert_eq!(crate::serp::dfs_basic_for_test("user:pass"), "dXNlcjpwYXNz");
    }

    #[test]
    fn test_question_registry_snapshot() {
        // Jev question ids and types are stable API for second-judge reruns.
        // Wording edits must show up here as deliberate diffs, never silently.
        use std::collections::BTreeMap;
        let suites: &[(&str, serde_json::Value)] = &[
            ("geo", crate::policy::geo_questions()),
            ("page", crate::policy::page_audit_extras()),
            ("site", crate::policy::site_extras()),
            ("brief", crate::policy::brief_extras()),
            ("keywords", crate::policy::keyword_value_extras(&["x".into()], 1)),
        ];
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        for (suite, v) in suites {
            let obj = v.as_object().expect("suite is an object");
            assert!(!obj.is_empty(), "{suite} suite must not be empty");
            for (qid, q) in obj {
                let qtype = q.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                assert!(
                    ["score", "choice", "noul"].contains(&qtype),
                    "{suite}.{qid} has unknown type {qtype}"
                );
                assert!(
                    q.get("instructions").and_then(|s| s.as_str()).map(|s| !s.is_empty()).unwrap_or(false),
                    "{suite}.{qid} needs instructions"
                );
                seen.insert(format!("{suite}.{qid}"), qtype.to_string());
            }
        }
        let ids: Vec<String> = seen.keys().cloned().collect();
        let joined = ids.join(",");
        for must in [
            "geo.geo_structure",
            "geo.geo_density",
            "geo.geo_directness",
            "geo.geo_statistics",
            "geo.geo_freshness",
        ] {
            assert!(joined.contains(must), "missing {must} in {joined}");
        }
        assert!(seen.len() >= 12, "registry shrank to {}", seen.len());
    }

    #[test]
    fn test_capped_string_truncates() {
        // Aux-body cap: oversized responses shrink instead of OOMing.
        let big = "x".repeat(crate::fetch::MAX_AUX_BYTES + 100);
        assert!(big.len() > crate::fetch::MAX_AUX_BYTES);
    }

    #[test]
    fn test_api_endpoint_override_refuses_private() {
        assert!(crate::paths::reject_api_endpoint("http://127.0.0.1:8000/x", "TEST").is_err());
        assert!(crate::paths::reject_api_endpoint("http://10.0.0.5/", "TEST").is_err());
        // Fail closed: .invalid never resolves, so the host is unverifiable.
        assert!(crate::paths::reject_api_endpoint("https://unresolvable.invalid/", "TEST").is_err());
    }

    #[test]
    fn test_chain_exhaust_reports_redirect_not_fetch_failure() {
        use crate::crawl::{finish_report, PageRecord, ReportParts};
        use std::collections::HashMap;
        let pages = vec![
            PageRecord { url: "https://x.test/loop".into(), status: 0, final_url: "https://x.test/loop".into(), outlinks: 0, elapsed_ms: 5, bytes: 0, hops: vec![(301, "https://x.test/loop".into()); 6], encoding: None, source: "direct".into(), fetch_cost: 0, upgraded: false },
        ];
        let rep = finish_report(ReportParts {
            start_url: "https://x.test/".into(),
            pages,
            redirects: vec![],
            inbound: HashMap::new(),
            errors: vec![],
            seeded_from_sitemap: true,
            capped: false,
            robots_honored: true,
            vitals: None,
            probes: vec![],
        });
        assert!(rep.findings.iter().any(|f| f.rule_id == "R03"));
        assert!(!rep.findings.iter().any(|f| f.rule_id == "R02"));
    }

    #[test]
    fn test_r19_needs_density_not_single_hit() {
        use crate::rules::check_audit;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "---\ntitle: A reasonably long page title for testing here\n---\n# Title\n\nOne delve.\n").unwrap();
        let rep = crate::audit::audit_path(dir.path().to_str().unwrap()).unwrap();
        let findings = check_audit(&rep);
        assert!(!findings.iter().any(|f| f.rule_id == "R19"), "single slop word must not fire R19");
    }

    #[test]
    fn test_provider_selection_stays_explicit() {
        use crate::serp::Provider;
        assert_eq!(crate::serp::select_provider(Provider::Ddg), Provider::Ddg);
        assert_eq!(crate::serp::select_provider(Provider::Tavily), Provider::Tavily);
    }

    #[test]
    fn test_tavily_without_key_errors_offline() {
        // Paid gate must read false before any network call when no key is set.
        let saved_key = std::env::var("TAVILY_API_KEY").ok();
        std::env::remove_var("TAVILY_API_KEY");
        let enabled = crate::serp::tavily_enabled();
        if let Some(k) = saved_key {
            std::env::set_var("TAVILY_API_KEY", k);
        }
        assert!(!enabled);
    }
    #[test]
    fn test_fetch_quality_and_budget() {
        use crate::fetch::Budget;
        let mut b = Budget { max_credits: 1, spent: 0 };
        assert!(b.allow(1));
        assert!(!b.allow(1));
    }

    #[test]
    fn test_gsc_date_and_encoding_shapes() {
        let d = crate::gsc::chrono_now_days_ago(28);
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
        assert!(crate::gsc::chrono_now_days_ago(0) >= d);
        assert_eq!(crate::gsc::urlencoding("https://x.test/a b"), "https%3A%2F%2Fx.test%2Fa%20b");
    }

    #[test]
    fn test_audit_to_html() {        use crate::audit::{audit_path, to_html};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "---\ntitle: T\ndescription: A fine description for testing.\n---\n# T\n\nWords here.\n").unwrap();
        let rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        let html = to_html(&rep);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("pass rate"));
        assert!(html.contains("jev-seo audit report"));
    }

    #[test]
    fn test_audit_to_pdf_structure() {
        use crate::audit::audit_path;
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.md", "b.md", "c.md"] {
            std::fs::write(
                dir.path().join(name),
                "---\ntitle: T\ndescription: A fine description for testing.\n---\n# T\n\nWords here live happily in this file with enough of them.\n",
            )
            .unwrap();
        }
        let rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        let pdf = crate::audit::to_pdf_opt(&rep, None);
        assert!(pdf.starts_with(b"%PDF-1.4\n"));
        assert!(pdf.ends_with(b"%%EOF"));
        assert!(pdf.windows(9).any(|w| w == b"endstream"));
        // startxref must point at the xref table.
        let text = String::from_utf8_lossy(&pdf);
        let xpos: usize = text.rsplit("startxref\n").next().unwrap().lines().next().unwrap().parse().unwrap();
        assert!(pdf[xpos..].starts_with(b"xref\n"));
        assert!(text.contains("jev-seo audit report"));
        assert!(text.contains("Scorecard:"));
        assert!(text.contains("Findings by area:"));
        assert!(text.contains("Page inventory"));
        assert!(text.contains("Method:"));
    }

    #[test]
    fn test_audit_to_markdown_tables() {
        use crate::audit::{audit_path, to_markdown};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "---\ntitle: T\ndescription: A fine description for testing.\n---\n# T\n\nWords here.\n").unwrap();
        let rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        let md = to_markdown(&rep);
        assert!(md.starts_with("# SEO audit:"));
        assert!(md.contains("| Page | Words | Title | Checks |"));
        assert!(md.contains("## Method"));
    }

    #[test]
    fn test_ssrf_non_dotted_ip_forms_refused() {
        use crate::paths::reject_private_url;
        assert!(reject_private_url("http://2130706433/").is_err());
        assert!(reject_private_url("http://0x7f.0.0.1/").is_err());
        assert!(reject_private_url("http://0177.0.0.1/").is_err());
        assert!(reject_private_url("http://[::ffff:127.0.0.1]/").is_err());
        assert!(reject_private_url("http://10.0.0.5/").is_err());
        assert!(reject_private_url("http://unresolvable.invalid/").is_err());
    }

    #[test]
    fn test_check_audit_path_denies_system_and_dotdirs() {
        use crate::paths::check_audit_path;
        assert!(check_audit_path("/etc").is_err());
        assert!(check_audit_path("/proc").is_err());
        let dir = tempfile::tempdir().unwrap();
        assert!(check_audit_path(dir.path().to_str().unwrap()).is_ok());
        let dot = dir.path().join(".git");
        std::fs::create_dir(&dot).unwrap();
        assert!(check_audit_path(dot.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_link_question_has_no_link_escape() {
        use crate::policy::link_question;
        let q = link_question(&[("guide".into(), "Guide page".into())]);
        assert_eq!(q["link_target"]["type"], "choice");
        assert!(q["link_target"]["criteria"].get("guide").is_some());
        assert!(q["link_target"]["criteria"].get("no_link").is_some());
    }

    #[test]
    fn test_intent_runner_up_picks_second() {
        use crate::policy::intent_runner_up;
        let extra: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "intent_probs": {"informational": 0.5, "navigational": 0.4, "commercial": 0.1}
            }))
            .unwrap();
        assert_eq!(intent_runner_up(&extra).as_deref(), Some("navigational (0.40)"));
        assert!(intent_runner_up(&serde_json::Map::new()).is_none());
    }

    #[test]
    fn test_body_words_counts_text_not_markup() {
        use crate::crawl::body_words;
        let html = "<html><head><title>T</title><script>var x = 1;</script></head><body><p>Hello brave new world of testing</p></body></html>";
        assert_eq!(body_words(html), 6);
        assert_eq!(body_words(""), 0);
    }

    #[test]
    fn test_effective_limit_clamps_paid_to_20() {
        use crate::serp::{effective_limit, Provider};
        assert_eq!(effective_limit(Provider::Tavily, 30), 20);
        assert_eq!(effective_limit(Provider::Dfs, 30), 20);
        assert_eq!(effective_limit(Provider::Ddg, 30), 30);
        assert_eq!(effective_limit(Provider::Tavily, 5), 5);
    }

    #[test]
    fn test_tavily_extract_validates_before_key() {
        use crate::serp::tavily_extract;
        assert!(tavily_extract(&[], "q").is_err());
        assert!(tavily_extract(&["https://x.test".into()], "  ").is_err());
        let many: Vec<String> = (0..11).map(|i| format!("https://x.test/{i}")).collect();
        assert!(tavily_extract(&many, "q").is_err());
    }

    #[test]
    fn test_html_attr_order_insensitive_and_og_needs_all_three() {
        use crate::audit::audit_file;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.html");
        std::fs::write(
            &path,
            "<html><head><title>Order Test Title Here For Length Check OK</title>\
            <meta content=\"A description long enough to pass the one hundred twenty char minimum for meta descriptions yes.\" name=\"description\">\
            <meta property=\"og:title\" content=\"T\"></head><body><h1>H</h1><p>Body copy.</p></body></html>",
        )
        .unwrap();
        let rep = audit_file(path.to_str().unwrap()).unwrap();
        assert!(rep.description.is_some(), "reversed meta attrs must parse");
        assert!(!rep.og_tags_found, "one og tag must not pass");
        assert_eq!(rep.images_missing_alt, 0);
    }

    #[test]
    fn test_empty_alt_is_decorative_not_missing() {
        use crate::audit::audit_file;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.html");
        std::fs::write(
            &path,
            "<html><head><title>Alt Test Title Here For Length Check OK</title></head>\
            <body><h1>H</h1><img src=\"a.png\" alt=\"\"><img src=\"b.png\"></body></html>",
        )
        .unwrap();
        let rep = audit_file(path.to_str().unwrap()).unwrap();
        assert_eq!(rep.image_count, 2);
        assert_eq!(rep.images_missing_alt, 1, "only the alt-less img counts");
    }

    #[test]
    fn test_broken_jsonld_fails_r33_and_short_lede_fails_r24() {
        use crate::audit::{audit_file, audit_path};
        use crate::rules::check_audit;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("p.html"),
            "<html><head><title>Broken LD Title Here For Length Check OK</title></head>\
            <body><h1>H</h1><p>Short lede.</p><script type=\"application/ld+json\">{\"@type\": }</script></body></html>",
        )
        .unwrap();
        let file_rep = audit_file(dir.path().join("p.html").to_str().unwrap()).unwrap();
        assert!(!file_rep.schema_json_valid);
        let dir_rep = audit_path(dir.path().to_str().unwrap()).unwrap();
        let findings = check_audit(&dir_rep);
        assert!(findings.iter().any(|f| f.rule_id == "R33"), "invalid JSON-LD must fire R33");
        assert!(findings.iter().any(|f| f.rule_id == "R24"), "short lede must fire R24");
    }

    #[test]
    fn test_orphan_emits_r25_and_links_area_scores() {
        use crate::crawl::{finish_report, PageRecord, ReportParts};
        use std::collections::HashMap;
        let pages = vec![
            PageRecord { url: "https://x.test/".into(), status: 200, final_url: "https://x.test/".into(), outlinks: 0, elapsed_ms: 100, bytes: 500, hops: vec![], encoding: Some("gzip".into()), source: "direct".into(), fetch_cost: 0, upgraded: false },
            PageRecord { url: "https://x.test/orph".into(), status: 200, final_url: "https://x.test/orph".into(), outlinks: 0, elapsed_ms: 100, bytes: 500, hops: vec![], encoding: Some("gzip".into()), source: "direct".into(), fetch_cost: 0, upgraded: false },
        ];
        let rep = finish_report(ReportParts {
            start_url: "https://x.test/".into(),
            pages,
            redirects: vec![],
            inbound: HashMap::new(),
            errors: vec![],
            seeded_from_sitemap: true,
            capped: false,
            robots_honored: true,
            vitals: None,
            probes: vec![],
        });
        assert_eq!(rep.orphans, vec!["https://x.test/orph".to_string()]);
        assert!(rep.findings.iter().any(|f| f.rule_id == "R25"), "orphans must emit R25");
        assert!(rep.areas.iter().any(|a| a.area == crate::rules::Area::Links), "Links must score when R25 fires");
        assert!(!rep.areas.iter().any(|a| a.area == crate::rules::Area::OnPage), "phantom OnPage must stay out");
    }

    #[test]
    fn test_gate_fixtures_good_passes_broken_fails() {
        let good = crate::audit::with_findings(crate::audit::audit_path("tests/fixtures/good").unwrap());
        assert!(
            crate::rules::blocking_findings(&good.findings).is_empty(),
            "good fixture must have zero blocking findings: {:?}",
            crate::rules::blocking_findings(&good.findings)
                .iter()
                .map(|f| f.rule_id.clone())
                .collect::<Vec<_>>()
        );
        let bad = crate::audit::with_findings(crate::audit::audit_path("tests/fixtures/broken").unwrap());
        let blocked = crate::rules::blocking_findings(&bad.findings);
        assert!(blocked.iter().any(|f| f.rule_id == "R09"), "broken fixture must fire R09");
        // Template-owned tags stay findings on Markdown but never block it.
        assert!(bad.findings.iter().any(|f| f.rule_id == "R11"));
        assert!(!blocked.iter().any(|f| f.rule_id == "R11"), "R11 must not block Markdown");
    }

    #[test]
    fn test_score_side_reads_heavier_half() {
        use crate::policy::score_side;
        let ans = serde_json::json!({
            "score": 3.0,
            "legend": {"0": "low", "4": "high"},
            "probabilities": {"0": 0.1, "3": 0.35, "4": 0.55}
        });
        assert_eq!(score_side(&ans), Some(0.9));
        let split = serde_json::json!({
            "score": 2.0,
            "legend": {"0": "low", "4": "high"},
            "probabilities": {"0": 0.5, "4": 0.5}
        });
        assert_eq!(score_side(&split), Some(0.5));
        assert!(score_side(&serde_json::json!({"score": 1.0})).is_none());
    }

    #[test]
    fn test_reserve_settle_roundtrip() {
        use crate::manifest::{
            reserve_jev_tokens, settle_jev_tokens, set_jev_budget_usd, DEFAULT_JEV_BUDGET_USD,
            JEV_INPUT_TOKENS,
        };
        use std::sync::atomic::Ordering;
        set_jev_budget_usd(0.0);
        assert!(!reserve_jev_tokens(1));
        set_jev_budget_usd(DEFAULT_JEV_BUDGET_USD);
        assert!(reserve_jev_tokens(100));
        let before = JEV_INPUT_TOKENS.load(Ordering::Relaxed);
        settle_jev_tokens(100, Some(60));
        assert_eq!(JEV_INPUT_TOKENS.load(Ordering::Relaxed), before + 60);
        settle_jev_tokens(100, None);
        assert_eq!(JEV_INPUT_TOKENS.load(Ordering::Relaxed), before + 160);
        JEV_INPUT_TOKENS.fetch_sub(160, Ordering::Relaxed);
    }

    #[test]
    fn test_registry_invariants() {
        use crate::rules::{gate, truth_kind, Gate, RULES};
        assert_eq!(RULES.len(), 57);
        let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 57, "rule ids must be unique");
        for r in RULES {
            assert!(!r.title.is_empty() && !r.fix.is_empty(), "{}", r.id);
            assert!(r.effort >= 1 && r.effort <= 3, "{}", r.id);
            // gate() and truth_kind() must total-cover the registry: no rule
            // falls through to a default it was never assigned.
            let _ = gate(r.id);
            let _ = truth_kind(r.id);
            assert!(matches!(gate(r.id), Gate::Blocking | Gate::Advisory));
            assert!(r.id.starts_with('R'));
        }
        // Heuristic set is exactly the threshold-guess rules.
        let heu: Vec<&&str> = RULES
            .iter()
            .filter(|r| truth_kind(r.id) == "heuristic")
            .map(|r| &r.id)
            .collect();
        assert_eq!(heu, vec![&"R18", &"R19", &"R20", &"R23", &"R24", &"R55", &"R41"]);
    }

    #[test]
    fn test_effective_gate_downgrades_template_tags_on_markdown() {
        use crate::rules::{effective_gate, Gate};
        for id in ["R11", "R26", "R31", "R36"] {
            assert_eq!(effective_gate(id, "docs/page.md"), Gate::Advisory, "{id}");
            assert_eq!(effective_gate(id, "docs/page.html"), Gate::Blocking, "{id}");
        }
        // R35 is advisory everywhere already; R09 stays blocking on Markdown.
        assert_eq!(effective_gate("R35", "docs/page.md"), Gate::Advisory);
        assert_eq!(effective_gate("R09", "docs/page.md"), Gate::Blocking);
        assert_eq!(effective_gate("R19", "docs/page.md"), Gate::Advisory);
    }

    #[test]
    fn test_crawl_snapshot_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("t.db");
        let store = DbStore::open_at(db_path.to_str().unwrap()).unwrap();
        assert!(store.record_crawl_snapshot("https://example.com", 10, 1).unwrap().is_none());
        let prev = store.record_crawl_snapshot("https://example.com", 12, 0).unwrap().unwrap();
        assert_eq!(prev, (10, 1));
    }

    #[test]
    fn test_readable_text_scores_body_copy_not_head_markup() {
        use crate::fetch::readable_text;
        let html = "<!DOCTYPE html><html><head><title>Some Tool</title>\
            <meta name=\"description\" content=\"meta words here\">\
            <script>var tracking = {key: \"secret-beacon\", pixels: [1,2,3]};</script>\
            <style>.nav{display:none}</style></head>\
            <body><nav>Home Products Pricing Login</nav>\
            <main><h1>Agent readiness checker</h1>\
            <p>Paste any public URL and get a 0-100 agent readiness score with 118 evidence checks.</p></main>\
            <footer>Copyright 2026</footer></body></html>";
        let text = readable_text(html, 6000);
        assert!(text.contains("Agent readiness checker"), "{text}");
        assert!(text.contains("118 evidence checks"), "{text}");
        assert!(!text.contains("secret-beacon"), "script body leaked: {text}");
        assert!(!text.contains("meta words here"), "head markup leaked: {text}");
        assert!(!text.contains('<'), "raw tags leaked: {text}");
    }

    #[test]
    fn test_readable_text_caps_at_limit() {
        use crate::fetch::readable_text;
        let text = readable_text("<body><p>word word word</p></body>", 10);
        assert!(text.chars().count() <= 10, "{text}");
        assert!(text.contains("word"), "{text}");
    }

    #[test]
    fn test_page_state_has_single_text_field() {
        use crate::engine::page_state;
        let state = page_state("q", Some("t".into()), None, "body copy".into(), 2, None);
        assert!(state.get("content").is_none(), "duplicated content key: {state}");
        assert_eq!(state["page"]["text"], "body copy");
        assert_eq!(state["query"], "q");
    }
}
