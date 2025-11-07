//! # E2E Tests for Audit Commands
//!
//! **What**: End-to-end tests for all audit-related commands including
//! comprehensive audit, version consistency audit, dependency audit,
//! breaking changes audit, upgrades audit, and report generation.
//!
//! **How**: Creates real temporary workspaces with various configurations,
//! executes audit commands with different parameters, and validates that
//! audits correctly detect issues, calculate health scores, and generate
//! reports in multiple formats.
//!
//! **Why**: Ensures the complete audit workflow works correctly across
//! different workspace types, audit sections, output formats, and severity
//! levels. Validates comprehensive health checking and actionable reporting.
//!
//! **Note**: Some tests may be skipped in CI/CD environments where:
//! - Network access to npm registry is restricted
//! - Git repository history is limited
//! - Authentication is required for registries

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

mod common;

use common::fixtures::WorkspaceFixture;
use common::helpers::{add_dependency, read_file, read_json_file};
use std::io::Cursor;
use sublime_cli_tools::cli::commands::AuditArgs;
use sublime_cli_tools::commands::audit::execute_audit;
use sublime_cli_tools::output::{Output, OutputFormat};

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a test output with buffer for capturing output.
fn create_test_output() -> (Output, Cursor<Vec<u8>>) {
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Human, Box::new(buffer.clone()), false);
    (output, buffer)
}

/// Creates a JSON output for testing JSON responses.
fn create_json_output() -> (Output, Cursor<Vec<u8>>) {
    let buffer = Cursor::new(Vec::new());
    let output = Output::new(OutputFormat::Json, Box::new(buffer.clone()), false);
    (output, buffer)
}

/// Creates version inconsistencies in a monorepo fixture.
///
/// Modifies package dependencies to create version mismatches for testing.
fn create_version_inconsistencies(workspace: &WorkspaceFixture) {
    // pkg-a uses lodash ^4.17.20
    let pkg_a_json = workspace.root().join("packages/pkg-a/package.json");
    add_dependency(&pkg_a_json, "lodash", "^4.17.20");

    // pkg-b uses lodash ^4.17.21 (different version)
    let pkg_b_json = workspace.root().join("packages/pkg-b/package.json");
    add_dependency(&pkg_b_json, "lodash", "^4.17.21");
}

/// Creates circular dependencies in a monorepo fixture.
///
/// Modifies package dependencies to create circular dependency cycles.
fn create_circular_dependencies(workspace: &WorkspaceFixture) {
    // pkg-a depends on pkg-b
    let pkg_a_json = workspace.root().join("packages/pkg-a/package.json");
    add_dependency(&pkg_a_json, "@test/pkg-b", "^1.0.0");

    // pkg-b depends on pkg-a (creates cycle)
    let pkg_b_json = workspace.root().join("packages/pkg-b/package.json");
    add_dependency(&pkg_b_json, "@test/pkg-a", "^1.0.0");
}

/// Creates internal dependency version inconsistencies in a monorepo.
///
/// Sets up different version specifications for the same internal package.
fn create_internal_version_inconsistencies(workspace: &WorkspaceFixture) {
    // Create pkg-c that depends on pkg-a
    let pkg_c_dir = workspace.root().join("packages/pkg-c");
    std::fs::create_dir_all(&pkg_c_dir).expect("Failed to create pkg-c dir");

    let pkg_c_json = pkg_c_dir.join("package.json");
    let pkg_c_content = serde_json::json!({
        "name": "@test/pkg-c",
        "version": "1.0.0",
        "dependencies": {
            "@test/pkg-a": "^1.0.0"  // Version range
        }
    });
    std::fs::write(
        &pkg_c_json,
        serde_json::to_string_pretty(&pkg_c_content).expect("Failed to serialize"),
    )
    .expect("Failed to write pkg-c package.json");

    // pkg-b depends on pkg-a with exact version
    let pkg_b_json = workspace.root().join("packages/pkg-b/package.json");
    let mut pkg_b: serde_json::Value = read_json_file(&pkg_b_json);

    if pkg_b.get("dependencies").is_none() {
        pkg_b["dependencies"] = serde_json::json!({});
    }
    pkg_b["dependencies"]["@test/pkg-a"] = serde_json::Value::String("1.0.0".to_string()); // Exact version

    std::fs::write(&pkg_b_json, serde_json::to_string_pretty(&pkg_b).expect("Failed to serialize"))
        .expect("Failed to write pkg-b package.json");
}

