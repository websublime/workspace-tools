//! Changes report data structures for aggregating package changes.
//!
//! **What**: Provides types for representing a complete analysis report of changes across
//! packages, including analysis metadata, affected packages, and summary statistics.
//!
//! **How**: Aggregates `PackageChanges` from multiple packages with analysis context
//! (mode, timestamp, git references) and provides filtering and querying capabilities.
//!
//! **Why**: To provide a comprehensive, queryable view of all changes in a workspace,
//! supporting version bumping, changelog generation, and release decision-making.
//!
//! # Examples
//!
//! ## Creating a changes report
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changes::{ChangesReport, AnalysisMode, PackageChanges};
//! use chrono::Utc;
//!
//! let report = ChangesReport {
//!     analyzed_at: Utc::now(),
//!     analysis_mode: AnalysisMode::WorkingDirectory,
//!     base_ref: None,
//!     head_ref: None,
//!     packages: vec![],
//!     summary: ChangesSummary::new(),
//!     is_monorepo: false,
//! };
//!
//! assert_eq!(report.analysis_mode, AnalysisMode::WorkingDirectory);
//! ```
//!
//! ## Filtering packages
//!
//! ```rust,ignore
//! # use sublime_pkg_tools::changes::ChangesReport;
//! let packages_with_changes = report.packages_with_changes();
//! let packages_without_changes = report.packages_without_changes();
//! ```

use crate::changes::{ChangesSummary, FileChangeType, PackageChanges};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Analysis mode indicating how changes were detected.
///
/// Different analysis modes provide different levels of information about changes.
/// For example, working directory analysis doesn't include commit information,
/// while commit range analysis includes full commit history.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changes::AnalysisMode;
///
/// let mode = AnalysisMode::WorkingDirectory;
/// assert!(mode.is_working_directory());
/// assert!(!mode.is_commit_range());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    /// Uncommitted changes (working tree + staging area).
    ///
    /// Analyzes files with changes that haven't been committed yet.
    /// This mode doesn't include commit information.
    WorkingDirectory,

    /// Changes between two commits or branches.
    ///
    /// Analyzes changes by comparing two git references (commits, tags, branches).
    /// Includes full commit history for the range.
    CommitRange,

    /// Changes in a single commit.
    ///
    /// Analyzes changes introduced by a specific commit.
    SingleCommit,

    /// Changes from a list of commits.
    ///
    /// Analyzes changes from a specific set of commits, typically from a changeset.
    CommitList,
}

impl AnalysisMode {
    /// Returns whether this mode analyzes the working directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::AnalysisMode;
    ///
    /// assert!(AnalysisMode::WorkingDirectory.is_working_directory());
    /// assert!(!AnalysisMode::CommitRange.is_working_directory());
    /// ```
    #[must_use]
    pub fn is_working_directory(&self) -> bool {
        matches!(self, Self::WorkingDirectory)
    }

    /// Returns whether this mode analyzes a commit range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::AnalysisMode;
    ///
    /// assert!(AnalysisMode::CommitRange.is_commit_range());
    /// assert!(!AnalysisMode::WorkingDirectory.is_commit_range());
    /// ```
    #[must_use]
    pub fn is_commit_range(&self) -> bool {
        matches!(self, Self::CommitRange)
    }

    /// Returns whether this mode analyzes a single commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::AnalysisMode;
    ///
    /// assert!(AnalysisMode::SingleCommit.is_single_commit());
    /// assert!(!AnalysisMode::CommitRange.is_single_commit());
    /// ```
    #[must_use]
    pub fn is_single_commit(&self) -> bool {
        matches!(self, Self::SingleCommit)
    }

    /// Returns whether this mode analyzes a commit list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::AnalysisMode;
    ///
    /// assert!(AnalysisMode::CommitList.is_commit_list());
    /// assert!(!AnalysisMode::WorkingDirectory.is_commit_list());
    /// ```
    #[must_use]
    pub fn is_commit_list(&self) -> bool {
        matches!(self, Self::CommitList)
    }

