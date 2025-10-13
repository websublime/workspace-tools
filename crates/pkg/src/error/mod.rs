//! Error handling module for sublime_pkg_tools.
//!
//! This module provides comprehensive error types and utilities for handling
//! various failure scenarios in package management operations. It follows
//! the standard error handling patterns established by sublime_standard_tools
//! and provides detailed context for debugging and user feedback.
//!
//! # What
//!
//! Defines all error types used throughout the package tools crate, organized
//! by domain:
//! - Version-related errors (parsing, resolution, conflicts)
//! - Changeset errors (creation, validation, application)
//! - Registry errors (network, authentication, publishing)
//! - Dependency errors (circular dependencies, resolution failures)
//! - Release errors (planning, execution, rollback)
//! - Changelog errors (generation, template, file operations)
//! - Configuration errors (invalid settings, missing configurations)
//! - Conventional commit errors (parsing, type validation)
//!
//! # How
//!
//! Uses `thiserror` for ergonomic error definitions with automatic trait
//! implementations. Integrates with sublime_standard_tools and sublime_git_tools
//! error types for consistent error handling across the ecosystem. Each domain
//! has its own module with specific error types and utility functions.
//!
//! # Why
//!
//! Centralized error handling ensures consistent error messages, proper
//! error propagation, and detailed context for troubleshooting. The modular
//! structure improves maintainability and follows the patterns established
//! by other sublime tools.

mod changelog;
mod changeset;
mod config;
mod conventional;
mod dependency;
mod registry;
mod release;
mod version;

#[cfg(test)]
mod tests;

use std::result::Result as StdResult;
use sublime_git_tools::RepoError;
use sublime_standard_tools::error::Error as StandardError;
use thiserror::Error;

// Re-export error types from individual modules
pub use changelog::{ChangelogError, ChangelogResult};
pub use changeset::{ChangesetError, ChangesetResult};
pub use config::{ConfigError, ConfigResult};
pub use conventional::{CommitTypeParseError, ConventionalCommitError, ConventionalCommitResult};
pub use dependency::{DependencyError, DependencyResult};
pub use registry::{RegistryError, RegistryResult};
pub use release::{ReleaseError, ReleaseResult};
pub use version::{VersionError, VersionResult};

