//! # Monorepo error types
//!
//! ## What
//! This module defines errors that can occur during monorepo operations,
//! including detection, parsing, and management of monorepo configurations.
//!
//! ## How
//! The `MonorepoError` enum provides specific variants for different monorepo
//! operation failures, with filesystem errors as the underlying source.
//!
//! ## Why
//! Separating monorepo errors allows for targeted error handling strategies
//! specific to monorepo detection and management scenarios.

use core::result::Result as CoreResult;
use thiserror::Error as ThisError;

use super::FileSystemError;

/// Errors that can occur during monorepo operations.
///
/// This enum represents all the ways that monorepo detection and
/// management operations can fail, with specific variants for common error conditions.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::MonorepoError;
/// use std::path::PathBuf;
/// use sublime_standard_tools::error::FileSystemError;
///
/// let fs_error = FileSystemError::NotFound { path: PathBuf::from("/missing/file.yaml") };
/// let error = MonorepoError::Detection { source: fs_error };
/// assert!(error.to_string().contains("Failed to detect monorepo type"));
/// ```
#[derive(ThisError, Debug, Clone)]
pub enum MonorepoError {
    /// Failed to detect the monorepo type.
    #[error("Failed to detect monorepo type: {source}")]
    Detection {
        /// The underlying filesystem error
        #[source]
        source: FileSystemError,
    },
    /// Failed to parse the monorepo descriptor file.
    #[error("Failed to parse monorepo descriptor: {source}")]
    Parsing {
        /// The underlying filesystem error
        #[source]
        source: FileSystemError,
    },
    /// Failed to read the monorepo descriptor file.
    #[error("Failed to read monorepo descriptor: {source}")]
    Reading {
        /// The underlying filesystem error
        #[source]
        source: FileSystemError,
    },
    /// Failed to write the monorepo descriptor file.
    #[error("Failed to write monorepo descriptor: {source}")]
    Writing {
        /// The underlying filesystem error
        #[source]
        source: FileSystemError,
    },
    /// Failed to find a package manager for the monorepo.
    #[error("Failed to find package manager")]
    ManagerNotFound,
}

/// Result type for monorepo operations.
///
/// This is a convenience type alias for Results with `MonorepoError`.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{MonorepoResult, MonorepoError};
/// use std::path::PathBuf;
///
/// fn detect_monorepo(path: &str) -> MonorepoResult<String> {
///     if path.is_empty() {
///         return Err(MonorepoError::ManagerNotFound);
///     }
///     // Actual implementation would detect the monorepo type
///     Ok("yarn".to_string())
/// }
/// ```
pub type MonorepoResult<T> = CoreResult<T, MonorepoError>;

impl AsRef<str> for MonorepoError {
    fn as_ref(&self) -> &str {
        match self {
            MonorepoError::Detection { .. } => "MonorepoError::Detection",
            MonorepoError::Parsing { .. } => "MonorepoError::Parsing",
            MonorepoError::Reading { .. } => "MonorepoError::Reading",
            MonorepoError::Writing { .. } => "MonorepoError::Writing",
            MonorepoError::ManagerNotFound => "MonorepoError::ManagerNotFound",
        }
    }
}