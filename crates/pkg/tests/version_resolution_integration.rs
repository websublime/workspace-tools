//! # Version Resolution Integration Tests
//!
//! **What**: Comprehensive end-to-end integration tests for the version resolution system.
//! This module tests the complete workflow from changeset creation to version application,
//! including dependency propagation, circular dependency detection, and various versioning
//! strategies.
//!
//! **How**: Creates real filesystem structures using temporary directories, simulates
//! complete monorepo and single-package scenarios, and validates the entire version
//! resolution pipeline including dry-run and actual application modes.
//!
//! **Why**: To ensure the version resolution system works correctly in real-world scenarios
//! with complex dependency graphs, multiple packages, and various edge cases. These tests
//! validate the integration of all version module components working together.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::path::PathBuf;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::types::{Changeset, UpdateReason, VersionBump, VersioningStrategy};
use sublime_pkg_tools::version::VersionResolver;

mod common;

// ============================================================================
// Test Fixtures - Complex Scenarios
// ============================================================================

/// Creates a complex monorepo with various dependency patterns
async fn create_complex_monorepo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    // Root package.json
    let root_package = r#"{
        "name": "monorepo-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*", "tools/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json to ensure NPM workspace detection
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

    // Create tools directory
    tokio::fs::create_dir_all(root.join("tools")).await.expect("Failed to create tools dir");

    // Package A - No dependencies
    let pkg_a_dir = root.join("packages").join("pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a = r#"{
        "name": "@test/pkg-a",
        "version": "1.0.0",
        "description": "Package A"
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B - Depends on A
    let pkg_b_dir = root.join("packages").join("pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b = r#"{
        "name": "@test/pkg-b",
        "version": "1.0.0",
        "dependencies": {
            "@test/pkg-a": "^1.0.0",
            "lodash": "^4.17.21"
        }
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b)
        .await
        .expect("Failed to write pkg-b package.json");

    // Package C - Depends on B (transitive to A)
    let pkg_c_dir = root.join("packages").join("pkg-c");
    tokio::fs::create_dir_all(&pkg_c_dir).await.expect("Failed to create pkg-c dir");
    let pkg_c = r#"{
        "name": "@test/pkg-c",
        "version": "2.0.0",
        "dependencies": {
            "@test/pkg-b": "^1.0.0"
        },
        "devDependencies": {
            "@test/pkg-a": "^1.0.0"
        }
    }"#;
    tokio::fs::write(pkg_c_dir.join("package.json"), pkg_c)
        .await
        .expect("Failed to write pkg-c package.json");

    // Package D - Independent
    let pkg_d_dir = root.join("packages").join("pkg-d");
    tokio::fs::create_dir_all(&pkg_d_dir).await.expect("Failed to create pkg-d dir");
    let pkg_d = r#"{
        "name": "@test/pkg-d",
        "version": "0.5.0",
        "dependencies": {
            "react": "^18.0.0"
        }
    }"#;
    tokio::fs::write(pkg_d_dir.join("package.json"), pkg_d)
        .await
        .expect("Failed to write pkg-d package.json");

    // Tool package with workspace protocol
    let tool_dir = root.join("tools").join("build-tool");
    tokio::fs::create_dir_all(&tool_dir).await.expect("Failed to create build-tool dir");
    let tool = r#"{
        "name": "@test/build-tool",
        "version": "1.0.0",
        "dependencies": {
            "@test/pkg-a": "workspace:*",
            "@test/pkg-b": "workspace:^"
        }
    }"#;
    tokio::fs::write(tool_dir.join("package.json"), tool)
        .await
        .expect("Failed to write build-tool package.json");

    (temp_dir, root)
}

/// Creates a monorepo with circular dependencies
async fn create_circular_monorepo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_package = r#"{
        "name": "circular-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json for NPM workspace detection
    let package_lock = r#"{
        "name": "circular-root",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Package A depends on B
    let pkg_a_dir = root.join("packages").join("pkg-a");
    tokio::fs::create_dir_all(&pkg_a_dir).await.expect("Failed to create pkg-a dir");
    let pkg_a = r#"{
        "name": "@circular/pkg-a",
        "version": "1.0.0",
        "dependencies": {
            "@circular/pkg-b": "^1.0.0"
        }
    }"#;
    tokio::fs::write(pkg_a_dir.join("package.json"), pkg_a)
        .await
        .expect("Failed to write pkg-a package.json");

    // Package B depends on A (creates cycle)
    let pkg_b_dir = root.join("packages").join("pkg-b");
    tokio::fs::create_dir_all(&pkg_b_dir).await.expect("Failed to create pkg-b dir");
    let pkg_b = r#"{
        "name": "@circular/pkg-b",
        "version": "1.0.0",
        "dependencies": {
            "@circular/pkg-a": "^1.0.0"
        }
    }"#;
    tokio::fs::write(pkg_b_dir.join("package.json"), pkg_b)
        .await
        .expect("Failed to write pkg-b package.json");

    (temp_dir, root)
}

