//! Changes analysis error types for package tools.
//!
//! **What**: Defines error types specific to changes analysis, file-to-package mapping,
//! commit range analysis, and working directory inspection operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! file paths, commit references, package names, and git integration details. Implements
//! `AsRef<str>` for string conversion.
//!
//! **Why**: To provide clear, actionable error messages for changes analysis operations,
//! enabling users to quickly identify and fix issues with file mapping, git operations,
//! and package detection.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{ChangesError, ChangesResult};
//! use std::path::PathBuf;
//!
//! fn analyze_file(path: &str) -> ChangesResult<String> {
//!     if path.is_empty() {
//!         return Err(ChangesError::InvalidPath {
//!             path: PathBuf::from(path),
//!             reason: "File path cannot be empty".to_string(),
//!         });
//!     }
//!     Ok("analysis-result".to_string())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for changes analysis operations.
///
/// This type alias simplifies error handling in changes analysis functions
/// by defaulting to `ChangesError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{ChangesError, ChangesResult};
///
/// fn analyze_changes() -> ChangesResult<Vec<String>> {
///     Ok(vec!["file1.js".to_string(), "file2.ts".to_string()])
/// }
/// ```
pub type ChangesResult<T> = Result<T, ChangesError>;

/// Errors that can occur during changes analysis operations.
///
/// This enum covers all possible error scenarios when analyzing file changes,
/// mapping files to packages, inspecting commit ranges, and detecting affected packages.
///
/// # Examples
///
/// ## Handling changes analysis errors
///
/// ```rust
/// use sublime_pkg_tools::error::ChangesError;
/// use std::path::PathBuf;
///
/// fn handle_changes_error(error: ChangesError) {
///     match error {
///         ChangesError::PackageNotFound { file, .. } => {
///             eprintln!("Could not find package for file: {}", file.display());
///         }
///         ChangesError::GitError { operation, reason } => {
///             eprintln!("Git operation '{}' failed: {}", operation, reason);
///         }
///         _ => eprintln!("Changes analysis error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::ChangesError;
/// use std::path::PathBuf;
///
/// let error = ChangesError::NoPackagesFound {
///     workspace_root: PathBuf::from("/workspace"),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("no packages found"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum ChangesError {
    /// Git operation failed during changes analysis.
    ///
    /// This error occurs when git commands or operations fail, such as
    /// retrieving commit information, diff operations, or status checks.
    #[error("Git operation failed: {operation} - {reason}")]
    GitError {
        /// Description of the git operation that failed.
        operation: String,
        /// Detailed error message from git.
        reason: String,
    },

    /// Invalid commit reference provided.
    ///
    /// This error occurs when a commit reference (hash, tag, branch) cannot
    /// be resolved to a valid commit in the repository.
    #[error("Invalid commit reference '{reference}': {reason}")]
    InvalidCommitRef {
        /// The invalid commit reference.
        reference: String,
        /// Description of why the reference is invalid.
        reason: String,
    },

    /// Commit range is invalid or empty.
    ///
    /// This error occurs when analyzing a commit range that has no commits
    /// or the range specification is invalid (e.g., `from` is after `to`).
    #[error("Invalid commit range from '{from}' to '{to}': {reason}")]
    InvalidCommitRange {
        /// Starting commit reference.
        from: String,
        /// Ending commit reference.
        to: String,
        /// Description of why the range is invalid.
        reason: String,
    },

    /// Package not found for the given file.
    ///
    /// This error occurs when attempting to map a file to its containing
    /// package but no package.json can be found in parent directories.
    #[error("Package not found for file '{file}' in workspace '{workspace_root}'")]
    PackageNotFound {
        /// Path to the file that couldn't be mapped to a package.
        file: PathBuf,
        /// Root path of the workspace.
        workspace_root: PathBuf,
    },

    /// No packages found in workspace.
    ///
    /// This error occurs when analyzing a workspace that contains no
    /// recognizable packages (no package.json files found).
    #[error("No packages found in workspace at '{workspace_root}'")]
    NoPackagesFound {
        /// Root path of the workspace where no packages were found.
        workspace_root: PathBuf,
    },

    /// Invalid file path provided.
    ///
    /// This error occurs when a file path is malformed, contains invalid
    /// characters, or points to an invalid location.
    #[error("Invalid file path '{path}': {reason}")]
    InvalidPath {
        /// The invalid file path.
        path: PathBuf,
        /// Description of why the path is invalid.
        reason: String,
    },

    /// File system error during changes analysis.
    ///
    /// This error occurs when filesystem operations (read, stat, traverse) fail
    /// during file mapping or package detection.
    #[error("Filesystem error at '{path}': {reason}")]
    FileSystemError {
        /// Path where the error occurred.
        path: PathBuf,
        /// Description of the filesystem error.
        reason: String,
    },

    /// Failed to parse package.json file.
    ///
    /// This error occurs when a package.json file exists but contains
    /// invalid JSON or is missing required fields.
    #[error("Failed to parse package.json at '{path}': {reason}")]
    PackageJsonParseError {
        /// Path to the package.json file.
        path: PathBuf,
        /// Description of the parsing error.
        reason: String,
    },

    /// Working directory has uncommitted changes that prevent analysis.
    ///
    /// This error occurs when attempting to analyze changes but the working
    /// directory state prevents accurate analysis.
    #[error("Working directory has uncommitted changes: {reason}")]
    UncommittedChanges {
        /// Description of the uncommitted changes.
        reason: String,
    },

    /// Monorepo detection failed.
    ///
    /// This error occurs when attempting to determine if a workspace is a
    /// monorepo but detection fails due to configuration errors or ambiguity.
    #[error("Failed to detect monorepo structure: {reason}")]
    MonorepoDetectionFailed {
        /// Description of why detection failed.
        reason: String,
    },

    /// No changes detected in the analyzed scope.
    ///
    /// This error occurs when analyzing a commit range or working directory
    /// but no file changes are detected.
    #[error("No changes detected in {scope}")]
    NoChangesDetected {
        /// Description of what was analyzed (e.g., "commit range", "working directory").
        scope: String,
    },

    /// File is outside workspace boundaries.
    ///
    /// This error occurs when a changed file is outside the workspace root,
    /// which shouldn't happen in normal operations.
    #[error("File '{path}' is outside workspace root '{workspace_root}'")]
    FileOutsideWorkspace {
        /// Path to the file outside the workspace.
        path: PathBuf,
        /// Root path of the workspace.
        workspace_root: PathBuf,
    },

    /// Invalid workspace root.
    ///
    /// This error occurs when the workspace root directory is invalid,
    /// doesn't exist, or is not a directory.
    #[error("Invalid workspace root '{path}': {reason}")]
    InvalidWorkspaceRoot {
        /// Path to the invalid workspace root.
        path: PathBuf,
        /// Description of why it's invalid.
        reason: String,
    },

    /// Failed to compute file statistics.
    ///
    /// This error occurs when computing statistics (lines added/deleted)
    /// for changed files fails.
    #[error("Failed to compute statistics for '{file}': {reason}")]
    StatisticsError {
        /// Path to the file.
        file: PathBuf,
        /// Description of the error.
        reason: String,
    },

    /// Pattern matching failed.
    ///
    /// This error occurs when file pattern matching (globs, regexes) fails
    /// due to invalid patterns or matching errors.
    #[error("Pattern matching failed for pattern '{pattern}': {reason}")]
    PatternError {
        /// The pattern that failed.
        pattern: String,
        /// Description of the error.
        reason: String,
    },

    /// Repository not found or invalid.
    ///
    /// This error occurs when the git repository cannot be found or opened
    /// at the expected location.
    #[error("Git repository not found at '{path}'")]
    RepositoryNotFound {
        /// Path where the repository was expected.
        path: PathBuf,
    },

    /// Merge conflict detected in files.
    ///
    /// This error occurs when analyzing files that contain unresolved
    /// merge conflict markers.
    #[error("Merge conflict detected in file '{file}'")]
    MergeConflict {
        /// Path to the file with merge conflicts.
        file: PathBuf,
    },

    /// Invalid configuration for changes analysis.
    ///
    /// This error occurs when the configuration for changes analysis
    /// is invalid or incomplete.
    #[error("Invalid changes analysis configuration: {reason}")]
    InvalidConfig {
        /// Description of the configuration problem.
        reason: String,
    },

    /// Analysis timeout exceeded.
    ///
    /// This error occurs when changes analysis takes longer than the
    /// configured timeout period.
    #[error("Changes analysis timed out after {duration_secs} seconds")]
    Timeout {
        /// Duration in seconds before timeout.
        duration_secs: u64,
    },

    /// Version calculation failed.
    ///
    /// This error occurs when calculating the next version for a package
    /// based on a version bump fails, typically due to version overflow
    /// or invalid version format.
    #[error(
        "Failed to calculate next version for package '{package}': cannot bump {current_version} with {bump_type} - {reason}"
    )]
    VersionCalculationFailed {
        /// Name of the package.
        package: String,
        /// Current version string.
        current_version: String,
        /// Type of bump attempted.
        bump_type: String,
        /// Description of why the calculation failed.
        reason: String,
    },
}

