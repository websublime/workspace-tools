mod test_utils;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceConfig, WorkspaceManager};
use test_utils::TestWorkspace;

#[cfg(test)]
mod workspace_discovery_tests {
    use super::*;

    #[test]
    fn test_basic_workspace_discovery() {
        // Create a test workspace with packages
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let result = manager.discover_workspace(test_workspace.path(), &options);
        assert!(result.is_ok(), "Workspace discovery failed: {:?}", result.err());

        let workspace = result.unwrap();

        // Verify the workspace was discovered correctly
        assert_eq!(workspace.root_path(), test_workspace.path());

        // Verify packages were discovered
        let pkg_a = workspace.get_package("pkg-a");
        assert!(pkg_a.is_some(), "Failed to discover pkg-a");

        let pkg_b = workspace.get_package("pkg-b");
        assert!(pkg_b.is_some(), "Failed to discover pkg-b");

        let pkg_c = workspace.get_package("pkg-c");
        assert!(pkg_c.is_some(), "Failed to discover pkg-c");

        let web_app = workspace.get_package("web-app");
        assert!(web_app.is_some(), "Failed to discover web-app");

        // Verify package count
        let all_packages = workspace.sorted_packages();
        assert_eq!(all_packages.len(), 5, "Expected 5 packages, found {}", all_packages.len());
    }

    #[test]
    fn test_workspace_config_loading() {
        // Create a test workspace with a custom structure
        let test_workspace = TestWorkspace::new();
        let path = test_workspace.path();

        // Create custom packages in non-standard locations
        test_workspace.create_package_json("custom/loc1", "custom-pkg-1", "1.0.0", &[]);
        test_workspace.create_package_json("another/place/pkg2", "custom-pkg-2", "1.0.0", &[]);

        // Create a workspace configuration
        let config = WorkspaceConfig::new(path.clone())
            .with_packages(vec!["custom/**/*.json", "another/**/*.json"])
            .with_package_manager(Some("pnpm"));

        // Load the workspace with this config
        let manager = WorkspaceManager::new();
        let result = manager.load_workspace(config);

        assert!(result.is_ok(), "Workspace loading failed: {:?}", result.err());

        let workspace = result.unwrap();

        // Verify custom packages were discovered
        let pkg1 = workspace.get_package("custom-pkg-1");
        assert!(pkg1.is_some(), "Failed to discover custom-pkg-1");

        let pkg2 = workspace.get_package("custom-pkg-2");
        assert!(pkg2.is_some(), "Failed to discover custom-pkg-2");
    }

    #[test]
    fn test_discovery_with_exclusions() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();

        // Create packages, including one in a "tests" directory that should be excluded
        test_workspace.create_package_json("packages/regular-pkg", "regular-pkg", "1.0.0", &[]);
        test_workspace.create_package_json("tests/test-pkg", "test-pkg", "1.0.0", &[]);

        // Create options that exclude the tests directory
        let options = DiscoveryOptions::new().exclude_patterns(vec!["**/tests/**"]);

        // Discover workspace
        let manager = WorkspaceManager::new();
        let result = manager.discover_workspace(test_workspace.path(), &options);
        assert!(result.is_ok());

        let workspace = result.unwrap();

        // Should find the regular package but not the test package
        assert!(workspace.get_package("regular-pkg").is_some());
        assert!(workspace.get_package("test-pkg").is_none());
    }
}
