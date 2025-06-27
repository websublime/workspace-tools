//! Enhanced package registry tests
//!
//! This module contains comprehensive tests for the enhanced dependency registry
//! functionality that includes package registry querying capabilities.

#![allow(clippy::unwrap_used)]

use semver::VersionReq;
use sublime_package_tools::{DependencyRegistry, LocalRegistry, NpmRegistry};

/// Test the enhanced dependency registry with local package registry
#[test]
fn test_dependency_registry_with_local_registry() {
    // Create a local registry with test data
    let local_registry = LocalRegistry::default();

    // Add some test packages to the local registry
    local_registry
        .add_package_versions("react", &["16.14.0", "17.0.2", "18.2.0", "18.3.1"])
        .unwrap();
    local_registry.add_package_versions("lodash", &["4.17.20", "4.17.21"]).unwrap();

    // Create dependency registry with the local package registry
    let mut registry = DependencyRegistry::with_package_registry(Box::new(local_registry));

    // Test basic functionality
    let react_dep = registry.get_or_create("react", "^17.0.0").unwrap();
    assert_eq!(react_dep.name(), "react");
    assert_eq!(react_dep.version().to_string(), "^17.0.0");

    // Test package registry capabilities
    assert!(registry.has_package_registry());

    // Test getting package versions
    let react_versions = registry.get_package_versions("react").unwrap();
    assert_eq!(react_versions.len(), 4);
    assert!(react_versions.contains(&"16.14.0".to_string()));
    assert!(react_versions.contains(&"17.0.2".to_string()));
    assert!(react_versions.contains(&"18.2.0".to_string()));
    assert!(react_versions.contains(&"18.3.1".to_string()));

    // Test finding highest compatible version
    let req1 = VersionReq::parse("^17.0.0").unwrap();
    let req2 = VersionReq::parse(">=17.0.0").unwrap();
    let highest = registry.find_highest_compatible_version("react", &[&req1, &req2]).unwrap();
    // ^17.0.0 matches any 17.x.x version, so should find the highest: 17.0.2
    assert_eq!(highest, "17.0.2");

    // Test with version that requires 18.x
    let req3 = VersionReq::parse("^18.0.0").unwrap();
    let highest_18 = registry.find_highest_compatible_version("react", &[&req3]).unwrap();
    assert_eq!(highest_18, "18.3.1");

    // Test with non-existent package
    let empty_versions = registry.get_package_versions("non-existent").unwrap();
    assert!(empty_versions.is_empty());

    let fallback = registry.find_highest_compatible_version("non-existent", &[&req1]).unwrap();
    assert_eq!(fallback, "0.0.0");
}

/// Test dependency registry without package registry (original behavior)
#[test]
fn test_dependency_registry_without_package_registry() {
    let mut registry = DependencyRegistry::new();

    // Test that it doesn't have package registry capabilities
    assert!(!registry.has_package_registry());

    // Create a dependency
    let react_dep = registry.get_or_create("react", "^17.0.2").unwrap();
    assert_eq!(react_dep.name(), "react");

    // Test that get_package_versions returns empty for non-configured registry
    let versions = registry.get_package_versions("react").unwrap();
    assert!(versions.is_empty());

    // Test find_highest_compatible_version falls back to existing dependency
    let req = VersionReq::parse("^17.0.0").unwrap();
    let highest = registry.find_highest_compatible_version("react", &[&req]).unwrap();
    assert_eq!(highest, "17.0.2");

    // Test with non-existent dependency returns fallback
    let fallback = registry.find_highest_compatible_version("non-existent", &[&req]).unwrap();
    assert_eq!(fallback, "0.0.0");
}

/// Test setting package registry after creation
#[test]
fn test_set_package_registry_after_creation() {
    let mut registry = DependencyRegistry::new();
    assert!(!registry.has_package_registry());

    // Create a local registry with test data
    let local_registry = LocalRegistry::default();

    // Add test data
    local_registry.add_package_versions("test-pkg", &["1.0.0", "2.0.0"]).unwrap();

    // Set the package registry
    registry.set_package_registry(Box::new(local_registry));
    assert!(registry.has_package_registry());

    // Test functionality
    let versions = registry.get_package_versions("test-pkg").unwrap();
    assert_eq!(versions.len(), 2);

    let req = VersionReq::parse("^1.0.0").unwrap();
    let highest = registry.find_highest_compatible_version("test-pkg", &[&req]).unwrap();
    assert!(highest == "1.0.0");
}

