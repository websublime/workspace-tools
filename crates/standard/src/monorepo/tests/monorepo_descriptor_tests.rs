//! # MonorepoDescriptor Tests
//!
//! ## What
//! This module tests the MonorepoDescriptor functionality, including
//! creation, package management, and dependency graph operations.
//!
//! ## How
//! Tests verify descriptor creation, package lookup, dependency graph
//! generation, and package dependency resolution.
//!
//! ## Why
//! Proper testing of MonorepoDescriptor ensures reliable monorepo analysis
//! and package management operations work correctly.

use super::test_utils::create_test_package;
use crate::monorepo::{MonorepoDescriptor, MonorepoKind};
use std::path::PathBuf;

#[tokio::test]
async fn test_monorepo_descriptor_creation() {
    let root = PathBuf::from("/fake/monorepo");
    let packages = vec![
        create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
        create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
    ];

    let descriptor =
        MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root.clone(), packages);

    assert_eq!(descriptor.kind().name(), "yarn");
    assert_eq!(descriptor.root(), root.as_path());
    assert_eq!(descriptor.packages().len(), 2);
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_get_package() {
    let root = PathBuf::from("/fake/monorepo");
    let packages = vec![
        create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
        create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
    ];

    let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

    // Test existing package
    let pkg_a = descriptor.get_package("pkg-a");
    assert!(pkg_a.is_some());
    assert_eq!(pkg_a.unwrap().name, "pkg-a");

    // Test non-existent package
    let pkg_c = descriptor.get_package("pkg-c");
    assert!(pkg_c.is_none());
}

#[allow(clippy::get_unwrap)]
#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_dependency_graph() {
    let root = PathBuf::from("/fake/monorepo");
    let packages = vec![
        create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
        create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        create_test_package("pkg-c", "1.0.0", "packages/c", &root, vec!["pkg-a", "pkg-b"], vec![]),
    ];

    let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

    let graph = descriptor.get_dependency_graph();

    // Check package A's dependents (B and C)
    let pkg_a_dependents = graph.get("pkg-a").unwrap();
    assert_eq!(pkg_a_dependents.len(), 2);
    assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-b"));
    assert!(pkg_a_dependents.iter().any(|pkg| pkg.name == "pkg-c"));

    // Check package B's dependents (C only)
    let pkg_b_dependents = graph.get("pkg-b").unwrap();
    assert_eq!(pkg_b_dependents.len(), 1);
    assert_eq!(pkg_b_dependents[0].name, "pkg-c");

    // Check package C has no dependents (it depends on A and B but nothing depends on C)
    let pkg_c_dependents = graph.get("pkg-c");
    // pkg-c might have an entry in the graph but with an empty vector
    if let Some(dependents) = pkg_c_dependents {
        assert_eq!(dependents.len(), 0);
    } else {
        // Or it might not have an entry at all
        assert!(pkg_c_dependents.is_none());
    }
}

#[tokio::test]
async fn test_find_dependencies_by_name() {
    let root = PathBuf::from("/fake/monorepo");
    let packages = vec![
        create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
        create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        create_test_package("pkg-c", "1.0.0", "packages/c", &root, vec!["pkg-a", "pkg-b"], vec![]),
        create_test_package("pkg-d", "1.0.0", "packages/d", &root, vec!["pkg-a"], vec!["pkg-b"]),
    ];

    let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

    // Test finding dependencies of pkg-a (should return the dependencies of pkg-a)
    let pkg_a_deps = descriptor.find_dependencies_by_name("pkg-a");
    assert_eq!(pkg_a_deps.len(), 0); // pkg-a has no dependencies

    // Test finding dependencies of pkg-b (should return pkg-a)
    let pkg_b_deps = descriptor.find_dependencies_by_name("pkg-b");
    assert_eq!(pkg_b_deps.len(), 1);
    assert_eq!(pkg_b_deps[0].name, "pkg-a");

    // Test finding dependencies of pkg-c (should return pkg-a and pkg-b)
    let pkg_c_deps = descriptor.find_dependencies_by_name("pkg-c");
    assert_eq!(pkg_c_deps.len(), 2);
    let names: Vec<&str> = pkg_c_deps.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"pkg-a"));
    assert!(names.contains(&"pkg-b"));

    // Test finding dependencies of pkg-d (should return pkg-a and pkg-b)
    let pkg_d_deps = descriptor.find_dependencies_by_name("pkg-d");
    assert_eq!(pkg_d_deps.len(), 2);
    let names: Vec<&str> = pkg_d_deps.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"pkg-a"));
    assert!(names.contains(&"pkg-b"));

    // Test non-existent package
    let non_existent = descriptor.find_dependencies_by_name("pkg-z");
    assert_eq!(non_existent.len(), 0);
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_find_package_for_path() {
    let root = PathBuf::from("/fake/monorepo");
    let packages = vec![
        create_test_package("pkg-a", "1.0.0", "packages/a", &root, vec![], vec![]),
        create_test_package("pkg-b", "1.0.0", "packages/b", &root, vec!["pkg-a"], vec![]),
        create_test_package("pkg-c", "1.0.0", "libs/c", &root, vec![], vec![]),
    ];

    let descriptor = MonorepoDescriptor::minimal(MonorepoKind::YarnWorkspaces, root, packages);

    // Test with exact path
    let pkg_a = descriptor.find_package_for_path(&PathBuf::from("packages/a"));
    assert!(pkg_a.is_some());
    assert_eq!(pkg_a.unwrap().name, "pkg-a");

    // Test with nested path
    let pkg_b = descriptor.find_package_for_path(&PathBuf::from("packages/b/src/main.rs"));
    assert!(pkg_b.is_some());
    assert_eq!(pkg_b.unwrap().name, "pkg-b");

    // Test with different base path
    let pkg_c = descriptor.find_package_for_path(&PathBuf::from("libs/c"));
    assert!(pkg_c.is_some());
    assert_eq!(pkg_c.unwrap().name, "pkg-c");

    // Test with non-matching path
    let no_match = descriptor.find_package_for_path(&PathBuf::from("other/path"));
    assert!(no_match.is_none());
}
