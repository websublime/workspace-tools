mod fixtures;

#[cfg(test)]
mod tests {
    use crate::fixtures::{cycle_monorepo, npm_monorepo};
    use rstest::*;
    use std::rc::Rc;
    use sublime_monorepo_tools::{
        BumpType, Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore,
        VersionBumpStrategy, VersionManager, Workspace, WorkspaceConfig,
    };
    use tempfile::TempDir;

    // Helper to create a workspace from fixture
    fn setup_workspace(temp_dir: &TempDir) -> Result<Workspace, Box<dyn std::error::Error>> {
        let repo_path = temp_dir.path();

        // Create workspace configuration
        let config = WorkspaceConfig::new(repo_path.to_path_buf());

        // Open Git repo from the fixture
        let git_repo = Some(sublime_git_tools::Repo::open(repo_path.to_str().unwrap())?);

        // Create workspace with the Git repo
        let mut workspace = Workspace::new(repo_path.to_path_buf(), config, git_repo)?;

        // Discover packages
        let options = DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]);
        workspace.discover_packages_with_options(&options)?;

        Ok(workspace)
    }

    #[rstest]
    fn test_version_manager_creation(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Create version manager without change tracker
        let _manager_no_tracker = VersionManager::new(&workspace, None);

        // The VersionManager should be created successfully
        // Just verify that we can call a basic method
        assert_eq!(manager.get_cycle_groups(), workspace.get_circular_dependencies());

        Ok(())
    }

    #[rstest]
    fn test_suggest_bumps(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for different packages
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Define Independent strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Test suggest_bumps method
        let suggestions = manager.suggest_bumps(&strategy)?;

        // Verify we got suggestions for our changed packages
        assert!(
            suggestions.contains_key("@scope/package-foo"),
            "Should have suggestion for package-foo"
        );
        assert!(
            suggestions.contains_key("@scope/package-bar"),
            "Should have suggestion for package-bar"
        );

        // Verify correct bump types
        assert_eq!(
            suggestions["@scope/package-foo"].bump_type,
            BumpType::Major,
            "Breaking change should get major bump"
        );

        assert_eq!(
            suggestions["@scope/package-bar"].bump_type,
            BumpType::Minor,
            "Feature should get minor bump"
        );

        // Test suggest_bumps_with_options method
        let suggestions_no_cycles = manager.suggest_bumps_with_options(&strategy, false)?;

        // Should still have the same base suggestions
        assert!(suggestions_no_cycles.contains_key("@scope/package-foo"));
        assert!(suggestions_no_cycles.contains_key("@scope/package-bar"));

        // Error case - when no change tracker
        let manager_no_tracker = VersionManager::new(&workspace, None);
        let result = manager_no_tracker.suggest_bumps(&strategy);
        assert!(result.is_err(), "suggest_bumps should fail without a change tracker");

        Ok(())
    }

    #[rstest]
    fn test_preview_bumps(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for different packages
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Define Independent strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Test preview_bumps method
        let preview = manager.preview_bumps(&strategy)?;

        // Verify preview contents
        assert!(!preview.changes.is_empty(), "Preview should have changes");

        // Find suggestions for our changed packages
        let foo_suggestion =
            preview.changes.iter().find(|s| s.package_name == "@scope/package-foo");
        let bar_suggestion =
            preview.changes.iter().find(|s| s.package_name == "@scope/package-bar");

        assert!(foo_suggestion.is_some(), "Should have suggestion for package-foo");
        assert!(bar_suggestion.is_some(), "Should have suggestion for package-bar");

        // Check bump types
        assert_eq!(
            foo_suggestion.unwrap().bump_type,
            BumpType::Major,
            "Breaking change should get major bump"
        );

        assert_eq!(
            bar_suggestion.unwrap().bump_type,
            BumpType::Minor,
            "Feature should get minor bump"
        );

        // Our fixture does have cycles between package-bar and package-baz
        assert!(preview.cycle_detected, "Cycles should be detected in this fixture");
        assert!(!preview.cycle_groups.is_empty(), "Cycle groups should be present");

        Ok(())
    }

    #[rstest]
    fn test_apply_bumps(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for different packages
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define Independent strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Declare version_changes outside the scope
        let version_changes;

        {
            // Create version manager in a new scope
            let manager = VersionManager::new(&workspace, Some(&tracker));

            // Test dry run first
            let dry_run_changes = manager.apply_bumps(&strategy, true)?;

            assert!(!dry_run_changes.is_empty(), "Should have version changes");

            // Verify package versions haven't changed (dry run)
            for change in &dry_run_changes {
                let pkg_info =
                    workspace.get_package(&change.package_name).expect("Package should exist");

                let current_version = pkg_info.borrow().package.borrow().version_str();
                assert_eq!(
                    current_version, change.previous_version,
                    "Version shouldn't change during dry run"
                );
            }

            // Now apply changes for real
            version_changes = manager.apply_bumps(&strategy, false)?;

            assert!(!version_changes.is_empty(), "Should have version changes");

            // Verify package versions have changed
            for change in &version_changes {
                let pkg_info =
                    workspace.get_package(&change.package_name).expect("Package should exist");

                let new_version = pkg_info.borrow().package.borrow().version_str();
                assert_eq!(
                    new_version, change.new_version,
                    "Version should be updated after applying changes"
                );

                // Check if version change has correct bump type
                match change.package_name.as_str() {
                    "@scope/package-foo" => assert_eq!(
                        change.bump_type,
                        BumpType::Major,
                        "Package foo should get major bump"
                    ),
                    "@scope/package-bar" => assert_eq!(
                        change.bump_type,
                        BumpType::Minor,
                        "Package bar should get minor bump"
                    ),
                    _ => {} // Other packages may get dependency updates
                }
            }

            // End the scope, dropping the VersionManager and any borrows it holds
        }

        // Now we can borrow tracker mutably
        VersionManager::mark_changes_as_released(&mut tracker, &version_changes, false)?;

        // Verify changes are marked as released
        let unreleased = tracker.unreleased_changes()?;

        assert!(
            !unreleased.contains_key("@scope/package-foo"),
            "Package foo changes should be released"
        );
        assert!(
            !unreleased.contains_key("@scope/package-bar"),
            "Package bar changes should be released"
        );

        Ok(())
    }

    #[rstest]
    fn test_validate_versions(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace and manager
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Test validate_versions method
        let validation = manager.validate_versions()?;

        // Our fixture does have cycles between package-bar and package-baz
        assert!(validation.has_cycles, "Fixture should have cycles");

        // There might be inconsistencies in test data - just ensure we can process them
        for inconsistency in &validation.inconsistencies {
            // Verify inconsistency structure
            assert!(!inconsistency.package_name.is_empty(), "Package name should be set");
            assert!(!inconsistency.dependency_name.is_empty(), "Dependency name should be set");
            assert!(!inconsistency.required_version.is_empty(), "Required version should be set");
            assert!(!inconsistency.actual_version.is_empty(), "Actual version should be set");
        }

        Ok(())
    }

    #[rstest]
    fn test_cycle_detection(cycle_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace with cycle fixture
        let workspace = setup_workspace(&cycle_monorepo)?;

        // Create version manager (no changes needed)
        let store = Box::new(MemoryChangeStore::new());
        let tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Test has_cycles method
        assert!(manager.has_cycles(), "Cycle fixture should have cycles");

        // Test get_cycle_groups
        let cycles = manager.get_cycle_groups();
        assert!(!cycles.is_empty(), "Should have cycle groups");

        // Test visualize_cycles
        let visualization = manager.visualize_cycles();
        assert!(
            visualization.contains("Circular Dependencies:"),
            "Should have visualization title"
        );
        assert!(visualization.contains("Cycle Group"), "Should list cycle groups");

        // Validate versions should detect cycles
        let validation = manager.validate_versions()?;
        assert!(validation.has_cycles, "Validation should detect cycles");

        Ok(())
    }

    #[rstest]
    fn test_generate_version_report(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for different packages
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Apply version bumps
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let version_changes = manager.apply_bumps(&strategy, true)?;

        // Generate report
        let report = manager.generate_version_report(&version_changes);

        // Report should be a non-empty string
        assert!(!report.is_empty(), "Report should not be empty");

        // Report should contain our packages
        assert!(report.contains("@scope/package-foo"), "Report should mention package-foo");
        assert!(report.contains("@scope/package-bar"), "Report should mention package-bar");

        // Report should have sections
        assert!(report.contains("Summary:"), "Report should have summary");

        Ok(())
    }
}
