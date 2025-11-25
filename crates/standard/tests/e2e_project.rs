//! # End-to-End Tests for Project Module
//!
//! ## What
//! Comprehensive e2e tests for project detection and management including
//! `ProjectDetector`, `ProjectManager`, and `ProjectDescriptor`.
//!
//! ## How
//! Tests create realistic project structures using temporary directories and verify
//! detection, validation, and management operations across different project types.
//!
//! ## Why
//! E2E tests ensure the project module correctly identifies, validates, and manages
//! both simple and monorepo projects with accurate metadata extraction.

#![allow(clippy::print_stdout)]

use sublime_standard_tools::{
    config::StandardConfig,
    error::Result,
    filesystem::{AsyncFileSystem, FileSystemManager},
    monorepo::MonorepoKind,
    node::RepoKind,
    project::{ProjectDetector, ProjectKind, ProjectManager},
};
use tempfile::TempDir;

// ============================================================================
// ProjectDetector Creation Tests
// ============================================================================

#[tokio::test]
async fn test_project_detector_new() -> Result<()> {
    let detector = ProjectDetector::new();
    // Should be created without errors - verify by using it
    let temp_dir = create_temp_dir()?;
    let result = detector.is_valid_project(temp_dir.path()).await;
    // Empty dir is not a valid project
    assert!(!result);
    Ok(())
}

#[tokio::test]
async fn test_project_detector_with_filesystem() -> Result<()> {
    let fs = FileSystemManager::new();
    let detector = ProjectDetector::with_filesystem(fs);
    // Should be created without errors
    let temp_dir = create_temp_dir()?;
    let result = detector.is_valid_project(temp_dir.path()).await;
    assert!(!result);
    Ok(())
}

// ============================================================================
// Simple Project Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_simple_nodejs_project() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create simple Node.js project
    let package_json = r#"{
        "name": "simple-app",
        "version": "1.0.0",
        "description": "A simple Node.js application",
        "main": "index.js"
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    assert_eq!(info.root(), temp_dir.path());

    Ok(())
}

#[tokio::test]
async fn test_detect_simple_project_with_typescript() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let package_json = r#"{
        "name": "ts-app",
        "version": "1.0.0",
        "devDependencies": {
            "typescript": "^5.0.0"
        }
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let tsconfig = r#"{
        "compilerOptions": {
            "target": "ES2020",
            "module": "commonjs"
        }
    }"#;
    fs.write_file_string(&temp_dir.path().join("tsconfig.json"), tsconfig).await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));

    Ok(())
}

#[tokio::test]
async fn test_detect_kind_simple() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create simple project (no workspaces)
    let package_json = r#"{"name": "simple", "version": "1.0.0"}"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;

    let detector = ProjectDetector::new();
    let kind = detector.detect_kind(temp_dir.path()).await?;

    assert_eq!(kind, ProjectKind::Repository(RepoKind::Simple));

    Ok(())
}

// ============================================================================
// Monorepo Project Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_monorepo_project() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create monorepo structure
    let package_json = r#"{
        "name": "@test/monorepo",
        "version": "1.0.0",
        "private": true,
        "workspaces": ["packages/*"]
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    // Create a workspace package
    let pkg_dir = temp_dir.path().join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    fs.write_file_string(
        &pkg_dir.join("package.json"),
        r#"{"name": "@test/lib", "version": "1.0.0"}"#,
    )
    .await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    // Should detect as monorepo
    assert!(info.kind().is_monorepo());

    Ok(())
}

#[tokio::test]
async fn test_detect_kind_monorepo() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create pnpm monorepo
    fs.write_file_string(
        &temp_dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - 'packages/*'\n",
    )
    .await?;
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "@test/root", "private": true}"#,
    )
    .await?;

    let detector = ProjectDetector::new();
    let kind = detector.detect_kind(temp_dir.path()).await?;

    assert!(kind.is_monorepo());

    Ok(())
}

// ============================================================================
// is_valid_project Tests
// ============================================================================

#[tokio::test]
async fn test_is_valid_project_true() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "valid", "version": "1.0.0"}"#,
    )
    .await?;

    let detector = ProjectDetector::new();
    assert!(detector.is_valid_project(temp_dir.path()).await);

    Ok(())
}

