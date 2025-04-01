//! Error types for versioning operations.

use std::io;
use thiserror::Error;

use crate::{ChangeError, WorkspaceError};
use sublime_package_tools::{DependencyResolutionError, PackageError, VersionError};

/// Errors that can occur during versioning operations.
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
pub type VersioningResult<T> = Result<T, VersioningError>;
