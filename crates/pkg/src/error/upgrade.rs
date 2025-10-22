//! Upgrade error types for package tools.
//!
//! **What**: Defines error types specific to dependency upgrade detection, registry operations,
//! backup management, and upgrade application operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! package names, dependency names, registry URLs, and backup paths. Implements
//! `AsRef<str>` for string conversion.
//!
//! **Why**: To provide clear, actionable error messages for upgrade operations, enabling
//! users to quickly identify and fix issues with registry communication, version resolution,
//! backup management, and upgrade application.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{UpgradeError, UpgradeResult};
//!
//! fn detect_upgrades(package: &str) -> UpgradeResult<Vec<String>> {
//!     if package.is_empty() {
//!         return Err(UpgradeError::InvalidPackageName {
//!             name: package.to_string(),
//!             reason: "Package name cannot be empty".to_string(),
//!         });
//!     }
//!     Ok(vec![])
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for upgrade operations.
///
/// This type alias simplifies error handling in upgrade-related functions
/// by defaulting to `UpgradeError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{UpgradeError, UpgradeResult};
///
/// fn apply_upgrades() -> UpgradeResult<usize> {
///     Ok(5)
/// }
/// ```
pub type UpgradeResult<T> = Result<T, UpgradeError>;

/// Errors that can occur during dependency upgrade operations.
///
/// This enum covers all possible error scenarios when working with dependency
/// upgrades, including registry communication, version detection, backup management,
/// and upgrade application.
///
/// # Examples
///
/// ## Handling upgrade errors
///
/// ```rust
/// use sublime_pkg_tools::error::UpgradeError;
///
/// fn handle_upgrade_error(error: UpgradeError) {
///     match error {
///         UpgradeError::RegistryError { package, reason } => {
///             eprintln!("Registry error for {}: {}", package, reason);
///         }
///         UpgradeError::BackupFailed { path, reason } => {
///             eprintln!("Backup failed at {}: {}", path.display(), reason);
///         }
///         _ => eprintln!("Upgrade error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::UpgradeError;
///
/// let error = UpgradeError::NoUpgradesAvailable;
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("no upgrades"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum UpgradeError {
    /// Registry communication error.
    ///
    /// This error occurs when communicating with npm registry or custom
    /// registries fails due to network issues, authentication, or rate limiting.
    #[error("Registry error for package '{package}': {reason}")]
    RegistryError {
        /// Name of the package being queried.
        package: String,
        /// Description of the registry error.
        reason: String,
    },

    /// Package not found in registry.
    ///
    /// This error occurs when a package does not exist in the configured
    /// registry or the registry returns a 404 response.
    #[error("Package '{package}' not found in registry '{registry}'")]
    PackageNotFound {
        /// Name of the package that was not found.
        package: String,
        /// URL of the registry that was queried.
        registry: String,
    },

    /// Registry authentication failed.
    ///
    /// This error occurs when registry access requires authentication but
    /// the provided credentials are invalid or missing.
    #[error("Authentication failed for registry '{registry}': {reason}")]
    AuthenticationFailed {
        /// URL of the registry.
        registry: String,
        /// Description of the authentication error.
        reason: String,
    },

    /// Registry request timed out.
    ///
    /// This error occurs when a registry request exceeds the configured
    /// timeout duration.
    #[error("Registry request timed out after {timeout_secs} seconds for package '{package}'")]
    RegistryTimeout {
        /// Name of the package being queried.
        package: String,
        /// Timeout duration in seconds.
        timeout_secs: u64,
    },

    /// Invalid registry response.
    ///
    /// This error occurs when the registry returns a response that cannot
    /// be parsed or contains invalid data.
    #[error("Invalid registry response for package '{package}': {reason}")]
    InvalidResponse {
        /// Name of the package.
        package: String,
        /// Description of why the response is invalid.
        reason: String,
    },

    /// Failed to create backup before applying upgrades.
    ///
    /// This error occurs when the backup operation fails, preventing
    /// upgrades from being applied to protect against data loss.
    #[error("Failed to create backup at '{path}': {reason}")]
    BackupFailed {
        /// Path where the backup should be created.
        path: PathBuf,
        /// Description of why the backup failed.
        reason: String,
    },

    /// No backup found for rollback.
    ///
    /// This error occurs when attempting to rollback upgrades but no
    /// backup exists to restore from.
    #[error("No backup found at '{path}' for rollback")]
    NoBackup {
        /// Path where the backup was expected.
        path: PathBuf,
    },

    /// Rollback operation failed.
    ///
    /// This error occurs when attempting to restore from a backup fails,
    /// leaving the system in a potentially inconsistent state.
    #[error("Rollback failed: {reason}")]
    RollbackFailed {
        /// Description of why the rollback failed.
        reason: String,
    },

    /// Failed to apply upgrades to package.json files.
    ///
    /// This error occurs when writing updated dependency versions to
    /// package.json files fails.
    #[error("Failed to apply upgrades to '{path}': {reason}")]
    ApplyFailed {
        /// Path to the package.json file.
        path: PathBuf,
        /// Description of why the application failed.
        reason: String,
    },

    /// No upgrades available for any dependencies.
    ///
    /// This error occurs when upgrade detection finds that all dependencies
    /// are already at their latest versions.
    #[error("No upgrades available for any dependencies")]
    NoUpgradesAvailable,

    /// Invalid package name.
    ///
    /// This error occurs when a package name is empty, contains invalid
    /// characters, or does not conform to npm naming rules.
    #[error("Invalid package name '{name}': {reason}")]
    InvalidPackageName {
        /// The invalid package name.
        name: String,
        /// Description of why the name is invalid.
        reason: String,
    },

    /// Invalid version specification.
    ///
    /// This error occurs when a version specification cannot be parsed
    /// or is in an invalid format.
    #[error("Invalid version specification '{spec}' for package '{package}': {reason}")]
    InvalidVersionSpec {
        /// Name of the package.
        package: String,
        /// The invalid version specification.
        spec: String,
        /// Description of why the spec is invalid.
        reason: String,
    },

    /// Invalid version string.
    ///
    /// This error occurs when a version string cannot be parsed as
    /// a valid semantic version.
    #[error("Invalid version '{version}': {message}")]
    InvalidVersion {
        /// The invalid version string.
        version: String,
        /// Description of why the version is invalid.
        message: String,
    },

    /// Version comparison failed.
    ///
    /// This error occurs when comparing two versions fails due to
    /// incompatible version formats or invalid semver.
    #[error("Failed to compare versions for package '{package}': {reason}")]
    VersionComparisonFailed {
        /// Name of the package.
        package: String,
        /// Description of the comparison error.
        reason: String,
    },

    /// Failed to parse .npmrc configuration.
    ///
    /// This error occurs when reading or parsing .npmrc files fails
    /// due to syntax errors or invalid configuration.
    #[error("Failed to parse .npmrc at '{path}': {reason}")]
    NpmrcParseError {
        /// Path to the .npmrc file.
        path: PathBuf,
        /// Description of the parsing error.
        reason: String,
    },

    /// File system error during upgrade operations.
    ///
    /// This error occurs when filesystem operations fail during upgrade
    /// detection, backup creation, or application.
    #[error("Filesystem error at '{path}': {reason}")]
    FileSystemError {
        /// Path where the error occurred.
        path: PathBuf,
        /// Description of the filesystem error.
        reason: String,
    },

    /// Package.json parse error.
    ///
    /// This error occurs when reading or parsing a package.json file
    /// fails due to invalid JSON or missing required fields.
    #[error("Failed to parse package.json at '{path}': {reason}")]
    PackageJsonError {
        /// Path to the package.json file.
        path: PathBuf,
        /// Description of the parsing error.
        reason: String,
    },

    /// Deprecated package detected.
    ///
    /// This error occurs when a dependency is marked as deprecated in
    /// the registry and strict mode is enabled.
    #[error("Package '{package}' is deprecated: {message}")]
    DeprecatedPackage {
        /// Name of the deprecated package.
        package: String,
        /// Deprecation message from the registry.
        message: String,
        /// Alternative package recommendation, if available.
        alternative: Option<String>,
    },

    /// Changeset creation failed during auto-changeset.
    ///
    /// This error occurs when automatic changeset creation is enabled
    /// but fails during upgrade application.
    #[error("Failed to create changeset for upgrades: {reason}")]
    ChangesetCreationFailed {
        /// Description of why changeset creation failed.
        reason: String,
    },

    /// Invalid upgrade configuration.
    ///
    /// This error occurs when the upgrade configuration is invalid,
    /// incomplete, or contains conflicting settings.
    #[error("Invalid upgrade configuration: {reason}")]
    InvalidConfig {
        /// Description of the configuration problem.
        reason: String,
    },

    /// Workspace not found or invalid.
    ///
    /// This error occurs when the workspace root cannot be determined
    /// or is not a valid workspace.
    #[error("Invalid workspace at '{path}': {reason}")]
    InvalidWorkspace {
        /// Path to the invalid workspace.
        path: PathBuf,
        /// Description of why it's invalid.
        reason: String,
    },

    /// No packages found in workspace.
    ///
    /// This error occurs when no package.json files can be found in the
    /// workspace during upgrade detection.
    #[error("No packages found in workspace at '{workspace_root}'")]
    NoPackagesFound {
        /// Path to the workspace root.
        workspace_root: PathBuf,
    },

    /// Concurrent modification detected.
    ///
    /// This error occurs when a package.json file is modified by another
    /// process between detection and application.
    #[error("Concurrent modification detected in '{path}'")]
    ConcurrentModification {
        /// Path to the modified file.
        path: PathBuf,
    },

    /// Network error during registry operations.
    ///
    /// This error occurs when network connectivity issues prevent
    /// communication with registries.
    #[error("Network error: {reason}")]
    NetworkError {
        /// Description of the network error.
        reason: String,
    },

    /// Rate limit exceeded for registry.
    ///
    /// This error occurs when too many requests are made to a registry
    /// and rate limiting is enforced.
    #[error("Rate limit exceeded for registry '{registry}': {reason}")]
    RateLimitExceeded {
        /// URL of the registry.
        registry: String,
        /// Description including retry information.
        reason: String,
    },

    /// Maximum backup limit exceeded.
    ///
    /// This error occurs when attempting to create a new backup but
    /// the maximum number of backups has been reached.
    #[error("Maximum backup limit ({max_backups}) exceeded at '{path}'")]
    MaxBackupsExceeded {
        /// Path to the backup directory.
        path: PathBuf,
        /// Maximum number of backups allowed.
        max_backups: usize,
    },

    /// Backup corruption detected.
    ///
    /// This error occurs when a backup file exists but is corrupted
    /// or cannot be restored.
    #[error("Backup corrupted at '{path}': {reason}")]
    BackupCorrupted {
        /// Path to the corrupted backup.
        path: PathBuf,
        /// Description of the corruption.
        reason: String,
    },
}

impl AsRef<str> for UpgradeError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::UpgradeError;
    ///
    /// let error = UpgradeError::NoUpgradesAvailable;
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("no upgrades"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::RegistryError { .. } => "registry error",
            Self::PackageNotFound { .. } => "package not found",
            Self::AuthenticationFailed { .. } => "authentication failed",
            Self::RegistryTimeout { .. } => "registry timeout",
            Self::InvalidResponse { .. } => "invalid response",
            Self::BackupFailed { .. } => "backup failed",
            Self::NoBackup { .. } => "no backup",
            Self::RollbackFailed { .. } => "rollback failed",
            Self::ApplyFailed { .. } => "apply failed",
            Self::NoUpgradesAvailable => "no upgrades available",
            Self::InvalidPackageName { .. } => "invalid package name",
            Self::InvalidVersionSpec { .. } => "invalid version spec",
            Self::InvalidVersion { .. } => "invalid version",
            Self::VersionComparisonFailed { .. } => "version comparison failed",
            Self::NpmrcParseError { .. } => "npmrc parse error",
            Self::FileSystemError { .. } => "filesystem error",
            Self::PackageJsonError { .. } => "package.json error",
            Self::DeprecatedPackage { .. } => "deprecated package",
            Self::ChangesetCreationFailed { .. } => "changeset creation failed",
            Self::InvalidConfig { .. } => "invalid config",
            Self::InvalidWorkspace { .. } => "invalid workspace",
            Self::NoPackagesFound { .. } => "no packages found",
            Self::ConcurrentModification { .. } => "concurrent modification",
            Self::NetworkError { .. } => "network error",
            Self::RateLimitExceeded { .. } => "rate limit exceeded",
            Self::MaxBackupsExceeded { .. } => "max backups exceeded",
            Self::BackupCorrupted { .. } => "backup corrupted",
        }
    }
}