    /// Returns whether this mode includes commit information.
    ///
    /// Working directory analysis doesn't include commits, while all other modes do.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::AnalysisMode;
    ///
    /// assert!(!AnalysisMode::WorkingDirectory.has_commits());
    /// assert!(AnalysisMode::CommitRange.has_commits());
    /// assert!(AnalysisMode::SingleCommit.has_commits());
    /// ```
    #[must_use]
    pub fn has_commits(&self) -> bool {
        !matches!(self, Self::WorkingDirectory)
    }
}

/// Complete report of changes across packages.
///
/// Contains comprehensive information about all changes in a workspace, including
/// which packages were affected, what files changed, commit history, and statistics.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changes::{ChangesReport, AnalysisMode};
/// use chrono::Utc;
///
/// let report = ChangesReport::new(AnalysisMode::WorkingDirectory, false);
/// assert_eq!(report.analysis_mode, AnalysisMode::WorkingDirectory);
/// assert!(!report.is_monorepo);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesReport {
    /// Timestamp when the analysis was performed.
    pub analyzed_at: DateTime<Utc>,

    /// Mode of analysis (working directory, commit range, etc.).
    pub analysis_mode: AnalysisMode,

    /// Base git reference for commit range analysis.
    ///
    /// This is `None` for working directory analysis.
    /// For commit range analysis, this is typically "main" or a commit hash.
    pub base_ref: Option<String>,

    /// Head git reference for commit range analysis.
    ///
    /// This is `None` for working directory analysis.
    /// For commit range analysis, this is typically "HEAD" or a branch name.
    pub head_ref: Option<String>,

    /// All packages analyzed, including those with and without changes.
    pub packages: Vec<PackageChanges>,

    /// Summary statistics across all packages.
    pub summary: ChangesSummary,

    /// Whether this workspace is a monorepo.
    pub is_monorepo: bool,
}

impl ChangesReport {
    /// Creates a new `ChangesReport` with default values.
    ///
    /// # Arguments
    ///
    /// * `analysis_mode` - The mode of analysis
    /// * `is_monorepo` - Whether this is a monorepo workspace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{ChangesReport, AnalysisMode};
    ///
    /// let report = ChangesReport::new(AnalysisMode::WorkingDirectory, true);
    /// assert_eq!(report.analysis_mode, AnalysisMode::WorkingDirectory);
    /// assert!(report.is_monorepo);
    /// assert!(report.packages.is_empty());
    /// ```
    #[must_use]
    pub fn new(analysis_mode: AnalysisMode, is_monorepo: bool) -> Self {
        Self {
            analyzed_at: Utc::now(),
            analysis_mode,
            base_ref: None,
            head_ref: None,
            packages: Vec::new(),
            summary: ChangesSummary::new(),
            is_monorepo,
        }
    }

    /// Creates a new `ChangesReport` for commit range analysis.
    ///
    /// # Arguments
    ///
    /// * `base_ref` - Base git reference
    /// * `head_ref` - Head git reference
    /// * `is_monorepo` - Whether this is a monorepo workspace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::ChangesReport;
    ///
    /// let report = ChangesReport::new_for_range("main", "HEAD", false);
    /// assert_eq!(report.base_ref, Some("main".to_string()));
    /// assert_eq!(report.head_ref, Some("HEAD".to_string()));
    /// ```
    #[must_use]
    pub fn new_for_range(base_ref: &str, head_ref: &str, is_monorepo: bool) -> Self {
        Self {
            analyzed_at: Utc::now(),
            analysis_mode: AnalysisMode::CommitRange,
            base_ref: Some(base_ref.to_string()),
            head_ref: Some(head_ref.to_string()),
            packages: Vec::new(),
            summary: ChangesSummary::new(),
            is_monorepo,
        }
    }

