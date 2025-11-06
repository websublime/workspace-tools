//! Tests for automatic changeset creation and integration with upgrade application.

#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use super::applier::{apply_with_changeset, extract_affected_packages};
use super::creator::create_changeset_for_upgrades;
use crate::changeset::{ChangesetManager, FileBasedChangesetStorage};
use crate::config::{ChangesetConfig, UpgradeConfig};
use crate::error::UpgradeError;
use crate::types::{DependencyType, VersionBump};
use crate::upgrade::UpgradeSelection;
use crate::upgrade::application::result::UpgradeResult as UpgradeResultType;
use crate::upgrade::application::result::{AppliedUpgrade, ApplySummary};
use crate::upgrade::detection::{DependencyUpgrade, PackageUpgrades, VersionInfo};
use crate::upgrade::registry::UpgradeType;
use chrono::Utc;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::FileSystemManager;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

async fn setup_test_repo() -> (TempDir, PathBuf, Repo) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().to_path_buf();
    let repo_path_str = repo_path.to_str().expect("Invalid path");

    // Initialize git repo
    let repo = Repo::create(repo_path_str).expect("Failed to create git repo");

    // Configure git
    repo.config("user.name", "Test User").expect("Failed to set git config");
    repo.config("user.email", "test@example.com").expect("Failed to set git config");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test\n").expect("Failed to write file");
    repo.add("README.md").expect("Failed to add file");
    repo.commit("Initial commit").expect("Failed to commit");

    (temp_dir, repo_path, repo)
}

async fn setup_test_workspace() -> (TempDir, PathBuf, Repo) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path().to_path_buf();
    let workspace_str = workspace_root.to_str().expect("Invalid path");

    // Initialize git repo
    let repo = Repo::create(workspace_str).expect("Failed to create git repo");
    repo.config("user.name", "Test User").expect("Failed to set git config");
    repo.config("user.email", "test@example.com").expect("Failed to set git config");

    // Create initial commit
    std::fs::write(workspace_root.join("README.md"), "# Test\n").expect("Failed to write file");
    repo.add("README.md").expect("Failed to add file");
    repo.commit("Initial commit").expect("Failed to commit");

    // Create a package
    let package_dir = workspace_root.join("packages").join("core");
    std::fs::create_dir_all(&package_dir).expect("Failed to create package dir");

    let package_json = serde_json::json!({
        "name": "@myorg/core",
        "version": "1.0.0",
        "dependencies": {
            "lodash": "^4.17.20"
        }
    });
    std::fs::write(
        package_dir.join("package.json"),
        serde_json::to_string_pretty(&package_json).expect("Failed to serialize"),
    )
    .expect("Failed to write package.json");

    (temp_dir, workspace_root, repo)
}

async fn setup_changeset_manager(
    workspace_root: &PathBuf,
) -> ChangesetManager<FileBasedChangesetStorage<FileSystemManager>> {
    let fs = FileSystemManager::new();
    let config = ChangesetConfig::default();
    let repo_path_str = workspace_root.to_str().expect("Invalid path");
    let repo = Repo::open(repo_path_str).ok();

    let storage = FileBasedChangesetStorage::new(
        workspace_root.clone(),
        config.path.clone(),
        config.history_path.clone(),
        fs,
    );
    ChangesetManager::with_storage(storage, workspace_root, repo, config)
}

fn create_test_upgrades(workspace_root: &Path) -> Vec<PackageUpgrades> {
    vec![PackageUpgrades {
        package_name: "@myorg/core".to_string(),
        package_path: workspace_root.join("packages").join("core"),
        current_version: Some("1.0.0".to_string()),
        upgrades: vec![DependencyUpgrade {
            name: "lodash".to_string(),
            dependency_type: DependencyType::Regular,
            current_version: "4.17.20".to_string(),
            latest_version: "4.17.21".to_string(),
            upgrade_type: UpgradeType::Patch,
            registry_url: "https://registry.npmjs.org".to_string(),
            version_info: VersionInfo {
                available_versions: vec!["4.17.20".to_string(), "4.17.21".to_string()],
                latest_stable: "4.17.21".to_string(),
                latest_prerelease: None,
                deprecated: None,
                published_at: Some(Utc::now()),
            },
        }],
    }]
}

// ============================================================================
// Creator Tests
// ============================================================================

