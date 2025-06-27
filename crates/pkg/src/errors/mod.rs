pub mod dependency;
pub mod package;
pub mod registry;
pub mod version;

use thiserror::Error;

pub use dependency::DependencyResolutionError;
pub use package::{PackageError, PackageRegistryError};
pub use registry::RegistryError;
pub use version::VersionError;

/// Main error type for package tools operations
/// 
/// This enum serves as a unified error type that aggregates all domain-specific
/// errors from the package tools crate into a single error type.
/// 
/// # Examples
/// 
/// ```
/// use sublime_package_tools::{Error, Result};
/// 
/// fn some_operation() -> Result<String> {
///     // This function can return any package tools error
///     // and it will be automatically converted to Error
///     Ok("success".to_string())
/// }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// Version-related errors
    #[error("Version error")]
    Version(#[from] VersionError),
    
    /// Dependency resolution errors
    #[error("Dependency resolution error")]
    DependencyResolution(#[from] DependencyResolutionError),
    
    /// Package management errors
    #[error("Package error")]
    Package(#[from] PackageError),
    
    /// Package registry errors
    #[error("Package registry error")]
    PackageRegistry(#[from] PackageRegistryError),
    
    /// Registry access errors
    #[error("Registry error")]
    Registry(#[from] RegistryError),
    
    /// I/O errors
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    /// JSON parsing errors
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    
    /// Generic errors with custom messages
    #[error("{0}")]
    Generic(String),
}

impl Error {
    /// Creates a new generic error with a custom message
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }
}

/// Result type alias for package tools operations
/// 
/// This is a convenience type alias for Results with the unified Error type.
/// 
/// # Examples
/// 
/// ```
/// use sublime_package_tools::{Result, Error};
/// 
/// fn parse_version(version_str: &str) -> Result<semver::Version> {
///     semver::Version::parse(version_str)
///         .map_err(|e| Error::generic(format!("Invalid version: {}", e)))
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;
