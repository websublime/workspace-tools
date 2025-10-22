//! Result types for upgrade application operations.
//!
//! **What**: Provides types that represent the outcome of applying dependency upgrades,
//! including details of what was changed, files modified, and summary statistics.
//!
//! **How**: The `UpgradeResult` struct contains all information about an upgrade operation,
//! including individual upgrade details (`AppliedUpgrade`), file paths, backup information,
//! and aggregated statistics (`ApplySummary`). Supports both dry-run and actual operations.
//!
//! **Why**: To provide comprehensive feedback about upgrade operations, enabling users to
//! understand exactly what changed, track modifications for rollback, and review statistics
//! for reporting and audit purposes.

use crate::types::DependencyType;
use crate::upgrade::registry::UpgradeType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of applying dependency upgrades.
///
/// Contains comprehensive information about what was changed during an upgrade
/// operation, including individual upgrade details, modified files, backup location,
/// and summary statistics. Supports both dry-run and actual operations.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::upgrade::{UpgradeManager, UpgradeSelection};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let manager: UpgradeManager = todo!();
/// // Apply upgrades and inspect result
/// let selection = UpgradeSelection::patch_only();
/// let result = manager.apply_upgrades(selection, false).await?;
///
/// println!("Modified {} packages", result.summary.packages_modified);
/// println!("Upgraded {} dependencies", result.summary.dependencies_upgraded);
/// println!("Applied {} patch upgrades", result.summary.patch_upgrades);
///
/// if let Some(backup) = result.backup_path {
///     println!("Backup saved to: {}", backup.display());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {
    /// Whether this was a dry-run operation.
    ///
    /// When `true`, no files were actually modified. The result represents
    /// what would have been changed.
    pub dry_run: bool,

    /// List of all applied upgrades.
    ///
    /// Contains detailed information about each dependency that was upgraded,
    /// including the package it belongs to, old and new versions, and upgrade type.
    pub applied: Vec<AppliedUpgrade>,

    /// Paths to all modified package.json files.
    ///
    /// Lists the absolute paths to all package.json files that were modified
    /// (or would be modified in dry-run mode). Empty for dry-run operations.
    pub modified_files: Vec<PathBuf>,

    /// Location of the backup directory.
    ///
    /// When upgrades are applied (not dry-run), all modified package.json files
    /// are backed up to this directory before modification. `None` for dry-run
    /// operations or when backup is disabled.
    pub backup_path: Option<PathBuf>,

    /// ID of automatically created changeset.
    ///
    /// When automatic changeset creation is enabled, contains the ID of the
    /// created changeset. `None` for dry-run operations or when automatic
    /// changeset creation is disabled.
    pub changeset_id: Option<String>,

    /// Summary statistics of the operation.
    ///
    /// Aggregated counts and metrics about the upgrade operation.
    pub summary: ApplySummary,
}

impl UpgradeResult {
    /// Creates a new dry-run result.
    ///
    /// # Arguments
    ///
    /// * `applied` - List of upgrades that would be applied
    /// * `summary` - Summary statistics
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{UpgradeResult, ApplySummary};
    ///
    /// let summary = ApplySummary::new();
    /// let result = UpgradeResult::dry_run(vec![], summary);
    /// assert!(result.dry_run);
    /// assert!(result.modified_files.is_empty());
    /// assert!(result.backup_path.is_none());
    /// ```
    #[must_use]
    pub fn dry_run(applied: Vec<AppliedUpgrade>, summary: ApplySummary) -> Self {
        Self {
            dry_run: true,
            applied,
            modified_files: Vec::new(),
            backup_path: None,
            changeset_id: None,
            summary,
        }
    }

