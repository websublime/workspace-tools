//! # Changeset manager service
//!
//! ## What
//! Provides high-level service layer for changeset management operations,
//! coordinating between storage, validation, and business logic.
//!
//! ## How
//! - Uses ChangesetStorage for persistence operations
//! - Orchestrates changeset lifecycle (create, apply, archive)
//! - Validates changesets against configuration rules
//! - Provides convenient query and management APIs
//!
//! ## Why
//! The manager separates business logic from storage concerns, providing
//! a clean API for changeset operations while maintaining flexibility
//! in storage implementation.

use crate::{
    changeset::{Changeset, ChangesetStorage, ReleaseInfo},
    error::{ChangesetError, ChangesetResult},
};
use chrono::{DateTime, Utc};

/// Changeset manager service.
///
/// Provides high-level operations for changeset management, coordinating
/// between storage and business logic. Acts as the primary interface for
/// all changeset-related operations.
///
/// # Architecture
///
/// The manager uses dependency injection for storage, allowing different
/// storage implementations to be used while maintaining the same API.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::{ChangesetManager, FileBasedChangesetStorage, Changeset};
/// use sublime_pkg_tools::config::ChangesetConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ChangesetConfig::default();
/// let fs = FileSystemManager::new();
/// let root = PathBuf::from("/path/to/project");
/// let storage = FileBasedChangesetStorage::new(fs, root, config.clone());
///
/// let manager = ChangesetManager::new(storage);
///
/// // Create a changeset
/// let changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());
/// let id = manager.create(&changeset).await?;
///
/// // List pending
/// let pending = manager.list_pending().await?;
/// println!("Pending changesets: {:?}", pending);
/// # Ok(())
/// # }
/// ```
pub struct ChangesetManager<S>
where
    S: ChangesetStorage,
{
    storage: S,
}

