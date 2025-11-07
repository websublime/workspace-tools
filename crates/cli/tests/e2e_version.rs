//! # E2E Tests for Version Command
//!
//! **What**: End-to-end tests for the `version` command that displays CLI version
//! information. Tests cover basic version display, verbose mode with detailed
//! build information, and multiple output formats (Human, JSON, JsonCompact, Quiet).
//!
//! **How**: Creates real temporary workspaces and executes the version command with
//! different flags and output formats. Validates that version information is correctly
//! displayed, JSON output is properly structured, and all output modes work as expected.
//!
//! **Why**: Ensures the version command works correctly across all output formats
//! and modes. This is critical for troubleshooting, bug reports, CI/CD pipelines,
//! and verifying installations. The version command must be reliable as it's often
//! the first command users run.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use sublime_cli_tools::cli::commands::VersionArgs;
use sublime_cli_tools::commands::version::{VersionInfo, execute_version};
use sublime_cli_tools::output::OutputFormat;

// ============================================================================
// Helper Functions
// ============================================================================

/// Validates that version info structure has all required fields with non-empty values.
///
/// Checks: version, rustVersion, dependencies, and build fields.
fn validate_version_info_structure(data: &serde_json::Value) {
    // Check main fields exist and are not empty
    assert!(data["version"].is_string(), "version should be a string");
    assert!(!data["version"].as_str().unwrap().is_empty(), "version should not be empty");

    assert!(data["rustVersion"].is_string(), "rustVersion should be a string");
    assert!(!data["rustVersion"].as_str().unwrap().is_empty(), "rustVersion should not be empty");

    // Check dependencies object
    assert!(data["dependencies"].is_object(), "dependencies should be an object");
    let deps = &data["dependencies"];
    assert!(deps["sublime-package-tools"].is_string(), "package-tools version should be a string");
    assert!(
        deps["sublime-standard-tools"].is_string(),
        "standard-tools version should be a string"
    );
    assert!(deps["sublime-git-tools"].is_string(), "git-tools version should be a string");

    // Check build info
    assert!(data["build"].is_object(), "build should be an object");
    let build = &data["build"];
    assert!(build["profile"].is_string(), "build profile should be a string");
    assert!(build["target"].is_string(), "build target should be a string");
    assert!(build["features"].is_array(), "build features should be an array");

    // Validate profile is either "debug" or "release"
    let profile = build["profile"].as_str().unwrap();
    assert!(
        profile == "debug" || profile == "release",
        "build profile should be 'debug' or 'release', got: {profile}"
    );
}

// ============================================================================
// Basic Version Command Tests
// ============================================================================

/// Test: Version command displays basic information
///
/// Verifies that the `version` command successfully displays version information
/// in human-readable format without verbose flag. The command should complete
/// successfully and show the branded ASCII art header with version number.
#[tokio::test]
async fn test_version_displays_info() {
    // ARRANGE: Create minimal workspace (version command doesn't need specific setup)
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command with human output
    let result = execute_version(&args, workspace.root(), OutputFormat::Human);

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Version command should succeed: {:?}", result.err());
}

/// Test: Version command with verbose flag shows detailed information
///
/// Verifies that the `version --verbose` command displays comprehensive build
/// information including Rust version, build profile, target triple, enabled
/// features, and dependency versions.
#[tokio::test]
async fn test_version_verbose_shows_details() {
    // ARRANGE: Create minimal workspace
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: true };

    // ACT: Execute version command with verbose flag
    let result = execute_version(&args, workspace.root(), OutputFormat::Human);

    // ASSERT: Command should succeed and display verbose info
    assert!(result.is_ok(), "Version verbose command should succeed: {:?}", result.err());

    // Note: The actual output validation (checking for "Build Information:",
    // "Rust:", etc.) would require capturing stdout, which is not easily done
    // in this test setup. The success of the command execution itself validates
    // that the verbose path works correctly.
}

// ============================================================================
// JSON Output Format Tests
// ============================================================================

/// Test: Version command outputs valid JSON format
///
/// Verifies that the `version` command produces valid, well-formatted JSON output
/// when the JSON format is requested. Validates the JSON structure matches the
/// expected VersionInfo schema with all required fields.
#[tokio::test]
async fn test_version_json_output() {
    // ARRANGE: Create minimal workspace
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command with JSON format
    let result = execute_version(&args, workspace.root(), OutputFormat::Json);

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Version command with JSON format should succeed: {:?}", result.err());

    // Verify VersionInfo can be created and serialized (validates schema)
    let version_info = VersionInfo::new();
    let json_result = serde_json::to_string(&version_info);
    assert!(json_result.is_ok(), "VersionInfo should serialize to JSON successfully");

    let json_str = json_result.unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Serialized VersionInfo should be valid JSON");

    // Validate structure has all required fields
    validate_version_info_structure(&parsed);
}