/// Main error type for package management operations.
///
/// This enum covers all possible error scenarios that can occur during
/// package management operations, from version resolution to release
/// execution. It aggregates domain-specific errors for unified error handling.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::PackageError;
///
/// fn handle_error(err: PackageError) {
///     match err {
///         PackageError::Version(v_err) => {
///             eprintln!("Version error: {}", v_err);
///         }
///         PackageError::Changeset(c_err) => {
///             eprintln!("Changeset error: {}", c_err);
///         }
///         _ => eprintln!("Other error: {}", err),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum PackageError {
    /// Version-related errors (parsing, resolution, conflicts)
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    /// Changeset-related errors (creation, validation, application)
    #[error("Changeset error: {0}")]
    Changeset(#[from] ChangesetError),

    /// Registry-related errors (publishing, fetching, authentication)
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Dependency-related errors (circular deps, resolution failures)
    #[error("Dependency error: {0}")]
    Dependency(#[from] DependencyError),

    /// Conventional commit parsing errors
    #[error("Conventional commit error: {0}")]
    ConventionalCommit(#[from] ConventionalCommitError),

    /// Release management errors
    #[error("Release error: {0}")]
    Release(#[from] ReleaseError),

    /// Changelog generation errors
    #[error("Changelog error: {0}")]
    Changelog(#[from] ChangelogError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Git operations errors
    #[error("Git error: {0}")]
    Git(#[from] RepoError),

    /// Standard tools errors
    #[error("Standard tools error: {0}")]
    Standard(#[from] StandardError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP client errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Generic operation errors with context
    #[error("Operation failed: {operation} - {reason}")]
    Operation {
        /// The operation that failed
        operation: String,
        /// The reason for failure
        reason: String,
    },
}

/// Result type alias for package management operations.
///
/// This is a convenience type alias that uses `PackageError` as the error type
/// for all package management operations.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::error::PackageResult;
///
/// fn parse_version(version_str: &str) -> PackageResult<semver::Version> {
///     semver::Version::parse(version_str)
///         .map_err(|e| PackageError::Version(VersionError::ParseFailed {
///             version: version_str.to_string(),
///             source: e,
///         }))
/// }
/// ```
pub type PackageResult<T> = StdResult<T, PackageError>;

impl PackageError {
    /// Creates an operation error with context.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation that failed
    /// * `reason` - The reason for failure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::PackageError;
    ///
    /// let error = PackageError::operation(
    ///     "create_changeset",
    ///     "No commits found since last release"
    /// );
    /// ```
    #[must_use]
    pub fn operation(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Operation { operation: operation.into(), reason: reason.into() }
    }

    /// Checks if this error is version-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, VersionError};
    ///
    /// let error = PackageError::Version(VersionError::InvalidFormat {
    ///     version: "invalid".to_string(),
    ///     reason: "Not semver".to_string(),
    /// });
    ///
    /// assert!(error.is_version_error());
    /// ```
    #[must_use]
    pub fn is_version_error(&self) -> bool {
        matches!(self, Self::Version(_))
    }

    /// Checks if this error is changeset-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, ChangesetError};
    /// use std::path::PathBuf;
    ///
    /// let error = PackageError::Changeset(ChangesetError::NotFound {
    ///     path: PathBuf::from("missing.json"),
    /// });
    ///
    /// assert!(error.is_changeset_error());
    /// ```
    #[must_use]
    pub fn is_changeset_error(&self) -> bool {
        matches!(self, Self::Changeset(_))
    }

    /// Checks if this error is registry-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, RegistryError};
    ///
    /// let error = PackageError::Registry(RegistryError::NetworkFailed {
    ///     registry: "https://registry.npmjs.org".to_string(),
    ///     reason: "Connection timeout".to_string(),
    /// });
    ///
    /// assert!(error.is_registry_error());
    /// ```
    #[must_use]
    pub fn is_registry_error(&self) -> bool {
        matches!(self, Self::Registry(_))
    }

    /// Checks if this error is dependency-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, DependencyError};
    ///
    /// let error = PackageError::Dependency(DependencyError::CircularDependency {
    ///     cycle: vec!["pkg-a".to_string(), "pkg-b".to_string()],
    /// });
    ///
    /// assert!(error.is_dependency_error());
    /// ```
    #[must_use]
    pub fn is_dependency_error(&self) -> bool {
        matches!(self, Self::Dependency(_))
    }

    /// Checks if this error is release-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, ReleaseError};
    ///
    /// let error = PackageError::Release(ReleaseError::PlanningFailed {
    ///     reason: "No changesets found".to_string(),
    /// });
    ///
    /// assert!(error.is_release_error());
    /// ```
    #[must_use]
    pub fn is_release_error(&self) -> bool {
        matches!(self, Self::Release(_))
    }

    /// Checks if this error is changelog-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, ChangelogError};
    ///
    /// let error = PackageError::Changelog(ChangelogError::GenerationFailed {
    ///     reason: "No releases found".to_string(),
    /// });
    ///
    /// assert!(error.is_changelog_error());
    /// ```
    #[must_use]
    pub fn is_changelog_error(&self) -> bool {
        matches!(self, Self::Changelog(_))
    }

    /// Checks if this error is configuration-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, ConfigError};
    ///
    /// let error = PackageError::Config(ConfigError::InvalidPackageConfig {
    ///     field: "strategy".to_string(),
    ///     reason: "Unknown strategy".to_string(),
    /// });
    ///
    /// assert!(error.is_config_error());
    /// ```
    #[must_use]
    pub fn is_config_error(&self) -> bool {
        matches!(self, Self::Config(_))
    }

    /// Checks if this error is conventional commit-related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, ConventionalCommitError};
    ///
    /// let error = PackageError::ConventionalCommit(ConventionalCommitError::InvalidFormat {
    ///     commit: "bad commit".to_string(),
    ///     reason: "Missing type".to_string(),
    /// });
    ///
    /// assert!(error.is_conventional_commit_error());
    /// ```
    #[must_use]
    pub fn is_conventional_commit_error(&self) -> bool {
        matches!(self, Self::ConventionalCommit(_))
    }

    /// Checks if this error is git-related.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::error::PackageError;
    /// use sublime_git_tools::RepoError;
    ///
    /// // Example would require git2 error construction
    /// let git_error = /* some RepoError */;
    /// let error = PackageError::Git(git_error);
    ///
    /// assert!(error.is_git_error());
    /// ```
    #[must_use]
    pub fn is_git_error(&self) -> bool {
        matches!(self, Self::Git(_))
    }

    /// Checks if this error is I/O related.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::PackageError;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let error = PackageError::Io(Error::new(ErrorKind::NotFound, "File not found"));
    ///
    /// assert!(error.is_io_error());
    /// ```
    #[must_use]
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    /// Gets the error category as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{PackageError, VersionError};
    ///
    /// let error = PackageError::Version(VersionError::InvalidFormat {
    ///     version: "invalid".to_string(),
    ///     reason: "Not semver".to_string(),
    /// });
    ///
    /// assert_eq!(error.category(), "version");
    /// ```
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::Version(_) => "version",
            Self::Changeset(_) => "changeset",
            Self::Registry(_) => "registry",
            Self::Dependency(_) => "dependency",
            Self::ConventionalCommit(_) => "conventional_commit",
            Self::Release(_) => "release",
            Self::Changelog(_) => "changelog",
            Self::Config(_) => "config",
            Self::Git(_) => "git",
            Self::Standard(_) => "standard",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::Http(_) => "http",
            Self::Operation { .. } => "operation",
        }
    }
}
