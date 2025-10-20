//! Changeset history and query functionality.
//!
//! **What**: Provides the `ChangesetHistory` struct for querying archived changesets with
//! flexible filtering options by date range, package, environment, and bump type.
//!
//! **How**: Uses the `ChangesetStorage` trait to load archived changesets and provides
//! query methods that filter the results based on various criteria. All queries operate
//! on the complete list of archived changesets and filter in memory.
//!
//! **Why**: To enable users to search through release history, audit past changes,
//! and understand the versioning history of packages over time.
//!
//! # Examples
//!
//! ## Query by date range
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changeset::ChangesetHistory;
//! use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use chrono::{Utc, Duration};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let storage = FileBasedChangesetStorage::new(
//!     workspace_root.clone(),
//!     PathBuf::from(".changesets"),
//!     PathBuf::from(".changesets/history"),
//!     fs
//! );
//!
//! let history = ChangesetHistory::new(Box::new(storage));
//!
//! // Query releases from the last 30 days
//! let start = Utc::now() - Duration::days(30);
//! let end = Utc::now();
//! let recent_releases = history.query_by_date(start, end).await?;
//!
//! println!("Found {} releases in the last 30 days", recent_releases.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Query by package
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changeset::ChangesetHistory;
//! # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
//! # use sublime_standard_tools::filesystem::FileSystemManager;
//! # use std::path::PathBuf;
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let storage = FileBasedChangesetStorage::new(
//! #     workspace_root.clone(),
//! #     PathBuf::from(".changesets"),
//! #     PathBuf::from(".changesets/history"),
//! #     fs
//! # );
//! # let history = ChangesetHistory::new(Box::new(storage));
//! // Find all releases that included a specific package
//! let pkg_releases = history.query_by_package("@myorg/core").await?;
//!
//! for archived in pkg_releases {
//!     println!(
//!         "Released on {} with bump {:?}",
//!         archived.release_info.applied_at,
//!         archived.changeset.bump
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Query by environment
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changeset::ChangesetHistory;
//! # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
//! # use sublime_standard_tools::filesystem::FileSystemManager;
//! # use std::path::PathBuf;
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let workspace_root = PathBuf::from(".");
//! # let fs = FileSystemManager::new();
//! # let storage = FileBasedChangesetStorage::new(
//! #     workspace_root.clone(),
//! #     PathBuf::from(".changesets"),
//! #     PathBuf::from(".changesets/history"),
//! #     fs
//! # );
//! # let history = ChangesetHistory::new(Box::new(storage));
//! // Find all production releases
//! let prod_releases = history.query_by_environment("production").await?;
//!
//! println!("Found {} production releases", prod_releases.len());
//! # Ok(())
//! # }
//! ```

use crate::changeset::ChangesetStorage;
use crate::error::ChangesetResult;
use crate::types::{ArchivedChangeset, VersionBump};
use chrono::{DateTime, Utc};

/// Query interface for changeset history.
///
/// This struct provides methods to search through archived changesets using various
/// filtering criteria. All queries are performed by loading the complete archive list
/// and filtering in memory.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changeset::{ChangesetHistory, FileBasedChangesetStorage};
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let fs = FileSystemManager::new();
/// let storage = FileBasedChangesetStorage::new(
///     workspace_root.clone(),
///     PathBuf::from(".changesets"),
///     PathBuf::from(".changesets/history"),
///     fs
/// );
///
/// let history = ChangesetHistory::new(Box::new(storage));
///
/// // List all archived changesets
/// let all_archives = history.list_all().await?;
/// println!("Total archived releases: {}", all_archives.len());
/// # Ok(())
/// # }
/// ```
pub struct ChangesetHistory {
    /// Storage backend for accessing archived changesets
    storage: Box<dyn ChangesetStorage>,
}

impl ChangesetHistory {
    /// Creates a new `ChangesetHistory` instance with the specified storage backend.
    ///
    /// # Arguments
    ///
    /// * `storage` - Storage backend implementing the `ChangesetStorage` trait
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changeset::{ChangesetHistory, FileBasedChangesetStorage};
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let workspace_root = PathBuf::from(".");
    /// let fs = FileSystemManager::new();
    /// let storage = FileBasedChangesetStorage::new(
    ///     workspace_root.clone(),
    ///     PathBuf::from(".changesets"),
    ///     PathBuf::from(".changesets/history"),
    ///     fs
    /// );
    ///
    /// let history = ChangesetHistory::new(Box::new(storage));
    /// ```
    #[must_use]
    pub fn new(storage: Box<dyn ChangesetStorage>) -> Self {
        Self { storage }
    }

    /// Lists all archived changesets.
    ///
    /// Returns a vector of all archived changesets in the history, sorted by
    /// applied date in descending order (most recent first).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The history directory cannot be read
    /// - Any archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// let all_archives = history.list_all().await?;
    ///
    /// for archived in all_archives {
    ///     println!(
    ///         "Branch: {}, Applied: {}",
    ///         archived.changeset.branch,
    ///         archived.release_info.applied_at
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_all(&self) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let mut archives = self.storage.list_archived().await?;

        // Sort by applied date, most recent first
        archives.sort_by(|a, b| b.release_info.applied_at.cmp(&a.release_info.applied_at));

        Ok(archives)
    }

    /// Gets a specific archived changeset by branch name.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name of the changeset to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No archived changeset exists for the specified branch
    /// - The archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// let archived = history.get("feature/new-api").await?;
    ///
    /// println!("Released by: {}", archived.release_info.applied_by);
    /// println!("Git commit: {}", archived.release_info.git_commit);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, branch: &str) -> ChangesetResult<ArchivedChangeset> {
        self.storage.load_archived(branch).await
    }

    /// Queries changesets by date range.
    ///
    /// Returns all archived changesets where the release was applied within the specified
    /// date range (inclusive). Results are sorted by applied date in descending order.
    ///
    /// # Arguments
    ///
    /// * `from` - Start of the date range (inclusive)
    /// * `to` - End of the date range (inclusive)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The history directory cannot be read
    /// - Any archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// use chrono::{Utc, Duration};
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// // Get releases from the last week
    /// let start = Utc::now() - Duration::days(7);
    /// let end = Utc::now();
    /// let recent = history.query_by_date(start, end).await?;
    ///
    /// println!("Releases in the last week: {}", recent.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_date(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let all_archives = self.list_all().await?;

        let filtered: Vec<ArchivedChangeset> = all_archives
            .into_iter()
            .filter(|archived| {
                let applied_at = archived.release_info.applied_at;
                applied_at >= from && applied_at <= to
            })
            .collect();

        Ok(filtered)
    }

    /// Queries changesets by package name.
    ///
    /// Returns all archived changesets that include the specified package in their
    /// packages list. Results are sorted by applied date in descending order.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name to search for
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The history directory cannot be read
    /// - Any archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// let pkg_history = history.query_by_package("@myorg/core").await?;
    ///
    /// println!("Package @myorg/core appeared in {} releases", pkg_history.len());
    ///
    /// for archived in pkg_history {
    ///     let version = archived.release_info.get_version("@myorg/core");
    ///     println!(
    ///         "Released version {} on {}",
    ///         version.unwrap_or("unknown"),
    ///         archived.release_info.applied_at
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_package(&self, package: &str) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let all_archives = self.list_all().await?;

        let filtered: Vec<ArchivedChangeset> = all_archives
            .into_iter()
            .filter(|archived| archived.changeset.has_package(package))
            .collect();

        Ok(filtered)
    }

    /// Queries changesets by environment.
    ///
    /// Returns all archived changesets that target the specified environment.
    /// Results are sorted by applied date in descending order.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment name to search for
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The history directory cannot be read
    /// - Any archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// // Find all production releases
    /// let prod_releases = history.query_by_environment("production").await?;
    ///
    /// println!("Production releases: {}", prod_releases.len());
    ///
    /// // Find all staging releases
    /// let staging_releases = history.query_by_environment("staging").await?;
    ///
    /// println!("Staging releases: {}", staging_releases.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_environment(
        &self,
        environment: &str,
    ) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let all_archives = self.list_all().await?;

        let filtered: Vec<ArchivedChangeset> = all_archives
            .into_iter()
            .filter(|archived| archived.changeset.environments.contains(&environment.to_string()))
            .collect();

        Ok(filtered)
    }

    /// Queries changesets by version bump type.
    ///
    /// Returns all archived changesets that used the specified version bump type.
    /// Results are sorted by applied date in descending order.
    ///
    /// # Arguments
    ///
    /// * `bump` - The version bump type to search for
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The history directory cannot be read
    /// - Any archived changeset file is corrupted or cannot be deserialized
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::ChangesetHistory;
    /// # use sublime_pkg_tools::changeset::FileBasedChangesetStorage;
    /// # use sublime_standard_tools::filesystem::FileSystemManager;
    /// # use sublime_pkg_tools::types::VersionBump;
    /// # use std::path::PathBuf;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let fs = FileSystemManager::new();
    /// # let storage = FileBasedChangesetStorage::new(
    /// #     workspace_root.clone(),
    /// #     PathBuf::from(".changesets"),
    /// #     PathBuf::from(".changesets/history"),
    /// #     fs
    /// # );
    /// # let history = ChangesetHistory::new(Box::new(storage));
    /// // Find all major releases
    /// let major_releases = history.query_by_bump(VersionBump::Major).await?;
    ///
    /// println!("Major releases: {}", major_releases.len());
    ///
    /// // Find all patch releases
    /// let patch_releases = history.query_by_bump(VersionBump::Patch).await?;
    ///
    /// println!("Patch releases: {}", patch_releases.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_by_bump(
        &self,
        bump: VersionBump,
    ) -> ChangesetResult<Vec<ArchivedChangeset>> {
        let all_archives = self.list_all().await?;

        let filtered: Vec<ArchivedChangeset> =
            all_archives.into_iter().filter(|archived| archived.changeset.bump == bump).collect();

        Ok(filtered)
    }
}
