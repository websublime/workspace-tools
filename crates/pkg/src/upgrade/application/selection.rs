//! Upgrade selection and filtering criteria.
//!
//! **What**: Provides types for specifying which upgrades to apply through
//! fine-grained filtering by upgrade type, package names, or dependency names.
//!
//! **How**: The `UpgradeSelection` struct defines multiple filtering criteria
//! that can be combined. Helper methods provide convenient presets for common
//! scenarios like patch-only or minor-and-patch upgrades.
//!
//! **Why**: To give users precise control over which dependency upgrades to apply,
//! enabling safe, incremental upgrade strategies that minimize risk of breaking changes.

use crate::upgrade::registry::UpgradeType;
use serde::{Deserialize, Serialize};

/// Selection criteria for applying upgrades.
///
/// Allows filtering upgrades by type, package, or specific dependencies.
/// Multiple filters can be combined for fine-grained control.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::upgrade::UpgradeSelection;
///
/// // Select all upgrades
/// let selection = UpgradeSelection::all();
///
/// // Select only patch upgrades
/// let selection = UpgradeSelection::patch_only();
///
/// // Select patch and minor upgrades (exclude major)
/// let selection = UpgradeSelection::minor_and_patch();
///
/// // Select upgrades for specific packages
/// let selection = UpgradeSelection::packages(vec![
///     "my-package".to_string(),
///     "another-package".to_string(),
/// ]);
///
/// // Select specific dependencies across all packages
/// let selection = UpgradeSelection::dependencies(vec![
///     "react".to_string(),
///     "lodash".to_string(),
/// ]);
///
/// // Combine filters
/// let mut selection = UpgradeSelection::patch_only();
/// selection.packages = Some(vec!["my-package".to_string()]);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpgradeSelection {
    /// Apply all available upgrades.
    ///
    /// When `true`, all detected upgrades will be applied regardless of type.
    /// This takes precedence over other type-based filters.
    ///
    /// # Default: `false`
    pub all: bool,

    /// Apply only patch upgrades.
    ///
    /// When `true`, only upgrades that increment the patch version
    /// (e.g., 1.2.3 → 1.2.4) will be applied.
    ///
    /// # Default: `false`
    pub patch_only: bool,

    /// Apply patch and minor upgrades.
    ///
    /// When `true`, upgrades that increment the patch or minor version
    /// (e.g., 1.2.3 → 1.2.4 or 1.2.3 → 1.3.0) will be applied.
    /// Major version upgrades will be excluded.
    ///
    /// # Default: `false`
    pub minor_and_patch: bool,

    /// Specific packages to upgrade (filter by package name).
    ///
    /// When set, only package.json files matching these package names
    /// (from the `name` field) will have upgrades applied.
    ///
    /// # Default: `None` (all packages)
    pub packages: Option<Vec<String>>,

    /// Specific dependencies to upgrade (filter by dependency name).
    ///
    /// When set, only these dependencies will be upgraded across all packages.
    /// This is useful for upgrading a single dependency across a monorepo.
    ///
    /// # Default: `None` (all dependencies)
    pub dependencies: Option<Vec<String>>,

    /// Maximum upgrade type to apply.
    ///
    /// Provides an alternative way to limit upgrade scope by specifying
    /// the maximum upgrade type allowed. For example, `Some(UpgradeType::Minor)`
    /// allows patch and minor upgrades but blocks major upgrades.
    ///
    /// # Default: `None` (no limit)
    pub max_upgrade_type: Option<UpgradeType>,
}

impl UpgradeSelection {
    /// Creates a selection that applies all available upgrades.
    ///
    /// No filtering is applied - all detected upgrades will be applied
    /// regardless of type, package, or dependency name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::all();
    /// assert!(selection.all);
    /// assert!(!selection.patch_only);
    /// assert!(!selection.minor_and_patch);
    /// ```
    #[must_use]
    pub fn all() -> Self {
        Self { all: true, ..Default::default() }
    }

    /// Creates a selection that applies only patch upgrades.
    ///
    /// Only upgrades that increment the patch version (e.g., 1.2.3 → 1.2.4)
    /// will be applied. Minor and major upgrades are excluded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::patch_only();
    /// assert!(selection.patch_only);
    /// assert!(!selection.all);
    /// ```
    #[must_use]
    pub fn patch_only() -> Self {
        Self { patch_only: true, ..Default::default() }
    }

    /// Creates a selection that applies patch and minor upgrades.
    ///
    /// Upgrades that increment the patch or minor version
    /// (e.g., 1.2.3 → 1.2.4 or 1.2.3 → 1.3.0) will be applied.
    /// Major version upgrades (e.g., 1.2.3 → 2.0.0) are excluded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::minor_and_patch();
    /// assert!(selection.minor_and_patch);
    /// assert!(!selection.patch_only);
    /// ```
    #[must_use]
    pub fn minor_and_patch() -> Self {
        Self { minor_and_patch: true, ..Default::default() }
    }

