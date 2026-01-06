//! Upgrade command type definitions for Node.js bindings.
//!
//! # What
//!
//! This module defines all NAPI-compatible type structures for upgrade commands,
//! including input parameters and response data types. Upgrade commands enable
//! detection and application of dependency updates from npm registries, with
//! backup and restore capabilities for safe upgrade workflows.
//!
//! # How
//!
//! Types are defined with the `#[napi(object)]` attribute to be automatically
//! exposed as JavaScript objects. The module provides:
//!
//! - **Input Parameters**:
//!   - `UpgradeCheckParams`: Parameters for checking available upgrades
//!   - `UpgradeApplyParams`: Parameters for applying upgrades
//!   - `BackupListParams`: Parameters for listing backups
//!   - `BackupRestoreParams`: Parameters for restoring from backup
//!   - `BackupCleanParams`: Parameters for cleaning old backups
//!
//! - **Response Data**:
//!   - `UpgradeCheckData`: Response containing available upgrades
//!   - `UpgradeApplyData`: Response containing applied upgrades
//!   - `BackupListData`: Response containing backup list
//!   - `BackupRestoreData`: Response containing restore results
//!   - `BackupCleanData`: Response containing cleanup results
//!
//! - **Supporting Types**:
//!   - `PackageUpgradeInfo`: Information about upgrades for a single package
//!   - `DependencyUpgradeInfo`: Information about a single dependency upgrade
//!   - `UpgradeSummaryInfo`: Summary statistics for available upgrades
//!   - `UpgradeSelectionInfo`: Selection criteria for which upgrades to apply
//!   - `AppliedUpgradeInfo`: Information about a successfully applied upgrade
//!   - `ApplySummaryInfo`: Summary of upgrade application results
//!   - `BackupInfo`: Information about a single backup
//!
//! - **API Responses**: Type-safe response wrappers for each command
//!
//! All types implement `Clone`, `Debug`, and `Serialize` for flexibility in
//! testing and serialization scenarios.
//!
//! # Why
//!
//! Upgrade commands provide controlled dependency update workflows:
//!
//! - **Check**: Detect available upgrades without making changes
//! - **Apply**: Apply selected upgrades with optional backup creation
//! - **Backup List**: View available backups for potential rollback
//! - **Backup Restore**: Rollback to a previous state if upgrades cause issues
//! - **Backup Clean**: Remove old backups to free disk space
//!
//! These types provide:
//! - **Type safety**: Strong typing for JavaScript/TypeScript consumers
//! - **Documentation**: Self-documenting API through TypeScript definitions
//! - **Consistency**: Matches the CLI JSON output structure for compatibility
//! - **Validation**: Enables parameter validation before CLI execution
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import {
//!   upgradeCheck,
//!   upgradeApply,
//!   backupList,
//!   backupRestore,
//!   backupClean,
//!   UpgradeCheckParams,
//!   UpgradeApplyParams
//! } from '@websublime/workspace-tools';
//!
//! // Check for available upgrades
//! const checkParams: UpgradeCheckParams = {
//!   root: '.',
//!   includeMajor: false,
//!   includeMinor: true,
//!   includePatch: true
//! };
//! const checkResult = await upgradeCheck(checkParams);
//! if (checkResult.success) {
//!   console.log(`Found ${checkResult.data.summary.totalUpgrades} upgrades`);
//!   for (const pkg of checkResult.data.packages) {
//!     for (const dep of pkg.dependencies) {
//!       console.log(`${dep.name}: ${dep.currentVersion} -> ${dep.latestVersion}`);
//!     }
//!   }
//! }
//!
//! // Apply minor and patch upgrades with backup
//! const applyParams: UpgradeApplyParams = {
//!   root: '.',
//!   createBackup: true,
//!   selection: { minor: true, patch: true }
//! };
//! const applyResult = await upgradeApply(applyParams);
//! if (applyResult.success) {
//!   console.log(`Applied ${applyResult.data.summary.totalApplied} upgrades`);
//!   if (applyResult.data.backupId) {
//!     console.log(`Backup created: ${applyResult.data.backupId}`);
//!   }
//! }
//!
//! // List available backups
//! const listResult = await backupList({ root: '.' });
//! if (listResult.success) {
//!   for (const backup of listResult.data.backups) {
//!     console.log(`${backup.id}: ${backup.createdAt} (${backup.sizeBytes} bytes)`);
//!   }
//! }
//!
//! // Restore from backup if needed
//! if (applyResult.success && applyResult.data.backupId) {
//!   const restoreResult = await backupRestore({
//!     root: '.',
//!     backupId: applyResult.data.backupId
//!   });
//!   if (restoreResult.success) {
//!     console.log(`Restored ${restoreResult.data.packagesRestored} packages`);
//!   }
//! }
//!
//! // Clean old backups
//! const cleanResult = await backupClean({ root: '.', keepCount: 3 });
//! if (cleanResult.success) {
//!   console.log(`Cleaned ${cleanResult.data.backupsRemoved} old backups`);
//! }
//! ```
//!
//! ## Rust Usage (Internal)
//!
//! ```rust,ignore
//! use sublime_node_tools::types::upgrade::{
//!     UpgradeCheckParams, UpgradeCheckData, PackageUpgradeInfo,
//!     DependencyUpgradeInfo, UpgradeSummaryInfo
//! };
//!
//! // Creating params for validation
//! let params = UpgradeCheckParams::new(".")
//!     .with_include_minor(true)
//!     .with_include_patch(true);
//!
//! // Constructing response data
//! let dep_upgrade = DependencyUpgradeInfo::new(
//!     "lodash",
//!     "4.17.20",
//!     "4.17.21",
//!     "patch",
//!     "regular"
//! );
//! let pkg_upgrade = PackageUpgradeInfo::new(
//!     "@scope/pkg1",
//!     "packages/pkg1",
//! ).with_dependency(dep_upgrade);
//! ```

use napi_derive::napi;
use serde::Serialize;

use crate::error::ErrorInfo;

// ============================================================================
// Constants
// ============================================================================

/// Valid upgrade type values for dependency upgrades.
///
/// These values indicate the type of version change:
/// - `"major"`: Breaking changes (e.g., 1.0.0 → 2.0.0)
/// - `"minor"`: New features, backwards compatible (e.g., 1.0.0 → 1.1.0)
/// - `"patch"`: Bug fixes, backwards compatible (e.g., 1.0.0 → 1.0.1)
#[allow(dead_code)]
pub(crate) const VALID_UPGRADE_TYPES: &[&str] = &["major", "minor", "patch"];

/// Valid dependency type values for upgraded dependencies.
///
/// - `"regular"`: Standard dependencies (dependencies)
/// - `"dev"`: Development dependencies (devDependencies)
/// - `"peer"`: Peer dependencies (peerDependencies)
/// - `"optional"`: Optional dependencies (optionalDependencies)
#[allow(dead_code)]
pub(crate) const VALID_DEPENDENCY_TYPES: &[&str] = &["regular", "dev", "peer", "optional"];

/// Default number of backups to keep when cleaning old backups.
#[allow(dead_code)]
pub(crate) const DEFAULT_KEEP_COUNT: u32 = 5;

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for the upgrade check command.
///
/// This structure defines the parameters for checking available dependency
/// upgrades in the workspace. The check is a read-only operation that detects
/// which dependencies have newer versions available.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `include_major`: Whether to include major version upgrades
/// - `include_minor`: Whether to include minor version upgrades
/// - `include_patch`: Whether to include patch version upgrades
/// - `include_dev_dependencies`: Whether to check devDependencies
/// - `include_peer_dependencies`: Whether to check peerDependencies
/// - `packages`: Optional filter to specific packages
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeCheckParams {
///   root: string;
///   configPath?: string;
///   includeMajor?: boolean;
///   includeMinor?: boolean;
///   includePatch?: boolean;
///   includeDevDependencies?: boolean;
///   includePeerDependencies?: boolean;
///   packages?: string[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Check all upgrade types
/// const allParams: UpgradeCheckParams = { root: '.' };
///
/// // Check only minor and patch upgrades (safer)
/// const safeParams: UpgradeCheckParams = {
///   root: '/path/to/workspace',
///   includeMajor: false,
///   includeMinor: true,
///   includePatch: true
/// };
///
/// // Check specific packages only
/// const filteredParams: UpgradeCheckParams = {
///   root: '.',
///   packages: ['@scope/pkg1', '@scope/pkg2']
/// };
/// ```
// Allow dead_code - will be used in story 8.2 (upgradeCheck command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeCheckParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    /// For monorepos, this should point to the root where the package manager
    /// configuration is located.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations (`repo.config.json`, `repo.config.toml`,
    /// `repo.config.yaml`) within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Whether to include major version upgrades.
    ///
    /// Major upgrades may contain breaking changes and should be reviewed
    /// carefully. Defaults to `true` when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_major: Option<bool>,

    /// Whether to include minor version upgrades.
    ///
    /// Minor upgrades typically add new features while maintaining
    /// backwards compatibility. Defaults to `true` when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_minor: Option<bool>,

    /// Whether to include patch version upgrades.
    ///
    /// Patch upgrades typically contain bug fixes and are generally
    /// safe to apply. Defaults to `true` when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_patch: Option<bool>,

    /// Whether to include development dependencies.
    ///
    /// When `true`, devDependencies are also checked for upgrades.
    /// Defaults to `true` when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_dev_dependencies: Option<bool>,

    /// Whether to include peer dependencies.
    ///
    /// When `true`, peerDependencies are also checked for upgrades.
    /// Defaults to `false` when not specified since peer dependency
    /// upgrades require careful consideration.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_peer_dependencies: Option<bool>,

    /// Filter to specific packages.
    ///
    /// When provided, only these packages will be checked for upgrades.
    /// Package names should include scope if applicable.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,
}

#[allow(dead_code)]
impl UpgradeCheckParams {
    /// Creates a new `UpgradeCheckParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `UpgradeCheckParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = UpgradeCheckParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            include_major: None,
            include_minor: None,
            include_patch: None,
            include_dev_dependencies: None,
            include_peer_dependencies: None,
            packages: None,
        }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets whether to include major upgrades.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include major upgrades
    ///
    /// # Returns
    ///
    /// Self with the include_major flag set.
    #[must_use]
    pub fn with_include_major(mut self, include: bool) -> Self {
        self.include_major = Some(include);
        self
    }

    /// Sets whether to include minor upgrades.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include minor upgrades
    ///
    /// # Returns
    ///
    /// Self with the include_minor flag set.
    #[must_use]
    pub fn with_include_minor(mut self, include: bool) -> Self {
        self.include_minor = Some(include);
        self
    }

    /// Sets whether to include patch upgrades.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include patch upgrades
    ///
    /// # Returns
    ///
    /// Self with the include_patch flag set.
    #[must_use]
    pub fn with_include_patch(mut self, include: bool) -> Self {
        self.include_patch = Some(include);
        self
    }

    /// Sets whether to include development dependencies.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include dev dependencies
    ///
    /// # Returns
    ///
    /// Self with the include_dev_dependencies flag set.
    #[must_use]
    pub fn with_include_dev_dependencies(mut self, include: bool) -> Self {
        self.include_dev_dependencies = Some(include);
        self
    }

    /// Sets whether to include peer dependencies.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include peer dependencies
    ///
    /// # Returns
    ///
    /// Self with the include_peer_dependencies flag set.
    #[must_use]
    pub fn with_include_peer_dependencies(mut self, include: bool) -> Self {
        self.include_peer_dependencies = Some(include);
        self
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Convenience method to set all inclusion flags at once.
    ///
    /// # Arguments
    ///
    /// * `major` - Whether to include major upgrades
    /// * `minor` - Whether to include minor upgrades
    /// * `patch` - Whether to include patch upgrades
    ///
    /// # Returns
    ///
    /// Self with all inclusion flags set.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Only include minor and patch (safe upgrades)
    /// let params = UpgradeCheckParams::new(".")
    ///     .with_upgrade_levels(false, true, true);
    /// ```
    #[must_use]
    pub fn with_upgrade_levels(mut self, major: bool, minor: bool, patch: bool) -> Self {
        self.include_major = Some(major);
        self.include_minor = Some(minor);
        self.include_patch = Some(patch);
        self
    }
}

/// Input parameters for the upgrade apply command.
///
/// This structure defines the parameters for applying selected dependency
/// upgrades. The apply operation modifies package.json files and optionally
/// creates backups and changesets.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `create_backup`: Whether to create a backup before applying
/// - `create_changeset`: Whether to create a changeset for the upgrades
/// - `selection`: Criteria for which upgrades to apply
/// - `dry_run`: Whether to simulate without making changes
/// - `packages`: Optional filter to specific packages
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeApplyParams {
///   root: string;
///   configPath?: string;
///   createBackup?: boolean;
///   createChangeset?: boolean;
///   selection?: UpgradeSelectionInfo;
///   dryRun?: boolean;
///   packages?: string[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Apply all upgrades with backup
/// const allParams: UpgradeApplyParams = {
///   root: '.',
///   createBackup: true
/// };
///
/// // Apply only patch upgrades (safest)
/// const patchOnly: UpgradeApplyParams = {
///   root: '.',
///   createBackup: true,
///   selection: { major: false, minor: false, patch: true }
/// };
///
/// // Dry run to preview changes
/// const preview: UpgradeApplyParams = {
///   root: '.',
///   dryRun: true
/// };
/// ```
// Allow dead_code - will be used in story 8.3 (upgradeApply command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeApplyParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Whether to create a backup before applying upgrades.
    ///
    /// When `true`, creates a backup of all package.json files that can
    /// be restored if the upgrades cause issues. Strongly recommended
    /// for production use.
    ///
    /// Defaults to the value in the configuration file, or `true` if not
    /// specified anywhere.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_backup: Option<bool>,

    /// Whether to create a changeset for the upgrades.
    ///
    /// When `true`, creates a changeset documenting all the dependency
    /// updates. This integrates with the bump workflow to include
    /// upgrade information in release notes.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_changeset: Option<bool>,

    /// Criteria for which upgrades to apply.
    ///
    /// Allows fine-grained control over which types of upgrades are applied.
    /// If not provided, applies all available upgrades.
    #[napi(ts_type = "UpgradeSelectionInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<UpgradeSelectionInfo>,

    /// Whether to perform a dry run.
    ///
    /// When `true`, simulates the upgrade process without actually
    /// modifying any files. Useful for previewing what would change.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,

    /// Filter to specific packages.
    ///
    /// When provided, only upgrades for these packages will be applied.
    /// Package names should include scope if applicable.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,
}

#[allow(dead_code)]
impl UpgradeApplyParams {
    /// Creates a new `UpgradeApplyParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `UpgradeApplyParams` instance with default optional values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = UpgradeApplyParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            config_path: None,
            create_backup: None,
            create_changeset: None,
            selection: None,
            dry_run: None,
            packages: None,
        }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets whether to create a backup.
    ///
    /// # Arguments
    ///
    /// * `create_backup` - Whether to create a backup
    ///
    /// # Returns
    ///
    /// Self with the create_backup flag set.
    #[must_use]
    pub fn with_create_backup(mut self, create_backup: bool) -> Self {
        self.create_backup = Some(create_backup);
        self
    }

    /// Sets whether to create a changeset.
    ///
    /// # Arguments
    ///
    /// * `create_changeset` - Whether to create a changeset
    ///
    /// # Returns
    ///
    /// Self with the create_changeset flag set.
    #[must_use]
    pub fn with_create_changeset(mut self, create_changeset: bool) -> Self {
        self.create_changeset = Some(create_changeset);
        self
    }

    /// Sets the upgrade selection criteria.
    ///
    /// # Arguments
    ///
    /// * `selection` - The selection criteria
    ///
    /// # Returns
    ///
    /// Self with the selection set.
    #[must_use]
    pub fn with_selection(mut self, selection: UpgradeSelectionInfo) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Sets whether to perform a dry run.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - Whether to perform a dry run
    ///
    /// # Returns
    ///
    /// Self with the dry_run flag set.
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = Some(dry_run);
        self
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }
}

