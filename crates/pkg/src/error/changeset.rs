//! Changeset error types for package tools.
//!
//! **What**: Defines error types specific to changeset creation, loading, storage,
//! archiving, and management operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! changeset IDs, branch names, storage paths, and git integration details. Implements
//! `AsRef<str>` for string conversion.
//!
//! **Why**: To provide clear, actionable error messages for changeset operations, enabling
//! users to quickly identify and fix issues with changeset storage, validation, and
//! git integration.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{ChangesetError, ChangesetResult};
//!
//! fn load_changeset(branch: &str) -> ChangesetResult<String> {
//!     if branch.is_empty() {
//!         return Err(ChangesetError::InvalidBranch {
//!             branch: branch.to_string(),
//!             reason: "Branch name cannot be empty".to_string(),
//!         });
//!     }
//!     Ok("changeset-data".to_string())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for changeset operations.
///
/// This type alias simplifies error handling in changeset-related functions
/// by defaulting to `ChangesetError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{ChangesetError, ChangesetResult};
///
/// fn create_changeset() -> ChangesetResult<String> {
///     Ok("changeset-id".to_string())
/// }
/// ```
pub type ChangesetResult<T> = Result<T, ChangesetError>;

/// Errors that can occur during changeset operations.
///
/// This enum covers all possible error scenarios when working with changesets,
/// including storage operations, validation, git integration, and archiving.
///
/// # Examples
///
/// ## Handling changeset errors
///
/// ```rust
/// use sublime_pkg_tools::error::ChangesetError;
/// use std::path::PathBuf;
///
/// fn handle_changeset_error(error: ChangesetError) {
///     match error {
///         ChangesetError::NotFound { branch } => {
///             eprintln!("Changeset not found for branch: {}", branch);
///         }
///         ChangesetError::StorageError { path, reason } => {
///             eprintln!("Storage error at {}: {}", path.display(), reason);
///         }
///         _ => eprintln!("Changeset error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::ChangesetError;
///
/// let error = ChangesetError::InvalidBranch {
///     branch: "".to_string(),
///     reason: "empty branch name".to_string(),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("invalid branch"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum ChangesetError {
    /// Changeset not found for the specified branch.
    ///
    /// This error occurs when attempting to load a changeset that does not exist
    /// for the given branch name.
    #[error("Changeset not found for branch '{branch}'")]
    NotFound {
        /// The branch name for which the changeset was not found.
        branch: String,
    },

    /// Invalid branch name provided.
    ///
    /// This error occurs when a branch name is empty, contains invalid characters,
    /// or does not meet the required format.
    #[error("Invalid branch name '{branch}': {reason}")]
    InvalidBranch {
        /// The invalid branch name.
        branch: String,
        /// Description of why the branch name is invalid.
        reason: String,
    },

    /// Changeset validation failed.
    /// Configuration validation failed with one or more validation errors.
    ///
    /// This error occurs when the changeset structure is invalid or incomplete,
    /// such as missing required fields or containing invalid data.
    #[error("Changeset validation failed")]
    ValidationFailed {
        /// List of validation error messages.
        errors: Vec<String>,
    },

    /// Storage operation failed.
    ///
    /// This error occurs when reading from or writing to changeset storage fails
    /// due to filesystem errors, permission issues, or corruption.
    #[error("Changeset storage error at '{path}': {reason}")]
    StorageError {
        /// Path where the storage error occurred.
        path: PathBuf,
        /// Description of the storage error.
        reason: String,
    },

    /// Failed to serialize or deserialize changeset data.
    ///
    /// This error occurs when converting changeset data to/from JSON or other
    /// serialization formats fails.
    #[error("Failed to {operation} changeset data: {reason}")]
    SerializationError {
        /// The operation that failed (e.g., "serialize", "deserialize").
        operation: String,
        /// Description of the serialization error.
        reason: String,
    },

    /// Changeset already exists for the branch.
    ///
    /// This error occurs when attempting to create a new changeset for a branch
    /// that already has one.
    #[error("Changeset already exists for branch '{branch}' at '{path}'")]
    AlreadyExists {
        /// The branch name.
        branch: String,
        /// Path to the existing changeset file.
        path: PathBuf,
    },

    /// Git operation failed during changeset operations.
    ///
    /// This error occurs when git commands or operations fail during changeset
    /// management, such as retrieving commit information.
    #[error("Git operation failed: {operation} - {reason}")]
    GitError {
        /// Description of the git operation that failed.
        operation: String,
        /// Detailed error message from git.
        reason: String,
    },

    /// Failed to archive changeset.
    ///
    /// This error occurs when moving a changeset to the history/archive location
    /// fails, possibly due to filesystem issues.
    #[error("Failed to archive changeset for branch '{branch}': {reason}")]
    ArchiveError {
        /// The branch name of the changeset being archived.
        branch: String,
        /// Description of why archiving failed.
        reason: String,
    },

    /// Invalid changeset ID format.
    ///
    /// This error occurs when a changeset ID does not match the expected format
    /// or contains invalid characters.
    #[error("Invalid changeset ID '{id}': {reason}")]
    InvalidId {
        /// The invalid changeset ID.
        id: String,
        /// Description of why the ID is invalid.
        reason: String,
    },

    /// Package not found in changeset.
    ///
    /// This error occurs when attempting to access or modify a package in a
    /// changeset that doesn't contain that package.
    #[error("Package '{package}' not found in changeset for branch '{branch}'")]
    PackageNotInChangeset {
        /// The branch name.
        branch: String,
        /// The package name that was not found.
        package: String,
    },

    /// Invalid environment name.
    ///
    /// This error occurs when an environment name is not in the list of
    /// configured available environments.
    #[error("Invalid environment '{environment}': not in available environments {available:?}")]
    InvalidEnvironment {
        /// The invalid environment name.
        environment: String,
        /// List of available/valid environment names.
        available: Vec<String>,
    },

    /// Empty changeset with no packages.
    ///
    /// This error occurs when attempting to save or process a changeset that
    /// contains no packages.
    #[error("Changeset for branch '{branch}' is empty (no packages)")]
    EmptyChangeset {
        /// The branch name of the empty changeset.
        branch: String,
    },

    /// Commit not found in repository.
    ///
    /// This error occurs when attempting to add a commit to a changeset but
    /// the commit hash does not exist in the git repository.
    #[error("Commit '{commit}' not found in repository")]
    CommitNotFound {
        /// The commit hash that was not found.
        commit: String,
    },

    /// Invalid commit hash format.
    ///
    /// This error occurs when a commit hash does not match the expected format
    /// (typically 40-character hex string for full SHA).
    #[error("Invalid commit hash '{commit}': {reason}")]
    InvalidCommit {
        /// The invalid commit hash.
        commit: String,
        /// Description of why the commit hash is invalid.
        reason: String,
    },

    /// History query failed.
    ///
    /// This error occurs when querying the changeset history fails, possibly
    /// due to corrupted archive files or filesystem issues.
    #[error("Failed to query changeset history: {reason}")]
    HistoryQueryFailed {
        /// Description of why the history query failed.
        reason: String,
    },

    /// Permission denied for changeset operation.
    ///
    /// This error occurs when the process lacks necessary permissions to read,
    /// write, or modify changeset files.
    #[error("Permission denied for changeset operation at '{path}': {operation}")]
    PermissionDenied {
        /// Path where permission was denied.
        path: PathBuf,
        /// The operation that was attempted.
        operation: String,
    },

    /// Concurrent modification detected.
    ///
    /// This error occurs when a changeset file has been modified by another
    /// process between read and write operations.
    #[error("Concurrent modification detected for changeset '{branch}': expected timestamp {expected}, found {actual}")]
    ConcurrentModification {
        /// The branch name of the changeset.
        branch: String,
        /// Expected last modification timestamp.
        expected: String,
        /// Actual last modification timestamp.
        actual: String,
    },

    /// Invalid changeset path configuration.
    ///
    /// This error occurs when the configured changeset storage path is invalid,
    /// inaccessible, or points to an invalid location.
    #[error("Invalid changeset path configuration '{path}': {reason}")]
    InvalidPath {
        /// The invalid path.
        path: PathBuf,
        /// Description of why the path is invalid.
        reason: String,
    },

    /// Failed to lock changeset for exclusive access.
    ///
    /// This error occurs when attempting to acquire an exclusive lock on a
    /// changeset file fails, possibly due to another process holding the lock.
    #[error("Failed to lock changeset for branch '{branch}': {reason}")]
    LockFailed {
        /// The branch name of the changeset.
        branch: String,
        /// Description of why the lock failed.
        reason: String,
    },
}

