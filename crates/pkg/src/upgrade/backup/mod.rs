//! Backup and rollback functionality for upgrade operations.
//!
//! **What**: Provides automatic backup creation before applying upgrades and rollback
//! functionality to restore files in case of failures.
//!
//! **How**: Creates timestamped backup directories containing copies of package.json files,
//! maintains metadata about backups, and provides functions to restore from backups or
//! clean up old backups based on configuration limits.
//!
//! **Why**: To enable safe dependency upgrades with the ability to recover from failures
//! by restoring the previous state of package.json files.
//!
//! # Backup Structure
//!
//! Backups are stored in a configurable directory (default: `.pkg-backups/`) with the
//! following structure:
//!
//! ```text
//! .pkg-backups/
//! ├── 2024-01-15T10-30-45-upgrade/
//! │   ├── package.json
//! │   └── packages/
//! │       ├── core/package.json
//! │       └── utils/package.json
//! ├── 2024-01-14T15-20-30-upgrade/
//! │   └── ...
//! └── metadata.json
//! ```
//!
//! # Metadata Format
//!
//! The `metadata.json` file tracks all backups:
//!
//! ```json
//! {
//!   "backups": [
//!     {
//!       "id": "2024-01-15T10-30-45-upgrade",
//!       "created_at": "2024-01-15T10:30:45Z",
//!       "operation": "upgrade",
//!       "files": [
//!         "/workspace/package.json",
//!         "/workspace/packages/core/package.json"
//!       ],
//!       "success": true
//!     }
//!   ]
//! }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::upgrade::backup::BackupManager;
//! use sublime_pkg_tools::config::BackupConfig;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let config = BackupConfig::default();
//!
//! let manager = BackupManager::new(workspace_root, config, fs);
//!
//! // Create a backup before applying upgrades
//! let files = vec![
//!     PathBuf::from("package.json"),
//!     PathBuf::from("packages/core/package.json"),
//! ];
//! let backup_id = manager.create_backup(&files, "upgrade").await?;
//! println!("Created backup: {}", backup_id);
//!
//! // If something goes wrong, restore from backup
//! manager.restore_backup(&backup_id).await?;
//! println!("Restored from backup: {}", backup_id);
//!
//! // On success, optionally clean up the backup
//! if !config.keep_after_success {
//!     manager.delete_backup(&backup_id).await?;
//! }
//! # Ok(())
//! # }
//! ```

use crate::config::BackupConfig;
use crate::error::{UpgradeError, UpgradeResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Manager for backup and rollback operations.
///
/// Handles creation of backups before applying upgrades, restoration of files
/// from backups, and management of backup lifecycle including cleanup.
///
/// # Example
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::backup::BackupManager;
/// use sublime_pkg_tools::config::BackupConfig;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = BackupManager::new(
///     PathBuf::from("."),
///     BackupConfig::default(),
///     FileSystemManager::new(),
/// );
///
/// let files = vec![PathBuf::from("package.json")];
/// let backup_id = manager.create_backup(&files, "upgrade").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BackupManager<F: AsyncFileSystem> {
    workspace_root: PathBuf,
    config: BackupConfig,
    fs: F,
}

/// Metadata about a single backup.
///
/// Contains information about when the backup was created, what operation
/// triggered it, which files were backed up, and whether the operation succeeded.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::upgrade::backup::BackupMetadata;
/// use chrono::Utc;
/// use std::path::PathBuf;
///
/// let metadata = BackupMetadata {
///     id: "2024-01-15T10-30-45-upgrade".to_string(),
///     created_at: Utc::now(),
///     operation: "upgrade".to_string(),
///     files: vec![PathBuf::from("package.json")],
///     success: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupMetadata {
    /// Unique identifier for the backup.
    ///
    /// Format: `{timestamp}-{operation}` (e.g., "2024-01-15T10-30-45-upgrade")
    pub id: String,

    /// Timestamp when the backup was created.
    pub created_at: DateTime<Utc>,

    /// Type of operation that triggered the backup.
    ///
    /// Typically "upgrade" for dependency upgrades.
    pub operation: String,

    /// List of absolute file paths that were backed up.
    pub files: Vec<PathBuf>,

    /// Whether the operation completed successfully.
    ///
    /// - `true`: Operation succeeded, backup can be cleaned up if configured
    /// - `false`: Operation failed, backup should be kept for rollback
    pub success: bool,
}

/// Collection of all backup metadata.
///
/// Stored in `metadata.json` in the backup directory root.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
struct BackupMetadataCollection {
    /// List of all backups, ordered from newest to oldest.
    backups: Vec<BackupMetadata>,
}

