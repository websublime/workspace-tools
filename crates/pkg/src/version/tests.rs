//! Tests for the version module.
//!
//! This module contains tests for version resolution, dependency graph construction,
//! and circular dependency detection.
//!
//! **What**: Comprehensive test suite for version resolution functionality including
//! `VersionResolver` initialization, project detection, and package discovery.
//!
//! **How**: Uses real filesystem with temporary directories to validate resolver behavior
//! in both monorepo and single-package scenarios, including error cases.
//!
//! **Why**: To ensure the version module correctly detects project structures, loads
//! packages, handles edge cases and errors robustly, and provides reliable version
//! resolution functionality.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use crate::config::PackageToolsConfig;
use crate::error::VersionError;
use crate::types::PackageInfo;
use crate::version::{DependencyGraph, VersionResolver};
use package_json::PackageJson;
use std::collections::HashMap;
use std::path::PathBuf;
use sublime_standard_tools::filesystem::AsyncFileSystem;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Creates a mock single-package workspace.
///
/// Sets up a simple workspace with a single package.json at the root.
async fn create_single_package_workspace() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package_json = r#"{
        "name": "my-package",
        "version": "1.0.0",
        "description": "A test package"
    }"#;

    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .expect("Failed to write package.json");

    (temp_dir, root)
}

/// Creates a mock monorepo workspace with multiple packages.
///
/// Sets up a monorepo structure with workspaces and multiple packages.
async fn create_monorepo_workspace() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Root package.json with workspaces
    let root_package_json = r#"{
        "name": "monorepo-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package_json)
        .await
        .expect("Failed to write root package.json");

    // Create packages directory
    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Package A
    let pkg_a_dir = root.join("packages").join("pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a_json = r#"{
        "name": "@monorepo/pkg-a",
        "version": "1.0.0",
        "dependencies": {
            "@monorepo/pkg-b": "1.0.0"
        }
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a_json)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B
    let pkg_b_dir = root.join("packages").join("pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b_json = r#"{
        "name": "@monorepo/pkg-b",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b_json)
        .await
        .expect("Failed to write pkg-b package.json");

    (temp_dir, root)
}

// ============================================================================
// VersionResolver Initialization Tests
// ============================================================================

#[tokio::test]
async fn test_new_with_invalid_workspace_root_not_exists() {
    let root = PathBuf::from("/nonexistent/path/that/does/not/exist");
    let config = PackageToolsConfig::default();

    let result = VersionResolver::new(root.clone(), config).await;

    assert!(result.is_err());
    match result {
        Err(VersionError::InvalidWorkspaceRoot { path, reason }) => {
            assert_eq!(path, root);
            assert!(reason.contains("does not exist"));
        }
        _ => panic!("Expected InvalidWorkspaceRoot error"),
    }
}

#[tokio::test]
async fn test_new_with_single_package_success() {
    let (_temp_dir, root) = create_single_package_workspace().await;
    let config = PackageToolsConfig::default();

    let result = VersionResolver::new(root.clone(), config).await;

    assert!(result.is_ok());
    let resolver = result.expect("Should create resolver");
    assert!(!resolver.is_monorepo());
    assert_eq!(resolver.workspace_root(), root.as_path());
    assert_eq!(resolver.strategy(), crate::types::VersioningStrategy::Independent);
}

// TODO: MonorepoDetector needs proper workspace configuration to detect monorepo correctly
// Will be fixed when workspace detection is enhanced in sublime_standard_tools
#[tokio::test]
#[ignore]
async fn test_new_with_monorepo_success() {
    let (_temp_dir, root) = create_monorepo_workspace().await;
    let config = PackageToolsConfig::default();

    let result = VersionResolver::new(root.clone(), config).await;

    assert!(result.is_ok());
    let resolver = result.expect("Should create resolver");
    assert!(resolver.is_monorepo());
    assert_eq!(resolver.workspace_root(), root.as_path());
}

#[tokio::test]
async fn test_strategy_is_respected() {
    let (_temp_dir, root) = create_single_package_workspace().await;
    let mut config = PackageToolsConfig::default();
    config.version.strategy = crate::config::VersioningStrategy::Unified;

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    assert_eq!(resolver.strategy(), crate::types::VersioningStrategy::Unified);
}

// ============================================================================
// VersionResolver Getter Methods Tests
// ============================================================================

#[tokio::test]
async fn test_getters() {
    let (_temp_dir, root) = create_single_package_workspace().await;
    let config = PackageToolsConfig::default();

    let resolver =
        VersionResolver::new(root.clone(), config.clone()).await.expect("Should create resolver");

    // Test workspace_root()
    assert_eq!(resolver.workspace_root(), root.as_path());

    // Test strategy()
    assert_eq!(resolver.strategy(), crate::types::VersioningStrategy::Independent);

    // Test config()
    assert_eq!(resolver.config().version.strategy, crate::config::VersioningStrategy::Independent);

    // Test filesystem()
    let fs_ref = resolver.filesystem();
    assert!(fs_ref.exists(&root).await);
}

// ============================================================================
// Package Discovery Tests
// ============================================================================

#[tokio::test]
async fn test_discover_packages_single_package() {
    let (_temp_dir, root) = create_single_package_workspace().await;
    let config = PackageToolsConfig::default();

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    let packages = resolver.discover_packages().await.expect("Should discover packages");

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name(), "my-package");
    assert_eq!(packages[0].version().to_string(), "1.0.0");
}

#[tokio::test]
async fn test_discover_packages_missing_package_json() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();
    // No package.json added

    let config = PackageToolsConfig::default();

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    let result = resolver.discover_packages().await;

    assert!(result.is_err());
    match result {
        Err(VersionError::PackageJsonError { path, reason }) => {
            assert!(path.ends_with("package.json"));
            assert!(reason.contains("Failed to read file"));
        }
        _ => panic!("Expected PackageJsonError"),
    }
}

#[tokio::test]
async fn test_discover_packages_invalid_json() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Add invalid JSON
    tokio::fs::write(root.join("package.json"), "not valid json")
        .await
        .expect("Failed to write invalid json");

    let config = PackageToolsConfig::default();

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    let result = resolver.discover_packages().await;

    assert!(result.is_err());
    match result {
        Err(VersionError::PackageJsonError { path, reason }) => {
            assert!(path.ends_with("package.json"));
            assert!(reason.contains("Failed to parse JSON"));
        }
        _ => panic!("Expected PackageJsonError"),
    }
}

// TODO: MonorepoDetector needs proper workspace configuration to detect monorepo correctly
// Will be fixed when workspace detection is enhanced in sublime_standard_tools
#[tokio::test]
#[ignore]
async fn test_is_monorepo_detection() {
    // Test single package
    let (_temp_dir_single, root_single) = create_single_package_workspace().await;
    let config_single = PackageToolsConfig::default();
    let resolver_single =
        VersionResolver::new(root_single, config_single).await.expect("Should create resolver");
    assert!(!resolver_single.is_monorepo());

    // Test monorepo
    let (_temp_dir_mono, root_mono) = create_monorepo_workspace().await;
    let config_mono = PackageToolsConfig::default();
    let resolver_mono =
        VersionResolver::new(root_mono, config_mono).await.expect("Should create resolver");
    assert!(resolver_mono.is_monorepo());
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_config_access() {
    let (_temp_dir, root) = create_single_package_workspace().await;
    let mut config = PackageToolsConfig::default();
    config.version.default_bump = "minor".to_string();

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    assert_eq!(resolver.config().version.default_bump, "minor");
}

// TODO: MonorepoDetector needs proper workspace configuration to detect monorepo correctly
// Will be fixed when workspace detection is enhanced in sublime_standard_tools
#[tokio::test]
#[ignore]
async fn test_discover_packages_monorepo() {
    let (_temp_dir, root) = create_monorepo_workspace().await;
    let config = PackageToolsConfig::default();

    let resolver = VersionResolver::new(root, config).await.expect("Should create resolver");

    let packages = resolver.discover_packages().await.expect("Should discover packages");

    // Should find 2 packages (pkg-a and pkg-b)
    assert_eq!(packages.len(), 2);

    let names: Vec<&str> = packages.iter().map(|p| p.name()).collect();
    assert!(names.contains(&"@monorepo/pkg-a"));
    assert!(names.contains(&"@monorepo/pkg-b"));
}

// ============================================================================
// Dependency Graph Tests
// ============================================================================

/// Helper function to create a test PackageInfo with specified dependencies.
fn create_package_info(name: &str, version: &str, dependencies: Vec<(&str, &str)>) -> PackageInfo {
    let mut pkg_json =
        PackageJson { name: name.to_string(), version: version.to_string(), ..Default::default() };

    if !dependencies.is_empty() {
        let mut deps = HashMap::new();
        for (dep_name, dep_version) in dependencies {
            deps.insert(dep_name.to_string(), dep_version.to_string());
        }
        pkg_json.dependencies = Some(deps);
    }

    PackageInfo::new(pkg_json, None, PathBuf::from(format!("/test/{}", name)))
}

/// Helper function to create a test PackageInfo with dev dependencies.
fn create_package_info_with_dev_deps(
    name: &str,
    version: &str,
    dependencies: Vec<(&str, &str)>,
    dev_dependencies: Vec<(&str, &str)>,
) -> PackageInfo {
    let mut pkg_json =
        PackageJson { name: name.to_string(), version: version.to_string(), ..Default::default() };

    if !dependencies.is_empty() {
        let mut deps = HashMap::new();
        for (dep_name, dep_version) in dependencies {
            deps.insert(dep_name.to_string(), dep_version.to_string());
        }
        pkg_json.dependencies = Some(deps);
    }

    if !dev_dependencies.is_empty() {
        let mut dev_deps = HashMap::new();
        for (dep_name, dep_version) in dev_dependencies {
            dev_deps.insert(dep_name.to_string(), dep_version.to_string());
        }
        pkg_json.dev_dependencies = Some(dev_deps);
    }

    PackageInfo::new(pkg_json, None, PathBuf::from(format!("/test/{}", name)))
}

#[test]
fn test_graph_empty() {
    let packages: Vec<PackageInfo> = vec![];
    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 0);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.all_packages().is_empty());
}

#[test]
fn test_graph_single_package_no_dependencies() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 1);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.contains("package-a"));
    assert!(graph.dependents("package-a").is_empty());
    assert!(graph.dependencies("package-a").is_empty());
}