    /// Creates a new result for an actual operation.
    ///
    /// # Arguments
    ///
    /// * `applied` - List of applied upgrades
    /// * `modified_files` - Paths to modified package.json files
    /// * `backup_path` - Optional backup directory path
    /// * `changeset_id` - Optional automatically created changeset ID
    /// * `summary` - Summary statistics
    #[must_use]
    pub fn applied(
        applied: Vec<AppliedUpgrade>,
        modified_files: Vec<PathBuf>,
        backup_path: Option<PathBuf>,
        changeset_id: Option<String>,
        summary: ApplySummary,
    ) -> Self {
        Self { dry_run: false, applied, modified_files, backup_path, changeset_id, summary }
    }

    /// Returns whether any upgrades were applied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{UpgradeResult, ApplySummary};
    ///
    /// let result = UpgradeResult::dry_run(vec![], ApplySummary::new());
    /// assert!(!result.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.applied.is_empty()
    }

    /// Returns the number of packages that were modified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{UpgradeResult, ApplySummary};
    ///
    /// let result = UpgradeResult::dry_run(vec![], ApplySummary::new());
    /// assert_eq!(result.packages_modified(), 0);
    /// ```
    #[must_use]
    pub fn packages_modified(&self) -> usize {
        self.summary.packages_modified
    }

    /// Returns the total number of dependencies that were upgraded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{UpgradeResult, ApplySummary};
    ///
    /// let result = UpgradeResult::dry_run(vec![], ApplySummary::new());
    /// assert_eq!(result.dependencies_upgraded(), 0);
    /// ```
    #[must_use]
    pub fn dependencies_upgraded(&self) -> usize {
        self.summary.dependencies_upgraded
    }
}

/// Details of a single applied upgrade.
///
/// Represents a single dependency upgrade within a package, including
/// the package location, dependency information, version change, and
/// upgrade classification.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
/// use sublime_pkg_tools::types::DependencyType;
/// use std::path::PathBuf;
///
/// let upgrade = AppliedUpgrade {
///     package_path: PathBuf::from("packages/my-package"),
///     dependency_name: "lodash".to_string(),
///     dependency_type: DependencyType::Regular,
///     old_version: "4.17.20".to_string(),
///     new_version: "4.17.21".to_string(),
///     upgrade_type: UpgradeType::Patch,
/// };
///
/// assert!(upgrade.is_patch());
/// assert!(!upgrade.is_breaking());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedUpgrade {
    /// Absolute path to the package directory.
    ///
    /// Points to the directory containing the package.json file where
    /// this upgrade was applied.
    pub package_path: PathBuf,

    /// Name of the upgraded dependency.
    ///
    /// The package name as it appears in the dependencies section
    /// (e.g., "lodash", "@types/node", "@myorg/utils").
    pub dependency_name: String,

    /// Type of dependency that was upgraded.
    ///
    /// Indicates which section of package.json this dependency belongs to:
    /// dependencies, devDependencies, peerDependencies, or optionalDependencies.
    pub dependency_type: DependencyType,

    /// Previous version specification.
    ///
    /// The version spec as it appeared in package.json before the upgrade
    /// (e.g., "^4.17.20", "~2.0.0", "1.2.3").
    pub old_version: String,

    /// New version specification.
    ///
    /// The updated version spec written to package.json
    /// (e.g., "^4.17.21", "~2.1.0", "1.2.4").
    pub new_version: String,

    /// Classification of the upgrade.
    ///
    /// Indicates whether this is a major, minor, or patch upgrade based
    /// on semantic versioning.
    pub upgrade_type: UpgradeType,
}

impl AppliedUpgrade {
    /// Returns whether this is a patch upgrade.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
    /// use sublime_pkg_tools::types::DependencyType;
    /// use std::path::PathBuf;
    ///
    /// let upgrade = AppliedUpgrade {
    ///     package_path: PathBuf::from("."),
    ///     dependency_name: "lodash".to_string(),
    ///     dependency_type: DependencyType::Regular,
    ///     old_version: "4.17.20".to_string(),
    ///     new_version: "4.17.21".to_string(),
    ///     upgrade_type: UpgradeType::Patch,
    /// };
    ///
    /// assert!(upgrade.is_patch());
    /// ```
    #[must_use]
    pub fn is_patch(&self) -> bool {
        self.upgrade_type == UpgradeType::Patch
    }