impl AsRef<str> for ChangesError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangesError::NoPackagesFound {
    ///     workspace_root: PathBuf::from("/workspace"),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("no packages found"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::GitError { .. } => "git error",
            Self::InvalidCommitRef { .. } => "invalid commit reference",
            Self::InvalidCommitRange { .. } => "invalid commit range",
            Self::PackageNotFound { .. } => "package not found",
            Self::NoPackagesFound { .. } => "no packages found",
            Self::InvalidPath { .. } => "invalid path",
            Self::FileSystemError { .. } => "filesystem error",
            Self::PackageJsonParseError { .. } => "package.json parse error",
            Self::UncommittedChanges { .. } => "uncommitted changes",
            Self::MonorepoDetectionFailed { .. } => "monorepo detection failed",
            Self::NoChangesDetected { .. } => "no changes detected",
            Self::FileOutsideWorkspace { .. } => "file outside workspace",
            Self::InvalidWorkspaceRoot { .. } => "invalid workspace root",
            Self::StatisticsError { .. } => "statistics error",
            Self::PatternError { .. } => "pattern error",
            Self::RepositoryNotFound { .. } => "repository not found",
            Self::MergeConflict { .. } => "merge conflict",
            Self::InvalidConfig { .. } => "invalid configuration",
            Self::Timeout { .. } => "analysis timeout",
            Self::VersionCalculationFailed { .. } => "version calculation failed",
        }
    }
}