#[test]
fn test_graph_two_packages_with_dependency() {
    // package-b depends on package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    // Check that package-a has package-b as a dependent
    let dependents_a = graph.dependents("package-a");
    assert_eq!(dependents_a.len(), 1);
    assert!(dependents_a.contains(&"package-b".to_string()));

    // Check that package-b depends on package-a
    let deps_b = graph.dependencies("package-b");
    assert_eq!(deps_b.len(), 1);
    assert!(deps_b.contains(&"package-a".to_string()));

    // Check that package-b has no dependents
    assert!(graph.dependents("package-b").is_empty());

    // Check that package-a has no dependencies
    assert!(graph.dependencies("package-a").is_empty());
}

#[test]
fn test_graph_chain_of_dependencies() {
    // package-c -> package-b -> package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-b", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 3);
    assert_eq!(graph.edge_count(), 2);

    // package-a is depended on by package-b
    let dependents_a = graph.dependents("package-a");
    assert_eq!(dependents_a.len(), 1);
    assert!(dependents_a.contains(&"package-b".to_string()));

    // package-b is depended on by package-c and depends on package-a
    let dependents_b = graph.dependents("package-b");
    assert_eq!(dependents_b.len(), 1);
    assert!(dependents_b.contains(&"package-c".to_string()));

    let deps_b = graph.dependencies("package-b");
    assert_eq!(deps_b.len(), 1);
    assert!(deps_b.contains(&"package-a".to_string()));

    // package-c depends on package-b
    let deps_c = graph.dependencies("package-c");
    assert_eq!(deps_c.len(), 1);
    assert!(deps_c.contains(&"package-b".to_string()));
}

#[test]
fn test_graph_multiple_dependencies() {
    // package-d depends on both package-a and package-b
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![]),
        create_package_info(
            "package-d",
            "1.0.0",
            vec![("package-a", "^1.0.0"), ("package-b", "^1.0.0")],
        ),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 3);
    assert_eq!(graph.edge_count(), 2);

    // package-d depends on both package-a and package-b
    let deps_d = graph.dependencies("package-d");
    assert_eq!(deps_d.len(), 2);
    assert!(deps_d.contains(&"package-a".to_string()));
    assert!(deps_d.contains(&"package-b".to_string()));

    // Both package-a and package-b are depended on by package-d
    let dependents_a = graph.dependents("package-a");
    assert_eq!(dependents_a.len(), 1);
    assert!(dependents_a.contains(&"package-d".to_string()));

    let dependents_b = graph.dependents("package-b");
    assert_eq!(dependents_b.len(), 1);
    assert!(dependents_b.contains(&"package-d".to_string()));
}

#[test]
fn test_graph_external_dependencies_filtered() {
    // package-a depends on external package (not in workspace)
    let packages = vec![create_package_info(
        "package-a",
        "1.0.0",
        vec![("external-package", "^1.0.0"), ("react", "^18.0.0")],
    )];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 1);
    assert_eq!(graph.edge_count(), 0); // External dependencies not in graph

    // package-a should have no dependencies in the graph
    assert!(graph.dependencies("package-a").is_empty());
}

#[test]
fn test_graph_workspace_protocol_filtered() {
    // Workspace protocol dependencies should be filtered by PackageInfo.all_dependencies()
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "workspace:*")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 2);
    // workspace:* is filtered out by PackageInfo.all_dependencies(), so no edge
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_graph_file_protocol_filtered() {
    // File protocol dependencies should be filtered by PackageInfo.all_dependencies()
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "file:../package-a")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 2);
    // file: protocol is filtered out by PackageInfo.all_dependencies()
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_graph_dev_dependencies_included() {
    // Dev dependencies should be included in the graph
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info_with_dev_deps(
            "package-b",
            "1.0.0",
            vec![],
            vec![("package-a", "^1.0.0")],
        ),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 2);
    assert_eq!(graph.edge_count(), 1);

    // package-b should depend on package-a (via devDependencies)
    let deps_b = graph.dependencies("package-b");
    assert_eq!(deps_b.len(), 1);
    assert!(deps_b.contains(&"package-a".to_string()));
}

#[test]
fn test_graph_circular_dependency_detection_simple() {
    // package-a depends on package-b, package-b depends on package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);

    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 2);
    assert!(cycle.involves("package-a"));
    assert!(cycle.involves("package-b"));
}

#[test]
fn test_graph_circular_dependency_detection_three_packages() {
    // package-a -> package-b -> package-c -> package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-a", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);

    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 3);
    assert!(cycle.involves("package-a"));
    assert!(cycle.involves("package-b"));
    assert!(cycle.involves("package-c"));
}

#[test]
fn test_graph_no_circular_dependencies() {
    // Linear chain with no cycles
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-b", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();
    assert!(cycles.is_empty());
}

#[test]
fn test_graph_multiple_circular_dependencies() {
    // Two separate circular dependency groups
    // Group 1: package-a <-> package-b
    // Group 2: package-c <-> package-d
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-d", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-c", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 2);

    // Check that each cycle contains the correct packages
    for cycle in &cycles {
        assert_eq!(cycle.len(), 2);
        let is_group1 = cycle.involves("package-a") && cycle.involves("package-b");
        let is_group2 = cycle.involves("package-c") && cycle.involves("package-d");
        assert!(is_group1 || is_group2);
    }
}

#[test]
fn test_graph_contains_existing_package() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert!(graph.contains("package-a"));
}

#[test]
fn test_graph_contains_nonexistent_package() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert!(!graph.contains("package-b"));
}

#[test]
fn test_graph_dependents_nonexistent_package() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let dependents = graph.dependents("nonexistent");
    assert!(dependents.is_empty());
}

#[test]
fn test_graph_dependencies_nonexistent_package() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let deps = graph.dependencies("nonexistent");
    assert!(deps.is_empty());
}

#[test]
fn test_graph_all_packages() {
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![]),
        create_package_info("package-c", "1.0.0", vec![]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let all_pkgs = graph.all_packages();
    assert_eq!(all_pkgs.len(), 3);
    assert!(all_pkgs.contains(&"package-a".to_string()));
    assert!(all_pkgs.contains(&"package-b".to_string()));
    assert!(all_pkgs.contains(&"package-c".to_string()));
}

