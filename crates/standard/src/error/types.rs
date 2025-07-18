//! # Error type definitions
//!
//! ## What
//! This file contains the core type definitions for errors used throughout
//! the `sublime_standard_tools` crate. It defines error enums and result type
//! aliases for various domains.
//!
//! ## How
//! Errors are defined using thiserror for automatic trait implementations.
//! Each error variant includes descriptive fields and error messages.
//!
//! ## Why
//! Centralizing error type definitions provides a clear overview of all
//! possible error conditions and ensures consistency in error handling.

use core::result::Result as CoreResult;
use std::path::PathBuf;
use std::time::Duration;
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

/// Errors that can occur during workspace operations.
///
/// This enum represents the various ways that workspace processing
/// can fail, specifically related to parsing and working with monorepo
/// workspace configurations.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::WorkspaceError;
///
/// // Creating a specific workspace error
/// let error = WorkspaceError::PackageNotFound("ui-components".to_string());
/// assert!(error.to_string().contains("Package not found"));
/// ```
#[derive(ThisError, Debug, Clone)]
pub enum WorkspaceError {
    /// Error parsing package.json format.
    #[error("Invalid package json format: {0}")]
    InvalidPackageJson(String),
    /// Error parsing workspaces pattern.
    #[error("Invalid workspaces pattern: {0}")]
    InvalidWorkspacesPattern(String),
    /// Error parsing pnpm workspace configuration.
    #[error("Invalid workspaces pattern: {0}")]
    InvalidPnpmWorkspace(String),
    /// Package not found in workspace.
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    /// Workspace not found.
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),
    /// Workspace configuration is missing.
    #[error("Workspace config is missing: {0}")]
    WorkspaceConfigMissing(String),
}

/// Result type for workspace operations.
///
/// This is a convenience type alias for Results with `WorkspaceError`.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{WorkspaceResult, WorkspaceError};
///
/// fn find_workspace_package(name: &str) -> WorkspaceResult<String> {
///     if name.is_empty() {
///         return Err(WorkspaceError::PackageNotFound("Empty name provided".to_string()));
///     }
///     // Implementation would look up the package
///     Ok(format!("Found package {}", name))
/// }
/// ```
pub type WorkspaceResult<T> = CoreResult<T, WorkspaceError>;

/// Errors that can occur during command execution.
///
/// This enum represents the various ways that command execution can fail,
/// from spawn failures to timeouts to non-zero exit codes, with specific
/// variants for common error conditions.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{CommandError, Error};
/// use std::time::Duration;
///
/// // Creating a timeout error
/// let error = CommandError::Timeout {
///     duration: Duration::from_secs(30)
/// };
///
/// // Converting to the general Error type
/// let general_error: Error = error.into();
/// ```
#[derive(ThisError, Debug, Clone)]
pub enum CommandError {
    /// The command failed to start (e.g., not found).
    #[error("Failed to spawn command '{cmd}': {message}")]
    SpawnFailed {
        /// The command that failed to start
        cmd: String,
        /// The spawn failure error message
        message: String,
    },

    /// The command execution process itself failed (e.g., internal I/O error).
    #[error("Command execution failed for '{cmd}': {message}")]
    ExecutionFailed {
        /// The command that failed during execution
        cmd: String,
        /// The execution failure error message
        message: String,
    },

    /// The command executed but returned a non-zero exit code.
    #[error("Command '{cmd}' failed with exit code {code:?}. Stderr: {stderr}")]
    NonZeroExitCode {
        /// The command that returned a non-zero exit code
        cmd: String,
        /// The exit code returned by the command
        code: Option<i32>,
        /// The error output captured from the command
        stderr: String,
    },

    /// The command timed out after the specified duration.
    #[error("Command timed out after {duration:?}")]
    Timeout {
        /// The time period after which the command timed out
        duration: Duration,
    },

    /// The command was killed (e.g., by a signal).
    #[error("Command was killed: {reason}")]
    Killed {
        /// The reason why the command was killed
        reason: String,
    },

    /// Invalid configuration provided for the command.
    #[error("Invalid command configuration: {description}")]
    Configuration {
        /// Description of the configuration error
        description: String,
    },

    /// Failed to capture stdout or stderr.
    #[error("Failed to capture {stream} stream")]
    CaptureFailed {
        /// Name of the stream that failed to capture (stdout/stderr)
        stream: String,
    },

    /// Error occurred while reading stdout or stderr stream.
    #[error("Error reading {stream} stream: {message}")]
    StreamReadError {
        /// Name of the stream that encountered a read error
        stream: String,
        /// The stream read error message
        message: String,
    },

    /// Generic error during command processing.
    #[error("Command processing error: {0}")]
    Generic(String),
}

/// Result type for command operations.
///
/// This is a convenience type alias for Results with `CommandError`.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{CommandResult, CommandError};
///
/// fn execute_build_command(args: &[&str]) -> CommandResult<String> {
///     if args.is_empty() {
///         return Err(CommandError::Configuration {
///             description: "No build arguments provided".to_string(),
///         });
///     }
///     // Actual implementation would execute the command
///     Ok("Build completed successfully".to_string())
/// }
/// ```
pub type CommandResult<T> = CoreResult<T, CommandError>;

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
