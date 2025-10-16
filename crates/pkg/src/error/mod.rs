//! Error types and error handling utilities for package tools operations.
//!
//! **What**: Provides a comprehensive error hierarchy for all package tools operations,
//! including detailed error contexts, error recovery strategies, and result type aliases.
//!
//! **How**: This module defines domain-specific error types for each major operation area
//! (changesets, versioning, dependencies, upgrades, changelog, audit), with rich context
//! information and support for error chaining and recovery.
//!
//! **Why**: To provide clear, actionable error messages that help users understand what
//! went wrong and how to fix it, while enabling robust error handling and recovery in
//! automated workflows.
//!
//! # Features
//!
//! - **Hierarchical Errors**: Structured error types organized by operation domain
//! - **Rich Context**: Detailed error information including paths, operations, and reasons
//! - **Error Chaining**: Support for nested errors to preserve error context
//! - **Error Conversion**: Automatic conversion from standard library and dependency errors
//! - **Display Formatting**: Human-readable error messages
//! - **Debug Information**: Detailed debug output for troubleshooting
//!
//! # Error Categories
//!
//! ## ConfigError
//! Errors related to configuration loading, parsing, and validation.
//!
//! ## VersionError
//! Errors related to version resolution, propagation, and application.
//!
//! ## ChangesetError
//! Errors related to changeset operations (create, load, update, archive).
//!
//! ## ChangesError
//! Errors related to changes analysis and file-to-package mapping.
//!
//! ## ChangelogError
//! Errors related to changelog generation and parsing.
//!
//! ## UpgradeError
//! Errors related to dependency upgrade detection and application.
//!
//! ## AuditError
//! Errors related to audits and health checks.
//!
//! # Example
//!
//! ```rust
//! use sublime_pkg_tools::error::{Error, Result};
//!
//! fn process_package() -> Result<()> {
//!     // Operation that might fail
//!     Ok(())
//! }
//!
//! match process_package() {
//!     Ok(_) => println!("Success!"),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! # Domain-Specific Results
//!
//! Each error module provides a result type alias for convenience:
//!
//! ```rust
//! use sublime_pkg_tools::error::{
//!     ConfigResult, VersionResult, ChangesetResult,
//!     ChangesResult, ChangelogResult, UpgradeResult, AuditResult
//! };
//!
//! fn load_config() -> ConfigResult<String> {
//!     Ok("config".to_string())
//! }
//!
//! fn resolve_version() -> VersionResult<String> {
//!     Ok("1.0.0".to_string())
//! }
//! ```
//!
//! # Error Conversion
//!
//! Errors from internal crates (`sublime_standard_tools`, `sublime_git_tools`) are
//! automatically converted to the appropriate domain error:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::error::{Error, Result};
//! use sublime_standard_tools::filesystem::FileSystemManager;
//!
//! async fn read_package_json() -> Result<String> {
//!     let fs = FileSystemManager::new();
//!     // FileSystemError automatically converts to Error::FileSystem
//!     let content = fs.read_file_string("package.json").await?;
//!     Ok(content)
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

// Re-export all domain-specific error types and result aliases
pub use self::audit::{AuditError, AuditResult};
pub use self::changelog::{ChangelogError, ChangelogResult};
pub use self::changes::{ChangesError, ChangesResult};
pub use self::changeset::{ChangesetError, ChangesetResult};
pub use self::config::{ConfigError, ConfigResult};
pub use self::upgrade::{UpgradeError, UpgradeResult};
pub use self::version::{VersionError, VersionResult};

// Re-export context and recovery types
pub use self::context::{ErrorContext, WithContext};
pub use self::recovery::{
    ErrorRecoveryManager, LogLevel, RecoveryResult, RecoveryStats, RecoveryStrategy,
};

// Domain-specific error modules
pub mod audit;
pub mod changelog;
pub mod changes;
pub mod changeset;
pub mod config;
pub mod upgrade;
pub mod version;

// Error handling utilities
pub mod context;
pub mod recovery;

#[cfg(test)]
mod tests;

