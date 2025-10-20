//! Integration tests for FileBasedChangesetStorage with real Git workflows.
//!
//! **What**: Tests the complete changeset storage workflow with real Git operations,
//! simulating actual developer workflows including branch creation, commits, changeset
//! creation, updates, and archiving.
//!
//! **How**: Creates real Git repositories (single-package and monorepo), performs actual
//! Git operations, and verifies that the FileBasedChangesetStorage correctly handles
//! the entire changeset lifecycle.
//!
//! **Why**: To ensure that the storage implementation works correctly in real-world
//! scenarios with actual Git repositories and file operations.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

mod common;

use common::{create_temp_dir, fixtures::MonorepoFixtureBuilder};
use std::collections::HashMap;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::{ChangesetStorage, FileBasedChangesetStorage};
use sublime_pkg_tools::types::{Changeset, ReleaseInfo, VersionBump};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

/// Helper to initialize a Git repository with initial commit
fn init_git_repo(path: &std::path::Path) -> Repo {
    let path_str = path.to_str().expect("Invalid path");
    let repo = Repo::create(path_str).expect("Failed to create repository");

    // Configure git user
    repo.config("Test User", "test@example.com").expect("Failed to configure git");

    repo
}

/// Helper to create and commit a file
async fn create_and_commit_file(
    repo: &Repo,
    fs: &FileSystemManager,
    file_path: &std::path::Path,
    content: &str,
    commit_message: &str,
) -> String {
    let full_path = repo.get_repo_path().join(file_path);

    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        fs.create_dir_all(parent).await.expect("Failed to create directories");
    }

    // Write file
    fs.write_file_string(&full_path, content).await.expect("Failed to write file");

    // Stage and commit
    repo.add(file_path.to_str().expect("Invalid path")).expect("Failed to stage file");

    repo.commit(commit_message).expect("Failed to commit")
}

/// Helper to get all commits on current branch
fn get_commits_since(repo: &Repo, base_ref: &str, head_ref: &str) -> Vec<String> {
    let log = repo.get_commits_between(base_ref, head_ref, &None).expect("Failed to get git log");

    log.into_iter().map(|commit| commit.hash).collect()
}

