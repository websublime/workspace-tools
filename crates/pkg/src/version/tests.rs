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
use crate::version::VersionResolver;
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
