//! # Changeset storage module
//!
//! ## What
//! Provides persistent storage for changesets using a file-based system.
//! Manages both pending changesets and archived history with full CRUD operations.
//!
//! ## How
//! - Stores changesets as JSON files in `.changesets/` directory
//! - Archives applied changesets to `.changesets/history/` with release metadata
//! - Uses FileSystemManager from sublime_standard_tools for all I/O
//! - Validates changesets against configured available environments
//! - Provides query operations for filtering and searching
//!
//! ## Why
//! File-based storage provides:
//! - Simple, Git-friendly changeset management
//! - Complete audit trail through history
//! - Easy manual inspection and modification
//! - No external database dependencies

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::AsyncFileSystem;

use crate::{
    changeset::Changeset,
    config::ChangesetConfig,
    error::{ChangesetError, ChangesetResult, PackageResult},
};

/// Trait for changeset storage operations.
///
/// Provides an abstraction over changeset persistence, enabling different
/// storage backends while maintaining a consistent API.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::storage::ChangesetStorage;
/// use sublime_pkg_tools::changeset::Changeset;
///
/// # async fn example<S: ChangesetStorage>(storage: &S) -> Result<(), Box<dyn std::error::Error>> {
/// // Save a changeset
/// let changeset = Changeset::new("feat/auth".to_string(), "dev@example.com".to_string());
/// let id = storage.save(&changeset).await?;
///
/// // Load it back
/// let loaded = storage.load(&id).await?;
/// assert_eq!(loaded.branch, "feat/auth");
///
/// // List all pending
/// let pending = storage.list_pending().await?;
/// println!("Pending changesets: {}", pending.len());
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ChangesetStorage: Send + Sync {
    /// Saves a changeset and returns its generated ID.
    ///
    /// # Arguments
    ///
    /// * `changeset` - The changeset to save
    ///
    /// # Returns
    ///
    /// The generated changeset ID (filename without extension)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset validation fails
    /// - File system write fails
    /// - Changeset already exists
    async fn save(&self, changeset: &Changeset) -> ChangesetResult<String>;

    /// Loads a changeset by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID (filename without extension)
    ///
    /// # Returns
    ///
    /// The loaded changeset
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset not found
    /// - Invalid JSON format
    /// - Validation fails
    async fn load(&self, id: &str) -> ChangesetResult<Changeset>;

    /// Checks if a changeset exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to check
    ///
    /// # Returns
    ///
    /// `true` if the changeset exists (pending or history)
    async fn exists(&self, id: &str) -> ChangesetResult<bool>;

    /// Deletes a pending changeset.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to delete
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset not found
    /// - File system delete fails
    async fn delete(&self, id: &str) -> ChangesetResult<()>;

    /// Lists all pending changeset IDs.
    ///
    /// # Returns
    ///
    /// Vector of changeset IDs sorted by creation time (newest first)
    async fn list_pending(&self) -> ChangesetResult<Vec<String>>;

    /// Lists all archived changeset IDs from history.
    ///
    /// # Returns
    ///
    /// Vector of changeset IDs sorted by creation time (newest first)
    async fn list_history(&self) -> ChangesetResult<Vec<String>>;

    /// Archives a changeset by moving it to history.
    ///
    /// The changeset must have `release_info` populated before archiving.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to archive
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Changeset not found
    /// - Changeset not applied (missing release_info)
    /// - File system operations fail
    async fn archive(&self, id: &str) -> ChangesetResult<()>;

    /// Loads a changeset from history.
    ///
    /// # Arguments
    ///
    /// * `id` - The changeset ID to load from history
    ///
    /// # Returns
    ///
    /// The loaded changeset from history
    ///
    /// # Errors
    ///
    /// Returns error if changeset not found in history
    async fn load_from_history(&self, id: &str) -> ChangesetResult<Changeset>;

    /// Queries pending changesets by branch name.
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name to filter by (exact match)
    ///
    /// # Returns
    ///
    /// Vector of matching changeset IDs
    async fn query_by_branch(&self, branch: &str) -> ChangesetResult<Vec<String>>;

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
    async fn query_history_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ChangesetResult<Vec<String>>;

    /// Queries history by package name.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Package name to search for
    ///
    /// # Returns
    ///
    /// Vector of changeset IDs that include the package
    async fn query_history_by_package(&self, package_name: &str) -> ChangesetResult<Vec<String>>;

    /// Gets the latest pending changeset for a specific branch.
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name to search for
    ///
    /// # Returns
    ///
    /// The changeset ID of the latest changeset, or `None` if not found
    async fn get_latest_for_branch(&self, branch: &str) -> ChangesetResult<Option<String>>;
}

