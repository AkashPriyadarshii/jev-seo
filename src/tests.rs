#[cfg(test)]
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
        let delta1 = db.track_keyword("example.com", "best rust framework", Some(5), Some("https://example.com/rust")).unwrap();
        assert_eq!(delta1.curr_rank, Some(5));

        let delta2 = db.track_keyword("example.com", "best rust framework", Some(3), Some("https://example.com/rust")).unwrap();
        assert_eq!(delta2.prev_rank, Some(5));
        assert_eq!(delta2.curr_rank, Some(3));
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
        assert_eq!(tools.len(), 7, "All 7 agent SEO tools must be exposed");

        let names: Vec<&str> = tools.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str())).collect();
        assert!(names.contains(&"seo_keywords"));
        assert!(names.contains(&"seo_serp_inspect"));
        assert!(names.contains(&"seo_audit"));
        assert!(names.contains(&"seo_geo"));
        assert!(names.contains(&"seo_schema"));
        assert!(names.contains(&"seo_robots"));
        assert!(names.contains(&"seo_brief"));
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
}