#[tokio::test]
async fn test_single_package_changeset_workflow() {
    // ==========================================
    // Setup: Create single package repository
    // ==========================================
    let temp_dir = create_temp_dir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();
    let fs = FileSystemManager::new();

    println!("Setting up single-package repository at {:?}", repo_path);

    // Create package.json
    let package_json = serde_json::json!({
        "name": "my-package",
        "version": "1.0.0",
        "description": "Test package"
    });
    let package_json_path = repo_path.join("package.json");
    fs.write_file_string(&package_json_path, &serde_json::to_string_pretty(&package_json).unwrap())
        .await
        .expect("Failed to write package.json");

    // Initialize Git repository
    let repo = init_git_repo(repo_path);

    // Initial commit on main
    let _initial_commit = create_and_commit_file(
        &repo,
        &fs,
        "package.json".as_ref(),
        &package_json.to_string(),
        "Initial commit",
    )
    .await;

    println!("✓ Created initial commit on main branch");

    // ==========================================
    // Step 1: Create feature branch
    // ==========================================
    let feature_branch = "feature/add-authentication";
    repo.create_branch(feature_branch).expect("Failed to create branch");
    repo.checkout(feature_branch).expect("Failed to checkout branch");

    println!("✓ Created and checked out branch: {}", feature_branch);

    // ==========================================
    // Step 2: Make some commits
    // ==========================================
    let commit1 = create_and_commit_file(
        &repo,
        &fs,
        "auth.js".as_ref(),
        "module.exports = { authenticate: () => {} };",
        "feat: add authentication module",
    )
    .await;

    let commit2 = create_and_commit_file(
        &repo,
        &fs,
        "auth.test.js".as_ref(),
        "test('auth works', () => {});",
        "test: add authentication tests",
    )
    .await;

    println!("✓ Created 2 commits on feature branch");
    println!("  - Commit 1: {}", &commit1[..8]);
    println!("  - Commit 2: {}", &commit2[..8]);

    // ==========================================
    // Step 3: Create changeset with commits
    // ==========================================
    let storage = FileBasedChangesetStorage::new(
        repo_path.to_path_buf(),
        ".changesets".to_string(),
        ".changesets/history".to_string(),
        fs.clone(),
    );

    let commits_on_branch = get_commits_since(&repo, "main", feature_branch);
    println!("✓ Found {} commits on feature branch", commits_on_branch.len());

    let mut changeset =
        Changeset::new(feature_branch, VersionBump::Minor, vec!["production".to_string()]);

    // Add the package (in single package, it's the root package)
    changeset.add_package("my-package");

    // Add all commits from the feature branch
    for commit in &commits_on_branch {
        changeset.add_commit(commit);
    }

    storage.save(&changeset).await.expect("Failed to save changeset");

    println!("✓ Created changeset for branch '{}'", feature_branch);
    println!("  - Version bump: {:?}", changeset.bump);
    println!("  - Packages: {:?}", changeset.packages);
    println!("  - Commits: {}", changeset.changes.len());

    // ==========================================
    // Step 4: Verify changeset exists
    // ==========================================
    let exists = storage.exists(feature_branch).await.expect("Failed to check existence");
    assert!(exists, "Changeset should exist");

    let loaded = storage.load(feature_branch).await.expect("Failed to load changeset");
    assert_eq!(loaded.branch, feature_branch);
    assert_eq!(loaded.packages.len(), 1);
    assert_eq!(loaded.changes.len(), 2);
    assert!(loaded.has_commit(&commit1));
    assert!(loaded.has_commit(&commit2));

    println!("✓ Verified changeset exists and has correct data");

    // ==========================================
    // Step 5: Make more changes and commit
    // ==========================================
    let commit3 = create_and_commit_file(
        &repo,
        &fs,
        "README.md".as_ref(),
        "# Authentication Package\n\nAdds auth support.",
        "docs: add README for authentication",
    )
    .await;

    println!("✓ Created additional commit: {}", &commit3[..8]);

    // ==========================================
    // Step 6: Update changeset with new commit
    // ==========================================
    let all_commits = get_commits_since(&repo, "main", feature_branch);
    let mut updated_changeset =
        storage.load(feature_branch).await.expect("Failed to load changeset");

    // Check if all commits are present
    let has_all_commits = all_commits.iter().all(|commit| updated_changeset.has_commit(commit));

    if !has_all_commits {
        println!("  Changeset missing commits, updating...");
        for commit in &all_commits {
            if !updated_changeset.has_commit(commit) {
                updated_changeset.add_commit(commit);
                println!("    + Added commit: {}", &commit[..8]);
            }
        }

        updated_changeset.touch(); // Update timestamp
        storage.save(&updated_changeset).await.expect("Failed to update changeset");
        println!("✓ Updated changeset with new commits");
    }

    // Verify update
    let final_changeset = storage.load(feature_branch).await.expect("Failed to load changeset");
    assert_eq!(final_changeset.changes.len(), 3);
    assert!(final_changeset.has_commit(&commit3));

    println!("✓ Verified changeset has all {} commits", final_changeset.changes.len());

    // ==========================================
    // Step 7: Merge branch (simulate)
    // ==========================================
    // In real workflow, we would merge, but for testing we'll simulate
    // the release process directly

    // Create release info
    let mut versions = HashMap::new();
    versions.insert("my-package".to_string(), "1.1.0".to_string());

    let release_info =
        ReleaseInfo::new("ci-bot@example.com".to_string(), "release-abc123".to_string(), versions);

    println!("✓ Simulating merge and release:");
    println!("  - Applied by: {}", release_info.applied_by);
    println!("  - Version: my-package -> 1.1.0");

    // ==========================================
    // Step 8: Archive changeset
    // ==========================================
    storage
        .archive(&final_changeset, release_info.clone())
        .await
        .expect("Failed to archive changeset");

    println!("✓ Archived changeset");

    // ==========================================
    // Step 9: Verify archive
    // ==========================================
    // Changeset should no longer be in pending
    let still_exists = storage.exists(feature_branch).await.expect("Failed to check existence");
    assert!(!still_exists, "Changeset should not exist in pending");

    // Changeset should be in archive
    let archived =
        storage.load_archived(feature_branch).await.expect("Failed to load archived changeset");
    assert_eq!(archived.changeset.branch, feature_branch);
    assert_eq!(archived.changeset.changes.len(), 3);
    assert_eq!(archived.release_info.applied_by, "ci-bot@example.com");
    assert_eq!(archived.release_info.versions.get("my-package").unwrap(), "1.1.0");

    println!("✓ Verified changeset is archived:");
    println!("  - Branch: {}", archived.changeset.branch);
    println!("  - Commits: {}", archived.changeset.changes.len());
    println!("  - Released at: {}", archived.release_info.applied_at);

    println!("\n✅ Single package workflow test completed successfully!");
}

