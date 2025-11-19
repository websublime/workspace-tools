//! # E2E Tests for Init Command
//!
//! **What**: End-to-end tests for the `workspace init` command that initializes
//! a workspace for changeset-based version management.
//!
//! **How**: Creates real temporary workspaces, executes the init command with various
//! configurations, validates that all expected files and directories are created
//! with correct content, and verifies output in all formats.
//!
//! **Why**: Ensures the init command works correctly across different workspace types,
//! configuration formats, and edge cases. Validates the entire initialization flow
//! and output formatting according to Pattern B.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use common::helpers::create_shared_json_output;
use std::io::Cursor;
use std::path::PathBuf;
use sublime_cli_tools::cli::commands::InitArgs;
use sublime_cli_tools::commands::init::execute_init;
use sublime_cli_tools::output::{Output, OutputFormat};

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to create standard init args for testing.
fn create_init_args(strategy: &str, config_format: &str) -> InitArgs {
    InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some(strategy.to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some(config_format.to_string()),
        force: false,
        non_interactive: true,
    }
}

/// Helper to verify JSON output structure from init command.
fn verify_json_output(json_str: &str, expected_strategy: &str, expected_format: &str) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");
    assert_eq!(json["success"], true, "Success should be true");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify all required fields in InitResult
    assert!(data.get("configFile").is_some(), "Should have configFile");
    assert!(data.get("configFormat").is_some(), "Should have configFormat");
    assert!(data.get("strategy").is_some(), "Should have strategy");
    assert!(data.get("changesetPath").is_some(), "Should have changesetPath");
    assert!(data.get("environments").is_some(), "Should have environments");
    assert!(data.get("defaultEnvironments").is_some(), "Should have defaultEnvironments");
    assert!(data.get("registry").is_some(), "Should have registry");

    // Verify specific values
    assert_eq!(data["strategy"].as_str().unwrap(), expected_strategy, "Strategy should match");
    assert_eq!(
        data["configFormat"].as_str().unwrap(),
        expected_format,
        "Config format should match"
    );
    assert_eq!(
        data["changesetPath"].as_str().unwrap(),
        ".changesets",
        "Changeset path should match"
    );
    assert_eq!(
        data["registry"].as_str().unwrap(),
        "https://registry.npmjs.org",
        "Registry should match"
    );

    // Verify arrays
    let environments = data["environments"].as_array().expect("Environments should be array");
    assert_eq!(environments.len(), 1, "Should have 1 environment");
    assert_eq!(environments[0], "production", "Environment should be production");

    let default_envs =
        data["defaultEnvironments"].as_array().expect("Default environments should be array");
    assert_eq!(default_envs.len(), 1, "Should have 1 default environment");
    assert_eq!(default_envs[0], "production", "Default environment should be production");
}

// ============================================================================
// Basic Init Tests
// ============================================================================

/// Test: Init creates configuration in single package workspace
#[tokio::test]
async fn test_init_single_package_creates_config() {
    // Create a single package workspace (no config)
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    // Execute init
    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

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

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    // Execute init
    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

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

    let args = create_init_args("unified", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed for unified strategy");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "unified", "json");

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

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with multiple environments");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(json["success"], true);
    let environments = json["data"]["environments"].as_array().unwrap();
    assert_eq!(environments.len(), 3, "Should have 3 environments");
    let default_envs = json["data"]["defaultEnvironments"].as_array().unwrap();
    assert_eq!(default_envs.len(), 2, "Should have 2 default environments");

    workspace.assert_config_exists();
}

// ============================================================================
// Config Format Tests
// ============================================================================

/// Test: Init with JSON config format
#[tokio::test]
async fn test_init_json_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

    let config_path = workspace.root().join("repo.config.json");
    assert!(config_path.exists(), "JSON config file should be created");
}

/// Test: Init with TOML config format
#[tokio::test]
async fn test_init_toml_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "toml");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "toml");

    let config_path = workspace.root().join("repo.config.toml");
    assert!(config_path.exists(), "TOML config file should be created");
}

/// Test: Init with YAML config format
#[tokio::test]
async fn test_init_yaml_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "yaml");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "yaml");

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

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, _buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
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

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init with force should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["strategy"], "unified");

    // Verify it has 2 environments as specified in args
    let environments = json["data"]["environments"].as_array().unwrap();
    assert_eq!(environments.len(), 2, "Should have 2 environments");
    assert!(environments.contains(&serde_json::Value::String("development".to_string())));
    assert!(environments.contains(&serde_json::Value::String("production".to_string())));

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

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, _buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, root, None).await;
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

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init with custom path should succeed");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(json["data"]["changesetPath"], ".custom-changesets");

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

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init with custom registry should succeed");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(json["data"]["registry"], "https://custom-registry.example.com");

    workspace.assert_config_exists();
}

