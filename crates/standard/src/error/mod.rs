//! # Error handling for `sublime_standard_tools`
//!
//! ## What
//! This module provides comprehensive error types for various operations within
//! the crate. It implements specific error types for different domains, such as
//! filesystem operations, command execution, and project management.
//!
//! ## How
//! Each domain has its own error type (e.g., `FileSystemError`) that implements
//! the `Error` trait from the standard library and uses the `thiserror` crate
//! for concise error definitions. Result type aliases are provided for convenience.
//!
//! ## Why
//! A structured approach to error handling enables callers to handle errors
//! appropriately based on their type and context, improving error reporting and
//! recovery strategies. The consistent pattern makes error handling predictable
//! across the crate.

mod command;
mod config;
mod filesystem;
mod monorepo;
mod recovery;
mod traits;
mod workspace;

#[cfg(test)]
mod tests;

use core::result::Result as CoreResult;
use thiserror::Error as ThisError;

// Re-export error types
pub use command::{CommandError, CommandResult};
pub use config::{ConfigError, ConfigResult};
pub use filesystem::FileSystemError;
pub use monorepo::{MonorepoError, MonorepoResult};
pub use recovery::{ErrorRecoveryManager, LogLevel, RecoveryResult, RecoveryStrategy};
pub use traits::ErrorContext;
pub use workspace::{WorkspaceError, WorkspaceResult};

/// Result type for filesystem operations.
///
/// This is a convenience type alias for Results with `FileSystemError`.
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
pub type FileSystemResult<T> = CoreResult<T, FileSystemError>;

/// General error type for the standard tools library.
///
/// This enum serves as a composite error type that aggregates all domain-specific
/// errors from the crate into a single error type. This allows for simplified error
/// handling in consumer code that may deal with multiple domains.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{Error, FileSystemError, MonorepoError};
/// use std::path::PathBuf;
///
/// // Creating an error from a filesystem error
/// let fs_error = FileSystemError::NotFound { path: PathBuf::from("/missing/file.txt") };
/// let error: Error = fs_error.into();
///
/// // Creating an error from a monorepo error
/// let monorepo_error = MonorepoError::ManagerNotFound;
/// let error: Error = monorepo_error.into();
///
/// // Using in a function that could have multiple error sources
/// fn complex_operation() -> sublime_standard_tools::error::Result<()> {
///     // This could return either a FileSystem or Monorepo error
///     // Both will be automatically converted to the Error enum
///     Ok(())
/// }
/// ```
#[derive(ThisError, Debug, Clone)]
pub enum Error {
    /// Monorepo-related error.
    #[error("Monorepo execution error")]
    Monorepo(#[from] MonorepoError),
    /// Filesystem-related error.
    #[error("FileSystem execution error")]
    FileSystem(#[from] FileSystemError),
    /// Workspace-related error.
    #[error("Workspace execution error")]
    Workspace(#[from] WorkspaceError),
    /// Command-related error.
    #[error("Command execution error")]
    Command(#[from] CommandError),
    /// Configuration-related error.
    #[error("Configuration error")]
    Config(#[from] ConfigError),
    /// General purpose errors with a custom message.
    #[error("Operation error: {0}")]
    Operation(String),
}

impl Error {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self {
        Self::Operation(message.into())
    }
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Error::Monorepo(_) => "Error::Monorepo",
            Error::FileSystem(_) => "Error::FileSystem",
            Error::Workspace(_) => "Error::Workspace",
            Error::Command(_) => "Error::Command",
            Error::Config(_) => "Error::Config",
            Error::Operation(_) => "Error::Operation",
        }
    }
}

/// Result type for general operations in the standard tools library.
///
/// This is a convenience type alias for Results with the composite Error type.
/// It simplifies error handling when functions may return errors from various domains.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{Result, Error, FileSystemError};
/// use std::path::PathBuf;
///
/// fn process_project_files(root_dir: &str) -> Result<Vec<String>> {
///     if root_dir.is_empty() {
///         return Err(FileSystemError::Validation {
///             path: PathBuf::from(root_dir),
///             reason: "Empty directory path".to_string(),
///         }.into());
///     }
///     // Implementation that might return various error types
///     Ok(vec!["file1.txt".to_string(), "file2.txt".to_string()])
/// }
/// ```
pub type Result<T> = CoreResult<T, Error>;
