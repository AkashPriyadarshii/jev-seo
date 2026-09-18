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
}
