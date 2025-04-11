#[cfg(test)]
mod changeset_tests {
    use sublime_monorepo_tools::{Change, ChangeId, ChangeType, Changeset};

    #[test]
    fn test_changeset_creation() {
        // Create changes
        let change1 =
            Change::new("@scope/package-foo", ChangeType::Feature, "Add button component", false);
        let change2 =
            Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation error", false);

        // Create changeset with summary
        let changeset_with_summary = Changeset::new(
            Some("PR #123: UI and API improvements"),
            vec![change1.clone(), change2.clone()],
        );

        // Verify fields
        assert_eq!(
            changeset_with_summary.summary,
            Some("PR #123: UI and API improvements".to_string())
        );
        assert_eq!(changeset_with_summary.changes.len(), 2);

        // Create changeset without summary
        let changeset_no_summary = Changeset::new::<String>(None, vec![change1, change2]);
        assert_eq!(changeset_no_summary.summary, None);
        assert_eq!(changeset_no_summary.changes.len(), 2);
    }

    #[test]
    fn test_changeset_is_released() {
        // Create unreleased change
        let unreleased =
            Change::new("@scope/package-foo", ChangeType::Feature, "Add button", false);

        // Create released change
        let released = Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation", false)
            .with_release_version("1.0.0");

        // Changeset with all released changes
        let all_released = Changeset::new::<String>(None, vec![released.clone()]);
        assert!(all_released.is_released());

        // Changeset with mixed release status
        let mixed = Changeset::new::<String>(None, vec![unreleased.clone(), released]);
        assert!(!mixed.is_released());

        // Empty changeset
        let empty = Changeset::new::<String>(None, vec![]);
        assert!(!empty.is_released());
    }

    #[test]
    fn test_changeset_package_names() {
        // Create changes for multiple packages
        let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add button", false);
        let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation", false);
        let change3 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix styling", false);

        // Create changeset with changes
        let changeset = Changeset::new::<String>(None, vec![change1, change2, change3]);

        // Get unique package names
        let packages = changeset.package_names();
        assert_eq!(packages.len(), 2);

        // Verify specific packages are included (order is not guaranteed)
        assert!(packages.contains(&"@scope/package-foo".to_string()));
        assert!(packages.contains(&"@scope/package-bar".to_string()));
    }

    #[test]
    fn test_changeset_with_custom_id() {
        // Create a changeset with a specific ID
        let id = ChangeId::new();
        let id_str = id.to_string();

        // Create changes
        let change = Change::new("@scope/package-baz", ChangeType::Feature, "New feature", false);

        // Currently the API doesn't allow setting a custom ID, so we can't test this directly
        // This is a limitation of the current API

        // Instead, just verify that a valid ID was generated
        let changeset = Changeset::new::<String>(None, vec![change]);
        assert!(!changeset.id.to_string().is_empty());
        assert_ne!(changeset.id.to_string(), id_str); // Should be different from our custom one
    }
}
