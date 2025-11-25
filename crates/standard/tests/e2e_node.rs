//! # End-to-End Tests for Node Module
//!
//! ## What
//! Comprehensive e2e tests for Node.js abstractions including
//! `PackageManager`, `PackageManagerKind`, `RepoKind`, and `RepositoryInfo`.
//!
//! ## How
//! Tests create realistic project structures with different lock files
//! to verify package manager detection across npm, yarn, pnpm, and bun.
//!
//! ## Why
//! E2E tests ensure the node module correctly detects and characterizes
//! different package managers based on project structure and lock files.

#![allow(clippy::print_stdout)]

use sublime_standard_tools::{
    error::Result,
    filesystem::{AsyncFileSystem, FileSystemManager},
    node::{PackageManager, PackageManagerKind, RepoKind},
};
use tempfile::TempDir;

// ============================================================================
// PackageManagerKind Tests
// ============================================================================

#[tokio::test]
async fn test_package_manager_kind_commands() -> Result<()> {
    assert_eq!(PackageManagerKind::Npm.command(), "npm");
    assert_eq!(PackageManagerKind::Yarn.command(), "yarn");
    assert_eq!(PackageManagerKind::Pnpm.command(), "pnpm");
    assert_eq!(PackageManagerKind::Bun.command(), "bun");
    assert_eq!(PackageManagerKind::Jsr.command(), "jsr");
    Ok(())
}

#[tokio::test]
async fn test_package_manager_kind_lock_files() -> Result<()> {
    assert_eq!(PackageManagerKind::Npm.lock_file(), "package-lock.json");
    assert_eq!(PackageManagerKind::Yarn.lock_file(), "yarn.lock");
    assert_eq!(PackageManagerKind::Pnpm.lock_file(), "pnpm-lock.yaml");
    assert_eq!(PackageManagerKind::Bun.lock_file(), "bun.lockb");
    assert_eq!(PackageManagerKind::Jsr.lock_file(), "jsr.json");
    Ok(())
}

#[tokio::test]
async fn test_package_manager_kind_names() -> Result<()> {
    assert_eq!(PackageManagerKind::Npm.name(), "npm");
    assert_eq!(PackageManagerKind::Yarn.name(), "yarn");
    assert_eq!(PackageManagerKind::Pnpm.name(), "pnpm");
    assert_eq!(PackageManagerKind::Bun.name(), "bun");
    assert_eq!(PackageManagerKind::Jsr.name(), "jsr");
    Ok(())
}

#[tokio::test]
async fn test_package_manager_kind_supports_workspaces() -> Result<()> {
    assert!(PackageManagerKind::Npm.supports_workspaces());
    assert!(PackageManagerKind::Yarn.supports_workspaces());
    assert!(PackageManagerKind::Pnpm.supports_workspaces());
    assert!(PackageManagerKind::Bun.supports_workspaces());
    // Jsr doesn't primarily support workspaces
    assert!(!PackageManagerKind::Jsr.supports_workspaces());
    Ok(())
}

// ============================================================================
// PackageManager Detection Tests - NPM
// ============================================================================

#[tokio::test]
async fn test_detect_npm_from_lock_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create npm project with package-lock.json
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "npm-project", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(
        &temp_dir.path().join("package-lock.json"),
        r#"{"name": "npm-project", "lockfileVersion": 3}"#,
    )
    .await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Npm);
    assert_eq!(pm.command(), "npm");
    assert!(pm.supports_workspaces());

    Ok(())
}

// ============================================================================
// PackageManager Detection Tests - Yarn
// ============================================================================

#[tokio::test]
async fn test_detect_yarn_from_lock_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create yarn project with yarn.lock
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "yarn-project", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("yarn.lock"), "# yarn lockfile v1\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Yarn);
    assert_eq!(pm.command(), "yarn");
    assert!(pm.supports_workspaces());

    Ok(())
}

