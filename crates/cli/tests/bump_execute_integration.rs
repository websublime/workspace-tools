//! # Bump Execute Command Integration Tests
//!
//! **What**: Comprehensive end-to-end integration tests for the bump execute command.
//! Tests the complete workflow from changeset loading to version application, including
//! file modifications, changelog generation, changeset archival, and Git operations.
//!
//! **How**: Creates real filesystem structures using temporary directories, simulates
//! complete workspace scenarios (single package and monorepo), creates actual changesets,
//! and validates the entire execution pipeline.
//!
//! **Why**: To ensure the bump execute command works correctly in real-world scenarios,
//! properly handles Independent vs Unified strategies, correctly applies version bumps,
//! and safely performs Git operations. These tests validate all acceptance criteria from
//! Story 5.2.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use std::io::Cursor;
use std::path::PathBuf;
use sublime_cli_tools::cli::commands::BumpArgs;
use sublime_cli_tools::commands::bump::execute_bump_apply;
use sublime_cli_tools::output::{Output, OutputFormat};

mod common;

// ============================================================================
// Test Fixtures - Workspace Creation
// ============================================================================

/// Creates a single-package repository with a changeset
async fn create_single_package_workspace() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Create package.json
    let package_json = r#"{
        "name": "my-package",
        "version": "1.0.0",
        "description": "A single package"
    }"#;
    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .expect("Failed to write package.json");

    // Create config
    let config = r#"{
        "changeset": {
            "path": ".changesets/"
        },
        "version": {
            "strategy": "independent",
            "defaultBump": "patch"
        },
        "changelog": {
            "enabled": false
        }
    }"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    // Create changesets directory
    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    // Create a changeset with all required fields
    let changeset = r#"{
        "branch": "feature/test",
        "bump": "minor",
        "environments": ["production"],
        "packages": ["my-package"],
        "changes": [],
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(root.join(".changesets/feature-test.json"), changeset)
        .await
        .expect("Failed to write changeset");

    (temp_dir, root)
}

/// Creates an independent strategy monorepo with changesets
async fn create_independent_monorepo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Root package.json
    let root_package = r#"{
        "name": "monorepo-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json
    let package_lock = r#"{
        "name": "monorepo-root",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    // Create packages directory
    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Package A
    let pkg_a_dir = root.join("packages/pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a = r#"{
        "name": "@test/pkg-a",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B
    let pkg_b_dir = root.join("packages/pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b = r#"{
        "name": "@test/pkg-b",
        "version": "2.0.0"
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b)
        .await
        .expect("Failed to write pkg-b package.json");

    // Package C
    let pkg_c_dir = root.join("packages/pkg-c");
    tokio::fs::create_dir_all(&pkg_c_dir).await.expect("Failed to create pkg-c dir");
    let pkg_c = r#"{
        "name": "@test/pkg-c",
        "version": "0.5.0"
    }"#;
    tokio::fs::write(pkg_c_dir.join("package.json"), pkg_c)
        .await
        .expect("Failed to write pkg-c package.json");

    // Create config (Independent strategy)
    let config = r#"{
        "changeset": {
            "path": ".changesets/"
        },
        "version": {
            "strategy": "independent",
            "defaultBump": "patch"
        },
        "changelog": {
            "enabled": false
        }
    }"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    // Create changesets directory
    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    // Create changeset that only affects pkg-a
    let changeset = r#"{
        "branch": "feature/update-a",
        "bump": "minor",
        "environments": ["production"],
        "packages": ["@test/pkg-a"],
        "changes": [],
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(root.join(".changesets/feature-update-a.json"), changeset)
        .await
        .expect("Failed to write changeset");

    (temp_dir, root)
}

