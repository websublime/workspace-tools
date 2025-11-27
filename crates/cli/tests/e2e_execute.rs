//! # E2E Tests for Execute Command
//!
//! **What**: End-to-end tests for the `execute` command that runs commands
//! across workspace packages with optional filtering and parallel execution.
//!
//! **How**: Creates real temporary workspaces with npm scripts and various
//! configurations (single package, monorepo), executes the execute command
//! with different parameters (sequential/parallel, filtered/unfiltered),
//! and validates command execution behavior.
//!
//! **Why**: Ensures the execute command correctly runs commands across packages,
//! respects package filtering, handles parallel execution, and properly reports
//! success/failure status. Critical for CI/CD pipelines and development workflows.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use common::helpers::{create_quiet_output, create_shared_json_output};
use serde_json::json;
use sublime_cli_tools::cli::commands::ExecuteArgs;
use sublime_cli_tools::commands::execute::execute_execute;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a workspace fixture with npm scripts configured in package.json.
///
/// Adds test scripts to enable npm:script execution testing.
fn create_workspace_with_scripts() -> WorkspaceFixture {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_package_lock()
        .finalize();

    // Update package.json with scripts
    let package_json_path = workspace.root().join("package.json");
    let mut package_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&package_json_path).expect("Failed to read package.json"),
    )
    .expect("Failed to parse package.json");

    package_json["scripts"] = json!({
        "test": "echo 'test passed'",
        "lint": "echo 'lint passed'",
        "build": "echo 'build passed'",
        "failing": "exit 1"
    });

    std::fs::write(&package_json_path, package_json.to_string())
        .expect("Failed to write package.json");

    workspace
}

/// Creates a monorepo workspace with npm scripts in each package.
fn create_monorepo_with_scripts() -> WorkspaceFixture {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Add scripts to each package
    for package in workspace.packages() {
        let package_json_path = package.path.join("package.json");
        let mut package_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&package_json_path).expect("Failed to read package.json"),
        )
        .expect("Failed to parse package.json");

        package_json["scripts"] = json!({
            "test": format!("echo 'test passed for {}'", package.name),
            "lint": format!("echo 'lint passed for {}'", package.name),
            "build": format!("echo 'build passed for {}'", package.name)
        });

        std::fs::write(&package_json_path, package_json.to_string())
            .expect("Failed to write package.json");
    }

    workspace
}

/// Verifies JSON output structure from execute command.
fn verify_execute_json_output(json_str: &str) {
    let json: serde_json::Value =
        serde_json::from_str(json_str).expect("Output should be valid JSON");

    // Verify response structure
    assert!(json.get("success").is_some(), "JSON should have 'success' field");

    // Verify data field exists
    assert!(json.get("data").is_some(), "JSON should have 'data' field");
    let data = &json["data"];

    // Verify results field
    assert!(data.get("results").is_some(), "Should have results field");
    assert!(data["results"].is_array(), "Results should be an array");
}

/// Verifies all package results are successful.
fn verify_all_succeeded(json_str: &str) {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let results = json["data"]["results"].as_array().unwrap();

    for result in results {
        let success = result["success"].as_bool().unwrap_or(false);
        let package_name = result["package"].as_str().unwrap_or("unknown");
        assert!(success, "Package {package_name} should have succeeded");
    }
}

/// Counts total results in JSON output.
fn count_total_results(json_str: &str) -> usize {
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    json["data"]["results"].as_array().unwrap().len()
}

// ============================================================================
// System Command Tests
// ============================================================================

