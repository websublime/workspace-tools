//! Tests for the changeset storage trait.
//!
//! **What**: Provides comprehensive tests for the `ChangesetStorage` trait using a mock
//! in-memory implementation to verify the storage contract and behavior.
//!
//! **How**: Implements an in-memory storage backend using `HashMap` wrapped in `Arc<RwLock<T>>`
//! for thread safety. Tests cover all trait methods including save, load, exists, delete,
//! list operations, and archiving functionality.
//!
//! **Why**: To ensure that any storage implementation adheres to the expected behavior
//! and contract defined by the `ChangesetStorage` trait, and to provide a reference
//! implementation for testing purposes.

#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::changeset::ChangesetStorage;
use crate::error::{ChangesetError, ChangesetResult};
use crate::types::{ArchivedChangeset, Changeset, ReleaseInfo, VersionBump};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper function to create a HashMap from a vector of tuples for ReleaseInfo.
fn versions_map(versions: Vec<(String, String)>) -> HashMap<String, String> {
    versions.into_iter().collect()
}

/// Mock in-memory storage implementation for testing.
///
/// This implementation stores changesets in memory using hash maps,
/// providing a simple and fast storage backend for testing purposes.
struct MockStorage {
    pending: Arc<RwLock<HashMap<String, Changeset>>>,
    archived: Arc<RwLock<HashMap<String, ArchivedChangeset>>>,
}

impl MockStorage {
    /// Creates a new empty mock storage instance.
    fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            archived: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ChangesetStorage for MockStorage {
    async fn save(&self, changeset: &Changeset) -> ChangesetResult<()> {
        let mut pending = self.pending.write().await;
        pending.insert(changeset.branch.clone(), changeset.clone());
        Ok(())
    }

    async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
        let pending = self.pending.read().await;
        pending
            .get(branch)
            .cloned()
            .ok_or_else(|| ChangesetError::NotFound { branch: branch.to_string() })
    }

    async fn exists(&self, branch: &str) -> ChangesetResult<bool> {
        let pending = self.pending.read().await;
        Ok(pending.contains_key(branch))
    }

    async fn delete(&self, branch: &str) -> ChangesetResult<()> {
        let mut pending = self.pending.write().await;
        pending.remove(branch);
        Ok(())
    }

    async fn list_pending(&self) -> ChangesetResult<Vec<Changeset>> {
        let pending = self.pending.read().await;
        Ok(pending.values().cloned().collect())
    }

    async fn archive(
        &self,
        changeset: &Changeset,
        release_info: ReleaseInfo,
    ) -> ChangesetResult<()> {
        let mut pending = self.pending.write().await;
        let mut archived = self.archived.write().await;

        // Remove from pending
        pending.remove(&changeset.branch);

        // Add to archived
        let archived_changeset = ArchivedChangeset::new(changeset.clone(), release_info);
        archived.insert(changeset.branch.clone(), archived_changeset);

        Ok(())
    }

    async fn load_archived(&self, branch: &str) -> ChangesetResult<ArchivedChangeset> {
        let archived = self.archived.read().await;
        archived
            .get(branch)
            .cloned()
            .ok_or_else(|| ChangesetError::NotFound { branch: branch.to_string() })
    }

    async fn list_archived(&self) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let archived = self.archived.read().await;
        Ok(archived.values().cloned().collect())
    }
}

#[tokio::test]
async fn test_save_and_load() {
    let storage = MockStorage::new();
    let changeset =
        Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);

    // Save the changeset
    let result = storage.save(&changeset).await;
    assert!(result.is_ok());

    // Load it back
    let loaded = storage.load("feature/test").await;
    assert!(loaded.is_ok());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.branch, "feature/test");
    assert_eq!(loaded.bump, VersionBump::Minor);
    assert_eq!(loaded.environments, vec!["production"]);
}