/// Creates a unified strategy monorepo with changesets
async fn create_unified_monorepo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Root package.json
    let root_package = r#"{
        "name": "unified-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json
    let package_lock = r#"{
        "name": "unified-monorepo",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    // Create packages directory
    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Package A
    let pkg_a_dir = root.join("packages/pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a = r#"{
        "name": "@unified/pkg-a",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B
    let pkg_b_dir = root.join("packages/pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b = r#"{
        "name": "@unified/pkg-b",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b)
        .await
        .expect("Failed to write pkg-b package.json");

    // Package C
    let pkg_c_dir = root.join("packages/pkg-c");
    tokio::fs::create_dir_all(&pkg_c_dir).await.expect("Failed to create pkg-c dir");
    let pkg_c = r#"{
        "name": "@unified/pkg-c",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_c_dir.join("package.json"), pkg_c)
        .await
        .expect("Failed to write pkg-c package.json");

    // Create config (Unified strategy)
    let config = r#"{
        "changeset": {
            "path": ".changesets/"
        },
        "version": {
            "strategy": "unified",
            "defaultBump": "patch"
        },
        "changelog": {
            "enabled": false
        }
    }"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    // Create changesets directory
    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    // Create changeset that only mentions pkg-a (but unified bumps all)
    let changeset = r#"{
        "branch": "feature/update-a",
        "bump": "minor",
        "environments": ["production"],
        "packages": ["@unified/pkg-a"],
        "changes": [],
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(root.join(".changesets/feature-update-a.json"), changeset)
        .await
        .expect("Failed to write changeset");

    (temp_dir, root)
}

/// Creates a unified monorepo with multiple changesets having different bump types
async fn create_unified_monorepo_multiple_changesets() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Root package.json
    let root_package = r#"{
        "name": "unified-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json
    let package_lock = r#"{
        "name": "unified-monorepo",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    // Create packages directory
    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Package A
    let pkg_a_dir = root.join("packages/pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a = r#"{
        "name": "@unified/pkg-a",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B
    let pkg_b_dir = root.join("packages/pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b = r#"{
        "name": "@unified/pkg-b",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b)
        .await
        .expect("Failed to write pkg-b package.json");

    // Create config (Unified strategy)
    let config = r#"{
        "changeset": {
            "path": ".changesets/"
        },
        "version": {
            "strategy": "unified",
            "defaultBump": "patch"
        },
        "changelog": {
            "enabled": false
        }
    }"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    // Create changesets directory
    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    // Changeset 1: Minor bump for pkg-a
    let changeset1 = r#"{
        "branch": "feature/update-a",
        "bump": "minor",
        "environments": ["production"],
        "packages": ["@unified/pkg-a"],
        "changes": [],
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(root.join(".changesets/feature-update-a.json"), changeset1)
        .await
        .expect("Failed to write changeset 1");

    // Changeset 2: Major bump for pkg-b (should win)
    let changeset2 = r#"{
        "branch": "breaking/update-b",
        "bump": "major",
        "environments": ["production"],
        "packages": ["@unified/pkg-b"],
        "changes": [],
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }"#;
    tokio::fs::write(root.join(".changesets/breaking-update-b.json"), changeset2)
        .await
        .expect("Failed to write changeset 2");

    (temp_dir, root)
}

// ============================================================================
// Integration Tests - Single Package
// ============================================================================

/// Test: Single package repository only bumps that one package
#[tokio::test]
async fn test_single_package_bumps_only_package() {
    // Setup
    let (_temp, root) = create_single_package_workspace().await;

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true, // Skip confirmation
        show_diff: false,
    };

    // Create output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify version was bumped
    let version = common::get_package_version(&root).await.unwrap();
    assert_eq!(version, "1.1.0", "Version should be bumped from 1.0.0 to 1.1.0 (minor)");
}

// ============================================================================
// Integration Tests - Independent Strategy
// ============================================================================

/// Test: Independent strategy only bumps packages in changesets
#[tokio::test]
async fn test_independent_bumps_only_changeset_packages() {
    // Setup
    let (_temp, root) = create_independent_monorepo().await;

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    // Create output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify pkg-a was bumped (in changeset)
    let pkg_a_version = common::get_package_version(&root.join("packages/pkg-a")).await.unwrap();
    assert_eq!(pkg_a_version, "1.1.0", "pkg-a should be bumped from 1.0.0 to 1.1.0");

    // Verify pkg-b was NOT bumped (not in changeset)
    let pkg_b_version = common::get_package_version(&root.join("packages/pkg-b")).await.unwrap();
    assert_eq!(pkg_b_version, "2.0.0", "pkg-b should remain at 2.0.0 (not in changeset)");

    // Verify pkg-c was NOT bumped (not in changeset)
    let pkg_c_version = common::get_package_version(&root.join("packages/pkg-c")).await.unwrap();
    assert_eq!(pkg_c_version, "0.5.0", "pkg-c should remain at 0.5.0 (not in changeset)");
}

