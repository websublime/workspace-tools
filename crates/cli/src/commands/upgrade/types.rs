//! Type definitions for upgrade commands.
//!
//! This module defines data structures used across upgrade command implementations.
//!
//! # What
//!
//! Provides:
//! - `DetectionOptions` - Options for upgrade detection
//! - `UpgradeSummary` - Summary of detected upgrades
//! - JSON response structures for upgrade commands
//!
//! # How
//!
//! These types:
//! 1. Are serializable for JSON output
//! 2. Mirror structures from `sublime-package-tools`
//! 3. Provide a stable API for command results
//! 4. Include display formatting for human output
//!
//! # Why
//!
//! Separating types from command logic:
//! - Improves testability
//! - Enables reuse across commands
//! - Provides clear API boundaries
//! - Simplifies serialization logic

use serde::{Deserialize, Serialize};

/// Summary statistics for upgrade detection.
///
/// Provides counts of upgrades by type for display in summary sections.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::types::UpgradeSummary;
///
/// let summary = UpgradeSummary {
///     total_packages: 5,
///     packages_with_upgrades: 3,
///     total_upgrades: 12,
///     major_upgrades: 2,
///     minor_upgrades: 7,
///     patch_upgrades: 3,
/// };
///
/// assert_eq!(summary.total_upgrades, 12);
/// assert_eq!(summary.non_breaking_count(), 10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeSummary {
    /// Total number of packages analyzed
    #[serde(rename = "totalPackages")]
    pub total_packages: usize,

    /// Number of packages with available upgrades
    #[serde(rename = "packagesWithUpgrades")]
    pub packages_with_upgrades: usize,

    /// Total number of upgrades across all packages
    #[serde(rename = "totalUpgrades")]
    pub total_upgrades: usize,

    /// Number of major version upgrades
    #[serde(rename = "major")]
    pub major_upgrades: usize,

    /// Number of minor version upgrades
    #[serde(rename = "minor")]
    pub minor_upgrades: usize,

    /// Number of patch version upgrades
    #[serde(rename = "patch")]
    pub patch_upgrades: usize,
}

impl UpgradeSummary {
    /// Creates a new empty upgrade summary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::UpgradeSummary;
    ///
    /// let summary = UpgradeSummary::new();
    /// assert_eq!(summary.total_upgrades, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            total_packages: 0,
            packages_with_upgrades: 0,
            total_upgrades: 0,
            major_upgrades: 0,
            minor_upgrades: 0,
            patch_upgrades: 0,
        }
    }

    /// Returns the count of non-breaking upgrades (minor + patch).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::UpgradeSummary;
    ///
    /// let summary = UpgradeSummary {
    ///     total_packages: 5,
    ///     packages_with_upgrades: 3,
    ///     total_upgrades: 12,
    ///     major_upgrades: 2,
    ///     minor_upgrades: 7,
    ///     patch_upgrades: 3,
    /// };
    ///
    /// assert_eq!(summary.non_breaking_count(), 10);
    /// ```
    pub fn non_breaking_count(&self) -> usize {
        self.minor_upgrades + self.patch_upgrades
    }

    /// Returns true if there are any upgrades available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::UpgradeSummary;
    ///
    /// let empty = UpgradeSummary::new();
    /// assert!(!empty.has_upgrades());
    ///
    /// let with_upgrades = UpgradeSummary {
    ///     total_packages: 1,
    ///     packages_with_upgrades: 1,
    ///     total_upgrades: 1,
    ///     major_upgrades: 0,
    ///     minor_upgrades: 1,
    ///     patch_upgrades: 0,
    /// };
    /// assert!(with_upgrades.has_upgrades());
    /// ```
    pub fn has_upgrades(&self) -> bool {
        self.total_upgrades > 0
    }
}

impl Default for UpgradeSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON response structure for upgrade check command.
///
/// Wraps the upgrade check results in a standard JSON format with success flag,
/// package details, and summary statistics.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::types::UpgradeCheckResponse;
/// use serde_json::json;
///
/// let response = UpgradeCheckResponse {
///     success: true,
///     packages: vec![],
///     summary: Default::default(),
/// };
///
/// let json = serde_json::to_string_pretty(&response).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeCheckResponse {
    /// Whether the command succeeded
    pub success: bool,

    /// Package upgrade details
    pub packages: Vec<PackageUpgradeInfo>,

    /// Summary statistics
    pub summary: UpgradeSummary,
}

