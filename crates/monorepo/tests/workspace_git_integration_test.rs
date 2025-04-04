use rstest::*;
use std::fs;
use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use tempfile::TempDir;

mod fixtures;

#[rstest]
fn test_git_repo_access(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Use canonicalize to resolve any symlinks in the path
    let canonical_temp_path =
        std::fs::canonicalize(root_path).expect("Failed to canonicalize temp path");

    let options = DiscoveryOptions::default().auto_detect_root(false);

    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Check that Git repo is accessible
    let git_repo = workspace.git_repo().expect("Git repo should be available");

    // Check that the Git repo has the correct path - use canonicalized paths for comparison
    let repo_path =
        std::fs::canonicalize(git_repo.get_repo_path()).expect("Failed to canonicalize repo path");
    assert_eq!(
        repo_path.to_str(),
        canonical_temp_path.to_str(),
        "Git repo path should match workspace root path"
    );
}

#[test]
fn test_workspace_without_git() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Create a minimal workspace without git initialization
    let root_pkg_json = r#"{
        "name": "root",
        "version": "0.0.0",
        "workspaces": ["packages/*"]
    }"#;

    fs::write(root_path.join("package.json"), root_pkg_json)
        .expect("Failed to write root package.json");

    // Create a package
    let pkg_dir = root_path.join("packages/test-pkg");
    fs::create_dir_all(&pkg_dir).expect("Failed to create package dir");

    let pkg_json = r#"{
        "name": "test-package",
        "version": "1.0.0"
    }"#;

    fs::write(pkg_dir.join("package.json"), pkg_json).expect("Failed to write package.json");

    // Discover workspace
    let options = DiscoveryOptions::default().auto_detect_root(false);
    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Git repo should be None
    assert!(workspace.git_repo().is_none(), "Git repo should be None for non-git workspace");
}

#[rstest]
fn test_workspace_with_modified_files(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default().auto_detect_root(false);

    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Get the Git repo
    let git_repo = workspace.git_repo().expect("Git repo should be available");

    // Modify a file in one of the packages
    let foo_index_path = root_path.join("packages/package-foo/index.mjs");

    // Ensure the file exists
    assert!(foo_index_path.exists(), "File to modify doesn't exist");

    // Read original content
    fs::read_to_string(&foo_index_path).expect("Failed to read original file");

    // Write new content
    fs::write(&foo_index_path, "export const foo = 'modified';").expect("Failed to modify file");

    // Get git status
    let status = git_repo.status_porcelain().expect("Failed to get git status");

    // Check for the exact file path in the status output
    let expected_path = "packages/package-foo/index.mjs".to_string();
    assert!(status.contains(&expected_path), "Modified file should be detected in git status");
}

#[rstest]
fn test_workspace_with_multiple_file_changes(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    let options = DiscoveryOptions::default().auto_detect_root(false);

    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    // Get the Git repo
    let git_repo = workspace.git_repo().expect("Git repo should be available");

    // 1. Modify an existing file
    let foo_index_path = root_path.join("packages/package-foo/index.mjs");
    fs::write(&foo_index_path, "export const foo = 'modified';").expect("Failed to modify file");

    // 2. Create a new file
    let new_file_path = root_path.join("packages/package-bar/new-file.js");
    fs::write(&new_file_path, "export const newFile = 'new';").expect("Failed to create new file");

    // 3. Add the new file to git
    git_repo.add("packages/package-bar/new-file.js").expect("Failed to add new file");

    // Get git status
    let status = git_repo.status_porcelain().expect("Failed to get git status");

    // Check for the modified file path
    let modified_path = "packages/package-foo/index.mjs".to_string();
    assert!(status.contains(&modified_path), "Modified file should be detected in git status");

    // Check for the new file path
    let new_path = "packages/package-bar/new-file.js".to_string();
    assert!(status.contains(&new_path), "New file should be detected in git status");

    // We should have exactly 2 changed files
    assert_eq!(status.len(), 2, "Should detect exactly 2 changed files");
}
