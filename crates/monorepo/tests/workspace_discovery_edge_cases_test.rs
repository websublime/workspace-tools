use rstest::*;
use std::fs;
use std::path::Path;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceError, WorkspaceManager};
use tempfile::TempDir;

mod fixtures;

fn create_empty_package(
    root_path: &Path,
    package_path: &str,
    name: &str,
    version: &str,
) -> std::io::Result<()> {
    let pkg_dir = root_path.join(package_path);
    fs::create_dir_all(&pkg_dir)?;

    let pkg_json = format!(
        r#"{{
        "name": "{name}",
        "version": "{version}"
    }}"#
    );

    fs::write(pkg_dir.join("package.json"), pkg_json)?;
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_max_depth_directory_traversal() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create directories at different depths
    let level1 = temp_dir.path().join("level1");
    fs::create_dir_all(&level1).expect("Failed to create level1");

    let level2 = level1.join("level2");
    fs::create_dir_all(&level2).expect("Failed to create level2");

    let level3 = level2.join("level3");
    fs::create_dir_all(&level3).expect("Failed to create level3");

    // Create package.json files at each level
    let pkg_json_root = r#"{"name": "root-pkg", "version": "1.0.0"}"#;
    fs::write(temp_dir.path().join("package.json"), pkg_json_root)
        .expect("Failed to write root package.json");

    let pkg_json_level1 = r#"{"name": "level1-pkg", "version": "1.0.0"}"#;
    fs::write(level1.join("package.json"), pkg_json_level1)
        .expect("Failed to write level1 package.json");

    let pkg_json_level2 = r#"{"name": "level2-pkg", "version": "1.0.0"}"#;
    fs::write(level2.join("package.json"), pkg_json_level2)
        .expect("Failed to write level2 package.json");

    let pkg_json_level3 = r#"{"name": "level3-pkg", "version": "1.0.0"}"#;
    fs::write(level3.join("package.json"), pkg_json_level3)
        .expect("Failed to write level3 package.json");

    // Test with different max_depth values
    let workspace_manager = WorkspaceManager::new();

    // With max_depth=1, should only find root-pkg
    let options1 = DiscoveryOptions::default()
        .auto_detect_root(false)
        .max_depth(1) // Only the root directory
        .include_patterns(vec!["**/package.json"]);

    let workspace1 = workspace_manager
        .discover_workspace(temp_dir.path(), &options1)
        .expect("Failed to discover workspace with depth 1");

    assert!(workspace1.get_package("root-pkg").is_some(), "Should find root-pkg with max_depth=1");
    assert!(
        workspace1.get_package("level1-pkg").is_none(),
        "Should NOT find level1-pkg with max_depth=1"
    );

    // With max_depth=2, should find root-pkg and level1-pkg
    let options2 = DiscoveryOptions::default()
        .auto_detect_root(false)
        .max_depth(2) // Root and one level of subdirectories
        .include_patterns(vec!["**/package.json"]);

    let workspace2 = workspace_manager
        .discover_workspace(temp_dir.path(), &options2)
        .expect("Failed to discover workspace with depth 2");

    assert!(workspace2.get_package("root-pkg").is_some(), "Should find root-pkg with max_depth=2");
    assert!(
        workspace2.get_package("level1-pkg").is_some(),
        "Should find level1-pkg with max_depth=2"
    );
    assert!(
        workspace2.get_package("level2-pkg").is_none(),
        "Should NOT find level2-pkg with max_depth=2"
    );

    // With max_depth=3, should find root, level1, and level2 packages
    let options3 = DiscoveryOptions::default()
        .auto_detect_root(false)
        .max_depth(3)
        .include_patterns(vec!["**/package.json"]);

    let workspace3 = workspace_manager
        .discover_workspace(temp_dir.path(), &options3)
        .expect("Failed to discover workspace with depth 3");

    assert!(workspace3.get_package("root-pkg").is_some(), "Should find root-pkg with max_depth=3");
    assert!(
        workspace3.get_package("level1-pkg").is_some(),
        "Should find level1-pkg with max_depth=3"
    );
    assert!(
        workspace3.get_package("level2-pkg").is_some(),
        "Should find level2-pkg with max_depth=3"
    );
    assert!(
        workspace3.get_package("level3-pkg").is_none(),
        "Should NOT find level3-pkg with max_depth=3"
    );

    // With max_depth=4, should find all packages
    let options4 = DiscoveryOptions::default()
        .auto_detect_root(false)
        .max_depth(4)
        .include_patterns(vec!["**/package.json"]);

    let workspace4 = workspace_manager
        .discover_workspace(temp_dir.path(), &options4)
        .expect("Failed to discover workspace with depth 4");

    assert!(workspace4.get_package("root-pkg").is_some(), "Should find root-pkg with max_depth=4");
    assert!(
        workspace4.get_package("level1-pkg").is_some(),
        "Should find level1-pkg with max_depth=4"
    );
    assert!(
        workspace4.get_package("level2-pkg").is_some(),
        "Should find level2-pkg with max_depth=4"
    );
    assert!(
        workspace4.get_package("level3-pkg").is_some(),
        "Should find level3-pkg with max_depth=4"
    );
}

