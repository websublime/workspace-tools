use rstest::*;
use std::fs;
use std::path::PathBuf;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use tempfile::TempDir;

mod fixtures;

#[rstest]
fn test_write_package_changes(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Load workspace
    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Get a package
    let foo_pkg = workspace.get_package("@scope/package-foo").expect("Failed to get package-foo");

    // Modify the version
    let new_version = "2.0.0";
    {
        let borrowed_pkg_info = foo_pkg.borrow();
        borrowed_pkg_info.update_version(new_version).expect("Failed to update version");
    }

    // Write changes to disk
    workspace.write_changes().expect("Failed to write changes");

    // Verify the change was written to disk
    let pkg_json_path = PathBuf::from(foo_pkg.borrow().package_json_path.clone());
    let pkg_json_content = fs::read_to_string(pkg_json_path).expect("Failed to read package.json");
    let pkg_json: serde_json::Value =
        serde_json::from_str(&pkg_json_content).expect("Failed to parse package.json");

    assert_eq!(
        pkg_json["version"].as_str().unwrap(),
        new_version,
        "Version in package.json should be updated"
    );
}

#[rstest]
fn test_update_dependency_version(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Load workspace
    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Get a package that has dependencies
    let foo_pkg = workspace.get_package("@scope/package-foo").expect("Failed to get package-foo");

    // Update dependency version
    let dep_name = "@scope/package-bar";
    let new_dep_version = "2.0.0";
    {
        let borrowed_pkg_info = foo_pkg.borrow();
        borrowed_pkg_info
            .update_dependency_version(dep_name, new_dep_version)
            .expect("Failed to update dependency version");
    }

    // Write changes to disk
    workspace.write_changes().expect("Failed to write changes");

    // Verify the dependency version was updated in package.json
    let pkg_json_path = PathBuf::from(foo_pkg.borrow().package_json_path.clone());
    let pkg_json_content = fs::read_to_string(pkg_json_path).expect("Failed to read package.json");
    let pkg_json: serde_json::Value =
        serde_json::from_str(&pkg_json_content).expect("Failed to parse package.json");

    assert_eq!(
        pkg_json["dependencies"][dep_name].as_str().unwrap(),
        new_dep_version,
        "Dependency version in package.json should be updated"
    );
}