    /// Returns a reference to a specific package by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to search for
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let package = report.get_package("@myorg/core");
    /// ```
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&PackageChanges> {
        self.packages.iter().find(|p| p.package_name() == name)
    }

    /// Returns all packages that have changes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let packages = report.packages_with_changes();
    /// for package in packages {
    ///     println!("Package {} has changes", package.package_name());
    /// }
    /// ```
    #[must_use]
    pub fn packages_with_changes(&self) -> Vec<&PackageChanges> {
        self.packages.iter().filter(|p| p.has_changes).collect()
    }

    /// Returns all packages without changes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let unchanged = report.packages_without_changes();
    /// ```
    #[must_use]
    pub fn packages_without_changes(&self) -> Vec<&PackageChanges> {
        self.packages.iter().filter(|p| !p.has_changes).collect()
    }

    /// Filters packages by file change type.
    ///
    /// Returns packages that have at least one file with the specified change type.
    ///
    /// # Arguments
    ///
    /// * `change_type` - The type of change to filter by
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::{ChangesReport, FileChangeType};
    /// let packages = report.filter_by_change_type(FileChangeType::Added);
    /// ```
    #[must_use]
    pub fn filter_by_change_type(&self, change_type: FileChangeType) -> Vec<&PackageChanges> {
        self.packages
            .iter()
            .filter(|p| p.files.iter().any(|f| f.change_type == change_type))
            .collect()
    }

    /// Returns packages where package.json was modified.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let packages = report.packages_with_package_json_changes();
    /// ```
    #[must_use]
    pub fn packages_with_package_json_changes(&self) -> Vec<&PackageChanges> {
        self.packages.iter().filter(|p| p.package_json_modified()).collect()
    }

    /// Returns whether any changes were detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changes::{ChangesReport, AnalysisMode};
    ///
    /// let report = ChangesReport::new(AnalysisMode::WorkingDirectory, false);
    /// assert!(!report.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.summary.has_changes()
    }

    /// Returns the total number of files changed across all packages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let count = report.total_files_changed();
    /// println!("Total files changed: {}", count);
    /// ```
    #[must_use]
    pub fn total_files_changed(&self) -> usize {
        self.summary.total_files_changed
    }

    /// Returns the total number of commits analyzed.
    ///
    /// This will be 0 for working directory analysis.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesReport;
    /// let count = report.total_commits();
    /// ```
    #[must_use]
    pub fn total_commits(&self) -> usize {
        self.summary.total_commits
    }

    /// Adds a package to the report.
    ///
    /// Updates the summary statistics automatically.
    ///
    /// # Arguments
    ///
    /// * `package` - The package changes to add
    pub fn add_package(&mut self, package: PackageChanges) {
        // Update summary
        self.summary.total_packages += 1;
        if package.has_changes {
            self.summary.packages_with_changes += 1;
        } else {
            self.summary.packages_without_changes += 1;
        }

        self.summary.total_files_changed += package.stats.files_changed;
        self.summary.total_lines_added += package.stats.lines_added;
        self.summary.total_lines_deleted += package.stats.lines_deleted;

        // Commits are counted uniquely across packages
        // For now, just add the package's commit count
        // (proper unique counting will be done in commit range analysis)
        self.summary.total_commits += package.stats.commits;

        self.packages.push(package);
    }

    /// Recalculates the summary statistics from current packages.
    ///
    /// This should be called after manually modifying packages.
    pub fn recalculate_summary(&mut self) {
        let mut summary = ChangesSummary::new();

        summary.total_packages = self.packages.len();

        for package in &self.packages {
            if package.has_changes {
                summary.packages_with_changes += 1;
            } else {
                summary.packages_without_changes += 1;
            }

            summary.total_files_changed += package.stats.files_changed;
            summary.total_lines_added += package.stats.lines_added;
            summary.total_lines_deleted += package.stats.lines_deleted;
            summary.total_commits += package.stats.commits;
        }

        self.summary = summary;
    }
}
