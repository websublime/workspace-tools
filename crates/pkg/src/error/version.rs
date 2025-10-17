//! Version error types for package tools.
//!
//! **What**: Defines error types specific to version resolution, dependency propagation,
//! and version application operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! package names, version strings, and dependency chains. Implements `AsRef<str>` for
//! string conversion.
//!
//! **Why**: To provide clear, actionable error messages for versioning issues, enabling
//! users to quickly identify and fix version resolution problems, circular dependencies,
//! and propagation errors.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{VersionError, VersionResult};
//!
//! fn parse_version(version_str: &str) -> VersionResult<String> {
//!     if version_str.is_empty() {
//!         return Err(VersionError::InvalidVersion {
//!             version: version_str.to_string(),
//!             reason: "Version string cannot be empty".to_string(),
//!         });
//!     }
//!     Ok(version_str.to_string())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for version operations.
///
/// This type alias simplifies error handling in version-related functions
/// by defaulting to `VersionError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{VersionError, VersionResult};
///
/// fn resolve_version() -> VersionResult<String> {
///     Ok("1.0.0".to_string())
/// }
/// ```
pub type VersionResult<T> = Result<T, VersionError>;

/// Errors that can occur during version operations.
///
/// This enum covers all possible error scenarios when working with version
/// resolution, dependency propagation, version bumping, and version application.
///
/// # Examples
///
/// ## Handling version errors
///
/// ```rust
/// use sublime_pkg_tools::error::VersionError;
///
/// fn handle_version_error(error: VersionError) {
///     match error {
///         VersionError::InvalidVersion { version, reason } => {
///             eprintln!("Invalid version '{}': {}", version, reason);
///         }
///         VersionError::CircularDependency { cycle } => {
///             eprintln!("Circular dependency detected: {}", cycle.join(" -> "));
///         }
///         _ => eprintln!("Version error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::VersionError;
///
/// let error = VersionError::InvalidVersion {
///     version: "not-a-version".to_string(),
///     reason: "does not match semver format".to_string(),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("invalid version"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum VersionError {
    /// Invalid version string format.
    ///
    /// This error occurs when a version string does not conform to semantic versioning
    /// format or contains invalid characters.
    #[error("Invalid version '{version}': {reason}")]
    InvalidVersion {
        /// The invalid version string.
        version: String,
        /// Description of why the version is invalid.
        reason: String,
    },

    /// Failed to parse version from string.
    ///
    /// This error occurs when version parsing fails due to syntax errors or
    /// malformed version strings.
    #[error("Failed to parse version '{version}': {reason}")]
    ParseError {
        /// The version string that failed to parse.
        version: String,
        /// The underlying parsing error message.
        reason: String,
    },

    /// Invalid version bump type.
    ///
    /// This error occurs when an invalid or unsupported bump type is specified
    /// (not one of: major, minor, patch, none).
    #[error("Invalid bump type '{bump}': {reason}")]
    InvalidBump {
        /// The invalid bump type string.
        bump: String,
        /// Description of why the bump type is invalid.
        reason: String,
    },

    /// Circular dependency detected in package dependency graph.
    ///
    /// This error occurs during dependency graph construction when packages
    /// depend on each other in a circular manner, preventing version resolution.
    #[error("Circular dependency detected")]
    CircularDependency {
        /// The chain of package names forming the circular dependency.
        cycle: Vec<String>,
    },

    /// Package not found in workspace.
    ///
    /// This error occurs when attempting to resolve versions for a package
    /// that does not exist in the workspace.
    #[error("Package '{name}' not found in workspace at '{workspace_root}'")]
    PackageNotFound {
        /// Name of the package that was not found.
        name: String,
        /// Root path of the workspace where the package was expected.
        workspace_root: PathBuf,
    },

    /// Failed to read or parse package.json file.
    ///
    /// This error occurs when a package.json file cannot be read or contains
    /// invalid JSON.
    #[error("Failed to read package.json at '{path}': {reason}")]
    PackageJsonError {
        /// Path to the package.json file.
        path: PathBuf,
        /// Description of the error.
        reason: String,
    },

    /// Version resolution failed.
    ///
    /// This error occurs when the version resolver cannot determine appropriate
    /// versions for packages, possibly due to conflicts or constraints.
    #[error("Version resolution failed for package '{package}': {reason}")]
    ResolutionFailed {
        /// Name of the package that failed resolution.
        package: String,
        /// Description of why resolution failed.
        reason: String,
    },

    /// Dependency propagation failed.
    ///
    /// This error occurs when updating dependency versions across the dependency
    /// graph fails, possibly due to constraint violations.
    #[error("Failed to propagate version update from '{from}' to '{to}': {reason}")]
    PropagationFailed {
        /// Name of the package triggering the propagation.
        from: String,
        /// Name of the package that should receive the update.
        to: String,
        /// Description of why propagation failed.
        reason: String,
    },

    /// Invalid versioning strategy.
    ///
    /// This error occurs when an unsupported or invalid versioning strategy
    /// is specified in configuration.
    #[error("Invalid versioning strategy '{strategy}': expected 'independent' or 'unified'")]
    InvalidStrategy {
        /// The invalid strategy string.
        strategy: String,
    },

    /// Failed to apply version updates to package files.
    ///
    /// This error occurs when writing updated version numbers to package.json
    /// files fails due to filesystem or permission errors.
    #[error("Failed to apply version updates to '{path}': {reason}")]
    ApplyFailed {
        /// Path to the file that failed to update.
        path: PathBuf,
        /// Description of why the update failed.
        reason: String,
    },

    /// Dependency not found in package dependencies.
    ///
    /// This error occurs during dependency propagation when a referenced
    /// dependency does not exist in the package's dependency lists.
    #[error("Dependency '{dependency}' not found in package '{package}'")]
    DependencyNotFound {
        /// Name of the package.
        package: String,
        /// Name of the missing dependency.
        dependency: String,
    },

    /// Invalid dependency version specification.
    ///
    /// This error occurs when a dependency version spec cannot be parsed
    /// or is in an invalid format.
    #[error("Invalid version spec '{spec}' for dependency '{dependency}' in package '{package}': {reason}")]
    InvalidVersionSpec {
        /// Name of the package containing the dependency.
        package: String,
        /// Name of the dependency.
        dependency: String,
        /// The invalid version specification.
        spec: String,
        /// Description of why the spec is invalid.
        reason: String,
    },

    /// Version conflict detected.
    ///
    /// This error occurs when multiple packages require incompatible versions
    /// of the same dependency.
    #[error("Version conflict for dependency '{dependency}': {conflict}")]
    VersionConflict {
        /// Name of the dependency with conflicting versions.
        dependency: String,
        /// Description of the version conflict.
        conflict: String,
    },

    /// Maximum propagation depth exceeded.
    ///
    /// This error occurs when dependency propagation reaches the configured
    /// maximum depth, possibly indicating a very deep or circular dependency chain.
    #[error("Maximum propagation depth ({max_depth}) exceeded starting from package '{package}'")]
    MaxDepthExceeded {
        /// Name of the package where propagation started.
        package: String,
        /// The maximum allowed depth.
        max_depth: usize,
    },

    /// Snapshot version generation failed.
    ///
    /// This error occurs when attempting to generate a snapshot version
    /// (e.g., for pre-release testing) fails.
    #[error("Failed to generate snapshot version for '{package}': {reason}")]
    SnapshotFailed {
        /// Name of the package.
        package: String,
        /// Description of why snapshot generation failed.
        reason: String,
    },

    /// No packages to update.
    ///
    /// This error occurs when version resolution is attempted but no packages
    /// require version updates (empty changeset).
    #[error("No packages require version updates")]
    NoPackagesToUpdate,

    /// Workspace root not found or invalid.
    ///
    /// This error occurs when the workspace root directory cannot be determined
    /// or does not exist.
    #[error("Invalid workspace root '{path}': {reason}")]
    InvalidWorkspaceRoot {
        /// Path to the invalid workspace root.
        path: PathBuf,
        /// Description of why it's invalid.
        reason: String,
    },

    /// File system error during version operations.
    ///
    /// This error occurs when filesystem operations (read, write, create) fail
    /// during version resolution or application.
    #[error("Filesystem error at '{path}': {reason}")]
    FileSystemError {
        /// Path where the error occurred.
        path: PathBuf,
        /// Description of the filesystem error.
        reason: String,
    },
}

