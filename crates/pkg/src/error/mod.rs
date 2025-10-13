//! Error handling module for sublime_pkg_tools.
//!
//! This module provides comprehensive error types and utilities for handling
//! various failure scenarios in package management operations. It follows
//! the standard error handling patterns established by sublime_standard_tools
//! and provides detailed context for debugging and user feedback.
//!
//! # What
//!
//! Defines all error types used throughout the package tools crate, including:
//! - Version-related errors (parsing, resolution, conflicts)
//! - Changeset errors (creation, validation, application)
//! - Registry errors (network, authentication, publishing)
//! - Dependency errors (circular dependencies, resolution failures)
//! - Configuration errors (invalid settings, missing files)
//!
//! # How
//!
//! Uses `thiserror` for ergonomic error definitions with automatic trait
//! implementations. Integrates with sublime_standard_tools error types
//! for consistent error handling across the ecosystem.
//!
//! # Why
//!
//! Centralized error handling ensures consistent error messages, proper
//! error propagation, and detailed context for troubleshooting. This
//! improves developer experience and system reliability.

use std::path::PathBuf;
use sublime_git_tools::RepoError;
use sublime_standard_tools::error::Error as StandardError;
use thiserror::Error;

/// Main error type for package management operations.
///
/// This enum covers all possible error scenarios that can occur during
/// package management operations, from version resolution to release
/// execution.
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

/// Version-related error types.
///
/// Covers all version management scenarios including parsing,
/// resolution, and version conflict detection.
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

/// Changeset-related error types.
///
/// Handles errors in changeset creation, validation, storage,
/// and application processes.
#[derive(Error, Debug)]
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
        /// Path to the invalid changeset
        path: PathBuf,
        /// Reason for invalidity
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
        /// Existing changeset identifier
        changeset_id: String,
    },

    /// Failed to create changeset
    #[error("Failed to create changeset for branch '{branch}': {reason}")]
    CreationFailed {
        /// Branch name
        branch: String,
        /// Failure reason
        reason: String,
    },

    /// Failed to apply changeset
    #[error("Failed to apply changeset '{changeset_id}': {reason}")]
    ApplicationFailed {
        /// Changeset identifier
        changeset_id: String,
        /// Failure reason
        reason: String,
    },

    /// Changeset history operation failed
    #[error("Changeset history operation failed: {operation} - {reason}")]
    HistoryOperationFailed {
        /// The history operation
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
}

/// Registry-related error types.
///
/// Covers NPM registry operations including authentication,
/// publishing, and package information retrieval.
#[derive(Error, Debug)]
pub enum RegistryError {
    /// Authentication failed
    #[error("Registry authentication failed for '{registry}': {reason}")]
    AuthenticationFailed {
        /// Registry URL
        registry: String,
        /// Failure reason
        reason: String,
    },

    /// Package not found in registry
    #[error("Package '{package}' not found in registry '{registry}'")]
    PackageNotFound {
        /// Package name
        package: String,
        /// Registry URL
        registry: String,
    },

    /// Publishing failed
    #[error("Failed to publish package '{package}' to '{registry}': {reason}")]
    PublishFailed {
        /// Package name
        package: String,
        /// Registry URL
        registry: String,
        /// Failure reason
        reason: String,
    },

    /// Network operation failed
    #[error("Network operation failed for registry '{registry}': {reason}")]
    NetworkFailed {
        /// Registry URL
        registry: String,
        /// Failure reason
        reason: String,
    },

    /// Invalid registry configuration
    #[error("Invalid registry configuration: {reason}")]
    InvalidConfig {
        /// Configuration error reason
        reason: String,
    },

    /// Package version already exists
    #[error("Package '{package}@{version}' already exists in registry '{registry}'")]
    VersionAlreadyExists {
        /// Package name
        package: String,
        /// Version
        version: String,
        /// Registry URL
        registry: String,
    },

    /// Registry timeout
    #[error("Registry operation timed out for '{registry}' after {timeout_ms}ms")]
    Timeout {
        /// Registry URL
        registry: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
}

/// Dependency-related error types.
///
/// Handles dependency graph analysis, circular dependency
/// detection, and dependency resolution failures.
#[derive(Error, Debug)]
pub enum DependencyError {
    /// Circular dependency detected
    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency {
        /// The packages involved in the cycle
        cycle: Vec<String>,
    },

    /// Dependency resolution failed
    #[error("Failed to resolve dependencies for package '{package}': {reason}")]
    ResolutionFailed {
        /// Package name
        package: String,
        /// Failure reason
        reason: String,
    },

    /// Missing dependency
    #[error("Missing dependency '{dependency}' for package '{package}'")]
    MissingDependency {
        /// Package name
        package: String,
        /// Missing dependency name
        dependency: String,
    },

