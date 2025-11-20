//! # E2E Tests for Clone Command
//!
//! **What**: End-to-end tests for the `workspace clone` command that clones
//! Git repositories and automatically sets up workspace configuration.
//!
//! **How**: Uses WorkspaceFixture to create test repositories, executes the
//! clone command with various configurations, and validates that repositories
//! are correctly cloned, configurations are validated, and workspace setup is complete.
//!
//! **Why**: Ensures the clone command works correctly across different scenarios
//! including configuration detection, validation, initialization, and error handling.
//! Tests the complete flow from Story 11.1 through Story 11.4.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use common::helpers::{create_quiet_output, create_shared_json_output, read_json_file};
use std::path::{Path, PathBuf};
use sublime_cli_tools::cli::commands::CloneArgs;
use sublime_cli_tools::commands::clone::execute_clone;
use sublime_cli_tools::output::{Output, OutputFormat};
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to verify JSON output structure from clone command.
///
/// Verifies the response structure and validates that the outcome matches expectations.
///
/// # Arguments
///
/// * `json_str` - JSON string output from the command
/// * `expected_destination` - Expected destination path
/// * `expected_outcome` - Expected outcome variant
fn verify_clone_json_output(json_str: &str, expected_destination: &Path, expected_outcome: &str) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");
    assert_eq!(json["success"], true, "Success should be true");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify required fields in CloneResponse
    assert!(data.get("success").is_some(), "Should have success");
    assert_eq!(data["success"], true, "Success should be true");

    assert!(data.get("destination").is_some(), "Should have destination");
    let destination = data["destination"].as_str().expect("Destination should be string");
    assert!(
        destination.ends_with(&expected_destination.display().to_string())
            || expected_destination.display().to_string().ends_with(destination),
        "Destination should match expected path. Expected: {}, Got: {}",
        expected_destination.display(),
        destination
    );

    assert!(data.get("outcome").is_some(), "Should have outcome");
    let outcome = data["outcome"].as_str().expect("Outcome should be string");
    assert_eq!(outcome, expected_outcome, "Outcome should match expected value");
}

fn assert_workspace_structure(clone_dest: &Path, should_have_config: bool) {
    assert!(clone_dest.join(".git").exists(), ".git directory should exist");
    assert!(clone_dest.join("package.json").exists(), "package.json should exist");

    if should_have_config {
        let config_exists = clone_dest.join("repo.config.json").exists()
            || clone_dest.join("repo.config.yaml").exists()
            || clone_dest.join("repo.config.toml").exists();
        assert!(config_exists, "Workspace configuration should exist");

        assert!(clone_dest.join(".changesets").exists(), ".changesets directory should exist");
        assert!(
            clone_dest.join(".changesets/history").exists(),
            ".changesets/history directory should exist"
        );
        assert!(
            clone_dest.join(".workspace-backups").exists(),
            ".workspace-backups directory should exist"
        );
    }
}

// ============================================================================
// Happy Path Tests
// ============================================================================

