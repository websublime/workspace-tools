//! Workflow tests
//!
//! Comprehensive tests for all workflow functionality including development,
//! release, and integration workflows.

use std::sync::Arc;
use tempfile::TempDir;

use super::{
    types::{ChangesetHookIntegration, ImpactLevel, ReleaseOptions},
    DevelopmentWorkflow, ReleaseWorkflow,
};
use crate::{core::MonorepoProject, AffectedPackagesAnalysis};
use crate::{ChangeAnalysis, ChangeSignificance, PackageChange, PackageChangeType};

/// Creates a test project with proper Git setup for testing workflows
#[allow(clippy::arc_with_non_send_sync)]
fn create_test_project() -> (TempDir, Arc<MonorepoProject>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Configure Git user for testing
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git name");

    // Create basic structure
    std::fs::create_dir_all(temp_dir.path().join("packages"))
        .expect("Failed to create packages dir");

    let root_package_json = r#"{
  "name": "test-monorepo",
  "version": "1.0.0",
  "private": true,
  "workspaces": ["packages/*"]
}"#;
    std::fs::write(temp_dir.path().join("package.json"), root_package_json)
        .expect("Failed to write package.json");

    // Create package-lock.json to make it detectable as an npm monorepo
    std::fs::write(temp_dir.path().join("package-lock.json"), "{}")
        .expect("Failed to write package-lock.json");

    let project =
        Arc::new(MonorepoProject::new(temp_dir.path()).expect("Failed to create MonorepoProject"));
    (temp_dir, project)
}

// Development Workflow Tests

#[tokio::test]
async fn test_development_workflow_creation() {
    let (_temp_dir, project) = create_test_project();
    let workflow = DevelopmentWorkflow::new(project).expect("Failed to create DevelopmentWorkflow");

    // Test that workflow is created successfully
    let result = workflow.execute(Some("HEAD")).await.expect("Failed to execute workflow");

    // Should complete without errors (may have no changes)
    assert!(result.duration.as_millis() > 0);
    assert_eq!(result.affected_tasks.len(), 0); // No affected packages initially
}

#[tokio::test]
async fn test_impact_level_determination() {
    let (_temp_dir, project) = create_test_project();
    let workflow = DevelopmentWorkflow::new(project).expect("Failed to create DevelopmentWorkflow");

    // Test small changes (should be low impact based on file count)
    let mut small_change_metadata = std::collections::HashMap::new();
    small_change_metadata.insert("total_files".to_string(), "2".to_string());

    let small_change = PackageChange {
        package_name: "@test/core".to_string(),
        change_type: PackageChangeType::SourceCode, // Fixed - no decisions made
        significance: ChangeSignificance::Low,      // Fixed - no decisions made
        changed_files: vec![
            sublime_git_tools::GitChangedFile {
                path: "README.md".to_string(),
                status: sublime_git_tools::GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
            sublime_git_tools::GitChangedFile {
                path: "package.json".to_string(),
                status: sublime_git_tools::GitFileStatus::Modified,
                staged: false,
                workdir: true,
            },
        ],
        suggested_version_bump: crate::config::VersionBumpType::Patch,
        metadata: small_change_metadata,
    };

    let impact = workflow.determine_impact_level(&small_change);
    assert_eq!(impact, ImpactLevel::Low);

    // Test large changes (should be high impact based on file count)
    let mut large_change_metadata = std::collections::HashMap::new();
    large_change_metadata.insert("total_files".to_string(), "20".to_string());

    let large_change = PackageChange {
        package_name: "@test/core".to_string(),
        change_type: PackageChangeType::SourceCode, // Fixed - no decisions made
        significance: ChangeSignificance::Low,      // Fixed - no decisions made
        changed_files: (0..20)
            .map(|i| sublime_git_tools::GitChangedFile {
                path: format!("src/file{i}.ts"),
                status: sublime_git_tools::GitFileStatus::Modified,
                staged: false,
                workdir: true,
            })
            .collect(),
        suggested_version_bump: crate::config::VersionBumpType::Patch,
        metadata: large_change_metadata,
    };

    let impact = workflow.determine_impact_level(&large_change);
    assert_eq!(impact, ImpactLevel::High);
}

#[tokio::test]
async fn test_recommendation_generation() {
    let (_temp_dir, project) = create_test_project();
    let workflow = DevelopmentWorkflow::new(project).expect("Failed to create DevelopmentWorkflow");

    // Test with no changes
    let empty_analysis = ChangeAnalysis {
        from_ref: "HEAD~1".to_string(),
        to_ref: "HEAD".to_string(),
        changed_files: Vec::new(),
        package_changes: Vec::new(),
        affected_packages: AffectedPackagesAnalysis::default(),
        significance_analysis: Vec::new(),
    };

    let recommendations = workflow
        .generate_recommendations(&empty_analysis, &[])
        .expect("Failed to generate recommendations");

    assert!(!recommendations.is_empty());
    assert!(recommendations[0].contains("No changes detected"));
}

// Release Workflow Tests

#[tokio::test]
async fn test_release_workflow_creation() {
    let (_temp_dir, project) = create_test_project();
    let workflow = ReleaseWorkflow::new(project).expect("Failed to create ReleaseWorkflow");

    // Test dry run release
    let options = ReleaseOptions {
        dry_run: true,
        skip_tests: true,
        skip_changelogs: true,
        target_environments: vec!["production".to_string()],
        force: false,
    };

    let result = workflow.execute(options).await.expect("Failed to execute release");

    // Should complete without errors in dry run mode
    assert!(result.duration.as_millis() > 0);
    assert!(result.changesets_applied.is_empty()); // No actual changes in dry run
}

#[tokio::test]
async fn test_release_workflow_with_changesets() {
    let (_temp_dir, project) = create_test_project();
    let workflow = ReleaseWorkflow::new(project).expect("Failed to create ReleaseWorkflow");

    // Test release with force option (to bypass validation)
    let options = ReleaseOptions {
        dry_run: true,
        skip_tests: true,
        skip_changelogs: true,
        target_environments: vec!["staging".to_string()],
        force: true,
    };

    let result = workflow.execute(options).await.expect("Failed to execute forced release");

    // Should complete successfully with force option
    assert!(result.duration.as_millis() > 0);
}

// Integration Workflow Tests

#[tokio::test]
async fn test_integration_workflow_creation() {
    let (_temp_dir, project) = create_test_project();
    let workflow =
        ChangesetHookIntegration::new(project).expect("Failed to create ChangesetHookIntegration");

    // Test setup validation
    let result = workflow.setup_integration().expect("Failed to setup integration");

    // Should complete setup successfully
    assert!(result);
}

#[tokio::test]
async fn test_integration_changeset_validation() {
    let (_temp_dir, project) = create_test_project();
    let workflow =
        ChangesetHookIntegration::new(project).expect("Failed to create ChangesetHookIntegration");

    // Test changeset validation with no affected packages
    let result = workflow.validate_changesets_for_commit().expect("Failed to validate changeset");

    // Should pass with no packages affected
    assert!(result);
}

#[tokio::test]
async fn test_integration_pre_push_validation() {
    let (_temp_dir, project) = create_test_project();
    let workflow =
        ChangesetHookIntegration::new(project).expect("Failed to create ChangesetHookIntegration");

    // Test pre-push validation with no commits
    let result = workflow.validate_tests_for_push(&[]).await.expect("Failed to validate pre-push");

    // Should pass with no commits to push
    assert!(result);
}