impl AsRef<str> for VersionError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::InvalidVersion {
    ///     version: "invalid".to_string(),
    ///     reason: "not semver".to_string(),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("invalid version"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidVersion { .. } => "invalid version",
            Self::ParseError { .. } => "version parse error",
            Self::InvalidBump { .. } => "invalid bump type",
            Self::CircularDependency { .. } => "circular dependency",
            Self::PackageNotFound { .. } => "package not found",
            Self::PackageJsonError { .. } => "package.json error",
            Self::ResolutionFailed { .. } => "version resolution failed",
            Self::PropagationFailed { .. } => "version propagation failed",
            Self::InvalidStrategy { .. } => "invalid versioning strategy",
            Self::ApplyFailed { .. } => "failed to apply version updates",
            Self::DependencyNotFound { .. } => "dependency not found",
            Self::InvalidVersionSpec { .. } => "invalid version specification",
            Self::VersionConflict { .. } => "version conflict",
            Self::MaxDepthExceeded { .. } => "max propagation depth exceeded",
            Self::SnapshotFailed { .. } => "snapshot version generation failed",
            Self::NoPackagesToUpdate => "no packages to update",
            Self::InvalidWorkspaceRoot { .. } => "invalid workspace root",
            Self::FileSystemError { .. } => "filesystem error",
        }
    }
}