impl UpgradeError {
    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some upgrade errors (like network errors or timeouts) might be
    /// recoverable through retry, while others (like invalid configurations) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::UpgradeError;
    ///
    /// let network_error = UpgradeError::NetworkError {
    ///     reason: "connection refused".to_string(),
    /// };
    /// assert!(network_error.is_transient());
    ///
    /// let invalid_config = UpgradeError::InvalidConfig {
    ///     reason: "missing registry url".to_string(),
    /// };
    /// assert!(!invalid_config.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::RegistryTimeout { .. }
                | Self::NetworkError { .. }
                | Self::FileSystemError { .. }
                | Self::ConcurrentModification { .. }
                | Self::RegistryError { .. }
        )
    }

    /// Returns whether this error is related to registry operations.
    ///
    /// This helper method identifies errors that originate from registry
    /// communication, useful for categorizing and handling registry errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::UpgradeError;
    ///
    /// let registry_error = UpgradeError::RegistryError {
    ///     package: "lodash".to_string(),
    ///     reason: "timeout".to_string(),
    /// };
    /// assert!(registry_error.is_registry_related());
    ///
    /// let backup_error = UpgradeError::BackupFailed {
    ///     path: std::path::PathBuf::from("/backup"),
    ///     reason: "disk full".to_string(),
    /// };
    /// assert!(!backup_error.is_registry_related());
    /// ```
    #[must_use]
    pub fn is_registry_related(&self) -> bool {
        matches!(
            self,
            Self::RegistryError { .. }
                | Self::PackageNotFound { .. }
                | Self::AuthenticationFailed { .. }
                | Self::RegistryTimeout { .. }
                | Self::InvalidResponse { .. }
                | Self::NetworkError { .. }
                | Self::RateLimitExceeded { .. }
        )
    }

    /// Returns whether this error is related to backup operations.
    ///
    /// This helper method identifies errors that occur during backup creation,
    /// management, or restoration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::UpgradeError;
    /// use std::path::PathBuf;
    ///
    /// let backup_error = UpgradeError::BackupFailed {
    ///     path: PathBuf::from("/backup"),
    ///     reason: "disk full".to_string(),
    /// };
    /// assert!(backup_error.is_backup_related());
    ///
    /// let registry_error = UpgradeError::RegistryError {
    ///     package: "lodash".to_string(),
    ///     reason: "timeout".to_string(),
    /// };
    /// assert!(!registry_error.is_backup_related());
    /// ```
    #[must_use]
    pub fn is_backup_related(&self) -> bool {
        matches!(
            self,
            Self::BackupFailed { .. }
                | Self::NoBackup { .. }
                | Self::RollbackFailed { .. }
                | Self::MaxBackupsExceeded { .. }
                | Self::BackupCorrupted { .. }
        )
    }

    /// Returns the alternative package recommendation for deprecated packages.
    ///
    /// This helper method extracts the alternative package name from
    /// `DeprecatedPackage` errors, if available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::UpgradeError;
    ///
    /// let error = UpgradeError::DeprecatedPackage {
    ///     package: "old-package".to_string(),
    ///     message: "Use new-package instead".to_string(),
    ///     alternative: Some("new-package".to_string()),
    /// };
    ///
    /// assert_eq!(error.alternative(), Some(&"new-package".to_string()));
    /// ```
    #[must_use]
    pub fn alternative(&self) -> Option<&String> {
        match self {
            Self::DeprecatedPackage { alternative, .. } => alternative.as_ref(),
            _ => None,
        }
    }
}
