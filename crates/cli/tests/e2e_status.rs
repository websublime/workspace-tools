//! # E2E Tests for Status Command
//!
//! **What**: End-to-end tests for the `status` command that displays workspace
//! information including repository type, package manager, active branch,
//! pending changesets, and package details.
//!
//! **How**: Creates real temporary workspaces with various configurations
//! (single package, monorepo, with/without git, with/without changesets),
//! executes the status command, and validates the output contains expected
//! workspace information.
//!
//! **Why**: Ensures the status command correctly detects and displays workspace
//! state across different workspace types, git states, and changeset scenarios.
//! Critical for users to understand their workspace configuration at a glance.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::{ChangesetBuilder, WorkspaceFixture};
use common::helpers::{create_quiet_output, create_shared_json_output, create_test_output};
use sublime_cli_tools::cli::commands::StatusArgs;
use sublime_cli_tools::commands::status::execute_status;
use sublime_cli_tools::output::OutputFormat;

// ============================================================================
// Helper Functions
// ============================================================================

/// Verifies JSON output structure from status command.
///
/// Validates that the output contains all expected status fields.
fn verify_status_json_output(json_str: &str) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");
    assert_eq!(json["success"], true, "Success should be true");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify all required status sections
    assert!(data.get("repository").is_some(), "Should have repository info");
    assert!(data.get("packageManager").is_some(), "Should have package manager info");
    assert!(data.get("packages").is_some(), "Should have packages list");

    // Repository info - uses "kind" not "type"
    let repository = &data["repository"];
    assert!(repository.get("kind").is_some(), "Should have repository kind");

    // Package manager info
    let pm = &data["packageManager"];
    assert!(pm.get("name").is_some(), "Should have package manager name");

    // Packages should be an array
    assert!(data["packages"].is_array(), "Packages should be an array");
}

/// Verifies JSON output contains expected repository kind.
fn verify_repository_type(json_str: &str, expected_type: &str) {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let repo_kind = json["data"]["repository"]["kind"].as_str().unwrap();
    assert_eq!(repo_kind, expected_type, "Repository kind should be {expected_type}");
}

/// Verifies JSON output contains expected number of packages.
fn verify_package_count(json_str: &str, expected_count: usize) {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let packages = json["data"]["packages"].as_array().unwrap();
    assert_eq!(
        packages.len(),
        expected_count,
        "Should have {expected_count} packages, found {}",
        packages.len()
    );
}

/// Verifies JSON output contains expected changeset count.
fn verify_changeset_count(json_str: &str, expected_count: usize) {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let changesets = json["data"]["changesets"].as_array().unwrap();
    assert_eq!(
        changesets.len(),
        expected_count,
        "Should have {expected_count} changesets, found {}",
        changesets.len()
    );
}

/// Verifies JSON output contains expected branch name.
fn verify_branch_name(json_str: &str, expected_branch: &str) {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    // Branch is optional and nested under "branch.name"
    let branch = json["data"]["branch"]["name"].as_str().unwrap_or("");
    assert_eq!(branch, expected_branch, "Branch should be {expected_branch}");
}

// ============================================================================
// Single Package Tests
// ============================================================================

/// Test: Status command displays single package workspace info
///
/// Verifies that the status command correctly identifies and displays
/// information for a single-package workspace.
#[tokio::test]
async fn test_status_single_package() {
    // ARRANGE: Create single package workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status command should succeed: {:?}", result.err());

    // Verify JSON output structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_status_json_output(&json_str);
    verify_repository_type(&json_str, "simple");
    verify_package_count(&json_str, 1);
}

/// Test: Status command with human output format
///
/// Verifies that status command produces human-readable output.
#[tokio::test]
async fn test_status_human_output() {
    // ARRANGE: Create single package workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command with human format
    let (output, _buffer) = create_test_output(OutputFormat::Human);
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status command with human output should succeed: {:?}", result.err());
}

/// Test: Status command with quiet output format
///
/// Verifies that status command works in quiet mode.
#[tokio::test]
async fn test_status_quiet_output() {
    // ARRANGE: Create single package workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command with quiet format
    let output = create_quiet_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status command with quiet output should succeed: {:?}", result.err());
}

// ============================================================================
// Monorepo Tests
// ============================================================================

/// Test: Status command displays monorepo workspace info
///
/// Verifies that the status command correctly identifies and displays
/// information for a monorepo workspace with multiple packages.
#[tokio::test]
async fn test_status_monorepo() {
    // ARRANGE: Create monorepo workspace
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status command should succeed for monorepo: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_status_json_output(&json_str);
    verify_repository_type(&json_str, "monorepo");
    verify_package_count(&json_str, 2);
}

/// Test: Status command displays monorepo with internal dependencies
///
/// Verifies that status correctly shows packages that depend on each other.
#[tokio::test]
async fn test_status_monorepo_with_dependencies() {
    // ARRANGE: Create monorepo with internal dependencies
    let workspace = WorkspaceFixture::monorepo_with_internal_deps()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed for monorepo with deps: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_status_json_output(&json_str);
    verify_package_count(&json_str, 2);
}

// ============================================================================
// Git Integration Tests
// ============================================================================

