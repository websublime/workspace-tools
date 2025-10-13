//! # Version error types and implementations
//!
//! ## What
//! This module provides error types specific to version management operations,
//! including version parsing, resolution, conflicts, and bump calculations.
//!
//! ## How
//! Provides detailed error types for version-related failures with specific
//! context and conversion from underlying errors like semver parsing errors.
//!
//! ## Why
//! Version management is a core functionality that requires precise error
//! handling to provide clear feedback about version conflicts, invalid formats,
//! and resolution failures.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for version operations.
///
/// This is a convenience type alias for Results with `VersionError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{VersionResult, VersionError};
///
/// fn parse_version(version: &str) -> VersionResult<semver::Version> {
///     semver::Version::parse(version).map_err(|e| VersionError::ParseFailed {
///         version: version.to_string(),
///         source: e,
///     })
/// }
/// ```
pub type VersionResult<T> = StdResult<T, VersionError>;

/// Version-related error types.
///
/// Covers all version management scenarios including parsing,
/// resolution, and version conflict detection.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::VersionError;
///
/// let error = VersionError::InvalidFormat {
///     version: "not-a-version".to_string(),
///     reason: "Missing major.minor.patch components".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Invalid version format: 'not-a-version' - Missing major.minor.patch components
/// ```
#[derive(Error, Debug)]
pub enum VersionError {
    /// Invalid version string format
    #[error("Invalid version format: '{version}' - {reason}")]
    InvalidFormat {
        /// The invalid version string
        version: String,
        /// Reason why it's invalid
        reason: String,
    },

    /// Version parsing failed
    #[error("Failed to parse version '{version}': {source}")]
    ParseFailed {
        /// The version string that failed to parse
        version: String,
        /// The underlying parse error
        #[source]
        source: semver::Error,
    },

    /// Snapshot version resolution failed
    #[error("Failed to resolve snapshot version for package '{package}': {reason}")]
    SnapshotResolutionFailed {
        /// Package name
        package: String,
        /// Failure reason
        reason: String,
    },

    /// Version conflict detected
    #[error("Version conflict for package '{package}': current={current}, requested={requested}")]
    Conflict {
        /// Package name with conflict
        package: String,
        /// Current version
        current: String,
        /// Requested version
        requested: String,
    },

    /// Version bump calculation failed
    #[error("Failed to calculate version bump for package '{package}': {reason}")]
    BumpCalculationFailed {
        /// Package name
        package: String,
        /// Failure reason
        reason: String,
    },

    /// Pre-release version handling error
    #[error("Pre-release version error for '{version}': {reason}")]
    PreReleaseError {
        /// Version with pre-release
        version: String,
        /// Error reason
        reason: String,
    },
}

impl VersionError {
    /// Creates an invalid format error.
    ///
    /// # Arguments
    ///
    /// * `version` - The invalid version string
    /// * `reason` - Why the version is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::invalid_format("1.2", "Missing patch version");
    /// assert!(error.to_string().contains("1.2"));
    /// ```
    #[must_use]
    pub fn invalid_format(version: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFormat { version: version.into(), reason: reason.into() }
    }

    /// Creates a snapshot resolution failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `reason` - Why snapshot resolution failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::snapshot_resolution_failed(
    ///     "my-package",
    ///     "No Git repository found"
    /// );
    /// assert!(error.to_string().contains("my-package"));
    /// ```
    #[must_use]
    pub fn snapshot_resolution_failed(
        package: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::SnapshotResolutionFailed { package: package.into(), reason: reason.into() }
    }

    /// Creates a version conflict error.
    ///
    /// # Arguments
    ///
    /// * `package` - The package with conflicting versions
    /// * `current` - Current version
    /// * `requested` - Requested version
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::conflict("my-package", "1.0.0", "2.0.0");
    /// assert!(error.to_string().contains("conflict"));
    /// ```
    #[must_use]
    pub fn conflict(
        package: impl Into<String>,
        current: impl Into<String>,
        requested: impl Into<String>,
    ) -> Self {
        Self::Conflict {
            package: package.into(),
            current: current.into(),
            requested: requested.into(),
        }
    }

