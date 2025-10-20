//! Statistics and summary types for changes analysis.
//!
//! **What**: Provides data structures for aggregating and summarizing changes across packages,
//! including file counts, line statistics, and commit counts.
//!
//! **How**: Defines serializable structures that accumulate statistics from file changes and
//! commits, providing both per-package and overall workspace-level summaries.
//!
//! **Why**: To provide clear, actionable metrics about the scope and impact of changes,
//! supporting decision-making for version bumps, changelog generation, and release planning.
//!
//! # Examples
//!
//! ## Creating package statistics
//!
//! ```rust
//! use sublime_pkg_tools::changes::PackageChangeStats;
//!
//! let stats = PackageChangeStats {
//!     files_changed: 5,
//!     files_added: 2,
//!     files_modified: 3,
//!     files_deleted: 0,
//!     commits: 3,
//!     lines_added: 150,
//!     lines_deleted: 30,
//! };
//!
//! assert_eq!(stats.net_lines_changed(), 120);
//! assert_eq!(stats.total_lines_changed(), 180);
//! ```
//!
//! ## Creating a summary
//!
//! ```rust
//! use sublime_pkg_tools::changes::ChangesSummary;
//!
//! let summary = ChangesSummary {
//!     total_packages: 10,
//!     packages_with_changes: 3,
//!     packages_without_changes: 7,
//!     total_files_changed: 15,
//!     total_commits: 5,
//!     total_lines_added: 250,
//!     total_lines_deleted: 50,
//! };
//!
//! assert_eq!(summary.change_percentage(), 30.0);
//! assert_eq!(summary.net_lines_changed(), 200);
//! ```

use serde::{Deserialize, Serialize};

/// Summary statistics for changes across all packages.
///
/// Provides aggregate statistics for an entire workspace analysis, including
/// package counts, file counts, commit counts, and line statistics.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changes::ChangesSummary;
///
/// let summary = ChangesSummary {
///     total_packages: 5,
///     packages_with_changes: 2,
///     packages_without_changes: 3,
///     total_files_changed: 10,
///     total_commits: 4,
///     total_lines_added: 100,
///     total_lines_deleted: 20,
/// };
///
/// assert_eq!(summary.change_percentage(), 40.0);
/// assert!(summary.has_changes());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangesSummary {
    /// Total number of packages analyzed.
    pub total_packages: usize,

    /// Number of packages with changes.
    pub packages_with_changes: usize,

    /// Number of packages without changes.
    pub packages_without_changes: usize,

    /// Total number of files changed across all packages.
    pub total_files_changed: usize,

    /// Total number of commits analyzed.
    pub total_commits: usize,

    /// Total lines added across all packages.
    pub total_lines_added: usize,

    /// Total lines deleted across all packages.
    pub total_lines_deleted: usize,
}

impl ChangesSummary {
    /// Creates a new empty `ChangesSummary`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let summary = ChangesSummary::new();
    /// assert_eq!(summary.total_packages, 0);
    /// assert!(!summary.has_changes());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_packages: 0,
            packages_with_changes: 0,
            packages_without_changes: 0,
            total_files_changed: 0,
            total_commits: 0,
            total_lines_added: 0,
            total_lines_deleted: 0,
        }
    }

    /// Returns whether any changes were detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let mut summary = ChangesSummary::new();
    /// assert!(!summary.has_changes());
    ///
    /// summary.packages_with_changes = 1;
    /// assert!(summary.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.packages_with_changes > 0 || self.total_files_changed > 0
    }

    /// Returns the percentage of packages that have changes.
    ///
    /// Returns 0.0 if there are no packages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let summary = ChangesSummary {
    ///     total_packages: 10,
    ///     packages_with_changes: 3,
    ///     packages_without_changes: 7,
    ///     total_files_changed: 15,
    ///     total_commits: 5,
    ///     total_lines_added: 100,
    ///     total_lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(summary.change_percentage(), 30.0);
    /// ```
    #[must_use]
    pub fn change_percentage(&self) -> f64 {
        if self.total_packages == 0 {
            0.0
        } else {
            (self.packages_with_changes as f64 / self.total_packages as f64) * 100.0
        }
    }

    /// Calculates the net line change (added - deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let summary = ChangesSummary {
    ///     total_packages: 5,
    ///     packages_with_changes: 2,
    ///     packages_without_changes: 3,
    ///     total_files_changed: 10,
    ///     total_commits: 4,
    ///     total_lines_added: 100,
    ///     total_lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(summary.net_lines_changed(), 80);
    /// ```
    #[must_use]
    pub fn net_lines_changed(&self) -> i64 {
        self.total_lines_added as i64 - self.total_lines_deleted as i64
    }

    /// Returns the total number of line changes (added + deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let summary = ChangesSummary {
    ///     total_packages: 5,
    ///     packages_with_changes: 2,
    ///     packages_without_changes: 3,
    ///     total_files_changed: 10,
    ///     total_commits: 4,
    ///     total_lines_added: 100,
    ///     total_lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(summary.total_lines_changed(), 120);
    /// ```
    #[must_use]
    pub fn total_lines_changed(&self) -> usize {
        self.total_lines_added + self.total_lines_deleted
    }

    /// Returns the average files changed per package with changes.
    ///
    /// Returns 0.0 if there are no packages with changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesSummary;
    ///
    /// let summary = ChangesSummary {
    ///     total_packages: 5,
    ///     packages_with_changes: 2,
    ///     packages_without_changes: 3,
    ///     total_files_changed: 10,
    ///     total_commits: 4,
    ///     total_lines_added: 100,
    ///     total_lines_deleted: 20,
    /// };
    ///
    /// assert_eq!(summary.average_files_per_package(), 5.0);
    /// ```
    #[must_use]
    pub fn average_files_per_package(&self) -> f64 {
        if self.packages_with_changes == 0 {
            0.0
        } else {
            self.total_files_changed as f64 / self.packages_with_changes as f64
        }
    }
}