// ============================================================================
// Additional Init Tests - Gap Coverage
// ============================================================================

/// Test: Init with default environments specified
#[tokio::test]
async fn test_init_with_default_environments() {
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

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init with default environments should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    assert_eq!(json["success"], true);
    let default_envs = json["data"]["defaultEnvironments"].as_array().unwrap();
    assert_eq!(default_envs.len(), 2);
    assert!(default_envs.contains(&serde_json::Value::String("staging".to_string())));
    assert!(default_envs.contains(&serde_json::Value::String("production".to_string())));

    workspace.assert_config_exists();

    // Verify config file was created successfully
    // Note: The actual structure of the config file is validated by the config module tests
    let config_path = workspace.root().join("repo.config.json");
    assert!(config_path.exists(), "Config file should exist");

    // Verify file is valid JSON and not empty
    let config_content = std::fs::read_to_string(&config_path).expect("Should read config file");
    assert!(!config_content.is_empty(), "Config file should not be empty");

    let _config: serde_json::Value =
        serde_json::from_str(&config_content).expect("Config should be valid JSON");
}

/// Test: Init fails with invalid strategy
#[tokio::test]
async fn test_init_invalid_strategy_fails() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("invalid-strategy".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    // Create output with buffer for capture
    let (output, _buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_err(), "Init should fail with invalid strategy");

    let error_msg = result.unwrap_err().to_string().to_lowercase();
    assert!(
        error_msg.contains("strategy") || error_msg.contains("invalid"),
        "Error should mention strategy validation: {error_msg}"
    );
}

/// Test: Init fails with invalid config format
#[tokio::test]
async fn test_init_invalid_format_fails() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("xml".to_string()), // Invalid format
        force: false,
        non_interactive: true,
    };

    // Create output with buffer for capture
    let (output, _buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_err(), "Init should fail with invalid config format");

    let error_msg = result.unwrap_err().to_string().to_lowercase();
    assert!(
        error_msg.contains("format") || error_msg.contains("invalid") || error_msg.contains("xml"),
        "Error should mention format validation: {error_msg}"
    );
}

/// Test: Init fails with invalid registry URL
#[tokio::test]
async fn test_init_invalid_registry_url_fails() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "not-a-valid-url".to_string(), // Invalid URL
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: true,
    };

    // Create output with buffer for capture
    let (output, _buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_err(), "Init should fail with invalid registry URL");

    let error_msg = result.unwrap_err().to_string().to_lowercase();
    assert!(
        error_msg.contains("registry")
            || error_msg.contains("url")
            || error_msg.contains("invalid"),
        "Error should mention registry URL validation: {error_msg}"
    );
}

/// Test: Init creates .gitignore entries for workspace directories
#[tokio::test]
async fn test_init_creates_gitignore_entries() {
    let workspace = WorkspaceFixture::single_package()
        .with_git() // Initialize git repository
        .finalize();

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with git repository");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

    // Verify .gitignore exists and contains workspace entries
    let gitignore_path = workspace.root().join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore should exist");

    let gitignore_content =
        std::fs::read_to_string(&gitignore_path).expect("Should read .gitignore file");

    // Check for workspace-specific gitignore entries
    assert!(
        gitignore_content.contains(".workspace-backups")
            || gitignore_content.contains("workspace-backups"),
        ".gitignore should contain workspace-backups entry"
    );
}

/// Test: Init succeeds without git initialized
#[tokio::test]
async fn test_init_with_git_not_initialized() {
    // Create workspace WITHOUT git
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed even without git repository");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

    workspace.assert_config_exists();

    // Verify directories are created
    assert!(workspace.root().join(".changesets").exists());
    assert!(workspace.root().join(".changesets/history").exists());
    assert!(workspace.root().join(".workspace-backups").exists());
}

/// Test: Init interactive mode prompts for configuration
#[tokio::test]
async fn test_init_interactive_prompts() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".changesets"),
        environments: Some(vec!["production".to_string()]),
        default_env: Some(vec!["production".to_string()]),
        strategy: Some("independent".to_string()),
        registry: "https://registry.npmjs.org".to_string(),
        config_format: Some("json".to_string()),
        force: false,
        non_interactive: false, // Interactive mode
    };

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    // Note: In non-interactive CI environments, this should still work
    // with provided defaults. Interactive prompts would only appear
    // if parameters are missing AND stdin is a TTY.
    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(
        result.is_ok(),
        "Init in interactive mode should succeed with all args provided: {:?}",
        result.err()
    );

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

    workspace.assert_config_exists();
}