    /// Invalid dependency specification
    #[error("Invalid dependency specification '{spec}' for package '{package}': {reason}")]
    InvalidSpecification {
        /// Package name
        package: String,
        /// Invalid dependency spec
        spec: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Dependency graph construction failed
    #[error("Failed to construct dependency graph: {reason}")]
    GraphConstructionFailed {
        /// Failure reason
        reason: String,
    },

    /// Dependency propagation failed
    #[error("Failed to propagate dependency updates: {reason}")]
    PropagationFailed {
        /// Failure reason
        reason: String,
    },

    /// Maximum propagation depth exceeded
    #[error("Maximum dependency propagation depth ({max_depth}) exceeded")]
    MaxDepthExceeded {
        /// Maximum allowed depth
        max_depth: u32,
    },
}

/// Conventional commit parsing error types.
///
/// Handles errors in parsing and validating conventional commits
/// according to the conventional commits specification.
#[derive(Error, Debug)]
pub enum ConventionalCommitError {
    /// Invalid commit format
    #[error("Invalid conventional commit format: '{commit}' - {reason}")]
    InvalidFormat {
        /// The invalid commit message
        commit: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Unknown commit type
    #[error("Unknown commit type '{commit_type}' in commit: '{commit}'")]
    UnknownType {
        /// Unknown commit type
        commit_type: String,
        /// Full commit message
        commit: String,
    },

    /// Commit parsing failed
    #[error("Failed to parse commit '{commit}': {reason}")]
    ParseFailed {
        /// Commit message
        commit: String,
        /// Failure reason
        reason: String,
    },

    /// Breaking change detection failed
    #[error("Failed to detect breaking changes in commit '{commit}': {reason}")]
    BreakingChangeDetectionFailed {
        /// Commit message
        commit: String,
        /// Failure reason
        reason: String,
    },
}

/// Release management error types.
///
/// Covers errors in release planning, execution, and post-release operations.
#[derive(Error, Debug)]
pub enum ReleaseError {
    /// Release planning failed
    #[error("Release planning failed: {reason}")]
    PlanningFailed {
        /// Failure reason
        reason: String,
    },

    /// Release execution failed
    #[error("Release execution failed for environment '{environment}': {reason}")]
    ExecutionFailed {
        /// Target environment
        environment: String,
        /// Failure reason
        reason: String,
    },

    /// Package release failed
    #[error("Failed to release package '{package}' to '{environment}': {reason}")]
    PackageReleaseFailed {
        /// Package name
        package: String,
        /// Target environment
        environment: String,
        /// Failure reason
        reason: String,
    },

    /// Tag creation failed
    #[error("Failed to create tag '{tag}': {reason}")]
    TagCreationFailed {
        /// Tag name
        tag: String,
        /// Failure reason
        reason: String,
    },

    /// Dry run failed
    #[error("Dry run failed: {reason}")]
    DryRunFailed {
        /// Failure reason
        reason: String,
    },

    /// Release strategy not supported
    #[error("Release strategy '{strategy}' not supported")]
    StrategyNotSupported {
        /// Unsupported strategy
        strategy: String,
    },

    /// Release rollback failed
    #[error("Release rollback failed: {reason}")]
    RollbackFailed {
        /// Failure reason
        reason: String,
    },
}

/// Changelog generation error types.
///
/// Handles errors in changelog creation, formatting, and file operations.
#[derive(Error, Debug)]
pub enum ChangelogError {
    /// Changelog generation failed
    #[error("Changelog generation failed: {reason}")]
    GenerationFailed {
        /// Failure reason
        reason: String,
    },

    /// Template not found
    #[error("Changelog template not found: {template_path}")]
    TemplateNotFound {
        /// Template file path
        template_path: PathBuf,
    },

    /// Template rendering failed
    #[error("Template rendering failed: {reason}")]
    TemplateRenderingFailed {
        /// Failure reason
        reason: String,
    },

    /// Changelog file write failed
    #[error("Failed to write changelog to '{path}': {reason}")]
    WriteFileFailed {
        /// Changelog file path
        path: PathBuf,
        /// Failure reason
        reason: String,
    },

    /// Invalid changelog format
    #[error("Invalid changelog format in '{path}': {reason}")]
    InvalidFormat {
        /// Changelog file path
        path: PathBuf,
        /// Reason for invalidity
        reason: String,
    },
}

/// Configuration error types specific to package tools.
///
/// Extends standard configuration errors with package-specific scenarios.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid package tools configuration
    #[error("Invalid package tools configuration: {field} - {reason}")]
    InvalidPackageConfig {
        /// Configuration field
        field: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Environment configuration invalid
    #[error("Invalid environment configuration: {environment} - {reason}")]
    InvalidEnvironmentConfig {
        /// Environment name
        environment: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Registry configuration invalid
    #[error("Invalid registry configuration: {registry} - {reason}")]
    InvalidRegistryConfig {
        /// Registry name/URL
        registry: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Version strategy configuration invalid
    #[error("Invalid version strategy configuration: {strategy} - {reason}")]
    InvalidVersionStrategy {
        /// Strategy name
        strategy: String,
        /// Reason for invalidity
        reason: String,
    },
}

/// Error type for parsing commit types from strings.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommitTypeParseError {
    /// Empty string provided
    #[error("Empty commit type string")]
    Empty,
    /// Invalid commit type format
    #[error("Invalid commit type format: '{0}'")]
    InvalidFormat(String),
}

/// Result type alias for package management operations.
///
/// This is a convenience type alias that uses `PackageError` as the error type
/// for all package management operations.
///
/// # Examples
///
/// ```rust
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
pub type PackageResult<T> = Result<T, PackageError>;

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
    #[must_use]
    pub fn is_changeset_error(&self) -> bool {
        matches!(self, Self::Changeset(_))
    }

    /// Checks if this error is registry-related.
    #[must_use]
    pub fn is_registry_error(&self) -> bool {
        matches!(self, Self::Registry(_))
    }

    /// Checks if this error is dependency-related.
    #[must_use]
    pub fn is_dependency_error(&self) -> bool {
        matches!(self, Self::Dependency(_))
    }

    /// Checks if this error is release-related.
    #[must_use]
    pub fn is_release_error(&self) -> bool {
        matches!(self, Self::Release(_))
    }
}

#[cfg(test)]
mod tests;
