#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod changelog_tests {
    use crate::{
        changelog::{ChangelogEntry, ChangelogGenerator, ChangelogSection},
        conventional::{CommitType, ConventionalCommit},
    };
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_changelog_generator_creation() {
        let generator = ChangelogGenerator::new(true, false, true, false);
        assert!(generator.include_commit_hash);
        assert!(!generator.include_authors);
        assert!(generator.group_by_type);
        assert!(!generator.include_date);
    }

    #[test]
    fn test_changelog_generator_builder() {
        let generator = ChangelogGenerator::default()
            .with_max_commits(100)
            .with_template(PathBuf::from("template.md"));

        assert_eq!(generator.max_commits_per_release, Some(100));
        assert_eq!(generator.template_file, Some(PathBuf::from("template.md")));
    }

    #[test]
    fn test_changelog_entry_from_commit() {
        let commit = ConventionalCommit {
            commit_type: CommitType::Feat,
            scope: Some("auth".to_string()),
            breaking: false,
            description: "add OAuth2 support".to_string(),
            body: None,
            footer: None,
            hash: "abc123def".to_string(),
            author: "Test Author".to_string(),
            date: Utc::now(),
        };

        let entry = ChangelogEntry::from_commit(&commit, true, true);
        assert_eq!(entry.entry_type, "feat");
        assert_eq!(entry.scope, Some("auth".to_string()));
        assert_eq!(entry.description, "add OAuth2 support");
        assert!(!entry.breaking);
        assert_eq!(entry.commit_hash, Some("abc123def".to_string()));
        assert_eq!(entry.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_changelog_entry_format_markdown() {
        let entry = ChangelogEntry {
            entry_type: "feat".to_string(),
            scope: Some("auth".to_string()),
            description: "add OAuth2 support".to_string(),
            breaking: false,
            commit_hash: Some("abc123def".to_string()),
            author: Some("Test Author".to_string()),
            timestamp: Utc::now(),
        };

        let formatted = entry.format_markdown();
        assert!(formatted.contains("**auth:**"));
        assert!(formatted.contains("add OAuth2 support"));
        assert!(formatted.contains("(abc123d)"));
        assert!(formatted.contains("by Test Author"));
    }

    #[test]
    fn test_changelog_entry_breaking_change() {
        let entry = ChangelogEntry {
            entry_type: "feat".to_string(),
            scope: None,
            description: "remove deprecated API".to_string(),
            breaking: true,
            commit_hash: None,
            author: None,
            timestamp: Utc::now(),
        };

        let formatted = entry.format_markdown();
        assert!(formatted.starts_with("**BREAKING:**"));
        assert!(formatted.contains("remove deprecated API"));
    }

    #[test]
    fn test_changelog_section() {
        let mut section = ChangelogSection::new("Features".to_string());
        assert_eq!(section.title, "Features");
        assert!(section.is_empty());
        assert_eq!(section.entry_count(), 0);

        let entry = ChangelogEntry {
            entry_type: "feat".to_string(),
            scope: None,
            description: "add new feature".to_string(),
            breaking: false,
            commit_hash: None,
            author: None,
            timestamp: Utc::now(),
        };

        section.add_entry(entry);
        assert!(!section.is_empty());
        assert_eq!(section.entry_count(), 1);
    }

    #[test]
    fn test_changelog_section_format_markdown() {
        let mut section = ChangelogSection::new("Features".to_string());

        let entry = ChangelogEntry {
            entry_type: "feat".to_string(),
            scope: None,
            description: "add new feature".to_string(),
            breaking: false,
            commit_hash: None,
            author: None,
            timestamp: Utc::now(),
        };

        section.add_entry(entry);

        let formatted = section.format_markdown();
        assert!(formatted.contains("### Features"));
        assert!(formatted.contains("- add new feature"));
    }

    #[test]
    fn test_empty_section_format() {
        let section = ChangelogSection::new("Empty".to_string());
        let formatted = section.format_markdown();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_changelog_generator_default() {
        let generator = ChangelogGenerator::default();
        assert!(generator.include_commit_hash);
        assert!(!generator.include_authors);
        assert!(generator.group_by_type);
        assert!(!generator.include_date);
        assert!(generator.max_commits_per_release.is_none());
        assert!(generator.template_file.is_none());
    }

    #[test]
    fn test_changelog_entry_without_scope() {
        let entry = ChangelogEntry {
            entry_type: "fix".to_string(),
            scope: None,
            description: "resolve memory leak".to_string(),
            breaking: false,
            commit_hash: Some("def456abc".to_string()),
            author: None,
            timestamp: Utc::now(),
        };

        let formatted = entry.format_markdown();
        assert!(formatted.contains("resolve memory leak"));
        assert!(formatted.contains("(def456a)"));
        assert!(!formatted.contains("**:**"));
    }

    #[test]
    fn test_changelog_entry_without_commit_hash() {
        let entry = ChangelogEntry {
            entry_type: "docs".to_string(),
            scope: Some("readme".to_string()),
            description: "update installation guide".to_string(),
            breaking: false,
            commit_hash: None,
            author: Some("Doc Writer".to_string()),
            timestamp: Utc::now(),
        };

        let formatted = entry.format_markdown();
        assert!(formatted.contains("**readme:**"));
        assert!(formatted.contains("update installation guide"));
        assert!(formatted.contains("by Doc Writer"));
        assert!(!formatted.contains("()"));
    }

    #[test]
    fn test_changelog_section_multiple_entries() {
        let mut section = ChangelogSection::new("Bug Fixes".to_string());

        for i in 1..=3 {
            let entry = ChangelogEntry {
                entry_type: "fix".to_string(),
                scope: None,
                description: format!("fix bug #{}", i),
                breaking: false,
                commit_hash: None,
                author: None,
                timestamp: Utc::now(),
            };
            section.add_entry(entry);
        }

        assert_eq!(section.entry_count(), 3);
        assert!(!section.is_empty());

        let formatted = section.format_markdown();
        assert!(formatted.contains("### Bug Fixes"));
        assert!(formatted.contains("fix bug #1"));
        assert!(formatted.contains("fix bug #2"));
        assert!(formatted.contains("fix bug #3"));
    }

    #[test]
    fn test_changelog_generator_with_template() {
        let template_path = PathBuf::from("custom-template.hbs");
        let generator = ChangelogGenerator::default().with_template(template_path.clone());

        assert_eq!(generator.template_file, Some(template_path));
    }

    #[test]
    fn test_changelog_generator_with_max_commits() {
        let generator = ChangelogGenerator::default().with_max_commits(50);

        assert_eq!(generator.max_commits_per_release, Some(50));
    }

    #[test]
    fn test_changelog_entry_serialization() {
        let entry = ChangelogEntry {
            entry_type: "feat".to_string(),
            scope: Some("api".to_string()),
            description: "add new endpoint".to_string(),
            breaking: false,
            commit_hash: Some("abcdef123456".to_string()),
            author: Some("API Developer".to_string()),
            timestamp: Utc::now(),
        };

        // Test JSON serialization
        let json_result = serde_json::to_string(&entry);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<ChangelogEntry, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_entry = deserialized.unwrap();
        assert_eq!(deserialized_entry.entry_type, "feat");
        assert_eq!(deserialized_entry.scope, Some("api".to_string()));
        assert_eq!(deserialized_entry.description, "add new endpoint");
    }

    #[test]
    fn test_changelog_section_serialization() {
        let mut section = ChangelogSection::new("Tests".to_string());

        let entry = ChangelogEntry {
            entry_type: "test".to_string(),
            scope: None,
            description: "add unit tests".to_string(),
            breaking: false,
            commit_hash: None,
            author: None,
            timestamp: Utc::now(),
        };

        section.add_entry(entry);

        // Test JSON serialization
        let json_result = serde_json::to_string(&section);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<ChangelogSection, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_section = deserialized.unwrap();
        assert_eq!(deserialized_section.title, "Tests");
        assert_eq!(deserialized_section.entry_count(), 1);
    }

    #[test]
    fn test_changelog_entry_commit_hash_truncation() {
        let long_hash = "abcdef1234567890abcdef1234567890abcdef12";
        let entry = ChangelogEntry {
            entry_type: "chore".to_string(),
            scope: None,
            description: "update dependencies".to_string(),
            breaking: false,
            commit_hash: Some(long_hash.to_string()),
            author: None,
            timestamp: Utc::now(),
        };

        let formatted = entry.format_markdown();
        // Should truncate to 7 characters
        assert!(formatted.contains("(abcdef1)"));
        assert!(!formatted.contains(long_hash));
    }
}