    /// Returns whether this is a minor upgrade.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
    /// use sublime_pkg_tools::types::DependencyType;
    /// use std::path::PathBuf;
    ///
    /// let upgrade = AppliedUpgrade {
    ///     package_path: PathBuf::from("."),
    ///     dependency_name: "react".to_string(),
    ///     dependency_type: DependencyType::Regular,
    ///     old_version: "17.0.0".to_string(),
    ///     new_version: "17.1.0".to_string(),
    ///     upgrade_type: UpgradeType::Minor,
    /// };
    ///
    /// assert!(upgrade.is_minor());
    /// ```
    #[must_use]
    pub fn is_minor(&self) -> bool {
        self.upgrade_type == UpgradeType::Minor
    }

    /// Returns whether this is a major upgrade.
    ///
    /// Major upgrades may contain breaking changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
    /// use sublime_pkg_tools::types::DependencyType;
    /// use std::path::PathBuf;
    ///
    /// let upgrade = AppliedUpgrade {
    ///     package_path: PathBuf::from("."),
    ///     dependency_name: "webpack".to_string(),
    ///     dependency_type: DependencyType::Dev,
    ///     old_version: "4.46.0".to_string(),
    ///     new_version: "5.0.0".to_string(),
    ///     upgrade_type: UpgradeType::Major,
    /// };
    ///
    /// assert!(upgrade.is_major());
    /// assert!(upgrade.is_breaking());
    /// ```
    #[must_use]
    pub fn is_major(&self) -> bool {
        self.upgrade_type == UpgradeType::Major
    }

    /// Returns whether this upgrade may contain breaking changes.
    ///
    /// Currently equivalent to `is_major()`, as major version bumps
    /// typically indicate breaking changes per semantic versioning.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
    /// use sublime_pkg_tools::types::DependencyType;
    /// use std::path::PathBuf;
    ///
    /// let upgrade = AppliedUpgrade {
    ///     package_path: PathBuf::from("."),
    ///     dependency_name: "express".to_string(),
    ///     dependency_type: DependencyType::Regular,
    ///     old_version: "4.18.0".to_string(),
    ///     new_version: "5.0.0".to_string(),
    ///     upgrade_type: UpgradeType::Major,
    /// };
    ///
    /// assert!(upgrade.is_breaking());
    /// ```
    #[must_use]
    pub fn is_breaking(&self) -> bool {
        self.is_major()
    }

    /// Returns a display string for the version change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{AppliedUpgrade, UpgradeType};
    /// use sublime_pkg_tools::types::DependencyType;
    /// use std::path::PathBuf;
    ///
    /// let upgrade = AppliedUpgrade {
    ///     package_path: PathBuf::from("."),
    ///     dependency_name: "lodash".to_string(),
    ///     dependency_type: DependencyType::Regular,
    ///     old_version: "4.17.20".to_string(),
    ///     new_version: "4.17.21".to_string(),
    ///     upgrade_type: UpgradeType::Patch,
    /// };
    ///
    /// assert_eq!(upgrade.version_change(), "4.17.20 → 4.17.21");
    /// ```
    #[must_use]
    pub fn version_change(&self) -> String {
        format!("{} → {}", self.old_version, self.new_version)
    }
}