/// Input parameters for the backup list command.
///
/// This structure defines the parameters for listing available backups
/// in the workspace. Backups are created by the upgrade apply command
/// when `createBackup` is enabled.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupListParams {
///   root: string;
///   configPath?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const params: BackupListParams = { root: '.' };
/// const result = await backupList(params);
/// ```
// Allow dead_code - will be used in story 8.4 (backupList command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupListParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

#[allow(dead_code)]
impl BackupListParams {
    /// Creates a new `BackupListParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `BackupListParams` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BackupListParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }
}

/// Input parameters for the backup restore command.
///
/// This structure defines the parameters for restoring package.json files
/// from a previous backup. This effectively rolls back dependency changes
/// to a known state.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `backup_id`: The ID of the backup to restore (required)
/// - `config_path`: Optional path to a custom configuration file
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupRestoreParams {
///   root: string;
///   backupId: string;
///   configPath?: string;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const params: BackupRestoreParams = {
///   root: '.',
///   backupId: 'backup-2024-01-15-123456'
/// };
/// const result = await backupRestore(params);
/// ```
// Allow dead_code - will be used in story 8.4 (backupRestore command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupRestoreParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// The ID of the backup to restore.
    ///
    /// This should match an ID returned by the backupList command.
    /// Backup IDs are typically in the format `backup-YYYY-MM-DD-HHMMSS`.
    pub backup_id: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

#[allow(dead_code)]
impl BackupRestoreParams {
    /// Creates a new `BackupRestoreParams` with required fields.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    /// * `backup_id` - The ID of the backup to restore
    ///
    /// # Returns
    ///
    /// A new `BackupRestoreParams` instance.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BackupRestoreParams::new(".", "backup-2024-01-15-123456");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>, backup_id: impl Into<String>) -> Self {
        Self { root: root.into(), backup_id: backup_id.into(), config_path: None }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }
}

