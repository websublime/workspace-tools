//! Changeset storage abstraction and trait definition.
//!
//! **What**: Defines the `ChangesetStorage` trait that abstracts changeset persistence,
//! enabling pluggable storage implementations (file-based, database, in-memory, etc.).
//!
//! **How**: Provides an async trait with methods for saving, loading, deleting, listing,
//! and archiving changesets. All methods return `ChangesetResult` for consistent error handling.
//! The trait is `Send + Sync` to support concurrent async operations.
//!
//! **Why**: Abstracts storage concerns from changeset management logic, allowing different
//! storage backends to be swapped without changing the core changeset functionality. This
//! enables testing with in-memory storage and production use with file-based or database storage.
//!
//! # Storage Operations
//!
//! ## Basic Operations
//!
//! - **save**: Persist a changeset (create or update)
//! - **load**: Retrieve a changeset by branch name
//! - **exists**: Check if a changeset exists for a branch
//! - **delete**: Remove a changeset from storage
//!
//! ## List Operations
//!
//! - **list_pending**: Get all active (non-archived) changesets
//! - **list_archived**: Get all archived changesets with release metadata
//!
//! ## Archive Operations
//!
//! - **archive**: Move a changeset to history with release information
//! - **load_archived**: Retrieve an archived changeset by branch name
//!
//! # Examples
//!
//! ## Implementing a custom storage backend
//!
//! ```rust
//! use sublime_pkg_tools::changeset::ChangesetStorage;
//! use sublime_pkg_tools::types::{Changeset, ArchivedChangeset, ReleaseInfo};
//! use sublime_pkg_tools::error::ChangesetResult;
//! use async_trait::async_trait;
//! use std::collections::HashMap;
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! /// In-memory storage implementation for testing
//! pub struct InMemoryStorage {
//!     pending: Arc<RwLock<HashMap<String, Changeset>>>,
//!     archived: Arc<RwLock<HashMap<String, ArchivedChangeset>>>,
//! }
//!
//! impl InMemoryStorage {
//!     pub fn new() -> Self {
//!         Self {
//!             pending: Arc::new(RwLock::new(HashMap::new())),
//!             archived: Arc::new(RwLock::new(HashMap::new())),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl ChangesetStorage for InMemoryStorage {
//!     async fn save(&self, changeset: &Changeset) -> ChangesetResult<()> {
//!         let mut pending = self.pending.write().await;
//!         pending.insert(changeset.branch.clone(), changeset.clone());
//!         Ok(())
//!     }
//!
//!     async fn load(&self, branch: &str) -> ChangesetResult<Changeset> {
//!         let pending = self.pending.read().await;
//!         pending.get(branch).cloned().ok_or_else(|| {
//!             sublime_pkg_tools::error::ChangesetError::NotFound {
//!                 branch: branch.to_string(),
//!             }
//!         })
//!     }
//!
//!     async fn exists(&self, branch: &str) -> ChangesetResult<bool> {
//!         let pending = self.pending.read().await;
//!         Ok(pending.contains_key(branch))
//!     }
//!
//!     async fn delete(&self, branch: &str) -> ChangesetResult<()> {
//!         let mut pending = self.pending.write().await;
//!         pending.remove(branch);
//!         Ok(())
//!     }
//!
//!     async fn list_pending(&self) -> ChangesetResult<Vec<Changeset>> {
//!         let pending = self.pending.read().await;
//!         Ok(pending.values().cloned().collect())
//!     }
//!
//!     async fn archive(
//!         &self,
//!         changeset: &Changeset,
//!         release_info: ReleaseInfo,
//!     ) -> ChangesetResult<()> {
//!         let mut pending = self.pending.write().await;
//!         let mut archived = self.archived.write().await;
//!
//!         pending.remove(&changeset.branch);
//!         let archived_changeset = ArchivedChangeset::new(
//!             changeset.clone(),
//!             release_info,
//!         );
//!         archived.insert(changeset.branch.clone(), archived_changeset);
//!         Ok(())
//!     }
//!
//!     async fn load_archived(&self, branch: &str) -> ChangesetResult<ArchivedChangeset> {
//!         let archived = self.archived.read().await;
//!         archived.get(branch).cloned().ok_or_else(|| {
//!             sublime_pkg_tools::error::ChangesetError::NotFound {
//!                 branch: branch.to_string(),
//!             }
//!         })
//!     }
//!
//!     async fn list_archived(&self) -> ChangesetResult<Vec<ArchivedChangeset>> {
//!         let archived = self.archived.read().await;
//!         Ok(archived.values().cloned().collect())
//!     }
//! }
//! ```
//!
//! ## Using the storage trait
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changeset::ChangesetStorage;
//! use sublime_pkg_tools::types::{Changeset, VersionBump, ReleaseInfo};
//!
//! async fn example<S: ChangesetStorage>(storage: &S) -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and save a changeset
//!     let changeset = Changeset::new(
//!         "feature/new-api",
//!         VersionBump::Minor,
//!         vec!["production".to_string()],
//!     );
//!     storage.save(&changeset).await?;
//!
//!     // Check if it exists
//!     let exists = storage.exists("feature/new-api").await?;
//!     assert!(exists);
//!
//!     // Load it back
//!     let loaded = storage.load("feature/new-api").await?;
//!     assert_eq!(loaded.branch, "feature/new-api");
//!
//!     // List all pending changesets
//!     let pending = storage.list_pending().await?;
//!     println!("Found {} pending changesets", pending.len());
//!
//!     // Archive it when released
//!     let release_info = ReleaseInfo::new(
//!         "user@example.com".to_string(),
//!         Some("abc123".to_string()),
//!         vec![("@myorg/api".to_string(), "1.2.0".to_string())],
//!     );
//!     storage.archive(&changeset, release_info).await?;
//!
//!     // Load from archive
//!     let archived = storage.load_archived("feature/new-api").await?;
//!     println!("Released at: {}", archived.release_info.applied_at);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Storage Implementations
//!
//! The following implementations are provided:
//!
//! - **FileBasedChangesetStorage** (Story 6.2): File-based storage using JSON files
//! - **InMemoryStorage** (for testing): In-memory storage using hash maps
//!
//! # Thread Safety
//!
//! The `ChangesetStorage` trait requires `Send + Sync`, making implementations safe to
//! share across async tasks. Implementations must ensure thread-safe access to underlying
//! storage mechanisms using appropriate synchronization primitives (e.g., `RwLock`, `Mutex`).
//!
//! # Error Handling
//!
//! All methods return `ChangesetResult<T>`, which is an alias for `Result<T, ChangesetError>`.
//! Implementations should use appropriate error variants from `ChangesetError` to provide
//! clear error messages with context.

