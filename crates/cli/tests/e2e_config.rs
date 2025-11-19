//! # E2E Tests for Config Commands
//!
//! **What**: End-to-end tests for configuration management commands including
//! `config show` and `config validate`. Tests cover displaying configuration,
//! JSON output, validation of valid and invalid configs, and default fallback.
//!
//! **How**: Creates real temporary workspaces with various configuration states,
//! executes config commands with different parameters, and validates that
//! configuration is correctly displayed and validated across all scenarios.
//! Uses Pattern B with output capture to verify command output.
//!
//! **Why**: Ensures the complete configuration workflow works correctly across
//! different workspace types, configuration formats, validation scenarios, and
//! output formats. Validates that users can inspect and verify their configuration
//! reliably with proper output verification.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use common::helpers::{create_quiet_output, create_shared_json_output, create_test_output};
use serde_json::json;
use sublime_cli_tools::cli::commands::{ConfigShowArgs, ConfigValidateArgs};
use sublime_cli_tools::commands::config::{execute_show, execute_validate};
use sublime_cli_tools::output::OutputFormat;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates an invalid configuration JSON for testing validation failures.
///
/// Returns a JSON string with invalid configuration that should fail validation.
fn create_invalid_config() -> String {
    json!({
        "changeset": {
            "path": ".changesets/",
            "available_environments": [],  // Empty list - should fail validation
            "default_environments": ["production"]
        },
        "version": {
            "strategy": "independent",
            "default_bump": "invalid_bump_type"  // Invalid bump type
        }
    })
    .to_string()
}

/// Creates a valid configuration JSON for testing.
///
/// Returns a JSON string with complete valid configuration.
fn create_valid_config() -> String {
    json!({
        "changeset": {
            "path": ".changesets/",
            "history_path": ".changesets/history/",
            "available_environments": ["development", "staging", "production"],
            "default_environments": ["production"]
        },
        "version": {
            "strategy": "independent",
            "default_bump": "patch",
            "snapshot_format": "{version}-{branch}.{short_commit}"
        },
        "dependency": {
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": false,
            "max_depth": 10,
            "fail_on_circular": true
        },
        "upgrade": {
            "auto_changeset": false,
            "changeset_bump": "patch",
            "registry": {
                "default_registry": "https://registry.npmjs.org",
                "scoped_registries": {},
                "timeout_secs": 30,
                "retry_attempts": 3
            },
            "backup": {
                "enabled": true,
                "backup_dir": ".workspace-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        },
        "changelog": {
            "enabled": true,
            "format": "keep-a-changelog",
            "include_commit_links": true,
            "repository_url": null
        },
        "audit": {
            "enabled": true,
            "min_severity": "info"
        }
    })
    .to_string()
}

/// Helper to verify JSON output structure from config show command.
///
/// Verifies that the output contains all expected configuration fields.
fn verify_config_show_json_output(json_str: &str) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");
    assert_eq!(json["success"], true, "Success should be true");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify all required configuration sections
    assert!(data.get("changeset").is_some(), "Should have changeset config");
    assert!(data.get("version").is_some(), "Should have version config");
    assert!(data.get("dependency").is_some(), "Should have dependency config");
    assert!(data.get("upgrade").is_some(), "Should have upgrade config");
    assert!(data.get("changelog").is_some(), "Should have changelog config");
    assert!(data.get("audit").is_some(), "Should have audit config");

    // Verify changeset section (note: field names are camelCase in JSON)
    let changeset = &data["changeset"];
    assert!(changeset.get("path").is_some(), "Should have changeset path");
    assert!(changeset.get("environments").is_some(), "Should have environments");
    assert!(changeset.get("defaultEnvironments").is_some(), "Should have defaultEnvironments");

    // Verify version section (note: field names are camelCase in JSON)
    let version = &data["version"];
    assert!(version.get("strategy").is_some(), "Should have strategy");
    assert!(version.get("defaultBump").is_some(), "Should have defaultBump");
    assert!(version.get("snapshotFormat").is_some(), "Should have snapshotFormat");
}

/// Helper to verify JSON output structure from config validate command.
///
/// Verifies that the validation output contains expected fields.
fn verify_config_validate_json_output(json_str: &str, expected_valid: bool) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");
    assert_eq!(json["success"], expected_valid, "Success should match validation result");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify validation result structure (note: field is "valid", not "is_valid")
    assert!(data.get("valid").is_some(), "Should have valid field");
    assert_eq!(data["valid"], expected_valid, "valid should match expected result");

    assert!(data.get("checks").is_some(), "Should have checks field");
    let checks = data["checks"].as_array().expect("checks should be array");
    assert!(!checks.is_empty(), "Should have at least one validation check");

    // Verify each check has required fields (note: no "description" field, only "name", "passed", and optional "error")
    for check in checks {
        assert!(check.get("name").is_some(), "Check should have name");
        assert!(check.get("passed").is_some(), "Check should have passed field");
        // error field is optional, only present when check failed
    }
}

