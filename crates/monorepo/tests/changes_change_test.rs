mod test_utils;

use chrono::Utc;
use std::str::FromStr;
use sublime_monorepo_tools::{Change, ChangeId, ChangeType};

#[cfg(test)]
mod changes_change_tests {
    use super::*;

    #[test]
    fn test_change_id_creation() {
        // Create a new change ID
        let id = ChangeId::new();

        // Verify it's not empty
        assert!(!id.to_string().is_empty());

        // Test string conversion
        let id_str = id.to_string();
        let parsed_id = ChangeId::from_str(&id_str).expect("Failed to parse ChangeId");

        // The parsed ID should equal the original
        assert_eq!(id.to_string(), parsed_id.to_string());
    }

    #[test]
    fn test_change_creation() {
        // Create a basic change
        let change = Change::new("test-pkg", ChangeType::Feature, "Add new feature", false);

        // Verify basic properties
        assert_eq!(change.package, "test-pkg");
        assert!(matches!(change.change_type, ChangeType::Feature));
        assert_eq!(change.description, "Add new feature");
        assert!(!change.breaking);
        assert!(change.release_version.is_none());

        // Timestamp should be recent
        let now = Utc::now();
        let diff = now.timestamp() - change.timestamp.timestamp();
        assert!(diff < 5, "Change timestamp should be recent");
    }

    #[test]
    fn test_change_builder_methods() {
        // Test builder pattern methods
        let change = Change::new("pkg", ChangeType::Fix, "Fix bug", true)
            .with_author("John Doe")
            .with_issues(vec!["#123", "#456"])
            .with_release_version("1.2.3");

        // Verify all properties
        assert_eq!(change.package, "pkg");
        assert!(matches!(change.change_type, ChangeType::Fix));
        assert_eq!(change.description, "Fix bug");
        assert!(change.breaking);
        assert_eq!(change.author, Some("John Doe".to_string()));
        assert_eq!(change.issues, vec!["#123", "#456"]);
        assert_eq!(change.release_version, Some("1.2.3".to_string()));
        assert!(change.is_released());
    }

    #[test]
    fn test_change_type_display() {
        // Test string representation of change types
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
        assert_eq!(ChangeType::Custom("custom".to_string()).to_string(), "custom");
    }

    #[test]
    fn test_change_summary() {
        // Test generating summaries for different change types
        let feature = Change::new("pkg", ChangeType::Feature, "Add new feature", false);
        assert_eq!(feature.summary(), "feature: Add new feature");

        let breaking_fix = Change::new("pkg", ChangeType::Fix, "Fix critical bug", true);
        assert_eq!(breaking_fix.summary(), "fix!: Fix critical bug");

        let custom = Change::new(
            "pkg",
            ChangeType::Custom("security".to_string()),
            "Fix vulnerability",
            false,
        );
        assert_eq!(custom.summary(), "security: Fix vulnerability");
    }
}
