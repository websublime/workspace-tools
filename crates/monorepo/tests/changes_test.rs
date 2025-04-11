#[cfg(test)]
mod changes_tests {
    use std::str::FromStr;
    use sublime_monorepo_tools::{Change, ChangeId, ChangeType};

    #[test]
    fn test_change_id_creation() {
        // Test new ID generation
        let id = ChangeId::new();
        let id_str = id.to_string();

        // Verify it's a valid UUID string
        assert!(!id_str.is_empty());
        assert_eq!(id_str.len(), 36); // UUID string length

        // Test string conversion
        let parsed_id = ChangeId::from_str(&id_str).expect("Failed to parse ID");
        assert_eq!(id.to_string(), parsed_id.to_string());

        // Test default creates new random ID
        let default_id = ChangeId::default();
        assert_ne!(id.to_string(), default_id.to_string());
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(ChangeType::Feature.to_string(), "feature");
        assert_eq!(ChangeType::Fix.to_string(), "fix");
        assert_eq!(ChangeType::Documentation.to_string(), "docs");
        assert_eq!(ChangeType::Performance.to_string(), "perf");
        assert_eq!(ChangeType::Refactor.to_string(), "refactor");
        assert_eq!(ChangeType::Test.to_string(), "test");
        assert_eq!(ChangeType::Chore.to_string(), "chore");
        assert_eq!(ChangeType::Build.to_string(), "build");
        assert_eq!(ChangeType::CI.to_string(), "ci");
        assert_eq!(ChangeType::Revert.to_string(), "revert");
        assert_eq!(ChangeType::Style.to_string(), "style");
        assert_eq!(ChangeType::Custom("migration".to_string()).to_string(), "migration");
        assert_eq!(ChangeType::Unknown.to_string(), "unknown");
        assert_eq!(ChangeType::Breaking.to_string(), "breaking");
    }

    #[test]
    fn test_change_creation() {
        // Create a basic change
        let change = Change::new(
            "@scope/package-foo",
            ChangeType::Feature,
            "Add new button component",
            false,
        );

        // Verify fields
        assert_eq!(change.package, "@scope/package-foo");
        assert!(matches!(change.change_type, ChangeType::Feature));
        assert_eq!(change.description, "Add new button component");
        assert!(!change.breaking);
        assert!(change.author.is_none());
        assert!(change.issues.is_empty());
        assert!(change.release_version.is_none());
        assert!(change.environments.is_empty());
    }

    #[test]
    fn test_change_builder_methods() {
        // Test builder methods
        let change =
            Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation bug", false)
                .with_author("Test Author")
                .with_issues(vec!["#123", "JIRA-456"])
                .with_environments(vec!["production", "staging"])
                .with_release_version("1.0.0");

        // Verify added properties
        assert_eq!(change.author, Some("Test Author".to_string()));
        assert_eq!(change.issues, vec!["#123", "JIRA-456"]);
        assert_eq!(change.environments, vec!["production", "staging"]);
        assert_eq!(change.release_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_change_is_released() {
        // Create unreleased change
        let unreleased =
            Change::new("@scope/package-baz", ChangeType::Feature, "New feature", false);
        assert!(!unreleased.is_released());

        // Create released change
        let released = Change::new("@scope/package-baz", ChangeType::Feature, "New feature", false)
            .with_release_version("1.0.0");
        assert!(released.is_released());
    }

    #[test]
    fn test_change_applies_to_environment() {
        // Change with no specific environments (applies to all)
        let all_envs = Change::new("@scope/package-charlie", ChangeType::Fix, "Bug fix", false);
        assert!(all_envs.applies_to_environment("production"));
        assert!(all_envs.applies_to_environment("staging"));

        // Change with specific environments
        let prod_only = Change::new("@scope/package-charlie", ChangeType::Fix, "Bug fix", false)
            .with_environments(vec!["production"]);
        assert!(prod_only.applies_to_environment("production"));
        assert!(!prod_only.applies_to_environment("staging"));
    }

    #[test]
    fn test_change_summary() {
        // Regular change
        let feature = Change::new("@scope/package-major", ChangeType::Feature, "Add button", false);
        assert_eq!(feature.summary(), "feature: Add button");

        // Breaking change
        let breaking =
            Change::new("@scope/package-major", ChangeType::Fix, "Update auth logic", true);
        assert_eq!(breaking.summary(), "fix!: Update auth logic");
    }
}