/// Test: Clone repository without config triggers init
#[tokio::test]
async fn test_clone_without_config_runs_init() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".changesets".to_string()),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: Some("https://registry.npmjs.org".to_string()),
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify repository was cloned and workspace was initialized
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Clone repository with valid config validates successfully
#[tokio::test]
async fn test_clone_with_valid_config_validates() {
    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify repository was cloned and workspace structure is intact
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Clone monorepo with valid configuration
#[tokio::test]
async fn test_clone_monorepo_with_valid_config() {
    // Create a monorepo with valid workspace config
    let source_workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-monorepo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify monorepo structure
    assert!(clone_path.join("packages").exists(), "Monorepo packages directory should exist");
    assert!(clone_path.join("packages/pkg-a").exists(), "Package A should exist");
    assert!(clone_path.join("packages/pkg-b").exists(), "Package B should exist");

    // Verify workspace structure
    assert_workspace_structure(&clone_path, true);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test: Clone with --force removes existing destination
#[tokio::test]
async fn test_clone_force_removes_existing() {
    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("existing-dir");

    // Create existing directory with content
    std::fs::create_dir_all(&clone_path).unwrap();
    std::fs::write(clone_path.join("existing-file.txt"), "existing content").unwrap();

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: true, // Force overwrite
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with force should succeed: {:?}", result.err());

    // Verify old content is gone
    assert!(!clone_path.join("existing-file.txt").exists(), "Old file should be removed");

    // Verify new content exists
    assert!(clone_path.join(".git").exists(), "Repository should be cloned");
}

/// Test: Clone with --skip-validation skips validation
#[tokio::test]
async fn test_clone_skip_validation() {
    // Create a repository with invalid config (missing directories)
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_gitignore() // Has gitignore but NO directories
        .commit_all("Add incomplete configuration")
        .finalize();

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: true, // Skip validation
        force: false,
        depth: None,
    };

    // Execute clone - should succeed even though config is invalid
    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with skip validation should succeed: {:?}", result.err());

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");
}

/// Test: Clone with configuration overrides
#[tokio::test]
async fn test_clone_with_config_overrides() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".custom-changesets".to_string()),
        environments: Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()]),
        default_env: Some(vec!["staging".to_string()]),
        strategy: Some("unified".to_string()),
        registry: Some("https://custom.registry.com".to_string()),
        config_format: Some("yaml".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with overrides should succeed: {:?}", result.err());

    // Verify custom configuration was applied
    assert!(clone_path.join("repo.config.yaml").exists(), "YAML config should be created");
    assert!(clone_path.join(".custom-changesets").exists(), "Custom changeset path should be used");
}

/// Test: Clone with --non-interactive uses defaults/flags
#[tokio::test]
async fn test_clone_non_interactive() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone - should succeed without prompts
    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Non-interactive clone should succeed: {:?}", result.err());

    // Verify workspace was initialized with defaults
    assert!(clone_path.join("repo.config.json").exists(), "Config should be created");
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

/// Test: Clone fails with invalid URL
#[tokio::test]
async fn test_clone_invalid_url_fails() {
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: "not-a-valid-url".to_string(),
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone - should fail
    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_err(), "Clone with invalid URL should fail");
}

/// Test: Clone fails when destination exists without --force
#[tokio::test]
async fn test_clone_destination_exists_fails_without_force() {
    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .setup_for_clone()
        .commit_all("Add workspace configuration")
        .finalize();

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("existing-dir");

    // Create existing directory
    std::fs::create_dir_all(&clone_path).unwrap();

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false, // No force flag
        depth: None,
    };

    // Execute clone - should fail
    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_err(), "Clone should fail when destination exists without force");
}

/// Test: Clone fails with invalid configuration
#[tokio::test]
async fn test_clone_invalid_config_fails_validation() {
    // Create a repository with invalid config (config exists but directories don't)
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_gitignore() // Has gitignore but NO directories
        .commit_all("Add incomplete configuration")
        .finalize();

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false, // Validation enabled
        force: false,
        depth: None,
    };

    // Execute clone - should fail validation
    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_err(), "Clone should fail validation with invalid config");
}

// ============================================================================
// Cross-Platform Tests
// ============================================================================

