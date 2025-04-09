//! Error types for versioning operations.
//!
//! This module defines the error types and result type aliases used throughout
//! the versioning system. It provides structured error reporting for various
//! failure modes that can occur during version operations.

use crate::{ChangeError, WorkspaceError};
use std::io;
use sublime_package_tools::{DependencyResolutionError, PackageError, VersionError};
use thiserror::Error;

/// Errors that can occur during versioning operations.
///
/// This enum represents all possible errors that might occur when working with
/// versioning functionality, including workspace issues, change tracking problems,
/// version parsing errors, and I/O failures.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::VersioningError;
///
/// // Create a specific error
/// let error = VersioningError::NoChangesFound("ui-components".to_string());
///
/// // Get error type as string
/// assert_eq!(error.as_ref(), "NoChangesFound");
/// ```
#[derive(Debug, Error)]
pub enum VersioningError {
    /// Failed to load or parse workspace data
    #[error("Workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),

    /// Error in change tracking operations
    #[error("Change tracking error: {0}")]
    ChangeError(#[from] ChangeError),

    /// Error in version handling
    #[error("Version error: {0}")]
    VersionError(#[from] VersionError),

    /// Error with package operations
    #[error("Package error: {0}")]
    PackageError(#[from] PackageError),

    /// Error with dependency resolution
    #[error("Dependency resolution error: {0}")]
    DependencyResolutionError(#[from] DependencyResolutionError),

    /// IO error during changelog generation
    #[error("IO error during changelog operation: {0}")]
    IoError(#[from] io::Error),

    /// No changes found for version bump
    #[error("No changes found for package {0}")]
    NoChangesFound(String),

    /// Invalid bump strategy
    #[error("Invalid bump strategy: {0}")]
    InvalidBumpStrategy(String),

    /// Missing package in workspace
    #[error("Package {0} not found in workspace")]
    PackageNotFound(String),

    /// No version suggestion possible
    #[error("Cannot suggest version for package {0}: {1}")]
    NoVersionSuggestion(String, String),

    /// Cyclic dependencies prevent synchronized versioning
    #[error("Cyclic dependencies prevent synchronized versioning: {0}")]
    CyclicDependencies(String),

    /// No version file found
    #[error("No version file found at {0}")]
    NoVersionFile(String),
}

impl AsRef<str> for VersioningError {
    /// Gets a string representation of the error type.
    ///
    /// This method provides a simple way to get the error variant name without
    /// the associated values, useful for categorizing errors.
    ///
    /// # Returns
    ///
    /// A string slice representing the error type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::VersioningError;
    ///
    /// let error = VersioningError::PackageNotFound("ui".to_string());
    /// assert_eq!(error.as_ref(), "PackageNotFound");
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            VersioningError::WorkspaceError(_) => "WorkspaceError",
            VersioningError::ChangeError(_) => "ChangeError",
            VersioningError::VersionError(_) => "VersionError",
            VersioningError::PackageError(_) => "PackageError",
            VersioningError::DependencyResolutionError(_) => "DependencyResolutionError",
            VersioningError::IoError(_) => "IoError",
            VersioningError::NoChangesFound(_) => "NoChangesFound",
            VersioningError::InvalidBumpStrategy(_) => "InvalidBumpStrategy",
            VersioningError::PackageNotFound(_) => "PackageNotFound",
            VersioningError::NoVersionSuggestion(_, _) => "NoVersionSuggestion",
            VersioningError::CyclicDependencies(_) => "CyclicDependencies",
            VersioningError::NoVersionFile(_) => "NoVersionFile",
        }
    }
}

/// Type alias for versioning operation results.
///
/// This type alias is used throughout the versioning system for functions
/// that can return errors.
pub type VersioningResult<T> = Result<T, VersioningError>;
