//! Error types for the pkg crate.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Errors that can occur in the pkg crate
#[derive(Debug)]
pub enum PkgError {
    /// Error parsing a version string
    VersionParseError { version: String, source: semver::Error },

    /// Error parsing a version requirement
    VersionReqParseError { requirement: String, source: semver::Error },

    /// Error parsing JSON
    JsonParseError { path: Option<PathBuf>, source: serde_json::Error },

    /// Error reading/writing files
    IoError { path: Option<PathBuf>, source: io::Error },

    /// Package not found
    PackageNotFound { name: String },

    /// Dependency not found
    DependencyNotFound { name: String, package: String },

    /// Circular dependency detected
    CircularDependency { path: Vec<String> },

    /// Dependency resolution error
    DependencyResolutionError,

    /// Generic error
    Other { message: String },
}

impl fmt::Display for PkgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PkgError::VersionParseError { version, source } => {
                write!(f, "Failed to parse version '{version}': {source}")
            }
            PkgError::VersionReqParseError { requirement, source } => {
                write!(f, "Failed to parse version requirement '{requirement}': {source}")
            }
            PkgError::JsonParseError { path, source } => match path {
                Some(path) => write!(f, "Failed to parse JSON at '{}': {}", path.display(), source),
                None => write!(f, "Failed to parse JSON: {source}"),
            },
            PkgError::IoError { path, source } => match path {
                Some(path) => write!(f, "IO error at '{}': {}", path.display(), source),
                None => write!(f, "IO error: {source}"),
            },
            PkgError::PackageNotFound { name } => {
                write!(f, "Package not found: '{name}'")
            }
            PkgError::DependencyNotFound { name, package } => {
                write!(f, "Dependency '{name}' not found in package '{package}'")
            }
            PkgError::CircularDependency { path } => {
                write!(f, "Circular dependency detected: {}", path.join(" -> "))
            }
            PkgError::DependencyResolutionError => {
                write!(f, "Error resolving dependencies")
            }
            PkgError::Other { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for PkgError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PkgError::VersionReqParseError { source, .. } => Some(source),
            PkgError::JsonParseError { source, .. } => Some(source),
            PkgError::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for PkgError {
    fn from(error: io::Error) -> Self {
        PkgError::IoError { path: None, source: error }
    }
}

impl From<semver::Error> for PkgError {
    fn from(error: semver::Error) -> Self {
        PkgError::VersionParseError { version: "unknown".to_string(), source: error }
    }
}

impl From<serde_json::Error> for PkgError {
    fn from(error: serde_json::Error) -> Self {
        PkgError::JsonParseError { path: None, source: error }
    }
}

/// Result type for the pkg crate
pub type Result<T> = std::result::Result<T, PkgError>;
