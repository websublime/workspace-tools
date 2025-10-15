//! # Changeset error types and implementations
//!
//! ## What
//! This module provides error types specific to changeset operations,
//! including creation, validation, storage, and application processes.
//!
//! ## How
//! Provides detailed error types for changeset-related failures with specific
//! context for different stages of the changeset lifecycle.
//!
//! ## Why
//! Changesets are central to the version management workflow and require
//! precise error handling to provide clear feedback about validation failures,
//! file system issues, and application problems.

use std::path::PathBuf;
use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for changeset operations.
///
/// This is a convenience type alias for Results with `ChangesetError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{ChangesetResult, ChangesetError};
/// use std::path::PathBuf;
///
/// fn load_changeset(path: &PathBuf) -> ChangesetResult<String> {
///     if !path.exists() {
///         return Err(ChangesetError::NotFound { path: path.clone() });
///     }
///     Ok("changeset content".to_string())
/// }
/// ```
pub type ChangesetResult<T> = StdResult<T, ChangesetError>;

/// Changeset-related error types.
///
/// Handles errors in changeset creation, validation, storage,
/// and application processes.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::ChangesetError;
/// use std::path::PathBuf;
///
/// let error = ChangesetError::NotFound {
///     path: PathBuf::from(".changesets/missing.json"),
/// };
///
/// println!("Error: {}", error);
/// // Output: Changeset file not found: .changesets/missing.json
/// ```
#[derive(Error, Debug, Clone)]
pub enum ChangesetError {
    /// Changeset file not found
    #[error("Changeset file not found: {path}")]
    NotFound {
        /// Path to the missing changeset file
        path: PathBuf,
    },

    /// Invalid changeset format
    #[error("Invalid changeset format in '{path}': {reason}")]
    InvalidFormat {
        /// Path to the invalid changeset file
        path: PathBuf,
        /// Reason why the format is invalid
        reason: String,
    },

    /// Changeset validation failed
    #[error("Changeset validation failed for '{changeset_id}': {errors:?}")]
    ValidationFailed {
        /// Changeset identifier
        changeset_id: String,
        /// List of validation errors
        errors: Vec<String>,
    },

    /// Changeset already exists
    #[error("Changeset already exists: {changeset_id}")]
    AlreadyExists {
        /// Changeset identifier
        changeset_id: String,
    },

    /// Changeset creation failed
    #[error("Failed to create changeset for branch '{branch}': {reason}")]
    CreationFailed {
        /// Branch name
        branch: String,
        /// Failure reason
        reason: String,
    },

    /// Changeset application failed
    #[error("Failed to apply changeset '{changeset_id}': {reason}")]
    ApplicationFailed {
        /// Changeset identifier
        changeset_id: String,
        /// Failure reason
        reason: String,
    },

    /// History operation failed
    #[error("Changeset history operation failed: {operation} - {reason}")]
    HistoryOperationFailed {
        /// The operation that failed
        operation: String,
        /// Failure reason
        reason: String,
    },

    /// Environment not found in changeset
    #[error("Environment '{environment}' not found in changeset '{changeset_id}'")]
    EnvironmentNotFound {
        /// Environment name
        environment: String,
        /// Changeset identifier
        changeset_id: String,
    },

    /// Package not found in changeset
    #[error("Package '{package}' not found in changeset '{changeset_id}'")]
    PackageNotFound {
        /// Package name
        package: String,
        /// Changeset identifier
        changeset_id: String,
    },

    /// Failed to analyze Git changes
    #[error("Failed to analyze Git changes: {reason}")]
    GitAnalysisFailed {
        /// Reason for failure
        reason: String,
    },

    /// Failed to detect affected packages
    #[error("Failed to detect affected packages: {reason}")]
    PackageDetectionFailed {
        /// Reason for failure
        reason: String,
    },

    /// No changes detected
    #[error("No changes detected since '{since_ref}'")]
    NoChangesDetected {
        /// Reference point for comparison
        since_ref: String,
    },

    /// Failed to build dependency graph
    #[error("Failed to build dependency graph: {reason}")]
    DependencyAnalysisFailed {
        /// Reason for failure
        reason: String,
    },

    /// Failed to parse version
    #[error("Failed to parse version for package '{package}': {reason}")]
    VersionParsingFailed {
        /// Package name
        package: String,
        /// Reason for failure
        reason: String,
    },

    /// Failed to read package.json
    #[error("Failed to read package.json at '{path}': {reason}")]
    PackageJsonReadFailed {
        /// Path to package.json
        path: PathBuf,
        /// Reason for failure
        reason: String,
    },