/// Input parameters for the backup clean command.
///
/// This structure defines the parameters for cleaning (removing) old backups.
/// This helps manage disk space by removing older backups while keeping
/// the most recent ones.
///
/// # Fields
///
/// - `root`: The workspace root directory path (required)
/// - `config_path`: Optional path to a custom configuration file
/// - `keep_count`: Number of recent backups to keep (optional, defaults to 5)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupCleanParams {
///   root: string;
///   configPath?: string;
///   keepCount?: number;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Keep last 3 backups, remove older ones
/// const params: BackupCleanParams = {
///   root: '.',
///   keepCount: 3
/// };
/// const result = await backupClean(params);
/// ```
// Allow dead_code - will be used in story 8.4 (backupClean command)
#[allow(dead_code)]
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanParams {
    /// Workspace root directory path.
    ///
    /// This is the absolute or relative path to the root of the workspace.
    pub root: String,

    /// Optional custom configuration file path.
    ///
    /// If not provided, the command will search for configuration files
    /// in standard locations within the workspace root.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,

    /// Number of recent backups to keep.
    ///
    /// Backups are sorted by creation date, and the most recent ones
    /// are kept. Older backups beyond this count are removed.
    /// Defaults to 5 if not specified.
    #[napi(ts_type = "number | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_count: Option<u32>,
}

#[allow(dead_code)]
impl BackupCleanParams {
    /// Creates a new `BackupCleanParams` with the required root path.
    ///
    /// # Arguments
    ///
    /// * `root` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// A new `BackupCleanParams` instance with default keep_count.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let params = BackupCleanParams::new("/path/to/workspace");
    /// ```
    #[must_use]
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into(), config_path: None, keep_count: None }
    }

    /// Sets the config path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Self with the config path set.
    #[must_use]
    pub fn with_config_path(mut self, config_path: impl Into<String>) -> Self {
        self.config_path = Some(config_path.into());
        self
    }

    /// Sets the number of backups to keep.
    ///
    /// # Arguments
    ///
    /// * `keep_count` - Number of recent backups to keep
    ///
    /// # Returns
    ///
    /// Self with the keep_count set.
    #[must_use]
    pub fn with_keep_count(mut self, keep_count: u32) -> Self {
        self.keep_count = Some(keep_count);
        self
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Selection criteria for which upgrades to apply.
///
/// This structure allows fine-grained control over which types of upgrades
/// are applied during the upgrade apply operation.
///
/// # Fields
///
/// - `major`: Whether to apply major version upgrades
/// - `minor`: Whether to apply minor version upgrades
/// - `patch`: Whether to apply patch version upgrades
/// - `packages`: Optional filter to specific packages
/// - `dependencies`: Optional filter to specific dependencies
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeSelectionInfo {
///   major?: boolean;
///   minor?: boolean;
///   patch?: boolean;
///   packages?: string[];
///   dependencies?: string[];
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// // Apply only patch upgrades (safest)
/// const patchOnly: UpgradeSelectionInfo = {
///   major: false,
///   minor: false,
///   patch: true
/// };
///
/// // Apply minor and patch, but not major
/// const safeUpgrades: UpgradeSelectionInfo = {
///   major: false,
///   minor: true,
///   patch: true
/// };
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeSelectionInfo {
    /// Whether to apply major version upgrades.
    ///
    /// Major upgrades may contain breaking changes. Defaults to `true`
    /// when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<bool>,

    /// Whether to apply minor version upgrades.
    ///
    /// Minor upgrades typically add new features. Defaults to `true`
    /// when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<bool>,

    /// Whether to apply patch version upgrades.
    ///
    /// Patch upgrades typically contain bug fixes. Defaults to `true`
    /// when not specified.
    #[napi(ts_type = "boolean | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<bool>,

    /// Filter to specific packages.
    ///
    /// When provided, only upgrades for these packages will be applied.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    /// Filter to specific dependencies.
    ///
    /// When provided, only these specific dependencies will be upgraded.
    #[napi(ts_type = "string[] | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
}

#[allow(dead_code)]
impl UpgradeSelectionInfo {
    /// Creates a new `UpgradeSelectionInfo` that applies all upgrade types.
    ///
    /// # Returns
    ///
    /// A new `UpgradeSelectionInfo` with all flags set to `true`.
    #[must_use]
    pub fn all() -> Self {
        Self {
            major: Some(true),
            minor: Some(true),
            patch: Some(true),
            packages: None,
            dependencies: None,
        }
    }

    /// Creates a selection for patch upgrades only.
    ///
    /// This is the safest option as patch upgrades typically only
    /// contain bug fixes.
    ///
    /// # Returns
    ///
    /// A new `UpgradeSelectionInfo` for patch upgrades only.
    #[must_use]
    pub fn patch_only() -> Self {
        Self {
            major: Some(false),
            minor: Some(false),
            patch: Some(true),
            packages: None,
            dependencies: None,
        }
    }

    /// Creates a selection for minor and patch upgrades.
    ///
    /// This is a moderate option that avoids breaking changes
    /// while still getting new features.
    ///
    /// # Returns
    ///
    /// A new `UpgradeSelectionInfo` for minor and patch upgrades.
    #[must_use]
    pub fn minor_and_patch() -> Self {
        Self {
            major: Some(false),
            minor: Some(true),
            patch: Some(true),
            packages: None,
            dependencies: None,
        }
    }

    /// Creates a selection for specific packages.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to include
    ///
    /// # Returns
    ///
    /// A new `UpgradeSelectionInfo` filtered to specific packages.
    #[must_use]
    pub fn for_packages(packages: Vec<String>) -> Self {
        Self {
            major: Some(true),
            minor: Some(true),
            patch: Some(true),
            packages: Some(packages),
            dependencies: None,
        }
    }

    /// Creates a selection for specific dependencies.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - List of dependency names to include
    ///
    /// # Returns
    ///
    /// A new `UpgradeSelectionInfo` filtered to specific dependencies.
    #[must_use]
    pub fn for_dependencies(dependencies: Vec<String>) -> Self {
        Self {
            major: Some(true),
            minor: Some(true),
            patch: Some(true),
            packages: None,
            dependencies: Some(dependencies),
        }
    }

    /// Sets the packages filter.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to filter
    ///
    /// # Returns
    ///
    /// Self with the packages filter set.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Sets the dependencies filter.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - List of dependency names to filter
    ///
    /// # Returns
    ///
    /// Self with the dependencies filter set.
    #[must_use]
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }
}

impl Default for UpgradeSelectionInfo {
    fn default() -> Self {
        Self::all()
    }
}

/// Information about a single dependency upgrade.
///
/// This structure contains details about an available or applied upgrade
/// for a specific dependency.
///
/// # Fields
///
/// - `name`: The dependency name
/// - `current_version`: The current version in package.json
/// - `latest_version`: The latest available version
/// - `upgrade_type`: The type of upgrade (major, minor, patch)
/// - `dependency_type`: Where the dependency is defined
///
/// # TypeScript Definition
///
/// ```typescript
/// interface DependencyUpgradeInfo {
///   name: string;
///   currentVersion: string;
///   latestVersion: string;
///   upgradeType: string;
///   dependencyType: string;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct DependencyUpgradeInfo {
    /// The name of the dependency.
    ///
    /// This is the package name as it appears in package.json,
    /// including any scope prefix.
    pub name: String,

    /// The current version specified in package.json.
    ///
    /// This is the version range or exact version currently specified.
    pub current_version: String,

    /// The latest available version from the registry.
    ///
    /// This is the exact version that would be installed if the
    /// upgrade is applied.
    pub latest_version: String,

    /// The type of version upgrade.
    ///
    /// One of: `"major"`, `"minor"`, `"patch"`
    pub upgrade_type: String,

    /// The type of dependency relationship.
    ///
    /// One of: `"regular"`, `"dev"`, `"peer"`, `"optional"`
    pub dependency_type: String,
}