#[rstest]
fn test_discovery_with_patterns(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Create a package in a separate directory structure
    let extras_path = "extras/special";
    create_empty_package(root_path, extras_path, "special-package", "1.0.0")
        .expect("Failed to create special package");

    // First, try to find all packages with a broad pattern
    let all_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["**/package.json"]);

    let all_workspace = workspace_manager
        .discover_workspace(root_path, &all_options)
        .expect("Failed to discover all workspace");

    // Check if all packages were found (both standard and special)
    assert!(
        all_workspace.get_package("@scope/package-foo").is_some(),
        "Should find standard packages"
    );
    assert!(all_workspace.get_package("special-package").is_some(), "Should find special package");

    // Now, try with a pattern that should only find the special package
    let special_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["extras/**/package.json"]);

    let special_workspace = workspace_manager
        .discover_workspace(root_path, &special_options)
        .expect("Failed to discover special workspace");

    // Only the special package should be found
    assert!(
        special_workspace.get_package("special-package").is_some(),
        "Should find special package"
    );
    assert!(
        special_workspace.get_package("@scope/package-foo").is_none(),
        "Should not find standard packages"
    );

    // Try with a pattern that should only find standard packages
    let standard_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["packages/**/package.json"]);

    let standard_workspace = workspace_manager
        .discover_workspace(root_path, &standard_options)
        .expect("Failed to discover standard workspace");

    // Only standard packages should be found
    assert!(
        standard_workspace.get_package("@scope/package-foo").is_some(),
        "Should find standard packages"
    );
    assert!(
        standard_workspace.get_package("special-package").is_none(),
        "Should not find special package"
    );

    // Test with excluding specific patterns
    let exclude_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["**/package.json"])
        .exclude_patterns(vec!["**/package-foo/**"]);

    let exclude_workspace = workspace_manager
        .discover_workspace(root_path, &exclude_options)
        .expect("Failed to discover exclude workspace");

    // package-foo should be excluded
    assert!(
        exclude_workspace.get_package("special-package").is_some(),
        "Should find special package"
    );
    assert!(
        exclude_workspace.get_package("@scope/package-bar").is_some(),
        "Should find other standard packages"
    );
    assert!(
        exclude_workspace.get_package("@scope/package-foo").is_none(),
        "Should not find excluded package-foo"
    );
}

#[rstest]
fn test_discovery_private_packages(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Create a private package
    let private_pkg_dir = root_path.join("packages/private-pkg");
    fs::create_dir_all(&private_pkg_dir).expect("Failed to create private package dir");

    let private_pkg_json = r#"{
        "name": "private-package",
        "version": "1.0.0",
        "private": true
    }"#;

    fs::write(private_pkg_dir.join("package.json"), private_pkg_json)
        .expect("Failed to write private package.json");

    // Try with include_private=true (default)
    let include_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["**/package.json"]);

    let include_workspace = workspace_manager
        .discover_workspace(root_path, &include_options)
        .expect("Failed to discover workspace with private packages");

    // The private package should be found
    assert!(
        include_workspace.get_package("private-package").is_some(),
        "With include_private=true, private package should be found"
    );

    // Try with include_private=false
    let exclude_options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_private(false)
        .include_patterns(vec!["**/package.json"]);

    let exclude_workspace = workspace_manager
        .discover_workspace(root_path, &exclude_options)
        .expect("Failed to discover workspace without private packages");

    // The private package should not be found
    assert!(
        exclude_workspace.get_package("private-package").is_none(),
        "With include_private=false, private package should not be found"
    );
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_empty_workspace() {
    // Create a temporary directory with no packages
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();

    // Create an empty package.json
    let empty_pkg_json = r#"{
        "name": "empty-workspace",
        "version": "0.0.0"
    }"#;

    fs::write(root_path.join("package.json"), empty_pkg_json)
        .expect("Failed to write empty package.json");

    let workspace_manager = WorkspaceManager::new();

    // This should fail because there are no packages
    let result = workspace_manager
        .discover_workspace(root_path, &DiscoveryOptions::default().auto_detect_root(false));

    match result {
        Err(WorkspaceError::NoPackagesFound(_)) => {
            assert!(true);
        }
        Err(err) => {
            panic!("Expected NoPackagesFound error, got: {err}");
        }
        Ok(_workspace) => {
            panic!("Should have failed with no packages error");
        }
    }
}

#[rstest]
fn test_nested_workspaces(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Create a nested workspace inside one of the packages
    let nested_workspace_dir = root_path.join("packages/nested-workspace");
    fs::create_dir_all(&nested_workspace_dir).expect("Failed to create nested workspace dir");

    let nested_pkg_json = r#"{
        "name": "nested-workspace",
        "version": "1.0.0",
        "workspaces": ["packages/*"]
    }"#;

    fs::write(nested_workspace_dir.join("package.json"), nested_pkg_json)
        .expect("Failed to write nested workspace package.json");

    // Create a package within the nested workspace
    let nested_pkg_dir = nested_workspace_dir.join("packages/nested-pkg");
    fs::create_dir_all(&nested_pkg_dir).expect("Failed to create nested package dir");

    let nested_child_pkg_json = r#"{
        "name": "nested-child",
        "version": "1.0.0"
    }"#;

    fs::write(nested_pkg_dir.join("package.json"), nested_child_pkg_json)
        .expect("Failed to write nested child package.json");

    // Discover the root workspace
    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Both the nested workspace itself and its child package should be found
    assert!(workspace.get_package("nested-workspace").is_some());
    assert!(workspace.get_package("nested-child").is_some());
}