    /// No packages found for changes
    #[error("No packages found affected by changes since '{since_ref}'")]
    NoPackagesAffected {
        /// Reference point for comparison
        since_ref: String,
    },
}

impl ChangesetError {
    /// Creates a not found error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the missing changeset file
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangesetError::not_found(PathBuf::from(".changesets/missing.json"));
    /// assert!(error.to_string().contains("not found"));
    /// ```
    #[must_use]
    pub fn not_found(path: PathBuf) -> Self {
        Self::NotFound { path }
    }

    /// Creates an invalid format error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the invalid changeset file
    /// * `reason` - Why the format is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangesetError::invalid_format(
    ///     PathBuf::from(".changesets/bad.json"),
    ///     "Missing required field 'branch'"
    /// );
    /// assert!(error.to_string().contains("Invalid changeset format"));
    /// ```
    #[must_use]
    pub fn invalid_format(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::InvalidFormat { path, reason: reason.into() }
    }

    /// Creates a validation failed error.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - Changeset identifier
    /// * `errors` - List of validation errors
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::validation_failed(
    ///     "feat-auth-20240115T103000Z",
    ///     vec!["Missing summary".to_string(), "Invalid package name".to_string()]
    /// );
    /// assert!(error.to_string().contains("validation failed"));
    /// ```
    #[must_use]
    pub fn validation_failed(changeset_id: impl Into<String>, errors: Vec<String>) -> Self {
        Self::ValidationFailed { changeset_id: changeset_id.into(), errors }
    }

    /// Creates an already exists error.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - Changeset identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::already_exists("feat-auth-20240115T103000Z");
    /// assert!(error.to_string().contains("already exists"));
    /// ```
    #[must_use]
    pub fn already_exists(changeset_id: impl Into<String>) -> Self {
        Self::AlreadyExists { changeset_id: changeset_id.into() }
    }

    /// Creates a creation failed error.
    ///
    /// # Arguments
    ///
    /// * `branch` - Branch name
    /// * `reason` - Why creation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::creation_failed("feat/auth", "No commits found");
    /// assert!(error.to_string().contains("Failed to create changeset"));
    /// ```
    #[must_use]
    pub fn creation_failed(branch: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CreationFailed { branch: branch.into(), reason: reason.into() }
    }

    /// Creates an application failed error.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - Changeset identifier
    /// * `reason` - Why application failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::application_failed(
    ///     "feat-auth-20240115T103000Z",
    ///     "Version conflict detected"
    /// );
    /// assert!(error.to_string().contains("Failed to apply changeset"));
    /// ```
    #[must_use]
    pub fn application_failed(changeset_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ApplicationFailed { changeset_id: changeset_id.into(), reason: reason.into() }
    }

    /// Creates a history operation failed error.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation that failed
    /// * `reason` - Why the operation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::history_operation_failed("archive", "Permission denied");
    /// assert!(error.to_string().contains("history operation failed"));
    /// ```
    #[must_use]
    pub fn history_operation_failed(
        operation: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::HistoryOperationFailed { operation: operation.into(), reason: reason.into() }
    }

    /// Creates an environment not found error.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    /// * `changeset_id` - Changeset identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::environment_not_found(
    ///     "production",
    ///     "feat-auth-20240115T103000Z"
    /// );
    /// assert!(error.to_string().contains("Environment"));
    /// ```
    #[must_use]
    pub fn environment_not_found(
        environment: impl Into<String>,
        changeset_id: impl Into<String>,
    ) -> Self {
        Self::EnvironmentNotFound {
            environment: environment.into(),
            changeset_id: changeset_id.into(),
        }
    }

    /// Creates a package not found error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `changeset_id` - Changeset identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::package_not_found(
    ///     "@myorg/missing-package",
    ///     "feat-auth-20240115T103000Z"
    /// );
    /// assert!(error.to_string().contains("Package"));
    /// ```
    #[must_use]
    pub fn package_not_found(package: impl Into<String>, changeset_id: impl Into<String>) -> Self {
        Self::PackageNotFound { package: package.into(), changeset_id: changeset_id.into() }
    }

    /// Checks if this is a file system related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangesetError::not_found(PathBuf::from("missing.json"));
    /// assert!(error.is_filesystem_error());
    ///
    /// let error = ChangesetError::validation_failed("id", vec![]);
    /// assert!(!error.is_filesystem_error());
    /// ```
    #[must_use]
    pub fn is_filesystem_error(&self) -> bool {
        matches!(self, Self::NotFound { .. } | Self::InvalidFormat { .. })
    }

    /// Checks if this is a validation related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::validation_failed("id", vec![]);
    /// assert!(error.is_validation_error());
    ///
    /// let error = ChangesetError::already_exists("id");
    /// assert!(!error.is_validation_error());
    /// ```
    #[must_use]
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationFailed { .. })
    }

    /// Checks if this is a lifecycle related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::creation_failed("branch", "reason");
    /// assert!(error.is_lifecycle_error());
    ///
    /// let error = ChangesetError::application_failed("id", "reason");
    /// assert!(error.is_lifecycle_error());
    /// ```
    #[must_use]
    pub fn is_lifecycle_error(&self) -> bool {
        matches!(
            self,
            Self::CreationFailed { .. }
                | Self::ApplicationFailed { .. }
                | Self::HistoryOperationFailed { .. }
        )
    }

    /// Creates a Git analysis failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why Git analysis failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::git_analysis_failed("Unable to retrieve commits");
    /// assert!(error.to_string().contains("Git changes"));
    /// ```
    #[must_use]
    pub fn git_analysis_failed(reason: impl Into<String>) -> Self {
        Self::GitAnalysisFailed { reason: reason.into() }
    }

    /// Creates a package detection failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why package detection failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::package_detection_failed("No packages found in workspace");
    /// assert!(error.to_string().contains("detect affected packages"));
    /// ```
    #[must_use]
    pub fn package_detection_failed(reason: impl Into<String>) -> Self {
        Self::PackageDetectionFailed { reason: reason.into() }
    }

    /// Creates a no changes detected error.
    ///
    /// # Arguments
    ///
    /// * `since_ref` - Reference point for comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::no_changes_detected("v1.0.0");
    /// assert!(error.to_string().contains("No changes detected"));
    /// ```
    #[must_use]
    pub fn no_changes_detected(since_ref: impl Into<String>) -> Self {
        Self::NoChangesDetected { since_ref: since_ref.into() }
    }

    /// Creates a dependency analysis failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why dependency analysis failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::dependency_analysis_failed("Circular dependency detected");
    /// assert!(error.to_string().contains("dependency graph"));
    /// ```
    #[must_use]
    pub fn dependency_analysis_failed(reason: impl Into<String>) -> Self {
        Self::DependencyAnalysisFailed { reason: reason.into() }
    }

    /// Creates a version parsing failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `reason` - Why version parsing failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::version_parsing_failed("@myorg/auth", "Invalid semver");
    /// assert!(error.to_string().contains("parse version"));
    /// ```
    #[must_use]
    pub fn version_parsing_failed(package: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::VersionParsingFailed { package: package.into(), reason: reason.into() }
    }

    /// Creates a package.json read failed error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to package.json
    /// * `reason` - Why reading failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangesetError::package_json_read_failed(
    ///     PathBuf::from("package.json"),
    ///     "File not found"
    /// );
    /// assert!(error.to_string().contains("package.json"));
    /// ```
    #[must_use]
    pub fn package_json_read_failed(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::PackageJsonReadFailed { path, reason: reason.into() }
    }

    /// Creates a no packages affected error.
    ///
    /// # Arguments
    ///
    /// * `since_ref` - Reference point for comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::no_packages_affected("v1.0.0");
    /// assert!(error.to_string().contains("No packages found"));
    /// ```
    #[must_use]
    pub fn no_packages_affected(since_ref: impl Into<String>) -> Self {
        Self::NoPackagesAffected { since_ref: since_ref.into() }
    }

    /// Checks if this is a Git operation related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::git_analysis_failed("reason");
    /// assert!(error.is_git_error());
    ///
    /// let error = ChangesetError::validation_failed("id", vec![]);
    /// assert!(!error.is_git_error());
    /// ```
    #[must_use]
    pub fn is_git_error(&self) -> bool {
        matches!(self, Self::GitAnalysisFailed { .. } | Self::NoChangesDetected { .. })
    }

    /// Checks if this is a package detection related error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::package_detection_failed("reason");
    /// assert!(error.is_package_error());
    ///
    /// let error = ChangesetError::validation_failed("id", vec![]);
    /// assert!(!error.is_package_error());
    /// ```
    #[must_use]
    pub fn is_package_error(&self) -> bool {
        matches!(
            self,
            Self::PackageDetectionFailed { .. }
                | Self::NoPackagesAffected { .. }
                | Self::PackageNotFound { .. }
                | Self::PackageJsonReadFailed { .. }
                | Self::VersionParsingFailed { .. }
        )
    }

    /// Gets the changeset ID from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    ///
    /// let error = ChangesetError::validation_failed("my-changeset", vec![]);
    /// assert_eq!(error.changeset_id(), Some("my-changeset"));
    ///
    /// let error = ChangesetError::creation_failed("branch", "reason");
    /// assert_eq!(error.changeset_id(), None);
    /// ```
    #[must_use]
    pub fn changeset_id(&self) -> Option<&str> {
        match self {
            Self::ValidationFailed { changeset_id, .. }
            | Self::AlreadyExists { changeset_id, .. }
            | Self::ApplicationFailed { changeset_id, .. }
            | Self::EnvironmentNotFound { changeset_id, .. }
            | Self::PackageNotFound { changeset_id, .. } => Some(changeset_id),
            Self::NotFound { .. }
            | Self::InvalidFormat { .. }
            | Self::CreationFailed { .. }
            | Self::HistoryOperationFailed { .. }
            | Self::GitAnalysisFailed { .. }
            | Self::PackageDetectionFailed { .. }
            | Self::NoChangesDetected { .. }
            | Self::DependencyAnalysisFailed { .. }
            | Self::VersionParsingFailed { .. }
            | Self::PackageJsonReadFailed { .. }
            | Self::NoPackagesAffected { .. } => None,
        }
    }

    /// Gets the file path from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangesetError;
    /// use std::path::PathBuf;
    ///
    /// let path = PathBuf::from("changeset.json");
    /// let error = ChangesetError::not_found(path.clone());
    /// assert_eq!(error.file_path(), Some(&path));
    ///
    /// let error = ChangesetError::validation_failed("id", vec![]);
    /// assert_eq!(error.file_path(), None);
    /// ```
    #[must_use]
    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            Self::NotFound { path, .. } | Self::InvalidFormat { path, .. } => Some(path),
            Self::ValidationFailed { .. }
            | Self::AlreadyExists { .. }
            | Self::CreationFailed { .. }
            | Self::ApplicationFailed { .. }
            | Self::HistoryOperationFailed { .. }
            | Self::EnvironmentNotFound { .. }
            | Self::PackageNotFound { .. }
            | Self::GitAnalysisFailed { .. }
            | Self::PackageDetectionFailed { .. }
            | Self::NoChangesDetected { .. }
            | Self::DependencyAnalysisFailed { .. }
            | Self::VersionParsingFailed { .. }
            | Self::NoPackagesAffected { .. } => None,
            Self::PackageJsonReadFailed { path, .. } => Some(path),
        }
    }
}