#[allow(dead_code)]
impl DependencyUpgradeInfo {
    /// Creates a new `DependencyUpgradeInfo`.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `current_version` - Current version in package.json
    /// * `latest_version` - Latest available version
    /// * `upgrade_type` - Type of upgrade (major, minor, patch)
    /// * `dependency_type` - Type of dependency (regular, dev, peer, optional)
    ///
    /// # Returns
    ///
    /// A new `DependencyUpgradeInfo` instance.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        current_version: impl Into<String>,
        latest_version: impl Into<String>,
        upgrade_type: impl Into<String>,
        dependency_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            current_version: current_version.into(),
            latest_version: latest_version.into(),
            upgrade_type: upgrade_type.into(),
            dependency_type: dependency_type.into(),
        }
    }

    /// Creates a new patch upgrade for a regular dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `current` - Current version
    /// * `latest` - Latest version
    ///
    /// # Returns
    ///
    /// A new `DependencyUpgradeInfo` for a patch upgrade.
    #[must_use]
    pub fn patch(
        name: impl Into<String>,
        current: impl Into<String>,
        latest: impl Into<String>,
    ) -> Self {
        Self::new(name, current, latest, "patch", "regular")
    }

    /// Creates a new minor upgrade for a regular dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `current` - Current version
    /// * `latest` - Latest version
    ///
    /// # Returns
    ///
    /// A new `DependencyUpgradeInfo` for a minor upgrade.
    #[must_use]
    pub fn minor(
        name: impl Into<String>,
        current: impl Into<String>,
        latest: impl Into<String>,
    ) -> Self {
        Self::new(name, current, latest, "minor", "regular")
    }

    /// Creates a new major upgrade for a regular dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `current` - Current version
    /// * `latest` - Latest version
    ///
    /// # Returns
    ///
    /// A new `DependencyUpgradeInfo` for a major upgrade.
    #[must_use]
    pub fn major(
        name: impl Into<String>,
        current: impl Into<String>,
        latest: impl Into<String>,
    ) -> Self {
        Self::new(name, current, latest, "major", "regular")
    }

    /// Creates a dev dependency upgrade.
    ///
    /// # Arguments
    ///
    /// * `name` - The dependency name
    /// * `current` - Current version
    /// * `latest` - Latest version
    /// * `upgrade_type` - Type of upgrade
    ///
    /// # Returns
    ///
    /// A new `DependencyUpgradeInfo` for a dev dependency.
    #[must_use]
    pub fn dev(
        name: impl Into<String>,
        current: impl Into<String>,
        latest: impl Into<String>,
        upgrade_type: impl Into<String>,
    ) -> Self {
        Self::new(name, current, latest, upgrade_type, "dev")
    }

    /// Returns true if this is a major upgrade.
    #[must_use]
    pub fn is_major(&self) -> bool {
        self.upgrade_type == "major"
    }

    /// Returns true if this is a minor upgrade.
    #[must_use]
    pub fn is_minor(&self) -> bool {
        self.upgrade_type == "minor"
    }

    /// Returns true if this is a patch upgrade.
    #[must_use]
    pub fn is_patch(&self) -> bool {
        self.upgrade_type == "patch"
    }

    /// Returns true if this is a dev dependency.
    #[must_use]
    pub fn is_dev_dependency(&self) -> bool {
        self.dependency_type == "dev"
    }
}

/// Information about available upgrades for a single package.
///
/// This structure contains all available dependency upgrades for a
/// specific workspace package.
///
/// # Fields
///
/// - `package_name`: The package name
/// - `package_path`: The path to the package directory
/// - `dependencies`: List of available dependency upgrades
///
/// # TypeScript Definition
///
/// ```typescript
/// interface PackageUpgradeInfo {
///   packageName: string;
///   packagePath: string;
///   dependencies: DependencyUpgradeInfo[];
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct PackageUpgradeInfo {
    /// The name of the package.
    ///
    /// This is the package name from package.json, including any scope.
    pub package_name: String,

    /// The path to the package directory.
    ///
    /// This is the relative path from the workspace root to the
    /// package directory.
    pub package_path: String,

    /// List of available dependency upgrades.
    ///
    /// Contains information about each dependency that has an
    /// upgrade available.
    pub dependencies: Vec<DependencyUpgradeInfo>,
}

#[allow(dead_code)]
impl PackageUpgradeInfo {
    /// Creates a new `PackageUpgradeInfo`.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name
    /// * `package_path` - Path to the package directory
    ///
    /// # Returns
    ///
    /// A new `PackageUpgradeInfo` with an empty dependency list.
    #[must_use]
    pub fn new(package_name: impl Into<String>, package_path: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            package_path: package_path.into(),
            dependencies: Vec::new(),
        }
    }

    /// Creates a new `PackageUpgradeInfo` with dependencies.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name
    /// * `package_path` - Path to the package directory
    /// * `dependencies` - List of dependency upgrades
    ///
    /// # Returns
    ///
    /// A new `PackageUpgradeInfo` with the provided dependencies.
    #[must_use]
    pub fn with_dependencies(
        package_name: impl Into<String>,
        package_path: impl Into<String>,
        dependencies: Vec<DependencyUpgradeInfo>,
    ) -> Self {
        Self { package_name: package_name.into(), package_path: package_path.into(), dependencies }
    }

    /// Adds a dependency upgrade to this package.
    ///
    /// # Arguments
    ///
    /// * `dependency` - The dependency upgrade to add
    ///
    /// # Returns
    ///
    /// Self with the dependency added.
    #[must_use]
    pub fn with_dependency(mut self, dependency: DependencyUpgradeInfo) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Returns the number of available upgrades.
    #[must_use]
    pub fn upgrade_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Returns the number of major upgrades.
    #[must_use]
    pub fn major_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.is_major()).count()
    }

    /// Returns the number of minor upgrades.
    #[must_use]
    pub fn minor_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.is_minor()).count()
    }

    /// Returns the number of patch upgrades.
    #[must_use]
    pub fn patch_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.is_patch()).count()
    }

    /// Returns true if there are any major upgrades.
    #[must_use]
    pub fn has_major_upgrades(&self) -> bool {
        self.dependencies.iter().any(DependencyUpgradeInfo::is_major)
    }
}

