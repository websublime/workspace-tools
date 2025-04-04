use rstest::*;
use std::path::PathBuf;
use sublime_monorepo_tools::WorkspaceConfig;
use tempfile::TempDir;

mod fixtures;

#[test]
fn test_workspace_config_creation() {
    let root_path = PathBuf::from("/test/path");

    // Test basic creation
    let config = WorkspaceConfig::new(root_path.clone());
    assert_eq!(config.root_path, root_path);
    assert!(config.packages.is_empty());
    assert!(config.package_manager.is_none());
    assert!(config.config.is_empty());

    // Test with packages
    let config =
        WorkspaceConfig::new(root_path.clone()).with_packages(vec!["packages/*", "apps/*"]);
    assert_eq!(config.packages.len(), 2);
    assert_eq!(config.packages[0], "packages/*");
    assert_eq!(config.packages[1], "apps/*");

    // Test with package manager
    let config = WorkspaceConfig::new(root_path.clone()).with_package_manager(Some("npm"));
    assert_eq!(config.package_manager, Some("npm".to_string()));

    // Test with additional config
    let config = WorkspaceConfig::new(root_path)
        .with_config("version", "1.0.0")
        .with_config("private", true);

    assert_eq!(config.config.len(), 2);
    assert_eq!(config.config["version"].as_str().unwrap(), "1.0.0");
    assert!(config.config["private"].as_bool().unwrap());
}

#[rstest]
fn test_workspace_with_config(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};

    let root_path = temp_dir.path().to_path_buf();

    // Create a custom config directly in the test
    let config_path = root_path.join("custom-config.json");
    let custom_config_content = r#"{
        "test_config": true,
        "custom_value": "something"
    }"#;
    std::fs::write(&config_path, custom_config_content).expect("Failed to write custom config");

    // Use WorkspaceManager which we know works with our fixtures
    let workspace_manager = WorkspaceManager::new();

    // Set up appropriate discovery options
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["**/package.json"])
        .exclude_patterns(vec!["**/node_modules/**"]); // Explicitly exclude node_modules

    // Discover the workspace
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify workspace was created correctly
    assert_eq!(workspace.root_path(), temp_dir.path());
    assert!(!workspace.is_empty());

    // Check for the package manager
    assert!(workspace.package_manager().is_some());
    if let Some(manager) = workspace.package_manager() {
        assert_eq!(format!("{manager}"), "npm");
    }

    // Verify the packages were found
    let packages = workspace.sorted_packages();
    assert!(!packages.is_empty());
    assert_eq!(packages.len(), 6); // We should have 6 packages

    // Verify specific packages
    assert!(workspace.get_package("@scope/package-foo").is_some());
    assert!(workspace.get_package("@scope/package-bar").is_some());
    assert!(workspace.get_package("@scope/package-baz").is_some());
    assert!(workspace.get_package("@scope/package-charlie").is_some());
    assert!(workspace.get_package("@scope/package-major").is_some());
    assert!(workspace.get_package("@scope/package-tom").is_some());
}

#[test]
fn test_workspace_config_builder() {
    use serde_json::json;
    use std::path::PathBuf;
    use sublime_monorepo_tools::WorkspaceConfig;

    // Test root path
    let root_path = PathBuf::from("/test/workspace/root");

    // Start with a basic config
    let config = WorkspaceConfig::new(root_path.clone());

    // Verify default values
    assert_eq!(config.root_path, root_path);
    assert!(config.packages.is_empty());
    assert!(config.package_manager.is_none());
    assert!(config.config.is_empty());

    // Test the builder pattern by chaining methods
    let config = WorkspaceConfig::new(root_path.clone())
        .with_packages(vec!["packages/*", "apps/*"])
        .with_package_manager(Some("npm"))
        .with_config("version", "1.0.0")
        .with_config("private", true)
        .with_config("workspaces", json!(["packages/*", "apps/*"]));

    // Verify all values were set correctly
    assert_eq!(config.root_path, root_path);
    assert_eq!(config.packages, vec!["packages/*", "apps/*"]);
    assert_eq!(config.package_manager, Some("npm".to_string()));

    // Check config entries
    assert_eq!(config.config.len(), 3);
    assert_eq!(config.config["version"].as_str().unwrap(), "1.0.0");
    assert!(config.config["private"].as_bool().unwrap());

    // Check array values
    let workspaces = config.config["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);
    assert_eq!(workspaces[0].as_str().unwrap(), "packages/*");
    assert_eq!(workspaces[1].as_str().unwrap(), "apps/*");

    // Test setting package manager to None explicitly
    let config = WorkspaceConfig::new(root_path).with_package_manager(None::<&str>);
    assert!(config.package_manager.is_none());
}

#[test]
fn test_workspace_config_with_complex_values() {
    use serde_json::json;
    use std::path::PathBuf;
    use sublime_monorepo_tools::WorkspaceConfig;

    let root_path = PathBuf::from("/test/workspace");

    // Test with more complex JSON values
    let config = WorkspaceConfig::new(root_path)
        .with_config(
            "scripts",
            json!({
                "build": "webpack",
                "test": "jest",
                "lint": "eslint ."
            }),
        )
        .with_config(
            "dependencies",
            json!({
                "react": "^17.0.0",
                "typescript": "^4.5.0"
            }),
        )
        .with_config(
            "nested",
            json!({
                "options": {
                    "debug": true,
                    "mode": "development"
                }
            }),
        );

    // Verify the complex values
    let scripts = config.config["scripts"].as_object().unwrap();
    assert_eq!(scripts.get("build").unwrap().as_str().unwrap(), "webpack");
    assert_eq!(scripts.get("test").unwrap().as_str().unwrap(), "jest");
    assert_eq!(scripts.get("lint").unwrap().as_str().unwrap(), "eslint .");

    let deps = config.config["dependencies"].as_object().unwrap();
    assert_eq!(deps.get("react").unwrap().as_str().unwrap(), "^17.0.0");
    assert_eq!(deps.get("typescript").unwrap().as_str().unwrap(), "^4.5.0");

    // Check nested objects
    let nested = config.config["nested"].as_object().unwrap();
    let options = nested.get("options").unwrap().as_object().unwrap();
    assert!(options.get("debug").unwrap().as_bool().unwrap());
    assert_eq!(options.get("mode").unwrap().as_str().unwrap(), "development");
}