#[tokio::test]
async fn test_is_valid_project_false_empty() -> Result<()> {
    let temp_dir = create_temp_dir()?;

    let detector = ProjectDetector::new();
    assert!(!detector.is_valid_project(temp_dir.path()).await);

    Ok(())
}

#[tokio::test]
async fn test_is_valid_project_false_no_package_json() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create some files but no package.json
    fs.write_file_string(&temp_dir.path().join("index.js"), "console.log('hello');").await?;
    fs.write_file_string(&temp_dir.path().join("README.md"), "# Test").await?;

    let detector = ProjectDetector::new();
    assert!(!detector.is_valid_project(temp_dir.path()).await);

    Ok(())
}

// ============================================================================
// ProjectManager Tests
// ============================================================================

#[tokio::test]
async fn test_project_manager_create_project() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let package_json = r#"{
        "name": "managed-project",
        "version": "2.0.0",
        "description": "A managed project"
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;

    let manager = ProjectManager::new();
    let project = manager.create_project(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    assert_eq!(info.root(), temp_dir.path());
    assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));

    Ok(())
}

#[tokio::test]
async fn test_project_manager_with_config() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "configured", "version": "1.0.0"}"#,
    )
    .await?;

    let config = StandardConfig::default();
    let manager = ProjectManager::new();
    let project = manager.create_project(temp_dir.path(), Some(&config)).await?;

    assert_eq!(project.as_project_info().kind(), ProjectKind::Repository(RepoKind::Simple));

    Ok(())
}

// ============================================================================
// ProjectKind Tests
// ============================================================================

#[tokio::test]
async fn test_project_kind_names() -> Result<()> {
    // Test simple kind
    let simple_kind = ProjectKind::Repository(RepoKind::Simple);
    assert_eq!(simple_kind.name(), "simple");
    assert!(!simple_kind.is_monorepo());

    // Test monorepo kinds
    // ProjectKind delegates to RepoKind.name() which returns "{kind} monorepo"
    let npm_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::NpmWorkSpace));
    assert_eq!(npm_mono.name(), "npm monorepo");
    assert!(npm_mono.is_monorepo());

    let yarn_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::YarnWorkspaces));
    assert_eq!(yarn_mono.name(), "yarn monorepo");
    assert!(yarn_mono.is_monorepo());

    let pnpm_mono = ProjectKind::Repository(RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces));
    assert_eq!(pnpm_mono.name(), "pnpm monorepo");
    assert!(pnpm_mono.is_monorepo());

    Ok(())
}

// ============================================================================
// Project Metadata Extraction Tests
// ============================================================================

#[tokio::test]
async fn test_project_extracts_package_manager() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create npm project
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "npm-app", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    // Should detect npm as package manager
    let pm = info.package_manager();
    assert!(pm.is_some());

    Ok(())
}

