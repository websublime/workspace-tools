use std::path::Path;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};

mod test_utils;
use test_utils::{TestContext, TestFixture, PackageManager};

#[test]
fn test_workspace_scan_with_fixture() {
    // Create a test context with a complete monorepo
    let ctx = TestContext::new();
    
    // Get the root path
    let root_path = ctx.get_path();
    
    // Create a workspace manager
    let manager = WorkspaceManager::new();
    let options = DiscoveryOptions::default();
    
    // Discover workspace
    let result = manager.discover_workspace(root_path, &options);
    
    // Verify the result
    assert!(result.is_ok());
    
    let workspace = result.unwrap();
    
    // Verify packages
    assert_eq!(workspace.sorted_packages().len(), 6);
    
    // Verify package names
    assert!(workspace.get_package("@scope/package-foo").is_some());
    assert!(workspace.get_package("@scope/package-bar").is_some());
    assert!(workspace.get_package("@scope/package-baz").is_some());
    assert!(workspace.get_package("@scope/package-charlie").is_some());
    assert!(workspace.get_package("@scope/package-major").is_some());
    assert!(workspace.get_package("@scope/package-tom").is_some());
}

#[test]
fn test_with_specific_package_manager() {
    // Create a test context with yarn package manager
    let ctx = TestContext::with_package_manager(PackageManager::Yarn);
    
    // Get the root path
    let root_path = ctx.get_path();
    
    // Verify yarn.lock exists
    let yarn_lock_path = root_path.join("yarn.lock");
    assert!(Path::new(&yarn_lock_path).exists());
}

#[test]
fn test_with_cycle_dependency() {
    // Create a test context with cycle dependencies
    let ctx = TestContext::with_cycle();
    
    // Get the root path
    let root_path = ctx.get_path();
    
    // Create a workspace manager
    let manager = WorkspaceManager::new();
    let options = DiscoveryOptions::default();
    
    // Discover workspace
    let result = manager.discover_workspace(root_path, &options);
    
    // Verify the result
    assert!(result.is_ok());
    
    let workspace = result.unwrap();
    
    // Verify we have a workspace with packages
    assert!(!workspace.is_empty());
    
    // Analyze dependencies should detect cycles
    let analysis = workspace.analyze_dependencies();
    assert!(analysis.is_ok());
    assert!(analysis.unwrap().cycles_detected);
}

#[test]
fn test_custom_fixture_creation() {
    // Create a custom fixture directly
    let mut fixture = TestFixture::new();
    
    // Add specific packages
    fixture
        .create_package_foo()
        .create_package_bar();
    
    // Verify packages were created
    assert_eq!(fixture.packages.len(), 2);
    
    // Verify package paths
    let foo_path = fixture.root_path.join("packages").join("package-foo");
    let bar_path = fixture.root_path.join("packages").join("package-bar");
    
    assert!(Path::new(&foo_path).exists());
    assert!(Path::new(&bar_path).exists());
    
    // Verify package.json files
    let foo_package_json = foo_path.join("package.json");
    let bar_package_json = bar_path.join("package.json");
    
    assert!(Path::new(&foo_package_json).exists());
    assert!(Path::new(&bar_package_json).exists());
} 