#[tokio::test]
async fn test_load_nonexistent() {
    let storage = MockStorage::new();

    // Try to load a changeset that doesn't exist
    let result = storage.load("nonexistent").await;
    assert!(result.is_err());
    match result {
        Err(ChangesetError::NotFound { branch }) => {
            assert_eq!(branch, "nonexistent");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_exists() {
    let storage = MockStorage::new();
    let changeset =
        Changeset::new("feature/exists-test", VersionBump::Patch, vec!["staging".to_string()]);

    // Initially should not exist
    let exists = storage.exists("feature/exists-test").await.unwrap();
    assert!(!exists);

    // Save and check again
    storage.save(&changeset).await.unwrap();
    let exists = storage.exists("feature/exists-test").await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_delete() {
    let storage = MockStorage::new();
    let changeset =
        Changeset::new("feature/delete-test", VersionBump::Major, vec!["production".to_string()]);

    // Save the changeset
    storage.save(&changeset).await.unwrap();
    assert!(storage.exists("feature/delete-test").await.unwrap());

    // Delete it
    let result = storage.delete("feature/delete-test").await;
    assert!(result.is_ok());

    // Verify it's gone
    assert!(!storage.exists("feature/delete-test").await.unwrap());
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let storage = MockStorage::new();

    // Deleting a nonexistent changeset should succeed (idempotent)
    let result = storage.delete("nonexistent").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_changeset() {
    let storage = MockStorage::new();
    let mut changeset =
        Changeset::new("feature/update-test", VersionBump::Minor, vec!["production".to_string()]);

    // Save initial version
    storage.save(&changeset).await.unwrap();

    // Update and save again
    changeset.add_package("package1");
    changeset.set_bump(VersionBump::Major);
    storage.save(&changeset).await.unwrap();

    // Load and verify updates
    let loaded = storage.load("feature/update-test").await.unwrap();
    assert_eq!(loaded.packages.len(), 1);
    assert_eq!(loaded.bump, VersionBump::Major);
}

#[tokio::test]
async fn test_list_pending_empty() {
    let storage = MockStorage::new();

    let pending = storage.list_pending().await.unwrap();
    assert_eq!(pending.len(), 0);
}

#[tokio::test]
async fn test_list_pending_multiple() {
    let storage = MockStorage::new();

    // Create and save multiple changesets
    let changeset1 =
        Changeset::new("feature/one", VersionBump::Minor, vec!["production".to_string()]);
    let changeset2 = Changeset::new("feature/two", VersionBump::Patch, vec!["staging".to_string()]);
    let changeset3 = Changeset::new(
        "feature/three",
        VersionBump::Major,
        vec!["production".to_string(), "staging".to_string()],
    );

    storage.save(&changeset1).await.unwrap();
    storage.save(&changeset2).await.unwrap();
    storage.save(&changeset3).await.unwrap();

    // List all pending
    let pending = storage.list_pending().await.unwrap();
    assert_eq!(pending.len(), 3);

    // Verify all branches are present
    let branches: Vec<String> = pending.iter().map(|c| c.branch.clone()).collect();
    assert!(branches.contains(&"feature/one".to_string()));
    assert!(branches.contains(&"feature/two".to_string()));
    assert!(branches.contains(&"feature/three".to_string()));
}

#[tokio::test]
async fn test_archive() {
    let storage = MockStorage::new();
    let mut changeset =
        Changeset::new("feature/archive-test", VersionBump::Minor, vec!["production".to_string()]);
    changeset.add_package("package1");
    changeset.add_package("package2");

    // Save the changeset
    storage.save(&changeset).await.unwrap();
    assert!(storage.exists("feature/archive-test").await.unwrap());

    // Create release info
    let release_info = ReleaseInfo::new(
        "test-user@example.com".to_string(),
        "abc123def456".to_string(),
        versions_map(vec![
            ("package1".to_string(), "1.2.0".to_string()),
            ("package2".to_string(), "2.0.0".to_string()),
        ]),
    );

    // Archive it
    let result = storage.archive(&changeset, release_info).await;
    assert!(result.is_ok());

    // Verify it's no longer in pending
    assert!(!storage.exists("feature/archive-test").await.unwrap());

    // Verify it's in archived
    let archived = storage.load_archived("feature/archive-test").await;
    assert!(archived.is_ok());
    let archived = archived.unwrap();
    assert_eq!(archived.changeset.branch, "feature/archive-test");
    assert_eq!(archived.release_info.applied_by, "test-user@example.com");
    assert_eq!(archived.release_info.versions.len(), 2);
}

#[tokio::test]
async fn test_load_archived_nonexistent() {
    let storage = MockStorage::new();

    let result = storage.load_archived("nonexistent").await;
    assert!(result.is_err());
    match result {
        Err(ChangesetError::NotFound { branch }) => {
            assert_eq!(branch, "nonexistent");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_list_archived_empty() {
    let storage = MockStorage::new();

    let archived = storage.list_archived().await.unwrap();
    assert_eq!(archived.len(), 0);
}

#[tokio::test]
async fn test_list_archived_multiple() {
    let storage = MockStorage::new();

    // Create and archive multiple changesets
    for i in 1..=3 {
        let mut changeset = Changeset::new(
            format!("feature/archived-{}", i),
            VersionBump::Minor,
            vec!["production".to_string()],
        );
        changeset.add_package(format!("package{}", i));

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            format!("user{}@example.com", i),
            format!("commit{}", i),
            versions_map(vec![(format!("package{}", i), format!("1.{}.0", i))]),
        );

        storage.archive(&changeset, release_info).await.unwrap();
    }

    // List all archived
    let archived = storage.list_archived().await.unwrap();
    assert_eq!(archived.len(), 3);

    // Verify all are archived
    for i in 1..=3 {
        let branch = format!("feature/archived-{}", i);
        let found = archived.iter().any(|a| a.changeset.branch == branch);
        assert!(found, "Expected to find archived changeset for {}", branch);
    }
}

#[tokio::test]
async fn test_concurrent_access() {
    let storage = Arc::new(MockStorage::new());

    // Spawn multiple tasks that save changesets concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let storage_clone = Arc::clone(&storage);
        let handle = tokio::spawn(async move {
            let changeset = Changeset::new(
                format!("feature/concurrent-{}", i),
                VersionBump::Patch,
                vec!["production".to_string()],
            );
            storage_clone.save(&changeset).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all changesets were saved
    let pending = storage.list_pending().await.unwrap();
    assert_eq!(pending.len(), 10);
}

#[tokio::test]
async fn test_save_with_packages_and_commits() {
    let storage = MockStorage::new();
    let mut changeset = Changeset::new(
        "feature/complex",
        VersionBump::Minor,
        vec!["staging".to_string(), "production".to_string()],
    );

    // Add packages and commits
    changeset.add_package("@myorg/core");
    changeset.add_package("@myorg/utils");
    changeset.add_commit("abc123");
    changeset.add_commit("def456");

    // Save and reload
    storage.save(&changeset).await.unwrap();
    let loaded = storage.load("feature/complex").await.unwrap();

    // Verify all data was preserved
    assert_eq!(loaded.packages.len(), 2);
    assert_eq!(loaded.changes.len(), 2);
    assert!(loaded.has_package("@myorg/core"));
    assert!(loaded.has_package("@myorg/utils"));
    assert!(loaded.has_commit("abc123"));
    assert!(loaded.has_commit("def456"));
}

#[tokio::test]
async fn test_archive_preserves_all_data() {
    let storage = MockStorage::new();
    let mut changeset =
        Changeset::new("feature/full-data", VersionBump::Major, vec!["production".to_string()]);

    // Add comprehensive data
    changeset.add_package("package1");
    changeset.add_package("package2");
    changeset.add_commit("commit1");
    changeset.add_commit("commit2");
    changeset.set_environments(vec!["production".to_string(), "staging".to_string()]);

    storage.save(&changeset).await.unwrap();

    // Archive with release info
    let release_info = ReleaseInfo::new(
        "release-bot@example.com".to_string(),
        "release-commit-123".to_string(),
        versions_map(vec![
            ("package1".to_string(), "2.0.0".to_string()),
            ("package2".to_string(), "3.0.0".to_string()),
        ]),
    );

    storage.archive(&changeset, release_info).await.unwrap();

    // Load archived and verify all data
    let archived = storage.load_archived("feature/full-data").await.unwrap();
    assert_eq!(archived.changeset.packages.len(), 2);
    assert_eq!(archived.changeset.changes.len(), 2);
    assert_eq!(archived.changeset.environments.len(), 2);
    assert_eq!(archived.changeset.bump, VersionBump::Major);
    assert_eq!(archived.release_info.versions.len(), 2);
    assert_eq!(archived.release_info.git_commit, "release-commit-123".to_string());
}

#[tokio::test]
async fn test_list_pending_excludes_archived() {
    let storage = MockStorage::new();

    // Create two changesets
    let changeset1 =
        Changeset::new("feature/pending", VersionBump::Minor, vec!["production".to_string()]);
    let changeset2 =
        Changeset::new("feature/archived", VersionBump::Patch, vec!["production".to_string()]);

    storage.save(&changeset1).await.unwrap();
    storage.save(&changeset2).await.unwrap();

    // Archive one of them
    let release_info = ReleaseInfo::new(
        "user@example.com".to_string(),
        "commit123".to_string(),
        versions_map(vec![("pkg".to_string(), "1.0.0".to_string())]),
    );
    storage.archive(&changeset2, release_info).await.unwrap();

    // List pending should only return the non-archived one
    let pending = storage.list_pending().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].branch, "feature/pending");

    // List archived should only return the archived one
    let archived = storage.list_archived().await.unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].changeset.branch, "feature/archived");
}

// ============================================================================
// FileBasedChangesetStorage Tests
// ============================================================================

mod file_based_storage_tests {
    use super::*;
    use crate::changeset::FileBasedChangesetStorage;
    use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
    use tempfile::TempDir;

    /// Helper to create a temporary directory and file-based storage.
    async fn setup_file_storage() -> (TempDir, FileBasedChangesetStorage<FileSystemManager>) {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();
        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".to_string(),
            ".changesets/history".to_string(),
            fs,
        );
        (temp_dir, storage)
    }

    #[tokio::test]
    async fn test_file_save_and_load() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset =
            Changeset::new("feature/test", VersionBump::Minor, vec!["production".to_string()]);

        // Save the changeset
        storage.save(&changeset).await.unwrap();

        // Load it back
        let loaded = storage.load("feature/test").await.unwrap();
        assert_eq!(loaded.branch, "feature/test");
        assert_eq!(loaded.bump, VersionBump::Minor);
        assert_eq!(loaded.environments, vec!["production"]);
    }

    #[tokio::test]
    async fn test_file_load_nonexistent() {
        let (_temp_dir, storage) = setup_file_storage().await;

        let result = storage.load("nonexistent").await;
        assert!(result.is_err());
        match result {
            Err(ChangesetError::NotFound { branch }) => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_file_exists() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset =
            Changeset::new("feature/exists-test", VersionBump::Patch, vec!["staging".to_string()]);

        // Initially should not exist
        let exists = storage.exists("feature/exists-test").await.unwrap();
        assert!(!exists);

        // Save and check again
        storage.save(&changeset).await.unwrap();
        let exists = storage.exists("feature/exists-test").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_file_delete() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset = Changeset::new(
            "feature/delete-test",
            VersionBump::Major,
            vec!["production".to_string()],
        );

        // Save the changeset
        storage.save(&changeset).await.unwrap();
        assert!(storage.exists("feature/delete-test").await.unwrap());

        // Delete it
        storage.delete("feature/delete-test").await.unwrap();

        // Verify it's gone
        assert!(!storage.exists("feature/delete-test").await.unwrap());
    }

    #[tokio::test]
    async fn test_file_delete_nonexistent() {
        let (_temp_dir, storage) = setup_file_storage().await;

        // Deleting a nonexistent changeset should succeed (idempotent)
        let result = storage.delete("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_update_changeset() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let mut changeset = Changeset::new(
            "feature/update-test",
            VersionBump::Minor,
            vec!["production".to_string()],
        );

        // Save initial version
        storage.save(&changeset).await.unwrap();

        // Update and save again
        changeset.add_package("package1");
        changeset.set_bump(VersionBump::Major);
        storage.save(&changeset).await.unwrap();

        // Load and verify updates
        let loaded = storage.load("feature/update-test").await.unwrap();
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.bump, VersionBump::Major);
    }

    #[tokio::test]
    async fn test_file_list_pending_empty() {
        let (_temp_dir, storage) = setup_file_storage().await;

        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_file_list_pending_multiple() {
        let (_temp_dir, storage) = setup_file_storage().await;

        // Create and save multiple changesets
        let changeset1 =
            Changeset::new("feature/one", VersionBump::Minor, vec!["production".to_string()]);
        let changeset2 =
            Changeset::new("feature/two", VersionBump::Patch, vec!["staging".to_string()]);
        let changeset3 = Changeset::new(
            "feature/three",
            VersionBump::Major,
            vec!["production".to_string(), "staging".to_string()],
        );

        storage.save(&changeset1).await.unwrap();
        storage.save(&changeset2).await.unwrap();
        storage.save(&changeset3).await.unwrap();

        // List all pending
        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 3);

        // Verify all branches are present
        let branches: Vec<String> = pending.iter().map(|c| c.branch.clone()).collect();
        assert!(branches.contains(&"feature/one".to_string()));
        assert!(branches.contains(&"feature/two".to_string()));
        assert!(branches.contains(&"feature/three".to_string()));
    }

    #[tokio::test]
    async fn test_file_archive() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let mut changeset = Changeset::new(
            "feature/archive-test",
            VersionBump::Minor,
            vec!["production".to_string()],
        );
        changeset.add_package("package1");
        changeset.add_package("package2");

        // Save the changeset
        storage.save(&changeset).await.unwrap();
        assert!(storage.exists("feature/archive-test").await.unwrap());

        // Create release info
        let release_info = ReleaseInfo::new(
            "test-user@example.com".to_string(),
            "abc123def456".to_string(),
            versions_map(vec![
                ("package1".to_string(), "1.2.0".to_string()),
                ("package2".to_string(), "2.0.0".to_string()),
            ]),
        );

        // Archive it
        storage.archive(&changeset, release_info).await.unwrap();

        // Verify it's no longer in pending
        assert!(!storage.exists("feature/archive-test").await.unwrap());

        // Verify it's in archived
        let archived = storage.load_archived("feature/archive-test").await.unwrap();
        assert_eq!(archived.changeset.branch, "feature/archive-test");
        assert_eq!(archived.release_info.applied_by, "test-user@example.com");
        assert_eq!(archived.release_info.versions.len(), 2);
    }

    #[tokio::test]
    async fn test_file_archive_already_exists() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset =
            Changeset::new("feature/duplicate", VersionBump::Minor, vec!["production".to_string()]);

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit1".to_string(),
            versions_map(vec![]),
        );

        // Archive once - should succeed
        storage.archive(&changeset, release_info.clone()).await.unwrap();

        // Try to archive again - should fail
        let result = storage.archive(&changeset, release_info).await;
        assert!(result.is_err());
        match result {
            Err(ChangesetError::AlreadyExists { branch, .. }) => {
                assert_eq!(branch, "feature/duplicate");
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_file_load_archived_nonexistent() {
        let (_temp_dir, storage) = setup_file_storage().await;

        let result = storage.load_archived("nonexistent").await;
        assert!(result.is_err());
        match result {
            Err(ChangesetError::NotFound { branch }) => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_file_list_archived_empty() {
        let (_temp_dir, storage) = setup_file_storage().await;

        let archived = storage.list_archived().await.unwrap();
        assert_eq!(archived.len(), 0);
    }

    #[tokio::test]
    async fn test_file_list_archived_multiple() {
        let (_temp_dir, storage) = setup_file_storage().await;

        // Create and archive multiple changesets
        for i in 1..=3 {
            let mut changeset = Changeset::new(
                format!("feature/archived-{}", i),
                VersionBump::Minor,
                vec!["production".to_string()],
            );
            changeset.add_package(format!("package{}", i));

            storage.save(&changeset).await.unwrap();

            let release_info = ReleaseInfo::new(
                format!("user{}@example.com", i),
                format!("commit{}", i),
                versions_map(vec![(format!("package{}", i), format!("1.{}.0", i))]),
            );

            storage.archive(&changeset, release_info).await.unwrap();
        }

        // List all archived
        let archived = storage.list_archived().await.unwrap();
        assert_eq!(archived.len(), 3);

        // Verify all are archived
        for i in 1..=3 {
            let branch = format!("feature/archived-{}", i);
            let found = archived.iter().any(|a| a.changeset.branch == branch);
            assert!(found, "Expected to find archived changeset for {}", branch);
        }
    }

    #[tokio::test]
    async fn test_file_sanitize_branch_names() {
        let (_temp_dir, storage) = setup_file_storage().await;

        // Test with various special characters in branch names
        let branches = vec![
            "feature/new-api",
            "bugfix\\windows-path",
            "hotfix:critical",
            "feature*wildcard",
            "test?question",
            "branch\"quotes",
            "branch<greater",
            "branch>less",
            "branch|pipe",
        ];

        for branch in branches {
            let changeset =
                Changeset::new(branch, VersionBump::Patch, vec!["production".to_string()]);

            // Save and load should work despite special characters
            storage.save(&changeset).await.unwrap();
            let loaded = storage.load(branch).await.unwrap();
            assert_eq!(loaded.branch, branch);
        }
    }

    #[tokio::test]
    async fn test_file_with_packages_and_commits() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let mut changeset = Changeset::new(
            "feature/complex",
            VersionBump::Minor,
            vec!["staging".to_string(), "production".to_string()],
        );

        // Add packages and commits
        changeset.add_package("@myorg/core");
        changeset.add_package("@myorg/utils");
        changeset.add_commit("abc123");
        changeset.add_commit("def456");

        // Save and reload
        storage.save(&changeset).await.unwrap();
        let loaded = storage.load("feature/complex").await.unwrap();

        // Verify all data was preserved
        assert_eq!(loaded.packages.len(), 2);
        assert_eq!(loaded.changes.len(), 2);
        assert!(loaded.has_package("@myorg/core"));
        assert!(loaded.has_package("@myorg/utils"));
        assert!(loaded.has_commit("abc123"));
        assert!(loaded.has_commit("def456"));
    }

    #[tokio::test]
    async fn test_file_archive_preserves_all_data() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let mut changeset =
            Changeset::new("feature/full-data", VersionBump::Major, vec!["production".to_string()]);

        // Add comprehensive data
        changeset.add_package("package1");
        changeset.add_package("package2");
        changeset.add_commit("commit1");
        changeset.add_commit("commit2");
        changeset.set_environments(vec!["production".to_string(), "staging".to_string()]);

        storage.save(&changeset).await.unwrap();

        // Archive with release info
        let release_info = ReleaseInfo::new(
            "release-bot@example.com".to_string(),
            "release-commit-123".to_string(),
            versions_map(vec![
                ("package1".to_string(), "2.0.0".to_string()),
                ("package2".to_string(), "3.0.0".to_string()),
            ]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        // Load archived and verify all data
        let archived = storage.load_archived("feature/full-data").await.unwrap();
        assert_eq!(archived.changeset.packages.len(), 2);
        assert_eq!(archived.changeset.changes.len(), 2);
        assert_eq!(archived.changeset.environments.len(), 2);
        assert_eq!(archived.changeset.bump, VersionBump::Major);
        assert_eq!(archived.release_info.versions.len(), 2);
        assert_eq!(archived.release_info.git_commit, "release-commit-123".to_string());
    }

    #[tokio::test]
    async fn test_file_list_pending_excludes_archived() {
        let (_temp_dir, storage) = setup_file_storage().await;

        // Create two changesets
        let changeset1 =
            Changeset::new("feature/pending", VersionBump::Minor, vec!["production".to_string()]);
        let changeset2 =
            Changeset::new("feature/archived", VersionBump::Patch, vec!["production".to_string()]);

        storage.save(&changeset1).await.unwrap();
        storage.save(&changeset2).await.unwrap();

        // Archive one of them
        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit123".to_string(),
            versions_map(vec![("pkg".to_string(), "1.0.0".to_string())]),
        );
        storage.archive(&changeset2, release_info).await.unwrap();

        // List pending should only return the non-archived one
        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].branch, "feature/pending");

        // List archived should only return the archived one
        let archived = storage.list_archived().await.unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].changeset.branch, "feature/archived");
    }

    #[tokio::test]
    async fn test_file_persistence_across_instances() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();

        // Create first storage instance and save
        {
            let storage = FileBasedChangesetStorage::new(
                temp_dir.path().to_path_buf(),
                ".changesets".to_string(),
                ".changesets/history".to_string(),
                fs.clone(),
            );

            let changeset = Changeset::new(
                "feature/persist",
                VersionBump::Minor,
                vec!["production".to_string()],
            );

            storage.save(&changeset).await.unwrap();
        }

        // Create second storage instance and load
        {
            let storage = FileBasedChangesetStorage::new(
                temp_dir.path().to_path_buf(),
                ".changesets".to_string(),
                ".changesets/history".to_string(),
                fs,
            );

            let loaded = storage.load("feature/persist").await.unwrap();
            assert_eq!(loaded.branch, "feature/persist");
        }
    }

    #[tokio::test]
    async fn test_file_json_format_readable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();
        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".to_string(),
            ".changesets/history".to_string(),
            fs.clone(),
        );

        let mut changeset =
            Changeset::new("feature/readable", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("test-package");

        storage.save(&changeset).await.unwrap();

        // Read the file directly and verify it's valid JSON
        let path = temp_dir.path().join(".changesets").join("feature-readable.json");
        let contents = fs.read_file_string(&path).await.unwrap();

        // Verify it's valid JSON and contains expected fields
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(json["branch"], "feature/readable");
        assert_eq!(json["bump"], "minor");
        assert!(json["packages"].is_array());
    }

    #[tokio::test]
    async fn test_file_concurrent_saves() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();
        let storage = Arc::new(FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".to_string(),
            ".changesets/history".to_string(),
            fs,
        ));

        // Spawn multiple tasks that save different changesets concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let changeset = Changeset::new(
                    format!("feature/concurrent-{}", i),
                    VersionBump::Patch,
                    vec!["production".to_string()],
                );
                storage_clone.save(&changeset).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all changesets were saved
        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 10);
    }

    #[tokio::test]
    async fn test_file_empty_packages_list() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset = Changeset::new(
            "feature/no-packages",
            VersionBump::None,
            vec!["production".to_string()],
        );

        // Save changeset with no packages
        storage.save(&changeset).await.unwrap();

        // Load and verify
        let loaded = storage.load("feature/no-packages").await.unwrap();
        assert!(loaded.packages.is_empty());
        assert_eq!(loaded.bump, VersionBump::None);
    }

    #[tokio::test]
    async fn test_file_timestamps_preserved() {
        let (_temp_dir, storage) = setup_file_storage().await;
        let changeset = Changeset::new(
            "feature/timestamps",
            VersionBump::Minor,
            vec!["production".to_string()],
        );

        let created_at = changeset.created_at;
        let updated_at = changeset.updated_at;

        // Save and reload
        storage.save(&changeset).await.unwrap();
        let loaded = storage.load("feature/timestamps").await.unwrap();

        // Verify timestamps are preserved
        assert_eq!(loaded.created_at, created_at);
        assert_eq!(loaded.updated_at, updated_at);
    }

    #[tokio::test]
    async fn test_file_list_ignores_non_json_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();
        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".to_string(),
            ".changesets/history".to_string(),
            fs.clone(),
        );

        // Create a valid changeset
        let changeset =
            Changeset::new("feature/valid", VersionBump::Minor, vec!["production".to_string()]);
        storage.save(&changeset).await.unwrap();

        // Create a non-JSON file in the same directory
        let dir_path = temp_dir.path().join(".changesets");
        let non_json_path = dir_path.join("README.md");
        fs.write_file_string(&non_json_path, "# Changesets").await.unwrap();

        // List should only return the valid changeset
        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].branch, "feature/valid");
    }
}

