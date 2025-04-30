//! # Error type definitions
//!
//! ## What
//! This file contains the core type definitions for errors used throughout
//! the sublime_standard_tools crate. It defines error enums and result type
//! aliases for various domains.
//!
//! ## How
//! Errors are defined using thiserror for automatic trait implementations.
//! Each error variant includes descriptive fields and error messages.
//!
//! ## Why
//! Centralizing error type definitions provides a clear overview of all
//! possible error conditions and ensures consistency in error handling.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during filesystem operations.
///
/// This enum represents all the ways that filesystem operations can fail,
/// with specific variants for common error conditions and descriptive
/// error messages.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::FileSystemError;
/// use std::path::PathBuf;
///
/// // Creating a "not found" error
/// let error = FileSystemError::NotFound { path: PathBuf::from("/missing/file.txt") };
/// assert!(error.to_string().contains("not found"));
/// ```
#[derive(Error, Debug)]
pub enum FileSystemError {
    /// Path not found.
    #[error("Path not found: {path}")]
    NotFound {
        /// The path that was not found
        path: PathBuf,
    },

    /// Permission denied for accessing the path.
    #[error("Permission denied for path: {path}")]
    PermissionDenied {
        /// The path for which permission was denied
        path: PathBuf,
    },

    /// Generic I/O error during filesystem operation.
    #[error("I/O error accessing path '{path}': {source}")]
    Io {
        /// The path where the I/O error occurred
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Attempted an operation requiring a directory on a file.
    #[error("Expected a directory but found a file: {path}")]
    NotADirectory {
        /// The path that was expected to be a directory but wasn't
        path: PathBuf,
    },

    /// Attempted an operation requiring a file on a directory.
    #[error("Expected a file but found a directory: {path}")]
    NotAFile {
        /// The path that was expected to be a file but wasn't
        path: PathBuf,
    },

    /// Failed to decode UTF-8 content from a file.
    #[error("Failed to decode UTF-8 content in file: {path}")]
    Utf8Decode {
        /// The path to the file with invalid UTF-8 content
        path: PathBuf,
        /// The underlying UTF-8 decoding error
        #[source]
        source: std::string::FromUtf8Error,
    },

    /// Path validation failed (e.g., contains '..', absolute path, symlink).
    #[error("Path validation failed for '{path}': {reason}")]
    Validation {
        /// The path that failed validation
        path: PathBuf,
        /// The reason why validation failed
        reason: String,
    },
}

/// Result type for filesystem operations.
///
/// This is a convenience type alias for Results with FileSystemError.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{FileSystemResult, FileSystemError};
/// use std::path::PathBuf;
///
/// fn read_config(path: &str) -> FileSystemResult<String> {
///     if path.is_empty() {
///         return Err(FileSystemError::Validation {
///             path: PathBuf::from(path),
///             reason: "Empty path".to_string(),
///         });
///     }
///     // Actual implementation would read the file
///     Ok("sample config".to_string())
/// }
/// ```
pub type FileSystemResult<T> = Result<T, FileSystemError>;
