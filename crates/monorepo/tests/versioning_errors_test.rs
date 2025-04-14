mod fixtures;

#[cfg(test)]
mod versioning_errors_tests {
    use std::rc::Rc;

    use crate::fixtures::{
        bun_cycle_monorepo, npm_cycle_monorepo, npm_monorepo, pnpm_cycle_monorepo,
        yarn_cycle_monorepo,
    };
    use rstest::*;
    use sublime_monorepo_tools::{
        ChangelogOptions, DiscoveryOptions, VersionBumpStrategy, VersionManager, VersioningError,
        Workspace, WorkspaceConfig, WorkspaceManager,
    };
    use tempfile::TempDir;

    // Helper to create a workspace
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
    fn test_missing_change_tracker_error(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create VersionManager without change tracker
        let manager = VersionManager::new(&workspace, None);

        // Attempt operations that require a change tracker
        let strategy = VersionBumpStrategy::default();

        // Test suggest_bumps
        let result = manager.suggest_bumps(&strategy);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.as_ref(), "InvalidBumpStrategy");
            assert!(err.to_string().contains("Change tracker required"));
        }

        // Test preview_bumps
        let result = manager.preview_bumps(&strategy);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.as_ref(), "InvalidBumpStrategy");
            assert!(err.to_string().contains("Change tracker required"));
        }

        // Test apply_bumps
        let result = manager.apply_bumps(&strategy, true);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.as_ref(), "InvalidBumpStrategy");
            assert!(err.to_string().contains("Change tracker required"));
        }

        // Test generate_changelogs
        let result = manager.generate_changelogs(&ChangelogOptions::default(), true);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.as_ref(), "InvalidBumpStrategy");
            assert!(err.to_string().contains("Change tracker required"));
        }

        Ok(())
    }

    #[rstest]
    fn test_package_not_found_error(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(sublime_monorepo_tools::MemoryChangeStore::new());
        let tracker = sublime_monorepo_tools::ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create version manager WITH a change tracker this time
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Create a Manual strategy with non-existent packages
        let mut versions = std::collections::HashMap::new();
        versions.insert("non-existent-package".to_string(), "1.0.0".to_string());

        let strategy = VersionBumpStrategy::Manual(versions);

        // Apply the strategy
        let result = manager.apply_bumps(&strategy, true);
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(
                err.as_ref(),
                "PackageNotFound",
                "Expected PackageNotFound error, got: {err}"
            );
            assert!(err.to_string().contains("non-existent-package"));
        }

        Ok(())
    }

    #[rstest]
    #[case::npm(npm_cycle_monorepo())]
    #[case::yarn(yarn_cycle_monorepo())]
    #[case::pnpm(pnpm_cycle_monorepo())]
    #[case::bun(bun_cycle_monorepo())]
    fn test_cyclic_dependencies_error(
        #[case] cycle_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace with cycles
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new()
            .include_patterns(vec!["packages/*/package.json"])
            .auto_detect_root(true);

        let workspace = manager.discover_workspace(cycle_monorepo.path(), &options)?;

        // Create version manager
        let version_manager = VersionManager::new(&workspace, None);

        // Verify it has cycles
        assert!(version_manager.has_cycles());
        let cycle_groups = version_manager.get_cycle_groups();
        assert!(!cycle_groups.is_empty());

        // Get visualization
        let visualization = version_manager.visualize_cycles();
        assert!(visualization.contains("Circular Dependencies:"));
        assert!(visualization.contains("@scope/package-foo"));
        assert!(visualization.contains("@scope/package-bar"));
        assert!(visualization.contains("@scope/package-baz"));

        Ok(())
    }

    #[test]
    fn test_versioning_error_as_ref() {
        // Test that error variants give correct string representations
        let error_map = [
            (
                VersioningError::WorkspaceError(
                    sublime_monorepo_tools::WorkspaceError::RootNotFound,
                ),
                "WorkspaceError",
            ),
            (VersioningError::NoChangesFound("test".to_string()), "NoChangesFound"),
            (VersioningError::InvalidBumpStrategy("test".to_string()), "InvalidBumpStrategy"),
            (VersioningError::PackageNotFound("test".to_string()), "PackageNotFound"),
            (
                VersioningError::NoVersionSuggestion("pkg".to_string(), "reason".to_string()),
                "NoVersionSuggestion",
            ),
            (VersioningError::CyclicDependencies("test".to_string()), "CyclicDependencies"),
            (VersioningError::NoVersionFile("test".to_string()), "NoVersionFile"),
        ];

        for (err, expected_ref) in error_map {
            assert_eq!(err.as_ref(), expected_ref);
            // Also check that Display is implemented
            assert!(!err.to_string().is_empty());
        }
    }
}
