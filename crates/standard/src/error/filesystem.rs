//! # Filesystem error types and implementations
//!
//! ## What
//! This module extends the FileSystemError type with additional implementations
//! and utility methods specific to filesystem operations.
//!
//! ## How
//! Provides conversion methods from io::Error and implements the AsRef trait
//! for better error categorization and compatibility.
//!
//! ## Why
//! These implementations enable seamless conversion between standard library
//! errors and our domain-specific filesystem errors.

use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

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
#[derive(ThisError, Debug, Clone)]
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
    #[error("I/O error accessing path '{path}': {message}")]
    Io {
        /// The path where the I/O error occurred
        path: PathBuf,
        /// The I/O error message
        message: String,
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
    #[error("Failed to decode UTF-8 content in file: {path} - {message}")]
    Utf8Decode {
        /// The path to the file with invalid UTF-8 content
        path: PathBuf,
        /// The UTF-8 decoding error message
        message: String,
    },

    /// Path validation failed (e.g., contains '..', absolute path, symlink).
    #[error("Path validation failed for '{path}': {reason}")]
    Validation {
        /// The path that failed validation
        path: PathBuf,
        /// The reason why validation failed
        reason: String,
    },

    /// Operation failed (e.g., timeout, concurrency limit exceeded).
    #[error("Operation failed: {0}")]
    Operation(String),
}

impl FileSystemError {
    /// Creates a new path validation error.
    pub fn validation(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::Validation { path: path.into(), reason: reason.into() }
    }
}

impl FileSystemError {
    /// Creates a `FileSystemError` from an `io::Error` and associated path.
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_io(error: io::Error, path: impl Into<PathBuf>) -> Self {
        let path_buf = path.into();
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path: path_buf },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path: path_buf },
            // Add other kinds if needed, e.g., IsADirectory, NotADirectory
            _ => Self::Io { path: path_buf, message: error.to_string() },
        }
    }
}

impl From<io::Error> for FileSystemError {
    fn from(error: io::Error) -> Self {
        // Create a dummy path or indicate unknown path
        let path = PathBuf::from("<unknown>");
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            _ => Self::Io { path, message: error.to_string() },
        }
    }
}

impl AsRef<str> for FileSystemError {
    fn as_ref(&self) -> &str {
        match self {
            FileSystemError::NotFound { .. } => "FileSystemError::NotFound",
            FileSystemError::PermissionDenied { .. } => "FileSystemError::PermissionDenied",
            FileSystemError::Io { .. } => "FileSystemError::Io",
            FileSystemError::NotADirectory { .. } => "FileSystemError::NotADirectory",
            FileSystemError::NotAFile { .. } => "FileSystemError::NotAFile",
            FileSystemError::Utf8Decode { .. } => "FileSystemError::Utf8Decode",
            FileSystemError::Validation { .. } => "FileSystemError::Validation",
            FileSystemError::Operation(_) => "FileSystemError::Operation",
        }
    }
}