    /// Creates a selection for specific packages by name.
    ///
    /// Only upgrades in package.json files matching the specified package names
    /// will be applied. The package name is read from the `name` field in package.json.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::packages(vec![
    ///     "my-package".to_string(),
    ///     "another-package".to_string(),
    /// ]);
    ///
    /// assert!(selection.packages.is_some());
    /// assert_eq!(selection.packages.as_ref().unwrap().len(), 2);
    /// ```
    #[must_use]
    pub fn packages(packages: Vec<String>) -> Self {
        Self { packages: Some(packages), ..Default::default() }
    }

    /// Creates a selection for specific dependencies by name.
    ///
    /// Only the specified dependencies will be upgraded across all packages.
    /// This is useful for upgrading a single dependency throughout a monorepo.
    ///
    /// # Arguments
    ///
    /// * `deps` - List of dependency names to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::dependencies(vec![
    ///     "react".to_string(),
    ///     "lodash".to_string(),
    /// ]);
    ///
    /// assert!(selection.dependencies.is_some());
    /// assert_eq!(selection.dependencies.as_ref().unwrap().len(), 2);
    /// ```
    #[must_use]
    pub fn dependencies(deps: Vec<String>) -> Self {
        Self { dependencies: Some(deps), ..Default::default() }
    }

    /// Checks if this selection matches the given upgrade type.
    ///
    /// Returns `true` if the upgrade type should be included based on
    /// the selection criteria.
    ///
    /// # Arguments
    ///
    /// * `upgrade_type` - The type of upgrade to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::{UpgradeSelection, UpgradeType};
    ///
    /// let selection = UpgradeSelection::patch_only();
    /// assert!(selection.matches_type(UpgradeType::Patch));
    /// assert!(!selection.matches_type(UpgradeType::Minor));
    /// assert!(!selection.matches_type(UpgradeType::Major));
    ///
    /// let selection = UpgradeSelection::minor_and_patch();
    /// assert!(selection.matches_type(UpgradeType::Patch));
    /// assert!(selection.matches_type(UpgradeType::Minor));
    /// assert!(!selection.matches_type(UpgradeType::Major));
    /// ```
    #[must_use]
    pub fn matches_type(&self, upgrade_type: UpgradeType) -> bool {
        if self.all {
            return true;
        }

        if self.patch_only {
            return upgrade_type == UpgradeType::Patch;
        }

        if self.minor_and_patch {
            return matches!(upgrade_type, UpgradeType::Patch | UpgradeType::Minor);
        }

        if let Some(max_type) = self.max_upgrade_type {
            return match max_type {
                UpgradeType::Patch => upgrade_type == UpgradeType::Patch,
                UpgradeType::Minor => {
                    matches!(upgrade_type, UpgradeType::Patch | UpgradeType::Minor)
                }
                UpgradeType::Major => true,
            };
        }

        // No type filter specified - allow all types
        true
    }

    /// Checks if this selection matches the given package name.
    ///
    /// Returns `true` if the package should be included based on
    /// the package filter, or if no package filter is specified.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The package name to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::packages(vec!["my-package".to_string()]);
    /// assert!(selection.matches_package("my-package"));
    /// assert!(!selection.matches_package("other-package"));
    ///
    /// let selection = UpgradeSelection::all();
    /// assert!(selection.matches_package("any-package"));
    /// ```
    #[must_use]
    pub fn matches_package(&self, package_name: &str) -> bool {
        if let Some(ref packages) = self.packages {
            packages.iter().any(|p| p == package_name)
        } else {
            true
        }
    }

    /// Checks if this selection matches the given dependency name.
    ///
    /// Returns `true` if the dependency should be included based on
    /// the dependency filter, or if no dependency filter is specified.
    ///
    /// # Arguments
    ///
    /// * `dependency_name` - The dependency name to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::dependencies(vec!["react".to_string()]);
    /// assert!(selection.matches_dependency("react"));
    /// assert!(!selection.matches_dependency("lodash"));
    ///
    /// let selection = UpgradeSelection::all();
    /// assert!(selection.matches_dependency("any-dependency"));
    /// ```
    #[must_use]
    pub fn matches_dependency(&self, dependency_name: &str) -> bool {
        if let Some(ref dependencies) = self.dependencies {
            dependencies.iter().any(|d| d == dependency_name)
        } else {
            true
        }
    }

    /// Checks if this selection has any active filters.
    ///
    /// Returns `true` if at least one filter criterion is specified,
    /// `false` if this is a default/empty selection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::upgrade::UpgradeSelection;
    ///
    /// let selection = UpgradeSelection::default();
    /// assert!(!selection.has_filters());
    ///
    /// let selection = UpgradeSelection::patch_only();
    /// assert!(selection.has_filters());
    /// ```
    #[must_use]
    pub fn has_filters(&self) -> bool {
        self.all
            || self.patch_only
            || self.minor_and_patch
            || self.packages.is_some()
            || self.dependencies.is_some()
            || self.max_upgrade_type.is_some()
    }
}