/// Summary of available upgrades.
///
/// This structure provides aggregate statistics about available upgrades
/// across all packages.
///
/// # Fields
///
/// - `packages_analyzed`: Number of packages checked
/// - `total_upgrades`: Total number of available upgrades
/// - `major_upgrades`: Number of major version upgrades
/// - `minor_upgrades`: Number of minor version upgrades
/// - `patch_upgrades`: Number of patch version upgrades
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeSummaryInfo {
///   packagesAnalyzed: number;
///   totalUpgrades: number;
///   majorUpgrades: number;
///   minorUpgrades: number;
///   patchUpgrades: number;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeSummaryInfo {
    /// Number of packages that were analyzed.
    pub packages_analyzed: u32,

    /// Total number of available upgrades.
    pub total_upgrades: u32,

    /// Number of major version upgrades available.
    pub major_upgrades: u32,

    /// Number of minor version upgrades available.
    pub minor_upgrades: u32,

    /// Number of patch version upgrades available.
    pub patch_upgrades: u32,
}

#[allow(dead_code)]
impl UpgradeSummaryInfo {
    /// Creates a new `UpgradeSummaryInfo`.
    ///
    /// # Arguments
    ///
    /// * `packages_analyzed` - Number of packages analyzed
    /// * `total_upgrades` - Total upgrade count
    /// * `major_upgrades` - Major upgrade count
    /// * `minor_upgrades` - Minor upgrade count
    /// * `patch_upgrades` - Patch upgrade count
    ///
    /// # Returns
    ///
    /// A new `UpgradeSummaryInfo` instance.
    #[must_use]
    pub fn new(
        packages_analyzed: u32,
        total_upgrades: u32,
        major_upgrades: u32,
        minor_upgrades: u32,
        patch_upgrades: u32,
    ) -> Self {
        Self { packages_analyzed, total_upgrades, major_upgrades, minor_upgrades, patch_upgrades }
    }

    /// Creates an empty summary (no upgrades found).
    ///
    /// # Arguments
    ///
    /// * `packages_analyzed` - Number of packages that were analyzed
    ///
    /// # Returns
    ///
    /// A new `UpgradeSummaryInfo` with zero upgrades.
    #[must_use]
    pub fn empty(packages_analyzed: u32) -> Self {
        Self {
            packages_analyzed,
            total_upgrades: 0,
            major_upgrades: 0,
            minor_upgrades: 0,
            patch_upgrades: 0,
        }
    }

    /// Creates a summary from a list of package upgrades.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package upgrade information
    ///
    /// # Returns
    ///
    /// A summary calculated from the provided packages.
    ///
    /// # Note
    ///
    /// The package count is truncated to `u32::MAX` if it exceeds that value,
    /// which is acceptable since workspaces with over 4 billion packages are
    /// not realistic.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_packages(packages: &[PackageUpgradeInfo]) -> Self {
        let mut major_upgrades = 0u32;
        let mut minor_upgrades = 0u32;
        let mut patch_upgrades = 0u32;

        for pkg in packages {
            for dep in &pkg.dependencies {
                match dep.upgrade_type.as_str() {
                    "major" => major_upgrades += 1,
                    "minor" => minor_upgrades += 1,
                    "patch" => patch_upgrades += 1,
                    _ => {}
                }
            }
        }

        let total_upgrades = major_upgrades + minor_upgrades + patch_upgrades;

        // Truncation is acceptable: workspaces with >4 billion packages are unrealistic
        Self {
            packages_analyzed: packages.len() as u32,
            total_upgrades,
            major_upgrades,
            minor_upgrades,
            patch_upgrades,
        }
    }

    /// Returns true if there are any breaking (major) changes.
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.major_upgrades > 0
    }

    /// Returns true if there are no upgrades available.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_upgrades == 0
    }
}

impl Default for UpgradeSummaryInfo {
    fn default() -> Self {
        Self::empty(0)
    }
}

/// Information about a successfully applied upgrade.
///
/// This structure contains details about a single dependency upgrade
/// that was successfully applied.
///
/// # Fields
///
/// - `package_name`: The package that was modified
/// - `dependency_name`: The dependency that was upgraded
/// - `old_version`: The previous version
/// - `new_version`: The new version
/// - `upgrade_type`: The type of upgrade
///
/// # TypeScript Definition
///
/// ```typescript
/// interface AppliedUpgradeInfo {
///   packageName: string;
///   dependencyName: string;
///   oldVersion: string;
///   newVersion: string;
///   upgradeType: string;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct AppliedUpgradeInfo {
    /// The name of the package that was modified.
    pub package_name: String,

    /// The name of the dependency that was upgraded.
    pub dependency_name: String,

    /// The previous version that was replaced.
    pub old_version: String,

    /// The new version that was applied.
    pub new_version: String,

    /// The type of upgrade that was applied.
    ///
    /// One of: `"major"`, `"minor"`, `"patch"`
    pub upgrade_type: String,
}

#[allow(dead_code)]
impl AppliedUpgradeInfo {
    /// Creates a new `AppliedUpgradeInfo`.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package that was modified
    /// * `dependency_name` - The dependency that was upgraded
    /// * `old_version` - Previous version
    /// * `new_version` - New version
    /// * `upgrade_type` - Type of upgrade
    ///
    /// # Returns
    ///
    /// A new `AppliedUpgradeInfo` instance.
    #[must_use]
    pub fn new(
        package_name: impl Into<String>,
        dependency_name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
        upgrade_type: impl Into<String>,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            dependency_name: dependency_name.into(),
            old_version: old_version.into(),
            new_version: new_version.into(),
            upgrade_type: upgrade_type.into(),
        }
    }

    /// Creates from a `DependencyUpgradeInfo` for a specific package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package that was modified
    /// * `upgrade` - The dependency upgrade info
    ///
    /// # Returns
    ///
    /// A new `AppliedUpgradeInfo` instance.
    #[must_use]
    pub fn from_dependency_upgrade(
        package_name: impl Into<String>,
        upgrade: &DependencyUpgradeInfo,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            dependency_name: upgrade.name.clone(),
            old_version: upgrade.current_version.clone(),
            new_version: upgrade.latest_version.clone(),
            upgrade_type: upgrade.upgrade_type.clone(),
        }
    }
}

/// Information about a skipped upgrade with reason.
///
/// This structure contains details about a dependency upgrade that
/// was not applied, along with the reason it was skipped.
///
/// # Fields
///
/// - `package_name`: The package where the upgrade was available
/// - `dependency_name`: The dependency that was skipped
/// - `current_version`: The current version
/// - `available_version`: The available version that was skipped
/// - `reason`: Why the upgrade was skipped
///
/// # TypeScript Definition
///
/// ```typescript
/// interface SkippedUpgradeInfo {
///   packageName: string;
///   dependencyName: string;
///   currentVersion: string;
///   availableVersion: string;
///   reason: string;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct SkippedUpgradeInfo {
    /// The name of the package where the upgrade was available.
    pub package_name: String,

    /// The name of the dependency that was skipped.
    pub dependency_name: String,

    /// The current version in package.json.
    pub current_version: String,

    /// The available version that was not applied.
    pub available_version: String,

    /// The reason the upgrade was skipped.
    ///
    /// Common reasons include: filtered by selection criteria,
    /// conflicting requirements, or user exclusion.
    pub reason: String,
}

