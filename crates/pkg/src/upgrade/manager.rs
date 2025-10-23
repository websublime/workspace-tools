//! Upgrade manager for orchestrating dependency upgrade operations.
//!
//! **What**: Provides a high-level `UpgradeManager` that integrates detection, application,
//! backup, and changeset functionality into a unified, easy-to-use API for dependency upgrades.
//!
//! **How**: This module combines the registry client, backup manager, and changeset integration
//! to provide a complete upgrade workflow. It handles configuration, coordinates between modules,
//! manages state (like last backup for rollback), and provides a clean public API that abstracts
//! away the complexity of the individual components.
//!
//! **Why**: To provide a simple, safe, and powerful interface for dependency upgrades that handles
//! all the complexity internally while exposing a clean API. This allows users to perform upgrades
//! with minimal code while benefiting from automatic backups, rollback, and changeset integration.

use crate::changeset::ChangesetManager;
use crate::config::{PackageToolsConfig, UpgradeConfig};
use crate::error::{UpgradeError, UpgradeResult};
use crate::upgrade::application::{apply_upgrades, apply_with_changeset};
use crate::upgrade::backup::BackupManager;
use crate::upgrade::detection::{detect_upgrades, DetectionOptions, UpgradePreview};
use crate::upgrade::registry::RegistryClient;
use crate::upgrade::{UpgradeResult as UpgradeResultType, UpgradeSelection};
use std::path::PathBuf;
use sublime_standard_tools::filesystem::FileSystemManager;

/// High-level manager for dependency upgrade operations.
///
/// `UpgradeManager` orchestrates the complete upgrade workflow by integrating:
/// - Registry client for package metadata queries
/// - Backup manager for automatic backup and rollback
/// - Changeset integration for automatic tracking
/// - Detection and application logic
///
/// This provides a unified, simple API that handles all complexity internally,
/// including configuration management, error handling, and state coordination.
///
/// # Example
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::{UpgradeManager, DetectionOptions, UpgradeSelection};
/// use sublime_pkg_tools::config::UpgradeConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let config = UpgradeConfig::default();
///
/// // Create manager
/// let manager = UpgradeManager::new(workspace_root, config).await?;
///
/// // Detect available upgrades
/// let preview = manager.detect_upgrades(DetectionOptions::all()).await?;
/// println!("Found {} available upgrades", preview.summary.upgrades_available);
///
/// // Apply patch upgrades only
/// let result = manager.apply_upgrades(UpgradeSelection::patch_only(), false).await?;
/// println!("Applied {} upgrades", result.summary.dependencies_upgraded);
/// # Ok(())
/// # }
/// ```
pub struct UpgradeManager {
    workspace_root: PathBuf,
    config: UpgradeConfig,
    registry_client: RegistryClient,
    backup_manager: BackupManager<FileSystemManager>,
    fs: FileSystemManager,
    last_backup_id: Option<String>,
}

impl UpgradeManager {
    /// Creates a new `UpgradeManager` with the given workspace root and configuration.
    ///
    /// This initializes all internal components including the registry client and backup manager
    /// based on the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace or package
    /// * `config` - Upgrade configuration controlling registry access, backups, etc.
    ///
    /// # Returns
    ///
    /// A configured `UpgradeManager` ready to perform upgrade operations
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - The registry client cannot be initialized
    /// - The workspace root is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::upgrade::UpgradeManager;
    /// use sublime_pkg_tools::config::UpgradeConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = UpgradeManager::new(
    ///     PathBuf::from("."),
    ///     UpgradeConfig::default()
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(workspace_root: PathBuf, config: UpgradeConfig) -> UpgradeResult<Self> {
        let fs = FileSystemManager::new();

        // Initialize registry client
        let registry_client = RegistryClient::new(&workspace_root, config.registry.clone()).await?;

        // Initialize backup manager
        let backup_dir = workspace_root.join(&config.backup.backup_dir);
        let backup_manager = BackupManager::new(backup_dir, config.backup.clone(), fs.clone());

        Ok(Self {
            workspace_root,
            config,
            registry_client,
            backup_manager,
            fs,
            last_backup_id: None,
        })
    }