// ============================================================================
// ChangesetManager Tests
// ============================================================================

mod manager_tests {
    use super::*;
    use crate::changeset::ChangesetManager;
    use crate::config::ChangesetConfig;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use sublime_standard_tools::filesystem::FileSystemManager;

    /// Mock storage implementation for testing ChangesetManager.
    #[derive(Debug, Clone)]
    struct MockManagerStorage {
        changesets: Arc<Mutex<HashMap<String, Changeset>>>,
    }

    impl MockManagerStorage {
        fn new() -> Self {
            Self { changesets: Arc::new(Mutex::new(HashMap::new())) }
        }

        fn get_count(&self) -> usize {
            self.changesets.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl ChangesetStorage for MockManagerStorage {
        async fn save(&self, changeset: &Changeset) -> ChangesetResult<()> {
            self.changesets.lock().unwrap().insert(changeset.branch.clone(), changeset.clone());
            Ok(())
        }

        async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
            self.changesets
                .lock()
                .unwrap()
                .get(branch)
                .cloned()
                .ok_or_else(|| ChangesetError::NotFound { branch: branch.to_string() })
        }

        async fn exists(&self, branch: &str) -> ChangesetResult<bool> {
            Ok(self.changesets.lock().unwrap().contains_key(branch))
        }

        async fn delete(&self, branch: &str) -> ChangesetResult<()> {
            self.changesets
                .lock()
                .unwrap()
                .remove(branch)
                .ok_or_else(|| ChangesetError::NotFound { branch: branch.to_string() })?;
            Ok(())
        }

        async fn list_pending(&self) -> ChangesetResult<Vec<Changeset>> {
            Ok(self.changesets.lock().unwrap().values().cloned().collect())
        }

        async fn archive(
            &self,
            _changeset: &Changeset,
            _release_info: ReleaseInfo,
        ) -> ChangesetResult<()> {
            // TODO: will be implemented on story 6.5
            Ok(())
        }

        async fn load_archived(&self, _id: &str) -> ChangesetResult<ArchivedChangeset> {
            // TODO: will be implemented on story 6.5
            Err(ChangesetError::NotFound { branch: "archived".to_string() })
        }

        async fn list_archived(&self) -> ChangesetResult<Vec<ArchivedChangeset>> {
            // TODO: will be implemented on story 6.5
            Ok(Vec::new())
        }
    }