impl AsRef<str> for ChangesetError {
    fn as_ref(&self) -> &str {
        match self {
            ChangesetError::NotFound { .. } => "ChangesetError::NotFound",
            ChangesetError::InvalidFormat { .. } => "ChangesetError::InvalidFormat",
            ChangesetError::ValidationFailed { .. } => "ChangesetError::ValidationFailed",
            ChangesetError::AlreadyExists { .. } => "ChangesetError::AlreadyExists",
            ChangesetError::CreationFailed { .. } => "ChangesetError::CreationFailed",
            ChangesetError::ApplicationFailed { .. } => "ChangesetError::ApplicationFailed",
            ChangesetError::HistoryOperationFailed { .. } => {
                "ChangesetError::HistoryOperationFailed"
            }
            ChangesetError::EnvironmentNotFound { .. } => "ChangesetError::EnvironmentNotFound",
            ChangesetError::PackageNotFound { .. } => "ChangesetError::PackageNotFound",
            ChangesetError::GitAnalysisFailed { .. } => "ChangesetError::GitAnalysisFailed",
            ChangesetError::PackageDetectionFailed { .. } => {
                "ChangesetError::PackageDetectionFailed"
            }
            ChangesetError::NoChangesDetected { .. } => "ChangesetError::NoChangesDetected",
            ChangesetError::DependencyAnalysisFailed { .. } => {
                "ChangesetError::DependencyAnalysisFailed"
            }
            ChangesetError::VersionParsingFailed { .. } => "ChangesetError::VersionParsingFailed",
            ChangesetError::PackageJsonReadFailed { .. } => "ChangesetError::PackageJsonReadFailed",
            ChangesetError::NoPackagesAffected { .. } => "ChangesetError::NoPackagesAffected",
        }
    }
}