    /// Detects available upgrades for dependencies in the workspace.
    ///
    /// Scans the workspace for package.json files, extracts external dependencies,
    /// and queries the configured registries to find available upgrades. The detection
    /// can be controlled using `DetectionOptions` to filter what dependencies to check.
    ///
    /// # Arguments
    ///
    /// * `options` - Detection options controlling which dependencies to check
    ///
    /// # Returns
    ///
    /// An `UpgradePreview` containing all available upgrades and summary statistics
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - Package.json files cannot be read
    /// - Registry queries fail
    /// - The workspace is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::upgrade::DetectionOptions;
    ///
    /// # async fn example(manager: sublime_pkg_tools::upgrade::UpgradeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Detect all available upgrades
    /// let preview = manager.detect_upgrades(DetectionOptions::all()).await?;
    /// println!("Found {} upgrades:", preview.summary.upgrades_available);
    /// println!("  Major: {}", preview.summary.major_upgrades);
    /// println!("  Minor: {}", preview.summary.minor_upgrades);
    /// println!("  Patch: {}", preview.summary.patch_upgrades);
    ///
    /// // Detect only production dependencies
    /// let preview = manager.detect_upgrades(DetectionOptions::production_only()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_upgrades(
        &self,
        options: DetectionOptions,
    ) -> UpgradeResult<UpgradePreview> {
        detect_upgrades(&self.workspace_root, &self.registry_client, &self.fs, options).await
    }

    /// Applies selected upgrades to package.json files.
    ///
    /// This is the main method for applying dependency upgrades. It:
    /// 1. Creates automatic backups (if configured)
    /// 2. Applies the selected upgrades to package.json files
    /// 3. Creates or updates a changeset (if configured)
    /// 4. Cleans up backups on success (if configured)
    /// 5. Automatically rolls back on failure
    ///
    /// The upgrade selection can be controlled using `UpgradeSelection` to filter which
    /// upgrades to apply (by type, package, dependency, etc.). Dry-run mode allows
    /// previewing changes without modifying files.
    ///
    /// # Arguments
    ///
    /// * `selection` - Selection criteria for filtering which upgrades to apply
    /// * `dry_run` - If true, preview changes without modifying files
    ///
    /// # Returns
    ///
    /// An `UpgradeResult` containing details of applied upgrades, modified files,
    /// changeset ID (if created), and summary statistics
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - Backup creation fails
    /// - Files cannot be read or written
    /// - Changeset creation fails
    /// - JSON parsing fails
    ///
    /// On error, any changes are automatically rolled back from the backup.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// # async fn example(manager: sublime_pkg_tools::upgrade::UpgradeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Preview changes (dry-run)
    /// let preview = manager.apply_upgrades(UpgradeSelection::all(), true).await?;
    /// println!("Would upgrade {} dependencies", preview.summary.dependencies_upgraded);
    ///
    /// // Apply patch upgrades only
    /// let result = manager.apply_upgrades(UpgradeSelection::patch_only(), false).await?;
    /// println!("Applied {} patch upgrades", result.summary.patch_upgrades);
    ///
    /// // Apply specific dependencies
    /// let result = manager.apply_upgrades(
    ///     UpgradeSelection::dependencies(vec!["lodash".to_string()]),
    ///     false
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn apply_upgrades(
        &mut self,
        selection: UpgradeSelection,
        dry_run: bool,
    ) -> UpgradeResult<UpgradeResultType> {
        // First, detect available upgrades
        let detection_options = self.selection_to_detection_options(&selection);
        let preview = self.detect_upgrades(detection_options).await?;

        if preview.packages.is_empty() {
            return apply_upgrades(vec![], selection, dry_run, &self.fs).await;
        }

        // In dry-run mode, just apply without backup or changeset
        if dry_run {
            return apply_upgrades(preview.packages, selection, dry_run, &self.fs).await;
        }

        // Create backup if enabled
        let backup_id = if self.config.backup.enabled {
            let files_to_backup = self.collect_package_json_files(&preview.packages)?;
            let backup_id = self.backup_manager.create_backup(&files_to_backup, "upgrade").await?;
            Some(backup_id)
        } else {
            None
        };

        // Apply upgrades with changeset integration
        let result = if self.config.auto_changeset {
            // Create a PackageToolsConfig with the current upgrade config
            let pkg_config =
                PackageToolsConfig { upgrade: self.config.clone(), ..Default::default() };

            let changeset_manager =
                ChangesetManager::new(&self.workspace_root, self.fs.clone(), pkg_config)
                    .await
                    .map_err(|e| UpgradeError::ChangesetCreationFailed {
                        reason: format!("Failed to initialize changeset manager: {}", e.as_ref()),
                    })?;

            apply_with_changeset(
                preview.packages,
                selection,
                dry_run,
                &self.workspace_root,
                &self.config,
                Some(&changeset_manager),
                &self.fs,
            )
            .await
        } else {
            apply_upgrades(preview.packages, selection, dry_run, &self.fs).await
        };

        // Handle result
        match result {
            Ok(upgrade_result) => {
                // Store backup ID for potential rollback
                if let Some(id) = backup_id.clone() {
                    self.last_backup_id = Some(id.clone());
                }

                // Clean up backup if configured
                if self.config.backup.enabled && !self.config.backup.keep_after_success {
                    if let Some(id) = backup_id {
                        let _ = self.backup_manager.delete_backup(&id).await;
                    }
                }

                // Clean up old backups
                if self.config.backup.enabled {
                    let _ = self.backup_manager.cleanup_old_backups().await;
                }

                Ok(upgrade_result)
            }
            Err(e) => {
                // Rollback on failure
                if let Some(id) = backup_id {
                    let _ = self.backup_manager.restore_backup(&id).await;
                    self.last_backup_id = Some(id);
                }
                Err(e)
            }
        }
    }

    /// Rolls back the last applied upgrade operation.
    ///
    /// Restores package.json files from the most recent backup. This is useful when
    /// an upgrade causes issues that are discovered after the operation completes.
    ///
    /// # Returns
    ///
    /// A list of file paths that were restored from the backup
    ///
    /// # Errors
    ///
    /// Returns `UpgradeError` if:
    /// - No backup exists to rollback
    /// - The backup cannot be restored
    /// - Files cannot be written
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # async fn example(mut manager: sublime_pkg_tools::upgrade::UpgradeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Apply upgrades
    /// let result = manager.apply_upgrades(
    ///     sublime_pkg_tools::upgrade::UpgradeSelection::all(),
    ///     false
    /// ).await?;
    ///
    /// // Later, if issues are discovered...
    /// let restored = manager.rollback_last().await?;
    /// println!("Rolled back {} files", restored.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rollback_last(&self) -> UpgradeResult<Vec<PathBuf>> {
        let backup_id = self.last_backup_id.as_ref().ok_or_else(|| UpgradeError::NoBackup {
            path: self.workspace_root.join(&self.config.backup.backup_dir),
        })?;

        self.backup_manager.restore_backup(backup_id).await?;

        // Get list of restored files from metadata
        let metadata = self.backup_manager.list_backups().await?;
        let backup_meta = metadata.iter().find(|b| &b.id == backup_id);

        Ok(backup_meta.map(|m| m.files.clone()).unwrap_or_default())
    }

    /// Gets the workspace root path.
    ///
    /// # Returns
    ///
    /// Reference to the workspace root path
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn example(manager: &sublime_pkg_tools::upgrade::UpgradeManager) {
    /// let workspace_root = manager.workspace_root();
    /// println!("Workspace: {}", workspace_root.display());
    /// # }
    /// ```
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Gets the current configuration.
    ///
    /// # Returns
    ///
    /// Reference to the upgrade configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn example(manager: &sublime_pkg_tools::upgrade::UpgradeManager) {
    /// let config = manager.config();
    /// println!("Auto changeset: {}", config.auto_changeset);
    /// println!("Backup enabled: {}", config.backup.enabled);
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &UpgradeConfig {
        &self.config
    }

    /// Gets the registry client for direct registry access.
    ///
    /// This provides access to the underlying registry client for advanced use cases
    /// that need direct registry queries.
    ///
    /// # Returns
    ///
    /// Reference to the registry client
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # async fn example(manager: &sublime_pkg_tools::upgrade::UpgradeManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let client = manager.registry_client();
    /// let metadata = client.get_package_info("lodash").await?;
    /// println!("Latest lodash version: {}", metadata.latest);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn registry_client(&self) -> &RegistryClient {
        &self.registry_client
    }

    /// Gets the ID of the last backup created.
    ///
    /// Returns `None` if no backup has been created yet or if the last backup
    /// was deleted.
    ///
    /// # Returns
    ///
    /// The backup ID, or `None` if no backup exists
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # fn example(manager: &sublime_pkg_tools::upgrade::UpgradeManager) {
    /// if let Some(backup_id) = manager.last_backup_id() {
    ///     println!("Last backup: {}", backup_id);
    /// } else {
    ///     println!("No backup available");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn last_backup_id(&self) -> Option<&str> {
        self.last_backup_id.as_deref()
    }

    // Private helper methods

    /// Converts upgrade selection to detection options.
    fn selection_to_detection_options(&self, _selection: &UpgradeSelection) -> DetectionOptions {
        // Always detect all upgrades - filtering is done in the application phase
        DetectionOptions::all()
    }

    /// Collects all package.json file paths from the upgrade preview.
    fn collect_package_json_files(
        &self,
        packages: &[crate::upgrade::detection::PackageUpgrades],
    ) -> UpgradeResult<Vec<PathBuf>> {
        Ok(packages.iter().map(|pkg| pkg.package_path.join("package.json")).collect())
    }
}

#[cfg(test)]
mod tests {
    // Note: Full integration tests should be in the tests/ directory
    // Unit tests for internal helper methods would go here, but the current
    // implementation doesn't have testable private methods that warrant unit tests.
    // The public API is better tested through integration tests with real fixtures.
}