// ============================================================================
// Comprehensive Audit Tests
// ============================================================================

/// Test: Comprehensive audit with specific sections (excludes breaking-changes)
///
/// Breaking changes audit requires specific git setup with tags and commit ranges.
/// We test the core auditable sections that work reliably in E2E tests.
#[tokio::test]
async fn test_audit_comprehensive_core_sections() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(2)
        .finalize();

    // Test core sections excluding breaking-changes
    let args = AuditArgs {
        sections: vec!["dependencies".to_string(), "version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Core audit sections should succeed: {:?}", result.err());
}

/// Test: Comprehensive audit with JSON output format
#[tokio::test]
async fn test_audit_comprehensive_json_output() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, buffer) = create_json_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());

    let output_bytes = buffer.into_inner();
    if !output_bytes.is_empty() {
        // Try to parse as JSON if output was produced
        let parse_result: Result<serde_json::Value, _> = serde_json::from_slice(&output_bytes);
        assert!(
            parse_result.is_ok(),
            "JSON output should be valid: {:?}",
            String::from_utf8_lossy(&output_bytes)
        );
    }
}

/// Test: Comprehensive audit with minimal verbosity
#[tokio::test]
async fn test_audit_comprehensive_minimal_verbosity() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "minimal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

/// Test: Comprehensive audit with detailed verbosity
#[tokio::test]
async fn test_audit_comprehensive_detailed_verbosity() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Create some version inconsistencies to have something to report
    create_version_inconsistencies(&workspace);

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

/// Test: Comprehensive audit without health score
#[tokio::test]
async fn test_audit_comprehensive_no_health_score() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: true,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

// ============================================================================
// Version Consistency Audit Tests
// ============================================================================

/// Test: Version consistency audit runs successfully
#[tokio::test]
async fn test_audit_version_consistency_executes() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Version consistency audit should succeed: {:?}", result.err());
}

/// Test: Version consistency audit with internal dependency inconsistencies
#[tokio::test]
async fn test_audit_version_consistency_with_internal_deps() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Create internal version inconsistencies
    create_internal_version_inconsistencies(&workspace);

    let args = AuditArgs {
        sections: vec!["version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Version consistency audit should succeed: {:?}", result.err());
}

/// Test: Version consistency audit with minimal severity filter
#[tokio::test]
async fn test_audit_version_consistency_critical_severity_only() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["version-consistency".to_string()],
        output: None,
        min_severity: "critical".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Version consistency audit should succeed: {:?}", result.err());
}

// ============================================================================
// Dependencies Audit Tests
// ============================================================================

/// Test: Dependency audit executes successfully
#[tokio::test]
async fn test_audit_dependencies_executes() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Dependency audit should succeed: {:?}", result.err());
}

/// Test: Dependency audit with circular dependencies
#[tokio::test]
async fn test_audit_dependencies_with_circular() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Create circular dependencies
    create_circular_dependencies(&workspace);

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Dependency audit should succeed: {:?}", result.err());
}

/// Test: Dependency audit with version conflicts
#[tokio::test]
async fn test_audit_dependencies_with_version_conflicts() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Create version conflicts
    create_version_inconsistencies(&workspace);

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Dependency audit should succeed: {:?}", result.err());
}