/// File-based implementation of changeset storage.
///
/// Stores changesets as JSON files using the configured filesystem manager.
/// Pending changesets are stored in the configured changeset path, and
/// applied changesets are archived to the history path.
///
/// # File Organization
///
/// ```text
/// .changesets/
/// ├── feat-user-auth-20240115T103000Z.json    (pending)
/// ├── fix-memory-leak-20240115T144530Z.json   (pending)
/// └── history/
///     ├── feat-oauth2-20240114T091500Z.json   (applied)
///     └── bugfix-security-20240113T120000Z.json (applied)
/// ```
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::storage::{ChangesetStorage, FileBasedChangesetStorage};
/// use sublime_pkg_tools::config::ChangesetConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ChangesetConfig::default();
/// let fs = FileSystemManager::new();
/// let root = PathBuf::from("/path/to/project");
///
/// let storage = FileBasedChangesetStorage::new(fs, root, config);
///
/// // List all pending changesets
/// let pending = storage.list_pending().await?;
/// println!("Found {} pending changesets", pending.len());
/// # Ok(())
/// # }
/// ```
pub struct FileBasedChangesetStorage<F>
where
    F: AsyncFileSystem,
{
    filesystem: F,
    root_path: PathBuf,
    config: ChangesetConfig,
}