impl<S> ChangesetManager<S>
where
    S: ChangesetStorage,
{
    /// Creates a new changeset manager.
    ///
    /// # Arguments
    ///
    /// * `storage` - The storage implementation to use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::{ChangesetManager, FileBasedChangesetStorage};
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let config = ChangesetConfig::default();
    /// let fs = FileSystemManager::new();
    /// let root = PathBuf::from("/path/to/project");
    /// let storage = FileBasedChangesetStorage::new(fs, root, config);
    ///
    /// let manager = ChangesetManager::new(storage);
    /// ```
    #[must_use]
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Creates a new changeset.
    ///
    /// Validates and persists the changeset, returning its generated ID.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to create
    ///
    /// # Returns
    ///
    /// The generated changeset ID
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset validation fails
    /// - Changeset already exists
    /// - Storage operation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetManager};
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let changeset = Changeset::new(
    ///     "feat/user-authentication".to_string(),
    ///     "developer@example.com".to_string(),
    /// );
    ///
    /// let id = manager.create(&changeset).await?;
    /// println!("Created changeset: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, changeset: &Changeset) -> ChangesetResult<String> {
        // Validate that changeset is pending (not already applied)
        if changeset.is_applied() {
            return Err(ChangesetError::CreationFailed {
                branch: changeset.branch.clone(),
                reason: "Cannot create changeset that is already applied".to_string(),
            });
        }

        // Validate packages are present
        if changeset.packages.is_empty() {
            return Err(ChangesetError::CreationFailed {
                branch: changeset.branch.clone(),
                reason: "Changeset must contain at least one package".to_string(),
            });
        }

        // Delegate to storage
        self.storage.save(changeset).await
    }

    /// Loads a changeset by ID.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to load
    ///
    /// # Returns
    ///
    /// The loaded changeset
    ///
    /// # Errors
    ///
    /// Returns error if changeset not found or load fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let changeset = manager.load("feat-auth-20240115T103000Z").await?;
    /// println!("Loaded changeset for branch: {}", changeset.branch);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load(&self, changeset_id: &str) -> ChangesetResult<Changeset> {
        self.storage.load(changeset_id).await
    }

    /// Checks if a changeset exists.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to check
    ///
    /// # Returns
    ///
    /// `true` if the changeset exists (pending or history)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// if manager.exists("feat-auth-20240115T103000Z").await? {
    ///     println!("Changeset exists");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists(&self, changeset_id: &str) -> ChangesetResult<bool> {
        self.storage.exists(changeset_id).await
    }

    /// Deletes a pending changeset.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to delete
    ///
    /// # Errors
    ///
    /// Returns error if changeset not found or delete fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// manager.delete("feat-auth-20240115T103000Z").await?;
    /// println!("Changeset deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, changeset_id: &str) -> ChangesetResult<()> {
        self.storage.delete(changeset_id).await
    }

    /// Lists all pending changeset IDs.
    ///
    /// Returns IDs sorted by creation time (newest first).
    ///
    /// # Returns
    ///
    /// Vector of pending changeset IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let pending = manager.list_pending().await?;
    /// println!("Found {} pending changesets", pending.len());
    /// for id in pending {
    ///     println!("  - {}", id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_pending(&self) -> ChangesetResult<Vec<String>> {
        self.storage.list_pending().await
    }

    /// Lists all applied changesets from history.
    ///
    /// Returns IDs sorted by creation time (newest first).
    ///
    /// # Returns
    ///
    /// Vector of archived changeset IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let history = manager.list_history().await?;
    /// println!("Found {} historical changesets", history.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_history(&self) -> ChangesetResult<Vec<String>> {
        self.storage.list_history().await
    }

    /// Applies release information to a changeset and archives it.
    ///
    /// This marks the changeset as applied and moves it to history.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to apply
    /// * `release_info` - Release information to attach
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset not found
    /// - Changeset already applied
    /// - Storage operation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::{ChangesetManager, ReleaseInfo, EnvironmentRelease};
    /// use std::collections::HashMap;
    /// use chrono::Utc;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let mut environments = HashMap::new();
    /// environments.insert(
    ///     "dev".to_string(),
    ///     EnvironmentRelease {
    ///         released_at: Utc::now(),
    ///         tag: "v1.3.0-dev".to_string(),
    ///     },
    /// );
    ///
    /// let release_info = ReleaseInfo {
    ///     applied_at: Utc::now(),
    ///     applied_by: "ci-bot".to_string(),
    ///     git_commit: "abc123".to_string(),
    ///     environments_released: environments,
    /// };
    ///
    /// manager.apply("feat-auth-20240115T103000Z", release_info).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply(
        &self,
        changeset_id: &str,
        release_info: ReleaseInfo,
    ) -> ChangesetResult<()> {
        // Load changeset
        let mut changeset = self.storage.load(changeset_id).await?;

        // Check if already applied
        if changeset.is_applied() {
            return Err(ChangesetError::ApplicationFailed {
                changeset_id: changeset_id.to_string(),
                reason: "Changeset has already been applied".to_string(),
            });
        }

        // Validate release info
        release_info.validate().map_err(|e| ChangesetError::ApplicationFailed {
            changeset_id: changeset_id.to_string(),
            reason: format!("Invalid release info: {}", e),
        })?;

        // Apply release info
        changeset.apply_release_info(release_info);

        // Delete the original pending changeset
        self.storage.delete(changeset_id).await?;

        // Save the updated changeset (with release info) back to pending
        // This ensures the archive operation reads the updated version
        self.storage.save(&changeset).await?;

        // Archive to history (this will delete from pending and move to history)
        self.storage.archive(changeset_id).await
    }

    /// Loads a changeset from history.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - ID of the changeset to load from history
    ///
    /// # Returns
    ///
    /// The loaded changeset from history
    ///
    /// # Errors
    ///
    /// Returns error if changeset not found in history
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let changeset = manager.load_from_history("feat-auth-20240115T103000Z").await?;
    /// assert!(changeset.is_applied());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_from_history(&self, changeset_id: &str) -> ChangesetResult<Changeset> {
        self.storage.load_from_history(changeset_id).await
    }

    /// Queries pending changesets by branch name.
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name to filter by (exact match)
    ///
    /// # Returns
    ///
    /// Vector of matching changeset IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let changesets = manager.query_by_branch("feat/auth").await?;
    /// println!("Found {} changesets for branch feat/auth", changesets.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_branch(&self, branch: &str) -> ChangesetResult<Vec<String>> {
        self.storage.query_by_branch(branch).await
    }

    /// Queries history by date range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start date (inclusive)
    /// * `end` - End date (inclusive)
    ///
    /// # Returns
    ///
    /// Vector of changeset IDs within the date range
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    /// use chrono::{Utc, Duration};
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let end = Utc::now();
    /// let start = end - Duration::days(7);
    ///
    /// let changesets = manager.query_history_by_date(start, end).await?;
    /// println!("Found {} changesets in the last week", changesets.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_history_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ChangesetResult<Vec<String>> {
        self.storage.query_history_by_date(start, end).await
    }

    /// Queries history by package name.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Package name to search for
    ///
    /// # Returns
    ///
    /// Vector of changeset IDs that include the package
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let changesets = manager.query_history_by_package("@myorg/auth-service").await?;
    /// println!("Found {} releases for auth-service", changesets.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_history_by_package(
        &self,
        package_name: &str,
    ) -> ChangesetResult<Vec<String>> {
        self.storage.query_history_by_package(package_name).await
    }

    /// Gets the latest pending changeset for a specific branch.
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name to search for
    ///
    /// # Returns
    ///
    /// The changeset ID of the latest changeset, or `None` if not found
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// if let Some(id) = manager.get_latest_for_branch("feat/auth").await? {
    ///     println!("Latest changeset: {}", id);
    /// } else {
    ///     println!("No changesets found for branch");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_for_branch(&self, branch: &str) -> ChangesetResult<Option<String>> {
        self.storage.get_latest_for_branch(branch).await
    }

    /// Gets a summary of all changesets (pending and history).
    ///
    /// # Returns
    ///
    /// A summary with counts and basic statistics
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetManager;
    ///
    /// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: sublime_pkg_tools::changeset::ChangesetStorage
    /// # {
    /// let summary = manager.get_summary().await?;
    /// println!("Pending: {}, History: {}", summary.pending_count, summary.history_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_summary(&self) -> ChangesetResult<ChangesetSummary> {
        let pending = self.list_pending().await?;
        let history = self.list_history().await?;

        Ok(ChangesetSummary {
            pending_count: pending.len(),
            history_count: history.len(),
            pending_ids: pending,
            history_ids: history,
        })
    }
}