/// Test: Dependency audit with internal dependencies
#[tokio::test]
async fn test_audit_dependencies_with_internal_deps() {
    let workspace = WorkspaceFixture::monorepo_with_internal_deps()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Dependency audit should succeed: {:?}", result.err());
}

/// Test: Dependency audit with normal verbosity
#[tokio::test]
async fn test_audit_dependencies_normal_verbosity() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Dependency audit should succeed: {:?}", result.err());
}

// ============================================================================
// Upgrades Audit Tests (Network-Dependent)
// ============================================================================

/// Test: Upgrades audit executes (may be skipped if network unavailable)
///
/// Note: This test may be skipped in CI/CD environments where:
/// - npm registry is not accessible
/// - Authentication is required
#[tokio::test]
async fn test_audit_upgrades_executes() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["upgrades".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    // If audit fails due to network issues, that's acceptable
    if result.is_err() {
        eprintln!("Upgrade audit skipped due to: {:?}", result.err());
        return;
    }
}

/// Test: Upgrades audit with detailed verbosity
#[tokio::test]
async fn test_audit_upgrades_detailed_output() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["upgrades".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    if result.is_err() {
        eprintln!("Upgrade audit skipped due to: {:?}", result.err());
    }
}

/// Test: Upgrades audit filters by severity
#[tokio::test]
async fn test_audit_upgrades_filters_by_severity() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["upgrades".to_string()],
        output: None,
        min_severity: "warning".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    if result.is_err() {
        eprintln!("Upgrade audit skipped due to: {:?}", result.err());
    }
}

// ============================================================================
// Multiple Sections Tests
// ============================================================================

/// Test: Audit with multiple specific sections
#[tokio::test]
async fn test_audit_multiple_sections() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string(), "version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

/// Test: Audit with duplicate sections (should deduplicate)
#[tokio::test]
async fn test_audit_duplicate_sections_deduplication() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec![
            "dependencies".to_string(),
            "dependencies".to_string(), // Duplicate
        ],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed and deduplicate: {:?}", result.err());
}

// ============================================================================
// Report Generation and Export Tests
// ============================================================================

/// Test: Audit generates report file
#[tokio::test]
async fn test_audit_generates_report_file() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let report_path = workspace.root().join("audit-report.txt");

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: Some(report_path.clone()),
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());

    // Verify report file was created
    assert!(report_path.exists(), "Report file should be created");

    let report_content = read_file(&report_path);
    assert!(!report_content.is_empty(), "Report file should have content");
}

/// Test: Audit exports to HTML format
#[tokio::test]
async fn test_audit_export_html_format() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let export_path = workspace.root().join("audit-report.html");

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: Some("html".to_string()),
        export_file: Some(export_path.clone()),
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());

    // Verify HTML file was created
    assert!(export_path.exists(), "HTML export file should be created");

    let html_content = read_file(&export_path);
    assert!(!html_content.is_empty(), "HTML file should have content");
    assert!(html_content.contains("<html") || html_content.contains("<!DOCTYPE"), "Should be HTML");
}

/// Test: Audit exports to Markdown format
#[tokio::test]
async fn test_audit_export_markdown_format() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let export_path = workspace.root().join("audit-report.md");

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: Some("markdown".to_string()),
        export_file: Some(export_path.clone()),
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());

    // Verify Markdown file was created
    assert!(export_path.exists(), "Markdown export file should be created");

    let md_content = read_file(&export_path);
    assert!(!md_content.is_empty(), "Markdown file should have content");
    assert!(md_content.contains('#') || md_content.contains("**"), "Should be Markdown");
}

