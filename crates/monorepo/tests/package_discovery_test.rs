mod fixtures;

#[cfg(test)]
mod package_discovery_tests {
    use crate::fixtures::npm_monorepo;
    use rstest::*;
    use sublime_git_tools::Repo;
    use sublime_monorepo_tools::{DiscoveryOptions, Workspace, WorkspaceConfig, WorkspaceManager};
    use tempfile::TempDir;

    #[rstest]
    fn test_discovery_with_pattern_options(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();

        // Create basic workspace config
        let config = WorkspaceConfig::new(root_path.to_path_buf());
        let git_repo = Repo::open(root_path.to_str().unwrap())?;

        // Create workspace
        let mut workspace =
            Workspace::new(root_path.to_path_buf(), config, Some(git_repo.clone()))?;

        // Test discovery with different include patterns

        // 1. Only discover package-foo
        let foo_options =
            DiscoveryOptions::new().include_patterns(vec!["packages/package-foo/package.json"]);

        workspace.discover_packages_with_options(&foo_options)?;
        assert!(!workspace.is_empty());
        assert!(workspace.get_package("@scope/package-foo").is_some());
        assert!(workspace.get_package("@scope/package-bar").is_none());

        // 2. Only discover package-bar and package-baz
        let bar_baz_options = DiscoveryOptions::new().include_patterns(vec![
            "packages/package-bar/package.json",
            "packages/package-baz/package.json",
        ]);

        // Create a new workspace since we can't clear the old one
        let mut workspace = Workspace::new(
            root_path.to_path_buf(),
            WorkspaceConfig::new(root_path.to_path_buf()),
            Some(git_repo),
        )?;

        workspace.discover_packages_with_options(&bar_baz_options)?;
        assert!(!workspace.is_empty());
        assert!(workspace.get_package("@scope/package-foo").is_none());
        assert!(workspace.get_package("@scope/package-bar").is_some());
        assert!(workspace.get_package("@scope/package-baz").is_some());

        Ok(())
    }

    #[rstest]
    fn test_discovery_with_exclude_patterns(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();

        // Create basic workspace config
        let config = WorkspaceConfig::new(root_path.to_path_buf());
        let git_repo = Repo::open(root_path.to_str().unwrap())?;

        // Create workspace
        let mut workspace = Workspace::new(root_path.to_path_buf(), config, Some(git_repo))?;

        // Test discovery with exclude patterns
        let options = DiscoveryOptions::new()
            .include_patterns(vec!["packages/*/package.json"])
            .exclude_patterns(vec!["**/package-foo/**", "**/package-bar/**"]);

        workspace.discover_packages_with_options(&options)?;
        assert!(!workspace.is_empty());

        // These should be excluded
        assert!(workspace.get_package("@scope/package-foo").is_none());
        assert!(workspace.get_package("@scope/package-bar").is_none());

        // These should be included
        assert!(workspace.get_package("@scope/package-baz").is_some());
        assert!(workspace.get_package("@scope/package-charlie").is_some());
        assert!(workspace.get_package("@scope/package-major").is_some());
        assert!(workspace.get_package("@scope/package-tom").is_some());

        Ok(())
    }