impl AsRef<str> for ChangesetError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::NotFound {
    ///     branch: "feature/new-api".to_string(),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("not found"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::NotFound { .. } => "changeset not found",
            Self::InvalidBranch { .. } => "invalid branch name",
            Self::ValidationFailed { .. } => "changeset validation failed",
            Self::StorageError { .. } => "changeset storage error",
            Self::SerializationError { .. } => "changeset serialization error",
            Self::AlreadyExists { .. } => "changeset already exists",
            Self::GitError { .. } => "git error",
            Self::ArchiveError { .. } => "changeset archive error",
            Self::InvalidId { .. } => "invalid changeset id",
            Self::PackageNotInChangeset { .. } => "package not in changeset",
            Self::InvalidEnvironment { .. } => "invalid environment",
            Self::EmptyChangeset { .. } => "empty changeset",
            Self::CommitNotFound { .. } => "commit not found",
            Self::InvalidCommit { .. } => "invalid commit",
            Self::HistoryQueryFailed { .. } => "history query failed",
            Self::PermissionDenied { .. } => "permission denied",
            Self::ConcurrentModification { .. } => "concurrent modification",
            Self::InvalidPath { .. } => "invalid changeset path",
            Self::LockFailed { .. } => "lock failed",
        }
    }
}

impl ChangesetError {
    /// Returns the number of errors for `ValidationFailed` variant.
    ///
    /// This helper method provides a convenient way to get the count of validation
    /// errors without pattern matching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::ValidationFailed {
    ///     errors: vec![
    ///         "Missing bump type".to_string(),
    ///         "Empty packages list".to_string(),
    ///     ],
    /// };
    ///
    /// assert_eq!(error.count(), 2);
    /// ```
    #[must_use]
    pub fn count(&self) -> usize {
        match self {
            Self::ValidationFailed { errors } => errors.len(),
            _ => 1,
        }
    }

