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
use common::helpers::read_json_file;
use std::path::{Path, PathBuf};
use sublime_cli_tools::cli::commands::CloneArgs;
use sublime_cli_tools::commands::clone::execute_clone;
use sublime_cli_tools::output::OutputFormat;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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

    let result = execute_clone(&args_absolute, clone_dest.path(), None, OutputFormat::Quiet).await;
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

    let result = execute_clone(&args_relative, clone_dest.path(), None, OutputFormat::Quiet).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Json).await;
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
    let result = execute_clone(&args, clone_dest.path(), None, OutputFormat::Quiet).await;
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