    fn create_test_config() -> ChangesetConfig {
        ChangesetConfig {
            path: ".changesets".into(),
            history_path: ".changesets/history".into(),
            available_environments: vec![
                "development".to_string(),
                "staging".to_string(),
                "production".to_string(),
            ],
            default_environments: vec!["production".to_string()],
        }
    }

    fn create_test_manager() -> ChangesetManager<MockManagerStorage> {
        let storage = MockManagerStorage::new();
        let config = create_test_config();
        ChangesetManager::with_storage(storage, None, config)
    }

    #[tokio::test]
    async fn test_create_changeset_success() {
        let manager = create_test_manager();

        let result = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await;

        assert!(result.is_ok());
        let changeset = result.unwrap();
        assert_eq!(changeset.branch, "feature/test");
        assert_eq!(changeset.bump, VersionBump::Minor);
        assert_eq!(changeset.environments, vec!["production".to_string()]);
    }

    #[tokio::test]
    async fn test_create_changeset_empty_branch() {
        let manager = create_test_manager();

        let result = manager.create("", VersionBump::Minor, vec!["production".to_string()]).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChangesetError::InvalidBranch { branch, reason } => {
                assert_eq!(branch, "");
                assert!(reason.contains("empty"));
            }
            _ => panic!("Expected InvalidBranch error"),
        }
    }