impl<F: AsyncFileSystem> BackupManager<F> {
    /// Creates a new BackupManager.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Backup configuration
    /// * `fs` - Filesystem implementation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// use sublime_pkg_tools::config::BackupConfig;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = BackupManager::new(
    ///     PathBuf::from("."),
    ///     BackupConfig::default(),
    ///     FileSystemManager::new(),
    /// );
    /// ```
    #[must_use]
    pub fn new(workspace_root: PathBuf, config: BackupConfig, fs: F) -> Self {
        Self { workspace_root, config, fs }
    }

    /// Creates a backup of the specified files.
    ///
    /// Copies all specified files to a timestamped backup directory and updates
    /// the metadata. The backup directory is created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `files` - List of file paths to backup (absolute or relative to workspace_root)
    /// * `operation` - Type of operation triggering the backup (e.g., "upgrade")
    ///
    /// # Returns
    ///
    /// The unique backup ID that can be used to restore or delete the backup.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Backups are disabled in configuration
    /// - Backup directory creation fails
    /// - File copying fails
    /// - Metadata update fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # use std::path::PathBuf;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// let files = vec![
    ///     PathBuf::from("package.json"),
    ///     PathBuf::from("packages/core/package.json"),
    /// ];
    /// let backup_id = manager.create_backup(&files, "upgrade").await?;
    /// println!("Created backup: {}", backup_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_backup(&self, files: &[PathBuf], operation: &str) -> UpgradeResult<String> {
        if !self.config.enabled {
            return Err(UpgradeError::BackupFailed {
                path: self.backup_dir(),
                reason: "Backups are disabled in configuration".to_string(),
            });
        }

        // Generate backup ID
        let backup_id = self.generate_backup_id(operation);
        let backup_path = self.backup_path(&backup_id);

        // Create backup directory
        self.fs.create_dir_all(&backup_path).await.map_err(|e| UpgradeError::BackupFailed {
            path: backup_path.clone(),
            reason: format!("Failed to create backup directory: {}", e),
        })?;

        // Copy each file to backup directory, preserving directory structure
        let mut backed_up_files = Vec::new();
        for file in files {
            let absolute_path = self.resolve_path(file);

            // Check if file exists before backing up
            let exists = self.fs.exists(&absolute_path).await;

            if !exists {
                return Err(UpgradeError::FileSystemError {
                    path: absolute_path,
                    reason: "File does not exist".to_string(),
                });
            }

            // Calculate relative path from workspace root
            let relative = if file.is_absolute() {
                file.strip_prefix(&self.workspace_root)
                    .map_err(|_| UpgradeError::FileSystemError {
                        path: file.clone(),
                        reason: "File is not within workspace".to_string(),
                    })?
                    .to_path_buf()
            } else {
                file.clone()
            };

            // Create target path in backup directory
            let target = backup_path.join(&relative);

            // Ensure parent directory exists
            if let Some(parent) = target.parent() {
                self.fs.create_dir_all(parent).await.map_err(|e| UpgradeError::BackupFailed {
                    path: parent.to_path_buf(),
                    reason: format!("Failed to create parent directory: {}", e),
                })?;
            }

            // Copy file
            let content = self.fs.read_file(&absolute_path).await.map_err(|e| {
                UpgradeError::BackupFailed {
                    path: absolute_path.clone(),
                    reason: format!("Failed to read file: {}", e),
                }
            })?;

            self.fs.write_file(&target, &content).await.map_err(|e| {
                UpgradeError::BackupFailed {
                    path: target.clone(),
                    reason: format!("Failed to write backup file: {}", e),
                }
            })?;

            backed_up_files.push(absolute_path);
        }

        // Create metadata entry
        let metadata = BackupMetadata {
            id: backup_id.clone(),
            created_at: Utc::now(),
            operation: operation.to_string(),
            files: backed_up_files,
            success: false, // Will be updated by mark_success
        };

        // Update metadata file
        self.add_backup_metadata(metadata).await?;

        // Clean up old backups if needed
        self.cleanup_old_backups().await?;