#[test]
fn test_graph_complex_dependency_structure() {
    // Complex structure:
    // package-a (no deps)
    // package-b depends on package-a
    // package-c depends on package-a
    // package-d depends on package-b and package-c
    // package-e depends on package-d
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info(
            "package-d",
            "1.0.0",
            vec![("package-b", "^1.0.0"), ("package-c", "^1.0.0")],
        ),
        create_package_info("package-e", "1.0.0", vec![("package-d", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 5);
    assert_eq!(graph.edge_count(), 5);

    // Verify package-a has two direct dependents
    let dependents_a = graph.dependents("package-a");
    assert_eq!(dependents_a.len(), 2);
    assert!(dependents_a.contains(&"package-b".to_string()));
    assert!(dependents_a.contains(&"package-c".to_string()));

    // Verify package-d has two direct dependencies
    let deps_d = graph.dependencies("package-d");
    assert_eq!(deps_d.len(), 2);
    assert!(deps_d.contains(&"package-b".to_string()));
    assert!(deps_d.contains(&"package-c".to_string()));

    // Verify package-e depends on package-d
    let deps_e = graph.dependencies("package-e");
    assert_eq!(deps_e.len(), 1);
    assert!(deps_e.contains(&"package-d".to_string()));
}

#[test]
fn test_graph_transitive_dependents() {
    // package-c -> package-b -> package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-b", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    // package-a should have both package-b and package-c as transitive dependents
    let transitive = graph.transitive_dependents("package-a");
    assert_eq!(transitive.len(), 2);
    assert!(transitive.contains(&"package-b".to_string()));
    assert!(transitive.contains(&"package-c".to_string()));

    // package-b should have only package-c as transitive dependent
    let transitive_b = graph.transitive_dependents("package-b");
    assert_eq!(transitive_b.len(), 1);
    assert!(transitive_b.contains(&"package-c".to_string()));

    // package-c should have no transitive dependents
    let transitive_c = graph.transitive_dependents("package-c");
    assert!(transitive_c.is_empty());
}

#[test]
fn test_graph_transitive_dependencies() {
    // package-c -> package-b -> package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-b", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    // package-c should transitively depend on both package-b and package-a
    let transitive = graph.transitive_dependencies("package-c");
    assert_eq!(transitive.len(), 2);
    assert!(transitive.contains(&"package-b".to_string()));
    assert!(transitive.contains(&"package-a".to_string()));

    // package-b should transitively depend only on package-a
    let transitive_b = graph.transitive_dependencies("package-b");
    assert_eq!(transitive_b.len(), 1);
    assert!(transitive_b.contains(&"package-a".to_string()));

    // package-a should have no transitive dependencies
    let transitive_a = graph.transitive_dependencies("package-a");
    assert!(transitive_a.is_empty());
}

#[test]
fn test_graph_transitive_nonexistent_package() {
    let packages = vec![create_package_info("package-a", "1.0.0", vec![])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let transitive_deps = graph.transitive_dependencies("nonexistent");
    assert!(transitive_deps.is_empty());

    let transitive_dependents = graph.transitive_dependents("nonexistent");
    assert!(transitive_dependents.is_empty());
}

#[test]
fn test_graph_diamond_dependency_structure() {
    // Diamond structure:
    //     package-d
    //     /       \
    // package-b  package-c
    //     \       /
    //     package-a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info(
            "package-d",
            "1.0.0",
            vec![("package-b", "^1.0.0"), ("package-c", "^1.0.0")],
        ),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    assert_eq!(graph.package_count(), 4);
    assert_eq!(graph.edge_count(), 4);

    // package-a should have two direct dependents
    let dependents_a = graph.dependents("package-a");
    assert_eq!(dependents_a.len(), 2);

    // package-d should transitively depend on all three packages
    let transitive_d = graph.transitive_dependencies("package-d");
    assert_eq!(transitive_d.len(), 3);
    assert!(transitive_d.contains(&"package-a".to_string()));
    assert!(transitive_d.contains(&"package-b".to_string()));
    assert!(transitive_d.contains(&"package-c".to_string()));

    // package-a should have all three packages as transitive dependents
    let transitive_a = graph.transitive_dependents("package-a");
    assert_eq!(transitive_a.len(), 3);
    assert!(transitive_a.contains(&"package-b".to_string()));
    assert!(transitive_a.contains(&"package-c".to_string()));
    assert!(transitive_a.contains(&"package-d".to_string()));
}

// ============================================================================
// Comprehensive Circular Dependency Detection Tests (Story 5.3)
// ============================================================================

#[test]
fn test_graph_self_loop_single_package() {
    // A package that depends on itself (should not create a cycle in our model
    // since we only track internal workspace dependencies, but test anyway)
    let packages = vec![create_package_info("package-a", "1.0.0", vec![("package-a", "^1.0.0")])];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    // Self-loops create a strongly connected component of size 1
    // Tarjan's algorithm will not report this as a cycle since we filter scc.len() > 1
    let cycles = graph.detect_cycles();
    assert!(cycles.is_empty(), "Self-loops should not be detected as cycles");
}

#[test]
fn test_graph_nested_cycles_complex() {
    // Complex nested structure:
    // Cycle 1: a -> b -> a
    // Cycle 2: c -> d -> e -> c
    // Additionally: b -> c (connecting the two cycles)
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info(
            "package-b",
            "1.0.0",
            vec![("package-a", "^1.0.0"), ("package-c", "^1.0.0")],
        ),
        create_package_info("package-c", "1.0.0", vec![("package-d", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-e", "^1.0.0")]),
        create_package_info("package-e", "1.0.0", vec![("package-c", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    // When cycles are connected, Tarjan's algorithm may merge them into one large SCC
    // We should have 1 large SCC containing all 5 packages since they're all connected
    assert!(!cycles.is_empty(), "Should detect interconnected cycles");

    // Verify that all packages in cycles are accounted for
    let mut all_cycle_packages = std::collections::HashSet::new();
    for cycle in &cycles {
        for pkg in &cycle.cycle {
            all_cycle_packages.insert(pkg.as_str());
        }
    }

    // All 5 packages should be part of the detected cycles
    assert_eq!(all_cycle_packages.len(), 5);
    assert!(all_cycle_packages.contains("package-a"));
    assert!(all_cycle_packages.contains("package-b"));
    assert!(all_cycle_packages.contains("package-c"));
    assert!(all_cycle_packages.contains("package-d"));
    assert!(all_cycle_packages.contains("package-e"));
}

#[test]
fn test_graph_cycle_with_independent_packages() {
    // Cycle: a -> b -> c -> a
    // Independent: d -> e (no cycle)
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-e", "^1.0.0")]),
        create_package_info("package-e", "1.0.0", vec![]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    assert_eq!(cycles.len(), 1, "Should detect exactly one cycle");

    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 3, "Cycle should contain 3 packages");
    assert!(cycle.involves("package-a"));
    assert!(cycle.involves("package-b"));
    assert!(cycle.involves("package-c"));
    assert!(!cycle.involves("package-d"), "package-d should not be in the cycle");
    assert!(!cycle.involves("package-e"), "package-e should not be in the cycle");
}

#[test]
fn test_graph_large_cycle_chain() {
    // Large cycle: a -> b -> c -> d -> e -> f -> g -> h -> i -> j -> a
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-d", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-e", "^1.0.0")]),
        create_package_info("package-e", "1.0.0", vec![("package-f", "^1.0.0")]),
        create_package_info("package-f", "1.0.0", vec![("package-g", "^1.0.0")]),
        create_package_info("package-g", "1.0.0", vec![("package-h", "^1.0.0")]),
        create_package_info("package-h", "1.0.0", vec![("package-i", "^1.0.0")]),
        create_package_info("package-i", "1.0.0", vec![("package-j", "^1.0.0")]),
        create_package_info("package-j", "1.0.0", vec![("package-a", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    assert_eq!(cycles.len(), 1, "Should detect exactly one large cycle");

    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 10, "Cycle should contain all 10 packages");

    // Verify all packages are involved
    for letter in &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'] {
        let pkg_name = format!("package-{}", letter);
        assert!(cycle.involves(&pkg_name), "Cycle should involve {}", pkg_name);
    }
}

#[test]
fn test_graph_bidirectional_dependencies() {
    // Multiple bidirectional pairs:
    // a <-> b
    // c <-> d
    // e <-> f
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-d", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-e", "1.0.0", vec![("package-f", "^1.0.0")]),
        create_package_info("package-f", "1.0.0", vec![("package-e", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    assert_eq!(cycles.len(), 3, "Should detect three separate bidirectional cycles");

    // Verify each cycle contains exactly 2 packages
    for cycle in &cycles {
        assert_eq!(cycle.len(), 2, "Each bidirectional cycle should have 2 packages");
    }
}

#[test]
fn test_graph_cycle_display_format() {
    // Test the display_cycle method
    let packages = vec![
        create_package_info("package-a", "1.0.0", vec![("package-b", "^1.0.0")]),
        create_package_info("package-b", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-a", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);

    let cycle = &cycles[0];
    let display = cycle.display_cycle();

    // The display should contain all package names separated by " -> "
    assert!(display.contains("package-a"));
    assert!(display.contains("package-b"));
    assert!(display.contains("package-c"));
    assert!(display.contains(" -> "));
}

#[test]
fn test_graph_complex_interconnected_cycles() {
    // Very complex structure with multiple interconnected cycles:
    // Cycle 1: a -> b -> a
    // Cycle 2: c -> d -> c
    // Bridge: a -> c (connects the cycles into one large SCC)
    // Additional: e -> d (another connection)
    let packages = vec![
        create_package_info(
            "package-a",
            "1.0.0",
            vec![("package-b", "^1.0.0"), ("package-c", "^1.0.0")],
        ),
        create_package_info("package-b", "1.0.0", vec![("package-a", "^1.0.0")]),
        create_package_info("package-c", "1.0.0", vec![("package-d", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![("package-c", "^1.0.0")]),
        create_package_info("package-e", "1.0.0", vec![("package-d", "^1.0.0")]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    assert!(!cycles.is_empty(), "Should detect interconnected cycles");

    // With the bridge, all cyclic packages form one large SCC
    // package-e is not part of the cycle since it only points to d
    let mut total_packages_in_cycles = std::collections::HashSet::new();
    for cycle in &cycles {
        for pkg in &cycle.cycle {
            total_packages_in_cycles.insert(pkg.as_str());
        }
    }

    // a, b, c, d should all be in cycles
    assert!(total_packages_in_cycles.contains("package-a"));
    assert!(total_packages_in_cycles.contains("package-b"));
    assert!(total_packages_in_cycles.contains("package-c"));
    assert!(total_packages_in_cycles.contains("package-d"));
}

#[test]
fn test_graph_no_false_positives() {
    // Ensure that valid dependency trees don't trigger false positives
    // Tree structure (no cycles):
    //        a
    //       / \
    //      b   c
    //     / \   \
    //    d   e   f
    let packages = vec![
        create_package_info(
            "package-a",
            "1.0.0",
            vec![("package-b", "^1.0.0"), ("package-c", "^1.0.0")],
        ),
        create_package_info(
            "package-b",
            "1.0.0",
            vec![("package-d", "^1.0.0"), ("package-e", "^1.0.0")],
        ),
        create_package_info("package-c", "1.0.0", vec![("package-f", "^1.0.0")]),
        create_package_info("package-d", "1.0.0", vec![]),
        create_package_info("package-e", "1.0.0", vec![]),
        create_package_info("package-f", "1.0.0", vec![]),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    let cycles = graph.detect_cycles();

    assert!(cycles.is_empty(), "Tree structure should have no cycles");
}

#[test]
fn test_graph_cycle_with_external_dependencies() {
    // Cycle with external dependencies (external deps should be filtered)
    // Internal cycle: a -> b -> a
    // External deps: a -> "external-lib", b -> "another-lib"
    let packages = vec![
        create_package_info(
            "package-a",
            "1.0.0",
            vec![("package-b", "^1.0.0"), ("external-lib", "^1.0.0")],
        ),
        create_package_info(
            "package-b",
            "1.0.0",
            vec![("package-a", "^1.0.0"), ("another-lib", "^2.0.0")],
        ),
    ];

    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");

    // Graph should only have 2 nodes (internal packages)
    assert_eq!(graph.package_count(), 2);

    // Should still detect the cycle between a and b
    let cycles = graph.detect_cycles();
    assert_eq!(cycles.len(), 1);

    let cycle = &cycles[0];
    assert_eq!(cycle.len(), 2);
    assert!(cycle.involves("package-a"));
    assert!(cycle.involves("package-b"));
}

// ============================================================================
// Performance Tests for Circular Dependency Detection (Story 5.3)
// ============================================================================

#[test]
fn test_graph_performance_100_packages_no_cycles() {
    // Performance test: 100 packages in a linear chain (no cycles)
    // Should complete in < 1s as per acceptance criteria
    let mut packages = Vec::new();

    // Create linear chain: 0 -> 1 -> 2 -> ... -> 99
    packages.push(create_package_info("package-0", "1.0.0", vec![]));
    for i in 1..100 {
        packages.push(create_package_info(
            &format!("package-{}", i),
            "1.0.0",
            vec![(&format!("package-{}", i - 1), "^1.0.0")],
        ));
    }

    let start = std::time::Instant::now();
    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
    let cycles = graph.detect_cycles();
    let elapsed = start.elapsed();

    assert!(cycles.is_empty(), "Linear chain should have no cycles");
    assert_eq!(graph.package_count(), 100);
    assert!(elapsed.as_secs() < 1, "Should complete in under 1 second, took {:?}", elapsed);
}

#[test]
fn test_graph_performance_100_packages_with_cycles() {
    // Performance test: 100 packages with multiple cycles
    // Create 10 separate cycles of 10 packages each
    let mut packages = Vec::new();

    for group in 0..10 {
        for i in 0..10 {
            let pkg_name = format!("package-g{}-{}", group, i);
            let next_i = (i + 1) % 10;
            let dep_name = format!("package-g{}-{}", group, next_i);

            packages.push(create_package_info(&pkg_name, "1.0.0", vec![(&dep_name, "^1.0.0")]));
        }
    }

    let start = std::time::Instant::now();
    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
    let cycles = graph.detect_cycles();
    let elapsed = start.elapsed();

    assert_eq!(graph.package_count(), 100);
    assert_eq!(cycles.len(), 10, "Should detect 10 separate cycles");
    assert!(elapsed.as_secs() < 1, "Should complete in under 1 second, took {:?}", elapsed);

    // Verify each cycle has 10 packages
    for cycle in &cycles {
        assert_eq!(cycle.len(), 10, "Each cycle should have 10 packages");
    }
}

#[test]
fn test_graph_performance_complex_interconnected() {
    // Performance test: Complex interconnected graph
    // Create a mesh-like structure with many cross-dependencies
    let mut packages = Vec::new();

    for i in 0..50 {
        let mut deps = vec![];
        // Each package depends on the next 3 packages (with wraparound)
        for j in 1..=3 {
            let dep_idx = (i + j) % 50;
            deps.push((format!("package-{}", dep_idx), "^1.0.0".to_string()));
        }

        let deps_ref: Vec<(&str, &str)> =
            deps.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
        packages.push(create_package_info(&format!("package-{}", i), "1.0.0", deps_ref));
    }

    let start = std::time::Instant::now();
    let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
    let cycles = graph.detect_cycles();
    let elapsed = start.elapsed();

    assert_eq!(graph.package_count(), 50);
    assert!(!cycles.is_empty(), "Complex mesh should have cycles");
    assert!(elapsed.as_secs() < 1, "Should complete in under 1 second, took {:?}", elapsed);
}

// ============================================================================
// Property-Based Tests for Circular Dependency Detection (Story 5.3)
// ============================================================================

#[cfg(test)]
mod circular_dependency_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_property_no_dependencies_no_cycles(package_count in 1usize..20) {
            // Property: Packages with no dependencies should never have cycles
            let mut packages = Vec::new();
            for i in 0..package_count {
                packages.push(create_package_info(&format!("package-{}", i), "1.0.0", vec![]));
            }

            let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
            let cycles = graph.detect_cycles();

            prop_assert!(cycles.is_empty(), "Packages with no dependencies should have no cycles");
        }

        #[test]
        fn test_property_linear_chain_no_cycles(package_count in 2usize..20) {
            // Property: A linear dependency chain should never have cycles
            let mut packages = Vec::new();
            packages.push(create_package_info("package-0", "1.0.0", vec![]));

            for i in 1..package_count {
                packages.push(create_package_info(
                    &format!("package-{}", i),
                    "1.0.0",
                    vec![(&format!("package-{}", i - 1), "^1.0.0")],
                ));
            }

            let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
            let cycles = graph.detect_cycles();

            prop_assert!(cycles.is_empty(), "Linear chains should have no cycles");
        }

        #[test]
        fn test_property_simple_cycle_always_detected(package_count in 2usize..10) {
            // Property: A simple circular chain should always be detected
            // Create cycle: 0 -> 1 -> 2 -> ... -> (n-1) -> 0
            let mut packages = Vec::new();

            for i in 0..package_count {
                let next = (i + 1) % package_count;
                packages.push(create_package_info(
                    &format!("package-{}", i),
                    "1.0.0",
                    vec![(&format!("package-{}", next), "^1.0.0")],
                ));
            }

            let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
            let cycles = graph.detect_cycles();

            prop_assert!(!cycles.is_empty(), "Circular chain should be detected");
            prop_assert_eq!(cycles.len(), 1, "Should detect exactly one cycle");
            prop_assert_eq!(cycles[0].len(), package_count, "Cycle should contain all packages");
        }

        #[test]
        fn test_property_bidirectional_is_cycle(name1_idx in 0usize..50, name2_idx in 0usize..50) {
            // Property: Two packages with bidirectional dependencies form a cycle
            if name1_idx == name2_idx {
                return Ok(());
            }

            let name1 = format!("package-{}", name1_idx);
            let name2 = format!("package-{}", name2_idx);

            let packages = vec![
                create_package_info(&name1, "1.0.0", vec![(&name2, "^1.0.0")]),
                create_package_info(&name2, "1.0.0", vec![(&name1, "^1.0.0")]),
            ];

            let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
            let cycles = graph.detect_cycles();

            prop_assert_eq!(cycles.len(), 1, "Bidirectional dependency should create one cycle");
            prop_assert_eq!(cycles[0].len(), 2, "Cycle should contain both packages");
            prop_assert!(cycles[0].involves(&name1), "Cycle should involve first package");
            prop_assert!(cycles[0].involves(&name2), "Cycle should involve second package");
        }

        #[test]
        fn test_property_tree_no_cycles(depth in 1usize..5, branching in 1usize..4) {
            // Property: A tree structure (no back edges) should have no cycles
            let mut packages = Vec::new();
            let mut counter = 0;

            fn add_tree_level(
                packages: &mut Vec<PackageInfo>,
                counter: &mut usize,
                parent: Option<String>,
                depth: usize,
                branching: usize,
            ) {
                if depth == 0 {
                    return;
                }

                for _ in 0..branching {
                    let name = format!("package-{}", *counter);
                    *counter += 1;

                    let deps = if let Some(ref p) = parent {
                        vec![(p.as_str(), "^1.0.0")]
                    } else {
                        vec![]
                    };

                    packages.push(create_package_info(&name, "1.0.0", deps));
                    add_tree_level(packages, counter, Some(name), depth - 1, branching);
                }
            }

            add_tree_level(&mut packages, &mut counter, None, depth, branching);

            if packages.is_empty() {
                return Ok(());
            }

            let graph = DependencyGraph::from_packages(&packages).expect("Failed to create graph");
            let cycles = graph.detect_cycles();

            prop_assert!(cycles.is_empty(), "Tree structure should have no cycles");
        }
    }

    // ============================================================================
    // Version Resolution Tests (Story 5.4)
    // ============================================================================

    #[allow(clippy::unwrap_used)]
    mod resolution_tests {
        use super::*;
        use crate::types::{
            Changeset, DependencyType, UpdateReason, Version, VersionBump, VersioningStrategy,
        };
        use crate::version::resolution::{PackageUpdate, VersionResolution, resolve_versions};
        use std::collections::HashMap;

        /// Test resolving versions with independent strategy and minor bump
        #[tokio::test]
        async fn test_resolve_independent_minor_bump() {
            let mut changeset = Changeset::new(
                "feature/new-api",
                VersionBump::Minor,
                vec!["production".to_string()],
            );
            changeset.add_package("@myorg/core");
            changeset.add_package("@myorg/utils");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.2.3", vec![]),
            );
            packages.insert(
                "@myorg/utils".to_string(),
                create_package_info("@myorg/utils", "0.5.0", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 2);
            assert!(resolution.circular_dependencies.is_empty());

            // Check core package
            let core_update = resolution.updates.iter().find(|u| u.name == "@myorg/core").unwrap();
            assert_eq!(core_update.current_version, Version::parse("1.2.3").unwrap());
            assert_eq!(core_update.next_version, Version::parse("1.3.0").unwrap());
            assert!(matches!(core_update.reason, UpdateReason::DirectChange));

            // Check utils package
            let utils_update =
                resolution.updates.iter().find(|u| u.name == "@myorg/utils").unwrap();
            assert_eq!(utils_update.current_version, Version::parse("0.5.0").unwrap());
            assert_eq!(utils_update.next_version, Version::parse("0.6.0").unwrap());
            assert!(matches!(utils_update.reason, UpdateReason::DirectChange));
        }

        /// Test resolving versions with independent strategy and major bump
        #[tokio::test]
        async fn test_resolve_independent_major_bump() {
            let mut changeset = Changeset::new(
                "breaking/api-changes",
                VersionBump::Major,
                vec!["production".to_string()],
            );
            changeset.add_package("@myorg/core");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.2.3", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 1);

            let update = &resolution.updates[0];
            assert_eq!(update.name, "@myorg/core");
            assert_eq!(update.current_version, Version::parse("1.2.3").unwrap());
            assert_eq!(update.next_version, Version::parse("2.0.0").unwrap());
            assert!(update.is_direct_change());
            assert!(!update.is_propagated());
        }

        /// Test resolving versions with independent strategy and patch bump
        #[tokio::test]
        async fn test_resolve_independent_patch_bump() {
            let mut changeset =
                Changeset::new("fix/bug-fix", VersionBump::Patch, vec!["production".to_string()]);
            changeset.add_package("@myorg/core");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.2.3", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 1);

            let update = &resolution.updates[0];
            assert_eq!(update.current_version, Version::parse("1.2.3").unwrap());
            assert_eq!(update.next_version, Version::parse("1.2.4").unwrap());
        }

        /// Test resolving versions with independent strategy and no bump
        #[tokio::test]
        async fn test_resolve_independent_no_bump() {
            let mut changeset = Changeset::new(
                "docs/update-readme",
                VersionBump::None,
                vec!["production".to_string()],
            );
            changeset.add_package("@myorg/core");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.2.3", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 1);

            let update = &resolution.updates[0];
            assert_eq!(update.current_version, Version::parse("1.2.3").unwrap());
            assert_eq!(update.next_version, Version::parse("1.2.3").unwrap());
        }

        /// Test resolving versions with unified strategy
        #[tokio::test]
        async fn test_resolve_unified_strategy() {
            let mut changeset = Changeset::new(
                "feature/unified",
                VersionBump::Minor,
                vec!["production".to_string()],
            );
            changeset.add_package("@myorg/core");
            changeset.add_package("@myorg/utils");
            changeset.add_package("@myorg/cli");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.5.0", vec![]),
            );
            packages.insert(
                "@myorg/utils".to_string(),
                create_package_info("@myorg/utils", "1.2.0", vec![]),
            );
            packages.insert(
                "@myorg/cli".to_string(),
                create_package_info("@myorg/cli", "1.3.5", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Unified).await.unwrap();

            assert_eq!(resolution.updates.len(), 3);

            // All packages should have the same next version (based on highest current version)
            // Highest is 1.5.0, so next should be 1.6.0
            let expected_next = Version::parse("1.6.0").unwrap();

            for update in &resolution.updates {
                assert_eq!(update.next_version, expected_next);
                assert!(update.is_direct_change());
            }

            // Verify individual current versions
            let core = resolution.updates.iter().find(|u| u.name == "@myorg/core").unwrap();
            assert_eq!(core.current_version, Version::parse("1.5.0").unwrap());

            let utils = resolution.updates.iter().find(|u| u.name == "@myorg/utils").unwrap();
            assert_eq!(utils.current_version, Version::parse("1.2.0").unwrap());

            let cli = resolution.updates.iter().find(|u| u.name == "@myorg/cli").unwrap();
            assert_eq!(cli.current_version, Version::parse("1.3.5").unwrap());
        }

        /// Test unified strategy with major bump
        #[tokio::test]
        async fn test_resolve_unified_major_bump() {
            let mut changeset =
                Changeset::new("breaking/v2", VersionBump::Major, vec!["production".to_string()]);
            changeset.add_package("@myorg/core");
            changeset.add_package("@myorg/utils");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.9.9", vec![]),
            );
            packages.insert(
                "@myorg/utils".to_string(),
                create_package_info("@myorg/utils", "1.5.0", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Unified).await.unwrap();

            assert_eq!(resolution.updates.len(), 2);

            // Highest version is 1.9.9, major bump gives 2.0.0
            let expected_next = Version::parse("2.0.0").unwrap();

            for update in &resolution.updates {
                assert_eq!(update.next_version, expected_next);
            }
        }

        /// Test error when package not found
        #[tokio::test]
        async fn test_resolve_package_not_found() {
            let mut changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
            changeset.add_package("@myorg/nonexistent");

            let packages = HashMap::new();

            let result =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent).await;

            assert!(result.is_err());
            match result.unwrap_err() {
                crate::error::VersionError::PackageNotFound { name, workspace_root: _ } => {
                    assert_eq!(name, "@myorg/nonexistent");
                }
                _ => panic!("Expected PackageNotFound error"),
            }
        }

        /// Test error when one of multiple packages not found
        #[tokio::test]
        async fn test_resolve_multiple_packages_one_not_found() {
            let mut changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
            changeset.add_package("@myorg/core");
            changeset.add_package("@myorg/missing");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.0.0", vec![]),
            );

            let result =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent).await;

            assert!(result.is_err());
            match result.unwrap_err() {
                crate::error::VersionError::PackageNotFound { name, workspace_root: _ } => {
                    assert_eq!(name, "@myorg/missing");
                }
                _ => panic!("Expected PackageNotFound error"),
            }
        }

        /// Test empty changeset (no packages)
        #[tokio::test]
        async fn test_resolve_empty_changeset() {
            let changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);

            let packages = HashMap::new();

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert!(resolution.updates.is_empty());
            assert!(!resolution.has_updates());
            assert_eq!(resolution.update_count(), 0);
        }

        /// Test VersionResolution methods
        #[test]
        fn test_version_resolution_methods() {
            let mut resolution = VersionResolution::new();

            assert!(!resolution.has_updates());
            assert_eq!(resolution.update_count(), 0);
            assert!(!resolution.has_circular_dependencies());

            let update = PackageUpdate::new(
                "@myorg/core".to_string(),
                PathBuf::from("/workspace/packages/core"),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.1.0").unwrap(),
                UpdateReason::DirectChange,
            );

            resolution.add_update(update);

            assert!(resolution.has_updates());
            assert_eq!(resolution.update_count(), 1);
        }

        /// Test PackageUpdate methods
        #[test]
        fn test_package_update_methods() {
            let update = PackageUpdate::new(
                "@myorg/core".to_string(),
                PathBuf::from("/workspace/packages/core"),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.1.0").unwrap(),
                UpdateReason::DirectChange,
            );

            assert!(update.is_direct_change());
            assert!(!update.is_propagated());
            assert_eq!(update.name, "@myorg/core");
            assert_eq!(update.dependency_updates.len(), 0);
        }

        /// Test propagated update reason
        #[test]
        fn test_package_update_propagated() {
            let update = PackageUpdate::new(
                "@myorg/utils".to_string(),
                PathBuf::from("/workspace/packages/utils"),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.0.1").unwrap(),
                UpdateReason::DependencyPropagation {
                    triggered_by: "@myorg/core".to_string(),
                    depth: 1,
                },
            );

            assert!(!update.is_direct_change());
            assert!(update.is_propagated());

            match &update.reason {
                UpdateReason::DependencyPropagation { triggered_by, depth } => {
                    assert_eq!(triggered_by, "@myorg/core");
                    assert_eq!(*depth, 1);
                }
                _ => panic!("Expected DependencyPropagation reason"),
            }
        }

        /// Test VersionResolution default
        #[test]
        fn test_version_resolution_default() {
            let resolution = VersionResolution::default();

            assert!(!resolution.has_updates());
            assert!(!resolution.has_circular_dependencies());
            assert_eq!(resolution.updates.len(), 0);
            assert_eq!(resolution.circular_dependencies.len(), 0);
        }

        /// Test resolving with prerelease versions
        #[tokio::test]
        async fn test_resolve_with_prerelease_versions() {
            let mut changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
            changeset.add_package("@myorg/core");

            let mut packages = HashMap::new();
            packages.insert(
                "@myorg/core".to_string(),
                create_package_info("@myorg/core", "1.0.0-beta.1", vec![]),
            );

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 1);

            let update = &resolution.updates[0];
            assert_eq!(update.current_version, Version::parse("1.0.0-beta.1").unwrap());
            // Minor bump on 1.0.0-beta.1 should give 1.1.0
            assert_eq!(update.next_version, Version::parse("1.1.0").unwrap());
        }

        /// Test resolving multiple packages with different versions (independent)
        #[tokio::test]
        async fn test_resolve_independent_different_versions() {
            let mut changeset =
                Changeset::new("feature/multi", VersionBump::Patch, vec!["production".to_string()]);
            changeset.add_package("@myorg/a");
            changeset.add_package("@myorg/b");
            changeset.add_package("@myorg/c");

            let mut packages = HashMap::new();
            packages
                .insert("@myorg/a".to_string(), create_package_info("@myorg/a", "1.0.0", vec![]));
            packages
                .insert("@myorg/b".to_string(), create_package_info("@myorg/b", "2.5.3", vec![]));
            packages
                .insert("@myorg/c".to_string(), create_package_info("@myorg/c", "0.1.0", vec![]));

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Independent)
                    .await
                    .unwrap();

            assert_eq!(resolution.updates.len(), 3);

            let a_update = resolution.updates.iter().find(|u| u.name == "@myorg/a").unwrap();
            assert_eq!(a_update.next_version, Version::parse("1.0.1").unwrap());

            let b_update = resolution.updates.iter().find(|u| u.name == "@myorg/b").unwrap();
            assert_eq!(b_update.next_version, Version::parse("2.5.4").unwrap());

            let c_update = resolution.updates.iter().find(|u| u.name == "@myorg/c").unwrap();
            assert_eq!(c_update.next_version, Version::parse("0.1.1").unwrap());
        }

        /// Test unified strategy with same starting versions
        #[tokio::test]
        async fn test_resolve_unified_same_versions() {
            let mut changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
            changeset.add_package("@myorg/a");
            changeset.add_package("@myorg/b");

            let mut packages = HashMap::new();
            packages
                .insert("@myorg/a".to_string(), create_package_info("@myorg/a", "1.0.0", vec![]));
            packages
                .insert("@myorg/b".to_string(), create_package_info("@myorg/b", "1.0.0", vec![]));

            let resolution =
                resolve_versions(&changeset, &packages, VersioningStrategy::Unified).await.unwrap();

            assert_eq!(resolution.updates.len(), 2);

            for update in &resolution.updates {
                assert_eq!(update.current_version, Version::parse("1.0.0").unwrap());
                assert_eq!(update.next_version, Version::parse("1.1.0").unwrap());
            }
        }

        /// Test DependencyType enum
        #[test]
        fn test_dependency_type() {
            let regular = DependencyType::Regular;
            let dev = DependencyType::Dev;
            let peer = DependencyType::Peer;

            assert_ne!(regular, dev);
            assert_ne!(regular, peer);
            assert_ne!(dev, peer);
        }

        /// Test serialization of VersionResolution
        #[test]
        fn test_version_resolution_serialization() {
            let mut resolution = VersionResolution::new();

            let update = PackageUpdate::new(
                "@myorg/core".to_string(),
                PathBuf::from("/workspace/packages/core"),
                Version::parse("1.0.0").unwrap(),
                Version::parse("1.1.0").unwrap(),
                UpdateReason::DirectChange,
            );

            resolution.add_update(update);

            let json = serde_json::to_string(&resolution).unwrap();
            let deserialized: VersionResolution = serde_json::from_str(&json).unwrap();

            assert_eq!(resolution, deserialized);
        }

        /// Test UpdateReason serialization
        #[test]
        fn test_update_reason_serialization() {
            let direct = UpdateReason::DirectChange;
            let json = serde_json::to_string(&direct).unwrap();
            let deserialized: UpdateReason = serde_json::from_str(&json).unwrap();
            assert_eq!(direct, deserialized);

            let propagated = UpdateReason::DependencyPropagation {
                triggered_by: "@myorg/core".to_string(),
                depth: 2,
            };
            let json = serde_json::to_string(&propagated).unwrap();
            let deserialized: UpdateReason = serde_json::from_str(&json).unwrap();
            assert_eq!(propagated, deserialized);
        }

        /// Test resolving with VersionResolver.resolve_versions method
        #[tokio::test]
        async fn test_version_resolver_resolve_versions_integration() {
            let temp_dir = tempfile::tempdir().unwrap();
            let workspace_root = temp_dir.path();

            // Create single package
            let package_json = serde_json::json!({
                "name": "@myorg/test",
                "version": "1.0.0"
            });

            std::fs::write(
                workspace_root.join("package.json"),
                serde_json::to_string_pretty(&package_json).unwrap(),
            )
            .unwrap();

            let config = crate::config::PackageToolsConfig::default();
            let resolver =
                VersionResolver::new(workspace_root.to_path_buf(), config).await.unwrap();

            let mut changeset =
                Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
            changeset.add_package("@myorg/test");

            let resolution = resolver.resolve_versions(&changeset).await.unwrap();

            assert_eq!(resolution.updates.len(), 1);
            assert_eq!(resolution.updates[0].name, "@myorg/test");
            assert_eq!(resolution.updates[0].current_version, Version::parse("1.0.0").unwrap());
            assert_eq!(resolution.updates[0].next_version, Version::parse("1.1.0").unwrap());
        }
    }

    // ============================================================================
    // Dependency Propagation Tests
    // ============================================================================

    #[allow(clippy::unwrap_used)]
    mod propagation_tests {
        use super::*;
        use crate::config::DependencyConfig;
        use crate::types::{DependencyType, UpdateReason, Version};
        use crate::version::propagation::DependencyPropagator;
        use crate::version::{PackageUpdate, VersionResolution};

        /// Creates a simple monorepo structure for propagation testing.
        ///
        /// Structure:
        /// - pkg-a (no dependencies)
        /// - pkg-b (depends on pkg-a)
        /// - pkg-c (depends on pkg-b)
        fn create_propagation_packages() -> (HashMap<String, PackageInfo>, DependencyGraph) {
            let pkg_a = create_package_info("@test/pkg-a", "1.0.0", vec![]);
            let pkg_b =
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "^1.0.0")]);
            let pkg_c =
                create_package_info("@test/pkg-c", "1.0.0", vec![("@test/pkg-b", "^1.0.0")]);

            let packages_for_graph = vec![
                create_package_info("@test/pkg-a", "1.0.0", vec![]),
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "^1.0.0")]),
                create_package_info("@test/pkg-c", "1.0.0", vec![("@test/pkg-b", "^1.0.0")]),
            ];

            let mut packages = HashMap::new();
            packages.insert("@test/pkg-a".to_string(), pkg_a);
            packages.insert("@test/pkg-b".to_string(), pkg_b);
            packages.insert("@test/pkg-c".to_string(), pkg_c);

            let graph = DependencyGraph::from_packages(&packages_for_graph).unwrap();

            (packages, graph)
        }

        #[test]
        fn test_propagation_basic_chain() {
            let (packages, graph) = create_propagation_packages();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            // Start with pkg-a getting updated
            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            // Propagate changes
            propagator.propagate(&mut resolution).unwrap();

            // Should have 3 updates: pkg-a (direct), pkg-b and pkg-c (propagated)
            assert_eq!(resolution.updates.len(), 3);

            // Find pkg-b update
            let pkg_b_update = resolution
                .updates
                .iter()
                .find(|u| u.name == "@test/pkg-b")
                .expect("pkg-b should be updated");
            assert_eq!(pkg_b_update.current_version, Version::new(1, 0, 0));
            assert_eq!(pkg_b_update.next_version, Version::new(1, 0, 1)); // patch bump
            assert!(pkg_b_update.is_propagated());

            // Find pkg-c update
            let pkg_c_update = resolution
                .updates
                .iter()
                .find(|u| u.name == "@test/pkg-c")
                .expect("pkg-c should be updated");
            assert_eq!(pkg_c_update.current_version, Version::new(1, 0, 0));
            assert_eq!(pkg_c_update.next_version, Version::new(1, 0, 1)); // patch bump
            assert!(pkg_c_update.is_propagated());
        }

        #[test]
        fn test_propagation_with_minor_bump() {
            let (packages, graph) = create_propagation_packages();
            let config =
                DependencyConfig { propagation_bump: "minor".to_string(), ..Default::default() };
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // pkg-b should get minor bump
            let pkg_b_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-b").unwrap();
            assert_eq!(pkg_b_update.next_version, Version::new(1, 1, 0));
        }

        #[test]
        fn test_propagation_respects_max_depth() {
            let (packages, graph) = create_propagation_packages();
            let config = DependencyConfig { max_depth: 1, ..Default::default() };
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // Should only propagate to pkg-b (depth 1), not pkg-c (depth 2)
            assert_eq!(resolution.updates.len(), 2);
            assert!(resolution.updates.iter().any(|u| u.name == "@test/pkg-a"));
            assert!(resolution.updates.iter().any(|u| u.name == "@test/pkg-b"));
            assert!(!resolution.updates.iter().any(|u| u.name == "@test/pkg-c"));
        }

        #[test]
        fn test_propagation_skips_workspace_protocol() {
            let pkg_a = create_package_info("@test/pkg-a", "1.0.0", vec![]);
            let pkg_b =
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "workspace:*")]);

            let packages_for_graph = vec![
                create_package_info("@test/pkg-a", "1.0.0", vec![]),
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "workspace:*")]),
            ];

            let mut packages = HashMap::new();
            packages.insert("@test/pkg-a".to_string(), pkg_a);
            packages.insert("@test/pkg-b".to_string(), pkg_b);

            let graph = DependencyGraph::from_packages(&packages_for_graph).unwrap();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // pkg-b should NOT be propagated because it uses workspace:*
            assert_eq!(resolution.updates.len(), 1);
            assert_eq!(resolution.updates[0].name, "@test/pkg-a");
        }

        #[test]
        fn test_propagation_skips_dev_dependencies_by_default() {
            let pkg_a = create_package_info("@test/pkg-a", "1.0.0", vec![]);
            let pkg_b = create_package_info_with_dev_deps(
                "@test/pkg-b",
                "1.0.0",
                vec![],
                vec![("@test/pkg-a", "^1.0.0")],
            );

            let packages_for_graph = vec![
                create_package_info("@test/pkg-a", "1.0.0", vec![]),
                create_package_info_with_dev_deps(
                    "@test/pkg-b",
                    "1.0.0",
                    vec![],
                    vec![("@test/pkg-a", "^1.0.0")],
                ),
            ];

            let mut packages = HashMap::new();
            packages.insert("@test/pkg-a".to_string(), pkg_a);
            packages.insert("@test/pkg-b".to_string(), pkg_b);

            let graph = DependencyGraph::from_packages(&packages_for_graph).unwrap();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // pkg-b should NOT be propagated (devDependencies disabled by default)
            assert_eq!(resolution.updates.len(), 1);
        }

        #[test]
        fn test_propagation_includes_dev_dependencies_when_enabled() {
            let pkg_a = create_package_info("@test/pkg-a", "1.0.0", vec![]);
            let pkg_b = create_package_info_with_dev_deps(
                "@test/pkg-b",
                "1.0.0",
                vec![],
                vec![("@test/pkg-a", "^1.0.0")],
            );

            let packages_for_graph = vec![
                create_package_info("@test/pkg-a", "1.0.0", vec![]),
                create_package_info_with_dev_deps(
                    "@test/pkg-b",
                    "1.0.0",
                    vec![],
                    vec![("@test/pkg-a", "^1.0.0")],
                ),
            ];

            let mut packages = HashMap::new();
            packages.insert("@test/pkg-a".to_string(), pkg_a);
            packages.insert("@test/pkg-b".to_string(), pkg_b);

            let graph = DependencyGraph::from_packages(&packages_for_graph).unwrap();
            let config =
                DependencyConfig { propagate_dev_dependencies: true, ..Default::default() };
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // pkg-b SHOULD be propagated when dev deps are enabled
            assert_eq!(resolution.updates.len(), 2);
            assert!(resolution.updates.iter().any(|u| u.name == "@test/pkg-b"));
        }

        #[test]
        fn test_propagation_updates_dependency_specs() {
            let (packages, graph) = create_propagation_packages();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // pkg-b should have dependency updates
            let pkg_b_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-b").unwrap();

            assert_eq!(pkg_b_update.dependency_updates.len(), 1);
            let dep_update = &pkg_b_update.dependency_updates[0];
            assert_eq!(dep_update.dependency_name, "@test/pkg-a");
            assert_eq!(dep_update.old_version_spec, "^1.0.0");
            assert_eq!(dep_update.new_version_spec, "^2.0.0");
            assert_eq!(dep_update.dependency_type, DependencyType::Regular);
        }

        #[test]
        fn test_propagation_preserves_range_operators() {
            let pkg_a = create_package_info("@test/pkg-a", "1.0.0", vec![]);
            let pkg_b =
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "~1.0.0")]);

            let packages_for_graph = vec![
                create_package_info("@test/pkg-a", "1.0.0", vec![]),
                create_package_info("@test/pkg-b", "1.0.0", vec![("@test/pkg-a", "~1.0.0")]),
            ];

            let mut packages = HashMap::new();
            packages.insert("@test/pkg-a".to_string(), pkg_a);
            packages.insert("@test/pkg-b".to_string(), pkg_b);

            let graph = DependencyGraph::from_packages(&packages_for_graph).unwrap();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            let pkg_b_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-b").unwrap();

            // Should preserve ~ operator
            assert_eq!(pkg_b_update.dependency_updates[0].new_version_spec, "~2.0.0");
        }

        #[test]
        fn test_propagation_with_none_bump() {
            let (packages, graph) = create_propagation_packages();
            let config =
                DependencyConfig { propagation_bump: "none".to_string(), ..Default::default() };
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // With none bump, only dependency specs are updated, not versions
            // However, dependents still get updates for dependency spec changes
            assert!(resolution.updates.len() >= 2);
        }

        #[test]
        fn test_propagation_tracks_depth() {
            let (packages, graph) = create_propagation_packages();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            propagator.propagate(&mut resolution).unwrap();

            // Check depth tracking
            let pkg_b_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-b").unwrap();

            if let UpdateReason::DependencyPropagation { depth, .. } = &pkg_b_update.reason {
                assert_eq!(depth, &1);
            } else {
                panic!("Expected DependencyPropagation reason");
            }

            let pkg_c_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-c").unwrap();

            if let UpdateReason::DependencyPropagation { depth, .. } = &pkg_c_update.reason {
                assert_eq!(depth, &2);
            } else {
                panic!("Expected DependencyPropagation reason");
            }
        }

        #[test]
        fn test_propagation_no_duplicate_updates() {
            let (packages, graph) = create_propagation_packages();
            let config = DependencyConfig::default();
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            // Propagate twice
            propagator.propagate(&mut resolution).unwrap();
            propagator.propagate(&mut resolution).unwrap();

            // Should still only have unique updates
            let mut names: Vec<_> = resolution.updates.iter().map(|u| u.name.as_str()).collect();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), resolution.updates.len());
        }

        #[test]
        fn test_propagation_invalid_bump_type() {
            let (packages, graph) = create_propagation_packages();
            let config =
                DependencyConfig { propagation_bump: "invalid".to_string(), ..Default::default() };
            let propagator = DependencyPropagator::new(&graph, &packages, &config);

            let mut resolution = VersionResolution::new();
            resolution.add_update(PackageUpdate::new(
                "@test/pkg-a".to_string(),
                PathBuf::from("/test/pkg-a"),
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                UpdateReason::DirectChange,
            ));

            // Should return error for invalid bump type
            let result = propagator.propagate(&mut resolution);
            assert!(result.is_err());
        }
    }
}

