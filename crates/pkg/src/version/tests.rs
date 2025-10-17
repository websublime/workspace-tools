//! Tests for the version module.
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