        Ok(backup_id)
    }

    /// Restores files from a backup.
    ///
    /// Copies all files from the specified backup back to their original locations,
    /// overwriting any existing files.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - ID of the backup to restore
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Backup doesn't exist
    /// - Metadata cannot be read
    /// - File restoration fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Restore from a specific backup
    /// manager.restore_backup("2024-01-15T10-30-45-upgrade").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restore_backup(&self, backup_id: &str) -> UpgradeResult<()> {
        let backup_path = self.backup_path(backup_id);

        // Check if backup exists
        let exists = self.fs.exists(&backup_path).await;

        if !exists {
            return Err(UpgradeError::NoBackup { path: backup_path });
        }

        // Load metadata to get file list
        let metadata = self.get_backup_metadata(backup_id).await?;

        // Restore each file
        for file_path in &metadata.files {
            let relative = file_path.strip_prefix(&self.workspace_root).map_err(|_| {
                UpgradeError::RollbackFailed {
                    reason: format!("File path not within workspace: {}", file_path.display()),
                }
            })?;

            let backup_file = backup_path.join(relative);
            let target_file = file_path;

            // Ensure parent directory exists
            if let Some(parent) = target_file.parent() {
                self.fs.create_dir_all(parent).await.map_err(|e| UpgradeError::RollbackFailed {
                    reason: format!("Failed to create parent directory: {}", e),
                })?;
            }

            // Copy file back
            let content = self.fs.read_file(&backup_file).await.map_err(|e| {
                UpgradeError::RollbackFailed {
                    reason: format!("Failed to read backup file {}: {}", backup_file.display(), e),
                }
            })?;

            self.fs.write_file(target_file, &content).await.map_err(|e| {
                UpgradeError::RollbackFailed {
                    reason: format!("Failed to restore file {}: {}", target_file.display(), e),
                }
            })?;
        }

        Ok(())
    }

    /// Restores files from the most recent backup.
    ///
    /// Convenience method to restore from the newest backup without needing
    /// to know its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No backups exist
    /// - Restoration fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Restore from the most recent backup
    /// manager.restore_last_backup().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restore_last_backup(&self) -> UpgradeResult<()> {
        let backups = self.list_backups().await?;

        if backups.is_empty() {
            return Err(UpgradeError::NoBackup { path: self.backup_dir() });
        }

        let last_backup = &backups[0]; // Already sorted newest first
        self.restore_backup(&last_backup.id).await
    }

    /// Lists all available backups.
    ///
    /// Returns backups sorted from newest to oldest.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata cannot be read.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// let backups = manager.list_backups().await?;
    /// for backup in backups {
    ///     println!("Backup: {} ({} files)", backup.id, backup.files.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_backups(&self) -> UpgradeResult<Vec<BackupMetadata>> {
        let collection = self.load_metadata_collection().await?;
        Ok(collection.backups)
    }

    /// Deletes a specific backup.
    ///
    /// Removes the backup directory and updates metadata.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - ID of the backup to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Backup doesn't exist
    /// - Deletion fails
    /// - Metadata update fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// manager.delete_backup("2024-01-15T10-30-45-upgrade").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_backup(&self, backup_id: &str) -> UpgradeResult<()> {
        let backup_path = self.backup_path(backup_id);

        // Check if backup exists
        let exists = self.fs.exists(&backup_path).await;

        if !exists {
            return Err(UpgradeError::NoBackup { path: backup_path });
        }

        // Remove backup directory
        self.fs.remove(&backup_path).await.map_err(|e| UpgradeError::FileSystemError {
            path: backup_path.clone(),
            reason: format!("Failed to delete backup: {}", e),
        })?;

        // Update metadata
        self.remove_backup_metadata(backup_id).await?;

        Ok(())
    }

    /// Marks a backup as successful.
    ///
    /// Updates the backup's metadata to indicate that the operation completed
    /// successfully. This is used to determine which backups can be cleaned up.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - ID of the backup to mark as successful
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Backup doesn't exist in metadata
    /// - Metadata update fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// let backup_id = manager.create_backup(&[], "upgrade").await?;
    /// // ... perform upgrade operation ...
    /// manager.mark_success(&backup_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mark_success(&self, backup_id: &str) -> UpgradeResult<()> {
        let mut collection = self.load_metadata_collection().await?;

        let backup = collection
            .backups
            .iter_mut()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| UpgradeError::NoBackup { path: self.backup_path(backup_id) })?;

        backup.success = true;

        self.save_metadata_collection(&collection).await?;

        Ok(())
    }

    /// Cleans up old backups based on configuration limits.
    ///
    /// Removes the oldest backups when the total number exceeds `max_backups`.
    /// Optionally removes successful backups based on `keep_after_success` setting.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion or metadata update fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::upgrade::backup::BackupManager;
    /// # async fn example(manager: BackupManager<impl sublime_standard_tools::filesystem::AsyncFileSystem>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Clean up old backups
    /// manager.cleanup_old_backups().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cleanup_old_backups(&self) -> UpgradeResult<()> {
        let mut collection = self.load_metadata_collection().await?;
        let mut to_remove_all: Vec<String> = Vec::new();

        // Collect successful backups to remove if configured
        if !self.config.keep_after_success {
            let to_remove: Vec<String> =
                collection.backups.iter().filter(|b| b.success).map(|b| b.id.clone()).collect();
            to_remove_all.extend(to_remove);
        }

        // Determine how many more to remove based on max_backups limit
        // Calculate this based on the collection BEFORE removing successful backups
        let remaining_after_success = collection.backups.len() - to_remove_all.len();
        if remaining_after_success > self.config.max_backups {
            let to_remove_count = remaining_after_success - self.config.max_backups;
            // Get oldest backups that aren't already marked for removal
            let to_remove: Vec<String> = collection
                .backups
                .iter()
                .rev() // Oldest first (collection is sorted newest first)
                .filter(|b| !to_remove_all.contains(&b.id))
                .take(to_remove_count)
                .map(|b| b.id.clone())
                .collect();
            to_remove_all.extend(to_remove);
        }

        // Remove all backups from filesystem
        for backup_id in &to_remove_all {
            let backup_path = self.backup_path(backup_id);
            if self.fs.exists(&backup_path).await {
                let _ = self.fs.remove(&backup_path).await;
            }
        }

        // Remove from collection in a single operation
        collection.backups.retain(|b| !to_remove_all.contains(&b.id));

        self.save_metadata_collection(&collection).await?;

        Ok(())
    }

    /// Returns the backup directory path.
    fn backup_dir(&self) -> PathBuf {
        self.workspace_root.join(&self.config.backup_dir)
    }

    /// Returns the path for a specific backup.
    fn backup_path(&self, backup_id: &str) -> PathBuf {
        self.backup_dir().join(backup_id)
    }

    /// Returns the path to the metadata file.
    fn metadata_path(&self) -> PathBuf {
        self.backup_dir().join("metadata.json")
    }

    /// Generates a unique backup ID.
    fn generate_backup_id(&self, operation: &str) -> String {
        let now = Utc::now();
        format!("{}-{}", now.format("%Y-%m-%dT%H-%M-%S"), operation)
    }

    /// Resolves a path to an absolute path.
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        }
    }

    /// Loads the metadata collection from disk.
    async fn load_metadata_collection(&self) -> UpgradeResult<BackupMetadataCollection> {
        let metadata_path = self.metadata_path();

        let exists = self.fs.exists(&metadata_path).await;

        if !exists {
            return Ok(BackupMetadataCollection::default());
        }

        let content = self.fs.read_file_string(&metadata_path).await.map_err(|e| {
            UpgradeError::FileSystemError {
                path: metadata_path.clone(),
                reason: format!("Failed to read metadata: {}", e),
            }
        })?;

        serde_json::from_str(&content).map_err(|e| UpgradeError::BackupCorrupted {
            path: metadata_path,
            reason: format!("Failed to parse metadata: {}", e),
        })
    }

    /// Saves the metadata collection to disk.
    async fn save_metadata_collection(
        &self,
        collection: &BackupMetadataCollection,
    ) -> UpgradeResult<()> {
        let metadata_path = self.metadata_path();

        // Ensure backup directory exists
        let backup_dir = self.backup_dir();
        self.fs.create_dir_all(&backup_dir).await.map_err(|e| UpgradeError::BackupFailed {
            path: backup_dir,
            reason: format!("Failed to create backup directory: {}", e),
        })?;

        let content =
            serde_json::to_string_pretty(collection).map_err(|e| UpgradeError::BackupFailed {
                path: metadata_path.clone(),
                reason: format!("Failed to serialize metadata: {}", e),
            })?;

        self.fs.write_file_string(&metadata_path, &content).await.map_err(|e| {
            UpgradeError::BackupFailed {
                path: metadata_path,
                reason: format!("Failed to write metadata: {}", e),
            }
        })?;

        Ok(())
    }

    /// Adds a backup to the metadata collection.
    async fn add_backup_metadata(&self, metadata: BackupMetadata) -> UpgradeResult<()> {
        let mut collection = self.load_metadata_collection().await?;

        // Insert at the beginning to keep newest first
        collection.backups.insert(0, metadata);

        self.save_metadata_collection(&collection).await
    }

    /// Removes a backup from the metadata collection.
    async fn remove_backup_metadata(&self, backup_id: &str) -> UpgradeResult<()> {
        let mut collection = self.load_metadata_collection().await?;
        collection.backups.retain(|b| b.id != backup_id);
        self.save_metadata_collection(&collection).await
    }

    /// Gets metadata for a specific backup.
    async fn get_backup_metadata(&self, backup_id: &str) -> UpgradeResult<BackupMetadata> {
        let collection = self.load_metadata_collection().await?;

        collection
            .backups
            .into_iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| UpgradeError::NoBackup { path: self.backup_path(backup_id) })
    }
}

#[cfg(test)]
mod tests;
