//! Standard error type for the sublime_standard_tools crate.
//!
//! What:
//! Defines the main error type used throughout the crate, providing a consistent
//! error handling interface.
//!
//! Who:
//! Used by both internal crate developers and external consumers who need to:
//! - Handle errors from any crate operation
//! - Convert specific errors to a standard format
//! - Add context to errors
//!
//! Why:
//! A standard error type ensures consistent error handling and proper error
//! context preservation throughout the crate.

use std::io;
use thiserror::Error;

use super::{CommandError, FileSystemError, ProcessError};

/// The main error type for the sublime_standard_tools crate.
/// It aggregates specific error types from different modules.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::error::{StandardError, CommandError};
///
/// fn execute_command() -> Result<(), StandardError> {
///     Err(CommandError::ExecutionFailed { cmd: "npm".to_string(), source: None }.into())
/// }
///
/// if let Err(e) = execute_command() {
///     eprintln!("Error: {}", e);
///     if let Some(source) = e.source() {
///         eprintln!("Caused by: {}", source);
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum StandardError {
    /// Errors related to command execution.
    #[error("Command execution error")]
    Command(#[from] CommandError),

    /// Errors related to process management.
    #[error("Process management error")]
    Process(#[from] ProcessError),

    /// Errors related to filesystem operations.
    #[error("Filesystem error")]
    FileSystem(#[from] FileSystemError),

    /// General I/O errors not covered by FileSystemError.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Errors during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Errors related to invalid UTF-8 encoding.
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// General purpose errors with a custom message.
    #[error("Operation error: {0}")]
    Operation(String),
}

// Optional: Implement convenience methods if needed, but thiserror handles basics.
impl StandardError {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self {
        Self::Operation(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, io};

    #[test]
    fn test_standard_error_display() {
        let op_error = StandardError::operation("test operation error");
        assert_eq!(op_error.to_string(), "Operation error: test operation error");

        let io_error = StandardError::from(io::Error::new(io::ErrorKind::NotFound, "not found"));
        assert_eq!(io_error.to_string(), "I/O error: not found");
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_error_source_chaining() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let fs_err = FileSystemError::Io { path: "/test".into(), source: io_err };
        let std_err = StandardError::from(fs_err);

        assert!(std_err.source().is_some());
        let source1 = std_err.source().expect("StandardError should have a source");
        assert!(source1.is::<FileSystemError>());

        let source2 = source1.source().expect("FileSystemError should have a source");
        assert!(source2.is::<io::Error>());
        assert_eq!(source2.to_string(), "Permission denied");
    }

    #[allow(clippy::expect_used)]
    #[test]
    fn test_from_command_error() {
        let cmd_err = CommandError::Timeout { duration: std::time::Duration::from_secs(5) };
        let std_err: StandardError = cmd_err.into();
        assert_eq!(std_err.to_string(), "Command execution error");
        assert!(std_err.source().is_some());
        assert!(std_err.source().expect("StandardError should have a source").is::<CommandError>());
    }
}