// ============================================================================
// Story 5.6: Snapshot Version Generation Tests
// ============================================================================

mod snapshot_tests {
    use crate::error::VersionError;
    use crate::types::Version;
    use crate::version::snapshot::{SnapshotContext, SnapshotGenerator, SnapshotVariable};

    #[test]
    fn test_snapshot_generator_new_with_valid_format() {
        let generator = SnapshotGenerator::new("{version}-{branch}.{commit}");
        assert!(generator.is_ok());

        let generator = generator.expect("Should create generator");
        assert_eq!(generator.format(), "{version}-{branch}.{commit}");
    }

    #[test]
    fn test_snapshot_generator_new_with_empty_format() {
        let result = SnapshotGenerator::new("");
        assert!(result.is_err());

        match result.expect_err("Should fail with empty format") {
            VersionError::SnapshotFailed { reason, .. } => {
                assert!(reason.contains("cannot be empty"));
            }
            _ => panic!("Expected SnapshotFailed error"),
        }
    }

    #[test]
    fn test_snapshot_generator_new_without_version_variable() {
        let result = SnapshotGenerator::new("{branch}.{commit}");
        assert!(result.is_err());

        match result.expect_err("Should fail without version variable") {
            VersionError::SnapshotFailed { reason, .. } => {
                assert!(reason.contains("must contain {version}"));
            }
            _ => panic!("Expected SnapshotFailed error"),
        }
    }