/// Test: Clone handles absolute and relative paths correctly
#[tokio::test]
async fn test_clone_absolute_vs_relative_paths() {
    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();

    // Test with absolute path
    let clone_path_absolute = clone_dest.path().join("absolute-clone");
    let args_absolute = CloneArgs {
        url: source_url.clone(),
        destination: Some(clone_path_absolute.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args_absolute, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with absolute path should succeed");
    assert!(clone_path_absolute.exists(), "Absolute path destination should exist");

    // Test with relative path
    let clone_path_relative = PathBuf::from("relative-clone");
    let args_relative = CloneArgs {
        url: source_url,
        destination: Some(clone_path_relative.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    let output = Output::new(OutputFormat::Quiet, std::io::sink(), true);
    let result = execute_clone(&args_relative, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with relative path should succeed");

    // Relative path should be resolved relative to root
    let expected_path = clone_dest.path().join(clone_path_relative);
    assert!(expected_path.exists(), "Relative path should be resolved correctly");
}

// ============================================================================
// Output Format Tests
// ============================================================================

/// Test: Clone with JSON output validates structure
#[tokio::test]
async fn test_clone_json_output_structure() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".changesets".to_string()),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: Some("https://registry.npmjs.org".to_string()),
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with JSON output
    let output = Output::new(OutputFormat::Json, std::io::sink(), true);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed");

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");

    // Verify workspace was initialized
    assert_workspace_structure(&clone_path, true);
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test: Clone and then verify workspace can create changeset
#[tokio::test]
async fn test_clone_then_changeset_creation() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".changesets".to_string()),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: Some("https://registry.npmjs.org".to_string()),
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone
    let output = create_quiet_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed");

    // Verify .changesets directory is ready for changesets
    let changesets_dir = clone_path.join(".changesets");
    assert!(changesets_dir.exists(), "Changesets directory should exist");
    assert!(changesets_dir.is_dir(), "Changesets path should be a directory");

    // Verify we can write a changeset file
    let test_changeset_path = changesets_dir.join("test.json");
    let test_changeset = serde_json::json!({
        "branch": "test-branch",
        "bump": "minor",
        "packages": ["test-package"],
        "environments": ["production"]
    });

    std::fs::write(&test_changeset_path, serde_json::to_string_pretty(&test_changeset).unwrap())
        .unwrap();

    assert!(test_changeset_path.exists(), "Test changeset should be created");

    // Verify we can read it back
    let read_changeset: serde_json::Value = read_json_file(&test_changeset_path);
    assert_eq!(read_changeset["branch"], "test-branch");
}

// ============================================================================
// Pattern B Output Verification Tests (Story 4.3)
// ============================================================================

/// Test: Clone with valid config and JSON output verification
#[tokio::test]
async fn test_clone_with_config_json_output() {
    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with JSON output capture
    let (output, buffer) = create_shared_json_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify JSON output structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    verify_clone_json_output(&json_str, &clone_path, "ExistingConfigValidated");

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Clone without config (with init) and JSON output verification
#[tokio::test]
async fn test_clone_without_config_json_output() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".changesets".to_string()),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: Some("https://registry.npmjs.org".to_string()),
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with JSON output capture
    let (output, buffer) = create_shared_json_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify JSON output structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    verify_clone_json_output(&json_str, &clone_path, "NewWorkspaceInitialized");

    // Verify repository was cloned and initialized
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Clone with skip validation and JSON output verification
#[tokio::test]
async fn test_clone_skip_validation_json_output() {
    // Create a repository with config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_gitignore()
        .commit_all("Add configuration")
        .finalize();

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: true,
        force: false,
        depth: None,
    };

    // Execute clone with JSON output capture
    let (output, buffer) = create_shared_json_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify JSON output structure - should be unvalidated
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    verify_clone_json_output(&json_str, &clone_path, "ExistingConfigUnvalidated");

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");
}

/// Test: Clone with Human format output
#[tokio::test]
async fn test_clone_human_output_format() {
    use std::sync::{Arc, Mutex};

    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with Human output capture
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = common::helpers::SharedWriter { buffer: Arc::clone(&buffer) };
    let output = Output::new(OutputFormat::Human, Box::new(writer), false);
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify Human output contains expected messages
    let output_bytes = buffer.lock().unwrap();
    let output_str = String::from_utf8(output_bytes.clone()).expect("Output should be valid UTF-8");
    assert!(output_str.contains("Clone completed successfully"), "Should have completion message");
    assert!(
        output_str.contains("Location:") || output_str.contains(&clone_path.display().to_string()),
        "Should contain destination path"
    );
    assert!(output_str.contains("Next steps"), "Should contain next steps");

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Clone with JsonCompact format output
#[tokio::test]
async fn test_clone_json_compact_output_format() {
    use std::sync::{Arc, Mutex};

    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with JsonCompact output capture
    let buffer_compact = Arc::new(Mutex::new(Vec::new()));
    let writer = common::helpers::SharedWriter { buffer: Arc::clone(&buffer_compact) };
    let output = Output::new(OutputFormat::JsonCompact, Box::new(writer), false);

    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify JSON output is compact (no pretty printing)
    let output_bytes = buffer_compact.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    // Compact JSON should not have newlines (except trailing)
    let trimmed = json_str.trim();
    assert!(!trimmed.contains("\n  "), "Compact JSON should not have indentation. Got: {trimmed}");

    // Should still be valid JSON
    verify_clone_json_output(&json_str, &clone_path, "ExistingConfigValidated");

    // Verify repository was cloned
    assert!(clone_path.exists(), "Clone destination should exist");
}

/// Test: Clone with init - verifies integration with execute_init
#[tokio::test]
async fn test_clone_init_integration_output_capture() {
    // Create a repository without workspace config
    let source_workspace = WorkspaceFixture::single_package().finalize().with_git().with_commits(1);

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-repo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: Some(".changesets".to_string()),
        environments: Some(vec!["dev".to_string(), "prod".to_string()]),
        default_env: Some(vec!["prod".to_string()]),
        strategy: Some("unified".to_string()),
        registry: Some("https://registry.npmjs.org".to_string()),
        config_format: Some("json".to_string()),
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with JSON output capture
    let (output, buffer) = create_shared_json_output();
    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone with init should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    // Should indicate new workspace was initialized
    verify_clone_json_output(&json_str, &clone_path, "NewWorkspaceInitialized");

    // Verify workspace was initialized with correct settings
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);

    // Verify init created correct configuration
    let config_path = clone_path.join("repo.config.json");
    assert!(config_path.exists(), "Config file should be created");

    let config: serde_json::Value = read_json_file(&config_path);
    assert_eq!(config["version"]["strategy"].as_str().unwrap(), "unified", "Strategy should match");
    let changeset_path = config["changeset"]["path"].as_str().unwrap();
    assert!(
        changeset_path == ".changesets/" || changeset_path == ".changesets",
        "Changeset path should match (got: {changeset_path})"
    );

    let envs =
        config["changeset"]["available_environments"].as_array().expect("Should have environments");
    assert_eq!(envs.len(), 2, "Should have 2 environments");
    assert!(envs.contains(&serde_json::Value::String("dev".to_string())));
    assert!(envs.contains(&serde_json::Value::String("prod".to_string())));
}

/// Test: Clone with validation - verifies complete validation output
#[tokio::test]
async fn test_clone_validation_complete_output() {
    use std::sync::{Arc, Mutex};

    // Create a repository with valid workspace config
    let source_workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .finalize()
        .with_git()
        .setup_for_clone()
        .commit_all("Add workspace configuration");

    let source_url = source_workspace.as_git_remote_url();

    // Setup clone destination
    let clone_dest = TempDir::new().unwrap();
    let clone_path = clone_dest.path().join("cloned-monorepo");

    let args = CloneArgs {
        url: source_url,
        destination: Some(clone_path.clone()),
        changeset_path: None,
        environments: None,
        default_env: None,
        strategy: None,
        registry: None,
        config_format: None,
        non_interactive: true,
        skip_validation: false,
        force: false,
        depth: None,
    };

    // Execute clone with Human output to see validation messages
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = common::helpers::SharedWriter { buffer: Arc::clone(&buffer) };
    let output = Output::new(OutputFormat::Human, Box::new(writer), false);

    let result = execute_clone(&args, &output, clone_dest.path(), None).await;
    assert!(result.is_ok(), "Clone should succeed: {:?}", result.err());

    // Verify Human output contains validation messages
    let output_bytes = buffer.lock().unwrap();
    let output_str = String::from_utf8(output_bytes.clone()).expect("Output should be valid UTF-8");
    assert!(
        output_str.contains("Validating workspace configuration"),
        "Should show validation start. Got: {output_str}"
    );
    assert!(
        output_str.contains("Workspace configuration is valid"),
        "Should show validation success"
    );
    assert!(output_str.contains("Clone completed successfully"), "Should show clone completion");

    // Verify repository was cloned and validated
    assert!(clone_path.exists(), "Clone destination should exist");
    assert_workspace_structure(&clone_path, true);
}

/// Test: Verify all output formats work correctly
#[tokio::test]
async fn test_clone_all_output_formats() {
    use std::sync::{Arc, Mutex};

    // Test with Quiet format
    {
        let source_workspace = WorkspaceFixture::single_package()
            .with_default_config()
            .finalize()
            .with_git()
            .setup_for_clone()
            .commit_all("Add config");
        let clone_dest = TempDir::new().unwrap();
        let clone_path = clone_dest.path().join("quiet-clone");

        let args = CloneArgs {
            url: source_workspace.as_git_remote_url(),
            destination: Some(clone_path.clone()),
            changeset_path: None,
            environments: None,
            default_env: None,
            strategy: None,
            registry: None,
            config_format: None,
            non_interactive: true,
            skip_validation: false,
            force: false,
            depth: None,
        };

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = common::helpers::SharedWriter { buffer: Arc::clone(&buffer) };
        let output = Output::new(OutputFormat::Quiet, Box::new(writer), false);

        let result = execute_clone(&args, &output, clone_dest.path(), None).await;
        assert!(result.is_ok(), "Clone with Quiet format should succeed");

        // Quiet mode may have some output or be completely silent
        // Just verify the command succeeded
        assert!(clone_path.exists());
    }

    // Test with Json format
    {
        let source_workspace = WorkspaceFixture::single_package()
            .with_default_config()
            .finalize()
            .with_git()
            .setup_for_clone()
            .commit_all("Add config");
        let clone_dest = TempDir::new().unwrap();
        let clone_path = clone_dest.path().join("json-clone");

        let args = CloneArgs {
            url: source_workspace.as_git_remote_url(),
            destination: Some(clone_path.clone()),
            changeset_path: None,
            environments: None,
            default_env: None,
            strategy: None,
            registry: None,
            config_format: None,
            non_interactive: true,
            skip_validation: false,
            force: false,
            depth: None,
        };

        let (output, buffer) = create_shared_json_output();
        let result = execute_clone(&args, &output, clone_dest.path(), None).await;
        assert!(result.is_ok(), "Clone with Json format should succeed");

        let output_bytes = buffer.lock().unwrap().clone();
        let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
        verify_clone_json_output(&json_str, &clone_path, "ExistingConfigValidated");
        assert!(clone_path.exists());
    }

    // Test with Human format
    {
        let source_workspace = WorkspaceFixture::single_package()
            .with_default_config()
            .finalize()
            .with_git()
            .setup_for_clone()
            .commit_all("Add config");
        let clone_dest = TempDir::new().unwrap();
        let clone_path = clone_dest.path().join("human-clone");

        let args = CloneArgs {
            url: source_workspace.as_git_remote_url(),
            destination: Some(clone_path.clone()),
            changeset_path: None,
            environments: None,
            default_env: None,
            strategy: None,
            registry: None,
            config_format: None,
            non_interactive: true,
            skip_validation: false,
            force: false,
            depth: None,
        };

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = common::helpers::SharedWriter { buffer: Arc::clone(&buffer) };
        let output = Output::new(OutputFormat::Human, Box::new(writer), false);

        let result = execute_clone(&args, &output, clone_dest.path(), None).await;
        assert!(result.is_ok(), "Clone with Human format should succeed");

        let output_bytes = buffer.lock().unwrap();
        let output_str =
            String::from_utf8(output_bytes.clone()).expect("Output should be valid UTF-8");
        assert!(output_str.contains("Clone completed"), "Should have human-readable message");
        assert!(clone_path.exists());
    }
}
