//! # E2E Tests for Init Command
//!
//! **What**: End-to-end tests for the `workspace init` command that initializes
//! a workspace for changeset-based version management.
//!
//! **How**: Creates real temporary workspaces, executes the init command with various
//! configurations, and validates that all expected files and directories are created
//! with correct content.
//!
//! **Why**: Ensures the init command works correctly across different workspace types,
//! configuration formats, and edge cases. Validates the entire initialization flow.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::{ChangesetBuilder, WorkspaceFixture};
use std::path::PathBuf;
use sublime_cli_tools::cli::commands::InitArgs;
use sublime_cli_tools::commands::init::execute_init;
use sublime_cli_tools::output::OutputFormat;

// ============================================================================
// Basic Init Tests
// ============================================================================

/// Test: Init creates configuration in single package workspace
#[tokio::test]
async fn test_init_single_package_creates_config() {
    // Create a single package workspace (no config)
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    // Execute init
    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init should succeed: {:?}", result.err());

    // Verify config file was created
    workspace.assert_config_exists();

    // Verify .changesets directory was created
    let changesets_dir = workspace.root().join(".changesets");
    assert!(changesets_dir.exists(), "Changesets directory should be created");

    // Verify .changesets/history directory was created
    let history_dir = workspace.root().join(".changesets/history");
    assert!(history_dir.exists(), "History directory should be created");

    // Verify .workspace-backups directory was created
    let backups_dir = workspace.root().join(".workspace-backups");
    assert!(backups_dir.exists(), "Backups directory should be created");
}

/// Test: Init creates configuration in monorepo workspace
#[tokio::test]
async fn test_init_monorepo_creates_config() {
    // Create an independent monorepo workspace (no config)
    let workspace = WorkspaceFixture::monorepo_independent().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    // Execute init
    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init should succeed: {:?}", result.err());

    // Verify config exists
    workspace.assert_config_exists();

    // Verify all required directories were created
    assert!(workspace.root().join(".changesets").exists());
    assert!(workspace.root().join(".changesets/history").exists());
    assert!(workspace.root().join(".workspace-backups").exists());
}

/// Test: Init with unified strategy
#[tokio::test]
async fn test_init_unified_strategy() {
    let workspace = WorkspaceFixture::monorepo_unified().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("unified".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init should succeed for unified strategy");

    workspace.assert_config_exists();
}

/// Test: Init with multiple environments
#[tokio::test]
async fn test_init_multiple_environments() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec![
            "development".to_string(),
            "staging".to_string(),
            "production".to_string(),
        ]),
        default_env: Some(vec!["staging".to_string(), "production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init should succeed with multiple environments");

    workspace.assert_config_exists();
}

// ============================================================================
// Config Format Tests
// ============================================================================

/// Test: Init with JSON config format
#[tokio::test]
async fn test_init_json_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok());

    let config_path = workspace.root().join("repo.config.json");
    assert!(config_path.exists(), "JSON config file should be created");
}

/// Test: Init with TOML config format
#[tokio::test]
async fn test_init_toml_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("toml".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok());

    let config_path = workspace.root().join("repo.config.toml");
    assert!(config_path.exists(), "TOML config file should be created");
}

/// Test: Init with YAML config format
#[tokio::test]
async fn test_init_yaml_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("yaml".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok());

    let config_path = workspace.root().join("repo.config.yaml");
    assert!(config_path.exists(), "YAML config file should be created");
}

// ============================================================================
// Force/Overwrite Tests
// ============================================================================

/// Test: Init fails when config already exists without force flag
#[tokio::test]
async fn test_init_fails_when_config_exists() {
    // Create workspace with existing config
    let workspace = WorkspaceFixture::single_package().with_default_config().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_err(), "Init should fail when config already exists");
}

/// Test: Init succeeds with force flag when config exists
#[tokio::test]
async fn test_init_force_overwrites_config() {
    // Create workspace with existing config
    let workspace = WorkspaceFixture::single_package().with_default_config().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["development".to_string(), "production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("unified".to_string()), // Different from default
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: true, // Force overwrite
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init with force should succeed: {:?}", result.err());

    workspace.assert_config_exists();
}

// ============================================================================
// Error Cases Tests
// ============================================================================

/// Test: Init fails in directory without package.json
#[tokio::test]
async fn test_init_fails_without_package_json() {
    // Create empty temp directory (no package.json)
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp_dir.path();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, root, OutputFormat::Json).await;
    assert!(result.is_err(), "Init should fail without package.json");
}

/// Test: Init with custom changeset path
#[tokio::test]
async fn test_init_custom_changeset_path() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".custom-changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init with custom path should succeed");

    // Verify custom path was created
    let custom_dir = workspace.root().join(".custom-changesets");
    assert!(custom_dir.exists(), "Custom changesets directory should be created");
}

/// Test: Init with custom NPM registry
#[tokio::test]
async fn test_init_custom_registry() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://custom-registry.example.com".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    let result = execute_init(&args, workspace.root(), OutputFormat::Json).await;
    assert!(result.is_ok(), "Init with custom registry should succeed");

    workspace.assert_config_exists();
}