#[allow(dead_code)]
impl SkippedUpgradeInfo {
    /// Creates a new `SkippedUpgradeInfo`.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name
    /// * `dependency_name` - The dependency name
    /// * `current_version` - Current version
    /// * `available_version` - Available version
    /// * `reason` - Reason for skipping
    ///
    /// # Returns
    ///
    /// A new `SkippedUpgradeInfo` instance.
    #[must_use]
    pub fn new(
        package_name: impl Into<String>,
        dependency_name: impl Into<String>,
        current_version: impl Into<String>,
        available_version: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            dependency_name: dependency_name.into(),
            current_version: current_version.into(),
            available_version: available_version.into(),
            reason: reason.into(),
        }
    }

    /// Creates a skipped upgrade for filtered selection.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name
    /// * `dependency_name` - The dependency name
    /// * `current_version` - Current version
    /// * `available_version` - Available version
    ///
    /// # Returns
    ///
    /// A new `SkippedUpgradeInfo` with a filtered reason.
    #[must_use]
    pub fn filtered(
        package_name: impl Into<String>,
        dependency_name: impl Into<String>,
        current_version: impl Into<String>,
        available_version: impl Into<String>,
    ) -> Self {
        Self::new(
            package_name,
            dependency_name,
            current_version,
            available_version,
            "Filtered by selection criteria",
        )
    }
}

/// Information about a failed upgrade attempt.
///
/// This structure contains details about a dependency upgrade that
/// failed to apply, along with the error message.
///
/// # Fields
///
/// - `package_name`: The package where the upgrade was attempted
/// - `dependency_name`: The dependency that failed to upgrade
/// - `current_version`: The current version
/// - `target_version`: The version that was attempted
/// - `error`: The error message
///
/// # TypeScript Definition
///
/// ```typescript
/// interface FailedUpgradeInfo {
///   packageName: string;
///   dependencyName: string;
///   currentVersion: string;
///   targetVersion: string;
///   error: string;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct FailedUpgradeInfo {
    /// The name of the package where the upgrade was attempted.
    pub package_name: String,

    /// The name of the dependency that failed to upgrade.
    pub dependency_name: String,

    /// The current version in package.json.
    pub current_version: String,

    /// The version that was attempted.
    pub target_version: String,

    /// The error message describing what went wrong.
    pub error: String,
}

#[allow(dead_code)]
impl FailedUpgradeInfo {
    /// Creates a new `FailedUpgradeInfo`.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name
    /// * `dependency_name` - The dependency name
    /// * `current_version` - Current version
    /// * `target_version` - Target version
    /// * `error` - Error message
    ///
    /// # Returns
    ///
    /// A new `FailedUpgradeInfo` instance.
    #[must_use]
    pub fn new(
        package_name: impl Into<String>,
        dependency_name: impl Into<String>,
        current_version: impl Into<String>,
        target_version: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            dependency_name: dependency_name.into(),
            current_version: current_version.into(),
            target_version: target_version.into(),
            error: error.into(),
        }
    }
}

/// Summary of upgrade application results.
///
/// This structure provides aggregate statistics about the results
/// of applying upgrades.
///
/// # Fields
///
/// - `total_applied`: Number of upgrades successfully applied
/// - `total_skipped`: Number of upgrades that were skipped
/// - `total_failed`: Number of upgrades that failed
/// - `packages_modified`: List of packages that were modified
///
/// # TypeScript Definition
///
/// ```typescript
/// interface ApplySummaryInfo {
///   totalApplied: number;
///   totalSkipped: number;
///   totalFailed: number;
///   packagesModified: string[];
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ApplySummaryInfo {
    /// Number of upgrades successfully applied.
    pub total_applied: u32,

    /// Number of upgrades that were skipped.
    pub total_skipped: u32,

    /// Number of upgrades that failed.
    pub total_failed: u32,

    /// List of package names that were modified.
    pub packages_modified: Vec<String>,
}

#[allow(dead_code)]
impl ApplySummaryInfo {
    /// Creates a new `ApplySummaryInfo`.
    ///
    /// # Arguments
    ///
    /// * `total_applied` - Number applied
    /// * `total_skipped` - Number skipped
    /// * `total_failed` - Number failed
    /// * `packages_modified` - List of modified packages
    ///
    /// # Returns
    ///
    /// A new `ApplySummaryInfo` instance.
    #[must_use]
    pub fn new(
        total_applied: u32,
        total_skipped: u32,
        total_failed: u32,
        packages_modified: Vec<String>,
    ) -> Self {
        Self { total_applied, total_skipped, total_failed, packages_modified }
    }

    /// Creates an empty summary (nothing happened).
    ///
    /// # Returns
    ///
    /// A new `ApplySummaryInfo` with zero counts.
    #[must_use]
    pub fn empty() -> Self {
        Self { total_applied: 0, total_skipped: 0, total_failed: 0, packages_modified: Vec::new() }
    }

    /// Returns true if all upgrades were successful.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.total_failed == 0 && self.total_applied > 0
    }

    /// Returns true if any upgrades failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.total_failed > 0
    }

    /// Returns the total number of upgrades processed.
    #[must_use]
    pub fn total_processed(&self) -> u32 {
        self.total_applied + self.total_skipped + self.total_failed
    }
}

impl Default for ApplySummaryInfo {
    fn default() -> Self {
        Self::empty()
    }
}

/// Information about a backup.
///
/// This structure contains metadata about a backup that was created
/// during upgrade operations.
///
/// # Fields
///
/// - `id`: Unique identifier for the backup
/// - `created_at`: When the backup was created (ISO 8601 format)
/// - `packages`: List of packages included in the backup
/// - `size_bytes`: Size of the backup in bytes
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupInfo {
///   id: string;
///   createdAt: string;
///   packages: string[];
///   sizeBytes: number;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    /// Unique identifier for the backup.
    ///
    /// Typically in the format `backup-YYYY-MM-DD-HHMMSS`.
    pub id: String,

    /// When the backup was created.
    ///
    /// ISO 8601 format (e.g., `2024-01-15T12:34:56Z`).
    pub created_at: String,

    /// List of package names included in the backup.
    pub packages: Vec<String>,

    /// Size of the backup in bytes.
    ///
    /// Note: Uses f64 for JavaScript compatibility. f64 can represent
    /// integers up to 2^53 without precision loss, which is sufficient
    /// for file sizes up to 9 petabytes.
    pub size_bytes: f64,
}

#[allow(dead_code)]
impl BackupInfo {
    /// Creates a new `BackupInfo`.
    ///
    /// # Arguments
    ///
    /// * `id` - Backup identifier
    /// * `created_at` - Creation timestamp
    /// * `packages` - List of packages
    /// * `size_bytes` - Size in bytes
    ///
    /// # Returns
    ///
    /// A new `BackupInfo` instance.
    ///
    /// # Note
    ///
    /// The `size_bytes` is converted from `u64` to `f64` for JavaScript compatibility.
    /// While f64 can only represent integers exactly up to 2^53, this is sufficient
    /// for file sizes up to 9 petabytes.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(
        id: impl Into<String>,
        created_at: impl Into<String>,
        packages: Vec<String>,
        size_bytes: u64,
    ) -> Self {
        // Convert u64 to f64 for JavaScript compatibility
        // Precision loss is acceptable: files larger than 9 petabytes are unrealistic
        Self {
            id: id.into(),
            created_at: created_at.into(),
            packages,
            size_bytes: size_bytes as f64,
        }
    }

    /// Returns the number of packages in the backup.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }
}

