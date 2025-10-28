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
#![allow(clippy::expect_used)]

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
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use sublime_standard_tools::filesystem::FileSystemManager;

    /// Mock storage implementation for testing ChangesetManager.
    #[derive(Debug, Clone)]
    pub(super) struct MockManagerStorage {
        changesets: Arc<Mutex<HashMap<String, Changeset>>>,
        archived: Arc<Mutex<HashMap<String, ArchivedChangeset>>>,
    }

    impl MockManagerStorage {
        fn new() -> Self {
            Self {
                changesets: Arc::new(Mutex::new(HashMap::new())),
                archived: Arc::new(Mutex::new(HashMap::new())),
            }
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
            changeset: &Changeset,
            release_info: ReleaseInfo,
        ) -> ChangesetResult<()> {
            // Remove from pending
            self.changesets.lock().unwrap().remove(&changeset.branch);

            // Add to archived
            let archived_changeset = ArchivedChangeset::new(changeset.clone(), release_info);
            self.archived.lock().unwrap().insert(changeset.branch.clone(), archived_changeset);

            Ok(())
        }

        async fn load_archived(&self, branch: &str) -> ChangesetResult<ArchivedChangeset> {
            self.archived
                .lock()
                .unwrap()
                .get(branch)
                .cloned()
                .ok_or_else(|| ChangesetError::NotFound { branch: branch.to_string() })
        }

        async fn list_archived(&self) -> ChangesetResult<Vec<ArchivedChangeset>> {
            Ok(self.archived.lock().unwrap().values().cloned().collect())
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

    pub(super) fn create_test_manager() -> ChangesetManager<MockManagerStorage> {
        let storage = MockManagerStorage::new();
        let config = create_test_config();
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        ChangesetManager::with_storage(storage, workspace_root, None, config)
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

    #[tokio::test]
    async fn test_manager_add_commits() {
        let manager = create_test_manager();

        // Create a changeset
        let mut changeset = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Add a package to satisfy validation
        changeset.add_package("test-package");
        manager.update(&changeset).await.unwrap();

        // Add commits manually
        let commits = vec!["abc123".to_string(), "def456".to_string()];
        let summary = manager.add_commits("feature/test", commits).await.unwrap();

        assert_eq!(summary.commits_added, 2);
        assert_eq!(summary.commit_ids.len(), 2);
        assert!(summary.new_packages.is_empty());

        // Verify commits were added
        let changeset = manager.load("feature/test").await.unwrap();
        assert_eq!(changeset.changes.len(), 2);
        assert!(changeset.has_commit("abc123"));
        assert!(changeset.has_commit("def456"));
    }

    #[tokio::test]
    async fn test_manager_add_commits_duplicate() {
        let manager = create_test_manager();

        // Create a changeset
        let mut changeset = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Add a package to satisfy validation
        changeset.add_package("test-package");
        manager.update(&changeset).await.unwrap();

        // Add commits
        let commits = vec!["abc123".to_string(), "def456".to_string()];
        manager.add_commits("feature/test", commits).await.unwrap();

        // Try to add same commits again
        let commits = vec!["abc123".to_string(), "ghi789".to_string()];
        let summary = manager.add_commits("feature/test", commits).await.unwrap();

        // Only the new commit should be added
        assert_eq!(summary.commits_added, 1);
        assert_eq!(summary.commit_ids, vec!["ghi789".to_string()]);

        let changeset = manager.load("feature/test").await.unwrap();
        assert_eq!(changeset.changes.len(), 3);
    }

    #[tokio::test]
    async fn test_manager_add_commits_empty() {
        let manager = create_test_manager();

        // Create a changeset
        let mut changeset = manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Add a package to satisfy validation
        changeset.add_package("test-package");
        manager.update(&changeset).await.unwrap();

        // Add empty commit list
        let summary = manager.add_commits("feature/test", vec![]).await.unwrap();

        assert_eq!(summary.commits_added, 0);
        assert!(summary.commit_ids.is_empty());
    }

    #[tokio::test]
    async fn test_manager_add_commits_nonexistent_changeset() {
        let manager = create_test_manager();

        let result = manager.add_commits("nonexistent", vec!["abc123".to_string()]).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ChangesetError::NotFound { branch } => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_manager_archive_success() {
        let manager = create_test_manager();

        // Create a changeset
        let mut changeset = manager
            .create("feature/archive-test", VersionBump::Major, vec!["production".to_string()])
            .await
            .unwrap();

        changeset.add_package("@myorg/core");
        changeset.add_package("@myorg/utils");
        manager.update(&changeset).await.unwrap();

        // Create release info
        let mut versions = HashMap::new();
        versions.insert("@myorg/core".to_string(), "2.0.0".to_string());
        versions.insert("@myorg/utils".to_string(), "1.5.0".to_string());

        let release_info = crate::types::ReleaseInfo::new(
            "ci-bot@example.com".to_string(),
            "abc123def456789".to_string(),
            versions,
        );

        // Archive the changeset
        let result = manager.archive("feature/archive-test", release_info).await;
        assert!(result.is_ok(), "Archive should succeed");

        // Verify it's no longer in pending
        let exists = manager.storage().exists("feature/archive-test").await.unwrap();
        assert!(!exists, "Changeset should not exist in pending after archiving");

        // Verify it's in archived
        let archived = manager.storage().load_archived("feature/archive-test").await;
        assert!(archived.is_ok(), "Archived changeset should be loadable");

        let archived = archived.unwrap();
        assert_eq!(archived.changeset.branch, "feature/archive-test");
        assert_eq!(archived.changeset.bump, VersionBump::Major);
        assert_eq!(archived.release_info.applied_by, "ci-bot@example.com");
        assert_eq!(archived.release_info.git_commit, "abc123def456789");
        assert_eq!(archived.release_info.package_count(), 2);
        assert_eq!(archived.release_info.get_version("@myorg/core"), Some("2.0.0"));
        assert_eq!(archived.release_info.get_version("@myorg/utils"), Some("1.5.0"));
    }

    #[tokio::test]
    async fn test_manager_archive_nonexistent_changeset() {
        let manager = create_test_manager();

        let versions = HashMap::new();
        let release_info = crate::types::ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit123".to_string(),
            versions,
        );

        let result = manager.archive("nonexistent", release_info).await;

        assert!(result.is_err(), "Archive should fail for nonexistent changeset");
        match result.unwrap_err() {
            ChangesetError::NotFound { branch } => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}

// ============================================================================
// Git Integration Tests
// ============================================================================

mod git_integration_tests {
    use crate::changeset::{ChangesetManager, FileBasedChangesetStorage, PackageDetector};
    use crate::config::ChangesetConfig;
    use crate::error::ChangesetError;
    use crate::types::VersionBump;
    use std::fs;
    use sublime_git_tools::Repo;
    use sublime_standard_tools::filesystem::FileSystemManager;
    use tempfile::TempDir;

    /// Helper to create a test git repository with commits.
    fn setup_git_repo() -> (TempDir, Repo) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        let repo = Repo::create(repo_path.to_str().unwrap()).unwrap();

        // Configure git user
        repo.config("user.name", "Test User").unwrap();
        repo.config("user.email", "test@example.com").unwrap();

        // Create initial commit
        fs::write(repo_path.join("README.md"), "# Test Repo").unwrap();
        repo.add("README.md").unwrap();
        repo.commit("Initial commit").unwrap();

        (temp_dir, repo)
    }

    /// Helper to create a monorepo structure with pnpm workspaces.
    fn setup_monorepo(repo_path: &std::path::Path) {
        // Create root package.json with private flag
        let root_package = serde_json::json!({
            "name": "@test/monorepo",
            "version": "1.0.0",
            "private": true,
            "scripts": {
                "build": "echo 'Building all packages...'"
            }
        });
        fs::write(
            repo_path.join("package.json"),
            serde_json::to_string_pretty(&root_package).unwrap(),
        )
        .unwrap();

        // Create pnpm-workspace.yaml (critical for pnpm monorepo detection)
        let pnpm_workspace = "packages:\n  - 'packages/*'\n";
        fs::write(repo_path.join("pnpm-workspace.yaml"), pnpm_workspace).unwrap();

        // Create packages directory
        fs::create_dir_all(repo_path.join("packages")).unwrap();

        // Create package1
        fs::create_dir_all(repo_path.join("packages/package1/src")).unwrap();
        let pkg1 = serde_json::json!({
            "name": "@test/package1",
            "version": "1.0.0",
            "main": "src/index.js",
            "scripts": {
                "build": "echo 'Building package1...'"
            }
        });
        fs::write(
            repo_path.join("packages/package1/package.json"),
            serde_json::to_string_pretty(&pkg1).unwrap(),
        )
        .unwrap();
        fs::write(
            repo_path.join("packages/package1/src/index.js"),
            "module.exports = { name: 'package1' };\n",
        )
        .unwrap();

        // Create package2
        fs::create_dir_all(repo_path.join("packages/package2/src")).unwrap();
        let pkg2 = serde_json::json!({
            "name": "@test/package2",
            "version": "1.0.0",
            "main": "src/index.js",
            "scripts": {
                "build": "echo 'Building package2...'"
            },
            "dependencies": {
                "@test/package1": "workspace:*"
            }
        });
        fs::write(
            repo_path.join("packages/package2/package.json"),
            serde_json::to_string_pretty(&pkg2).unwrap(),
        )
        .unwrap();
        fs::write(
            repo_path.join("packages/package2/src/index.js"),
            "module.exports = { name: 'package2' };\n",
        )
        .unwrap();
    }

    /// Helper to create a single package structure.
    fn setup_single_package(repo_path: &std::path::Path) {
        let package = serde_json::json!({
            "name": "single-package",
            "version": "1.0.0",
            "main": "src/index.js",
            "scripts": {
                "build": "echo 'Building...'"
            }
        });
        fs::write(repo_path.join("package.json"), serde_json::to_string_pretty(&package).unwrap())
            .unwrap();

        // Create src directory and a source file
        fs::create_dir_all(repo_path.join("src")).unwrap();
        fs::write(repo_path.join("src/index.js"), "module.exports = { name: 'single-package' };\n")
            .unwrap();
    }

    #[tokio::test]
    async fn test_package_detector_is_monorepo() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        // Commit the monorepo setup
        repo.add_all().unwrap();
        repo.commit("Setup monorepo").unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let is_monorepo = detector.is_monorepo().await.unwrap();
        assert!(is_monorepo);
    }

    #[tokio::test]
    async fn test_package_detector_is_not_monorepo() {
        let (temp_dir, repo) = setup_git_repo();
        setup_single_package(temp_dir.path());

        // Commit the single package setup
        repo.add_all().unwrap();
        repo.commit("Setup single package").unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let is_monorepo = detector.is_monorepo().await.unwrap();
        assert!(!is_monorepo);
    }

    #[tokio::test]
    async fn test_package_detector_list_packages_monorepo() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        // Commit the monorepo setup
        repo.add_all().unwrap();
        repo.commit("Setup monorepo").unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let packages = detector.list_packages().await.unwrap();
        // With proper pnpm workspace setup, should find 2 packages
        assert_eq!(packages.len(), 2, "Should detect both packages in monorepo");
        assert!(packages.contains(&"@test/package1".to_string()));
        assert!(packages.contains(&"@test/package2".to_string()));
    }

    #[tokio::test]
    async fn test_package_detector_list_packages_single() {
        let (temp_dir, repo) = setup_git_repo();
        setup_single_package(temp_dir.path());

        // Commit the single package setup
        repo.add_all().unwrap();
        repo.commit("Setup single package").unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let packages = detector.list_packages().await.unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0], "single-package");
    }

    #[tokio::test]
    async fn test_package_detector_detect_affected_packages_monorepo() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        // Commit the monorepo setup
        repo.add_all().unwrap();
        repo.commit("Setup monorepo").unwrap();

        // Make changes to package1
        fs::write(temp_dir.path().join("packages/package1/src/index.js"), "console.log('hello');")
            .unwrap();
        repo.add("packages/package1/src/index.js").unwrap();
        repo.commit("Update package1").unwrap();

        // Get the last commit hash
        let commits = repo.get_commits_since(None, &None).unwrap();
        let last_commit = &commits[0].hash;

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let affected =
            detector.detect_affected_packages(std::slice::from_ref(last_commit)).await.unwrap();
        // Should detect the affected package
        assert!(!affected.is_empty(), "Should detect at least one affected package");
        assert!(
            affected.contains(&"@test/package1".to_string()),
            "Should detect package1 was affected"
        );
    }

    #[tokio::test]
    async fn test_package_detector_detect_affected_packages_multiple() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        // Commit the monorepo setup
        repo.add_all().unwrap();
        repo.commit("Setup monorepo").unwrap();

        // Make changes to package1
        fs::write(temp_dir.path().join("packages/package1/src/index.js"), "console.log('hello');")
            .unwrap();
        repo.add("packages/package1/src/index.js").unwrap();
        repo.commit("Update package1").unwrap();
        let commit1 = repo.get_current_sha().unwrap();

        // Make changes to package2
        fs::write(temp_dir.path().join("packages/package2/src/index.js"), "console.log('world');")
            .unwrap();
        repo.add("packages/package2/src/index.js").unwrap();
        repo.commit("Update package2").unwrap();
        let commit2 = repo.get_current_sha().unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let affected = detector.detect_affected_packages(&[commit1, commit2]).await.unwrap();
        // Should detect both affected packages from multiple commits
        assert_eq!(affected.len(), 2, "Should detect both affected packages");
        assert!(affected.contains(&"@test/package1".to_string()));
        assert!(affected.contains(&"@test/package2".to_string()));
    }

    #[tokio::test]
    async fn test_package_detector_detect_affected_packages_single() {
        let (temp_dir, repo) = setup_git_repo();
        setup_single_package(temp_dir.path());

        // Commit the setup
        repo.add_all().unwrap();
        repo.commit("Setup single package").unwrap();

        // Make changes
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/index.js"), "console.log('test');").unwrap();
        repo.add("src/index.js").unwrap();
        repo.commit("Update code").unwrap();

        let commits = repo.get_commits_since(None, &None).unwrap();
        let last_commit = &commits[0].hash;

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let affected =
            detector.detect_affected_packages(std::slice::from_ref(last_commit)).await.unwrap();
        // Should detect the single package
        assert!(!affected.is_empty(), "Should detect the single package when files change");
    }

    #[tokio::test]
    async fn test_package_detector_empty_commit_list() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let affected = detector.detect_affected_packages(&[]).await.unwrap();
        assert_eq!(affected.len(), 0);
    }

    #[tokio::test]
    async fn test_package_detector_get_commits_since() {
        let (temp_dir, repo) = setup_git_repo();

        // Create a few commits
        for i in 1..=3 {
            fs::write(temp_dir.path().join(format!("file{}.txt", i)), format!("content {}", i))
                .unwrap();
            repo.add(&format!("file{}.txt", i)).unwrap();
            repo.commit(&format!("Commit {}", i)).unwrap();
        }

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        // Get all commits
        let commits = detector.get_commits_since(None).unwrap();
        assert!(commits.len() >= 3, "Should have at least 3 commits plus initial");

        // Get commits since first commit
        if commits.len() > 1 {
            let first_commit = commits.last().unwrap().hash.clone();
            let recent_commits = detector.get_commits_since(Some(first_commit)).unwrap();
            assert!(!recent_commits.is_empty(), "Should have commits since first commit");
        }
    }

    #[allow(clippy::len_zero)]
    #[tokio::test]
    async fn test_package_detector_get_commits_between() {
        let (temp_dir, repo) = setup_git_repo();

        // Create commits and tag them
        fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
        repo.add("file1.txt").unwrap();
        repo.commit("Commit 1").unwrap();
        repo.create_tag("v1.0.0", Some("Version 1.0.0".to_string())).unwrap();

        fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();
        repo.add("file2.txt").unwrap();
        repo.commit("Commit 2").unwrap();

        fs::write(temp_dir.path().join("file3.txt"), "content 3").unwrap();
        repo.add("file3.txt").unwrap();
        repo.commit("Commit 3").unwrap();
        repo.create_tag("v2.0.0", Some("Version 2.0.0".to_string())).unwrap();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let commits = detector.get_commits_between("v1.0.0", "v2.0.0").unwrap();
        assert!(!commits.is_empty(), "Should have commits between tags");
        assert!(commits.len() >= 1, "Should have at least one commit between v1.0.0 and v2.0.0");
    }

    #[tokio::test]
    async fn test_package_detector_workspace_root() {
        let (temp_dir, repo) = setup_git_repo();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        assert_eq!(detector.workspace_root(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_package_detector_repo_reference() {
        let (temp_dir, repo) = setup_git_repo();

        let detector =
            PackageDetector::new(temp_dir.path().to_path_buf(), &repo, FileSystemManager::new());

        let repo_ref = detector.repo();
        // Just verify we can access the repo reference
        let repo_path = repo_ref.get_repo_path();
        // get_repo_path returns &Path directly
        assert!(!repo_path.as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_manager_add_commits_from_git_monorepo() {
        let (temp_dir, repo) = setup_git_repo();
        setup_monorepo(temp_dir.path());

        // Commit the monorepo setup
        repo.add_all().unwrap();
        repo.commit("Setup monorepo").unwrap();

        // Make changes to package1
        fs::write(
            temp_dir.path().join("packages/package1/src/index.js"),
            "console.log('updated');",
        )
        .unwrap();
        repo.add("packages/package1/src/index.js").unwrap();
        repo.commit("Update package1").unwrap();
        let commit1 = repo.get_current_sha().unwrap();

        // Make changes to package2
        fs::write(
            temp_dir.path().join("packages/package2/src/index.js"),
            "console.log('also updated');",
        )
        .unwrap();
        repo.add("packages/package2/src/index.js").unwrap();
        repo.commit("Update package2").unwrap();
        let commit2 = repo.get_current_sha().unwrap();

        // Create a changeset
        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".into(),
            ".changesets/history".into(),
            FileSystemManager::new(),
        );

        let config = ChangesetConfig {
            path: ".changesets".into(),
            history_path: ".changesets/history".into(),
            available_environments: vec!["production".to_string()],
            default_environments: vec!["production".to_string()],
        };

        let manager = ChangesetManager::with_storage(
            storage,
            temp_dir.path().to_path_buf(),
            Some(repo),
            config,
        );

        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Add initial package and setup commit to changeset so get_commits_since knows where to start
        let mut changeset = manager.load("feature/test").await.unwrap();
        changeset.add_package("@test/package1");
        // Get all commits and find the setup commit
        let all_commits =
            manager.git_repo().as_ref().unwrap().get_commits_since(None, &None).unwrap();
        // Find the "Setup monorepo" commit
        let setup_commit = all_commits
            .iter()
            .find(|c| c.message.contains("Setup monorepo"))
            .map(|c| &c.hash)
            .expect("Setup monorepo commit not found");
        changeset.add_commit(setup_commit);
        manager.update(&changeset).await.unwrap();

        // Use add_commits_from_git to automatically detect and add commits
        let summary = manager.add_commits_from_git("feature/test").await.unwrap();

        // Verify summary - expect only the 2 update commits (package1 and package2)
        assert_eq!(summary.commits_added, 2, "Should have added 2 commits");
        let total_packages = summary.new_packages.len() + summary.existing_packages.len();
        assert_eq!(total_packages, 2, "Should have affected 2 packages");

        let all_packages: Vec<String> =
            summary.new_packages.iter().chain(summary.existing_packages.iter()).cloned().collect();
        assert!(all_packages.contains(&"@test/package1".to_string()));
        assert!(all_packages.contains(&"@test/package2".to_string()));

        // Verify changeset was updated
        let changeset = manager.load("feature/test").await.unwrap();
        assert_eq!(changeset.packages.len(), 2, "Should have 2 packages in changeset");
        assert!(changeset.packages.contains(&"@test/package1".to_string()));
        assert!(changeset.packages.contains(&"@test/package2".to_string()));

        // Verify commits were added
        assert!(changeset.changes.contains(&commit1));
        assert!(changeset.changes.contains(&commit2));
    }

    #[tokio::test]
    async fn test_manager_add_commits_from_git_single_package() {
        let (temp_dir, repo) = setup_git_repo();
        setup_single_package(temp_dir.path());

        // Commit the setup
        repo.add_all().unwrap();
        repo.commit("Setup single package").unwrap();

        // Make changes
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/index.js"), "console.log('test');").unwrap();
        repo.add("src/index.js").unwrap();
        repo.commit("Update code").unwrap();
        let commit_hash = repo.get_current_sha().unwrap();

        // Create a changeset
        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".into(),
            ".changesets/history".into(),
            FileSystemManager::new(),
        );

        let config = ChangesetConfig {
            path: ".changesets".into(),
            history_path: ".changesets/history".into(),
            available_environments: vec!["production".to_string()],
            default_environments: vec!["production".to_string()],
        };

        let manager = ChangesetManager::with_storage(
            storage,
            temp_dir.path().to_path_buf(),
            Some(repo),
            config,
        );

        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Add initial package and setup commit to changeset so get_commits_since knows where to start
        let mut changeset = manager.load("feature/test").await.unwrap();
        changeset.add_package("single-package");
        // Get all commits and find the setup commit
        let all_commits =
            manager.git_repo().as_ref().unwrap().get_commits_since(None, &None).unwrap();
        // Find the "Setup single package" commit
        let setup_commit = all_commits
            .iter()
            .find(|c| c.message.contains("Setup single package"))
            .map(|c| &c.hash)
            .expect("Setup single package commit not found");
        changeset.add_commit(setup_commit);
        manager.update(&changeset).await.unwrap();

        // Use add_commits_from_git
        let summary = manager.add_commits_from_git("feature/test").await.unwrap();

        // Verify summary - expect only the 1 update commit
        assert_eq!(summary.commits_added, 1, "Should have added 1 commit");
        let total_packages = summary.new_packages.len() + summary.existing_packages.len();
        assert_eq!(total_packages, 1, "Should have affected 1 package");

        let all_packages: Vec<String> =
            summary.new_packages.iter().chain(summary.existing_packages.iter()).cloned().collect();
        assert!(all_packages.contains(&"single-package".to_string()));

        // Verify changeset was updated
        let changeset = manager.load("feature/test").await.unwrap();
        assert_eq!(changeset.packages.len(), 1, "Should have 1 package in changeset");
        assert!(changeset.packages.contains(&"single-package".to_string()));

        // Verify commit was added
        assert!(changeset.changes.contains(&commit_hash));
    }

    #[tokio::test]
    async fn test_manager_add_commits_from_git_no_git_repo() {
        let temp_dir = TempDir::new().unwrap();

        let storage = FileBasedChangesetStorage::new(
            temp_dir.path().to_path_buf(),
            ".changesets".into(),
            ".changesets/history".into(),
            FileSystemManager::new(),
        );

        let config = ChangesetConfig {
            path: ".changesets".into(),
            history_path: ".changesets/history".into(),
            available_environments: vec!["production".to_string()],
            default_environments: vec!["production".to_string()],
        };

        // Create manager without Git repo
        let manager =
            ChangesetManager::with_storage(storage, temp_dir.path().to_path_buf(), None, config);

        manager
            .create("feature/test", VersionBump::Minor, vec!["production".to_string()])
            .await
            .unwrap();

        // Attempt to use add_commits_from_git should fail
        let result = manager.add_commits_from_git("feature/test").await;
        assert!(result.is_err(), "Should fail when no Git repo is available");

        if let Err(ChangesetError::GitIntegration { operation, reason }) = result {
            assert_eq!(operation, "add commits from git");
            assert!(reason.contains("Git repository not available"));
        } else {
            panic!("Expected GitIntegration error");
        }
    }
}

// ============================================================================
// ChangesetHistory Tests
// ============================================================================

mod history_tests {
    use super::*;
    use crate::changeset::ChangesetHistory;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn test_history_list_all_empty() {
        let storage = MockStorage::new();
        let history = ChangesetHistory::new(Box::new(storage));

        let all = history.list_all().await.unwrap();
        assert_eq!(all.len(), 0);
    }

    #[tokio::test]
    async fn test_history_list_all_multiple() {
        let storage = MockStorage::new();

        // Archive multiple changesets
        for i in 1..=5 {
            let mut changeset = Changeset::new(
                format!("feature/history-{}", i),
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

        let history = ChangesetHistory::new(Box::new(storage));
        let all = history.list_all().await.unwrap();
        assert_eq!(all.len(), 5);

        // Verify all branches are present
        let branches: Vec<String> = all.iter().map(|a| a.changeset.branch.clone()).collect();
        for i in 1..=5 {
            let expected = format!("feature/history-{}", i);
            assert!(branches.contains(&expected), "Expected to find branch {}", expected);
        }
    }

    #[tokio::test]
    async fn test_history_list_all_sorted_by_date() {
        let storage = MockStorage::new();
        let now = Utc::now();

        // Archive changesets with different timestamps
        for i in 1..=3 {
            let mut changeset = Changeset::new(
                format!("feature/sorted-{}", i),
                VersionBump::Minor,
                vec!["production".to_string()],
            );
            changeset.add_package("package");

            storage.save(&changeset).await.unwrap();

            // Each release is older than the previous
            let applied_at = now - Duration::days(i);
            let mut release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
            );
            release_info.applied_at = applied_at;

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));
        let all = history.list_all().await.unwrap();

        // Verify sorting (most recent first)
        assert_eq!(all[0].changeset.branch, "feature/sorted-1");
        assert_eq!(all[1].changeset.branch, "feature/sorted-2");
        assert_eq!(all[2].changeset.branch, "feature/sorted-3");
    }

    #[tokio::test]
    async fn test_history_get_existing() {
        let storage = MockStorage::new();
        let mut changeset =
            Changeset::new("feature/get-test", VersionBump::Major, vec!["production".to_string()]);
        changeset.add_package("test-package");

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "ci-bot@example.com".to_string(),
            "abc123".to_string(),
            versions_map(vec![("test-package".to_string(), "2.0.0".to_string())]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));
        let archived = history.get("feature/get-test").await.unwrap();

        assert_eq!(archived.changeset.branch, "feature/get-test");
        assert_eq!(archived.changeset.bump, VersionBump::Major);
        assert_eq!(archived.release_info.applied_by, "ci-bot@example.com");
        assert_eq!(archived.release_info.git_commit, "abc123");
    }

    #[tokio::test]
    async fn test_history_get_nonexistent() {
        let storage = MockStorage::new();
        let history = ChangesetHistory::new(Box::new(storage));

        let result = history.get("nonexistent").await;
        assert!(result.is_err());

        match result {
            Err(ChangesetError::NotFound { branch }) => {
                assert_eq!(branch, "nonexistent");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_query_by_date_range() {
        let storage = MockStorage::new();
        let now = Utc::now();

        // Create changesets with different dates
        let dates = [
            now - Duration::days(10), // Old
            now - Duration::days(5),  // In range
            now - Duration::days(3),  // In range
            now - Duration::days(1),  // Recent
        ];

        for (i, date) in dates.iter().enumerate() {
            let mut changeset = Changeset::new(
                format!("feature/date-{}", i),
                VersionBump::Patch,
                vec!["production".to_string()],
            );
            changeset.add_package("package");

            storage.save(&changeset).await.unwrap();

            let mut release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
            );
            release_info.applied_at = *date;

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));

        // Query for dates 6 days ago to 2 days ago
        let start = now - Duration::days(6);
        let end = now - Duration::days(2);
        let results = history.query_by_date(start, end).await.unwrap();

        // Should only get the two in range
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|a| a.changeset.branch == "feature/date-1"));
        assert!(results.iter().any(|a| a.changeset.branch == "feature/date-2"));
    }

    #[tokio::test]
    async fn test_query_by_date_no_results() {
        let storage = MockStorage::new();
        let now = Utc::now();

        // Create a changeset from long ago
        let mut changeset =
            Changeset::new("feature/old", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("package");

        storage.save(&changeset).await.unwrap();

        let mut release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit".to_string(),
            versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
        );
        release_info.applied_at = now - Duration::days(100);

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));

        // Query recent dates
        let start = now - Duration::days(7);
        let end = now;
        let results = history.query_by_date(start, end).await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_query_by_package_single_match() {
        let storage = MockStorage::new();

        // Create changesets with different packages
        for i in 1..=3 {
            let mut changeset = Changeset::new(
                format!("feature/pkg-{}", i),
                VersionBump::Minor,
                vec!["production".to_string()],
            );

            if i == 2 {
                changeset.add_package("target-package");
            } else {
                changeset.add_package(format!("other-package-{}", i));
            }

            storage.save(&changeset).await.unwrap();

            let release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
            );

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));
        let results = history.query_by_package("target-package").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].changeset.branch, "feature/pkg-2");
    }

    #[tokio::test]
    async fn test_query_by_package_multiple_matches() {
        let storage = MockStorage::new();

        // Create multiple changesets with the same package
        for i in 1..=4 {
            let mut changeset = Changeset::new(
                format!("feature/common-pkg-{}", i),
                VersionBump::Patch,
                vec!["production".to_string()],
            );
            changeset.add_package("@myorg/core");
            changeset.add_package(format!("package-{}", i));

            storage.save(&changeset).await.unwrap();

            let release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![
                    ("@myorg/core".to_string(), format!("1.{}.0", i)),
                    (format!("package-{}", i), "1.0.0".to_string()),
                ]),
            );

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));
        let results = history.query_by_package("@myorg/core").await.unwrap();

        assert_eq!(results.len(), 4);
        for result in results {
            assert!(result.changeset.has_package("@myorg/core"));
        }
    }

    #[tokio::test]
    async fn test_query_by_package_no_matches() {
        let storage = MockStorage::new();

        let mut changeset =
            Changeset::new("feature/other", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("package1");

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit".to_string(),
            versions_map(vec![("package1".to_string(), "1.0.0".to_string())]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));
        let results = history.query_by_package("nonexistent-package").await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_query_by_environment_single() {
        let storage = MockStorage::new();

        // Create changesets with different environments
        let envs = [vec!["production"], vec!["staging"], vec!["production"], vec!["development"]];

        for (i, env) in envs.iter().enumerate() {
            let changeset = Changeset::new(
                format!("feature/env-{}", i),
                VersionBump::Minor,
                env.iter().map(|s| s.to_string()).collect(),
            );

            storage.save(&changeset).await.unwrap();

            let release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
            );

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));
        let results = history.query_by_environment("production").await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|a| a.changeset.branch == "feature/env-0"));
        assert!(results.iter().any(|a| a.changeset.branch == "feature/env-2"));
    }

    #[tokio::test]
    async fn test_query_by_environment_multiple() {
        let storage = MockStorage::new();

        // Create changeset with multiple environments
        let mut changeset = Changeset::new(
            "feature/multi-env",
            VersionBump::Major,
            vec!["staging".to_string(), "production".to_string()],
        );
        changeset.add_package("package");

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit".to_string(),
            versions_map(vec![("package".to_string(), "2.0.0".to_string())]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));

        // Should match when querying for either environment
        let staging_results = history.query_by_environment("staging").await.unwrap();
        assert_eq!(staging_results.len(), 1);

        let prod_results = history.query_by_environment("production").await.unwrap();
        assert_eq!(prod_results.len(), 1);
    }

    #[tokio::test]
    async fn test_query_by_environment_no_matches() {
        let storage = MockStorage::new();

        let changeset =
            Changeset::new("feature/prod-only", VersionBump::Minor, vec!["production".to_string()]);

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit".to_string(),
            versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));
        let results = history.query_by_environment("development").await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_query_by_bump_type() {
        let storage = MockStorage::new();

        // Create changesets with different bump types
        let bumps = [
            VersionBump::Major,
            VersionBump::Minor,
            VersionBump::Major,
            VersionBump::Patch,
            VersionBump::Minor,
        ];

        for (i, bump) in bumps.iter().enumerate() {
            let changeset = Changeset::new(
                format!("feature/bump-{}", i),
                *bump,
                vec!["production".to_string()],
            );

            storage.save(&changeset).await.unwrap();

            let release_info = ReleaseInfo::new(
                "user@example.com".to_string(),
                format!("commit{}", i),
                versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
            );

            storage.archive(&changeset, release_info).await.unwrap();
        }

        let history = ChangesetHistory::new(Box::new(storage));

        // Query for major bumps
        let major_results = history.query_by_bump(VersionBump::Major).await.unwrap();
        assert_eq!(major_results.len(), 2);
        assert!(major_results.iter().any(|a| a.changeset.branch == "feature/bump-0"));
        assert!(major_results.iter().any(|a| a.changeset.branch == "feature/bump-2"));

        // Query for minor bumps
        let minor_results = history.query_by_bump(VersionBump::Minor).await.unwrap();
        assert_eq!(minor_results.len(), 2);
        assert!(minor_results.iter().any(|a| a.changeset.branch == "feature/bump-1"));
        assert!(minor_results.iter().any(|a| a.changeset.branch == "feature/bump-4"));

        // Query for patch bumps
        let patch_results = history.query_by_bump(VersionBump::Patch).await.unwrap();
        assert_eq!(patch_results.len(), 1);
        assert_eq!(patch_results[0].changeset.branch, "feature/bump-3");
    }

    #[tokio::test]
    async fn test_query_by_bump_none() {
        let storage = MockStorage::new();

        let changeset =
            Changeset::new("feature/no-bump", VersionBump::None, vec!["production".to_string()]);

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit".to_string(),
            versions_map(vec![("package".to_string(), "1.0.0".to_string())]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));

        let results = history.query_by_bump(VersionBump::None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].changeset.branch, "feature/no-bump");

        // Other bumps should return empty
        let major_results = history.query_by_bump(VersionBump::Major).await.unwrap();
        assert_eq!(major_results.len(), 0);
    }

    #[tokio::test]
    async fn test_combined_queries() {
        let storage = MockStorage::new();
        let now = Utc::now();

        // Create a diverse set of changesets
        let mut changeset1 = Changeset::new(
            "feature/combined-1",
            VersionBump::Major,
            vec!["production".to_string()],
        );
        changeset1.add_package("@myorg/core");

        let mut changeset2 =
            Changeset::new("feature/combined-2", VersionBump::Minor, vec!["staging".to_string()]);
        changeset2.add_package("@myorg/utils");

        let mut changeset3 = Changeset::new(
            "feature/combined-3",
            VersionBump::Major,
            vec!["production".to_string()],
        );
        changeset3.add_package("@myorg/core");

        storage.save(&changeset1).await.unwrap();
        storage.save(&changeset2).await.unwrap();
        storage.save(&changeset3).await.unwrap();

        let mut release_info1 = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit1".to_string(),
            versions_map(vec![("@myorg/core".to_string(), "2.0.0".to_string())]),
        );
        release_info1.applied_at = now - Duration::days(2);

        let mut release_info2 = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit2".to_string(),
            versions_map(vec![("@myorg/utils".to_string(), "1.1.0".to_string())]),
        );
        release_info2.applied_at = now - Duration::days(5);

        let mut release_info3 = ReleaseInfo::new(
            "user@example.com".to_string(),
            "commit3".to_string(),
            versions_map(vec![("@myorg/core".to_string(), "3.0.0".to_string())]),
        );
        release_info3.applied_at = now - Duration::days(1);

        storage.archive(&changeset1, release_info1).await.unwrap();
        storage.archive(&changeset2, release_info2).await.unwrap();
        storage.archive(&changeset3, release_info3).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));

        // Query by package
        let core_releases = history.query_by_package("@myorg/core").await.unwrap();
        assert_eq!(core_releases.len(), 2);

        // Query by environment
        let prod_releases = history.query_by_environment("production").await.unwrap();
        assert_eq!(prod_releases.len(), 2);

        // Query by bump
        let major_releases = history.query_by_bump(VersionBump::Major).await.unwrap();
        assert_eq!(major_releases.len(), 2);

        // Query by date (last 3 days should get 2 releases)
        let start = now - Duration::days(3);
        let end = now;
        let recent_releases = history.query_by_date(start, end).await.unwrap();
        assert_eq!(recent_releases.len(), 2);
    }

    #[tokio::test]
    async fn test_history_with_release_version_info() {
        let storage = MockStorage::new();

        let mut changeset =
            Changeset::new("feature/versions", VersionBump::Minor, vec!["production".to_string()]);
        changeset.add_package("package1");
        changeset.add_package("package2");

        storage.save(&changeset).await.unwrap();

        let release_info = ReleaseInfo::new(
            "ci-bot@example.com".to_string(),
            "abc123def456".to_string(),
            versions_map(vec![
                ("package1".to_string(), "1.5.0".to_string()),
                ("package2".to_string(), "2.3.0".to_string()),
            ]),
        );

        storage.archive(&changeset, release_info).await.unwrap();

        let history = ChangesetHistory::new(Box::new(storage));
        let archived = history.get("feature/versions").await.unwrap();

        // Verify version information
        assert_eq!(archived.release_info.get_version("package1"), Some("1.5.0"));
        assert_eq!(archived.release_info.get_version("package2"), Some("2.3.0"));
        assert_eq!(archived.release_info.get_version("nonexistent"), None);
        assert_eq!(archived.release_info.package_count(), 2);
    }
}