use crate::error::ChangesetResult;
use crate::types::{ArchivedChangeset, Changeset, ReleaseInfo};
use async_trait::async_trait;

/// Trait for changeset storage operations.
///
/// This trait abstracts the persistence layer for changesets, enabling different
/// storage backends (filesystem, database, in-memory) to be used interchangeably.
/// All implementations must be thread-safe (`Send + Sync`) and use async operations.
///
/// # Lifecycle
///
/// A changeset goes through the following lifecycle states:
///
/// 1. **Created**: Saved via `save()` to pending storage
/// 2. **Updated**: Modified and saved again via `save()`
/// 3. **Archived**: Moved to history via `archive()` with release metadata
///
/// Once archived, a changeset is removed from pending storage and can only be
/// accessed via `load_archived()` and `list_archived()`.
///
/// # Concurrency
///
/// Implementations should handle concurrent access appropriately:
/// - Multiple reads can occur simultaneously
/// - Writes should be atomic and properly synchronized
/// - Archive operations should be transactional (remove from pending + add to archive)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changeset::ChangesetStorage;
/// use sublime_pkg_tools::types::{Changeset, VersionBump};
///
/// async fn save_changeset<S: ChangesetStorage>(
///     storage: &S,
///     branch: &str,
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     let changeset = Changeset::new(
///         branch,
///         VersionBump::Patch,
///         vec!["production".to_string()],
///     );
///
///     storage.save(&changeset).await?;
///     println!("Changeset saved for branch: {}", branch);
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ChangesetStorage: Send + Sync {
    /// Saves a changeset to storage.
    ///
    /// This method persists a changeset, creating a new entry if it doesn't exist
    /// or updating an existing one. The changeset's `updated_at` timestamp should
    /// be set before calling this method to track modification time.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to save
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the changeset was successfully saved, or a `ChangesetError`
    /// if the operation failed.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::ValidationFailed` - If the changeset fails validation
    /// * `ChangesetError::StorageError` - If writing to storage fails
    /// * `ChangesetError::SerializationError` - If serialization fails
    /// * `ChangesetError::PermissionDenied` - If lacking write permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feature/oauth",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    /// changeset.add_package("@myorg/auth");
    /// changeset.touch(); // Update timestamp
    ///
    /// storage.save(&changeset).await?;
    /// ```
    async fn save(&self, changeset: &Changeset) -> ChangesetResult<()>;

    /// Loads a changeset from storage by branch name.
    ///
    /// Retrieves the changeset associated with the specified branch name from
    /// pending storage. This does not load archived changesets; use `load_archived()`
    /// for that purpose.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name identifying the changeset
    ///
    /// # Returns
    ///
    /// Returns the loaded `Changeset` if found, or a `ChangesetError` if not found
    /// or if loading fails.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::NotFound` - If no changeset exists for the branch
    /// * `ChangesetError::StorageError` - If reading from storage fails
    /// * `ChangesetError::SerializationError` - If deserialization fails
    /// * `ChangesetError::PermissionDenied` - If lacking read permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let changeset = storage.load("feature/oauth").await?;
    /// println!("Loaded changeset with {} packages", changeset.packages.len());
    /// ```
    async fn load(&self, branch: &str) -> ChangesetResult<Changeset>;

    /// Checks if a changeset exists for the given branch.
    ///
    /// This is a lightweight operation that checks for the existence of a changeset
    /// without loading its full content. Useful for conditional logic and validation.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the changeset exists, `Ok(false)` if it doesn't,
    /// or a `ChangesetError` if the check operation fails.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::StorageError` - If the existence check fails
    /// * `ChangesetError::PermissionDenied` - If lacking read permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if storage.exists("feature/oauth").await? {
    ///     println!("Changeset already exists");
    /// } else {
    ///     println!("Creating new changeset");
    /// }
    /// ```
    async fn exists(&self, branch: &str) -> ChangesetResult<bool>;

    /// Deletes a changeset from storage.
    ///
    /// Permanently removes a pending changeset from storage. This operation cannot
    /// be undone. Archived changesets cannot be deleted through this method; they
    /// must be managed separately through the history storage.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name identifying the changeset to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the changeset was successfully deleted, or a `ChangesetError`
    /// if the operation failed. Returns `Ok(())` even if the changeset doesn't exist
    /// (idempotent operation).
    ///
    /// # Errors
    ///
    /// * `ChangesetError::StorageError` - If deletion fails
    /// * `ChangesetError::PermissionDenied` - If lacking delete permissions
    /// * `ChangesetError::LockFailed` - If unable to acquire exclusive lock
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// storage.delete("feature/old-branch").await?;
    /// println!("Changeset deleted");
    /// ```
    async fn delete(&self, branch: &str) -> ChangesetResult<()>;

    /// Lists all pending (non-archived) changesets.
    ///
    /// Retrieves all changesets currently in pending storage. The order of changesets
    /// in the returned vector is implementation-dependent (may be sorted by branch name,
    /// creation time, or unordered).
    ///
    /// # Returns
    ///
    /// Returns a vector of all pending changesets, or a `ChangesetError` if the
    /// listing operation fails. Returns an empty vector if no changesets exist.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::StorageError` - If listing fails
    /// * `ChangesetError::SerializationError` - If deserialization of any changeset fails
    /// * `ChangesetError::PermissionDenied` - If lacking read permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pending = storage.list_pending().await?;
    /// for changeset in pending {
    ///     println!("Branch: {}, Packages: {}",
    ///              changeset.branch,
    ///              changeset.packages.len());
    /// }
    /// ```
    async fn list_pending(&self) -> ChangesetResult<Vec<Changeset>>;

    /// Archives a changeset with release information.
    ///
    /// Moves a changeset from pending storage to archive storage, adding release
    /// metadata. This is typically called after successfully applying versions and
    /// creating a release. The operation should be atomic: the changeset is removed
    /// from pending storage and added to archive storage as a single transaction.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to archive
    /// * `release_info` - Metadata about the release (who, when, versions applied)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the changeset was successfully archived, or a `ChangesetError`
    /// if the operation failed.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::ArchiveError` - If archiving fails
    /// * `ChangesetError::StorageError` - If storage operations fail
    /// * `ChangesetError::SerializationError` - If serialization fails
    /// * `ChangesetError::PermissionDenied` - If lacking necessary permissions
    /// * `ChangesetError::AlreadyExists` - If an archived changeset with this branch already exists
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::types::ReleaseInfo;
    ///
    /// let changeset = storage.load("feature/oauth").await?;
    /// let release_info = ReleaseInfo::new(
    ///     "ci-bot@example.com".to_string(),
    ///     Some("abc123def456".to_string()),
    ///     vec![
    ///         ("@myorg/auth".to_string(), "2.0.0".to_string()),
    ///         ("@myorg/core".to_string(), "1.5.0".to_string()),
    ///     ],
    /// );
    ///
    /// storage.archive(&changeset, release_info).await?;
    /// println!("Changeset archived successfully");
    /// ```
    async fn archive(
        &self,
        changeset: &Changeset,
        release_info: ReleaseInfo,
    ) -> ChangesetResult<()>;

    /// Loads an archived changeset by branch name.
    ///
    /// Retrieves an archived changeset from history storage. This includes both the
    /// original changeset data and the release metadata added during archiving.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name identifying the archived changeset
    ///
    /// # Returns
    ///
    /// Returns the `ArchivedChangeset` if found, or a `ChangesetError` if not found
    /// or if loading fails.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::NotFound` - If no archived changeset exists for the branch
    /// * `ChangesetError::StorageError` - If reading from storage fails
    /// * `ChangesetError::SerializationError` - If deserialization fails
    /// * `ChangesetError::PermissionDenied` - If lacking read permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let archived = storage.load_archived("feature/oauth").await?;
    /// println!("Released at: {}", archived.release_info.applied_at);
    /// println!("Released by: {}", archived.release_info.applied_by);
    /// for (pkg, version) in &archived.release_info.versions {
    ///     println!("  {} -> {}", pkg, version);
    /// }
    /// ```
    async fn load_archived(&self, branch: &str) -> ChangesetResult<ArchivedChangeset>;

    /// Lists all archived changesets.
    ///
    /// Retrieves all changesets from archive storage, including their release metadata.
    /// The order of changesets in the returned vector is implementation-dependent
    /// (may be sorted by archive time, branch name, or unordered).
    ///
    /// # Returns
    ///
    /// Returns a vector of all archived changesets, or a `ChangesetError` if the
    /// listing operation fails. Returns an empty vector if no archived changesets exist.
    ///
    /// # Errors
    ///
    /// * `ChangesetError::StorageError` - If listing fails
    /// * `ChangesetError::SerializationError` - If deserialization of any archived changeset fails
    /// * `ChangesetError::PermissionDenied` - If lacking read permissions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let archived = storage.list_archived().await?;
    /// println!("Found {} archived changesets", archived.len());
    /// for changeset in archived {
    ///     println!("Branch: {}, Released: {}",
    ///              changeset.changeset.branch,
    ///              changeset.release_info.applied_at);
    /// }
    /// ```
    async fn list_archived(&self) -> ChangesetResult<Vec<ArchivedChangeset>>;
}