#[tokio::test]
async fn test_project_extracts_package_json() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    let package_json = r#"{
        "name": "@scope/my-package",
        "version": "3.2.1",
        "description": "Test package"
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    let pkg_json = info.package_json();
    assert!(pkg_json.is_some());

    let pkg = pkg_json.expect("should have package.json");
    assert_eq!(pkg.name, "@scope/my-package");
    assert_eq!(pkg.version, "3.2.1");

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_detect_multiple_project_types() -> Result<()> {
    let detector = ProjectDetector::new();
    let fs = FileSystemManager::new();

    // Test 1: Simple npm project
    let npm_dir = create_temp_dir()?;
    fs.write_file_string(
        &npm_dir.path().join("package.json"),
        r#"{"name": "npm-project", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&npm_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let npm_project = detector.detect(npm_dir.path(), None).await?;
    assert_eq!(npm_project.as_project_info().kind(), ProjectKind::Repository(RepoKind::Simple));

    // Test 2: pnpm monorepo
    let pnpm_dir = create_temp_dir()?;
    fs.write_file_string(
        &pnpm_dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - 'packages/*'\n",
    )
    .await?;
    fs.write_file_string(
        &pnpm_dir.path().join("package.json"),
        r#"{"name": "@test/pnpm-mono", "version": "1.0.0", "private": true}"#,
    )
    .await?;
    let pkg_dir = pnpm_dir.path().join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    fs.write_file_string(
        &pkg_dir.join("package.json"),
        r#"{"name": "@test/lib", "version": "1.0.0"}"#,
    )
    .await?;

    let pnpm_project = detector.detect(pnpm_dir.path(), None).await?;
    assert!(pnpm_project.as_project_info().kind().is_monorepo());

    // Test 3: yarn workspaces
    let yarn_dir = create_temp_dir()?;
    fs.write_file_string(
        &yarn_dir.path().join("package.json"),
        r#"{"name": "@test/yarn-mono", "version": "1.0.0", "private": true, "workspaces": ["packages/*"]}"#,
    )
    .await?;
    fs.write_file_string(&yarn_dir.path().join("yarn.lock"), "# yarn lockfile v1\n").await?;
    let pkg_dir = yarn_dir.path().join("packages").join("lib");
    fs.create_dir_all(&pkg_dir).await?;
    fs.write_file_string(
        &pkg_dir.join("package.json"),
        r#"{"name": "@test/lib", "version": "1.0.0"}"#,
    )
    .await?;

    let yarn_project = detector.detect(yarn_dir.path(), None).await?;
    assert!(yarn_project.as_project_info().kind().is_monorepo());

    Ok(())
}

#[tokio::test]
async fn test_project_with_all_common_files() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create a comprehensive project structure
    let package_json = r#"{
        "name": "@full/project",
        "version": "1.0.0",
        "description": "Full project with all common files",
        "main": "dist/index.js",
        "types": "dist/index.d.ts",
        "scripts": {
            "build": "tsc",
            "test": "jest",
            "lint": "eslint ."
        },
        "dependencies": {
            "express": "^4.18.0"
        },
        "devDependencies": {
            "typescript": "^5.0.0",
            "@types/node": "^20.0.0",
            "jest": "^29.0.0"
        }
    }"#;
    fs.write_file_string(&temp_dir.path().join("package.json"), package_json).await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    // TypeScript config
    fs.write_file_string(
        &temp_dir.path().join("tsconfig.json"),
        r#"{"compilerOptions": {"outDir": "./dist"}}"#,
    )
    .await?;

    // Create source directory
    fs.create_dir_all(&temp_dir.path().join("src")).await?;
    fs.write_file_string(
        &temp_dir.path().join("src").join("index.ts"),
        "export const hello = 'world';",
    )
    .await?;

    // Create test directory
    fs.create_dir_all(&temp_dir.path().join("tests")).await?;
    fs.write_file_string(
        &temp_dir.path().join("tests").join("index.test.ts"),
        "test('example', () => {});",
    )
    .await?;

    let detector = ProjectDetector::new();
    let project = detector.detect(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    assert_eq!(info.kind(), ProjectKind::Repository(RepoKind::Simple));
    assert!(info.package_json().is_some());
    assert!(info.package_manager().is_some());

    Ok(())
}

#[tokio::test]
async fn test_detect_project_error_on_invalid_json() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create invalid package.json
    fs.write_file_string(&temp_dir.path().join("package.json"), "{ invalid json }").await?;

    let detector = ProjectDetector::new();
    let result = detector.detect(temp_dir.path(), None).await;

    // Should fail due to invalid JSON
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_project_validation_status() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "validated", "version": "1.0.0"}"#,
    )
    .await?;

    let manager = ProjectManager::new();
    let project = manager.create_project(temp_dir.path(), None).await?;
    let info = project.as_project_info();

    // Check validation status is captured
    let status = info.validation_status();
    println!("Validation status: {status:?}");

    Ok(())
}

// ============================================================================
// Sequential Detection Tests
// ============================================================================

#[tokio::test]
async fn test_sequential_project_detection() -> Result<()> {
    let detector = ProjectDetector::new();
    let fs = FileSystemManager::new();

    // Create multiple projects
    let mut temp_dirs = Vec::new();
    for i in 0..5 {
        let temp_dir = create_temp_dir()?;
        fs.write_file_string(
            &temp_dir.path().join("package.json"),
            &format!(r#"{{"name": "project-{i}", "version": "1.0.0"}}"#),
        )
        .await?;
        temp_dirs.push(temp_dir);
    }

    // Detect all projects sequentially
    for (i, temp_dir) in temp_dirs.iter().enumerate() {
        let project = detector.detect(temp_dir.path(), None).await?;
        assert_eq!(project.as_project_info().kind(), ProjectKind::Repository(RepoKind::Simple));
        println!("Project {i} detected successfully");
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a temporary directory for testing
fn create_temp_dir() -> Result<TempDir> {
    TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })
}