#[tokio::test]
async fn test_detect_yarn_berry() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create yarn berry project
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "yarn-berry", "version": "1.0.0", "packageManager": "yarn@3.6.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("yarn.lock"), "__metadata:\n  version: 6\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Yarn);

    Ok(())
}

// ============================================================================
// PackageManager Detection Tests - PNPM
// ============================================================================

#[tokio::test]
async fn test_detect_pnpm_from_lock_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create pnpm project with pnpm-lock.yaml
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "pnpm-project", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Pnpm);
    assert_eq!(pm.command(), "pnpm");
    assert!(pm.supports_workspaces());

    Ok(())
}

#[tokio::test]
async fn test_detect_pnpm_from_workspace_yaml() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create pnpm workspace project
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "@org/root", "version": "1.0.0", "private": true}"#,
    )
    .await?;
    fs.write_file_string(
        &temp_dir.path().join("pnpm-workspace.yaml"),
        "packages:\n  - 'packages/*'\n",
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Pnpm);

    Ok(())
}

// ============================================================================
// PackageManager Detection Tests - Bun
// ============================================================================

#[tokio::test]
async fn test_detect_bun_from_lock_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create bun project with bun.lockb
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "bun-project", "version": "1.0.0"}"#,
    )
    .await?;
    // bun.lockb is binary, but an empty file is enough for detection
    fs.write_file(&temp_dir.path().join("bun.lockb"), &[0u8; 4]).await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.kind(), PackageManagerKind::Bun);
    assert_eq!(pm.command(), "bun");
    assert!(pm.supports_workspaces());

    Ok(())
}

// ============================================================================
// PackageManager Detection Priority Tests
// ============================================================================

#[tokio::test]
async fn test_pnpm_has_priority_over_npm() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create project with both npm and pnpm lock files
    // (this can happen during migration)
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "dual-lock", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;
    fs.write_file_string(&temp_dir.path().join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    // pnpm should have higher priority
    assert_eq!(pm.kind(), PackageManagerKind::Pnpm);

    Ok(())
}

#[tokio::test]
async fn test_yarn_has_priority_over_npm() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create project with both npm and yarn lock files
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "yarn-npm", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;
    fs.write_file_string(&temp_dir.path().join("yarn.lock"), "# yarn lockfile v1\n").await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    // yarn should have higher priority
    assert_eq!(pm.kind(), PackageManagerKind::Yarn);

    Ok(())
}

// ============================================================================
// PackageManager Detection Error Tests
// ============================================================================

#[tokio::test]
async fn test_detect_fails_without_lock_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create project without any lock file
    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "no-lock", "version": "1.0.0"}"#,
    )
    .await?;

    let result = PackageManager::detect(temp_dir.path());

    // Should fail - no lock file to determine package manager
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_detect_fails_empty_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;

    let result = PackageManager::detect(temp_dir.path());

    // Should fail - no package.json
    assert!(result.is_err());

    Ok(())
}

// ============================================================================
// PackageManager Methods Tests
// ============================================================================

#[tokio::test]
async fn test_package_manager_root() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "root-test", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let pm = PackageManager::detect(temp_dir.path())?;

    assert_eq!(pm.root(), temp_dir.path());

    Ok(())
}

#[tokio::test]
async fn test_package_manager_lock_file_path() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    fs.write_file_string(
        &temp_dir.path().join("package.json"),
        r#"{"name": "lock-path-test", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&temp_dir.path().join("package-lock.json"), r#"{"lockfileVersion": 3}"#)
        .await?;

    let pm = PackageManager::detect(temp_dir.path())?;
    let lock_path = pm.lock_file_path();

    assert_eq!(lock_path, temp_dir.path().join("package-lock.json"));

    Ok(())
}

// ============================================================================
// RepoKind Tests
// ============================================================================