    #[tokio::test]
    async fn test_create_changeset_already_exists() {
        let manager = create_test_manager();

        // Create first changeset
        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Try to create duplicate
        let result = manager
            .create("feature/test", VersionBump::Patch, vec!["production".to_string()])
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChangesetError::AlreadyExists { branch, .. } => {
                assert_eq!(branch, "feature/test");
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_create_changeset_invalid_environment() {
        let manager = create_test_manager();

        let result = manager
            .create("feature/test", VersionBump::Minor, vec!["invalid-env".to_string()])
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChangesetError::InvalidEnvironment { environment, available } => {
                assert_eq!(environment, "invalid-env");
                assert!(available.contains(&"production".to_string()));
            }
            _ => panic!("Expected InvalidEnvironment error"),
        }
    }

    #[tokio::test]
    async fn test_load_changeset_success() {
        let manager = create_test_manager();

        // Create a changeset first
        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Load it
        let result = manager.load("feature/test").await;
        assert!(result.is_ok());

        let changeset = result.unwrap();
        assert_eq!(changeset.branch, "feature/test");
        assert_eq!(changeset.bump, VersionBump::Minor);
    }

    #[tokio::test]
    async fn test_load_changeset_not_found() {
        let manager = create_test_manager();

        let result = manager.load("nonexistent").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ChangesetError::NotFound { branch } => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_changeset_success() {
        let manager = create_test_manager();

        // Create a changeset
        let changeset = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        let original_updated_at = changeset.updated_at;

        // Give a tiny delay to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Modify and update
        let mut modified = changeset.clone();
        modified.add_package("test-package");

        let result = manager.update(&modified).await;
        assert!(result.is_ok());

        // Load and verify
        let loaded = manager.load("feature/test").await.unwrap();
        assert!(loaded.packages.contains(&"test-package".to_string()));
        assert!(loaded.updated_at > original_updated_at);
    }

    #[tokio::test]
    async fn test_update_changeset_validation_failure() {
        let manager = create_test_manager();

        // Create a changeset with invalid environment manually
        let changeset =
            Changeset::new("feature/test", VersionBump::Minor, vec!["invalid-env".to_string()]);

        let result = manager.update(&changeset).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ChangesetError::ValidationFailed { .. } => {
                // Expected
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_delete_changeset_success() {
        let manager = create_test_manager();

        // Create a changeset
        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Delete it
        let result = manager.delete("feature/test").await;
        assert!(result.is_ok());

        // Verify it's gone
        let load_result = manager.load("feature/test").await;
        assert!(load_result.is_err());
    }

    #[tokio::test]
    async fn test_delete_changeset_not_found() {
        let manager = create_test_manager();

        let result = manager.delete("nonexistent").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ChangesetError::NotFound { branch } => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_list_pending_empty() {
        let manager = create_test_manager();

        let result = manager.list_pending().await;
        assert!(result.is_ok());

        let changesets = result.unwrap();
        assert_eq!(changesets.len(), 0);
    }

    #[tokio::test]
    async fn test_list_pending_multiple() {
        let manager = create_test_manager();

        // Create multiple changesets
        manager
            .create("feature/one", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        manager
            .create("feature/two", VersionBump::Patch, vec!["staging".to_string()])
            .await
            .unwrap();

        manager
            .create("feature/three", VersionBump::Major, vec!["development".to_string()])
            .await
            .unwrap();

        // List them
        let result = manager.list_pending().await;
        assert!(result.is_ok());

        let changesets = result.unwrap();
        assert_eq!(changesets.len(), 3);

        // Verify all branches are present
        let branches: Vec<&str> = changesets.iter().map(|cs| cs.branch.as_str()).collect();
        assert!(branches.contains(&"feature/one"));
        assert!(branches.contains(&"feature/two"));
        assert!(branches.contains(&"feature/three"));
    }

    #[tokio::test]
    async fn test_manager_accessors() {
        let manager = create_test_manager();

        // Test storage accessor
        let storage = manager.storage();
        assert_eq!(storage.get_count(), 0);

        // Test git_repo accessor
        assert!(manager.git_repo().is_none());

        // Test config accessor
        let config = manager.config();
        assert_eq!(config.path, ".changesets");
        assert_eq!(config.available_environments.len(), 3);
    }

    #[tokio::test]
    async fn test_update_with_multiple_modifications() {
        let manager = create_test_manager();

        // Create a changeset
        let changeset = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Make multiple modifications
        let mut modified = changeset.clone();
        modified.add_package("package-1");
        modified.add_package("package-2");
        modified.add_commit("abc123");
        modified.add_commit("def456");
        modified.set_bump(VersionBump::Major);

        // Update
        manager.update(&modified).await.unwrap();

        // Load and verify all changes
        let loaded = manager.load("feature/test").await.unwrap();
        assert_eq!(loaded.packages.len(), 2);
        assert!(loaded.packages.contains(&"package-1".to_string()));
        assert!(loaded.packages.contains(&"package-2".to_string()));
        assert_eq!(loaded.changes.len(), 2);
        assert!(loaded.changes.contains(&"abc123".to_string()));
        assert!(loaded.changes.contains(&"def456".to_string()));
        assert_eq!(loaded.bump, VersionBump::Major);
    }

    #[tokio::test]
    async fn test_create_with_multiple_environments() {
        let manager = create_test_manager();

        let changeset = manager
            .create(
                "feature/test",
                VersionBump::Minor,
                vec!["development".to_string(), "staging".to_string(), "production".to_string()],
            )
            .await
            .unwrap();

        assert_eq!(changeset.environments.len(), 3);
        assert!(changeset.environments.contains(&"development".to_string()));
        assert!(changeset.environments.contains(&"staging".to_string()));
        assert!(changeset.environments.contains(&"production".to_string()));
    }

    #[tokio::test]
    async fn test_manager_with_file_based_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fs = FileSystemManager::new();

        let config = crate::config::PackageToolsConfig {
            changeset: create_test_config(),
            ..Default::default()
        };

        let manager =
            ChangesetManager::new(temp_dir.path().to_path_buf(), fs, config).await.unwrap();

        // Create a changeset
        let changeset = manager
            .create("feature/file-test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Load it back
        let loaded = manager.load("feature/file-test").await.unwrap();
        assert_eq!(loaded.branch, changeset.branch);
        assert_eq!(loaded.bump, changeset.bump);

        // Update it
        let mut modified = loaded.clone();
        modified.add_package("test-package");
        manager.update(&modified).await.unwrap();

        // Verify update persisted
        let updated = manager.load("feature/file-test").await.unwrap();
        assert!(updated.packages.contains(&"test-package".to_string()));

        // Delete it
        manager.delete("feature/file-test").await.unwrap();

        // Verify deletion
        let result = manager.load("feature/file-test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_validates_all_environments() {
        let manager = create_test_manager();

        // Try to create with mix of valid and invalid environments
        let result = manager
            .create(
                "feature/test",
                VersionBump::Minor,
                vec!["production".to_string(), "invalid".to_string()],
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChangesetError::InvalidEnvironment { environment, .. } => {
                assert_eq!(environment, "invalid");
            }
            _ => panic!("Expected InvalidEnvironment error"),
        }
    }

    #[tokio::test]
    async fn test_list_pending_returns_loaded_changesets() {
        let manager = create_test_manager();

        // Create changesets with different properties
        manager
            .create("feature/one", VersionBump::Major, vec!["production".to_string()])
            .await
            .unwrap();

        manager
            .create("feature/two", VersionBump::Minor, vec!["staging".to_string()])
            .await
            .unwrap();

        // List and verify each is fully loaded
        let changesets = manager.list_pending().await.unwrap();
        assert_eq!(changesets.len(), 2);

        for changeset in changesets {
            assert!(!changeset.branch.is_empty());
            assert!(!changeset.environments.is_empty());
        }
    }
}