impl Default for ChangesSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for changes in a single package.
///
/// Provides detailed metrics about file changes, commits, and line statistics
/// for an individual package.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changes::PackageChangeStats;
///
/// let stats = PackageChangeStats {
///     files_changed: 5,
///     files_added: 2,
///     files_modified: 3,
///     files_deleted: 0,
///     commits: 3,
///     lines_added: 150,
///     lines_deleted: 30,
/// };
///
/// assert_eq!(stats.net_lines_changed(), 120);
/// assert!(stats.has_changes());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageChangeStats {
    /// Total number of files changed in this package.
    pub files_changed: usize,

    /// Number of files added.
    pub files_added: usize,

    /// Number of files modified.
    pub files_modified: usize,

    /// Number of files deleted.
    pub files_deleted: usize,

    /// Number of commits affecting this package.
    pub commits: usize,

    /// Total lines added in this package.
    pub lines_added: usize,

    /// Total lines deleted in this package.
    pub lines_deleted: usize,
}

impl PackageChangeStats {
    /// Creates a new empty `PackageChangeStats`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats::new();
    /// assert_eq!(stats.files_changed, 0);
    /// assert!(!stats.has_changes());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            files_changed: 0,
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            commits: 0,
            lines_added: 0,
            lines_deleted: 0,
        }
    }

    /// Returns whether this package has any changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let mut stats = PackageChangeStats::new();
    /// assert!(!stats.has_changes());
    ///
    /// stats.files_changed = 1;
    /// assert!(stats.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.files_changed > 0
    }

    /// Calculates the net line change (added - deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 3,
    ///     files_deleted: 0,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.net_lines_changed(), 120);
    /// ```
    #[must_use]
    pub fn net_lines_changed(&self) -> i64 {
        self.lines_added as i64 - self.lines_deleted as i64
    }

    /// Returns the total number of line changes (added + deleted).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 3,
    ///     files_deleted: 0,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.total_lines_changed(), 180);
    /// ```
    #[must_use]
    pub fn total_lines_changed(&self) -> usize {
        self.lines_added + self.lines_deleted
    }

    /// Returns the average lines changed per file.
    ///
    /// Returns 0.0 if there are no changed files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 3,
    ///     files_deleted: 0,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.average_lines_per_file(), 36.0);
    /// ```
    #[must_use]
    pub fn average_lines_per_file(&self) -> f64 {
        if self.files_changed == 0 {
            0.0
        } else {
            self.total_lines_changed() as f64 / self.files_changed as f64
        }
    }

    /// Returns the percentage of files that were added.
    ///
    /// Returns 0.0 if there are no changed files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 3,
    ///     files_deleted: 0,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.added_percentage(), 40.0);
    /// ```
    #[must_use]
    pub fn added_percentage(&self) -> f64 {
        if self.files_changed == 0 {
            0.0
        } else {
            (self.files_added as f64 / self.files_changed as f64) * 100.0
        }
    }

    /// Returns the percentage of files that were modified.
    ///
    /// Returns 0.0 if there are no changed files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 3,
    ///     files_deleted: 0,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.modified_percentage(), 60.0);
    /// ```
    #[must_use]
    pub fn modified_percentage(&self) -> f64 {
        if self.files_changed == 0 {
            0.0
        } else {
            (self.files_modified as f64 / self.files_changed as f64) * 100.0
        }
    }

    /// Returns the percentage of files that were deleted.
    ///
    /// Returns 0.0 if there are no changed files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::PackageChangeStats;
    ///
    /// let stats = PackageChangeStats {
    ///     files_changed: 5,
    ///     files_added: 2,
    ///     files_modified: 2,
    ///     files_deleted: 1,
    ///     commits: 3,
    ///     lines_added: 150,
    ///     lines_deleted: 30,
    /// };
    ///
    /// assert_eq!(stats.deleted_percentage(), 20.0);
    /// ```
    #[must_use]
    pub fn deleted_percentage(&self) -> f64 {
        if self.files_changed == 0 {
            0.0
        } else {
            (self.files_deleted as f64 / self.files_changed as f64) * 100.0
        }
    }
}

impl Default for PackageChangeStats {
    fn default() -> Self {
        Self::new()
    }
}
