use rstest::*;
use std::path::PathBuf;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use tempfile::TempDir;

mod fixtures;

#[rstest]
fn test_workspace_discovery_with_npm(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Set up discovery options
    let options = DiscoveryOptions::default()
        .auto_detect_root(false) // We know the root path
        .include_patterns(vec!["**/package.json"]);

    // Discover workspace
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify package count
    let packages = workspace.sorted_packages();
    assert_eq!(packages.len(), 6, "Expected 6 packages in workspace");

    // Verify package names
    let package_names: Vec<String> =
        packages.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    assert!(package_names.contains(&"@scope/package-foo".to_string()));
    assert!(package_names.contains(&"@scope/package-bar".to_string()));
    assert!(package_names.contains(&"@scope/package-baz".to_string()));
    assert!(package_names.contains(&"@scope/package-charlie".to_string()));
    assert!(package_names.contains(&"@scope/package-major".to_string()));
    assert!(package_names.contains(&"@scope/package-tom".to_string()));

    // Verify package manager detection
    assert!(workspace.package_manager().is_some());
    if let Some(manager) = workspace.package_manager() {
        assert_eq!(format!("{manager}"), "npm");
    }
}

#[rstest]
fn test_workspace_discovery_with_yarn(#[from(fixtures::yarn_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify package manager detection
    assert!(workspace.package_manager().is_some());
    if let Some(manager) = workspace.package_manager() {
        assert_eq!(format!("{manager}"), "yarn");
    }
}

#[rstest]
fn test_workspace_discovery_with_pnpm(#[from(fixtures::pnpm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify package manager detection
    assert!(workspace.package_manager().is_some());
    if let Some(manager) = workspace.package_manager() {
        assert_eq!(format!("{manager}"), "pnpm");
    }
}

#[rstest]
fn test_workspace_discovery_with_exclude_patterns(
    #[from(fixtures::npm_monorepo)] temp_dir: TempDir,
) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Set up discovery options that exclude package-foo and package-bar
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .exclude_patterns(vec!["**/package-foo/**", "**/package-bar/**"]);

    // Discover workspace
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify package count (6 total packages - 2 excluded = 4 remaining)
    let packages = workspace.sorted_packages();
    assert_eq!(packages.len(), 4, "Expected 4 packages after exclusions");

    // Verify excluded packages are not present
    let package_names: Vec<String> =
        packages.iter().map(|pkg| pkg.borrow().package.borrow().name().to_string()).collect();

    assert!(!package_names.contains(&"@scope/package-foo".to_string()));
    assert!(!package_names.contains(&"@scope/package-bar".to_string()));
    assert!(package_names.contains(&"@scope/package-baz".to_string()));
}

#[rstest]
#[allow(clippy::useless_conversion)]
fn test_workspace_discovery_with_additional_paths(
    #[from(fixtures::npm_monorepo)] temp_dir: TempDir,
) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Create an additional package outside the standard packages directory
    let extra_dir = root_path.join("extra");
    std::fs::create_dir_all(&extra_dir).expect("Failed to create extra directory");

    let extra_pkg_json = r#"{
        "name": "@scope/extra-package",
        "version": "1.0.0",
        "description": "Extra package"
    }"#;

    std::fs::write(extra_dir.join("package.json"), extra_pkg_json)
        .expect("Failed to write extra package.json");

    // Set up discovery options with additional path
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .additional_package_paths(vec![PathBuf::from(extra_dir)]);

    // Discover workspace
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Verify the additional package was found
    assert!(workspace.get_package("@scope/extra-package").is_some());
}
