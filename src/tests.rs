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
        assert_eq!(tools.len(), 8, "All 8 agent SEO tools must be exposed");

        let names: Vec<&str> = tools.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str())).collect();
        assert!(names.contains(&"seo_keywords"));
        assert!(names.contains(&"seo_serp_inspect"));
        assert!(names.contains(&"seo_audit"));
        assert!(names.contains(&"seo_geo"));
        assert!(names.contains(&"seo_schema"));
        assert!(names.contains(&"seo_robots"));
        assert!(names.contains(&"seo_brief"));
        assert!(names.contains(&"seo_sitemap"));
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
}