/// Information about available upgrades for a single package.
///
/// Contains the package name, path, and list of available dependency upgrades.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::types::PackageUpgradeInfo;
///
/// let package_info = PackageUpgradeInfo {
///     name: "@org/core".to_string(),
///     path: "packages/core".to_string(),
///     upgrades: vec![],
/// };
///
/// assert_eq!(package_info.name, "@org/core");
/// assert_eq!(package_info.upgrade_count(), 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpgradeInfo {
    /// Package name
    pub name: String,

    /// Relative path to package from workspace root
    pub path: String,

    /// Available upgrades for this package's dependencies
    pub upgrades: Vec<DependencyUpgradeInfo>,
}

impl PackageUpgradeInfo {
    /// Returns the number of upgrades for this package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::PackageUpgradeInfo;
    ///
    /// let package_info = PackageUpgradeInfo {
    ///     name: "test".to_string(),
    ///     path: "packages/test".to_string(),
    ///     upgrades: vec![],
    /// };
    ///
    /// assert_eq!(package_info.upgrade_count(), 0);
    /// ```
    pub fn upgrade_count(&self) -> usize {
        self.upgrades.len()
    }

    /// Returns true if this package has any upgrades available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::PackageUpgradeInfo;
    ///
    /// let package_info = PackageUpgradeInfo {
    ///     name: "test".to_string(),
    ///     path: "packages/test".to_string(),
    ///     upgrades: vec![],
    /// };
    ///
    /// assert!(!package_info.has_upgrades());
    /// ```
    pub fn has_upgrades(&self) -> bool {
        !self.upgrades.is_empty()
    }
}

/// Information about a single dependency upgrade.
///
/// Contains details about an available upgrade for a specific dependency
/// including current version, latest version, and upgrade type.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::types::DependencyUpgradeInfo;
///
/// let upgrade = DependencyUpgradeInfo {
///     package: "typescript".to_string(),
///     current_version: "5.0.0".to_string(),
///     latest_version: "5.3.3".to_string(),
///     upgrade_type: "minor".to_string(),
///     breaking: false,
/// };
///
/// assert!(!upgrade.breaking);
/// assert_eq!(upgrade.upgrade_type, "minor");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyUpgradeInfo {
    /// Dependency package name
    pub package: String,

    /// Current version
    #[serde(rename = "currentVersion")]
    pub current_version: String,

    /// Latest available version
    #[serde(rename = "latestVersion")]
    pub latest_version: String,

    /// Upgrade type (major, minor, patch)
    #[serde(rename = "type")]
    pub upgrade_type: String,

    /// Whether this upgrade includes breaking changes
    pub breaking: bool,
}

impl DependencyUpgradeInfo {
    /// Returns true if this is a major version upgrade.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::DependencyUpgradeInfo;
    ///
    /// let upgrade = DependencyUpgradeInfo {
    ///     package: "eslint".to_string(),
    ///     current_version: "8.0.0".to_string(),
    ///     latest_version: "9.0.0".to_string(),
    ///     upgrade_type: "major".to_string(),
    ///     breaking: true,
    /// };
    ///
    /// assert!(upgrade.is_major());
    /// ```
    pub fn is_major(&self) -> bool {
        self.upgrade_type == "major"
    }

    /// Returns true if this is a minor version upgrade.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::DependencyUpgradeInfo;
    ///
    /// let upgrade = DependencyUpgradeInfo {
    ///     package: "typescript".to_string(),
    ///     current_version: "5.0.0".to_string(),
    ///     latest_version: "5.3.3".to_string(),
    ///     upgrade_type: "minor".to_string(),
    ///     breaking: false,
    /// };
    ///
    /// assert!(upgrade.is_minor());
    /// ```
    pub fn is_minor(&self) -> bool {
        self.upgrade_type == "minor"
    }

    /// Returns true if this is a patch version upgrade.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::upgrade::types::DependencyUpgradeInfo;
    ///
    /// let upgrade = DependencyUpgradeInfo {
    ///     package: "vitest".to_string(),
    ///     current_version: "1.0.0".to_string(),
    ///     latest_version: "1.0.1".to_string(),
    ///     upgrade_type: "patch".to_string(),
    ///     breaking: false,
    /// };
    ///
    /// assert!(upgrade.is_patch());
    /// ```
    pub fn is_patch(&self) -> bool {
        self.upgrade_type == "patch"
    }
}
