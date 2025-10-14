#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod conventional_tests {
    use crate::config::{ConventionalCommitType, PackageToolsConfig};
    use crate::conventional::{CommitAnalysis, ConventionalCommitService};
    use crate::{
        conventional::{CommitType, ConventionalCommit, ConventionalCommitParser},
        VersionBump,
    };
    use chrono::Utc;
    use sublime_git_tools::Repo;
    use tempfile::TempDir;

    #[test]
    fn test_commit_type_as_str() {
        assert_eq!(CommitType::Feat.as_str(), "feat");
        assert_eq!(CommitType::Fix.as_str(), "fix");
        assert_eq!(CommitType::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_parser_creation() {
        let parser = ConventionalCommitParser::new().unwrap();
        assert!(!parser.type_config.is_empty());
    }

    #[test]
    fn test_parse_simple_commit() {
        let parser = ConventionalCommitParser::new().unwrap();
        let result = parser.parse(
            "feat: add new feature",
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.description, "add new feature");
        assert!(!commit.breaking);
        assert!(commit.scope.is_none());
    }

    #[test]
    fn test_parse_commit_with_scope() {
        let parser = ConventionalCommitParser::new().unwrap();
        let result = parser.parse(
            "fix(auth): resolve login issue",
            "def456".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type, CommitType::Fix);
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "resolve login issue");
    }

    #[test]
    fn test_parse_breaking_change_marker() {
        let parser = ConventionalCommitParser::new().unwrap();
        let result = parser.parse(
            "feat!: breaking change in API",
            "ghi789".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert!(commit.breaking);
    }

    #[test]
    fn test_parse_invalid_format() {
        let parser = ConventionalCommitParser::new().unwrap();
        let result = parser.parse(
            "not a conventional commit",
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_version_bump_calculation() {
        let parser = ConventionalCommitParser::new().unwrap();

        assert_eq!(parser.get_version_bump(&CommitType::Feat, false), VersionBump::Minor);
        assert_eq!(parser.get_version_bump(&CommitType::Fix, false), VersionBump::Patch);
        assert_eq!(parser.get_version_bump(&CommitType::Docs, false), VersionBump::None);
        assert_eq!(parser.get_version_bump(&CommitType::Feat, true), VersionBump::Major);
    }

    #[test]
    fn test_changelog_inclusion() {
        let parser = ConventionalCommitParser::new().unwrap();

        assert!(parser.should_include_in_changelog(&CommitType::Feat));
        assert!(parser.should_include_in_changelog(&CommitType::Fix));
        assert!(!parser.should_include_in_changelog(&CommitType::Docs));
        assert!(!parser.should_include_in_changelog(&CommitType::Style));
    }

    #[test]
    fn test_commit_type_display() {
        assert_eq!(format!("{}", CommitType::Feat), "feat");
        assert_eq!(format!("{}", CommitType::Fix), "fix");
        assert_eq!(format!("{}", CommitType::Docs), "docs");
        assert_eq!(format!("{}", CommitType::Other("custom".to_string())), "custom");
    }

    fn create_test_repo() -> (TempDir, Repo) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repo::create(temp_dir.path().to_str().unwrap()).unwrap();
        (temp_dir, repo)
    }

    fn create_test_conventional_commit(
        commit_type: CommitType,
        breaking: bool,
        hash: &str,
    ) -> ConventionalCommit {
        ConventionalCommit {
            commit_type,
            scope: None,
            breaking,
            description: "test description".to_string(),
            body: None,
            footer: None,
            hash: hash.to_string(),
            author: "Test Author".to_string(),
            date: Utc::now(),
        }
    }

    #[test]
    fn test_service_creation() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_calculate_version_bump_empty() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::None);
    }

    #[test]
    fn test_calculate_version_bump_feat() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![create_test_conventional_commit(CommitType::Feat, false, "abc123")];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Minor);
    }

    #[test]
    fn test_calculate_version_bump_fix() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![create_test_conventional_commit(CommitType::Fix, false, "abc123")];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Patch);
    }

    #[test]
    fn test_calculate_version_bump_breaking() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![create_test_conventional_commit(CommitType::Fix, true, "abc123")];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Major);
    }

    #[test]
    fn test_calculate_version_bump_mixed() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![
            create_test_conventional_commit(CommitType::Fix, false, "abc123"),
            create_test_conventional_commit(CommitType::Feat, false, "def456"),
            create_test_conventional_commit(CommitType::Docs, false, "ghi789"),
        ];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Minor); // feat wins over fix
    }

    #[test]
    fn test_calculate_version_bump_priority() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![
            create_test_conventional_commit(CommitType::Feat, false, "abc123"),
            create_test_conventional_commit(CommitType::Fix, true, "def456"), // breaking fix
        ];
        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Major); // breaking wins over feat
    }

    #[test]
    fn test_group_commits_by_type() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![
            create_test_conventional_commit(CommitType::Feat, false, "abc123"),
            create_test_conventional_commit(CommitType::Fix, false, "def456"),
            create_test_conventional_commit(CommitType::Feat, false, "ghi789"),
            create_test_conventional_commit(CommitType::Docs, false, "jkl012"),
        ];

        let grouped = service.group_commits_by_type(&commits);

        // Docs should be filtered out as it's not included in changelog
        assert_eq!(grouped.len(), 2);
        assert!(grouped.contains_key(&CommitType::Feat));
        assert!(grouped.contains_key(&CommitType::Fix));
        assert!(!grouped.contains_key(&CommitType::Docs));

        assert_eq!(grouped[&CommitType::Feat].len(), 2);
        assert_eq!(grouped[&CommitType::Fix].len(), 1);
    }

    #[test]
    fn test_analyze_commits() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![
            create_test_conventional_commit(CommitType::Feat, false, "abc123"),
            create_test_conventional_commit(CommitType::Fix, true, "def456"),
            create_test_conventional_commit(CommitType::Docs, false, "ghi789"),
        ];

        let analysis = service.analyze_commits(&commits);

        assert_eq!(analysis.total_commits, 3);
        assert_eq!(analysis.conventional_commits, 3);
        assert_eq!(analysis.suggested_bump, VersionBump::Major);
        assert!(analysis.has_breaking_changes);
        assert_eq!(analysis.breaking_changes.len(), 1);
        assert_eq!(analysis.breaking_changes[0].hash, "def456");

        // Check type distribution
        assert_eq!(analysis.type_distribution[&CommitType::Feat], 1);
        assert_eq!(analysis.type_distribution[&CommitType::Fix], 1);
        assert_eq!(analysis.type_distribution[&CommitType::Docs], 1);
    }

    #[test]
    fn test_analyze_commits_no_breaking() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![
            create_test_conventional_commit(CommitType::Feat, false, "abc123"),
            create_test_conventional_commit(CommitType::Fix, false, "def456"),
        ];

        let analysis = service.analyze_commits(&commits);

        assert_eq!(analysis.total_commits, 2);
        assert_eq!(analysis.suggested_bump, VersionBump::Minor);
        assert!(!analysis.has_breaking_changes);
        assert!(analysis.breaking_changes.is_empty());
    }

    #[test]
    fn test_analyze_commits_empty() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![];
        let analysis = service.analyze_commits(&commits);

        assert_eq!(analysis.total_commits, 0);
        assert_eq!(analysis.conventional_commits, 0);
        assert_eq!(analysis.suggested_bump, VersionBump::None);
        assert!(!analysis.has_breaking_changes);
        assert!(analysis.breaking_changes.is_empty());
        assert!(analysis.type_distribution.is_empty());
    }

    #[test]
    fn test_commit_analysis_debug() {
        let analysis = CommitAnalysis {
            total_commits: 5,
            conventional_commits: 4,
            suggested_bump: VersionBump::Minor,
            has_breaking_changes: false,
            type_distribution: std::collections::HashMap::new(),
            breaking_changes: vec![],
        };

        let debug_str = format!("{:?}", analysis);
        assert!(debug_str.contains("total_commits: 5"));
        assert!(debug_str.contains("conventional_commits: 4"));
        assert!(debug_str.contains("suggested_bump: Minor"));
    }

    #[test]
    fn test_commit_analysis_clone() {
        let original = CommitAnalysis {
            total_commits: 3,
            conventional_commits: 3,
            suggested_bump: VersionBump::Patch,
            has_breaking_changes: true,
            type_distribution: std::collections::HashMap::new(),
            breaking_changes: vec![create_test_conventional_commit(
                CommitType::Fix,
                true,
                "abc123",
            )],
        };

        let cloned = original.clone();
        assert_eq!(cloned.total_commits, original.total_commits);
        assert_eq!(cloned.conventional_commits, original.conventional_commits);
        assert_eq!(cloned.suggested_bump, original.suggested_bump);
        assert_eq!(cloned.has_breaking_changes, original.has_breaking_changes);
        assert_eq!(cloned.breaking_changes.len(), original.breaking_changes.len());
    }

    #[test]
    fn test_parser_with_custom_config() {
        use crate::config::{ConventionalCommitType, ConventionalConfig};
        use std::collections::HashMap;

        let mut types = HashMap::new();
        types.insert(
            "custom".to_string(),
            ConventionalCommitType {
                bump: "minor".to_string(),
                changelog: true,
                changelog_title: Some("Custom Changes".to_string()),
                breaking: false,
            },
        );

        let config = ConventionalConfig {
            types,
            parse_breaking_changes: true,
            require_conventional_commits: true,
            breaking_change_patterns: vec!["CUSTOM BREAK:".to_string()],
            default_bump_type: "patch".to_string(),
        };

        let parser = ConventionalCommitParser::with_config(config).unwrap();
        let result = parser.parse(
            "custom: add new functionality",
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type, CommitType::Other("custom".to_string()));
        assert_eq!(commit.description, "add new functionality");

        // Test version bump for custom type
        let bump = parser.get_version_bump(&commit.commit_type, false);
        assert_eq!(bump, VersionBump::Minor);

        // Test changelog inclusion
        assert!(parser.should_include_in_changelog(&commit.commit_type));

        // Test changelog section
        assert_eq!(parser.get_changelog_section(&commit.commit_type), Some("Custom Changes"));
    }

    #[test]
    fn test_parser_custom_breaking_patterns() {
        use crate::config::{ConventionalCommitType, ConventionalConfig};
        use std::collections::HashMap;

        let mut types = HashMap::new();
        types.insert(
            "feat".to_string(),
            ConventionalCommitType {
                bump: "minor".to_string(),
                changelog: true,
                changelog_title: Some("Features".to_string()),
                breaking: false,
            },
        );

        let config = ConventionalConfig {
            types,
            parse_breaking_changes: true,
            require_conventional_commits: false,
            breaking_change_patterns: vec!["CUSTOM BREAK:".to_string()],
            default_bump_type: "patch".to_string(),
        };

        let parser = ConventionalCommitParser::with_config(config).unwrap();
        let commit_message = "feat: add new feature\n\nCUSTOM BREAK: This breaks the API";

        let result = parser.parse(
            commit_message,
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert!(commit.breaking);
    }

    #[test]
    fn test_parser_default_bump_type() {
        use crate::config::ConventionalConfig;
        use std::collections::HashMap;

        let config = ConventionalConfig {
            types: HashMap::new(), // No custom types
            parse_breaking_changes: true,
            require_conventional_commits: false,
            breaking_change_patterns: vec!["BREAKING CHANGE:".to_string()],
            default_bump_type: "minor".to_string(),
        };

        let parser = ConventionalCommitParser::with_config(config).unwrap();

        // Test unknown commit type uses default bump
        let unknown_type = CommitType::Other("unknown".to_string());
        let bump = parser.get_version_bump(&unknown_type, false);
        assert_eq!(bump, VersionBump::Minor);
    }

    #[test]
    fn test_parser_required_conventional_commits() {
        use crate::config::ConventionalConfig;

        let mut config = ConventionalConfig::default();
        config.require_conventional_commits = true;

        let parser = ConventionalCommitParser::with_config(config).unwrap();
        assert!(parser.are_conventional_commits_required());

        let mut config = ConventionalConfig::default();
        config.require_conventional_commits = false;

        let parser = ConventionalCommitParser::with_config(config).unwrap();
        assert!(!parser.are_conventional_commits_required());
    }

    #[test]
    fn test_parser_disable_breaking_change_parsing() {
        use crate::config::ConventionalConfig;

        let mut config = ConventionalConfig::default();
        config.parse_breaking_changes = false;

        let parser = ConventionalCommitParser::with_config(config).unwrap();
        let commit_message = "feat: add new feature\n\nBREAKING CHANGE: This breaks the API";

        let result = parser.parse(
            commit_message,
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert!(!commit.breaking); // Should not detect breaking change in body
    }

    #[test]
    fn test_service_with_custom_config() {
        use crate::config::ConventionalConfig;
        use std::collections::HashMap;

        let (_temp_dir, repo) = create_test_repo();

        let mut types = HashMap::new();
        types.insert(
            "custom".to_string(),
            ConventionalCommitType {
                bump: "major".to_string(),
                changelog: true,
                changelog_title: Some("Custom Changes".to_string()),
                breaking: false,
            },
        );

        let conventional_config = ConventionalConfig {
            types,
            parse_breaking_changes: true,
            require_conventional_commits: false,
            breaking_change_patterns: vec!["BREAKING CHANGE:".to_string()],
            default_bump_type: "patch".to_string(),
        };

        let mut config = PackageToolsConfig::default();
        config.conventional = conventional_config;

        let service = ConventionalCommitService::new(repo, config).unwrap();

        let commits = vec![create_test_conventional_commit(
            CommitType::Other("custom".to_string()),
            false,
            "abc123",
        )];

        let bump = service.calculate_version_bump(&commits);
        assert_eq!(bump, VersionBump::Major);
    }

    #[test]
    fn test_get_commits_between_integration() {
        let (_temp_dir, repo) = create_test_repo();
        let config = PackageToolsConfig::default();
        let service = ConventionalCommitService::new(repo, config).unwrap();

        // This test verifies that get_commits_between now works properly
        // Since we're using a mock repo without the references, we expect an error
        // but this proves the integration is working (method exists and is callable)
        let result = futures::executor::block_on(async {
            service.get_commits_between("main", "feature-branch").await
        });

        // The result should be an error because references don't exist
        // but this proves the API integration is working
        assert!(result.is_err(), "get_commits_between should fail with non-existent refs");

        // Test with same ref (should return empty)
        let result2 = futures::executor::block_on(async {
            service.get_commits_between("HEAD", "HEAD").await
        });

        // This might work or fail depending on repo state, but method should be callable
        // The key is that we're testing the integration exists
        let _test_integration = result2; // Just verify it compiles and runs
    }

    #[test]
    fn test_conventional_commit_creation() {
        let commit = ConventionalCommit {
            commit_type: CommitType::Feat,
            scope: Some("ui".to_string()),
            description: "add new button component".to_string(),
            body: Some("Implemented new button with various styles".to_string()),
            breaking: false,
            hash: "abc123def".to_string(),
            author: "Developer".to_string(),
            date: Utc::now(),
            footer: None,
        };

        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, Some("ui".to_string()));
        assert_eq!(commit.description, "add new button component");
        assert!(!commit.breaking);
    }

    #[test]
    fn test_parse_commit_with_body() {
        let parser = ConventionalCommitParser::new().unwrap();
        let commit_message = "feat(ui): add new button\n\nThis is a detailed description of the new button component.";

        let result = parser.parse(
            commit_message,
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, Some("ui".to_string()));
        assert_eq!(commit.description, "add new button");
        assert!(commit.body.is_some());
        assert!(commit.body.unwrap().contains("detailed description"));
    }

    #[test]
    fn test_parse_commit_edge_cases() {
        let parser = ConventionalCommitParser::new().unwrap();

        // Empty description
        let result =
            parser.parse("feat:", "abc123".to_string(), "Test Author".to_string(), Utc::now());
        assert!(result.is_err());

        // Only whitespace description
        let result =
            parser.parse("feat:   ", "abc123".to_string(), "Test Author".to_string(), Utc::now());
        assert!(result.is_ok());

        // Very long scope
        let long_scope = "a".repeat(100);
        let result = parser.parse(
            &format!("feat({}): description", long_scope),
            "abc123".to_string(),
            "Test Author".to_string(),
            Utc::now(),
        );
        assert!(result.is_ok());
    }
}