impl<F> FileBasedChangesetStorage<F>
where
    F: AsyncFileSystem,
{
    /// Creates a new file-based changeset storage.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem manager to use for I/O operations
    /// * `root_path` - Root path of the project
    /// * `config` - Changeset configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::storage::FileBasedChangesetStorage;
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let config = ChangesetConfig::default();
    /// let fs = FileSystemManager::new();
    /// let root = PathBuf::from("/path/to/project");
    ///
    /// let storage = FileBasedChangesetStorage::new(fs, root, config);
    /// ```
    #[must_use]
    pub fn new(filesystem: F, root_path: PathBuf, config: ChangesetConfig) -> Self {
        Self { filesystem, root_path, config }
    }

    /// Gets the full path to the changeset directory.
    fn changeset_dir(&self) -> PathBuf {
        self.root_path.join(&self.config.path)
    }

    /// Gets the full path to the history directory.
    fn history_dir(&self) -> PathBuf {
        self.root_path.join(&self.config.history_path)
    }

    /// Gets the full path to a pending changeset file.
    fn changeset_path(&self, id: &str) -> PathBuf {
        self.changeset_dir().join(format!("{}.json", id))
    }

    /// Gets the full path to a history changeset file.
    fn history_path(&self, id: &str) -> PathBuf {
        self.history_dir().join(format!("{}.json", id))
    }

    /// Validates changeset against configuration.
    fn validate_changeset(&self, changeset: &Changeset) -> ChangesetResult<()> {
        // Validate using changeset's own validation
        changeset.validate(Some(&self.config.available_environments))?;

        // Additional storage-specific validation
        if changeset.branch.is_empty() {
            return Err(ChangesetError::ValidationFailed {
                changeset_id: "unknown".to_string(),
                errors: vec!["Branch name cannot be empty".to_string()],
            });
        }

        if changeset.author.is_empty() {
            return Err(ChangesetError::ValidationFailed {
                changeset_id: "unknown".to_string(),
                errors: vec!["Author cannot be empty".to_string()],
            });
        }

        Ok(())
    }

    /// Reads and parses a changeset file.
    async fn read_changeset_file(&self, path: &Path) -> ChangesetResult<Changeset> {
        // Check if file exists
        if !self.filesystem.exists(path).await {
            return Err(ChangesetError::NotFound { path: path.to_path_buf() });
        }

        // Read file content
        let content = self.filesystem.read_file_string(path).await.map_err(|e| {
            ChangesetError::InvalidFormat {
                path: path.to_path_buf(),
                reason: format!("Failed to read file: {}", e),
            }
        })?;

        // Parse JSON
        let changeset: Changeset =
            serde_json::from_str(&content).map_err(|e| ChangesetError::InvalidFormat {
                path: path.to_path_buf(),
                reason: format!("Invalid JSON: {}", e),
            })?;

        // Validate
        self.validate_changeset(&changeset)?;

        Ok(changeset)
    }

    /// Writes a changeset to a file.
    async fn write_changeset_file(
        &self,
        path: &Path,
        changeset: &Changeset,
    ) -> ChangesetResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.filesystem.create_dir_all(parent).await.map_err(|e| {
                ChangesetError::CreationFailed {
                    branch: changeset.branch.clone(),
                    reason: format!("Failed to create directory: {}", e),
                }
            })?;
        }

        // Serialize to pretty JSON
        let content = serde_json::to_string_pretty(changeset).map_err(|e| {
            ChangesetError::CreationFailed {
                branch: changeset.branch.clone(),
                reason: format!("Failed to serialize: {}", e),
            }
        })?;

        // Write to file
        self.filesystem.write_file_string(path, &content).await.map_err(|e| {
            ChangesetError::CreationFailed {
                branch: changeset.branch.clone(),
                reason: format!("Failed to write file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Lists changeset files in a directory.
    async fn list_changesets_in_dir(&self, dir: &Path) -> ChangesetResult<Vec<String>> {
        // Check if directory exists
        if !self.filesystem.exists(dir).await {
            return Ok(Vec::new());
        }

        // Read directory entries
        let entries = self.filesystem.read_dir(dir).await.map_err(|e| {
            ChangesetError::HistoryOperationFailed {
                operation: "list".to_string(),
                reason: format!("Failed to read directory: {}", e),
            }
        })?;

        // Collect and sort by filename (which includes timestamp)
        let mut ids: Vec<String> = entries
            .into_iter()
            .filter_map(|path| {
                if path.extension()?.to_str()? == "json" {
                    path.file_stem()?.to_str().map(String::from)
                } else {
                    None
                }
            })
            .collect();

        // Sort by filename (newest first - lexicographic sort works with ISO datetime)
        ids.sort_by(|a, b| b.cmp(a));

        Ok(ids)
    }

    /// Queries changesets by applying a filter function.
    async fn query_changesets<P>(&self, dir: &Path, predicate: P) -> ChangesetResult<Vec<String>>
    where
        P: Fn(&Changeset) -> bool,
    {
        let ids = self.list_changesets_in_dir(dir).await?;
        let mut results = Vec::new();

        for id in ids {
            let path = dir.join(format!("{}.json", id));
            if let Ok(changeset) = self.read_changeset_file(&path).await {
                if predicate(&changeset) {
                    results.push(id);
                }
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl<F> ChangesetStorage for FileBasedChangesetStorage<F>
where
    F: AsyncFileSystem + Send + Sync,
{
    async fn save(&self, changeset: &Changeset) -> ChangesetResult<String> {
        // Validate changeset
        self.validate_changeset(changeset)?;

        // Generate ID
        let id = changeset.generate_id();

        // Check if already exists (in pending or history)
        let pending_path = self.changeset_path(&id);
        let history_path = self.history_path(&id);

        let pending_exists = self.filesystem.exists(&pending_path).await;

        let history_exists = self.filesystem.exists(&history_path).await;

        if pending_exists || history_exists {
            return Err(ChangesetError::AlreadyExists { changeset_id: id });
        }

        // Write to file
        self.write_changeset_file(&pending_path, changeset).await?;

        Ok(id)
    }

    async fn load(&self, id: &str) -> ChangesetResult<Changeset> {
        let path = self.changeset_path(id);
        self.read_changeset_file(&path).await
    }

    async fn exists(&self, id: &str) -> ChangesetResult<bool> {
        let pending_path = self.changeset_path(id);
        let history_path = self.history_path(id);

        let pending_exists = self.filesystem.exists(&pending_path).await;

        if pending_exists {
            return Ok(true);
        }

        let history_exists = self.filesystem.exists(&history_path).await;

        Ok(history_exists)
    }

    async fn delete(&self, id: &str) -> ChangesetResult<()> {
        let path = self.changeset_path(id);

        if !self.filesystem.exists(&path).await {
            return Err(ChangesetError::NotFound { path });
        }

        self.filesystem.remove(&path).await.map_err(|e| {
            ChangesetError::HistoryOperationFailed {
                operation: "delete".to_string(),
                reason: format!("Failed to delete file: {}", e),
            }
        })?;

        Ok(())
    }

    async fn list_pending(&self) -> ChangesetResult<Vec<String>> {
        self.list_changesets_in_dir(&self.changeset_dir()).await
    }

    async fn list_history(&self) -> ChangesetResult<Vec<String>> {
        self.list_changesets_in_dir(&self.history_dir()).await
    }

    async fn archive(&self, id: &str) -> ChangesetResult<()> {
        // Load changeset
        let changeset = self.load(id).await?;

        // Verify it has release info
        if changeset.release_info.is_none() {
            return Err(ChangesetError::ApplicationFailed {
                changeset_id: id.to_string(),
                reason: "Cannot archive changeset without release info".to_string(),
            });
        }

        // Ensure history directory exists
        let history_dir = self.history_dir();
        self.filesystem.create_dir_all(&history_dir).await.map_err(|e| {
            ChangesetError::HistoryOperationFailed {
                operation: "archive".to_string(),
                reason: format!("Failed to create history directory: {}", e),
            }
        })?;

        // Write to history
        let history_path = self.history_path(id);
        self.write_changeset_file(&history_path, &changeset).await?;

        // Delete from pending
        let pending_path = self.changeset_path(id);
        self.filesystem.remove(&pending_path).await.map_err(|e| {
            ChangesetError::HistoryOperationFailed {
                operation: "archive".to_string(),
                reason: format!("Failed to remove pending file: {}", e),
            }
        })?;

        Ok(())
    }

    async fn load_from_history(&self, id: &str) -> ChangesetResult<Changeset> {
        let path = self.history_path(id);
        self.read_changeset_file(&path).await
    }

    async fn query_by_branch(&self, branch: &str) -> ChangesetResult<Vec<String>> {
        self.query_changesets(&self.changeset_dir(), |cs| cs.branch == branch).await
    }

    async fn query_history_by_date(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ChangesetResult<Vec<String>> {
        self.query_changesets(&self.history_dir(), |cs| {
            cs.created_at >= start && cs.created_at <= end
        })
        .await
    }

    async fn query_history_by_package(&self, package_name: &str) -> ChangesetResult<Vec<String>> {
        self.query_changesets(&self.history_dir(), |cs| {
            cs.packages.iter().any(|pkg| pkg.name == package_name)
        })
        .await
    }

    async fn get_latest_for_branch(&self, branch: &str) -> ChangesetResult<Option<String>> {
        let matches = self.query_by_branch(branch).await?;
        Ok(matches.into_iter().next())
    }
}

impl<F> FileBasedChangesetStorage<F>
where
    F: AsyncFileSystem + Send + Sync,
{
    /// Creates storage from package result wrapper for error conversion.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - The filesystem manager to use
    /// * `root_path` - Root path of the project
    /// * `config` - Changeset configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::storage::FileBasedChangesetStorage;
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ChangesetConfig::default();
    /// let fs = FileSystemManager::new();
    /// let root = PathBuf::from("/path/to/project");
    ///
    /// let storage = FileBasedChangesetStorage::from_package_result(fs, root, config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_package_result(
        filesystem: F,
        root_path: PathBuf,
        config: ChangesetConfig,
    ) -> PackageResult<Self> {
        Ok(Self::new(filesystem, root_path, config))
    }
}