// ============================================================================
// Config Show Command Tests
// ============================================================================

/// Test: Config show displays current configuration
///
/// Verifies that the `config show` command correctly displays the current
/// configuration when a valid config file exists.
#[tokio::test]
async fn test_config_show_displays_current() {
    // ARRANGE: Create workspace with custom configuration
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with captured output
    let (output, _buffer) = create_test_output(OutputFormat::Human);
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show should succeed: {:?}", result.err());
}

/// Test: Config show outputs valid JSON format
///
/// Verifies that the `config show` command outputs valid JSON when the
/// JSON format is requested and that the output contains all expected fields.
#[tokio::test]
async fn test_config_show_json_output() {
    // ARRANGE: Create workspace with configuration
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with JSON format and captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show with JSON format should succeed: {:?}", result.err());

    // Verify JSON output structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);
}

/// Test: Config show with missing config file uses defaults
///
/// Verifies that the `config show` command gracefully handles missing
/// configuration files by displaying default values.
#[tokio::test]
async fn test_config_show_missing_config_uses_defaults() {
    // ARRANGE: Create workspace WITHOUT configuration file
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show command (no config file exists) with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed and show defaults
    assert!(result.is_ok(), "Config show should succeed with defaults: {:?}", result.err());

    // Verify JSON output is valid (will have defaults)
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);

    // Verify it uses default strategy
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(
        json["data"]["version"]["strategy"], "independent",
        "Should use default independent strategy"
    );
}

/// Test: Config show with custom config path
///
/// Verifies that the `config show` command can read configuration from
/// a custom path specified via command-line argument.
#[tokio::test]
async fn test_config_show_with_custom_config_path() {
    // ARRANGE: Create workspace with config in custom location
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create config file in custom location
    let custom_config_path = workspace.root().join("custom.config.json");
    std::fs::write(&custom_config_path, create_valid_config())
        .expect("Failed to write custom config");

    let args = ConfigShowArgs {};

    // ACT: Execute config show with custom path and captured output
    let (output, buffer) = create_shared_json_output();
    let result =
        execute_show(&args, &output, workspace.root(), Some(custom_config_path.as_path())).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show with custom path should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);
}

/// Test: Config show with non-existent custom config path fails
///
/// Verifies that the `config show` command returns an error when provided
/// with a custom config path that doesn't exist.
#[tokio::test]
async fn test_config_show_custom_path_not_found() {
    // ARRANGE: Create workspace without custom config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let non_existent_path = workspace.root().join("non-existent.config.json");
    let args = ConfigShowArgs {};

    // ACT: Execute config show with non-existent path
    let output = create_quiet_output();
    let result =
        execute_show(&args, &output, workspace.root(), Some(non_existent_path.as_path())).await;

    // ASSERT: Command should fail with appropriate error
    assert!(result.is_err(), "Config show should fail with non-existent custom path");
}

/// Test: Config show with quiet output format
///
/// Verifies that the `config show` command produces minimal output
/// in quiet mode.
#[tokio::test]
async fn test_config_show_quiet_output() {
    // ARRANGE: Create workspace with configuration
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show with quiet format and captured output
    let (output, _buffer) = create_test_output(OutputFormat::Quiet);
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show with quiet format should succeed: {:?}", result.err());
}

