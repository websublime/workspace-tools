#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod conventional_tests {
    use crate::{
        conventional::{CommitType, ConventionalCommit, ConventionalCommitParser},
        VersionBump,
    };
    use chrono::Utc;

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