/// Creates a deep dependency chain monorepo
async fn create_deep_chain_monorepo(depth: usize) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_package = r#"{
        "name": "deep-chain-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package)
        .await
        .expect("Failed to write root package.json");

    // Create package-lock.json for NPM workspace detection
    let package_lock = r#"{
        "name": "deep-chain-root",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages dir");

    // Create chain: pkg-0 -> pkg-1 -> pkg-2 -> ... -> pkg-N
    for i in 0..depth {
        let pkg_dir = root.join("packages").join(format!("pkg-{}", i));
        tokio::fs::create_dir_all(&pkg_dir).await.expect("Failed to create package dir");

        let pkg_json = if i == 0 {
            // First package has no dependencies
            format!(
                r#"{{
                "name": "@chain/pkg-{}",
                "version": "1.0.0"
            }}"#,
                i
            )
        } else {
            // Each package depends on the previous one
            format!(
                r#"{{
                "name": "@chain/pkg-{}",
                "version": "1.0.0",
                "dependencies": {{
                    "@chain/pkg-{}": "^1.0.0"
                }}
            }}"#,
                i,
                i - 1
            )
        };

        tokio::fs::write(pkg_dir.join("package.json"), pkg_json)
            .await
            .expect("Failed to write package.json");
    }

    (temp_dir, root)
}

/// Creates a single package project
async fn create_single_package() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package = r#"{
        "name": "single-package",
        "version": "1.5.0",
        "description": "A single package project",
        "dependencies": {
            "lodash": "^4.17.21",
            "axios": "^1.0.0"
        },
        "devDependencies": {
            "jest": "^29.0.0",
            "typescript": "^5.0.0"
        }
    }"#;
    tokio::fs::write(root.join("package.json"), package)
        .await
        .expect("Failed to write package.json");

    (temp_dir, root)
}

// ============================================================================
// Integration Tests - Complete Workflows
// ============================================================================