    /// Returns the formatted error list as a single string.
    ///
    /// This helper method formats all validation errors as a bulleted list,
    /// useful for displaying multiple errors in a user-friendly format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::ValidationFailed {
    ///     errors: vec![
    ///         "Invalid bump".to_string(),
    ///         "Missing packages".to_string(),
    ///     ],
    /// };
    ///
    /// let formatted = error.errors();
    /// assert!(formatted.contains("Invalid bump"));
    /// assert!(formatted.contains("Missing packages"));
    /// ```
    #[must_use]
    pub fn errors(&self) -> String {
        match self {
            Self::ValidationFailed { errors } => {
                errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n")
            }
            _ => self.to_string(),
        }
    }

    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some changeset errors (like concurrent modifications or lock failures)
    /// might be recoverable through retry, while others (like validation errors) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let lock_error = ChangesetError::LockFailed {
    ///     branch: "main".to_string(),
    ///     reason: "already locked".to_string(),
    /// };
    /// assert!(lock_error.is_transient());
    ///
    /// let validation_error = ChangesetError::ValidationFailed {
    ///     errors: vec!["invalid data".to_string()],
    /// };
    /// assert!(!validation_error.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::LockFailed { .. }
                | Self::ConcurrentModification { .. }
                | Self::StorageError { .. }
                | Self::GitError { .. }
        )
    }
}