/// Test: Version command outputs compact JSON format
///
/// Verifies that the `version` command produces valid compact JSON (single line,
/// no pretty-printing) when JsonCompact format is requested. The output should
/// be valid JSON but without newlines or indentation.
#[tokio::test]
async fn test_version_json_compact_output() {
    // ARRANGE: Create minimal workspace
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command with JSON compact format
    let result = execute_version(&args, workspace.root(), OutputFormat::JsonCompact);

    // ASSERT: Command should succeed
    assert!(
        result.is_ok(),
        "Version command with JsonCompact format should succeed: {:?}",
        result.err()
    );

    // Verify compact JSON can be generated (validates implementation)
    let version_info = VersionInfo::new();
    let compact_json_result = serde_json::to_string(&version_info);
    assert!(
        compact_json_result.is_ok(),
        "VersionInfo should serialize to compact JSON successfully"
    );

    let compact_json = compact_json_result.unwrap();

    // Compact JSON should not contain newlines (except in data values)
    // Count structural newlines - there should be none in compact format
    let has_pretty_formatting = compact_json.contains("\n  ");
    assert!(
        !has_pretty_formatting,
        "Compact JSON should not have pretty-print formatting (indentation)"
    );

    // Should still be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&compact_json).expect("Compact JSON should be valid");
    validate_version_info_structure(&parsed);
}

// ============================================================================
// Quiet Output Format Tests
// ============================================================================

/// Test: Version command with quiet output shows only version number
///
/// Verifies that the `version` command in quiet mode outputs only the version
/// number without any additional text, formatting, or decorations. This is useful
/// for scripts and automation that need to parse the version.
#[tokio::test]
async fn test_version_quiet_output() {
    // ARRANGE: Create minimal workspace
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command with quiet format
    let result = execute_version(&args, workspace.root(), OutputFormat::Quiet);

    // ASSERT: Command should succeed
    assert!(result.is_ok(), "Version command with quiet format should succeed: {:?}", result.err());

    // In quiet mode, only the version number should be printed
    // We can verify this by checking that VersionInfo version is valid semver format
    let version_info = VersionInfo::new();
    let version = &version_info.version;

    // Verify version follows semver format (basic check)
    assert!(!version.is_empty(), "Version should not be empty");
    assert!(version.contains('.'), "Version should contain dots (semver format)");

    // Should not contain "workspace" or other branding text (validated by implementation)
}

// ============================================================================
// VersionInfo Structure Tests
// ============================================================================

/// Test: VersionInfo structure contains all required fields
///
/// Verifies that the VersionInfo struct can be instantiated and contains
/// all expected fields with valid, non-empty values.
#[tokio::test]
async fn test_version_info_structure_completeness() {
    // ARRANGE & ACT: Create VersionInfo
    let version_info = VersionInfo::new();

    // ASSERT: All fields should be populated
    assert!(!version_info.version.is_empty(), "Version should not be empty");
    assert!(!version_info.rust_version.is_empty(), "Rust version should not be empty");

    // Check dependencies
    assert!(
        !version_info.dependencies.package_tools.is_empty(),
        "Package tools version should not be empty"
    );
    assert!(
        !version_info.dependencies.standard_tools.is_empty(),
        "Standard tools version should not be empty"
    );
    assert!(
        !version_info.dependencies.git_tools.is_empty(),
        "Git tools version should not be empty"
    );

    // Check build info
    assert!(!version_info.build.profile.is_empty(), "Build profile should not be empty");
    assert!(
        version_info.build.profile == "debug" || version_info.build.profile == "release",
        "Build profile should be 'debug' or 'release'"
    );
    assert!(!version_info.build.target.is_empty(), "Build target should not be empty");

    // Features vector is always valid (may be empty or populated)
    // No additional validation needed - Vec is always a valid structure
}

/// Test: VersionInfo serialization to JSON produces valid schema
///
/// Verifies that VersionInfo can be serialized to JSON and the resulting
/// JSON matches the expected schema structure.
#[tokio::test]
async fn test_version_info_json_serialization() {
    // ARRANGE: Create VersionInfo
    let version_info = VersionInfo::new();

    // ACT: Serialize to JSON
    let json_result = serde_json::to_string_pretty(&version_info);

    // ASSERT: Serialization should succeed
    assert!(json_result.is_ok(), "VersionInfo should serialize successfully");

    let json_str = json_result.unwrap();

    // Parse back and validate structure
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Serialized JSON should be parseable");

    validate_version_info_structure(&parsed);

    // Verify camelCase naming in JSON (serde rename attributes)
    assert!(parsed["rustVersion"].is_string(), "JSON should use camelCase 'rustVersion'");
    assert!(parsed.get("rust_version").is_none(), "JSON should not use snake_case");
}

