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
