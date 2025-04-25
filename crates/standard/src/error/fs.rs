//! Filesystem error type.
//!
//! What:
//! Defines specific error types for filesystem operations, providing
//! detailed error information for file and directory management.
//!
//! Who:
//! Used by developers who need to:
//! - Handle filesystem operation failures
//! - Provide context about filesystem errors (e.g., path involved)
//! - Implement custom filesystem error handling
//!
//! Why:
//! Filesystem operations require specific error handling to provide proper
//! context about which operation failed on which path.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error type for filesystem operations.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::error::FileSystemError;
/// use std::path::PathBuf;
///
/// let error = FileSystemError::NotFound { path: PathBuf::from("/path/to/missing") };
/// assert!(error.to_string().contains("Path not found"));
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

// Convenience constructors can be added if needed, for example:
impl FileSystemError {
    /// Creates a new path validation error.
    pub fn validation(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::Validation { path: path.into(), reason: reason.into() }
    }
}

// Map common io::Error kinds to specific FileSystemError variants
// Add #[from] io::Error if direct conversion is desired,
// but from_io provides path context which is usually better.
impl FileSystemError {
    /// Creates a FileSystemError from an io::Error and associated path.
    pub fn from_io(error: io::Error, path: impl Into<PathBuf>) -> Self {
        let path_buf = path.into();
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path: path_buf },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path: path_buf },
            // Add other kinds if needed, e.g., IsADirectory, NotADirectory
            _ => Self::Io { path: path_buf, source: error },
        }
    }
}

// We add From<io::Error> for convenience with `?` in simple cases,
// This From implementation is useful for simple error propagation with `?`
// when the path context isn't immediately available or necessary at the point of conversion.
// However, using from_io is preferred when path context *is* available.
// Add #[from] here if direct io::Error conversion is desired without context.
impl From<io::Error> for FileSystemError {
    fn from(error: io::Error) -> Self {
        // Create a dummy path or indicate unknown path
        let path = PathBuf::from("<unknown>");
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            _ => Self::Io { path, source: error },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_error_display() {
        let not_found = FileSystemError::NotFound { path: "/test".into() };
        assert_eq!(not_found.to_string(), "Path not found: /test");

        let io_error =
            FileSystemError::from_io(io::Error::new(io::ErrorKind::Other, "disk full"), "/data");
        assert_eq!(io_error.to_string(), "I/O error accessing path '/data': disk full");
    }

    #[test]
    fn test_validation_error() {
        let validation_error = FileSystemError::validation("/a/../b", "Parent traversal");
        assert_eq!(
            validation_error.to_string(),
            "Path validation failed for '/a/../b': Parent traversal"
        );
    }
}
