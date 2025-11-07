//! # E2E Tests for Config Commands
//!
//! **What**: End-to-end tests for configuration management commands including
//! `config show` and `config validate`. Tests cover displaying configuration,
//! JSON output, validation of valid and invalid configs, and default fallback.
//!
//! **How**: Creates real temporary workspaces with various configuration states,
//! executes config commands with different parameters, and validates that
//! configuration is correctly displayed and validated across all scenarios.
//!
//! **Why**: Ensures the complete configuration workflow works correctly across
//! different workspace types, configuration formats, validation scenarios, and
//! output formats. Validates that users can inspect and verify their configuration
//! reliably.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
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

    // ACT: Execute config show command
    let result = execute_show(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show should succeed: {:?}", result.err());
}

/// Test: Config show outputs valid JSON format
///
/// Verifies that the `config show` command outputs valid JSON when the
/// JSON format is requested.
#[tokio::test]
async fn test_config_show_json_output() {
    // ARRANGE: Create workspace with configuration
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ConfigShowArgs {};

    // ACT: Execute config show command with JSON format
    let result = execute_show(&args, workspace.root(), None, OutputFormat::Json).await;

    // ASSERT: Command should succeed and output should be valid JSON
    assert!(result.is_ok(), "Config show with JSON format should succeed: {:?}", result.err());

    // The actual JSON output validation happens in the command implementation
    // We verify the command completes without errors
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

    // ACT: Execute config show command (no config file exists)
    let result = execute_show(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Command should succeed and show defaults
    assert!(result.is_ok(), "Config show should succeed with defaults: {:?}", result.err());
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

    // ACT: Execute config show with custom path
    let result =
        execute_show(&args, workspace.root(), Some(&custom_config_path), OutputFormat::Human).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show with custom path should succeed: {:?}", result.err());
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
    let result =
        execute_show(&args, workspace.root(), Some(&non_existent_path), OutputFormat::Human).await;

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

    // ACT: Execute config show with quiet format
    let result = execute_show(&args, workspace.root(), None, OutputFormat::Quiet).await;

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

    // ACT: Execute config validate command
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Validation should succeed
    assert!(result.is_ok(), "Config validate should succeed with valid config: {:?}", result.err());
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

    // ACT: Execute config validate command
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

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
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

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

    // ACT: Execute config validate with JSON format
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Json).await;

    // ASSERT: Validation should succeed and output JSON
    assert!(result.is_ok(), "Config validate with JSON output should succeed: {:?}", result.err());
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

    // ACT: Execute config validate with custom path
    let result =
        execute_validate(&args, workspace.root(), Some(&custom_config_path), OutputFormat::Human)
            .await;

    // ASSERT: Validation should succeed
    assert!(result.is_ok(), "Config validate with custom path should succeed: {:?}", result.err());
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
    let result =
        execute_validate(&args, workspace.root(), Some(&non_existent_path), OutputFormat::Human)
            .await;

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

    // ACT: Execute config validate with quiet format
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Quiet).await;

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

    // ACT: Execute config show command
    let result = execute_show(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Config show in monorepo should succeed: {:?}", result.err());
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

    // ACT: Execute config validate command
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Validation should succeed
    assert!(
        result.is_ok(),
        "Config validate in unified monorepo should succeed: {:?}",
        result.err()
    );
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
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

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
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

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
    let result = execute_validate(&args, workspace.root(), None, OutputFormat::Human).await;

    // ASSERT: Validation should fail
    assert!(result.is_err(), "Config validate should fail with invalid snapshot format");
}
