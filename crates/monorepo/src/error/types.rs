//! Error types for the monorepo tools library
//!
//! This module provides a comprehensive error hierarchy that integrates errors from all base crates
//! (git, standard, and package tools) as well as monorepo-specific errors.

use thiserror::Error;
use sublime_package_tools::errors::{VersionError, DependencyResolutionError, PackageRegistryError, RegistryError, PackageError};

/// Main error type for monorepo tools operations
#[derive(Error, Debug)]
pub enum Error {
    /// Git-related errors from `sublime_git_tools`
    #[error("Git operation failed")]
    Git(#[from] sublime_git_tools::RepoError),

    /// Standard tools errors (filesystem, command, monorepo detection)
    #[error("Standard tools error")]
    Standard(#[from] sublime_standard_tools::error::Error),

    /// Package tools errors (version, dependency, registry)
    #[error("Package tools error: {0}")]
    Package(String),

    /// Version-related errors
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    /// Dependency resolution errors
    #[error("Dependency resolution error: {0}")]
    DependencyResolution(#[from] DependencyResolutionError),

    /// Package registry errors
    #[error("Package registry error: {0}")]
    PackageRegistry(#[from] PackageRegistryError),

    /// Registry management errors
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Analysis errors
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Versioning workflow errors
    #[error("Versioning error: {0}")]
    Versioning(String),

    /// Task execution errors
    #[error("Task execution error: {0}")]
    Task(String),

    /// Changeset management errors
    #[error("Changeset error: {0}")]
    Changeset(String),

    /// Hook execution errors
    #[error("Hook error: {0}")]
    Hook(String),

    /// Changelog generation errors
    #[error("Changelog error: {0}")]
    Changelog(String),

    /// Plugin system errors
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Workflow orchestration errors
    #[error("Workflow error: {0}")]
    Workflow(String),

    /// Project initialization errors
    #[error("Project initialization error: {0}")]
    ProjectInit(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic errors with custom messages
    #[error("{0}")]
    Generic(String),
}

/// Result type alias for monorepo tools operations
pub type Result<T> = std::result::Result<T, Error>;

impl From<PackageError> for Error {
    fn from(err: PackageError) -> Self {
        Error::Package(err.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Generic(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Generic(msg.to_string())
    }
}

impl Error {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Create an analysis error
    pub fn analysis(msg: impl Into<String>) -> Self {
        Error::Analysis(msg.into())
    }

    /// Create a versioning error
    pub fn versioning(msg: impl Into<String>) -> Self {
        Error::Versioning(msg.into())
    }

    /// Create a task error
    pub fn task(msg: impl Into<String>) -> Self {
        Error::Task(msg.into())
    }

    /// Create a changeset error
    pub fn changeset(msg: impl Into<String>) -> Self {
        Error::Changeset(msg.into())
    }

    /// Create a package error
    pub fn package(msg: impl Into<String>) -> Self {
        Error::Package(msg.into())
    }

    /// Create a hook error
    pub fn hook(msg: impl Into<String>) -> Self {
        Error::Hook(msg.into())
    }

    /// Create a changelog error
    pub fn changelog(msg: impl Into<String>) -> Self {
        Error::Changelog(msg.into())
    }

    /// Create a plugin error
    pub fn plugin(msg: impl Into<String>) -> Self {
        Error::Plugin(msg.into())
    }

    /// Create a workflow error
    pub fn workflow(msg: impl Into<String>) -> Self {
        Error::Workflow(msg.into())
    }

    /// Create a project initialization error
    pub fn project_init(msg: impl Into<String>) -> Self {
        Error::ProjectInit(msg.into())
    }

    /// Create a generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Error::Generic(msg.into())
    }

    /// Create a package not found error
    #[must_use]
    pub fn package_not_found(package_name: &str) -> Self {
        Error::Package(format!("Package not found: {package_name}"))
    }
}
