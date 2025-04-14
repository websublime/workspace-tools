mod fixtures;

#[allow(clippy::unnecessary_wraps)]
#[cfg(test)]
mod workspace_creation_tests {
    use crate::fixtures::{bun_monorepo, npm_monorepo, pnpm_monorepo, yarn_monorepo};
    use rstest::*;
    use std::path::PathBuf;
    use sublime_monorepo_tools::{
        DiscoveryOptions, Workspace, WorkspaceConfig, WorkspaceError, WorkspaceManager,
    };
    use tempfile::TempDir;

    #[rstest]
    fn test_workspace_creation_with_config(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_path = npm_monorepo.path();

        // Create basic workspace config
        let config = WorkspaceConfig::new(root_path.to_path_buf());

        // Create workspace
        let git_repo = sublime_git_tools::Repo::open(root_path.to_str().unwrap())?;
        let mut workspace = Workspace::new(root_path.to_path_buf(), config, Some(git_repo))?;

        // At this point, no packages should be discovered yet
        assert!(workspace.is_empty());

        // Root path should be set correctly
        assert_eq!(workspace.root_path(), root_path);

        // Git repo should be available
        assert!(workspace.git_repo().is_some());

        // Default package manager should be None
        assert!(workspace.package_manager().is_none());

        // Now discover packages
        let options = DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]);
        let workspace = workspace.discover_packages_with_options(&options)?;

        // Now packages should be discovered
        assert!(!workspace.is_empty());

        // We should have at least the expected packages
        let expected_packages = [
            "@scope/package-foo",
            "@scope/package-bar",
            "@scope/package-baz",
            "@scope/package-charlie",
            "@scope/package-major",
            "@scope/package-tom",
        ];

        for pkg_name in expected_packages {
            let package = workspace.get_package(pkg_name);
            assert!(package.is_some(), "Package {pkg_name} should be discovered");
        }

        Ok(())
    }

    #[rstest]
    #[case(None)]
    #[case(Some("npm"))]
    #[case(Some("yarn"))]
    #[case(Some("pnpm"))]
    #[case(Some("bun"))]
    fn test_workspace_config_with_package_manager(
        #[case] package_manager: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create config with package manager
        let config =
            WorkspaceConfig::new(PathBuf::from("/tmp")).with_package_manager(package_manager);

        // Verify the package manager is set correctly
        assert_eq!(config.package_manager.as_deref(), package_manager);

        // Test with_packages method
        let pkg_patterns = vec!["packages/*", "apps/*"];
        let config = config.with_packages(pkg_patterns.clone());

        assert_eq!(config.packages.len(), pkg_patterns.len());
        for (idx, pattern) in pkg_patterns.iter().enumerate() {
            assert_eq!(config.packages[idx], *pattern);
        }

        // Test with_config method
        let config = config.with_config("npmClient", "yarn").with_config("useWorkspaces", true);

        assert!(config.config.contains_key("npmClient"));
        assert!(config.config.contains_key("useWorkspaces"));

        assert_eq!(config.config["npmClient"], serde_json::json!("yarn"));
        assert_eq!(config.config["useWorkspaces"], serde_json::json!(true));

        Ok(())
    }

    #[rstest]
    fn test_workspace_with_different_package_managers(
        npm_monorepo: TempDir,
        yarn_monorepo: TempDir,
        pnpm_monorepo: TempDir,
        bun_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Function to create and test a workspace with a package manager
        let test_workspace_with_pm = |temp_dir: &TempDir,
                                      expected_pm_name: &str|
         -> Result<(), Box<dyn std::error::Error>> {
            let manager = WorkspaceManager::new();
            let options = DiscoveryOptions::new()
                .detect_package_manager(true)
                .include_patterns(vec!["packages/*/package.json"]);

            let workspace = manager.discover_workspace(temp_dir.path(), &options)?;

            // Check package manager detection
            let pm = workspace.package_manager();
            assert!(pm.is_some(), "Package manager should be detected");

            // Convert package manager to string and check
            let pm_str = pm.as_ref().map(|pm| pm.to_string().to_lowercase()).unwrap_or_default();
            assert_eq!(
                pm_str, expected_pm_name,
                "Expected package manager {expected_pm_name} but got {pm_str}"
            );

            Ok(())
        };

        // Test each package manager
        test_workspace_with_pm(&npm_monorepo, "npm")?;
        test_workspace_with_pm(&yarn_monorepo, "yarn")?;
        test_workspace_with_pm(&pnpm_monorepo, "pnpm")?;
        test_workspace_with_pm(&bun_monorepo, "bun")?;

        Ok(())
    }

    #[rstest]
    fn test_workspace_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Test with WorkspaceManager which has more validation
        let manager = WorkspaceManager::new();

        // Attempt to discover workspace from completely non-existent path
        let nonexistent_path = PathBuf::from("/path/that/definitely/does/not/exist");
        let options = DiscoveryOptions::new();

        // This should fail
        let result = manager.discover_workspace(&nonexistent_path, &options);
        assert!(result.is_err(), "Expected error for non-existent path");

        // Test PackageNotFound error - try to access a non-existent package explicitly
        let temp_dir = tempfile::tempdir()?;
        let config = WorkspaceConfig::new(temp_dir.path().to_path_buf());
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), config, None)?;

        // Create a new empty workspace
        let non_existent_package = "non-existent-package";

        // Try to get a package that doesn't exist
        assert!(workspace.get_package(non_existent_package).is_none());

        // Test error types with as_ref() method
        let error_map = [
            (WorkspaceError::RootNotFound, "RootNotFound"),
            (WorkspaceError::NoPackagesFound(PathBuf::from("/")), "NoPackagesFound"),
            (WorkspaceError::PackageNotFound("test".to_string()), "PackageNotFound"),
            (WorkspaceError::InvalidConfiguration("test".to_string()), "InvalidConfiguration"),
            (WorkspaceError::CycleDetected("test".to_string()), "CycleDetected"),
        ];

        for (err, expected_ref) in error_map {
            assert_eq!(err.as_ref(), expected_ref);
            // Also check that Display is implemented
            assert!(!err.to_string().is_empty());
        }

        Ok(())
    }
}