// ============================================================================
// Response Data Types
// ============================================================================

/// Response data for the upgrade check command.
///
/// This structure contains the results of checking for available
/// dependency upgrades.
///
/// # Fields
///
/// - `packages`: List of packages with available upgrades
/// - `summary`: Aggregate statistics about available upgrades
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeCheckData {
///   packages: PackageUpgradeInfo[];
///   summary: UpgradeSummaryInfo;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeCheckData {
    /// List of packages with available upgrades.
    ///
    /// Each entry contains information about the package and its
    /// dependencies that have upgrades available.
    pub packages: Vec<PackageUpgradeInfo>,

    /// Summary statistics about available upgrades.
    pub summary: UpgradeSummaryInfo,
}

#[allow(dead_code)]
impl UpgradeCheckData {
    /// Creates a new `UpgradeCheckData`.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package upgrades
    /// * `summary` - Upgrade summary
    ///
    /// # Returns
    ///
    /// A new `UpgradeCheckData` instance.
    #[must_use]
    pub fn new(packages: Vec<PackageUpgradeInfo>, summary: UpgradeSummaryInfo) -> Self {
        Self { packages, summary }
    }

    /// Creates an empty result (no upgrades available).
    ///
    /// # Arguments
    ///
    /// * `packages_analyzed` - Number of packages that were checked
    ///
    /// # Returns
    ///
    /// A new `UpgradeCheckData` with no upgrades.
    #[must_use]
    pub fn empty(packages_analyzed: u32) -> Self {
        Self { packages: Vec::new(), summary: UpgradeSummaryInfo::empty(packages_analyzed) }
    }

    /// Creates from a list of packages, calculating the summary.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package upgrades
    ///
    /// # Returns
    ///
    /// A new `UpgradeCheckData` with calculated summary.
    #[must_use]
    pub fn from_packages(packages: Vec<PackageUpgradeInfo>) -> Self {
        let summary = UpgradeSummaryInfo::from_packages(&packages);
        Self { packages, summary }
    }

    /// Returns true if there are any upgrades available.
    #[must_use]
    pub fn has_upgrades(&self) -> bool {
        self.summary.total_upgrades > 0
    }

    /// Returns true if there are any breaking changes.
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.summary.has_breaking_changes()
    }
}

/// Response data for the upgrade apply command.
///
/// This structure contains the results of applying dependency upgrades.
///
/// # Fields
///
/// - `applied`: List of successfully applied upgrades
/// - `skipped`: List of skipped upgrades with reasons
/// - `failed`: List of failed upgrades with errors
/// - `summary`: Aggregate statistics about the operation
/// - `backup_id`: ID of the backup created (if backup was enabled)
/// - `changeset_id`: ID of the changeset created (if enabled)
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeApplyData {
///   applied: AppliedUpgradeInfo[];
///   skipped: SkippedUpgradeInfo[];
///   failed: FailedUpgradeInfo[];
///   summary: ApplySummaryInfo;
///   backupId?: string;
///   changesetId?: string;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeApplyData {
    /// List of successfully applied upgrades.
    pub applied: Vec<AppliedUpgradeInfo>,

    /// List of upgrades that were skipped.
    pub skipped: Vec<SkippedUpgradeInfo>,

    /// List of upgrades that failed.
    pub failed: Vec<FailedUpgradeInfo>,

    /// Summary statistics about the operation.
    pub summary: ApplySummaryInfo,

    /// ID of the backup created, if backup was enabled.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_id: Option<String>,

    /// ID of the changeset created, if changeset creation was enabled.
    #[napi(ts_type = "string | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changeset_id: Option<String>,
}

#[allow(dead_code)]
impl UpgradeApplyData {
    /// Creates a new `UpgradeApplyData`.
    ///
    /// # Arguments
    ///
    /// * `applied` - List of applied upgrades
    /// * `skipped` - List of skipped upgrades
    /// * `failed` - List of failed upgrades
    /// * `summary` - Apply summary
    ///
    /// # Returns
    ///
    /// A new `UpgradeApplyData` instance.
    #[must_use]
    pub fn new(
        applied: Vec<AppliedUpgradeInfo>,
        skipped: Vec<SkippedUpgradeInfo>,
        failed: Vec<FailedUpgradeInfo>,
        summary: ApplySummaryInfo,
    ) -> Self {
        Self { applied, skipped, failed, summary, backup_id: None, changeset_id: None }
    }

    /// Creates an empty result (nothing was applied).
    ///
    /// # Returns
    ///
    /// A new `UpgradeApplyData` with empty lists.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            applied: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            summary: ApplySummaryInfo::empty(),
            backup_id: None,
            changeset_id: None,
        }
    }

    /// Sets the backup ID.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - The backup ID
    ///
    /// # Returns
    ///
    /// Self with the backup ID set.
    #[must_use]
    pub fn with_backup_id(mut self, backup_id: impl Into<String>) -> Self {
        self.backup_id = Some(backup_id.into());
        self
    }

    /// Sets the changeset ID.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - The changeset ID
    ///
    /// # Returns
    ///
    /// Self with the changeset ID set.
    #[must_use]
    pub fn with_changeset_id(mut self, changeset_id: impl Into<String>) -> Self {
        self.changeset_id = Some(changeset_id.into());
        self
    }

    /// Returns true if a backup was created.
    #[must_use]
    pub fn has_backup(&self) -> bool {
        self.backup_id.is_some()
    }

    /// Returns true if a changeset was created.
    #[must_use]
    pub fn has_changeset(&self) -> bool {
        self.changeset_id.is_some()
    }

    /// Returns true if all upgrades succeeded.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.summary.all_succeeded()
    }

    /// Returns true if any upgrades failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.summary.has_failures()
    }
}

/// Response data for the backup list command.
///
/// This structure contains the list of available backups.
///
/// # Fields
///
/// - `backups`: List of available backups
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupListData {
///   backups: BackupInfo[];
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupListData {
    /// List of available backups, ordered by creation date (newest first).
    pub backups: Vec<BackupInfo>,
}

#[allow(dead_code)]
impl BackupListData {
    /// Creates a new `BackupListData`.
    ///
    /// # Arguments
    ///
    /// * `backups` - List of backups
    ///
    /// # Returns
    ///
    /// A new `BackupListData` instance.
    #[must_use]
    pub fn new(backups: Vec<BackupInfo>) -> Self {
        Self { backups }
    }

    /// Creates an empty result (no backups available).
    ///
    /// # Returns
    ///
    /// A new `BackupListData` with an empty list.
    #[must_use]
    pub fn empty() -> Self {
        Self { backups: Vec::new() }
    }

    /// Returns the number of backups.
    #[must_use]
    pub fn count(&self) -> usize {
        self.backups.len()
    }

    /// Returns true if there are no backups.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.backups.is_empty()
    }
}

/// Response data for the backup restore command.
///
/// This structure contains the results of restoring from a backup.
///
/// # Fields
///
/// - `backup_id`: The ID of the backup that was restored
/// - `packages_restored`: Number of packages restored
/// - `packages`: List of package names that were restored
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupRestoreData {
///   backupId: string;
///   packagesRestored: number;
///   packages: string[];
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupRestoreData {
    /// The ID of the backup that was restored.
    pub backup_id: String,

    /// Number of packages that were restored.
    pub packages_restored: u32,

    /// List of package names that were restored.
    pub packages: Vec<String>,
}

#[allow(dead_code)]
impl BackupRestoreData {
    /// Creates a new `BackupRestoreData`.
    ///
    /// # Arguments
    ///
    /// * `backup_id` - The backup ID
    /// * `packages` - List of restored packages
    ///
    /// # Returns
    ///
    /// A new `BackupRestoreData` instance.
    ///
    /// # Note
    ///
    /// The package count is truncated to `u32::MAX` if it exceeds that value,
    /// which is acceptable since workspaces with over 4 billion packages are
    /// not realistic.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(backup_id: impl Into<String>, packages: Vec<String>) -> Self {
        // Truncation is acceptable: workspaces with >4 billion packages are unrealistic
        let packages_restored = packages.len() as u32;
        Self { backup_id: backup_id.into(), packages_restored, packages }
    }
}