#[tokio::test]
async fn test_monorepo_changeset_workflow() {
    // ==========================================
    // Setup: Create monorepo repository
    // ==========================================
    let temp_dir = create_temp_dir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();
    let fs = FileSystemManager::new();

    println!("Setting up monorepo at {:?}", repo_path);

    // Create monorepo structure
    let fixture = MonorepoFixtureBuilder::new("my-monorepo")
        .add_package("packages/core", "@myorg/core", "1.0.0")
        .add_package("packages/utils", "@myorg/utils", "1.0.0")
        .add_package("packages/api", "@myorg/api", "2.0.0")
        .build();

    fixture.write_to_dir(repo_path).expect("Failed to write fixture");

    println!("✓ Created monorepo with 3 packages:");
    println!("  - @myorg/core (1.0.0)");
    println!("  - @myorg/utils (1.0.0)");
    println!("  - @myorg/api (2.0.0)");

    // Initialize Git repository
    let repo = init_git_repo(repo_path);

    // Initial commit on main
    repo.add_all().expect("Failed to stage all files");
    let _initial_commit = repo.commit("Initial commit: Setup monorepo").expect("Failed to commit");

    println!("✓ Created initial commit on main branch");

    // ==========================================
    // Step 1: Create feature branch
    // ==========================================
    let feature_branch = "feature/improve-utils";
    repo.create_branch(feature_branch).expect("Failed to create branch");
    repo.checkout(feature_branch).expect("Failed to checkout branch");

    println!("✓ Created and checked out branch: {}", feature_branch);

    // ==========================================
    // Step 2: Make changes to utils package
    // ==========================================
    let commit1 = create_and_commit_file(
        &repo,
        &fs,
        "packages/utils/helpers.js".as_ref(),
        "module.exports = { formatDate: () => {} };",
        "feat(utils): add date formatting helper",
    )
    .await;

    let commit2 = create_and_commit_file(
        &repo,
        &fs,
        "packages/utils/helpers.test.js".as_ref(),
        "test('formatDate works', () => {});",
        "test(utils): add tests for date formatting",
    )
    .await;

    println!("✓ Created 2 commits affecting @myorg/utils");
    println!("  - Commit 1: {}", &commit1[..8]);
    println!("  - Commit 2: {}", &commit2[..8]);

    // ==========================================
    // Step 3: Make changes to core package
    // ==========================================
    let commit3 = create_and_commit_file(
        &repo,
        &fs,
        "packages/core/config.js".as_ref(),
        "module.exports = { debug: false };",
        "feat(core): add configuration module",
    )
    .await;

    println!("✓ Created 1 commit affecting @myorg/core");
    println!("  - Commit 3: {}", &commit3[..8]);

    // ==========================================
    // Step 4: Create changeset with all commits
    // ==========================================
    let storage = FileBasedChangesetStorage::new(
        repo_path.to_path_buf(),
        ".changesets".to_string(),
        ".changesets/history".to_string(),
        fs.clone(),
    );

    let commits_on_branch = get_commits_since(&repo, "main", feature_branch);
    println!("✓ Found {} commits on feature branch", commits_on_branch.len());

    let mut changeset = Changeset::new(
        feature_branch,
        VersionBump::Minor,
        vec!["staging".to_string(), "production".to_string()],
    );

    // In a real scenario, we would analyze which packages were affected
    // For this test, we know utils and core were changed
    changeset.add_package("@myorg/utils");
    changeset.add_package("@myorg/core");

    // Add all commits
    for commit in &commits_on_branch {
        changeset.add_commit(commit);
    }

    storage.save(&changeset).await.expect("Failed to save changeset");

    println!("✓ Created changeset for branch '{}'", feature_branch);
    println!("  - Version bump: {:?}", changeset.bump);
    println!("  - Packages affected: {:?}", changeset.packages);
    println!("  - Commits: {}", changeset.changes.len());
    println!("  - Environments: {:?}", changeset.environments);

    // ==========================================
    // Step 5: Verify changeset exists
    // ==========================================
    let exists = storage.exists(feature_branch).await.expect("Failed to check existence");
    assert!(exists, "Changeset should exist");

    let loaded = storage.load(feature_branch).await.expect("Failed to load changeset");
    assert_eq!(loaded.branch, feature_branch);
    assert_eq!(loaded.packages.len(), 2);
    assert!(loaded.has_package("@myorg/utils"));
    assert!(loaded.has_package("@myorg/core"));
    assert_eq!(loaded.changes.len(), 3);

    println!("✓ Verified changeset exists and has correct data");

    // ==========================================
    // Step 6: Make additional changes
    // ==========================================
    let commit4 = create_and_commit_file(
        &repo,
        &fs,
        "packages/api/routes.js".as_ref(),
        "module.exports = { healthCheck: () => {} };",
        "feat(api): add health check endpoint",
    )
    .await;

    println!("✓ Created additional commit affecting @myorg/api");
    println!("  - Commit 4: {}", &commit4[..8]);

    // ==========================================
    // Step 7: Update changeset
    // ==========================================
    let all_commits = get_commits_since(&repo, "main", feature_branch);
    let mut updated_changeset =
        storage.load(feature_branch).await.expect("Failed to load changeset");

    // Add new package
    if !updated_changeset.has_package("@myorg/api") {
        updated_changeset.add_package("@myorg/api");
        println!("  + Added package: @myorg/api");
    }

    // Add new commits
    let has_all_commits = all_commits.iter().all(|commit| updated_changeset.has_commit(commit));
    if !has_all_commits {
        for commit in &all_commits {
            if !updated_changeset.has_commit(commit) {
                updated_changeset.add_commit(commit);
                println!("    + Added commit: {}", &commit[..8]);
            }
        }
    }

    updated_changeset.touch();
    storage.save(&updated_changeset).await.expect("Failed to update changeset");

    println!("✓ Updated changeset with new package and commits");

    // Verify update
    let final_changeset = storage.load(feature_branch).await.expect("Failed to load changeset");
    assert_eq!(final_changeset.packages.len(), 3);
    assert_eq!(final_changeset.changes.len(), 4);
    assert!(final_changeset.has_package("@myorg/api"));
    assert!(final_changeset.has_commit(&commit4));

    println!("✓ Verified changeset has all data:");
    println!("  - Packages: {}", final_changeset.packages.len());
    println!("  - Commits: {}", final_changeset.changes.len());

    // ==========================================
    // Step 8: List all pending changesets
    // ==========================================
    let pending = storage.list_pending().await.expect("Failed to list pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].branch, feature_branch);

    println!("✓ Listed pending changesets: {}", pending.len());

    // ==========================================
    // Step 9: Merge and archive
    // ==========================================
    let mut versions = HashMap::new();
    versions.insert("@myorg/utils".to_string(), "1.1.0".to_string());
    versions.insert("@myorg/core".to_string(), "1.1.0".to_string());
    versions.insert("@myorg/api".to_string(), "2.1.0".to_string());

    let release_info = ReleaseInfo::new(
        "release-pipeline@example.com".to_string(),
        "merge-commit-xyz789".to_string(),
        versions,
    );

    println!("✓ Simulating merge and release:");
    println!("  - @myorg/utils: 1.0.0 -> 1.1.0");
    println!("  - @myorg/core: 1.0.0 -> 1.1.0");
    println!("  - @myorg/api: 2.0.0 -> 2.1.0");

    storage
        .archive(&final_changeset, release_info.clone())
        .await
        .expect("Failed to archive changeset");

    println!("✓ Archived changeset");

    // ==========================================
    // Step 10: Verify archive
    // ==========================================
    let still_pending = storage.exists(feature_branch).await.expect("Failed to check existence");
    assert!(!still_pending, "Changeset should not be in pending");

    let archived =
        storage.load_archived(feature_branch).await.expect("Failed to load archived changeset");
    assert_eq!(archived.changeset.branch, feature_branch);
    assert_eq!(archived.changeset.packages.len(), 3);
    assert_eq!(archived.changeset.changes.len(), 4);
    assert_eq!(archived.release_info.versions.len(), 3);

    println!("✓ Verified archived changeset:");
    println!("  - Branch: {}", archived.changeset.branch);
    println!("  - Packages released: {}", archived.release_info.versions.len());
    println!("  - Commits: {}", archived.changeset.changes.len());
    println!("  - Released at: {}", archived.release_info.applied_at);

    // Verify no pending changesets
    let final_pending = storage.list_pending().await.expect("Failed to list pending");
    assert_eq!(final_pending.len(), 0, "Should have no pending changesets");

    // Verify one archived changeset
    let archived_list = storage.list_archived().await.expect("Failed to list archived");
    assert_eq!(archived_list.len(), 1);

    println!("✓ Final state:");
    println!("  - Pending changesets: {}", final_pending.len());
    println!("  - Archived changesets: {}", archived_list.len());

    println!("\n✅ Monorepo workflow test completed successfully!");
}

#[tokio::test]
async fn test_multiple_changesets_lifecycle() {
    // Test managing multiple changesets simultaneously
    let temp_dir = create_temp_dir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();
    let fs = FileSystemManager::new();

    println!("Setting up repository for multiple changesets test");

    // Create simple package.json
    let package_json = r#"{"name": "test-pkg", "version": "1.0.0"}"#;
    fs.write_file_string(&repo_path.join("package.json"), package_json)
        .await
        .expect("Failed to write package.json");

    let repo = init_git_repo(repo_path);
    repo.add_all().expect("Failed to stage");
    repo.commit("Initial commit").expect("Failed to commit");

    let storage = FileBasedChangesetStorage::new(
        repo_path.to_path_buf(),
        ".changesets".to_string(),
        ".changesets/history".to_string(),
        fs.clone(),
    );

    // Create multiple feature branches with changesets
    let branches = vec!["feature/a", "feature/b", "bugfix/critical"];

    for branch in &branches {
        repo.create_branch(branch).expect("Failed to create branch");
        repo.checkout(branch).expect("Failed to checkout");

        // Make a commit
        let _commit = create_and_commit_file(
            &repo,
            &fs,
            format!("{}.txt", branch.replace('/', "-")).as_ref(),
            "test content",
            &format!("Add file for {}", branch),
        )
        .await;

        // Create changeset
        let bump =
            if branch.starts_with("bugfix") { VersionBump::Patch } else { VersionBump::Minor };
        let mut changeset = Changeset::new(*branch, bump, vec!["production".to_string()]);
        changeset.add_package("test-pkg");

        storage.save(&changeset).await.expect("Failed to save changeset");
        println!("✓ Created changeset for branch: {}", branch);
    }

    // Verify all changesets exist
    let pending = storage.list_pending().await.expect("Failed to list pending");
    assert_eq!(pending.len(), 3);
    println!("✓ All {} changesets created", pending.len());

    // Archive one changeset
    let to_archive = pending.iter().find(|c| c.branch == "bugfix/critical").unwrap();
    let mut versions = HashMap::new();
    versions.insert("test-pkg".to_string(), "1.0.1".to_string());
    let release_info =
        ReleaseInfo::new("bot@example.com".to_string(), "commit1".to_string(), versions);

    storage.archive(to_archive, release_info).await.expect("Failed to archive");
    println!("✓ Archived changeset: bugfix/critical");

    // Verify counts
    let remaining_pending = storage.list_pending().await.expect("Failed to list pending");
    let archived = storage.list_archived().await.expect("Failed to list archived");

    assert_eq!(remaining_pending.len(), 2);
    assert_eq!(archived.len(), 1);

    println!("✓ Final state:");
    println!("  - Pending: {}", remaining_pending.len());
    println!("  - Archived: {}", archived.len());

    println!("\n✅ Multiple changesets lifecycle test completed!");
}