#[tokio::test]
async fn test_create_new_changeset() {
    let (_temp, repo_path, repo) = setup_test_repo().await;

    // Create a feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout branch");

    let manager = setup_changeset_manager(&repo_path).await;

    let mut packages = HashSet::new();
    packages.insert("@myorg/core".to_string());
    packages.insert("@myorg/utils".to_string());

    // Create changeset
    let result = create_changeset_for_upgrades(&manager, packages, &repo_path, "patch").await;

    assert!(result.is_ok());
    let changeset_id = result.expect("Failed to create changeset");
    assert!(changeset_id.is_some());
    assert_eq!(changeset_id.expect("No changeset ID"), "feature/upgrade-deps");

    // Verify changeset was created
    let changeset = manager.load("feature/upgrade-deps").await.expect("Failed to load");
    assert_eq!(changeset.branch, "feature/upgrade-deps");
    assert_eq!(changeset.bump, VersionBump::Patch);
    assert_eq!(changeset.packages.len(), 2);
    assert!(changeset.packages.contains(&"@myorg/core".to_string()));
    assert!(changeset.packages.contains(&"@myorg/utils".to_string()));
}

#[tokio::test]
async fn test_update_existing_changeset() {
    let (_temp, repo_path, repo) = setup_test_repo().await;

    // Create a feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout branch");

    let manager = setup_changeset_manager(&repo_path).await;

    // Create initial changeset
    let mut packages1 = HashSet::new();
    packages1.insert("@myorg/core".to_string());

    create_changeset_for_upgrades(&manager, packages1, &repo_path, "patch")
        .await
        .expect("Failed to create changeset");

    // Update with more packages
    let mut packages2 = HashSet::new();
    packages2.insert("@myorg/utils".to_string());
    packages2.insert("@myorg/cli".to_string());

    let result = create_changeset_for_upgrades(&manager, packages2, &repo_path, "patch").await;

    assert!(result.is_ok());

    // Verify changeset was updated
    let changeset = manager.load("feature/upgrade-deps").await.expect("Failed to load");
    assert_eq!(changeset.packages.len(), 3);
    assert!(changeset.packages.contains(&"@myorg/core".to_string()));
    assert!(changeset.packages.contains(&"@myorg/utils".to_string()));
    assert!(changeset.packages.contains(&"@myorg/cli".to_string()));
}

#[tokio::test]
async fn test_no_duplicate_packages() {
    let (_temp, repo_path, repo) = setup_test_repo().await;

    // Create a feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout branch");

    let manager = setup_changeset_manager(&repo_path).await;

    // Create initial changeset
    let mut packages1 = HashSet::new();
    packages1.insert("@myorg/core".to_string());

    create_changeset_for_upgrades(&manager, packages1, &repo_path, "patch")
        .await
        .expect("Failed to create changeset");

    // Update with overlapping packages
    let mut packages2 = HashSet::new();
    packages2.insert("@myorg/core".to_string()); // Duplicate
    packages2.insert("@myorg/utils".to_string());

    create_changeset_for_upgrades(&manager, packages2, &repo_path, "patch")
        .await
        .expect("Failed to update changeset");

    // Verify no duplicates
    let changeset = manager.load("feature/upgrade-deps").await.expect("Failed to load");
    assert_eq!(changeset.packages.len(), 2);
    assert!(changeset.packages.contains(&"@myorg/core".to_string()));
    assert!(changeset.packages.contains(&"@myorg/utils".to_string()));
}

#[tokio::test]
async fn test_empty_packages() {
    let (_temp, repo_path, _repo) = setup_test_repo().await;
    let manager = setup_changeset_manager(&repo_path).await;

    let packages = HashSet::new();

    // Should return None for empty packages
    let result = create_changeset_for_upgrades(&manager, packages, &repo_path, "patch").await;

    assert!(result.is_ok());
    assert!(result.expect("Failed").is_none());
}

#[tokio::test]
async fn test_invalid_bump_type() {
    let (_temp, repo_path, repo) = setup_test_repo().await;

    // Create a feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout branch");

    let manager = setup_changeset_manager(&repo_path).await;

    let mut packages = HashSet::new();
    packages.insert("@myorg/core".to_string());

    // Should fail with invalid bump type
    let result = create_changeset_for_upgrades(&manager, packages, &repo_path, "invalid").await;

    assert!(result.is_err());
    match result {
        Err(UpgradeError::ChangesetCreationFailed { reason }) => {
            assert!(reason.contains("Invalid bump type"));
        }
        _ => panic!("Expected ChangesetCreation error"),
    }
}