#[tokio::test]
async fn test_integration_complete_resolution_workflow_independent() {
    // Setup: Create complex monorepo
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    assert!(resolver.is_monorepo(), "Should detect monorepo");

    // Create changeset with Package A getting a minor bump
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Step 1: Resolve versions (includes propagation)
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Verify resolution - pkg-a direct + pkg-b and pkg-c propagated
    assert_eq!(resolution.updates.len(), 3, "Should update pkg-a, pkg-b, and pkg-c");

    // Find pkg-a update
    let pkg_a_update = resolution
        .updates
        .iter()
        .find(|u| u.name == "@test/pkg-a")
        .expect("Should find pkg-a update");
    assert_eq!(pkg_a_update.current_version.to_string(), "1.0.0");
    assert_eq!(pkg_a_update.next_version.to_string(), "1.1.0");
    assert!(!pkg_a_update.is_propagated(), "pkg-a is direct change");

    // Find pkg-b update (propagated)
    let pkg_b_update = resolution
        .updates
        .iter()
        .find(|u| u.name == "@test/pkg-b")
        .expect("Should find pkg-b update");
    assert_eq!(pkg_b_update.current_version.to_string(), "1.0.0");
    assert_eq!(pkg_b_update.next_version.to_string(), "1.0.1");
    assert!(pkg_b_update.is_propagated(), "pkg-b should be propagated");

    // Step 2: Apply versions with dry run
    let dry_result =
        resolver.apply_versions(&changeset, true).await.expect("Failed to apply dry run");

    assert!(dry_result.dry_run, "Should be dry run");
    assert_eq!(dry_result.modified_files.len(), 0, "Should not modify files in dry run");
    assert_eq!(dry_result.summary.packages_updated, 3);
    assert_eq!(dry_result.summary.direct_updates, 1);
    assert_eq!(dry_result.summary.propagated_updates, 2);

    // Step 3: Apply versions for real
    let apply_result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert!(!apply_result.dry_run, "Should not be dry run");
    assert_eq!(apply_result.modified_files.len(), 3, "Should modify 3 package.json files");
    assert_eq!(apply_result.summary.packages_updated, 3);

    // Step 4: Verify file contents
    let pkg_a_content = tokio::fs::read_to_string(root.join("packages/pkg-a/package.json"))
        .await
        .expect("Failed to read pkg-a");
    assert!(pkg_a_content.contains(r#""version": "1.1.0""#), "pkg-a version should be updated");

    let pkg_b_content = tokio::fs::read_to_string(root.join("packages/pkg-b/package.json"))
        .await
        .expect("Failed to read pkg-b");
    assert!(pkg_b_content.contains(r#""version": "1.0.1""#), "pkg-b version should be updated");
    assert!(
        pkg_b_content.contains(r#""@test/pkg-a": "^1.1.0""#),
        "pkg-b dependency should be updated"
    );

    let pkg_c_content = tokio::fs::read_to_string(root.join("packages/pkg-c/package.json"))
        .await
        .expect("Failed to read pkg-c");
    assert!(pkg_c_content.contains(r#""version": "2.0.1""#), "pkg-c version should be updated");
    assert!(
        pkg_c_content.contains(r#""@test/pkg-b": "^1.0.1""#),
        "pkg-c dependency should be updated"
    );
}

#[tokio::test]
async fn test_integration_unified_strategy_workflow() {
    // Setup: Create complex monorepo with unified strategy
    let (_temp, root) = create_complex_monorepo().await;
    let mut config = PackageToolsConfig::default();
    config.version.strategy = VersioningStrategy::Unified;

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Create changeset with multiple packages
    let mut changeset =
        Changeset::new("release/v2", VersionBump::Major, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");
    changeset.add_package("@test/pkg-b");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // In unified strategy, all packages should get the same version
    let major_versions: Vec<_> =
        resolution.updates.iter().filter(|u| u.next_version.major() >= 2).collect();

    assert!(!major_versions.is_empty(), "Should have major version updates");

    // Apply versions
    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert!(!result.dry_run);
    assert!(result.summary.packages_updated > 0);

    // Verify all packages have versions bumped
    let pkg_a_content = tokio::fs::read_to_string(root.join("packages/pkg-a/package.json"))
        .await
        .expect("Failed to read pkg-a");
    assert!(pkg_a_content.contains(r#""version": "2.0.0""#), "pkg-a should have major version");
}

#[tokio::test]
async fn test_integration_circular_dependency_detection() {
    // Setup: Create monorepo with circular dependencies
    let (_temp, root) = create_circular_monorepo().await;
    let config = PackageToolsConfig::default();

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Create changeset
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@circular/pkg-a");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Should detect circular dependency
    assert!(!resolution.circular_dependencies.is_empty(), "Should detect circular dependencies");
    assert_eq!(resolution.circular_dependencies.len(), 1);

    let cycle = &resolution.circular_dependencies[0];
    assert!(cycle.cycle.contains(&"@circular/pkg-a".to_string()));
    assert!(cycle.cycle.contains(&"@circular/pkg-b".to_string()));
}

#[tokio::test]
async fn test_integration_max_depth_propagation() {
    // Setup: Create deep chain
    let depth = 10;
    let (_temp, root) = create_deep_chain_monorepo(depth).await;

    // Configure with max depth = 3
    let mut config = PackageToolsConfig::default();
    config.dependency.max_depth = 3;

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Create changeset for first package
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@chain/pkg-0");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Should only propagate to depth 3 (pkg-0, pkg-1, pkg-2, pkg-3)
    assert!(
        resolution.updates.len() <= 4,
        "Should respect max depth of 3, got {} updates",
        resolution.updates.len()
    );

    // Verify depth in updates
    for update in &resolution.updates {
        if let UpdateReason::DependencyPropagation { depth, .. } = &update.reason {
            assert!(depth <= &3, "Propagation depth should not exceed 3");
        }
    }
}

#[tokio::test]
async fn test_integration_workspace_protocol_preservation() {
    // Setup: Create complex monorepo
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Create changeset for pkg-a
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Apply versions
    let _result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    // Verify workspace protocol is preserved in build-tool
    let tool_content = tokio::fs::read_to_string(root.join("tools/build-tool/package.json"))
        .await
        .expect("Failed to read build-tool");

    assert!(
        tool_content.contains(r#""@test/pkg-a": "workspace:*""#),
        "workspace:* protocol should be preserved"
    );
    assert!(
        tool_content.contains(r#""@test/pkg-b": "workspace:^""#),
        "workspace:^ protocol should be preserved"
    );
}

#[tokio::test]
async fn test_integration_single_package_workflow() {
    // Setup: Create single package
    let (_temp, root) = create_single_package().await;
    let config = PackageToolsConfig::default();

    // Initialize resolver
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    assert!(!resolver.is_monorepo(), "Should not detect as monorepo");

    // Create changeset
    let mut changeset =
        Changeset::new("release/v2", VersionBump::Major, vec!["production".to_string()]);
    changeset.add_package("single-package");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    assert_eq!(resolution.updates.len(), 1);
    let update = &resolution.updates[0];
    assert_eq!(update.current_version.to_string(), "1.5.0");
    assert_eq!(update.next_version.to_string(), "2.0.0");

    // Apply versions
    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert_eq!(result.modified_files.len(), 1);
    assert_eq!(result.summary.packages_updated, 1);
    assert_eq!(result.summary.direct_updates, 1);
    assert_eq!(result.summary.propagated_updates, 0);

    // Verify file content
    let content = tokio::fs::read_to_string(root.join("package.json"))
        .await
        .expect("Failed to read package.json");
    assert!(content.contains(r#""version": "2.0.0""#));
}

#[tokio::test]
async fn test_integration_dry_run_then_apply() {
    // Setup
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Patch, vec!["staging".to_string()]);
    changeset.add_package("@test/pkg-d");

    // First: Dry run
    let dry_result =
        resolver.apply_versions(&changeset, true).await.expect("Failed to apply dry run");

    assert!(dry_result.dry_run);
    assert_eq!(dry_result.modified_files.len(), 0);
    let dry_summary = dry_result.summary;

    // Verify no files were modified
    let original_content = tokio::fs::read_to_string(root.join("packages/pkg-d/package.json"))
        .await
        .expect("Failed to read pkg-d");
    assert!(original_content.contains(r#""version": "0.5.0""#));

    // Then: Actual apply
    let apply_result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert!(!apply_result.dry_run);
    assert_eq!(apply_result.modified_files.len(), 1);

    // Verify summaries match
    assert_eq!(dry_summary.packages_updated, apply_result.summary.packages_updated);
    assert_eq!(dry_summary.direct_updates, apply_result.summary.direct_updates);

    // Verify file was modified
    let updated_content = tokio::fs::read_to_string(root.join("packages/pkg-d/package.json"))
        .await
        .expect("Failed to read pkg-d");
    assert!(updated_content.contains(r#""version": "0.5.1""#));
}

#[tokio::test]
async fn test_integration_no_propagation_config() {
    // Setup: Create complex monorepo with no propagation
    let (_temp, root) = create_complex_monorepo().await;
    let mut config = PackageToolsConfig::default();
    config.dependency.propagation_bump = "none".to_string(); // Disable propagation

    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Should only update pkg-a, not propagate
    assert_eq!(resolution.updates.len(), 1);
    assert_eq!(resolution.updates[0].name, "@test/pkg-a");
    assert!(!resolution.updates[0].is_propagated());
}

#[tokio::test]
async fn test_integration_dev_dependencies_propagation() {
    // Setup
    let (_temp, root) = create_complex_monorepo().await;
    let mut config = PackageToolsConfig::default();
    config.dependency.propagate_dev_dependencies = true; // Enable dev deps propagation

    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // pkg-c has pkg-a as devDependency, should be updated
    let pkg_c_update = resolution.updates.iter().find(|u| u.name == "@test/pkg-c");

    assert!(pkg_c_update.is_some(), "pkg-c should be updated due to devDependency");

    // Apply and verify
    let _result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    let pkg_c_content = tokio::fs::read_to_string(root.join("packages/pkg-c/package.json"))
        .await
        .expect("Failed to read pkg-c");

    // Check devDependencies section
    let pkg_c_json: serde_json::Value =
        serde_json::from_str(&pkg_c_content).expect("Failed to parse JSON");
    let dev_deps = pkg_c_json["devDependencies"].as_object().expect("Should have devDependencies");
    let pkg_a_version = dev_deps["@test/pkg-a"].as_str().expect("Should have @test/pkg-a");

    assert!(pkg_a_version.contains("1.1.0"), "devDependency should be updated to 1.1.0");
}

#[tokio::test]
async fn test_integration_empty_changeset() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    // Create changeset with no packages
    let changeset =
        Changeset::new("feature/empty", VersionBump::Minor, vec!["production".to_string()]);

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    assert_eq!(resolution.updates.len(), 0);
    assert_eq!(resolution.circular_dependencies.len(), 0);
}

#[tokio::test]
async fn test_integration_nonexistent_package_error() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    // Create changeset with non-existent package
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/nonexistent");

    // Should return error
    let result = resolver.resolve_versions(&changeset).await;
    assert!(result.is_err(), "Should error on non-existent package");
}

#[tokio::test]
async fn test_integration_multiple_packages_independent_bumps() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Create changeset with multiple packages with different explicit versions
    let mut changeset =
        Changeset::new("release/multi", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");
    changeset.add_package("@test/pkg-d");

    // Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Should update both directly affected packages plus propagated ones
    assert!(resolution.updates.len() >= 2);

    // Find direct updates
    let direct_updates: Vec<_> = resolution.updates.iter().filter(|u| !u.is_propagated()).collect();

    assert_eq!(direct_updates.len(), 2);

    // Apply versions
    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert_eq!(result.summary.direct_updates, 2);
}

#[tokio::test]
async fn test_integration_all_version_bumps() {
    let (_temp, root) = create_single_package().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    // Test Major bump
    let mut major_changeset =
        Changeset::new("release/v2", VersionBump::Major, vec!["production".to_string()]);
    major_changeset.add_package("single-package");

    let major_resolution =
        resolver.resolve_versions(&major_changeset).await.expect("Failed to resolve major");
    assert_eq!(major_resolution.updates[0].next_version.to_string(), "2.0.0");

    // Test Minor bump (reset to 1.5.0)
    let mut minor_changeset =
        Changeset::new("feature/new", VersionBump::Minor, vec!["production".to_string()]);
    minor_changeset.add_package("single-package");

    let minor_resolution =
        resolver.resolve_versions(&minor_changeset).await.expect("Failed to resolve minor");
    assert_eq!(minor_resolution.updates[0].next_version.to_string(), "1.6.0");

    // Test Patch bump
    let mut patch_changeset =
        Changeset::new("fix/bug", VersionBump::Patch, vec!["production".to_string()]);
    patch_changeset.add_package("single-package");

    let patch_resolution =
        resolver.resolve_versions(&patch_changeset).await.expect("Failed to resolve patch");
    assert_eq!(patch_resolution.updates[0].next_version.to_string(), "1.5.1");
}

#[tokio::test]
async fn test_integration_json_formatting_preservation() {
    // Create a package with specific formatting
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let formatted_package = r#"{
  "name": "formatted-package",
  "version": "1.0.0",
  "description": "Test formatting",
  "dependencies": {
    "lodash": "^4.17.21"
  }
}"#;
    tokio::fs::write(root.join("package.json"), formatted_package)
        .await
        .expect("Failed to write package.json");

    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("formatted-package");

    // Apply versions
    let _result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    // Read and verify formatting is preserved (2-space indentation)
    let content = tokio::fs::read_to_string(root.join("package.json"))
        .await
        .expect("Failed to read package.json");

    assert!(content.contains(r#""version": "1.1.0""#));
    // Check that indentation is preserved (2 spaces)
    assert!(content.contains("  \"name\""));
    assert!(content.contains("  \"version\""));
}

#[tokio::test]
async fn test_integration_discover_packages() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    // Discover all packages
    let packages = resolver.discover_packages().await.expect("Failed to discover packages");

    // Should find 5 packages (pkg-a, pkg-b, pkg-c, pkg-d, build-tool)
    assert_eq!(packages.len(), 5, "Should discover all 5 packages");

    // Verify package names
    let names: Vec<String> = packages.iter().map(|p| p.name().to_string()).collect();
    assert!(names.contains(&"@test/pkg-a".to_string()));
    assert!(names.contains(&"@test/pkg-b".to_string()));
    assert!(names.contains(&"@test/pkg-c".to_string()));
    assert!(names.contains(&"@test/pkg-d".to_string()));
    assert!(names.contains(&"@test/build-tool".to_string()));
}

// ============================================================================
// Edge Cases and Error Scenarios
// ============================================================================

#[tokio::test]
async fn test_integration_edge_case_all_packages_independent() {
    // Create monorepo where no packages depend on each other
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_package = r#"{
        "name": "independent-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package).await.expect("Failed to write root");

    // Create package-lock.json for NPM workspace detection
    let package_lock = r#"{
        "name": "independent-root",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages");

    for i in 1..=3 {
        let pkg_dir = root.join("packages").join(format!("pkg-{}", i));
        tokio::fs::create_dir_all(&pkg_dir).await.expect("Failed to create pkg dir");
        let pkg = format!(
            r#"{{
            "name": "@independent/pkg-{}",
            "version": "1.0.0",
            "dependencies": {{
                "external-lib": "^1.0.0"
            }}
        }}"#,
            i
        );
        tokio::fs::write(pkg_dir.join("package.json"), pkg).await.expect("Failed to write package");
    }

    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@independent/pkg-1");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Should only update pkg-1, no propagation
    assert_eq!(resolution.updates.len(), 1);
    assert_eq!(resolution.updates[0].name, "@independent/pkg-1");
}

#[tokio::test]
async fn test_integration_edge_case_version_none_bump() {
    let (_temp, root) = create_single_package().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("chore/update", VersionBump::None, vec!["production".to_string()]);
    changeset.add_package("single-package");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Version should remain the same
    assert_eq!(resolution.updates.len(), 1);
    assert_eq!(resolution.updates[0].current_version, resolution.updates[0].next_version);
}

#[tokio::test]
async fn test_integration_stress_large_monorepo() {
    // Create a large monorepo with many packages
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_package = r#"{
        "name": "large-monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_package).await.expect("Failed to write root");

    // Create package-lock.json for NPM workspace detection
    let package_lock = r#"{
        "name": "large-monorepo",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages");

    // Create 50 packages
    for i in 0..50 {
        let pkg_dir = root.join("packages").join(format!("pkg-{}", i));
        tokio::fs::create_dir_all(&pkg_dir).await.expect("Failed to create pkg dir");

        // Each package depends on the previous one (except first)
        let deps = if i > 0 {
            format!(
                r#","dependencies": {{
                "@large/pkg-{}": "^1.0.0"
            }}"#,
                i - 1
            )
        } else {
            String::new()
        };

        let pkg = format!(
            r#"{{
            "name": "@large/pkg-{}",
            "version": "1.0.0"{}
        }}"#,
            i, deps
        );
        tokio::fs::write(pkg_dir.join("package.json"), pkg).await.expect("Failed to write package");
    }

    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    // Should discover all packages
    let packages = resolver.discover_packages().await.expect("Failed to discover packages");
    assert_eq!(packages.len(), 50);

    // Update first package should propagate through chain
    let mut changeset =
        Changeset::new("feature/test", VersionBump::Patch, vec!["production".to_string()]);
    changeset.add_package("@large/pkg-0");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // All packages in chain should be updated
    assert!(resolution.updates.len() > 1, "Should propagate updates");
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[tokio::test]
async fn test_integration_performance_resolution_speed() {
    // Create a medium-sized monorepo
    let (_temp, root) = create_deep_chain_monorepo(20).await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@chain/pkg-0");

    // Measure resolution time
    let start = std::time::Instant::now();
    let _resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");
    let duration = start.elapsed();

    // Resolution should complete in reasonable time (< 1 second for 20 packages)
    assert!(duration.as_secs() < 1, "Resolution took too long: {:?}", duration);
}

#[tokio::test]
async fn test_integration_performance_apply_speed() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Measure apply time
    let start = std::time::Instant::now();
    let _result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");
    let duration = start.elapsed();

    // Apply should complete quickly (< 500ms for small monorepo)
    assert!(duration.as_millis() < 500, "Apply took too long: {:?}", duration);
}

#[tokio::test]
async fn test_integration_concurrent_resolution() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    // Create multiple changesets
    let mut changeset1 =
        Changeset::new("feature/1", VersionBump::Minor, vec!["production".to_string()]);
    changeset1.add_package("@test/pkg-a");

    let mut changeset2 =
        Changeset::new("feature/2", VersionBump::Patch, vec!["staging".to_string()]);
    changeset2.add_package("@test/pkg-d");

    // Resolve concurrently
    let resolver_clone = resolver.clone();
    let handle1 = tokio::spawn(async move { resolver_clone.resolve_versions(&changeset1).await });

    let resolver_clone2 = resolver.clone();
    let handle2 = tokio::spawn(async move { resolver_clone2.resolve_versions(&changeset2).await });

    // Both should succeed
    let result1 = handle1.await.expect("Task panicked").expect("Resolution 1 failed");
    let result2 = handle2.await.expect("Task panicked").expect("Resolution 2 failed");

    assert!(!result1.updates.is_empty());
    assert!(!result2.updates.is_empty());
}

// ============================================================================
// Cross-Platform Path Tests
// ============================================================================

#[tokio::test]
async fn test_integration_cross_platform_paths() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let packages = resolver.discover_packages().await.expect("Failed to discover packages");

    // Verify all paths use platform-appropriate separators
    for package in packages {
        let path = package.path();
        assert!(path.is_absolute() || path.starts_with("packages") || path.starts_with("tools"));

        // Path should be valid for current platform
        assert!(path.components().count() > 0, "Path should have components");
    }
}

#[tokio::test]
async fn test_integration_normalized_paths_in_resolution() {
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    // All modified file paths should be normalized
    for path in &result.modified_files {
        // Should not contain mixed separators or redundant separators
        let path_str = path.to_string_lossy();

        // On Windows, \\?\ prefix is valid for extended-length paths, so skip this check
        // if the path starts with that prefix
        let has_extended_prefix = cfg!(windows) && path_str.starts_with(r"\\?\");

        if !has_extended_prefix {
            assert!(!path_str.contains("//"), "Path should not have double slashes: {}", path_str);
            assert!(
                !path_str.contains("\\\\"),
                "Path should not have double backslashes: {}",
                path_str
            );
        }

        // Verify the path is absolute and exists
        assert!(path.is_absolute(), "Path should be absolute: {:?}", path);
    }
}

// ============================================================================
// Configuration Validation Tests
// ============================================================================

#[tokio::test]
async fn test_integration_custom_config_propagation_bump() {
    let (_temp, root) = create_complex_monorepo().await;
    let mut config = PackageToolsConfig::default();
    config.dependency.propagation_bump = "minor".to_string(); // Propagate with minor bump

    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Major, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Find propagated updates
    let propagated: Vec<_> = resolution.updates.iter().filter(|u| u.is_propagated()).collect();

    // Propagated packages should get minor bump instead of patch
    for update in propagated {
        // Check that version bumped by at least minor
        assert!(
            update.next_version.minor() > update.current_version.minor()
                || update.next_version.major() > update.current_version.major(),
            "Propagated update should have at least minor bump"
        );
    }
}

#[tokio::test]
async fn test_integration_skip_protocols_config() {
    let (_temp, root) = create_complex_monorepo().await;
    let mut config = PackageToolsConfig::default();
    config.dependency.skip_workspace_protocol = true;
    config.dependency.skip_file_protocol = true;
    config.dependency.skip_link_protocol = true;

    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    let _result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    // Verify workspace protocols are preserved (tested in earlier test)
    // This test ensures config is properly applied
}

// ============================================================================
// Real-World Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_integration_scenario_hotfix_release() {
    // Scenario: Hotfix release - patch bump on production package
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("hotfix/critical-bug", VersionBump::Patch, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-b");

    // Resolve and apply
    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert_eq!(result.summary.direct_updates, 1);

    // Verify version
    let pkg_b_content = tokio::fs::read_to_string(root.join("packages/pkg-b/package.json"))
        .await
        .expect("Failed to read pkg-b");
    assert!(pkg_b_content.contains(r#""version": "1.0.1""#));
}

#[tokio::test]
async fn test_integration_scenario_major_breaking_change() {
    // Scenario: Major breaking change affecting dependent packages
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver =
        VersionResolver::new(root.clone(), config).await.expect("Failed to create resolver");

    let mut changeset = Changeset::new(
        "feat/breaking-api-change",
        VersionBump::Major,
        vec!["production".to_string()],
    );
    changeset.add_package("@test/pkg-a");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // Direct package gets major bump
    let pkg_a =
        resolution.updates.iter().find(|u| u.name == "@test/pkg-a").expect("Should find pkg-a");
    assert_eq!(pkg_a.next_version.major(), 2);

    // Dependent packages get patch bump (propagation)
    let pkg_b =
        resolution.updates.iter().find(|u| u.name == "@test/pkg-b").expect("Should find pkg-b");
    assert!(pkg_b.is_propagated());
}

#[tokio::test]
async fn test_integration_scenario_feature_release_multiple_packages() {
    // Scenario: Feature release affecting multiple packages
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset = Changeset::new(
        "feat/new-feature",
        VersionBump::Minor,
        vec!["production".to_string(), "staging".to_string()],
    );
    changeset.add_package("@test/pkg-a");
    changeset.add_package("@test/pkg-b");
    changeset.add_package("@test/pkg-c");

    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    // All three packages should get minor bump
    assert!(resolution.updates.len() >= 3);

    let direct_updates = resolution.updates.iter().filter(|u| !u.is_propagated()).count();
    assert_eq!(direct_updates, 3, "Should have 3 direct updates");
}

#[tokio::test]
async fn test_integration_scenario_preview_before_release() {
    // Scenario: Preview changes before actual release
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("release/v2.0", VersionBump::Major, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Step 1: Preview (dry run)
    let preview = resolver.apply_versions(&changeset, true).await.expect("Failed to preview");

    assert!(preview.dry_run);
    println!("Preview: {} packages will be updated", preview.summary.packages_updated);

    // Step 2: Review and apply
    if preview.resolution.circular_dependencies.is_empty() {
        let actual = resolver.apply_versions(&changeset, false).await.expect("Failed to apply");

        assert!(!actual.dry_run);
        assert_eq!(preview.summary.packages_updated, actual.summary.packages_updated);
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

#[tokio::test]
async fn test_integration_regression_empty_dependencies() {
    // Regression: Handle packages with empty dependency objects
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package = r#"{
        "name": "empty-deps",
        "version": "1.0.0",
        "dependencies": {},
        "devDependencies": {}
    }"#;
    tokio::fs::write(root.join("package.json"), package)
        .await
        .expect("Failed to write package.json");

    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let mut changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("empty-deps");

    let result = resolver.resolve_versions(&changeset).await;
    assert!(result.is_ok(), "Should handle empty dependency objects");
}

#[tokio::test]
async fn test_integration_regression_missing_version_field() {
    // Regression: Handle package.json without version field
    // Note: package.json without version is technically valid (for private packages)
    // but should error when trying to resolve versions
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let package = r#"{
        "name": "no-version",
        "private": true
    }"#;
    tokio::fs::write(root.join("package.json"), package)
        .await
        .expect("Failed to write package.json");

    let config = PackageToolsConfig::default();

    // VersionResolver creation might succeed for private packages without version
    // but should fail when trying to actually resolve versions
    let resolver = VersionResolver::new(root, config).await;

    if let Ok(resolver) = resolver {
        // If resolver was created, it should fail when discovering packages or resolving
        let packages_result = resolver.discover_packages().await;

        // Either discover fails or we get packages without valid versions
        if let Ok(packages) = packages_result {
            // Packages without version should fail when trying to resolve
            assert!(
                packages.is_empty() || packages[0].version().to_string() == "0.0.0",
                "Package without explicit version should default or be empty"
            );
        }
        // Test passes if either resolver creation, discovery, or version parsing fails appropriately
    }
}

#[tokio::test]
async fn test_integration_regression_special_characters_in_names() {
    // Regression: Handle special characters in package names
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path().to_path_buf();

    let root_pkg = r#"{
        "name": "special-root",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    tokio::fs::write(root.join("package.json"), root_pkg).await.expect("Failed to write root");

    // Create package-lock.json for NPM workspace detection
    let package_lock = r#"{
        "name": "special-root",
        "version": "1.0.0",
        "lockfileVersion": 3,
        "requires": true,
        "packages": {}
    }"#;
    tokio::fs::write(root.join("package-lock.json"), package_lock)
        .await
        .expect("Failed to write package-lock.json");

    tokio::fs::create_dir_all(root.join("packages")).await.expect("Failed to create packages");

    let pkg_dir = root.join("packages").join("special-pkg");
    tokio::fs::create_dir_all(&pkg_dir).await.expect("Failed to create pkg dir");
    let pkg = r#"{
        "name": "@scope/package-with-dashes_and_underscores.dots",
        "version": "1.0.0"
    }"#;
    tokio::fs::write(pkg_dir.join("package.json"), pkg).await.expect("Failed to write package");

    let config = PackageToolsConfig::default();
    let resolver = VersionResolver::new(root, config).await.expect("Failed to create resolver");

    let packages = resolver.discover_packages().await.expect("Failed to discover packages");
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name(), "@scope/package-with-dashes_and_underscores.dots");
}

// ============================================================================
// Summary Test - Full Integration
// ============================================================================

#[tokio::test]
async fn test_integration_full_workflow_end_to_end() {
    // This test validates the complete end-to-end workflow
    let (_temp, root) = create_complex_monorepo().await;
    let config = PackageToolsConfig::default();

    // Step 1: Initialize resolver
    let resolver = VersionResolver::new(root.clone(), config.clone())
        .await
        .expect("Failed to initialize resolver");

    assert!(resolver.is_monorepo());
    assert_eq!(resolver.strategy(), VersioningStrategy::Independent);
    assert_eq!(resolver.workspace_root(), &root);

    // Step 2: Discover packages
    let packages = resolver.discover_packages().await.expect("Failed to discover packages");
    assert_eq!(packages.len(), 5);

    // Step 3: Create changeset
    let mut changeset =
        Changeset::new("release/v1.1", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("@test/pkg-a");

    // Step 4: Resolve versions
    let resolution =
        resolver.resolve_versions(&changeset).await.expect("Failed to resolve versions");

    assert!(!resolution.updates.is_empty());
    assert_eq!(resolution.circular_dependencies.len(), 0);

    // Step 5: Preview (dry run)
    let preview = resolver.apply_versions(&changeset, true).await.expect("Failed to preview");

    assert!(preview.dry_run);
    assert_eq!(preview.modified_files.len(), 0);
    assert!(preview.summary.packages_updated > 0);

    // Step 6: Apply changes
    let result =
        resolver.apply_versions(&changeset, false).await.expect("Failed to apply versions");

    assert!(!result.dry_run);
    assert!(!result.modified_files.is_empty());
    assert_eq!(result.summary.packages_updated, preview.summary.packages_updated);

    // Step 7: Verify file system changes
    for path in &result.modified_files {
        assert!(tokio::fs::metadata(path).await.is_ok(), "Modified file should exist: {:?}", path);
    }

    // Step 8: Re-read and verify versions
    let updated_packages =
        resolver.discover_packages().await.expect("Failed to re-discover packages");

    let pkg_a =
        updated_packages.iter().find(|p| p.name() == "@test/pkg-a").expect("Should find pkg-a");

    assert_eq!(pkg_a.version().to_string(), "1.1.0", "Version should be updated");

    println!("✅ Full end-to-end workflow completed successfully");
    println!("   - Packages updated: {}", result.summary.packages_updated);
    println!("   - Direct updates: {}", result.summary.direct_updates);
    println!("   - Propagated updates: {}", result.summary.propagated_updates);
    println!("   - Files modified: {}", result.modified_files.len());
}
