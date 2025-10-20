//! Version application types and results.
//!
//! **What**: Defines the data structures used when applying version changes to package.json
//! files, including results, summaries, and application status information.
//!
//! **How**: Provides `ApplyResult` containing the resolution details, modified files list,
//! and summary statistics. Supports both dry-run mode (preview only) and actual application
//! mode (writes to filesystem).
//!
//! **Why**: To provide clear, structured information about version application operations,
//! enabling users to preview changes before applying them and understand what was modified
//! after application.
//!
//! # Examples
//!
//! ## Using ApplyResult in dry-run mode
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::VersionResolver;
//! use sublime_pkg_tools::types::Changeset;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let resolver: VersionResolver = todo!();
//! # let changeset: Changeset = todo!();
//! // Preview changes without modifying files
//! let result = resolver.apply_versions(&changeset, true).await?;
//!
//! assert!(result.dry_run);
//! assert!(result.modified_files.is_empty());
//!
//! println!("Would update {} packages:", result.summary.packages_updated);
//! println!("  Direct updates: {}", result.summary.direct_updates);
//! println!("  Propagated updates: {}", result.summary.propagated_updates);
//! println!("  Dependency updates: {}", result.summary.dependency_updates);
//! # Ok(())
//! # }
//! ```
//!
//! ## Applying changes to filesystem
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::VersionResolver;
//! use sublime_pkg_tools::types::Changeset;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let resolver: VersionResolver = todo!();
//! # let changeset: Changeset = todo!();
//! // Apply changes to package.json files
//! let result = resolver.apply_versions(&changeset, false).await?;
//!
//! assert!(!result.dry_run);
//! println!("Modified {} files:", result.modified_files.len());
//! for file in &result.modified_files {
//!     println!("  - {}", file.display());
//! }
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

use crate::version::resolution::VersionResolution;

/// Result of applying version changes to package.json files.
///
/// This type contains all information about a version application operation,
/// including whether it was a dry-run, the resolution details, modified files,
/// and a summary of changes.
///
/// # Fields
///
/// * `dry_run` - Whether this was a preview-only operation (no files modified)
/// * `resolution` - The version resolution details including all updates
/// * `modified_files` - List of package.json files that were modified (empty if dry_run)
/// * `summary` - Statistical summary of the application
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::VersionResolver;
/// use sublime_pkg_tools::types::Changeset;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let resolver: VersionResolver = todo!();
/// # let changeset: Changeset = todo!();
/// let result = resolver.apply_versions(&changeset, false).await?;
///
/// if result.dry_run {
///     println!("This was a preview - no files were modified");
/// } else {
///     println!("Modified {} package.json files", result.modified_files.len());
/// }
///
/// println!("Total packages updated: {}", result.summary.packages_updated);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ApplyResult {
    /// Whether this was a dry-run (preview only, no files modified).
    pub dry_run: bool,

    /// Version resolution details including all package updates.
    pub resolution: VersionResolution,

    /// List of package.json files that were modified.
    ///
    /// This will be empty if `dry_run` is true. When `dry_run` is false,
    /// this contains the absolute paths to all package.json files that
    /// were successfully updated.
    pub modified_files: Vec<PathBuf>,

    /// Summary of changes applied or previewed.
    pub summary: ApplySummary,
}

impl ApplyResult {
    /// Creates a new `ApplyResult`.
    ///
    /// # Arguments
    ///
    /// * `dry_run` - Whether this was a preview-only operation
    /// * `resolution` - The version resolution details
    /// * `modified_files` - List of modified package.json files (empty for dry-run)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplyResult;
    /// use sublime_pkg_tools::version::resolution::VersionResolution;
    ///
    /// let resolution = VersionResolution::default();
    /// let result = ApplyResult::new(true, resolution, vec![]);
    ///
    /// assert!(result.dry_run);
    /// assert!(result.modified_files.is_empty());
    /// ```
    #[must_use]
    pub fn new(dry_run: bool, resolution: VersionResolution, modified_files: Vec<PathBuf>) -> Self {
        let summary = ApplySummary::from_resolution(&resolution, modified_files.len());

        Self { dry_run, resolution, modified_files, summary }
    }

    /// Returns whether any packages were updated.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplyResult;
    /// use sublime_pkg_tools::version::resolution::VersionResolution;
    ///
    /// let resolution = VersionResolution::default();
    /// let result = ApplyResult::new(true, resolution, vec![]);
    ///
    /// if result.has_updates() {
    ///     println!("Packages were updated");
    /// }
    /// ```
    #[must_use]
    pub fn has_updates(&self) -> bool {
        self.summary.packages_updated > 0
    }

    /// Returns whether any files were modified.
    ///
    /// This will always be false if `dry_run` is true.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplyResult;
    /// use std::path::PathBuf;
    ///
    /// # let result: ApplyResult = todo!();
    /// if result.has_modified_files() {
    ///     println!("Modified files:");
    ///     for file in &result.modified_files {
    ///         println!("  - {}", file.display());
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn has_modified_files(&self) -> bool {
        !self.modified_files.is_empty()
    }

    /// Returns the number of packages that were updated.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplyResult;
    ///
    /// # let result: ApplyResult = todo!();
    /// println!("Updated {} packages", result.update_count());
    /// ```
    #[must_use]
    pub fn update_count(&self) -> usize {
        self.summary.packages_updated
    }