#[tokio::test]
async fn test_different_bump_types() {
    let (_temp, repo_path, repo) = setup_test_repo().await;
    let manager = setup_changeset_manager(&repo_path).await;

    // Test patch
    repo.create_branch("feature/patch").expect("Failed to create branch");
    repo.checkout("feature/patch").expect("Failed to checkout branch");

    let mut packages = HashSet::new();
    packages.insert("@myorg/core".to_string());

    create_changeset_for_upgrades(&manager, packages.clone(), &repo_path, "patch")
        .await
        .expect("Failed to create changeset");

    let changeset = manager.load("feature/patch").await.expect("Failed to load");
    assert_eq!(changeset.bump, VersionBump::Patch);

    // Test minor
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature/minor").expect("Failed to create branch");
    repo.checkout("feature/minor").expect("Failed to checkout branch");

    create_changeset_for_upgrades(&manager, packages.clone(), &repo_path, "minor")
        .await
        .expect("Failed to create changeset");

    let changeset = manager.load("feature/minor").await.expect("Failed to load");
    assert_eq!(changeset.bump, VersionBump::Minor);

    // Test major
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature/major").expect("Failed to create branch");
    repo.checkout("feature/major").expect("Failed to checkout branch");

    create_changeset_for_upgrades(&manager, packages, &repo_path, "major")
        .await
        .expect("Failed to create changeset");

    let changeset = manager.load("feature/major").await.expect("Failed to load");
    assert_eq!(changeset.bump, VersionBump::Major);
}

#[tokio::test]
async fn test_packages_sorted() {
    let (_temp, repo_path, repo) = setup_test_repo().await;

    // Create a feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout branch");

    let manager = setup_changeset_manager(&repo_path).await;

    // Create with packages in random order
    let mut packages = HashSet::new();
    packages.insert("zebra".to_string());
    packages.insert("apple".to_string());
    packages.insert("mango".to_string());

    create_changeset_for_upgrades(&manager, packages, &repo_path, "patch")
        .await
        .expect("Failed to create changeset");

    // Verify packages are sorted
    let changeset = manager.load("feature/upgrade-deps").await.expect("Failed to load");
    let sorted: Vec<String> = changeset.packages.clone();
    let mut expected = sorted.clone();
    expected.sort();
    assert_eq!(sorted, expected);
}

// ============================================================================
// Applier Tests
// ============================================================================

#[tokio::test]
async fn test_apply_with_changeset_enabled() {
    let (_temp, workspace_root, repo) = setup_test_workspace().await;

    // Create feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout");

    let manager = setup_changeset_manager(&workspace_root).await;
    let fs = FileSystemManager::new();
    let config = UpgradeConfig::default();
    assert!(config.auto_changeset); // Should be enabled by default

    let upgrades = create_test_upgrades(&workspace_root);
    let selection = UpgradeSelection::all();

    let result = apply_with_changeset(
        upgrades,
        selection,
        false,
        &workspace_root,
        &config,
        Some(&manager),
        &fs,
    )
    .await
    .expect("Failed to apply with changeset");

    // Verify upgrades were applied
    assert_eq!(result.applied.len(), 1);
    assert!(!result.dry_run);

    // Verify changeset was created
    assert!(result.changeset_id.is_some());
    assert_eq!(result.changeset_id.expect("No changeset ID"), "feature/upgrade-deps");

    // Verify changeset contents
    let changeset = manager.load("feature/upgrade-deps").await.expect("Failed to load changeset");
    assert_eq!(changeset.branch, "feature/upgrade-deps");
    assert_eq!(changeset.bump, VersionBump::Patch);
    assert!(changeset.packages.contains(&"@myorg/core".to_string()));
}