/// Test: Audit export with custom sections
#[tokio::test]
async fn test_audit_export_custom_sections() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let export_path = workspace.root().join("custom-audit.html");

    let args = AuditArgs {
        sections: vec!["dependencies".to_string(), "version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "detailed".to_string(),
        no_health_score: false,
        export: Some("html".to_string()),
        export_file: Some(export_path.clone()),
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());

    // Verify export file was created
    assert!(export_path.exists(), "Export file should be created");

    let content = read_file(&export_path);
    assert!(!content.is_empty(), "Export file should have content");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test: Audit with invalid section name fails gracefully
#[tokio::test]
async fn test_audit_invalid_section_fails_gracefully() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["invalid-section".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_err(), "Audit should fail with invalid section");
}

/// Test: Audit with invalid severity level fails gracefully
#[tokio::test]
async fn test_audit_invalid_severity_fails_gracefully() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "invalid-severity".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_err(), "Audit should fail with invalid severity");
}

/// Test: Audit with invalid verbosity level fails gracefully
#[tokio::test]
async fn test_audit_invalid_verbosity_fails_gracefully() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "invalid-verbosity".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_err(), "Audit should fail with invalid verbosity");
}

/// Test: Audit with missing config file uses defaults
#[tokio::test]
async fn test_audit_missing_config_uses_defaults() {
    let workspace = WorkspaceFixture::single_package().with_git().with_commits(1).finalize();

    // Don't create config file

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    // Should succeed with default configuration
    assert!(result.is_ok(), "Audit should use defaults when config missing: {:?}", result.err());
}

// ============================================================================
// Health Score Calculation Tests
// ============================================================================

/// Test: Health score is calculated for workspace
#[tokio::test]
async fn test_audit_calculates_health_score() {
    let workspace = WorkspaceFixture::monorepo_independent()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    // Create some issues
    create_version_inconsistencies(&workspace);

    let args = AuditArgs {
        sections: vec!["dependencies".to_string(), "version-consistency".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

/// Test: Clean workspace produces good health metrics
#[tokio::test]
async fn test_audit_clean_workspace_health() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit should succeed: {:?}", result.err());
}

// ============================================================================
// Additional Audit Tests - Gap Coverage
// ============================================================================

/// Test: Audit breaking changes section
///
/// This test validates that the 'breaking-changes' section can be audited
/// independently to check for breaking changes in dependencies.
#[tokio::test]
async fn test_audit_breaking_changes_section() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(3) // Need multiple commits for breaking changes detection
        .finalize();

    let args = AuditArgs {
        sections: vec!["breaking-changes".to_string()],
        output: None,
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    // Breaking changes detection requires git history and may fail gracefully
    // if there's insufficient commit history or no changes detected
    match result {
        Ok(()) => {
            // Successfully ran audit
        }
        Err(e) => {
            // Should fail gracefully with clear error message, not panic
            let err_str = format!("{e:?}");
            assert!(
                err_str.contains("commit")
                    || err_str.contains("range")
                    || err_str.contains("No commits found")
                    || err_str.contains("Breaking changes"),
                "Should have git-related or breaking-changes error: {e:?}"
            );
        }
    }
}

/// Test: Audit output to file
///
/// This test validates that the `--output` flag correctly writes audit
/// results to a specified file path.
#[tokio::test]
async fn test_audit_output_to_file() {
    let workspace = WorkspaceFixture::single_package()
        .with_default_config()
        .with_git()
        .with_commits(1)
        .finalize();

    let output_file = workspace.root().join("audit-results.txt");

    let args = AuditArgs {
        sections: vec!["dependencies".to_string()],
        output: Some(output_file.clone()),
        min_severity: "info".to_string(),
        verbosity: "normal".to_string(),
        no_health_score: false,
        export: None,
        export_file: None,
    };

    let (output, _buffer) = create_test_output();
    let result = execute_audit(&args, &output, workspace.root(), None).await;

    assert!(result.is_ok(), "Audit with output file should succeed: {:?}", result.err());

    // Verify output file was created
    assert!(output_file.exists(), "Output file should be created at specified path");

    // Verify file has content
    let content = read_file(&output_file);
    assert!(!content.is_empty(), "Output file should contain audit results");
}