impl VersionError {
    /// Returns the circular dependency cycle if this is a `CircularDependency` error.
    ///
    /// This helper method provides convenient access to the dependency cycle
    /// without pattern matching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::CircularDependency {
    ///     cycle: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()],
    /// };
    ///
    /// assert_eq!(error.cycle(), Some(&vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()]));
    /// ```
    #[must_use]
    pub fn cycle(&self) -> Option<&Vec<String>> {
        match self {
            Self::CircularDependency { cycle } => Some(cycle),
            _ => None,
        }
    }

    /// Returns the formatted circular dependency cycle as a string.
    ///
    /// This helper method formats the dependency cycle as an arrow-separated
    /// chain for display purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::VersionError;
    ///
    /// let error = VersionError::CircularDependency {
    ///     cycle: vec!["pkg-a".to_string(), "pkg-b".to_string(), "pkg-a".to_string()],
    /// };
    ///
    /// assert_eq!(error.cycle_display(), Some("pkg-a -> pkg-b -> pkg-a".to_string()));
    /// ```
    #[must_use]
    pub fn cycle_display(&self) -> Option<String> {
        self.cycle().map(|c| c.join(" -> "))
    }

    /// Returns whether this error is recoverable.
    ///
    /// Some version errors (like file system errors) might be recoverable through
    /// retry, while others (like circular dependencies) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::VersionError;
    /// use std::path::PathBuf;
    ///
    /// let fs_error = VersionError::FileSystemError {
    ///     path: PathBuf::from("package.json"),
    ///     reason: "temporary lock".to_string(),
    /// };
    /// assert!(fs_error.is_recoverable());
    ///
    /// let circular_error = VersionError::CircularDependency {
    ///     cycle: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    /// };
    /// assert!(!circular_error.is_recoverable());
    /// ```
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::FileSystemError { .. } | Self::PackageJsonError { .. } | Self::ApplyFailed { .. }
        )
    }
}