    /// Returns whether circular dependencies were detected.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplyResult;
    ///
    /// # let result: ApplyResult = todo!();
    /// if result.has_circular_dependencies() {
    ///     eprintln!("Warning: Circular dependencies detected!");
    ///     eprintln!("Count: {}", result.summary.circular_dependencies);
    /// }
    /// ```
    #[must_use]
    pub fn has_circular_dependencies(&self) -> bool {
        self.summary.circular_dependencies > 0
    }
}

/// Summary statistics for version application.
///
/// This type provides aggregate statistics about the version application
/// operation, including counts of different types of updates.
///
/// # Fields
///
/// * `packages_updated` - Total number of packages that had version changes
/// * `direct_updates` - Number of packages updated directly from changeset
/// * `propagated_updates` - Number of packages updated via dependency propagation
/// * `dependency_updates` - Total number of dependency reference updates
/// * `circular_dependencies` - Number of circular dependencies detected
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::application::ApplySummary;
/// use sublime_pkg_tools::version::resolution::VersionResolution;
///
/// let resolution = VersionResolution::default();
/// let summary = ApplySummary::from_resolution(&resolution, 3);
///
/// println!("Packages updated: {}", summary.packages_updated);
/// println!("Direct: {}, Propagated: {}",
///     summary.direct_updates,
///     summary.propagated_updates
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplySummary {
    /// Total number of packages that had version changes.
    pub packages_updated: usize,

    /// Number of packages updated directly from the changeset.
    ///
    /// These are packages explicitly listed in the changeset that
    /// receive the bump specified in the changeset.
    pub direct_updates: usize,

    /// Number of packages updated via dependency propagation.
    ///
    /// These are packages that were not in the changeset but had their
    /// versions bumped because they depend on packages that were updated.
    pub propagated_updates: usize,

    /// Total number of dependency reference updates across all packages.
    ///
    /// This counts each individual dependency spec update in package.json
    /// files (dependencies, devDependencies, peerDependencies).
    pub dependency_updates: usize,

    /// Number of circular dependencies detected during resolution.
    ///
    /// Circular dependencies are reported but do not prevent version
    /// resolution or application.
    pub circular_dependencies: usize,
}

impl ApplySummary {
    /// Creates a new `ApplySummary` from a `VersionResolution`.
    ///
    /// This constructor calculates all summary statistics from the
    /// resolution details.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The version resolution containing update details
    /// * `files_modified` - Number of package.json files modified
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    /// use sublime_pkg_tools::version::resolution::VersionResolution;
    ///
    /// let resolution = VersionResolution::default();
    /// let summary = ApplySummary::from_resolution(&resolution, 5);
    ///
    /// assert_eq!(summary.packages_updated, 0); // No updates in default
    /// ```
    #[must_use]
    pub fn from_resolution(resolution: &VersionResolution, _files_modified: usize) -> Self {
        let packages_updated = resolution.updates.len();

        let direct_updates = resolution.updates.iter().filter(|u| u.is_direct_change()).count();

        let propagated_updates = resolution.updates.iter().filter(|u| u.is_propagated()).count();

        let dependency_updates =
            resolution.updates.iter().map(|u| u.dependency_updates.len()).sum();

        let circular_dependencies = resolution.circular_dependencies.len();

        Self {
            packages_updated,
            direct_updates,
            propagated_updates,
            dependency_updates,
            circular_dependencies,
        }
    }

    /// Returns whether any packages were updated.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    ///
    /// # let summary: ApplySummary = todo!();
    /// if summary.has_updates() {
    ///     println!("Changes were made");
    /// } else {
    ///     println!("No changes needed");
    /// }
    /// ```
    #[must_use]
    pub fn has_updates(&self) -> bool {
        self.packages_updated > 0
    }

    /// Returns whether any packages were updated via propagation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    ///
    /// # let summary: ApplySummary = todo!();
    /// if summary.has_propagated_updates() {
    ///     println!("Dependency propagation occurred");
    /// }
    /// ```
    #[must_use]
    pub fn has_propagated_updates(&self) -> bool {
        self.propagated_updates > 0
    }

    /// Returns whether any dependency references were updated.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    ///
    /// # let summary: ApplySummary = todo!();
    /// if summary.has_dependency_updates() {
    ///     println!("{} dependency references updated", summary.dependency_updates);
    /// }
    /// ```
    #[must_use]
    pub fn has_dependency_updates(&self) -> bool {
        self.dependency_updates > 0
    }

    /// Returns whether circular dependencies were detected.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    ///
    /// # let summary: ApplySummary = todo!();
    /// if summary.has_circular_dependencies() {
    ///     eprintln!("Warning: {} circular dependencies found",
    ///         summary.circular_dependencies);
    /// }
    /// ```
    #[must_use]
    pub fn has_circular_dependencies(&self) -> bool {
        self.circular_dependencies > 0
    }
}

impl Default for ApplySummary {
    /// Creates a default `ApplySummary` with all counts set to zero.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::application::ApplySummary;
    ///
    /// let summary = ApplySummary::default();
    /// assert_eq!(summary.packages_updated, 0);
    /// assert_eq!(summary.direct_updates, 0);
    /// assert_eq!(summary.propagated_updates, 0);
    /// ```
    fn default() -> Self {
        Self {
            packages_updated: 0,
            direct_updates: 0,
            propagated_updates: 0,
            dependency_updates: 0,
            circular_dependencies: 0,
        }
    }
}