/// Test npm registry creation and cloning capability
#[test]
fn test_npm_registry_cloning() {
    let npm_registry = NpmRegistry::new("https://registry.npmjs.org");
    let cloned_registry = npm_registry.clone();

    // Create dependency registry with npm registry
    let registry = DependencyRegistry::with_package_registry(Box::new(npm_registry));
    assert!(registry.has_package_registry());

    // Create another registry with cloned npm registry
    let registry2 = DependencyRegistry::with_package_registry(Box::new(cloned_registry));
    assert!(registry2.has_package_registry());
}

/// Test version conflict resolution with package registry
#[test]
fn test_version_conflict_resolution_with_registry() {
    // Create a local registry with multiple versions
    let local_registry = LocalRegistry::default();

    local_registry
        .add_package_versions("react", &["16.0.0", "17.0.0", "17.0.2", "18.0.0"])
        .unwrap();

    let mut registry = DependencyRegistry::with_package_registry(Box::new(local_registry));

    // Create dependencies with different version requirements
    let _dep1 = registry.get_or_create("react", "^17.0.0").unwrap();
    let _dep2 = registry.get_or_create("react", "^17.0.2").unwrap();

    // Test that the registry updated to the higher version
    let react_dep = registry.get("react").unwrap();
    assert_eq!(react_dep.version().to_string(), "^17.0.2");

    // Test resolution using package registry
    let resolution_result = registry.resolve_version_conflicts().unwrap();

    // Should have resolved versions
    assert!(!resolution_result.resolved_versions.is_empty());

    // Check if react was resolved
    if let Some(resolved_version) = resolution_result.resolved_versions.get("react") {
        // Should resolve to one of the available versions
        assert!(resolved_version == "17.0.0" || resolved_version == "17.0.2");
    }
}

/// Test error handling when package registry fails
#[test]
fn test_error_handling_with_failed_registry() {
    // Create a registry that will have no packages
    let local_registry = LocalRegistry::default();
    let registry = DependencyRegistry::with_package_registry(Box::new(local_registry));

    // Test with non-existent package - should not error but return fallback
    let req = VersionReq::parse("^1.0.0").unwrap();
    let result = registry.find_highest_compatible_version("non-existent-package", &[&req]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0.0.0");

    // Test getting versions for non-existent package
    let versions = registry.get_package_versions("non-existent-package").unwrap();
    assert!(versions.is_empty());
}

/// Test complex version requirement matching
#[test]
fn test_complex_version_requirements() {
    let local_registry = LocalRegistry::default();

    local_registry
        .add_package_versions("complex-pkg", &["1.0.0", "1.1.0", "1.2.0", "2.0.0", "2.1.0"])
        .unwrap();

    let registry = DependencyRegistry::with_package_registry(Box::new(local_registry));

    // Test various requirement patterns
    let req1 = VersionReq::parse("^1.0.0").unwrap(); // Should match 1.x versions
    let highest_1x = registry.find_highest_compatible_version("complex-pkg", &[&req1]).unwrap();
    assert_eq!(highest_1x, "1.2.0");

    let req2 = VersionReq::parse("~1.1.0").unwrap(); // Should match 1.1.x versions
    let highest_11x = registry.find_highest_compatible_version("complex-pkg", &[&req2]).unwrap();
    assert_eq!(highest_11x, "1.1.0");

    let req3 = VersionReq::parse(">=2.0.0").unwrap(); // Should match 2.x and above
    let highest_2x = registry.find_highest_compatible_version("complex-pkg", &[&req3]).unwrap();
    assert_eq!(highest_2x, "2.1.0");

    // Test multiple requirements (intersection)
    let req4 = VersionReq::parse(">=1.1.0").unwrap();
    let req5 = VersionReq::parse("<2.0.0").unwrap();
    let intersection =
        registry.find_highest_compatible_version("complex-pkg", &[&req4, &req5]).unwrap();
    assert_eq!(intersection, "1.2.0");
}