/// Summary statistics of an upgrade operation.
///
/// Provides aggregated counts and metrics about what was upgraded,
/// broken down by package count, dependency count, and upgrade type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::upgrade::ApplySummary;
///
/// let summary = ApplySummary {
///     packages_modified: 3,
///     dependencies_upgraded: 5,
///     direct_updates: 4,
///     propagated_updates: 1,
///     dependency_updates: 5,
///     major_upgrades: 1,
///     minor_upgrades: 2,
///     patch_upgrades: 2,
///     applied_at: chrono::Utc::now(),
/// };
///
/// assert_eq!(summary.total_upgrades(), 5);
/// assert!(summary.has_major_upgrades());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplySummary {
    /// Total number of packages that were modified.
    ///
    /// Counts unique package.json files that had at least one dependency upgraded.
    pub packages_modified: usize,

    /// Total number of dependencies that were upgraded.
    ///
    /// Counts individual dependency upgrades across all packages.
    pub dependencies_upgraded: usize,

    /// Number of direct updates.
    ///
    /// Updates that were explicitly selected for upgrade (not propagated).
    /// Note: This field is for future use with dependency propagation features.
    pub direct_updates: usize,

    /// Number of propagated updates.
    ///
    /// Updates that were applied due to dependency propagation.
    /// Note: This field is for future use with dependency propagation features.
    pub propagated_updates: usize,

    /// Total dependency updates across all sections.
    ///
    /// Includes updates to dependencies, devDependencies, peerDependencies,
    /// and optionalDependencies.
    pub dependency_updates: usize,

    /// Number of major version upgrades.
    ///
    /// Count of dependencies upgraded to a new major version (e.g., 1.x → 2.x).
    pub major_upgrades: usize,

    /// Number of minor version upgrades.
    ///
    /// Count of dependencies upgraded to a new minor version (e.g., 1.2.x → 1.3.x).
    pub minor_upgrades: usize,

    /// Number of patch version upgrades.
    ///
    /// Count of dependencies upgraded to a new patch version (e.g., 1.2.3 → 1.2.4).
    pub patch_upgrades: usize,

    /// Timestamp when the operation was performed.
    ///
    /// For dry-run operations, this is when the preview was generated.
    pub applied_at: DateTime<Utc>,
}

impl ApplySummary {
    /// Creates a new empty summary with the current timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::ApplySummary;
    ///
    /// let summary = ApplySummary::new();
    /// assert_eq!(summary.packages_modified, 0);
    /// assert_eq!(summary.dependencies_upgraded, 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            packages_modified: 0,
            dependencies_upgraded: 0,
            direct_updates: 0,
            propagated_updates: 0,
            dependency_updates: 0,
            major_upgrades: 0,
            minor_upgrades: 0,
            patch_upgrades: 0,
            applied_at: Utc::now(),
        }
    }

    /// Returns the total number of upgrades performed.
    ///
    /// Sum of major, minor, and patch upgrades.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::ApplySummary;
    ///
    /// let mut summary = ApplySummary::new();
    /// summary.major_upgrades = 1;
    /// summary.minor_upgrades = 2;
    /// summary.patch_upgrades = 3;
    ///
    /// assert_eq!(summary.total_upgrades(), 6);
    /// ```
    #[must_use]
    pub fn total_upgrades(&self) -> usize {
        self.major_upgrades + self.minor_upgrades + self.patch_upgrades
    }

    /// Returns whether any major upgrades were performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::ApplySummary;
    ///
    /// let mut summary = ApplySummary::new();
    /// assert!(!summary.has_major_upgrades());
    ///
    /// summary.major_upgrades = 1;
    /// assert!(summary.has_major_upgrades());
    /// ```
    #[must_use]
    pub fn has_major_upgrades(&self) -> bool {
        self.major_upgrades > 0
    }

    /// Returns whether any changes were made.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::ApplySummary;
    ///
    /// let summary = ApplySummary::new();
    /// assert!(!summary.has_changes());
    ///
    /// let mut summary = ApplySummary::new();
    /// summary.patch_upgrades = 1;
    /// summary.dependencies_upgraded = 1;
    /// assert!(summary.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.dependencies_upgraded > 0
    }
}

impl Default for ApplySummary {
    fn default() -> Self {
        Self::new()
    }
}
