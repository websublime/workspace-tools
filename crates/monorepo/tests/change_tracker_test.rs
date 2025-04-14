mod fixtures;

#[cfg(test)]
mod change_tracker_tests {
    use rstest::*;
    use std::fs::File;
    use std::io::Write;
    use std::rc::Rc;
    use sublime_monorepo_tools::{
        Change, ChangeError, ChangeScope, ChangeTracker, ChangeType, DiscoveryOptions,
        MemoryChangeStore, Workspace, WorkspaceConfig,
    };
    use tempfile::TempDir;

    // Import fixtures directly
    use crate::fixtures::{
        bun_cycle_monorepo, npm_cycle_monorepo, npm_monorepo, pnpm_cycle_monorepo,
        yarn_cycle_monorepo,
    };

    // Helper to create a workspace from a monorepo fixture
    fn setup_workspace(temp_dir: &TempDir) -> Result<Rc<Workspace>, Box<dyn std::error::Error>> {
        let repo_path = temp_dir.path();

        // Create workspace configuration
        let config = WorkspaceConfig::new(repo_path.to_path_buf());

        // Create workspace
        let mut workspace = Workspace::new(repo_path.to_path_buf(), config, None)?;

        // Discover packages
        let options = DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]);

        // Since discover_packages_with_options returns &Self, we need to use it as a method call
        // that mutates the workspace instance instead of reassigning
        workspace.discover_packages_with_options(&options)?;

        // Now return the Rc<Workspace>
        Ok(Rc::new(workspace))
    }

    #[rstest]
    fn test_change_tracker_init(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store
        let store = Box::new(MemoryChangeStore::new());

        // Create tracker
        let tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Basic checks
        assert_eq!(tracker.get_workspace_root_path(), workspace.root_path());

        Ok(())
    }

    #[rstest]
    fn test_map_file_to_scope(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Test mapping files to scopes

        // Package files - using the actual package names from fixtures
        let scope1 = tracker.map_file_to_scope("packages/package-foo/package.json")?;
        assert!(matches!(scope1, ChangeScope::Package(name) if name == "@scope/package-foo"));

        let scope2 = tracker.map_file_to_scope("packages/package-bar/index.mjs")?;
        assert!(matches!(scope2, ChangeScope::Package(name) if name == "@scope/package-bar"));

        // Root level file
        let scope3 = tracker.map_file_to_scope("package.json")?;
        assert!(matches!(scope3, ChangeScope::Root));

        // Monorepo file (not in package, not at root)
        let scope4 = tracker.map_file_to_scope("scripts/some-script.js")?;
        assert!(matches!(scope4, ChangeScope::Monorepo));

        // Test cache clearing
        tracker.clear_cache();

        Ok(())
    }

    #[rstest]
    fn test_record_change(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Record a change for an existing package - using package name from fixtures
        let change = Change::new(
            "@scope/package-foo",
            ChangeType::Feature,
            "Add new button component",
            false,
        );
        tracker.record_change(change.clone())?;

        // Try recording a change for non-existent package
        let invalid_change =
            Change::new("non-existent-package", ChangeType::Feature, "This should fail", false);
        let result = tracker.record_change(invalid_change);
        assert!(matches!(result, Err(ChangeError::InvalidPackage(_))));

        // Verify the change was recorded in store
        let store = tracker.store();
        let changes = store.get_unreleased_changes("@scope/package-foo")?;

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].description, "Add new button component");

        Ok(())
    }

    #[rstest]
    fn test_create_changeset(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create changes - using package names from fixtures
        let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add button", false);
        let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation", false);

        // Create changeset
        let changeset = tracker.create_changeset(
            Some("PR #123: UI and API improvements".to_string()),
            vec![change1, change2],
        )?;

        // Verify changeset
        assert_eq!(changeset.changes.len(), 2);
        assert_eq!(changeset.summary, Some("PR #123: UI and API improvements".to_string()));

        // Verify the changeset was stored
        let store = tracker.store();
        let stored_changeset = store.get_changeset(&changeset.id)?;
        assert!(stored_changeset.is_some());

        Ok(())
    }

    #[rstest]
    fn test_unreleased_changes(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create and record changes - using package names from fixtures
        let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add button", false);
        let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix validation", false);
        let change3 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix styling", false);

        tracker.create_changeset(None, vec![change1, change2, change3])?;

        // Get unreleased changes
        let unreleased = tracker.unreleased_changes()?;

        // Verify unreleased changes
        assert_eq!(unreleased.len(), 2); // Two packages with changes
        assert_eq!(unreleased["@scope/package-foo"].len(), 2);
        assert_eq!(unreleased["@scope/package-bar"].len(), 1);

        Ok(())
    }

    #[rstest]
    fn test_mark_released(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create and record changes - using package names from fixtures
        let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add button", false);
        let change2 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix styling", false);

        tracker.create_changeset(None, vec![change1, change2])?;

        // Verify initial state
        let initial = tracker.unreleased_changes()?;
        assert_eq!(initial["@scope/package-foo"].len(), 2);

        // Mark changes as released
        let marked = tracker.mark_released("@scope/package-foo", "1.0.0", false)?;
        assert_eq!(marked.len(), 2);

        // Verify changes are now released
        let after = tracker.unreleased_changes()?;
        assert!(!after.contains_key("@scope/package-foo"));

        Ok(())
    }

    #[rstest]
    fn test_environment_specific_release(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create environment-specific changes - using package names from fixtures
        let prod_change =
            Change::new("@scope/package-foo", ChangeType::Feature, "Prod feature", false)
                .with_environments(vec!["production"]);

        let staging_change =
            Change::new("@scope/package-foo", ChangeType::Feature, "Staging feature", false)
                .with_environments(vec!["staging"]);

        let all_env_change =
            Change::new("@scope/package-foo", ChangeType::Fix, "All envs fix", false);

        tracker.create_changeset(None, vec![prod_change, staging_change, all_env_change])?;

        // Get unreleased changes for production
        let prod_unreleased = tracker.unreleased_changes_for_environment("production")?;
        assert_eq!(prod_unreleased["@scope/package-foo"].len(), 2); // prod_change + all_env_change

        // Mark changes as released for production only
        tracker.mark_released_for_environment(
            "@scope/package-foo",
            "1.0.0",
            "production",
            false,
        )?;

        // Verify production changes are now released
        let after_prod = tracker.unreleased_changes_for_environment("production")?;
        assert!(!after_prod.contains_key("@scope/package-foo"));

        // But staging changes should still be unreleased
        let after_staging = tracker.unreleased_changes_for_environment("staging")?;
        assert!(after_staging.contains_key("@scope/package-foo"));
        assert_eq!(after_staging["@scope/package-foo"].len(), 1); // Only staging_change

        Ok(())
    }

    #[rstest]
    fn test_specific_change_release(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create changes - using package names from fixtures
        let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Feature A", false);
        let change2 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix B", false);

        tracker.create_changeset(None, vec![change1.clone(), change2.clone()])?;

        // Mark only specific change as released
        let to_mark = vec![change1.id.clone()]; // Only mark the first change

        tracker.mark_specific_changes_as_released(
            "@scope/package-foo",
            "1.0.0",
            &to_mark,
            false,
        )?;

        // Get unreleased changes
        let unreleased = tracker.unreleased_changes()?;

        // Verify only one change is still unreleased
        assert_eq!(unreleased["@scope/package-foo"].len(), 1);
        assert_eq!(unreleased["@scope/package-foo"][0].description, "Fix B");

        // Get released changes from store
        let released = tracker.store().get_released_changes("@scope/package-foo")?;

        // Verify only one change is released
        assert_eq!(released.len(), 1);
        assert_eq!(released[0].description, "Feature A");
        assert_eq!(released[0].release_version, Some("1.0.0".to_string()));

        Ok(())
    }

    #[rstest]
    fn test_generate_changes_report(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(workspace, store);

        // Create and record various change types - using package names from fixtures
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Feature, "Feature A", false),
            Change::new("@scope/package-foo", ChangeType::Fix, "Fix B", false),
            Change::new("@scope/package-bar", ChangeType::Performance, "Optimize C", false),
            Change::new("@scope/package-baz", ChangeType::Breaking, "Breaking D", true),
        ];

        tracker.create_changeset(None, changes)?;

        // Generate report
        let report = tracker.generate_changes_report(false)?;

        // Verify report contains expected content
        assert!(report.contains("Package Changes:"));
        assert!(report.contains("@scope/package-foo:"));
        assert!(report.contains("@scope/package-bar:"));
        assert!(report.contains("@scope/package-baz:"));
        assert!(report.contains("Feature A"));
        assert!(report.contains("Fix B"));
        assert!(report.contains("Optimize C"));
        assert!(report.contains("Breaking D"));
        assert!(report.contains("[BREAKING]")); // Breaking change indicator

        // Generate report with cycle info
        let report_with_cycles = tracker.generate_changes_report(true)?;

        // Basic report validation
        assert!(!report_with_cycles.is_empty());

        Ok(())
    }

    #[rstest]
    fn test_visualize_dependency_graph(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the npm_monorepo fixture
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let tracker = ChangeTracker::new(workspace, store);

        // Generate dependency graph visualization
        let graph = tracker.visualize_dependency_graph(true)?;

        // Basic validation of the visualization
        assert!(graph.contains("Package Dependency Graph:"));
        assert!(!graph.is_empty());

        // Should contain at least some of our packages from the fixtures
        assert!(
            graph.contains("@scope/package-foo")
                || graph.contains("@scope/package-bar")
                || graph.contains("@scope/package-baz")
        );

        Ok(())
    }

    #[rstest]
    #[case::npm(npm_cycle_monorepo())]
    #[case::yarn(yarn_cycle_monorepo())]
    #[case::pnpm(pnpm_cycle_monorepo())]
    #[case::bun(bun_cycle_monorepo())]
    fn test_with_cycle_dependencies(
        #[case] cycle_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace using the cycle_monorepo fixture
        let workspace = setup_workspace(&cycle_monorepo)?;

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Record some changes to make the report more meaningful
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Feature, "Feature for foo", false),
            Change::new("@scope/package-bar", ChangeType::Fix, "Fix for bar", false),
            Change::new("@scope/package-baz", ChangeType::Refactor, "Refactor baz", false),
        ];
        tracker.create_changeset(Some("Changes for cyclic packages".to_string()), changes)?;

        // Generate changes report with cycle info
        let report = tracker.generate_changes_report(true)?;

        // Verify cycle info is included
        if !report.contains("Circular dependency groups detected:") {
            // Look for other cycle indicators
            if report.contains("cycle group") {
                // There are cycles detected, just different wording
                assert!(report.contains("cycle group"), "Report should mention cycle groups");
            } else {
                // No cycles detected
                panic!("Expected cycles were not detected");
            }
        }

        // Generate dependency graph
        let graph = tracker.visualize_dependency_graph(true)?;

        // Verify cycle info is in the visualization
        assert!(
            graph.contains("Cycle") || graph.contains("cycle"),
            "Graph should include cycle information"
        );

        Ok(())
    }

    #[rstest]
    fn test_detect_changes_between(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // npm_monorepo is the result of crate::fixtures::create_complete_monorepo(Some("npm"))
        let repo_path = npm_monorepo.path();

        // Repository should exist in the fixture - open it
        let repo = match sublime_git_tools::Repo::open(repo_path.to_str().unwrap()) {
            Ok(r) => r,
            Err(e) => {
                return Err(e.into());
            }
        };

        // Create workspace configuration
        let config = WorkspaceConfig::new(repo_path.to_path_buf());

        // Create workspace WITH the repo reference
        let mut workspace = Workspace::new(repo_path.to_path_buf(), config, Some(repo.clone()))?;

        // Discover packages
        let options = DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]);
        workspace.discover_packages_with_options(&options)?;

        // Wrap in Rc
        let workspace = Rc::new(workspace);

        // Create memory store and tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create a new branch for testing
        let branch_name = "test-changes-detect";
        repo.create_branch(branch_name)?;
        repo.checkout(branch_name)?;

        // Make a file change in package-foo
        let file_path = repo_path.join("packages/package-foo/new-test-file.js");
        let mut file = File::create(&file_path)?;
        writeln!(file, "// Test file for change detection")?;

        // Add and commit the change
        repo.add_all()?;
        repo.commit("feat: add test file for change detection")?;

        // Go back to main branch
        repo.checkout("main")?;

        // Try to detect changes
        let result = tracker.detect_changes_between(branch_name, None);

        match result {
            Ok(changes) => {
                assert_eq!(changes.len(), 1, "Should detect exactly one change");
                assert_eq!(
                    changes[0].package, "@scope/package-foo",
                    "Change should be for package-foo"
                );
            }
            Err(e) => match e {
                ChangeError::NoGitRepository => {
                    panic!("NoGitRepository error - workspace was not initialized with a Git repo");
                }
                ChangeError::NoChangesFound => {
                    panic!("NoChangesFound error - No changes were detected between the branches");
                }
                _ => {
                    return Err(e.into());
                }
            },
        }

        Ok(())
    }
}