/// Test: Status command displays git branch information
///
/// Verifies that status shows the current git branch.
#[tokio::test]
async fn test_status_displays_git_branch() {
    // ARRANGE: Create workspace with git on main branch
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify git branch is shown
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Branch info should be present (it's under "branch", not "git")
    assert!(json["data"].get("branch").is_some(), "Should have branch info");
}

/// Test: Status command displays feature branch name
///
/// Verifies that status shows the correct branch when on a feature branch.
#[tokio::test]
async fn test_status_displays_feature_branch() {
    // ARRANGE: Create workspace with feature branch
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_branch("feature/test-branch")
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify branch name
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_branch_name(&json_str, "feature/test-branch");
}

/// Test: Status command without git repository
///
/// Verifies that status works in a non-git directory.
#[tokio::test]
async fn test_status_without_git() {
    // ARRANGE: Create workspace WITHOUT git
    let workspace = WorkspaceFixture::single_package().with_default_config().finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed even without git
    assert!(result.is_ok(), "Status should succeed without git: {:?}", result.err());

    // Verify output is valid
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_status_json_output(&json_str);
}

// ============================================================================
// Changeset Tests
// ============================================================================

/// Test: Status command displays pending changesets
///
/// Verifies that status shows pending changeset IDs.
#[tokio::test]
async fn test_status_displays_pending_changesets() {
    // ARRANGE: Create workspace with changesets
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .add_changeset(ChangesetBuilder::minor().branch("feature/add-feature"))
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify changesets are shown
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_changeset_count(&json_str, 1);
}

/// Test: Status command displays multiple changesets
///
/// Verifies that status correctly counts multiple pending changesets.
#[tokio::test]
async fn test_status_displays_multiple_changesets() {
    // ARRANGE: Create workspace with multiple changesets
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .add_changeset(ChangesetBuilder::minor().branch("feature/feature-a"))
        .add_changeset(ChangesetBuilder::patch().branch("fix/bug-fix"))
        .add_changeset(ChangesetBuilder::major().branch("breaking/api-change"))
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify all changesets are shown
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_changeset_count(&json_str, 3);
}

/// Test: Status command with no changesets
///
/// Verifies that status handles workspaces with no pending changesets.
#[tokio::test]
async fn test_status_no_changesets() {
    // ARRANGE: Create workspace without changesets
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed with no changesets: {:?}", result.err());

    // Verify zero changesets
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_changeset_count(&json_str, 0);
}

// ============================================================================
// Package Manager Detection Tests
// ============================================================================

/// Test: Status detects npm package manager
///
/// Verifies that status correctly identifies npm as the package manager.
#[tokio::test]
async fn test_status_detects_npm() {
    // ARRANGE: Create workspace with package-lock.json (npm)
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_package_lock()
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify package manager is detected
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let pm_name = json["data"]["packageManager"]["name"].as_str().unwrap();
    assert_eq!(pm_name, "npm", "Package manager should be npm");
}

// ============================================================================
// Custom Config Path Tests
// ============================================================================

/// Test: Status with custom config path
///
/// Verifies that status uses custom configuration path.
#[tokio::test]
async fn test_status_with_custom_config_path() {
    // ARRANGE: Create workspace with custom config location
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create config in custom location
    let custom_config = serde_json::json!({
        "changeset": {
            "path": ".changesets/"
        },
        "version": {
            "strategy": "independent"
        }
    });

    let custom_config_path = workspace.root().join("custom.config.json");
    std::fs::write(&custom_config_path, custom_config.to_string())
        .expect("Failed to write custom config");

    let args = StatusArgs {};

    // ACT: Execute status with custom config
    let (output, buffer) = create_shared_json_output();
    let result =
        execute_status(&args, &output, workspace.root(), Some(custom_config_path.as_path())).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status with custom config should succeed: {:?}", result.err());

    // Verify output is valid
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_status_json_output(&json_str);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test: Status fails gracefully without package.json
///
/// Verifies that status returns appropriate error when no package.json exists.
#[tokio::test]
async fn test_status_no_package_json() {
    // ARRANGE: Create empty temp directory (no package.json)
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    let args = StatusArgs {};

    // ACT: Execute status command
    let output = create_quiet_output();
    let result = execute_status(&args, &output, temp_dir.path(), None).await;

    // ASSERT: Command should fail
    assert!(result.is_err(), "Status should fail without package.json");
}

// ============================================================================
// Package Version Display Tests
// ============================================================================

/// Test: Status displays package versions correctly
///
/// Verifies that package versions are shown in the output.
#[tokio::test]
async fn test_status_displays_package_versions() {
    // ARRANGE: Create monorepo workspace
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify packages have version info
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let packages = json["data"]["packages"].as_array().unwrap();
    for package in packages {
        assert!(package.get("name").is_some(), "Package should have name");
        assert!(package.get("version").is_some(), "Package should have version");
        assert!(package.get("path").is_some(), "Package should have path");
    }
}

/// Test: Status displays package names correctly
///
/// Verifies that package names are correctly identified.
#[tokio::test]
async fn test_status_displays_package_names() {
    // ARRANGE: Create monorepo workspace
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = StatusArgs {};

    // ACT: Execute status command
    let (output, buffer) = create_shared_json_output();
    let result = execute_status(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Status should succeed: {:?}", result.err());

    // Verify package names
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let packages = json["data"]["packages"].as_array().unwrap();
    let names: Vec<&str> = packages.iter().map(|p| p["name"].as_str().unwrap()).collect();

    assert!(names.contains(&"@test/pkg-a"), "Should contain pkg-a");
    assert!(names.contains(&"@test/pkg-b"), "Should contain pkg-b");
}