/// Main error type for package tools operations.
///
/// This enum provides a unified error type that encompasses all possible errors
/// that can occur during package tools operations. It includes domain-specific
/// errors as well as conversions from external dependencies.
///
/// # Examples
///
/// ## Handling errors by category
///
/// ```rust
/// use sublime_pkg_tools::error::Error;
///
/// fn handle_error(error: Error) {
///     match error {
///         Error::Config(e) => eprintln!("Configuration error: {}", e),
///         Error::Version(e) => eprintln!("Version error: {}", e),
///         Error::Changeset(e) => eprintln!("Changeset error: {}", e),
///         Error::Changes(e) => eprintln!("Changes analysis error: {}", e),
///         Error::Changelog(e) => eprintln!("Changelog error: {}", e),
///         Error::Upgrade(e) => eprintln!("Upgrade error: {}", e),
///         Error::Audit(e) => eprintln!("Audit error: {}", e),
///         Error::FileSystem(e) => eprintln!("Filesystem error: {}", e),
///         Error::Git(e) => eprintln!("Git error: {}", e),
///         Error::IO(e) => eprintln!("I/O error: {}", e),
///         Error::Json(e) => eprintln!("JSON error: {}", e),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::Error;
///
/// let error = Error::Config(
///     sublime_pkg_tools::error::ConfigError::NotFound {
///         path: std::path::PathBuf::from("config.toml"),
///     }
/// );
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("configuration"));
/// ```
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error.
    ///
    /// This variant wraps errors from configuration loading, parsing, and validation.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Version resolution or propagation error.
    ///
    /// This variant wraps errors from version resolution, dependency propagation,
    /// and version application operations.
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    /// Changeset operation error.
    ///
    /// This variant wraps errors from changeset creation, loading, storage, and archiving.
    #[error("Changeset error: {0}")]
    Changeset(#[from] ChangesetError),

    /// Changes analysis error.
    ///
    /// This variant wraps errors from changes analysis, file mapping, and commit range operations.
    #[error("Changes analysis error: {0}")]
    Changes(#[from] ChangesError),

    /// Changelog operation error.
    ///
    /// This variant wraps errors from changelog generation, parsing, and formatting.
    #[error("Changelog error: {0}")]
    Changelog(#[from] ChangelogError),

    /// Upgrade operation error.
    ///
    /// This variant wraps errors from dependency upgrade detection and application.
    #[error("Upgrade error: {0}")]
    Upgrade(#[from] UpgradeError),

    /// Audit operation error.
    ///
    /// This variant wraps errors from audits, health checks, and dependency analysis.
    #[error("Audit error: {0}")]
    Audit(#[from] AuditError),

    /// Filesystem operation error from sublime_standard_tools.
    ///
    /// This variant wraps errors from filesystem operations provided by the
    /// sublime_standard_tools crate.
    #[error("Filesystem error: {0}")]
    FileSystem(String),

    /// Git operation error from sublime_git_tools.
    ///
    /// This variant wraps errors from git operations provided by the
    /// sublime_git_tools crate.
    #[error("Git error: {0}")]
    Git(String),

    /// Standard I/O error.
    ///
    /// This variant wraps errors from standard library I/O operations.
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    ///
    /// This variant wraps errors from JSON parsing and serialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AsRef<str> for Error {
    /// Returns a string representation of the error category.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{Error, ConfigError};
    /// use std::path::PathBuf;
    ///
    /// let error = Error::Config(ConfigError::NotFound {
    ///     path: PathBuf::from("config.toml"),
    /// });
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("configuration"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::Config(e) => e.as_ref(),
            Self::Version(e) => e.as_ref(),
            Self::Changeset(e) => e.as_ref(),
            Self::Changes(e) => e.as_ref(),
            Self::Changelog(e) => e.as_ref(),
            Self::Upgrade(e) => e.as_ref(),
            Self::Audit(e) => e.as_ref(),
            Self::FileSystem(_) => "filesystem error",
            Self::Git(_) => "git error",
            Self::IO(_) => "io error",
            Self::Json(_) => "json error",
        }
    }
}

/// Result type alias for package tools operations.
///
/// This type alias simplifies error handling in package tools functions
/// by defaulting to `Error` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::Result;
///
/// fn process_package() -> Result<String> {
///     Ok("success".to_string())
/// }
///
/// # fn main() {
/// match process_package() {
///     Ok(result) => println!("Result: {}", result),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Converts sublime_standard_tools FileSystemError to our Error type.
///
/// This implementation enables seamless error propagation from standard tools
/// filesystem operations.
impl From<sublime_standard_tools::error::FileSystemError> for Error {
    fn from(error: sublime_standard_tools::error::FileSystemError) -> Self {
        Self::FileSystem(error.to_string())
    }
}

/// Converts sublime_standard_tools ConfigError to our ConfigError type.
///
/// This implementation maps configuration errors from standard tools to our
/// domain-specific configuration errors.
impl From<sublime_standard_tools::error::ConfigError> for ConfigError {
    fn from(error: sublime_standard_tools::error::ConfigError) -> Self {
        match error {
            sublime_standard_tools::error::ConfigError::FileNotFound { path } => {
                ConfigError::NotFound { path }
            }
            sublime_standard_tools::error::ConfigError::FileReadError { path, message } => {
                ConfigError::ParseError { path, reason: message }
            }
            sublime_standard_tools::error::ConfigError::FileWriteError { path, message } => {
                ConfigError::InvalidPath { path, reason: message }
            }
            sublime_standard_tools::error::ConfigError::ParseError { format, message } => {
                ConfigError::InvalidConfig { message: format!("{}: {}", format, message) }
            }
            sublime_standard_tools::error::ConfigError::SerializeError { format, message } => {
                ConfigError::InvalidConfig {
                    message: format!("serialization {}: {}", format, message),
                }
            }
            sublime_standard_tools::error::ConfigError::ValidationError { message } => {
                ConfigError::ValidationFailed { errors: vec![message] }
            }
            sublime_standard_tools::error::ConfigError::EnvironmentError { message } => {
                ConfigError::EnvVarError { var_name: "UNKNOWN".to_string(), reason: message }
            }
            sublime_standard_tools::error::ConfigError::TypeError { expected, actual } => {
                ConfigError::InvalidFieldType { field: "unknown".to_string(), expected, actual }
            }
            sublime_standard_tools::error::ConfigError::KeyNotFound { key } => {
                ConfigError::MissingField { field: key }
            }
            sublime_standard_tools::error::ConfigError::MergeConflict { message } => {
                ConfigError::MergeConflict { field: "unknown".to_string(), reason: message }
            }
            sublime_standard_tools::error::ConfigError::ProviderError { provider, message } => {
                ConfigError::InvalidConfig {
                    message: format!("provider '{}': {}", provider, message),
                }
            }
            sublime_standard_tools::error::ConfigError::Other(msg) => {
                ConfigError::InvalidConfig { message: msg }
            }
        }
    }
}

/// Converts sublime_git_tools RepoError to our Error type.
///
/// This implementation enables seamless error propagation from git tools
/// operations.
impl From<sublime_git_tools::RepoError> for Error {
    fn from(error: sublime_git_tools::RepoError) -> Self {
        Self::Git(error.to_string())
    }
}

/// Converts sublime_git_tools RepoError to ChangesError for git-related operations.
///
/// This implementation provides more specific error context for git operations
/// during changes analysis.
impl From<sublime_git_tools::RepoError> for ChangesError {
    fn from(error: sublime_git_tools::RepoError) -> Self {
        ChangesError::GitError { operation: "git operation".to_string(), reason: error.to_string() }
    }
}

/// Converts sublime_git_tools RepoError to ChangelogError for git-related operations.
///
/// This implementation provides more specific error context for git operations
/// during changelog generation.
impl From<sublime_git_tools::RepoError> for ChangelogError {
    fn from(error: sublime_git_tools::RepoError) -> Self {
        ChangelogError::GitError {
            operation: "git operation".to_string(),
            reason: error.to_string(),
        }
    }
}

/// Converts sublime_git_tools RepoError to ChangesetError for git-related operations.
///
/// This implementation provides more specific error context for git operations
/// during changeset management.
impl From<sublime_git_tools::RepoError> for ChangesetError {
    fn from(error: sublime_git_tools::RepoError) -> Self {
        ChangesetError::GitError {
            operation: "git operation".to_string(),
            reason: error.to_string(),
        }
    }
}

impl Error {
    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some errors (like network or filesystem issues) might be recoverable through
    /// retry, while others (like validation errors) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{Error, ChangesError};
    /// use std::path::PathBuf;
    ///
    /// let fs_error = Error::Changes(ChangesError::FileSystemError {
    ///     path: PathBuf::from("package.json"),
    ///     reason: "temporary lock".to_string(),
    /// });
    /// assert!(fs_error.is_transient());
    ///
    /// let invalid_error = Error::Config(
    ///     sublime_pkg_tools::error::ConfigError::InvalidConfig {
    ///         message: "invalid value".to_string(),
    ///     }
    /// );
    /// assert!(!invalid_error.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Version(e) => e.is_recoverable(),
            Self::Changeset(e) => e.is_transient(),
            Self::Changes(e) => e.is_transient(),
            Self::Changelog(e) => e.is_transient(),
            Self::Upgrade(e) => e.is_transient(),
            Self::Audit(e) => e.is_transient(),
            Self::FileSystem(_) | Self::Git(_) | Self::IO(_) => true,
            Self::Config(_) | Self::Json(_) => false,
        }
    }

    /// Creates a filesystem error with a path and reason.
    ///
    /// This helper method provides a convenient way to create filesystem errors
    /// with consistent formatting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::Error;
    /// use std::path::PathBuf;
    ///
    /// let error = Error::filesystem_error(
    ///     PathBuf::from("package.json"),
    ///     "file not found"
    /// );
    ///
    /// assert!(error.to_string().contains("package.json"));
    /// ```
    #[must_use]
    pub fn filesystem_error(path: PathBuf, reason: &str) -> Self {
        Self::FileSystem(format!("Filesystem error at '{}': {}", path.display(), reason))
    }

    /// Creates a git error with an operation description and reason.
    ///
    /// This helper method provides a convenient way to create git errors
    /// with consistent formatting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::Error;
    ///
    /// let error = Error::git_error("fetch commits", "network timeout");
    ///
    /// assert!(error.to_string().contains("fetch commits"));
    /// ```
    #[must_use]
    pub fn git_error(operation: &str, reason: &str) -> Self {
        Self::Git(format!("Git operation '{}' failed: {}", operation, reason))
    }
}