// ============================================================================
// Output Format Tests
// ============================================================================

/// Test: Init with Human output format
#[tokio::test]
async fn test_init_human_output_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with Human format using SharedWriter
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = common::helpers::SharedWriter { buffer: std::sync::Arc::clone(&buffer) };
    let output = Output::new(OutputFormat::Human, Box::new(writer), false);

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with Human format");

    // Verify human-readable output
    let output_bytes = buffer.lock().unwrap().clone();
    let output_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    assert!(
        output_str.contains("Configuration initialized successfully")
            || output_str.contains("initialized"),
        "Human output should contain success message: {output_str}"
    );
    assert!(
        output_str.contains("Config file") || output_str.contains("config"),
        "Human output should mention config file"
    );
    assert!(
        output_str.contains("Strategy") || output_str.contains("independent"),
        "Human output should mention strategy"
    );
    assert!(output_str.contains("production"), "Human output should mention environments");

    workspace.assert_config_exists();
}

/// Test: Init with JsonCompact output format
#[tokio::test]
async fn test_init_json_compact_output_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with JsonCompact format
    let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer = common::helpers::SharedWriter { buffer: std::sync::Arc::clone(&buffer) };
    let output = Output::new(OutputFormat::JsonCompact, Box::new(writer), false);

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with JsonCompact format");

    // Verify compact JSON output (no newlines or extra whitespace)
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");

    // Compact JSON should not have pretty formatting
    assert!(
        !json_str.contains("\n  ") && !json_str.contains("    "),
        "Compact JSON should not have indentation"
    );

    // But should still be valid JSON
    verify_json_output(&json_str, "independent", "json");

    workspace.assert_config_exists();
}

/// Test: Init with Quiet output format
#[tokio::test]
async fn test_init_quiet_output_format() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with Quiet format
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Quiet, Box::new(buffer.clone()), false);

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with Quiet format");

    // Verify minimal quiet output
    let output_str = String::from_utf8(buffer.into_inner()).expect("Output should be valid UTF-8");

    // Quiet mode should have minimal output
    assert!(
        output_str.contains("Configuration initialized") || output_str.trim().is_empty(),
        "Quiet output should be minimal"
    );

    workspace.assert_config_exists();
}

/// Test: Init JSON output with all fields populated
#[tokio::test]
async fn test_init_json_output_complete() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = InitArgs {
        changeset_path: PathBuf::from(".custom-changesets"),
        environments: Some(vec![
            "development".to_string(),
            "staging".to_string(),
            "production".to_string(),
        ]),
        default_env: Some(vec!["staging".to_string(), "production".to_string()]),
        strategy: Some("unified".to_string()),
        registry: "https://custom-registry.example.com".to_string(),
        config_format: Some("toml".to_string()),
        force: false,
        non_interactive: true,
    };

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed");

    // Verify complete JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify all fields are present and correct
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["configFile"], "repo.config.toml");
    assert_eq!(json["data"]["configFormat"], "toml");
    assert_eq!(json["data"]["strategy"], "unified");
    assert_eq!(json["data"]["changesetPath"], ".custom-changesets");
    assert_eq!(json["data"]["registry"], "https://custom-registry.example.com");

    let environments = json["data"]["environments"].as_array().unwrap();
    assert_eq!(environments.len(), 3);

    let default_envs = json["data"]["defaultEnvironments"].as_array().unwrap();
    assert_eq!(default_envs.len(), 2);

    // Verify TOML config file was created (not JSON)
    let toml_config_path = workspace.root().join("repo.config.toml");
    assert!(toml_config_path.exists(), "TOML config file should exist");
}

/// Test: Init with config_path parameter (for future use)
#[tokio::test]
async fn test_init_with_config_path_parameter() {
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = create_init_args("independent", "json");

    // Create output with buffer for capture
    let (output, buffer) = create_shared_json_output();

    // Test with None config_path (current behavior)
    let result = execute_init(&args, &output, workspace.root(), None).await;
    assert!(result.is_ok(), "Init should succeed with None config_path");

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_json_output(&json_str, "independent", "json");

    workspace.assert_config_exists();
}