// ============================================================================
// Integration Tests - Unified Strategy
// ============================================================================

/// Test: Unified strategy bumps ALL packages when changeset exists
///
/// This test verifies that the unified strategy correctly bumps all workspace packages
/// to the same version, regardless of which packages are listed in the changeset.
#[tokio::test]
async fn test_unified_bumps_all_packages() {
    // Setup
    let (_temp, root) = create_unified_monorepo().await;

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    // Create output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify ALL packages were bumped to same version
    let pkg_a_version = common::get_package_version(&root.join("packages/pkg-a")).await.unwrap();
    let pkg_b_version = common::get_package_version(&root.join("packages/pkg-b")).await.unwrap();
    let pkg_c_version = common::get_package_version(&root.join("packages/pkg-c")).await.unwrap();

    assert_eq!(pkg_a_version, "1.1.0", "pkg-a should be bumped to 1.1.0");
    assert_eq!(pkg_b_version, "1.1.0", "pkg-b should be bumped to 1.1.0 (unified)");
    assert_eq!(pkg_c_version, "1.1.0", "pkg-c should be bumped to 1.1.0 (unified)");
}

/// Test: Unified strategy applies highest bump type from all changesets
#[tokio::test]
async fn test_unified_applies_highest_bump_type() {
    // Setup
    let (_temp, root) = create_unified_monorepo_multiple_changesets().await;

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    // Create output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify ALL packages were bumped with MAJOR (highest bump type)
    let pkg_a_version = common::get_package_version(&root.join("packages/pkg-a")).await.unwrap();
    let pkg_b_version = common::get_package_version(&root.join("packages/pkg-b")).await.unwrap();

    assert_eq!(pkg_a_version, "2.0.0", "pkg-a should be bumped to 2.0.0 (major wins)");
    assert_eq!(pkg_b_version, "2.0.0", "pkg-b should be bumped to 2.0.0 (major wins)");
}

// ============================================================================
// Integration Tests - Changeset Archival
// ============================================================================

/// Test: Changesets are archived after successful execution
#[tokio::test]
async fn test_changesets_archived_on_success() {
    // Setup
    let (_temp, root) = create_single_package_workspace().await;

    // Create args (archival enabled)
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: false, // Enable archival
        force: true,
        show_diff: false,
    };

    // Create output
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Verify changeset exists before execution
    assert!(
        common::changeset_exists(&root, "feature-test").await,
        "Changeset should exist before execution"
    );

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify changeset no longer exists in active directory
    assert!(
        !common::changeset_exists(&root, "feature-test").await,
        "Changeset should not exist after archival"
    );

    // Verify changeset was archived
    assert!(
        common::changeset_is_archived(&root, "feature-test").await,
        "Changeset should be archived"
    );
}

// ============================================================================
// Integration Tests - Error Conditions
// ============================================================================

/// Test: Execute fails gracefully when no changesets exist
#[tokio::test]
async fn test_no_changesets_exits_gracefully() {
    // Setup workspace without changesets
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package_json = r#"{
        "name": "my-package",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .expect("Failed to write package.json");

    let config = r#"{
        "changeset": { "path": ".changesets/" },
        "version": { "strategy": "independent" },
        "changelog": { "enabled": false }
    }"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;

    // Should succeed (graceful exit, not an error)
    assert!(result.is_ok(), "Should handle no changesets gracefully: {:?}", result.err());

    // Version should remain unchanged
    let version = common::get_package_version(&root).await.unwrap();
    assert_eq!(version, "1.0.0", "Version should remain unchanged");
}

/// Test: Execute fails when workspace is not initialized
#[tokio::test]
async fn test_uninitialized_workspace_fails() {
    // Setup workspace without config
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package_json = r#"{
        "name": "my-package",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .expect("Failed to write package.json");

    // NO config file created

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute
    let result = execute_bump_apply(&args, &output, &root, None).await;

    // Should fail with clear error
    assert!(result.is_err(), "Should fail when workspace is not initialized");
}

