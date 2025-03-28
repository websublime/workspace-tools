use std::io;
use std::path::PathBuf;
use sublime_git_tools::RepoError;
use sublime_package_tools::{DependencyResolutionError, PackageError, VersionError};
use thiserror::Error;

/// Errors that can occur during workspace operations.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// Failed to find workspace root
    #[error("Could not find workspace root")]
    RootNotFound,

    /// Failed to read workspace manifest
    #[error("Failed to read workspace manifest at {path}: {error}")]
    ManifestReadError { path: PathBuf, error: io::Error },

    /// Failed to parse workspace manifest
    #[error("Failed to parse workspace manifest at {path}: {error}")]
    ManifestParseError { path: PathBuf, error: serde_json::Error },

    /// No packages found in workspace
    #[error("No packages found in workspace at {0}")]
    NoPackagesFound(PathBuf),

    /// Package not found in workspace
    #[error("Package '{0}' not found in workspace")]
    PackageNotFound(String),

    /// Git repository error
    #[error("Git repository error: {0}")]
    GitError(#[from] RepoError),

    /// Package error
    #[error("Package error: {0}")]
    PackageError(#[from] PackageError),

    /// Version error
    #[error("Version error: {0}")]
    VersionError(#[from] VersionError),

    /// Dependency resolution error
    #[error("Dependency resolution error: {0}")]
    DependencyResolutionError(#[from] DependencyResolutionError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Invalid workspace configuration
    #[error("Invalid workspace configuration: {0}")]
    InvalidConfiguration(String),

    /// Cycle in workspace packages
    #[error("Cycle detected in workspace packages: {0}")]
    CycleDetected(String),
}

impl AsRef<str> for WorkspaceError {
    fn as_ref(&self) -> &str {
        match self {
            WorkspaceError::RootNotFound => "RootNotFound",
            WorkspaceError::ManifestReadError { path: _, error: _ } => "ManifestReadError",
            WorkspaceError::ManifestParseError { path: _, error: _ } => "ManifestParseError",
            WorkspaceError::NoPackagesFound(_) => "NoPackagesFound",
            WorkspaceError::PackageNotFound(_) => "PackageNotFound",
            WorkspaceError::GitError(_) => "GitError",
            WorkspaceError::PackageError(_) => "PackageError",
            WorkspaceError::VersionError(_) => "VersionError",
            WorkspaceError::DependencyResolutionError(_) => "DependencyResolutionError",
            WorkspaceError::IoError(_) => "IoError",
            WorkspaceError::InvalidConfiguration(_) => "InvalidConfiguration",
            WorkspaceError::CycleDetected(_) => "CycleDetected",
        }
    }
}