#[tokio::test]
async fn test_apply_with_changeset_disabled() {
    let (_temp, workspace_root, repo) = setup_test_workspace().await;

    // Create feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout");

    let manager = setup_changeset_manager(&workspace_root).await;
    let fs = FileSystemManager::new();
    let config = UpgradeConfig { auto_changeset: false, ..UpgradeConfig::default() };

    let upgrades = create_test_upgrades(&workspace_root);
    let selection = UpgradeSelection::all();

    let result = apply_with_changeset(
        upgrades,
        selection,
        false,
        &workspace_root,
        &config,
        Some(&manager),
        &fs,
    )
    .await
    .expect("Failed to apply with changeset");

    // Verify upgrades were applied
    assert_eq!(result.applied.len(), 1);

    // Verify changeset was NOT created
    assert!(result.changeset_id.is_none());
}

#[tokio::test]
async fn test_apply_with_changeset_dry_run() {
    let (_temp, workspace_root, repo) = setup_test_workspace().await;

    // Create feature branch
    repo.create_branch("feature/upgrade-deps").expect("Failed to create branch");
    repo.checkout("feature/upgrade-deps").expect("Failed to checkout");

    let manager = setup_changeset_manager(&workspace_root).await;
    let fs = FileSystemManager::new();
    let config = UpgradeConfig::default();

    let upgrades = create_test_upgrades(&workspace_root);
    let selection = UpgradeSelection::all();

    let result = apply_with_changeset(
        upgrades,
        selection,
        true, // Dry run
        &workspace_root,
        &config,
        Some(&manager),
        &fs,
    )
    .await
    .expect("Failed to apply with changeset");

    // Verify this was a dry run
    assert!(result.dry_run);

    // Verify changeset was NOT created (dry run)
    assert!(result.changeset_id.is_none());
}

#[tokio::test]
async fn test_apply_with_changeset_no_manager() {
    let (_temp, workspace_root, _repo) = setup_test_workspace().await;

    let fs = FileSystemManager::new();
    let config = UpgradeConfig::default();

    let upgrades = create_test_upgrades(&workspace_root);
    let selection = UpgradeSelection::all();

    // No manager provided
    let result = apply_with_changeset::<_, FileBasedChangesetStorage<FileSystemManager>>(
        upgrades,
        selection,
        false,
        &workspace_root,
        &config,
        None, // No manager
        &fs,
    )
    .await
    .expect("Failed to apply with changeset");

    // Verify upgrades were applied
    assert_eq!(result.applied.len(), 1);

    // Verify changeset was NOT created (no manager)
    assert!(result.changeset_id.is_none());
}

#[tokio::test]
async fn test_extract_affected_packages() {
    let workspace_root = PathBuf::from("/workspace");
    let result = UpgradeResultType::applied(
        vec![
            AppliedUpgrade {
                package_path: workspace_root.join("packages").join("core"),
                dependency_name: "lodash".to_string(),
                dependency_type: DependencyType::Regular,
                old_version: "4.17.20".to_string(),
                new_version: "4.17.21".to_string(),
                upgrade_type: UpgradeType::Patch,
            },
            AppliedUpgrade {
                package_path: workspace_root.join("packages").join("core"),
                dependency_name: "react".to_string(),
                dependency_type: DependencyType::Regular,
                old_version: "17.0.0".to_string(),
                new_version: "18.0.0".to_string(),
                upgrade_type: UpgradeType::Major,
            },
        ],
        vec![],
        None,
        None,
        ApplySummary::new(),
    );

    let packages = extract_affected_packages(&result);

    // Should have 1 unique package
    assert_eq!(packages.len(), 1);
    assert!(packages.contains("core"));
}

#[tokio::test]
async fn test_different_changeset_bump_types() {
    let (_temp, workspace_root, repo) = setup_test_workspace().await;

    // Create feature branch
    repo.create_branch("feature/minor-upgrade").expect("Failed to create branch");
    repo.checkout("feature/minor-upgrade").expect("Failed to checkout");

    let manager = setup_changeset_manager(&workspace_root).await;
    let fs = FileSystemManager::new();
    let config = UpgradeConfig { changeset_bump: "minor".to_string(), ..UpgradeConfig::default() };

    let upgrades = create_test_upgrades(&workspace_root);
    let selection = UpgradeSelection::all();

    let _result = apply_with_changeset(
        upgrades,
        selection,
        false,
        &workspace_root,
        &config,
        Some(&manager),
        &fs,
    )
    .await
    .expect("Failed to apply with changeset");

    // Verify changeset was created with minor bump
    let changeset = manager.load("feature/minor-upgrade").await.expect("Failed to load changeset");
    assert_eq!(changeset.bump, VersionBump::Minor);
}