    /// Creates a bump calculation failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - The package name
    /// * `reason` - Why bump calculation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::bump_calculation_failed(
    ///     "my-package",
    ///     "No conventional commits found"
    /// );
    /// assert!(error.to_string().contains("bump"));
    /// ```
    #[must_use]
    pub fn bump_calculation_failed(package: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::BumpCalculationFailed { package: package.into(), reason: reason.into() }
    }

    /// Creates a pre-release error.
    ///
    /// # Arguments
    ///
    /// * `version` - The version with pre-release component
    /// * `reason` - Why pre-release handling failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::pre_release_error(
    ///     "1.0.0-alpha.1",
    ///     "Invalid pre-release identifier"
    /// );
    /// assert!(error.to_string().contains("pre-release"));
    /// ```
    #[must_use]
    pub fn pre_release_error(version: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::PreReleaseError { version: version.into(), reason: reason.into() }
    }

    /// Checks if this is a parsing error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::invalid_format("1.2", "Missing patch");
    /// assert!(!error.is_parse_error());
    /// ```
    #[must_use]
    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::ParseFailed { .. })
    }

    /// Checks if this is a conflict error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::conflict("pkg", "1.0.0", "2.0.0");
    /// assert!(error.is_conflict_error());
    /// ```
    #[must_use]
    pub fn is_conflict_error(&self) -> bool {
        matches!(self, Self::Conflict { .. })
    }

    /// Checks if this is a snapshot resolution error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::snapshot_resolution_failed("pkg", "No git");
    /// assert!(error.is_snapshot_error());
    /// ```
    #[must_use]
    pub fn is_snapshot_error(&self) -> bool {
        matches!(self, Self::SnapshotResolutionFailed { .. })
    }

    /// Gets the package name from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::conflict("my-package", "1.0.0", "2.0.0");
    /// assert_eq!(error.package_name(), Some("my-package"));
    ///
    /// let error = VersionError::invalid_format("1.2", "Missing patch");
    /// assert_eq!(error.package_name(), None);
    /// ```
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::SnapshotResolutionFailed { package, .. }
            | Self::Conflict { package, .. }
            | Self::BumpCalculationFailed { package, .. } => Some(package),
            Self::InvalidFormat { .. }
            | Self::ParseFailed { .. }
            | Self::PreReleaseError { .. } => None,
        }
    }

    /// Gets the version string from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::invalid_format("1.2.3", "reason");
    /// assert_eq!(error.version_string(), Some("1.2.3"));
    ///
    /// let error = VersionError::conflict("pkg", "1.0.0", "2.0.0");
    /// assert_eq!(error.version_string(), None);
    /// ```
    #[must_use]
    pub fn version_string(&self) -> Option<&str> {
        match self {
            Self::InvalidFormat { version, .. }
            | Self::ParseFailed { version, .. }
            | Self::PreReleaseError { version, .. } => Some(version),
            Self::SnapshotResolutionFailed { .. }
            | Self::Conflict { .. }
            | Self::BumpCalculationFailed { .. } => None,
        }
    }
}

impl AsRef<str> for VersionError {
    fn as_ref(&self) -> &str {
        match self {
            VersionError::InvalidFormat { .. } => "VersionError::InvalidFormat",
            VersionError::ParseFailed { .. } => "VersionError::ParseFailed",
            VersionError::SnapshotResolutionFailed { .. } => {
                "VersionError::SnapshotResolutionFailed"
            }
            VersionError::Conflict { .. } => "VersionError::Conflict",
            VersionError::BumpCalculationFailed { .. } => "VersionError::BumpCalculationFailed",
            VersionError::PreReleaseError { .. } => "VersionError::PreReleaseError",
        }
    }
}
