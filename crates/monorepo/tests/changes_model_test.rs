use std::str::FromStr;

use sublime_monorepo_tools::{Change, ChangeId, ChangeType, Changeset};

mod fixtures;

#[test]
fn test_change_id() {
    // Test creation
    let id = ChangeId::new();
    let id_str = id.to_string();

    // Test parsing
    let parsed_id = ChangeId::from_str(&id_str).expect("Failed to parse Change ID");
    assert_eq!(id.to_string(), parsed_id.to_string());

    // Test default
    let default_id = ChangeId::default();
    assert_ne!(id.to_string(), default_id.to_string()); // Should be different UUIDs
}

#[test]
fn test_change_type() {
    // Test display
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
    assert_eq!(ChangeType::Custom("custom-type".to_string()).to_string(), "custom-type");
    assert_eq!(ChangeType::Unknown.to_string(), "unknown");
    assert_eq!(ChangeType::Breaking.to_string(), "breaking");
}

#[test]
fn test_change() {
    // Create a basic change
    let change = Change::new("@scope/package-foo", ChangeType::Feature, "Add new feature", false);

    // Check basic properties
    assert_eq!(change.package, "@scope/package-foo");
    assert!(matches!(change.change_type, ChangeType::Feature));
    assert_eq!(change.description, "Add new feature");
    assert!(!change.breaking);

    // Check released status
    assert!(!change.is_released());

    // Test summary
    assert_eq!(change.summary(), "feature: Add new feature");

    // Test with breaking
    let breaking_change =
        Change::new("@scope/package-bar", ChangeType::Fix, "Fix critical bug", true);

    // Check summary with breaking
    assert_eq!(breaking_change.summary(), "fix!: Fix critical bug");

    // Test environment targeting
    assert!(change.applies_to_environment("production"));

    let mut env_change =
        Change::new("@scope/package-baz", ChangeType::Feature, "Feature for staging", false);
    env_change = env_change.with_environments(vec!["staging", "development"]);

    // Check environment applicability
    assert!(env_change.applies_to_environment("staging"));
    assert!(env_change.applies_to_environment("development"));
    assert!(!env_change.applies_to_environment("production"));

    // Test with author
    let authored_change =
        Change::new("@scope/package-foo", ChangeType::Feature, "Add feature", false)
            .with_author("Test User");

    assert_eq!(authored_change.author.as_deref(), Some("Test User"));

    // Test with issues
    let issue_change = Change::new("@scope/package-foo", ChangeType::Feature, "Add feature", false)
        .with_issues(vec!["#123", "#456"]);

    assert_eq!(issue_change.issues.len(), 2);
    assert_eq!(issue_change.issues[0], "#123");
    assert_eq!(issue_change.issues[1], "#456");
}

#[test]
fn test_changeset() {
    // Create changes
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Feature 1", false);

    let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix 1", false);

    // Create changeset
    let changeset = Changeset::new(Some("Test changeset".to_string()), vec![change1, change2]);

    // Verify properties
    assert_eq!(changeset.summary.as_deref(), Some("Test changeset"));
    assert_eq!(changeset.changes.len(), 2);

    // Test package names
    let packages = changeset.package_names();
    assert_eq!(packages.len(), 2);
    assert!(packages.contains(&"@scope/package-foo".to_string()));
    assert!(packages.contains(&"@scope/package-bar".to_string()));

    // Test is_released
    assert!(!changeset.is_released());

    // Test with released changes
    let mut released_change =
        Change::new("@scope/package-foo", ChangeType::Feature, "Released feature", false);
    released_change.release_version = Some("1.0.0".to_string());

    let released_changeset = Changeset::new::<std::string::String>(None, vec![released_change]);
    assert!(released_changeset.is_released());
}