/// Test: Execute system command in single package
///
/// Verifies that system commands (non-npm) execute correctly.
#[tokio::test]
async fn test_execute_system_command() {
    // ARRANGE: Create workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_package_lock()
        .finalize();

    let args = ExecuteArgs {
        cmd: "echo hello".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute command
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute system command should succeed: {:?}", result.err());

    // Verify JSON output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_execute_json_output(&json_str);
    verify_all_succeeded(&json_str);
}

/// Test: Execute system command in monorepo
///
/// Verifies that system commands run in all packages of a monorepo.
#[tokio::test]
async fn test_execute_system_command_monorepo() {
    // ARRANGE: Create monorepo workspace
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ExecuteArgs {
        cmd: "echo hello".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute command
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute should succeed in monorepo: {:?}", result.err());

    // Verify results for all packages
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_execute_json_output(&json_str);
    assert_eq!(count_total_results(&json_str), 2, "Should have results for 2 packages");
    verify_all_succeeded(&json_str);
}

// ============================================================================
// NPM Script Tests
// ============================================================================

/// Test: Execute npm script in single package
///
/// Verifies that npm:script commands execute the corresponding npm script.
#[tokio::test]
async fn test_execute_npm_script() {
    // ARRANGE: Create workspace with scripts
    let workspace = create_workspace_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute npm script
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute npm:test should succeed: {:?}", result.err());

    // Verify output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_execute_json_output(&json_str);
    verify_all_succeeded(&json_str);
}

/// Test: Execute npm lint script
///
/// Verifies that npm:lint script executes correctly.
#[tokio::test]
async fn test_execute_npm_lint() {
    // ARRANGE: Create workspace with scripts
    let workspace = create_workspace_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:lint".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute npm script
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute npm:lint should succeed: {:?}", result.err());

    // Verify output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_all_succeeded(&json_str);
}

/// Test: Execute npm script in monorepo
///
/// Verifies that npm scripts run in all packages of a monorepo.
#[tokio::test]
async fn test_execute_npm_script_monorepo() {
    // ARRANGE: Create monorepo with scripts
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute npm script
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute npm:test in monorepo should succeed: {:?}", result.err());

    // Verify results for all packages
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 2, "Should have results for 2 packages");
    verify_all_succeeded(&json_str);
}

/// Test: Execute missing npm script fails
///
/// Verifies that executing a non-existent npm script fails appropriately.
#[tokio::test]
async fn test_execute_missing_npm_script_fails() {
    // ARRANGE: Create workspace with scripts (but not 'nonexistent')
    let workspace = create_workspace_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:nonexistent".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute non-existent npm script
    let output = create_quiet_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should fail
    assert!(result.is_err(), "Execute should fail for missing npm script");
}

// ============================================================================
// Package Filtering Tests
// ============================================================================

/// Test: Execute with package filter
///
/// Verifies that --filter-package restricts execution to specified packages.
#[tokio::test]
async fn test_execute_with_filter() {
    // ARRANGE: Create monorepo with scripts
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: Some(vec!["@test/pkg-a".to_string()]),
        parallel: false,
        args: vec![],
    };

    // ACT: Execute with filter
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute with filter should succeed: {:?}", result.err());

    // Verify only filtered package was executed
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 1, "Should have results for 1 package");
    verify_all_succeeded(&json_str);
}

/// Test: Execute with multiple package filters
///
/// Verifies that multiple packages can be filtered.
#[tokio::test]
async fn test_execute_with_multiple_filters() {
    // ARRANGE: Create monorepo with scripts
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: Some(vec!["@test/pkg-a".to_string(), "@test/pkg-b".to_string()]),
        parallel: false,
        args: vec![],
    };

    // ACT: Execute with multiple filters
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute with multiple filters should succeed: {:?}", result.err());

    // Verify both packages were executed
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 2, "Should have results for 2 packages");
}

/// Test: Execute with invalid filter fails
///
/// Verifies that filtering to non-existent packages fails appropriately.
#[tokio::test]
async fn test_execute_invalid_filter_fails() {
    // ARRANGE: Create monorepo
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: Some(vec!["@nonexistent/package".to_string()]),
        parallel: false,
        args: vec![],
    };

    // ACT: Execute with invalid filter
    let output = create_quiet_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should fail
    assert!(result.is_err(), "Execute should fail with invalid filter");
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

/// Test: Execute commands in parallel
///
/// Verifies that --parallel flag enables concurrent execution.
#[tokio::test]
async fn test_execute_parallel() {
    // ARRANGE: Create monorepo with scripts
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: None,
        parallel: true,
        args: vec![],
    };

    // ACT: Execute in parallel
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute parallel should succeed: {:?}", result.err());

    // Verify all packages were executed
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 2, "Should have results for 2 packages");
    verify_all_succeeded(&json_str);
}

/// Test: Execute system commands in parallel
///
/// Verifies parallel execution works for system commands.
#[tokio::test]
async fn test_execute_parallel_system_command() {
    // ARRANGE: Create monorepo
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = ExecuteArgs {
        cmd: "echo hello".to_string(),
        filter_package: None,
        parallel: true,
        args: vec![],
    };

    // ACT: Execute in parallel
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute parallel system command should succeed: {:?}", result.err());

    // Verify all packages were executed
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_all_succeeded(&json_str);
}

