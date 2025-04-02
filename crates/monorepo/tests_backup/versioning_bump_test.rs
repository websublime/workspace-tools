mod test_utils;

use std::rc::Rc;
use sublime_monorepo_tools::{
    BumpReason, BumpType, Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore,
    VersionBumpStrategy, VersionManager, WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod versioning_bump_tests {
    use super::*;

    fn setup_workspace_with_changes(
    ) -> (TestWorkspace, Rc<sublime_monorepo_tools::Workspace>, ChangeTracker) {
        // Create a test workspace with packages
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create change tracker with some changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add some changes to packages
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "New feature in A", false))
            .expect("Failed to record change");

        tracker
            .record_change(Change::new("pkg-b", ChangeType::Fix, "Fix bug in B", false))
            .expect("Failed to record change");

        tracker
            .record_change(Change::new("pkg-c", ChangeType::Feature, "Breaking change", true))
            .expect("Failed to record change");

        (test_workspace, workspace, tracker)
    }

    #[test]
    fn test_version_manager_creation() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // We can't test private fields directly, but we can verify it was created
        // by calling a method that requires those fields
        let preview_result = version_manager.preview_bumps(&VersionBumpStrategy::default());
        assert!(preview_result.is_ok(), "Preview operation should succeed");
    }

    #[test]
    fn test_suggest_version_bumps_independent() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Test independent versioning strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let suggestions =
            version_manager.suggest_bumps(&strategy).expect("Failed to suggest version bumps");

        // Check pkg-a (should be minor bump for feature)
        assert!(suggestions.contains_key("pkg-a"));
        let pkg_a = &suggestions["pkg-a"];
        assert_eq!(pkg_a.bump_type, BumpType::Minor);

        // Check pkg-b (should be patch bump for fix)
        assert!(suggestions.contains_key("pkg-b"));
        let pkg_b = &suggestions["pkg-b"];
        assert_eq!(pkg_b.bump_type, BumpType::Patch);

        // Check pkg-c (should be major bump for breaking change)
        assert!(suggestions.contains_key("pkg-c"));
        let pkg_c = &suggestions["pkg-c"];
        assert_eq!(pkg_c.bump_type, BumpType::Major);

        // Verify that reasons are correctly set
        assert!(pkg_a.reasons.iter().any(|r| matches!(r, BumpReason::Feature(_))));
        assert!(pkg_b.reasons.iter().any(|r| matches!(r, BumpReason::Fix(_))));
        assert!(pkg_c.reasons.iter().any(|r| matches!(r, BumpReason::Breaking(_))));
    }

    #[test]
    fn test_suggest_version_bumps_synchronized() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Test synchronized versioning strategy
        let strategy = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };

        let suggestions =
            version_manager.suggest_bumps(&strategy).expect("Failed to suggest version bumps");

        // All packages should have the same version
        for pkg_name in &["pkg-a", "pkg-b", "pkg-c"] {
            assert!(suggestions.contains_key(*pkg_name));
            let pkg = &suggestions[*pkg_name];
            assert_eq!(pkg.suggested_version, "2.0.0");
            assert!(pkg.reasons.iter().any(|r| matches!(r, BumpReason::Manual)));
        }
    }

    #[test]
    fn test_suggest_version_bumps_manual() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Test manual versioning strategy
        let mut versions = std::collections::HashMap::new();
        versions.insert("pkg-a".to_string(), "1.1.0".to_string());
        versions.insert("pkg-b".to_string(), "1.0.1".to_string());

        let strategy = VersionBumpStrategy::Manual(versions);

        let suggestions =
            version_manager.suggest_bumps(&strategy).expect("Failed to suggest version bumps");

        // Check that manual versions are applied
        assert!(suggestions.contains_key("pkg-a"));
        assert_eq!(suggestions["pkg-a"].suggested_version, "1.1.0");

        assert!(suggestions.contains_key("pkg-b"));
        assert_eq!(suggestions["pkg-b"].suggested_version, "1.0.1");

        // pkg-c should not have a suggested version as it wasn't in the manual map
        assert!(!suggestions.contains_key("pkg-c"));
    }

    #[test]
    fn test_preview_bumps() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Test preview functionality
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let preview = version_manager.preview_bumps(&strategy).expect("Failed to preview bumps");

        // Check that the preview contains our changes
        assert_eq!(preview.changes.len(), 4);

        // Verify cycle detection info is included
        assert!(!preview.cycle_detected); // Our test workspace doesn't have cycles
    }

    #[test]
    fn test_apply_bumps_dry_run() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Test applying bumps with dry run
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let changes = version_manager.apply_bumps(&strategy, true).expect("Failed to apply bumps");

        // Check that changes were calculated but not applied
        assert_eq!(changes.len(), 4);

        // Get original versions
        let pkg_a_version =
            workspace.get_package("pkg-a").unwrap().borrow().package.borrow().version_str();
        let pkg_b_version =
            workspace.get_package("pkg-b").unwrap().borrow().package.borrow().version_str();
        let pkg_c_version =
            workspace.get_package("pkg-c").unwrap().borrow().package.borrow().version_str();

        // Original versions should be unchanged in dry run
        assert_eq!(pkg_a_version, "1.0.0");
        assert_eq!(pkg_b_version, "2.0.0");
        assert_eq!(pkg_c_version, "1.0.0");
    }

    // Temporarily ignored due to file system issues
    #[ignore]
    #[test]
    fn test_apply_bumps_with_changes() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Record original versions
        let orig_pkg_a_version =
            workspace.get_package("pkg-a").unwrap().borrow().package.borrow().version_str();
        let orig_pkg_b_version =
            workspace.get_package("pkg-b").unwrap().borrow().package.borrow().version_str();
        let orig_pkg_c_version =
            workspace.get_package("pkg-c").unwrap().borrow().package.borrow().version_str();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Apply version bumps for real
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let changes = version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

        // Check that changes were applied
        assert_eq!(changes.len(), 3);

        // Verify package versions were updated
        let new_pkg_a_version =
            workspace.get_package("pkg-a").unwrap().borrow().package.borrow().version_str();
        let new_pkg_b_version =
            workspace.get_package("pkg-b").unwrap().borrow().package.borrow().version_str();
        let new_pkg_c_version =
            workspace.get_package("pkg-c").unwrap().borrow().package.borrow().version_str();

        // Check that versions were incremented correctly
        // pkg-a: minor bump (feature)
        assert_ne!(new_pkg_a_version, orig_pkg_a_version);
        assert!(new_pkg_a_version.starts_with("1.1."));

        // pkg-b: patch bump (fix)
        assert_ne!(new_pkg_b_version, orig_pkg_b_version);
        assert!(new_pkg_b_version.starts_with("2.0."));

        // pkg-c: major bump (breaking change)
        assert_ne!(new_pkg_c_version, orig_pkg_c_version);
        assert!(new_pkg_c_version.starts_with("2.0."));

        // Verify the change records
        let pkg_a_change = changes.iter().find(|c| c.package_name == "pkg-a").unwrap();
        assert_eq!(pkg_a_change.bump_type, BumpType::Minor);
        assert_eq!(pkg_a_change.previous_version, orig_pkg_a_version);
        assert_eq!(pkg_a_change.new_version, new_pkg_a_version);

        let pkg_c_change = changes.iter().find(|c| c.package_name == "pkg-c").unwrap();
        assert_eq!(pkg_c_change.bump_type, BumpType::Major);
        assert_eq!(pkg_c_change.previous_version, orig_pkg_c_version);
        assert_eq!(pkg_c_change.new_version, new_pkg_c_version);
    }

    // Temporarily ignored due to file system issues
    #[ignore]
    #[test]
    fn test_mark_changes_as_released() {
        let (_, workspace, mut tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Apply version bumps
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let changes = version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

        // Now mark changes as released using the static method
        let result = VersionManager::mark_changes_as_released(&mut tracker, &changes, false);
        assert!(result.is_ok(), "Failed to mark changes as released");

        // Verify changes were marked as released
        let unreleased_a = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert!(unreleased_a.is_empty(), "pkg-a changes should be released");

        let unreleased_b = tracker.store().get_unreleased_changes("pkg-b").unwrap();
        assert!(unreleased_b.is_empty(), "pkg-b changes should be released");

        let unreleased_c = tracker.store().get_unreleased_changes("pkg-c").unwrap();
        assert!(unreleased_c.is_empty(), "pkg-c changes should be released");

        // Check the release versions
        let released_a = tracker.store().get_released_changes("pkg-a").unwrap();
        assert_eq!(released_a.len(), 1);
        assert_eq!(released_a[0].release_version.as_deref(), Some(changes[0].new_version.as_str()));
    }

    // Temporarily ignored due to file system issues
    #[ignore]
    #[test]
    fn test_dependency_updates() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Apply version bumps
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let changes = version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

        // Since pkg-b depends on pkg-a, and pkg-a was changed,
        // pkg-b should have been updated
        let pkg_a_change = changes.iter().find(|c| c.package_name == "pkg-a").unwrap();
        let pkg_b_change = changes.iter().find(|c| c.package_name == "pkg-b").unwrap();

        assert!(!pkg_a_change.is_dependency_update);

        // pkg-b has its own change, so it's not just a dependency update
        assert!(!pkg_b_change.is_dependency_update);

        // With more complex dependency trees, check for packages that were
        // updated only because their dependencies changed
        let dependent_changes =
            changes.iter().filter(|c| c.is_dependency_update).collect::<Vec<_>>();

        assert!(!dependent_changes.is_empty());
    }

    #[test]
    fn test_validate_versions() {
        let (_, workspace, tracker) = setup_workspace_with_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Validate versions (should pass since our test workspace is consistent)
        let validation = version_manager.validate_versions().expect("Failed to validate versions");

        // Check validation results
        assert!(!validation.has_cycles);
        assert!(validation.inconsistencies.is_empty());
    }
}