    #[rstest]
    fn test_discovery_with_max_depth(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();

        // Create basic workspace config
        let config = WorkspaceConfig::new(root_path.to_path_buf());
        let git_repo = Repo::open(root_path.to_str().unwrap())?;

        // Create workspace
        let mut workspace = Workspace::new(root_path.to_path_buf(), config, Some(git_repo))?;

        // Test discovery with max_depth=1 (should find nothing)
        let shallow_options =
            DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]).max_depth(1);

        // This should create an empty workspace because the depth isn't enough to reach packages/X/package.json
        let result = workspace.discover_packages_with_options(&shallow_options);

        // It shouldn't fail, but should have no packages
        assert!(result.is_ok());
        assert!(workspace.is_empty());

        // Now try with sufficient depth
        let deep_options =
            DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]).max_depth(3); // This should be enough to reach packages/X/package.json

        workspace.discover_packages_with_options(&deep_options)?;
        assert!(!workspace.is_empty());

        // Verify packages were found
        let expected_packages = [
            "@scope/package-foo",
            "@scope/package-bar",
            "@scope/package-baz",
            "@scope/package-charlie",
            "@scope/package-major",
            "@scope/package-tom",
        ];

        for pkg_name in expected_packages {
            assert!(
                workspace.get_package(pkg_name).is_some(),
                "Package {pkg_name} should be found"
            );
        }

        Ok(())
    }

    #[rstest]
    fn test_workspace_discovery_priority(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();
        let git_repo = Repo::open(root_path.to_str().unwrap())?;

        // SCENARIO 1: Workspace config provides packages - these should take priority
        let config_with_packages =
            WorkspaceConfig::new(root_path.to_path_buf()).with_packages(vec![
                "packages/package-foo/package.json", // Only include foo and bar
                "packages/package-bar/package.json",
            ]);

        let mut workspace =
            Workspace::new(root_path.to_path_buf(), config_with_packages, Some(git_repo.clone()))?;

        // Even with default discovery options (which include "**/package.json"),
        // the workspace config patterns should take priority
        let default_options = DiscoveryOptions::default();
        workspace.discover_packages_with_options(&default_options)?;

        // Should only find foo and bar (from config), not baz or others
        assert!(!workspace.is_empty(), "Workspace should have packages from config");
        assert!(
            workspace.get_package("@scope/package-foo").is_some(),
            "Should find package-foo from config"
        );
        assert!(
            workspace.get_package("@scope/package-bar").is_some(),
            "Should find package-bar from config"
        );
        assert!(
            workspace.get_package("@scope/package-baz").is_none(),
            "Should NOT find package-baz (not in config)"
        );
        assert!(
            workspace.get_package("@scope/package-charlie").is_none(),
            "Should NOT find package-charlie (not in config)"
        );

        // SCENARIO 2: Empty config but discovery options provided - should use options
        let empty_config = WorkspaceConfig::new(root_path.to_path_buf());
        // Ensure packages array is empty (should be by default)
        assert!(empty_config.packages.is_empty(), "Config should have empty packages");

        let mut workspace =
            Workspace::new(root_path.to_path_buf(), empty_config, Some(git_repo.clone()))?;

        // Use custom options that only include specific packages
        let specific_options = DiscoveryOptions::new().include_patterns(vec![
            "packages/package-charlie/package.json",
            "packages/package-major/package.json",
        ]);

        workspace.discover_packages_with_options(&specific_options)?;

        // Should find only charlie and major (from options)
        assert!(!workspace.is_empty(), "Workspace should have packages from options");
        assert!(
            workspace.get_package("@scope/package-foo").is_none(),
            "Should NOT find package-foo"
        );
        assert!(
            workspace.get_package("@scope/package-bar").is_none(),
            "Should NOT find package-bar"
        );
        assert!(
            workspace.get_package("@scope/package-charlie").is_some(),
            "Should find package-charlie from options"
        );
        assert!(
            workspace.get_package("@scope/package-major").is_some(),
            "Should find package-major from options"
        );

        // SCENARIO 3: Both config and options empty - should use default pattern
        let empty_config = WorkspaceConfig::new(root_path.to_path_buf());
        // Ensure packages array is empty
        assert!(empty_config.packages.is_empty(), "Config should have empty packages");

        let mut workspace = Workspace::new(root_path.to_path_buf(), empty_config, Some(git_repo))?;

        // Create discovery options with empty include patterns
        let empty_options = DiscoveryOptions::new().include_patterns(Vec::<String>::new());

        // Verify options are actually empty
        assert!(
            empty_options.include_patterns.is_empty(),
            "Options should have empty include_patterns"
        );

        // Discover packages - should use default pattern
        workspace.discover_packages_with_options(&empty_options)?;

        // Should find all packages with default pattern
        assert!(!workspace.is_empty(), "Workspace should have all packages with default pattern");
        let all_packages = [
            "@scope/package-foo",
            "@scope/package-bar",
            "@scope/package-baz",
            "@scope/package-charlie",
            "@scope/package-major",
            "@scope/package-tom",
        ];

        // Check if all packages were found
        for pkg_name in all_packages {
            assert!(
                workspace.get_package(pkg_name).is_some(),
                "Should find {pkg_name} with default pattern"
            );
        }

        Ok(())
    }

    #[rstest]
    fn test_workspace_package_config_patterns(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();

        // Create a workspace config with package patterns - using the full path with package.json
        let config = WorkspaceConfig::new(root_path.to_path_buf()).with_packages(vec![
            "packages/package-foo/package.json",
            "packages/package-bar/package.json",
        ]);

        let git_repo = sublime_git_tools::Repo::open(root_path.to_str().unwrap())?;

        // FIRST TEST: Using default options (which contains "**/package.json")
        // Create workspace
        let mut workspace =
            Workspace::new(root_path.to_path_buf(), config.clone(), Some(git_repo.clone()))?;
        let default_options = DiscoveryOptions::default();

        workspace.discover_packages_with_options(&default_options)?;

        // With the fixed behavior, this should ONLY find foo and bar (from config)
        // despite default options having "**/package.json"
        assert!(!workspace.is_empty(), "Workspace should have packages from config");
        assert!(
            workspace.get_package("@scope/package-foo").is_some(),
            "Should find package-foo from config"
        );
        assert!(
            workspace.get_package("@scope/package-bar").is_some(),
            "Should find package-bar from config"
        );
        assert!(
            workspace.get_package("@scope/package-baz").is_none(),
            "Should NOT find package-baz (not in config)"
        );
        assert!(
            workspace.get_package("@scope/package-charlie").is_none(),
            "Should NOT find package-charlie (not in config)"
        );

        // SECOND TEST: Using custom specific options
        // Create a new workspace
        let mut workspace = Workspace::new(root_path.to_path_buf(), config, Some(git_repo))?;

        // Use custom options with patterns that would find package-baz
        let custom_options = DiscoveryOptions::new().include_patterns(vec![
            "packages/package-baz/package.json".to_string(),
            "packages/package-charlie/package.json".to_string(),
        ]);

        workspace.discover_packages_with_options(&custom_options)?;

        // Even with these custom patterns, config should still take priority
        // so we should ONLY see foo and bar
        assert!(!workspace.is_empty(), "Workspace should have packages from config");
        assert!(
            workspace.get_package("@scope/package-foo").is_some(),
            "Should find package-foo from config"
        );
        assert!(
            workspace.get_package("@scope/package-bar").is_some(),
            "Should find package-bar from config"
        );
        assert!(
            workspace.get_package("@scope/package-baz").is_none(),
            "Should NOT find package-baz despite options pattern"
        );
        assert!(
            workspace.get_package("@scope/package-charlie").is_none(),
            "Should NOT find package-charlie despite options pattern"
        );

        Ok(())
    }

    #[rstest]
    fn test_workspace_manager_discovery(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();
        let manager = WorkspaceManager::new();

        // Test with default options
        let default_options = DiscoveryOptions::default();
        let workspace = manager.discover_workspace(root_path, &default_options)?;

        // Should find all packages
        assert!(!workspace.is_empty());

        // Test with custom options
        let custom_options = DiscoveryOptions::new()
            .include_patterns(vec!["packages/package-foo/package.json"])
            .auto_detect_root(true)
            .detect_package_manager(true);

        let workspace = manager.discover_workspace(root_path, &custom_options)?;

        // Should only find package-foo
        assert!(!workspace.is_empty());
        assert!(workspace.get_package("@scope/package-foo").is_some());
        assert!(workspace.get_package("@scope/package-bar").is_none());

        // Should have detected npm as package manager
        assert!(workspace.package_manager().is_some());
        assert_eq!(workspace.package_manager().unwrap().to_string().to_lowercase(), "npm");

        Ok(())
    }
}
