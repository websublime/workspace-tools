mod test_utils;

use chrono::Utc;
use std::collections::HashSet;
use sublime_monorepo_tools::{Change, ChangeType, Changeset};

#[cfg(test)]
mod changes_changeset_tests {
    use super::*;

    #[test]
    fn test_changeset_creation() {
        // Create a changeset with no changes
        let empty_changeset = Changeset::new::<String>(None, vec![]);

        // Verify properties
        assert!(empty_changeset.changes.is_empty());
        assert!(empty_changeset.summary.is_none());

        // Timestamp should be recent
        let now = Utc::now();
        let diff = now.timestamp() - empty_changeset.created_at.timestamp();
        assert!(diff < 5, "Changeset timestamp should be recent");
    }

    #[test]
    fn test_changeset_with_changes() {
        // Create some changes
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 = Change::new("pkg-b", ChangeType::Fix, "Fix B", false);

        // Create a changeset with these changes
        let changeset = Changeset::new(Some("Test changeset"), vec![change1, change2]);

        // Verify properties
        assert_eq!(changeset.changes.len(), 2);
        assert_eq!(changeset.summary, Some("Test changeset".to_string()));

        // Check package names
        let package_names = changeset.package_names();
        assert_eq!(package_names.len(), 2);
        let package_set: HashSet<_> = package_names.into_iter().collect();
        assert!(package_set.contains("pkg-a"));
        assert!(package_set.contains("pkg-b"));
    }

    #[test]
    fn test_changeset_release_status() {
        // Create changes with mixed release status
        let unreleased = Change::new("pkg-a", ChangeType::Feature, "Unreleased", false);
        let released =
            Change::new("pkg-b", ChangeType::Fix, "Released", false).with_release_version("1.0.0");

        // Changeset with all unreleased changes
        let unreleased_changeset = Changeset::new::<String>(None, vec![unreleased.clone()]);
        assert!(!unreleased_changeset.is_released());

        // Changeset with all released changes
        let released_changeset = Changeset::new::<String>(None, vec![released.clone()]);
        assert!(released_changeset.is_released());

        // Changeset with mixed release status
        let mixed_changeset = Changeset::new::<String>(None, vec![unreleased, released]);
        assert!(!mixed_changeset.is_released());

        // Empty changeset (should not be considered released)
        let empty_changeset = Changeset::new::<String>(None, vec![]);
        assert!(!empty_changeset.is_released());
    }

    #[test]
    fn test_changeset_package_names() {
        // Multiple changes for the same package
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature 1", false);
        let change2 = Change::new("pkg-a", ChangeType::Feature, "Feature 2", false);
        let change3 = Change::new("pkg-b", ChangeType::Fix, "Fix", false);

        let changeset = Changeset::new::<String>(None, vec![change1, change2, change3]);

        // Should deduplicate package names
        let package_names = changeset.package_names();
        assert_eq!(package_names.len(), 2);

        // Set should contain both packages
        let package_set: HashSet<_> = package_names.into_iter().collect();
        assert!(package_set.contains("pkg-a"));
        assert!(package_set.contains("pkg-b"));
    }
}