impl ChangesError {
    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some changes errors (like filesystem errors or git errors) might be
    /// recoverable through retry, while others (like invalid paths) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesError;
    /// use std::path::PathBuf;
    ///
    /// let fs_error = ChangesError::FileSystemError {
    ///     path: PathBuf::from("package.json"),
    ///     reason: "temporary lock".to_string(),
    /// };
    /// assert!(fs_error.is_transient());
    ///
    /// let invalid_path = ChangesError::InvalidPath {
    ///     path: PathBuf::from(""),
    ///     reason: "empty path".to_string(),
    /// };
    /// assert!(!invalid_path.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::FileSystemError { .. }
                | Self::GitError { .. }
                | Self::Timeout { .. }
                | Self::StatisticsError { .. }
        )
    }

    /// Returns whether this error is related to git operations.
    ///
    /// This helper method identifies errors that originate from git operations,
    /// useful for categorizing and handling git-specific errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesError;
    ///
    /// let git_error = ChangesError::GitError {
    ///     operation: "fetch".to_string(),
    ///     reason: "network error".to_string(),
    /// };
    /// assert!(git_error.is_git_related());
    ///
    /// let invalid_ref = ChangesError::InvalidCommitRef {
    ///     reference: "invalid-ref".to_string(),
    ///     reason: "not found".to_string(),
    /// };
    /// assert!(invalid_ref.is_git_related());
    /// ```
    #[must_use]
    pub fn is_git_related(&self) -> bool {
        matches!(
            self,
            Self::GitError { .. }
                | Self::InvalidCommitRef { .. }
                | Self::InvalidCommitRange { .. }
                | Self::RepositoryNotFound { .. }
                | Self::MergeConflict { .. }
                | Self::UncommittedChanges { .. }
        )
    }
}