// ============================================================================
// Config Validate Command Tests
// ============================================================================

/// Test: Config validate succeeds with valid configuration
///
/// Verifies that the `config validate` command passes when provided with
/// a complete and valid configuration file.
#[tokio::test]
async fn test_config_validate_valid_config() {
    // ARRANGE: Create workspace with valid configuration
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create valid config file
    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, create_valid_config()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed
    assert!(result.is_ok(), "Config validate should succeed with valid config: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}

/// Test: Config validate fails with invalid configuration
///
/// Verifies that the `config validate` command correctly identifies and
/// reports validation errors in invalid configuration files.
#[tokio::test]
async fn test_config_validate_invalid_config() {
    // ARRANGE: Create workspace with invalid configuration
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create invalid config file
    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, create_invalid_config()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command with captured output
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with invalid config");
}

/// Test: Config validate fails with missing config file
///
/// Verifies that the `config validate` command returns an appropriate error
/// when no configuration file exists.
#[tokio::test]
async fn test_config_validate_missing_file() {
    // ARRANGE: Create workspace WITHOUT configuration file
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command (no config exists)
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should fail with missing file error
    assert!(result.is_err(), "Config validate should fail when config file is missing");
}

/// Test: Config validate with JSON output format
///
/// Verifies that validation results are correctly formatted as JSON
/// when JSON output is requested.
#[tokio::test]
async fn test_config_validate_json_output() {
    // ARRANGE: Create workspace with valid configuration
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create valid config file
    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, create_valid_config()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate with JSON format and captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed and output JSON
    assert!(result.is_ok(), "Config validate with JSON output should succeed: {:?}", result.err());

    // Verify JSON output structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}

/// Test: Config validate with custom config path
///
/// Verifies that the `config validate` command can validate configuration
/// from a custom path.
#[tokio::test]
async fn test_config_validate_with_custom_path() {
    // ARRANGE: Create workspace with config in custom location
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create config file in custom location
    let custom_config_path = workspace.root().join("custom.config.json");
    std::fs::write(&custom_config_path, create_valid_config())
        .expect("Failed to write custom config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate with custom path and captured output
    let (output, buffer) = create_shared_json_output();
    let result =
        execute_validate(&args, &output, workspace.root(), Some(&custom_config_path)).await;

    // ASSERT: Validation should succeed
    assert!(result.is_ok(), "Config validate with custom path should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}

/// Test: Config validate with custom path not found
///
/// Verifies that validation fails appropriately when the custom config
/// path doesn't exist.
#[tokio::test]
async fn test_config_validate_custom_path_not_found() {
    // ARRANGE: Create workspace without custom config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let non_existent_path = workspace.root().join("non-existent.config.json");
    let args = ConfigValidateArgs {};

    // ACT: Execute config validate with non-existent path
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), Some(&non_existent_path)).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with non-existent custom path");
}

/// Test: Config validate with quiet output format
///
/// Verifies that validation results are correctly formatted in quiet mode,
/// showing only "valid" or "invalid".
#[tokio::test]
async fn test_config_validate_quiet_output() {
    // ARRANGE: Create workspace with valid configuration
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create valid config file
    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, create_valid_config()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate with quiet format and captured output
    let (output, _buffer) = create_test_output(OutputFormat::Quiet);
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed
    assert!(result.is_ok(), "Config validate with quiet format should succeed: {:?}", result.err());
}

// ============================================================================
// Monorepo Configuration Tests
// ============================================================================

/// Test: Config show in monorepo with independent strategy
///
/// Verifies that configuration is correctly displayed in a monorepo
/// workspace with independent versioning strategy.
#[tokio::test]
async fn test_config_show_monorepo_independent() {
    // ARRANGE: Create monorepo workspace with independent strategy
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show in monorepo should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);

    // Verify strategy is independent
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(
        json["data"]["version"]["strategy"], "independent",
        "Strategy should be independent"
    );
}

/// Test: Config validate in monorepo with unified strategy
///
/// Verifies that validation works correctly in a monorepo with
/// unified versioning strategy.
#[tokio::test]
async fn test_config_validate_monorepo_unified() {
    // ARRANGE: Create monorepo workspace with unified config
    let workspace = WorkspaceFixture::monorepo_unified().with_git().with_commits(1).finalize();

    // Create unified strategy config
    let config = json!({
        "changeset": {
            "path": ".changesets/",
            "history_path": ".changesets/history/",
            "available_environments": ["production"],
            "default_environments": ["production"]
        },
        "version": {
            "strategy": "unified",  // Unified strategy
            "default_bump": "patch",
            "snapshot_format": "{version}-{branch}.{short_commit}"
        },
        "dependency": {
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": false,
            "max_depth": 10,
            "fail_on_circular": true
        },
        "upgrade": {
            "auto_changeset": false,
            "changeset_bump": "patch",
            "registry": {
                "default_registry": "https://registry.npmjs.org",
                "scoped_registries": {},
                "timeout_secs": 30,
                "retry_attempts": 3
            },
            "backup": {
                "enabled": true,
                "backup_dir": ".workspace-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        },
        "changelog": {
            "enabled": true,
            "format": "keep-a-changelog",
            "include_commit_links": true,
            "repository_url": null
        },
        "audit": {
            "enabled": true,
            "min_severity": "info"
        }
    });

    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, config.to_string()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed
    assert!(
        result.is_ok(),
        "Config validate in unified monorepo should succeed: {:?}",
        result.err()
    );

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

/// Test: Config validate with conflicting environment settings
///
/// Verifies that validation fails when default environments are not
/// present in the available environments list.
#[tokio::test]
async fn test_config_validate_conflicting_environments() {
    // ARRANGE: Create workspace with conflicting environment config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let config = json!({
        "changeset": {
            "path": ".changesets/",
            "history_path": ".changesets/history/",
            "available_environments": ["development", "staging"],
            "default_environments": ["production"]  // Not in available list!
        },
        "version": {
            "strategy": "independent",
            "default_bump": "patch",
            "snapshot_format": "{version}-{branch}.{short_commit}"
        },
        "dependency": {
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": false,
            "max_depth": 10,
            "fail_on_circular": true
        },
        "upgrade": {
            "auto_changeset": false,
            "changeset_bump": "patch",
            "registry": {
                "default_registry": "https://registry.npmjs.org",
                "scoped_registries": {},
                "timeout_secs": 30,
                "retry_attempts": 3
            },
            "backup": {
                "enabled": true,
                "backup_dir": ".workspace-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        },
        "changelog": {
            "enabled": true,
            "format": "keep-a-changelog",
            "include_commit_links": true,
            "repository_url": null
        },
        "audit": {
            "enabled": true,
            "min_severity": "info"
        }
    });

    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, config.to_string()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with conflicting environments");
}

/// Test: Config validate with invalid registry URL
///
/// Verifies that validation fails when registry URL doesn't start with
/// http:// or https://.
#[tokio::test]
async fn test_config_validate_invalid_registry_url() {
    // ARRANGE: Create workspace with invalid registry URL
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let config = json!({
        "changeset": {
            "path": ".changesets/",
            "history_path": ".changesets/history/",
            "available_environments": ["production"],
            "default_environments": ["production"]
        },
        "version": {
            "strategy": "independent",
            "default_bump": "patch",
            "snapshot_format": "{version}-{branch}.{short_commit}"
        },
        "dependency": {
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": false,
            "max_depth": 10,
            "fail_on_circular": true
        },
        "upgrade": {
            "auto_changeset": false,
            "changeset_bump": "patch",
            "registry": {
                "default_registry": "invalid-url-without-protocol",  // Invalid!
                "scoped_registries": {},
                "timeout_secs": 30,
                "retry_attempts": 3
            },
            "backup": {
                "enabled": true,
                "backup_dir": ".workspace-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        },
        "changelog": {
            "enabled": true,
            "format": "keep-a-changelog",
            "include_commit_links": true,
            "repository_url": null
        },
        "audit": {
            "enabled": true,
            "min_severity": "info"
        }
    });

    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, config.to_string()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with invalid registry URL");
}

/// Test: Config validate with invalid snapshot format
///
/// Verifies that validation fails when snapshot format doesn't contain
/// the required {version} placeholder.
#[tokio::test]
async fn test_config_validate_invalid_snapshot_format() {
    // ARRANGE: Create workspace with invalid snapshot format
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    let config = json!({
        "changeset": {
            "path": ".changesets/",
            "history_path": ".changesets/history/",
            "available_environments": ["production"],
            "default_environments": ["production"]
        },
        "version": {
            "strategy": "independent",
            "default_bump": "patch",
            "snapshot_format": "{branch}.{short_commit}"  // Missing {version}!
        },
        "dependency": {
            "propagation_bump": "patch",
            "propagate_dependencies": true,
            "propagate_dev_dependencies": false,
            "propagate_peer_dependencies": false,
            "max_depth": 10,
            "fail_on_circular": true
        },
        "upgrade": {
            "auto_changeset": false,
            "changeset_bump": "patch",
            "registry": {
                "default_registry": "https://registry.npmjs.org",
                "scoped_registries": {},
                "timeout_secs": 30,
                "retry_attempts": 3
            },
            "backup": {
                "enabled": true,
                "backup_dir": ".workspace-backups",
                "keep_after_success": false,
                "max_backups": 5
            }
        },
        "changelog": {
            "enabled": true,
            "format": "keep-a-changelog",
            "include_commit_links": true,
            "repository_url": null
        },
        "audit": {
            "enabled": true,
            "min_severity": "info"
        }
    });

    let config_path = workspace.root().join("repo.config.json");
    std::fs::write(&config_path, config.to_string()).expect("Failed to write config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command
    let output = create_quiet_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with invalid snapshot format");
}

// ============================================================================
// Config Format Support Tests - Gap Coverage
// ============================================================================

/// Test: Config show with TOML format configuration
///
/// Verifies that the `config show` command correctly reads and displays
/// configuration stored in TOML format.
#[tokio::test]
async fn test_config_show_toml_format() {
    // ARRANGE: Create workspace with TOML config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create TOML config file
    let toml_config = r#"
[changeset]
path = ".changesets/"
history_path = ".changesets/history/"
available_environments = ["production", "staging"]
default_environments = ["production"]

[version]
strategy = "independent"
default_bump = "patch"
snapshot_format = "{version}-{branch}.{short_commit}"

[dependency]
propagation_bump = "patch"
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = false
max_depth = 10
fail_on_circular = true

[upgrade]
auto_changeset = false
changeset_bump = "patch"

[upgrade.registry]
default_registry = "https://registry.npmjs.org"
timeout_secs = 30
retry_attempts = 3

[upgrade.registry.scoped_registries]

[upgrade.backup]
enabled = true
backup_dir = ".workspace-backups"
keep_after_success = false
max_backups = 5

[changelog]
enabled = true
format = "keep-a-changelog"
include_commit_links = false

[audit]
enabled = true
min_severity = "info"
"#;

    let config_path = workspace.root().join("repo.config.toml");
    std::fs::write(&config_path, toml_config).expect("Failed to write TOML config");

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed with TOML format
    assert!(result.is_ok(), "Config show should succeed with TOML format: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);
}

/// Test: Config show with YAML format configuration
///
/// Verifies that the `config show` command correctly reads and displays
/// configuration stored in YAML format.
#[tokio::test]
async fn test_config_show_yaml_format() {
    // ARRANGE: Create workspace with YAML config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create YAML config file
    let yaml_config = r#"
changeset:
  path: .changesets/
  history_path: .changesets/history/
  available_environments:
    - production
    - staging
  default_environments:
    - production

version:
  strategy: independent
  default_bump: patch
  snapshot_format: "{version}-{branch}.{short_commit}"

dependency:
  propagation_bump: patch
  propagate_dependencies: true
  propagate_dev_dependencies: false
  propagate_peer_dependencies: false
  max_depth: 10
  fail_on_circular: true

upgrade:
  auto_changeset: false
  changeset_bump: patch
  registry:
    default_registry: https://registry.npmjs.org
    scoped_registries: {}
    timeout_secs: 30
    retry_attempts: 3
  backup:
    enabled: true
    backup_dir: .workspace-backups
    keep_after_success: false
    max_backups: 5

changelog:
  enabled: true
  format: keep-a-changelog
  include_commit_links: true
  repository_url: null

audit:
  enabled: true
  min_severity: info
"#;

    let config_path = workspace.root().join("repo.config.yaml");
    std::fs::write(&config_path, yaml_config).expect("Failed to write YAML config");

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_show(&args, &output, workspace.root(), None).await;

    // ASSERT: Command should succeed with YAML format
    assert!(result.is_ok(), "Config show should succeed with YAML format: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_show_json_output(&json_str);
}

/// Test: Config validate with TOML format configuration
///
/// Verifies that the `config validate` command correctly validates
/// configuration stored in TOML format.
#[tokio::test]
async fn test_config_validate_toml_format() {
    // ARRANGE: Create workspace with TOML config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create valid TOML config file
    let toml_config = r#"
[changeset]
path = ".changesets/"
history_path = ".changesets/history/"
available_environments = ["production", "staging"]
default_environments = ["production"]

[version]
strategy = "independent"
default_bump = "patch"
snapshot_format = "{version}-{branch}.{short_commit}"

[dependency]
propagation_bump = "patch"
propagate_dependencies = true
propagate_dev_dependencies = false
propagate_peer_dependencies = false
max_depth = 10
fail_on_circular = true

[upgrade]
auto_changeset = false
changeset_bump = "patch"

[upgrade.registry]
default_registry = "https://registry.npmjs.org"
timeout_secs = 30
retry_attempts = 3

[upgrade.registry.scoped_registries]

[upgrade.backup]
enabled = true
backup_dir = ".workspace-backups"
keep_after_success = false
max_backups = 5

[changelog]
enabled = true
format = "keep-a-changelog"
include_commit_links = false

[audit]
enabled = true
min_severity = "info"
"#;

    let config_path = workspace.root().join("repo.config.toml");
    std::fs::write(&config_path, toml_config).expect("Failed to write TOML config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed with TOML format
    assert!(result.is_ok(), "Config validate should succeed with TOML format: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}

/// Test: Config validate with YAML format configuration
///
/// Verifies that the `config validate` command correctly validates
/// configuration stored in YAML format.
#[tokio::test]
async fn test_config_validate_yaml_format() {
    // ARRANGE: Create workspace with YAML config
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Create valid YAML config file
    let yaml_config = r#"
changeset:
  path: .changesets/
  history_path: .changesets/history/
  available_environments:
    - production
    - staging
  default_environments:
    - production

version:
  strategy: independent
  default_bump: patch
  snapshot_format: "{version}-{branch}.{short_commit}"

dependency:
  propagation_bump: patch
  propagate_dependencies: true
  propagate_dev_dependencies: false
  propagate_peer_dependencies: false
  max_depth: 10
  fail_on_circular: true

upgrade:
  auto_changeset: false
  changeset_bump: patch
  registry:
    default_registry: https://registry.npmjs.org
    scoped_registries: {}
    timeout_secs: 30
    retry_attempts: 3
  backup:
    enabled: true
    backup_dir: .workspace-backups
    keep_after_success: false
    max_backups: 5

changelog:
  enabled: true
  format: keep-a-changelog
  include_commit_links: true
  repository_url: null

audit:
  enabled: true
  min_severity: info
"#;

    let config_path = workspace.root().join("repo.config.yaml");
    std::fs::write(&config_path, yaml_config).expect("Failed to write YAML config");

    let args = ConfigValidateArgs {};

    // ACT: Execute config validate command with captured output
    let (output, buffer) = create_shared_json_output();
    let result = execute_validate(&args, &output, workspace.root(), None).await;

    // ASSERT: Validation should succeed with YAML format
    assert!(result.is_ok(), "Config validate should succeed with YAML format: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_config_validate_json_output(&json_str, true);
}