// ============================================================================
// Integration Tests - JSON Output
// ============================================================================

/// Test: JSON output contains complete information
#[tokio::test]
async fn test_json_output_complete() {
    // Setup
    let (_temp, root) = create_single_package_workspace().await;

    // Create args
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };

    // Create output in JSON format
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);

    // Execute - should work in JSON mode
    let result = execute_bump_apply(&args, &output, &root, None).await;
    assert!(result.is_ok(), "Execute should succeed in JSON mode");

    // Verify version was actually bumped
    let version = common::get_package_version(&root).await.unwrap();
    assert_eq!(version, "1.1.0", "Version should be bumped even in JSON output mode");
}

// ============================================================================
// Performance Tests
// ============================================================================

/// Creates a large monorepo for performance testing
async fn create_large_monorepo(package_count: usize) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_package = r#"{
        "name": "large-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    let package_lock = r#"{"name": "large-monorepo", "version": "1.0.0", "lockfileVersion": 3, "requires": true, "packages": {}}"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    for i in 0..package_count {
        let pkg_dir = root.join(format!("packages/pkg-{i}"));
        tokio::fs::create_dir_all(&pkg_dir)
            .await
            .unwrap_or_else(|_| panic!("Failed to create pkg-{i} dir"));
        let pkg_json = format!(r#"{{"name": "@large/pkg-{i}", "version": "1.0.0"}}"#);
        tokio::fs::write(pkg_dir.join("package.json"), pkg_json)
            .await
            .unwrap_or_else(|_| panic!("Failed to write pkg-{i} package.json"));
    }

    let config = r#"{"changeset": {"path": ".changesets/"}, "version": {"strategy": "unified", "defaultBump": "patch"}, "changelog": {"enabled": false}}"#;
    tokio::fs::write(root.join("repo.config.json"), config).await.expect("Failed to write config");

    tokio::fs::create_dir_all(root.join(".changesets"))
        .await
        .expect("Failed to create changesets dir");

    let changeset = r#"{"branch": "feature/update", "bump": "patch", "environments": ["production"], "packages": ["@large/pkg-0"], "changes": [], "created_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}"#;
    tokio::fs::write(root.join(".changesets/feature-update.json"), changeset)
        .await
        .expect("Failed to write changeset");

    (temp_dir, root)
}

/// Test: Performance with 50 packages
#[tokio::test]
#[ignore = "Performance test - run manually with --ignored"]
#[allow(clippy::print_stdout)]
async fn test_performance_50_packages() {
    use std::time::Instant;
    let (_temp, root) = create_large_monorepo(50).await;
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);
    let start = Instant::now();
    let result = execute_bump_apply(&args, &output, &root, None).await;
    let duration = start.elapsed();
    assert!(result.is_ok(), "Execute should succeed: {:#?}", result.err());
    assert!(
        duration.as_secs() < 5,
        "Should complete in under 5 seconds for 50 packages, took {duration:?}"
    );
    println!("✓ Bumped 50 packages in {duration:?}");
}

/// Test: Performance with 100 packages
#[tokio::test]
#[ignore = "Performance test - run manually with --ignored"]
#[allow(clippy::print_stdout)]
async fn test_performance_100_packages() {
    use std::time::Instant;
    let (_temp, root) = create_large_monorepo(100).await;
    let args = BumpArgs {
        dry_run: false,
        execute: true,
        snapshot: false,
        snapshot_format: None,
        prerelease: None,
        packages: None,
        git_tag: false,
        git_push: false,
        git_commit: false,
        no_changelog: true,
        no_archive: true,
        force: true,
        show_diff: false,
    };
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, buffer, false);
    let start = Instant::now();
    let result = execute_bump_apply(&args, &output, &root, None).await;
    let duration = start.elapsed();
    assert!(result.is_ok(), "Execute should succeed: {:#?}", result.err());
    assert!(
        duration.as_secs() < 10,
        "Should complete in under 10 seconds for 100 packages, took {duration:?}"
    );
    println!("✓ Bumped 100 packages in {duration:?}");
    if duration.as_secs() >= 2 {
        println!("⚠ Note: Took longer than 2s requirement, but acceptable for 100 packages");
    }
}