    #[test]
    fn test_snapshot_generator_new_with_unsupported_variable() {
        let result = SnapshotGenerator::new("{version}-{invalid}");
        assert!(result.is_err());

        match result.expect_err("Should fail with unsupported variable") {
            VersionError::SnapshotFailed { reason, .. } => {
                assert!(reason.contains("unsupported variable"));
                assert!(reason.contains("{invalid}"));
            }
            _ => panic!("Expected SnapshotFailed error"),
        }
    }

    #[test]
    fn test_snapshot_generator_validate() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");
        assert!(generator.validate().is_ok());
    }

    #[test]
    fn test_snapshot_generator_variables() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");

        let vars = generator.variables();
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&SnapshotVariable::Version));
        assert!(vars.contains(&SnapshotVariable::Branch));
        assert!(vars.contains(&SnapshotVariable::Commit));
        assert!(!vars.contains(&SnapshotVariable::Timestamp));
    }

    #[test]
    fn test_snapshot_generator_variables_with_timestamp() {
        let generator = SnapshotGenerator::new("{version}-snapshot.{timestamp}")
            .expect("Should create generator");

        let vars = generator.variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&SnapshotVariable::Version));
        assert!(vars.contains(&SnapshotVariable::Timestamp));
    }

    #[test]
    fn test_snapshot_generator_variables_deduplication() {
        let generator = SnapshotGenerator::new("{version}-{version}.{branch}-{branch}")
            .expect("Should create generator");

        let vars = generator.variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&SnapshotVariable::Version));
        assert!(vars.contains(&SnapshotVariable::Branch));
    }

    #[test]
    fn test_snapshot_generator_generate_basic() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.2.3").expect("Should parse version"),
            "feat/oauth".to_string(),
            "abc123def456".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.2.3-feat-oauth.abc123d");
    }

    #[test]
    fn test_snapshot_generator_generate_with_timestamp() {
        let generator = SnapshotGenerator::new("{version}-snapshot.{timestamp}")
            .expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("2.0.0").expect("Should parse version"),
            "main".to_string(),
            "def456abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "2.0.0-snapshot.1640000000");
    }

    #[test]
    fn test_snapshot_generator_generate_version_only() {
        let generator = SnapshotGenerator::new("{version}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("3.1.4").expect("Should parse version"),
            "develop".to_string(),
            "xyz789".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "3.1.4");
    }

    #[test]
    fn test_snapshot_generator_generate_all_variables() {
        let generator = SnapshotGenerator::new("{version}-{branch}.{commit}.{timestamp}")
            .expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "feature/new-api".to_string(),
            "aabbccddee".to_string(),
            1234567890,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-feature-new-api.aabbccd.1234567890");
    }

    #[test]
    fn test_snapshot_context_new_with_current_timestamp() {
        let context = SnapshotContext::new(
            Version::parse("1.2.3").expect("Should parse version"),
            "main".to_string(),
            "abc123".to_string(),
        );

        assert_eq!(context.version.to_string(), "1.2.3");
        assert_eq!(context.branch, "main");
        assert_eq!(context.commit, "abc123");
        assert!(context.timestamp > 0);
    }

    #[test]
    fn test_snapshot_context_with_timestamp() {
        let context = SnapshotContext::with_timestamp(
            Version::parse("2.3.4").expect("Should parse version"),
            "develop".to_string(),
            "def456".to_string(),
            1640000000,
        );

        assert_eq!(context.version.to_string(), "2.3.4");
        assert_eq!(context.branch, "develop");
        assert_eq!(context.commit, "def456");
        assert_eq!(context.timestamp, 1640000000);
    }

    #[test]
    fn test_branch_sanitization_slashes() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "feat/oauth".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-feat-oauth");
    }

    #[test]
    fn test_branch_sanitization_uppercase() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "FIX/JIRA-123".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-fix-jira-123");
    }

    #[test]
    fn test_branch_sanitization_special_characters() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "feature/api__v2@test".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-feature-api__v2test");
    }

    #[test]
    fn test_branch_sanitization_multiple_hyphens() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "feat---multiple---hyphens".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-feat-multiple-hyphens");
    }

    #[test]
    fn test_branch_sanitization_leading_trailing_hyphens() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "-feature-branch-".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-feature-branch");
    }

    #[test]
    fn test_commit_hash_shortening() {
        let generator =
            SnapshotGenerator::new("{version}.{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "main".to_string(),
            "abc123def456789012345678".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0.abc123d");
    }

    #[test]
    fn test_commit_hash_short_not_shortened() {
        let generator =
            SnapshotGenerator::new("{version}.{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "main".to_string(),
            "abc12".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0.abc12");
    }

    #[test]
    fn test_format_with_mixed_separators() {
        let generator =
            SnapshotGenerator::new("{version}+{branch}-{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.2.3").expect("Should parse version"),
            "main".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.2.3+main-abc123");
    }

    #[test]
    fn test_format_with_no_separators() {
        let generator =
            SnapshotGenerator::new("{version}{branch}{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "dev".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0devabc123");
    }

    #[test]
    fn test_different_formats_produce_different_snapshots() {
        let context = SnapshotContext::with_timestamp(
            Version::parse("1.2.3").expect("Should parse version"),
            "feat/new".to_string(),
            "abc123def".to_string(),
            1640000000,
        );

        let gen1 =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");
        let snapshot1 = gen1.generate(&context).expect("Should generate");

        let gen2 =
            SnapshotGenerator::new("{version}.{commit}-{branch}").expect("Should create generator");
        let snapshot2 = gen2.generate(&context).expect("Should generate");

        assert_ne!(snapshot1, snapshot2);
    }

    #[test]
    fn test_snapshot_variable_equality() {
        assert_eq!(SnapshotVariable::Version, SnapshotVariable::Version);
        assert_eq!(SnapshotVariable::Branch, SnapshotVariable::Branch);
        assert_eq!(SnapshotVariable::Commit, SnapshotVariable::Commit);
        assert_eq!(SnapshotVariable::Timestamp, SnapshotVariable::Timestamp);

        assert_ne!(SnapshotVariable::Version, SnapshotVariable::Branch);
        assert_ne!(SnapshotVariable::Commit, SnapshotVariable::Timestamp);
    }

    #[test]
    fn test_complex_branch_names() {
        let test_cases = vec![
            ("feature/PROJ-123-add-auth", "feature-proj-123-add-auth"),
            ("hotfix/bug_fix_v2", "hotfix-bug_fix_v2"),
            ("release/2.0.0-beta", "release-2.0.0-beta"),
            ("feat/user@domain.com", "feat-userdomain.com"),
            ("fix/issue#456", "fix-issue456"),
        ];

        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        for (input_branch, expected_sanitized) in test_cases {
            let context = SnapshotContext::with_timestamp(
                Version::parse("1.0.0").expect("Should parse version"),
                input_branch.to_string(),
                "abc123".to_string(),
                1640000000,
            );

            let snapshot = generator.generate(&context).expect("Should generate snapshot");
            let expected = format!("1.0.0-{}", expected_sanitized);
            assert_eq!(snapshot, expected, "Failed for branch: {}", input_branch);
        }
    }

    #[test]
    fn test_prerelease_versions() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("2.0.0-alpha.1").expect("Should parse version"),
            "develop".to_string(),
            "def456".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "2.0.0-alpha.1-develop.def456");
    }

    #[test]
    fn test_version_with_build_metadata() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0+build.123").expect("Should parse version"),
            "main".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0+build.123-main");
    }

    #[test]
    fn test_empty_branch_name() {
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        // Empty branch results in just version with hyphen separator
        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert!(!snapshot.is_empty());
        assert!(snapshot.starts_with("1.0.0"));
    }

    #[test]
    fn test_format_validation_with_valid_semver_result() {
        // This format should produce valid semver
        let generator =
            SnapshotGenerator::new("{version}-{branch}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "main".to_string(),
            "abc123".to_string(),
            1640000000,
        );

        let result = generator.generate(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_format() {
        let generator =
            SnapshotGenerator::new("{version}-{timestamp}").expect("Should create generator");

        let timestamp = 1234567890_i64;
        let context = SnapshotContext::with_timestamp(
            Version::parse("1.0.0").expect("Should parse version"),
            "main".to_string(),
            "abc123".to_string(),
            timestamp,
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");
        assert_eq!(snapshot, "1.0.0-1234567890");
    }

    #[test]
    fn test_multiple_formats() {
        let formats = vec![
            "{version}-{branch}.{commit}",
            "{version}-snapshot.{timestamp}",
            "{version}.{commit}",
            "{version}-{branch}-{timestamp}",
        ];

        for format in formats {
            let result = SnapshotGenerator::new(format);
            assert!(result.is_ok(), "Format '{}' should be valid", format);
        }
    }

    #[test]
    fn test_real_world_scenario() {
        // Simulate a real CI/CD scenario
        let generator =
            SnapshotGenerator::new("{version}-{branch}.{commit}").expect("Should create generator");

        let context = SnapshotContext::with_timestamp(
            Version::parse("1.2.3").expect("Should parse version"),
            "feat/oauth-integration".to_string(),
            "a1b2c3d4e5f6789".to_string(),
            chrono::Utc::now().timestamp(),
        );

        let snapshot = generator.generate(&context).expect("Should generate snapshot");

        // Verify it contains expected parts
        assert!(snapshot.starts_with("1.2.3-feat-oauth-integration."));
        assert!(snapshot.ends_with("a1b2c3d"));
    }
}

// ============================================================================
// Story 5.7: Apply Versions with Dry-Run Tests
// ============================================================================

mod application_tests {
    use super::*;
    use crate::types::{Changeset, VersionBump};

    /// Creates a test single-package workspace for version application testing.
    async fn create_test_package_for_application(
        name: &str,
        version: &str,
    ) -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        let package_json = serde_json::json!({
            "name": name,
            "version": version,
            "dependencies": {
                "external-lib": "^2.0.0"
            },
            "devDependencies": {
                "jest": "^29.0.0"
            }
        });

        std::fs::write(
            root.join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("Should serialize JSON"),
        )
        .expect("Should write package.json");

        (temp_dir, root)
    }

    #[tokio::test]
    async fn test_apply_versions_dry_run_no_files_modified() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // Apply with dry_run = true
        let result =
            resolver.apply_versions(&changeset, true).await.expect("Should apply versions");

        // Verify dry-run behavior
        assert!(result.dry_run, "Should be dry-run mode");
        assert!(result.modified_files.is_empty(), "No files should be modified in dry-run");
        assert!(result.has_updates(), "Should have updates");
        assert!(!result.has_modified_files(), "Should not have modified files");

        // Verify original files unchanged
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(content.contains(r#""version": "1.0.0""#), "Version should remain unchanged");
    }

    #[tokio::test]
    async fn test_apply_versions_actual_modifies_files() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // Apply with dry_run = false
        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify application behavior
        assert!(!result.dry_run, "Should not be dry-run mode");
        assert!(!result.modified_files.is_empty(), "Files should be modified");
        assert!(result.has_modified_files(), "Should have modified files");
        assert_eq!(result.summary.packages_updated, 1, "Should update one package");

        // Verify file was actually modified
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(content.contains(r#""version": "1.1.0""#), "Version should be updated to 1.1.0");
    }

    #[tokio::test]
    async fn test_apply_versions_preserves_json_formatting() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-b", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Patch, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-b");

        // Apply versions
        resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Read and verify formatting
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("Should be valid JSON");
        assert_eq!(parsed["version"], "1.0.1", "Version should be updated");
        assert_eq!(parsed["name"], "@test/pkg-b", "Name should be preserved");

        // Verify pretty formatting (has newlines and indentation)
        assert!(content.contains('\n'), "Should have newlines");
        assert!(content.contains("  "), "Should have indentation");
    }

    #[tokio::test]
    async fn test_apply_versions_no_dependency_updates_single_package() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-b", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-b");

        // Apply versions
        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify version was updated
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(content.contains(r#""version": "1.1.0""#), "Version should be updated");

        // Single package with no internal dependencies won't have dependency updates
        assert_eq!(result.summary.packages_updated, 1, "Should update one package");
    }

    #[tokio::test]
    async fn test_apply_versions_summary_statistics() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Major, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // Apply versions
        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify summary
        assert_eq!(result.summary.packages_updated, 1, "Should update 1 package");
        assert_eq!(result.summary.direct_updates, 1, "Direct update");
        assert_eq!(result.summary.propagated_updates, 0, "No propagated updates");
        assert!(result.summary.has_updates(), "Should have updates");
    }

    #[tokio::test]
    async fn test_apply_versions_single_package_project() {
        let (_temp, root) = create_single_package_workspace().await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Patch, vec!["production".to_string()]);
        changeset.add_package("my-package");

        // Apply versions
        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify
        assert!(!result.dry_run, "Should not be dry-run");
        assert_eq!(result.modified_files.len(), 1, "Should modify one file");
        assert_eq!(result.summary.packages_updated, 1, "Should update one package");

        // Verify file content
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(content.contains(r#""version": "1.0.1""#), "Version should be updated");
    }

    #[tokio::test]
    async fn test_apply_versions_dry_run_then_actual() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // First do dry-run
        let dry_result =
            resolver.apply_versions(&changeset, true).await.expect("Should do dry-run");
        assert!(dry_result.dry_run, "Should be dry-run");
        assert!(dry_result.modified_files.is_empty(), "No files modified");

        // Verify nothing changed
        let pkg_path = root.join("package.json");
        let before_content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(
            before_content.contains(r#""version": "1.0.0""#),
            "Version unchanged after dry-run"
        );

        // Now apply for real
        let actual_result = resolver.apply_versions(&changeset, false).await.expect("Should apply");
        assert!(!actual_result.dry_run, "Should not be dry-run");
        assert!(!actual_result.modified_files.is_empty(), "Files should be modified");

        // Verify changes applied
        let after_content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(after_content.contains(r#""version": "1.1.0""#), "Version should be updated");
    }

    #[tokio::test]
    async fn test_apply_versions_empty_changeset() {
        let (_temp, root) = create_test_package_for_application("@test/pkg", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        // No packages added

        // Apply versions with empty changeset
        let result = resolver
            .apply_versions(&changeset, false)
            .await
            .expect("Should handle empty changeset");

        // Verify no changes
        assert_eq!(result.summary.packages_updated, 0, "No packages should be updated");
        assert!(!result.has_updates(), "Should not have updates");
        assert!(result.modified_files.is_empty(), "No files should be modified");
    }

    #[tokio::test]
    async fn test_apply_versions_package_not_found_error() {
        let (_temp, root) = create_test_package_for_application("@test/pkg", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("nonexistent-package");

        // Should fail with package not found
        let result = resolver.apply_versions(&changeset, false).await;
        assert!(result.is_err(), "Should return error for nonexistent package");

        if let Err(e) = result {
            assert!(
                matches!(e, VersionError::PackageNotFound { .. }),
                "Should be PackageNotFound error"
            );
        }
    }

    #[tokio::test]
    async fn test_apply_result_methods() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Test all ApplyResult methods
        assert!(result.has_updates(), "Should have updates");
        assert!(result.has_modified_files(), "Should have modified files");
        assert_eq!(result.update_count(), 1, "Should have 1 update");
        assert!(!result.has_circular_dependencies(), "Should not have circular dependencies");
    }

    #[tokio::test]
    async fn test_apply_summary_methods() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-b", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-b");

        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");
        let summary = &result.summary;

        // Test ApplySummary methods
        assert!(summary.has_updates(), "Should have updates");
        assert!(!summary.has_circular_dependencies(), "Should not have circular dependencies");
        assert_eq!(summary.direct_updates, 1, "Should have 1 direct update");
        assert_eq!(summary.propagated_updates, 0, "Should have 0 propagated updates");
    }

    #[tokio::test]
    async fn test_apply_versions_cross_platform_paths() {
        let (_temp, root) = create_test_package_for_application("@test/pkg-a", "1.0.0").await;
        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Patch, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // Apply versions
        let result =
            resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify paths are canonical and work cross-platform
        for path in &result.modified_files {
            assert!(path.is_absolute(), "Path should be absolute");
            assert!(path.exists(), "Modified file should exist");
            assert!(path.ends_with("package.json"), "Should be package.json file");
        }
    }

    #[tokio::test]
    async fn test_is_skipped_version_spec() {
        use crate::version::VersionResolver;
        use sublime_standard_tools::filesystem::FileSystemManager;

        // Test workspace protocol
        assert!(
            VersionResolver::<FileSystemManager>::is_skipped_version_spec("workspace:*"),
            "Should skip workspace:*"
        );
        assert!(
            VersionResolver::<FileSystemManager>::is_skipped_version_spec("workspace:^1.0.0"),
            "Should skip workspace:^1.0.0"
        );

        // Test file protocol
        assert!(
            VersionResolver::<FileSystemManager>::is_skipped_version_spec("file:../local-pkg"),
            "Should skip file: protocol"
        );

        // Test link protocol
        assert!(
            VersionResolver::<FileSystemManager>::is_skipped_version_spec("link:../linked"),
            "Should skip link: protocol"
        );

        // Test portal protocol
        assert!(
            VersionResolver::<FileSystemManager>::is_skipped_version_spec("portal:../portal"),
            "Should skip portal: protocol"
        );

        // Test normal version specs (should not be skipped)
        assert!(
            !VersionResolver::<FileSystemManager>::is_skipped_version_spec("^1.2.3"),
            "Should not skip normal semver"
        );
        assert!(
            !VersionResolver::<FileSystemManager>::is_skipped_version_spec("~2.0.0"),
            "Should not skip tilde range"
        );
        assert!(
            !VersionResolver::<FileSystemManager>::is_skipped_version_spec(">=1.0.0 <2.0.0"),
            "Should not skip range"
        );
    }

    // Note: Monorepo tests are skipped until MonorepoDetector enhancement is complete
    // This test validates the is_skipped_version_spec logic used in apply_versions
    #[tokio::test]
    async fn test_apply_versions_respects_skipped_specs() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        // Create a single package with workspace protocol dependency
        let package_json = serde_json::json!({
            "name": "@test/pkg-a",
            "version": "1.0.0",
            "dependencies": {
                "@test/internal": "workspace:*",
                "external": "^1.0.0"
            }
        });

        std::fs::write(
            root.join("package.json"),
            serde_json::to_string_pretty(&package_json).expect("Should serialize JSON"),
        )
        .expect("Should write package.json");

        let config = PackageToolsConfig::default();
        let resolver =
            VersionResolver::new(root.clone(), config).await.expect("Should create resolver");

        let mut changeset =
            Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("@test/pkg-a");

        // Apply versions
        resolver.apply_versions(&changeset, false).await.expect("Should apply versions");

        // Verify workspace: protocol was not modified
        let pkg_path = root.join("package.json");
        let content = tokio::fs::read_to_string(&pkg_path).await.expect("Should read file");
        assert!(
            content.contains(r#""@test/internal": "workspace:*""#),
            "workspace: protocol should remain unchanged"
        );
        assert!(content.contains(r#""version": "1.1.0""#), "Version should be updated");
    }
}
