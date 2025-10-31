//! Bump snapshot and report structures.
//!
//! This module defines the data structures used to represent version bump previews
//! and results. These structures are used for both human-readable table output and
//! JSON serialization.
//!
//! # What
//!
//! Provides:
//! - `BumpSnapshot` - Complete bump preview with all package updates
//! - `PackageBumpInfo` - Information about a single package's version bump
//! - `BumpSummary` - Summary statistics about the bump operation
//! - JSON serialization support for automation and CI/CD
//!
//! # How
//!
//! These structures are populated by:
//! 1. Loading active changesets
//! 2. Using `VersionResolver` to calculate version bumps
//! 3. Determining which packages will bump based on versioning strategy
//! 4. Collecting all information into the snapshot structure
//!
//! The snapshot can then be:
//! - Displayed as a formatted table (human mode)
//! - Serialized to JSON (automation mode)
//!
//! # Why
//!
//! Separating bump data structures provides:
//! - Clear separation between data and presentation
//! - Consistent structure for both preview and execute modes
//! - Easy JSON serialization for CI/CD integration
//! - Type-safe representation of bump operations
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::commands::bump::snapshot::{
//!     BumpSnapshot, PackageBumpInfo, BumpSummary
//! };
//! use sublime_pkg_tools::types::VersionBump;
//!
//! let package_info = PackageBumpInfo {
//!     name: "@org/core".to_string(),
//!     path: "packages/core".to_string(),
//!     current_version: "1.2.3".to_string(),
//!     next_version: "1.3.0".to_string(),
//!     bump_type: VersionBump::Minor,
//!     will_bump: true,
//!     reason: "direct change from changeset".to_string(),
//! };
//! ```

use serde::Serialize;
use sublime_pkg_tools::types::VersionBump;

/// Complete snapshot of a version bump operation.
///
/// Contains all information about packages that will be bumped, changesets
/// being processed, and summary statistics. Used for both preview (dry-run)
/// and execute modes.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::BumpSnapshot;
///
/// let snapshot = BumpSnapshot {
///     strategy: "independent".to_string(),
///     packages: vec![],
///     changesets: vec![],
///     summary: Default::default(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpSnapshot {
    /// Versioning strategy being used (Independent or Unified).
    pub strategy: String,

    /// List of all workspace packages with their bump information.
    ///
    /// Includes both packages that will bump and those that won't.
    /// Check `will_bump` field to distinguish.
    pub packages: Vec<PackageBumpInfo>,

    /// List of changesets being processed in this bump.
    pub changesets: Vec<ChangesetInfo>,

    /// Summary statistics for the bump operation.
    pub summary: BumpSummary,
}

/// Information about a single package's version bump.
///
/// Contains current and next versions, bump type, and whether the package
/// will actually be bumped based on the versioning strategy and changesets.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::PackageBumpInfo;
/// use sublime_pkg_tools::types::VersionBump;
///
/// let info = PackageBumpInfo {
///     name: "@org/core".to_string(),
///     path: "packages/core".to_string(),
///     current_version: "1.2.3".to_string(),
///     next_version: "1.3.0".to_string(),
///     bump_type: VersionBump::Minor,
///     will_bump: true,
///     reason: "direct change from changeset".to_string(),
/// };
///
/// // Example: Package with dependency update but not bumping (Independent mode)
/// let info2 = PackageBumpInfo {
///     name: "@org/utils".to_string(),
///     path: "packages/utils".to_string(),
///     current_version: "2.0.0".to_string(),
///     next_version: "2.0.0".to_string(),
///     bump_type: VersionBump::None,
///     will_bump: false,
///     reason: "dependency updated (not bumping in independent mode)".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageBumpInfo {
    /// Package name (e.g., "@org/core").
    pub name: String,

    /// Relative path to package directory.
    pub path: String,

    /// Current version (e.g., "1.2.3").
    pub current_version: String,

    /// Next version after bump (e.g., "1.3.0").
    ///
    /// If `will_bump` is false, this is the same as `current_version`.
    pub next_version: String,

    /// Type of version bump.
    ///
    /// None if the package won't be bumped.
    pub bump_type: VersionBump,

    /// Whether this package will actually be bumped.
    ///
    /// - Independent strategy: true only if package is in a changeset
    /// - Unified strategy: true for all packages if any changeset exists
    pub will_bump: bool,

    /// Human-readable reason for bump or no-bump.
    ///
    /// Examples:
    /// - **Independent strategy**:
    ///   - "direct change from changeset" - Package is in a changeset and will bump
    ///   - "in changeset but version unchanged" - Package in changeset but no version change needed
    ///   - "dependency updated (not bumping in independent mode)" - Dependency changed but package not in changeset
    ///   - "not in any changeset" - Package not affected by any changeset
    /// - **Unified strategy**:
    ///   - "unified bump (package in changeset)" - Package is in a changeset causing unified bump
    ///   - "unified bump (all packages bumped together)" - Package bumped due to unified strategy
    pub reason: String,
}

/// Information about a changeset being processed.
///
/// Summarizes a changeset's key details for display and JSON output.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::ChangesetInfo;
/// use sublime_pkg_tools::types::VersionBump;
///
/// let info = ChangesetInfo {
///     id: "feature-new-api".to_string(),
///     branch: "feature/new-api".to_string(),
///     bump_type: VersionBump::Minor,
///     packages: vec!["@org/core".to_string()],
///     commit_count: 5,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetInfo {
    /// Changeset ID.
    pub id: String,

    /// Git branch name.
    pub branch: String,

    /// Bump type for this changeset.
    pub bump_type: VersionBump,

    /// List of packages affected by this changeset.
    pub packages: Vec<String>,

    /// Number of commits in this changeset.
    pub commit_count: usize,
}

/// Summary statistics for a bump operation.
///
/// Provides high-level counts and information about the bump.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::bump::snapshot::BumpSummary;
///
/// let summary = BumpSummary {
///     total_packages: 10,
///     packages_to_bump: 3,
///     packages_unchanged: 7,
///     total_changesets: 2,
///     has_circular_dependencies: false,
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumpSummary {
    /// Total number of packages in workspace.
    pub total_packages: usize,

    /// Number of packages that will be bumped.
    pub packages_to_bump: usize,

    /// Number of packages that won't be bumped.
    pub packages_unchanged: usize,

    /// Total number of changesets being processed.
    pub total_changesets: usize,

    /// Whether circular dependencies were detected.
    pub has_circular_dependencies: bool,
}

impl BumpSummary {
    /// Creates a new BumpSummary from package and changeset counts.
    ///
    /// # Arguments
    ///
    /// * `total_packages` - Total number of packages in workspace
    /// * `packages_to_bump` - Number of packages that will be bumped
    /// * `total_changesets` - Number of changesets being processed
    /// * `has_circular_dependencies` - Whether circular dependencies exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::commands::bump::snapshot::BumpSummary;
    ///
    /// let summary = BumpSummary::new(10, 3, 2, false);
    /// assert_eq!(summary.packages_unchanged, 7);
    /// ```
    #[must_use]
    pub const fn new(
        total_packages: usize,
        packages_to_bump: usize,
        total_changesets: usize,
        has_circular_dependencies: bool,
    ) -> Self {
        Self {
            total_packages,
            packages_to_bump,
            packages_unchanged: total_packages.saturating_sub(packages_to_bump),
            total_changesets,
            has_circular_dependencies,
        }
    }
}