/// Response data for the backup clean command.
///
/// This structure contains the results of cleaning old backups.
///
/// # Fields
///
/// - `backups_removed`: Number of backups that were removed
/// - `backups_kept`: Number of backups that were kept
/// - `bytes_freed`: Approximate bytes freed by the cleanup
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupCleanData {
///   backupsRemoved: number;
///   backupsKept: number;
///   bytesFreed: number;
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanData {
    /// Number of backups that were removed.
    pub backups_removed: u32,

    /// Number of backups that were kept.
    pub backups_kept: u32,

    /// Approximate number of bytes freed by the cleanup.
    ///
    /// Note: Uses f64 for JavaScript compatibility. f64 can represent
    /// integers up to 2^53 without precision loss, which is sufficient
    /// for file sizes up to 9 petabytes.
    pub bytes_freed: f64,
}

#[allow(dead_code)]
impl BackupCleanData {
    /// Creates a new `BackupCleanData`.
    ///
    /// # Arguments
    ///
    /// * `backups_removed` - Number removed
    /// * `backups_kept` - Number kept
    /// * `bytes_freed` - Bytes freed
    ///
    /// # Returns
    ///
    /// A new `BackupCleanData` instance.
    ///
    /// # Note
    ///
    /// The `bytes_freed` is converted from `u64` to `f64` for JavaScript compatibility.
    /// While f64 can only represent integers exactly up to 2^53, this is sufficient
    /// for file sizes up to 9 petabytes.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(backups_removed: u32, backups_kept: u32, bytes_freed: u64) -> Self {
        // Convert u64 to f64 for JavaScript compatibility
        // Precision loss is acceptable: files larger than 9 petabytes are unrealistic
        Self { backups_removed, backups_kept, bytes_freed: bytes_freed as f64 }
    }

    /// Creates a result indicating nothing was cleaned.
    ///
    /// # Arguments
    ///
    /// * `backups_kept` - Number of backups that exist
    ///
    /// # Returns
    ///
    /// A new `BackupCleanData` with zero removed.
    #[must_use]
    pub fn nothing_to_clean(backups_kept: u32) -> Self {
        Self { backups_removed: 0, backups_kept, bytes_freed: 0.0 }
    }
}

// ============================================================================
// API Response Types
// ============================================================================

/// API response for the upgrade check command.
///
/// This structure wraps `UpgradeCheckData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeCheckApiResponse {
///   success: boolean;
///   data?: UpgradeCheckData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await upgradeCheck({ root: '.' });
///
/// if (result.success) {
///   console.log(`Found ${result.data.summary.totalUpgrades} upgrades`);
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeCheckApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The check result data if successful.
    ///
    /// Contains the list of available upgrades and summary statistics.
    #[napi(ts_type = "UpgradeCheckData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<UpgradeCheckData>,

    /// Error information if the operation failed.
    ///
    /// Contains the error code, message, and context when the
    /// operation fails.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl UpgradeCheckApiResponse {
    /// Creates a successful response with check data.
    ///
    /// # Arguments
    ///
    /// * `data` - The upgrade check data
    ///
    /// # Returns
    ///
    /// A new successful `UpgradeCheckApiResponse`.
    #[must_use]
    pub fn success(data: UpgradeCheckData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `UpgradeCheckApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the upgrade apply command.
///
/// This structure wraps `UpgradeApplyData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface UpgradeApplyApiResponse {
///   success: boolean;
///   data?: UpgradeApplyData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await upgradeApply({
///   root: '.',
///   createBackup: true,
///   selection: { patch: true }
/// });
///
/// if (result.success) {
///   console.log(`Applied ${result.data.summary.totalApplied} upgrades`);
///   if (result.data.backupId) {
///     console.log(`Backup: ${result.data.backupId}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeApplyApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The apply result data if successful.
    ///
    /// Contains information about applied, skipped, and failed upgrades.
    #[napi(ts_type = "UpgradeApplyData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<UpgradeApplyData>,

    /// Error information if the operation failed.
    ///
    /// Contains the error code, message, and context when the
    /// operation fails.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl UpgradeApplyApiResponse {
    /// Creates a successful response with apply data.
    ///
    /// # Arguments
    ///
    /// * `data` - The upgrade apply data
    ///
    /// # Returns
    ///
    /// A new successful `UpgradeApplyApiResponse`.
    #[must_use]
    pub fn success(data: UpgradeApplyData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `UpgradeApplyApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the backup list command.
///
/// This structure wraps `BackupListData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupListApiResponse {
///   success: boolean;
///   data?: BackupListData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await backupList({ root: '.' });
///
/// if (result.success) {
///   for (const backup of result.data.backups) {
///     console.log(`${backup.id}: ${backup.createdAt}`);
///   }
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupListApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The list of backups if successful.
    #[napi(ts_type = "BackupListData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BackupListData>,

    /// Error information if the operation failed.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BackupListApiResponse {
    /// Creates a successful response with backup list data.
    ///
    /// # Arguments
    ///
    /// * `data` - The backup list data
    ///
    /// # Returns
    ///
    /// A new successful `BackupListApiResponse`.
    #[must_use]
    pub fn success(data: BackupListData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BackupListApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the backup restore command.
///
/// This structure wraps `BackupRestoreData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupRestoreApiResponse {
///   success: boolean;
///   data?: BackupRestoreData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await backupRestore({
///   root: '.',
///   backupId: 'backup-2024-01-15-123456'
/// });
///
/// if (result.success) {
///   console.log(`Restored ${result.data.packagesRestored} packages`);
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupRestoreApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The restore result data if successful.
    #[napi(ts_type = "BackupRestoreData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BackupRestoreData>,

    /// Error information if the operation failed.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BackupRestoreApiResponse {
    /// Creates a successful response with restore data.
    ///
    /// # Arguments
    ///
    /// * `data` - The backup restore data
    ///
    /// # Returns
    ///
    /// A new successful `BackupRestoreApiResponse`.
    #[must_use]
    pub fn success(data: BackupRestoreData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BackupRestoreApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// API response for the backup clean command.
///
/// This structure wraps `BackupCleanData` in the standard `ApiResponse`
/// format, providing a consistent interface for success and error cases.
///
/// # TypeScript Definition
///
/// ```typescript
/// interface BackupCleanApiResponse {
///   success: boolean;
///   data?: BackupCleanData;
///   error?: ErrorInfo;
/// }
/// ```
///
/// # Examples
///
/// ```typescript
/// const result = await backupClean({ root: '.', keepCount: 3 });
///
/// if (result.success) {
///   console.log(`Removed ${result.data.backupsRemoved} backups`);
///   console.log(`Freed ${result.data.bytesFreed} bytes`);
/// } else {
///   console.error(`Error: ${result.error.message}`);
/// }
/// ```
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanApiResponse {
    /// Whether the operation was successful.
    pub success: bool,

    /// The cleanup result data if successful.
    #[napi(ts_type = "BackupCleanData | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<BackupCleanData>,

    /// Error information if the operation failed.
    #[napi(ts_type = "ErrorInfo | undefined")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl BackupCleanApiResponse {
    /// Creates a successful response with cleanup data.
    ///
    /// # Arguments
    ///
    /// * `data` - The backup clean data
    ///
    /// # Returns
    ///
    /// A new successful `BackupCleanApiResponse`.
    #[must_use]
    pub fn success(data: BackupCleanData) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failure response with error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information
    ///
    /// # Returns
    ///
    /// A new failed `BackupCleanApiResponse`.
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Returns true if the response indicates success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns true if the response indicates failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }
}