/// Summary of changeset statistics.
///
/// Provides counts and lists of both pending and archived changesets.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::ChangesetManager;
///
/// # async fn example<S>(manager: &ChangesetManager<S>) -> Result<(), Box<dyn std::error::Error>>
/// # where S: sublime_pkg_tools::changeset::ChangesetStorage
/// # {
/// let summary = manager.get_summary().await?;
/// println!("Status Report:");
/// println!("  Pending: {}", summary.pending_count);
/// println!("  Archived: {}", summary.history_count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ChangesetSummary {
    /// Number of pending changesets
    pub pending_count: usize,

    /// Number of archived changesets
    pub history_count: usize,

    /// List of pending changeset IDs
    pub pending_ids: Vec<String>,

    /// List of archived changeset IDs
    pub history_ids: Vec<String>,
}

impl ChangesetSummary {
    /// Checks if there are any pending changesets.
    ///
    /// # Returns
    ///
    /// `true` if pending changesets exist
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::changeset::ChangesetSummary;
    ///
    /// let summary = ChangesetSummary {
    ///     pending_count: 5,
    ///     history_count: 10,
    ///     pending_ids: vec!["id1".to_string()],
    ///     history_ids: vec![],
    /// };
    ///
    /// assert!(summary.has_pending());
    /// ```
    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.pending_count > 0
    }

    /// Checks if there is any history.
    ///
    /// # Returns
    ///
    /// `true` if archived changesets exist
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::changeset::ChangesetSummary;
    ///
    /// let summary = ChangesetSummary {
    ///     pending_count: 0,
    ///     history_count: 10,
    ///     pending_ids: vec![],
    ///     history_ids: vec!["id1".to_string()],
    /// };
    ///
    /// assert!(summary.has_history());
    /// ```
    #[must_use]
    pub fn has_history(&self) -> bool {
        self.history_count > 0
    }

    /// Gets the total number of changesets (pending + history).
    ///
    /// # Returns
    ///
    /// Total count of all changesets
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::changeset::ChangesetSummary;
    ///
    /// let summary = ChangesetSummary {
    ///     pending_count: 5,
    ///     history_count: 10,
    ///     pending_ids: vec![],
    ///     history_ids: vec![],
    /// };
    ///
    /// assert_eq!(summary.total_count(), 15);
    /// ```
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.pending_count + self.history_count
    }
}