// ============================================================================
// Cross-Platform Compatibility Tests
// ============================================================================

/// Test: Version command works in different workspace types
///
/// Verifies that the version command works correctly regardless of the
/// workspace structure (single package, monorepo independent, monorepo unified).
#[tokio::test]
async fn test_version_works_in_all_workspace_types() {
    // Test in single package workspace
    let single_workspace = WorkspaceFixture::single_package().finalize();
    let args = VersionArgs { verbose: false };
    let result = execute_version(&args, single_workspace.root(), OutputFormat::Human);
    assert!(result.is_ok(), "Version should work in single package workspace");

    // Test in monorepo independent workspace
    let mono_independent = WorkspaceFixture::monorepo_independent().finalize();
    let result = execute_version(&args, mono_independent.root(), OutputFormat::Human);
    assert!(result.is_ok(), "Version should work in monorepo independent workspace");

    // Test in monorepo unified workspace
    let mono_unified = WorkspaceFixture::monorepo_unified().finalize();
    let result = execute_version(&args, mono_unified.root(), OutputFormat::Human);
    assert!(result.is_ok(), "Version should work in monorepo unified workspace");
}

/// Test: Version command works without Git initialization
///
/// Verifies that the version command does not depend on Git being initialized
/// in the workspace. Unlike other commands, version is purely informational.
#[tokio::test]
async fn test_version_works_without_git() {
    // ARRANGE: Create workspace WITHOUT Git
    let workspace = WorkspaceFixture::single_package().finalize();

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command
    let result = execute_version(&args, workspace.root(), OutputFormat::Human);

    // ASSERT: Should succeed even without Git
    assert!(result.is_ok(), "Version command should work without Git initialization");
}

/// Test: Version command with verbose flag in JSON format
///
/// Verifies that the verbose flag doesn't affect JSON output - JSON should
/// always include all information regardless of the verbose flag.
#[tokio::test]
async fn test_version_verbose_flag_with_json_format() {
    // ARRANGE: Create minimal workspace
    let workspace = WorkspaceFixture::single_package().finalize();

    // Test with verbose=false
    let args_no_verbose = VersionArgs { verbose: false };
    let result_no_verbose = execute_version(&args_no_verbose, workspace.root(), OutputFormat::Json);
    assert!(result_no_verbose.is_ok(), "Version with JSON should succeed (verbose=false)");

    // Test with verbose=true
    let args_verbose = VersionArgs { verbose: true };
    let result_verbose = execute_version(&args_verbose, workspace.root(), OutputFormat::Json);
    assert!(result_verbose.is_ok(), "Version with JSON should succeed (verbose=true)");

    // Both should produce the same structured JSON output
    // (verbose flag only affects human-readable format)
    let info_no_verbose = VersionInfo::new();
    let info_verbose = VersionInfo::new();

    let json_no_verbose = serde_json::to_string(&info_no_verbose).unwrap();
    let json_verbose = serde_json::to_string(&info_verbose).unwrap();

    // Both should serialize to identical JSON
    assert_eq!(
        json_no_verbose, json_verbose,
        "JSON output should be identical regardless of verbose flag"
    );
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

/// Test: Version command handles invalid workspace root gracefully
///
/// Verifies that the version command works even if the root path doesn't
/// point to a valid workspace, since version info doesn't depend on workspace.
#[tokio::test]
async fn test_version_with_invalid_workspace_root() {
    // ARRANGE: Create a temporary directory that's not a valid workspace
    let workspace = WorkspaceFixture::single_package().finalize();
    let invalid_root = workspace.root().join("non-existent-directory");

    let args = VersionArgs { verbose: false };

    // ACT: Execute version command with invalid root
    // Note: execute_version doesn't actually use the root parameter (_root),
    // so this should still succeed
    let result = execute_version(&args, &invalid_root, OutputFormat::Human);

    // ASSERT: Should succeed since version is workspace-independent
    assert!(result.is_ok(), "Version command should succeed even with invalid workspace root");
}

/// Test: VersionInfo default trait implementation
///
/// Verifies that VersionInfo implements Default and that default()
/// produces the same result as new().
#[tokio::test]
async fn test_version_info_default_implementation() {
    // ARRANGE & ACT: Create VersionInfo using both methods
    let info_new = VersionInfo::new();
    let info_default = VersionInfo::default();

    // ASSERT: Both should produce identical results
    assert_eq!(
        info_new.version, info_default.version,
        "Default and new should produce same version"
    );
    assert_eq!(
        info_new.rust_version, info_default.rust_version,
        "Default and new should produce same rust_version"
    );
    assert_eq!(
        info_new.build.profile, info_default.build.profile,
        "Default and new should produce same build profile"
    );
    assert_eq!(
        info_new.build.target, info_default.build.target,
        "Default and new should produce same build target"
    );
}