#[tokio::test]
async fn test_repo_kind_simple() -> Result<()> {
    assert_eq!(RepoKind::Simple.name(), "simple");
    assert!(!RepoKind::Simple.is_monorepo());

    Ok(())
}

#[tokio::test]
async fn test_repo_kind_monorepo() -> Result<()> {
    use sublime_standard_tools::monorepo::MonorepoKind;

    // RepoKind::Monorepo.name() returns "{kind.name()} monorepo"
    let npm_mono = RepoKind::Monorepo(MonorepoKind::NpmWorkSpace);
    assert_eq!(npm_mono.name(), "npm monorepo");
    assert!(npm_mono.is_monorepo());

    let yarn_mono = RepoKind::Monorepo(MonorepoKind::YarnWorkspaces);
    assert_eq!(yarn_mono.name(), "yarn monorepo");
    assert!(yarn_mono.is_monorepo());

    let pnpm_mono = RepoKind::Monorepo(MonorepoKind::PnpmWorkspaces);
    assert_eq!(pnpm_mono.name(), "pnpm monorepo");
    assert!(pnpm_mono.is_monorepo());

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_package_manager_detection_all_types() -> Result<()> {
    let fs = FileSystemManager::new();

    // Test each package manager type
    let test_cases = vec![
        ("npm", "package-lock.json", r#"{"lockfileVersion": 3}"#, PackageManagerKind::Npm),
        ("yarn", "yarn.lock", "# yarn lockfile v1\n", PackageManagerKind::Yarn),
        ("pnpm", "pnpm-lock.yaml", "lockfileVersion: 5.4\n", PackageManagerKind::Pnpm),
    ];

    for (name, lock_file, lock_content, expected_kind) in test_cases {
        let temp_dir = create_temp_dir()?;

        fs.write_file_string(
            &temp_dir.path().join("package.json"),
            &format!(r#"{{"name": "{name}-test", "version": "1.0.0"}}"#),
        )
        .await?;
        fs.write_file_string(&temp_dir.path().join(lock_file), lock_content).await?;

        let pm = PackageManager::detect(temp_dir.path())?;

        assert_eq!(pm.kind(), expected_kind, "Failed for {name}");
        println!("Successfully detected {name} package manager");
    }

    Ok(())
}

#[tokio::test]
async fn test_package_manager_in_nested_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create nested project structure
    let nested_path = temp_dir.path().join("projects").join("app");
    fs.create_dir_all(&nested_path).await?;

    fs.write_file_string(
        &nested_path.join("package.json"),
        r#"{"name": "nested-app", "version": "1.0.0"}"#,
    )
    .await?;
    fs.write_file_string(&nested_path.join("yarn.lock"), "# yarn lockfile v1\n").await?;

    let pm = PackageManager::detect(&nested_path)?;

    assert_eq!(pm.kind(), PackageManagerKind::Yarn);
    assert_eq!(pm.root(), nested_path.as_path());

    Ok(())
}

#[tokio::test]
async fn test_package_manager_with_workspaces() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create a monorepo with workspaces
    let root = temp_dir.path();

    fs.write_file_string(
        &root.join("package.json"),
        r#"{"name": "@org/monorepo", "private": true, "workspaces": ["packages/*"]}"#,
    )
    .await?;
    fs.write_file_string(&root.join("pnpm-lock.yaml"), "lockfileVersion: 5.4\n").await?;

    // Create workspace packages
    for pkg_name in ["lib", "app"] {
        let pkg_dir = root.join("packages").join(pkg_name);
        fs.create_dir_all(&pkg_dir).await?;
        fs.write_file_string(
            &pkg_dir.join("package.json"),
            &format!(r#"{{"name": "@org/{pkg_name}", "version": "1.0.0"}}"#),
        )
        .await?;
    }

    let pm = PackageManager::detect(root)?;

    assert_eq!(pm.kind(), PackageManagerKind::Pnpm);
    assert!(pm.supports_workspaces());

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