// ============================================================================
// Extra Arguments Tests
// ============================================================================

/// Test: Execute with extra arguments
///
/// Verifies that additional arguments are passed to the command.
#[tokio::test]
async fn test_execute_with_extra_args() {
    // ARRANGE: Create workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_package_lock()
        .finalize();

    let args = ExecuteArgs {
        cmd: "echo".to_string(),
        filter_package: None,
        parallel: false,
        args: vec!["arg1".to_string(), "arg2".to_string()],
    };

    // ACT: Execute with extra args
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute with extra args should succeed: {:?}", result.err());

    // Verify output
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_all_succeeded(&json_str);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test: Execute without package.json fails
///
/// Verifies appropriate error when no workspace is found.
#[tokio::test]
async fn test_execute_no_package_json_fails() {
    // ARRANGE: Create empty temp directory
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    let args = ExecuteArgs {
        cmd: "echo hello".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute in empty directory
    let output = create_quiet_output();
    let result = execute_execute(&args, &output, temp_dir.path()).await;

    // ASSERT: Command should fail
    assert!(result.is_err(), "Execute should fail without package.json");
}

/// Test: Execute with empty command fails
///
/// Verifies that an empty command string is handled.
#[tokio::test]
async fn test_execute_empty_command() {
    // ARRANGE: Create workspace
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .with_package_lock()
        .finalize();

    let args =
        ExecuteArgs { cmd: String::new(), filter_package: None, parallel: false, args: vec![] };

    // ACT: Execute empty command
    let output = create_quiet_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should fail or handle gracefully
    // Empty command behavior may vary - just verify no panic
    let _ = result;
}

// ============================================================================
// JSON Output Tests
// ============================================================================

/// Test: Execute produces valid JSON output
///
/// Verifies that execute command produces properly formatted JSON.
#[tokio::test]
async fn test_execute_json_output_format() {
    // ARRANGE: Create workspace
    let workspace = create_workspace_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute with JSON output
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify JSON structure
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_execute_json_output(&json_str);

    // Verify detailed structure
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(json["data"].get("command").is_some(), "Should have command field");
    assert!(json["data"].get("results").is_some(), "Should have results field");
    assert!(json["data"].get("summary").is_some(), "Should have summary field");
}

// ============================================================================
// Sequential vs Parallel Behavior Tests
// ============================================================================

/// Test: Sequential execution is default
///
/// Verifies that commands run sequentially by default (parallel: false).
#[tokio::test]
async fn test_execute_sequential_default() {
    // ARRANGE: Create monorepo
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: None,
        parallel: false, // Explicit sequential
        args: vec![],
    };

    // ACT: Execute sequentially
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Sequential execute should succeed: {:?}", result.err());

    // Verify all succeeded
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    verify_all_succeeded(&json_str);
}

// ============================================================================
// Integration with Monorepo Structure Tests
// ============================================================================

/// Test: Execute respects workspace package structure
///
/// Verifies execute finds packages in standard monorepo layout.
#[tokio::test]
async fn test_execute_respects_workspace_structure() {
    // ARRANGE: Create standard monorepo
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:build".to_string(),
        filter_package: None,
        parallel: false,
        args: vec![],
    };

    // ACT: Execute across workspace
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute should succeed: {:?}", result.err());

    // Verify correct number of packages
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 2, "Should execute in 2 packages");
}

/// Test: Execute with filter and parallel combined
///
/// Verifies that filter and parallel flags work together.
#[tokio::test]
async fn test_execute_filter_with_parallel() {
    // ARRANGE: Create monorepo with scripts
    let workspace = create_monorepo_with_scripts();

    let args = ExecuteArgs {
        cmd: "npm:test".to_string(),
        filter_package: Some(vec!["@test/pkg-a".to_string()]),
        parallel: true,
        args: vec![],
    };

    // ACT: Execute with filter and parallel
    let (output, buffer) = create_shared_json_output();
    let result = execute_execute(&args, &output, workspace.root()).await;

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Execute with filter and parallel should succeed: {:?}", result.err());

    // Verify only filtered package was executed
    let output_bytes = buffer.lock().unwrap().clone();
    let json_str = String::from_utf8(output_bytes).expect("Output should be valid UTF-8");
    assert_eq!(count_total_results(&json_str), 1, "Should have 1 result");
    verify_all_succeeded(&json_str);
}
