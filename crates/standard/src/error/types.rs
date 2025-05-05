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

use core::result::Result as CoreResult;
use std::io;
use std::path::PathBuf;
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
#[derive(ThisError, Debug)]
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
#[derive(ThisError, Debug)]
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
/// This is a convenience type alias for Results with MonorepoError.
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
#[derive(ThisError, Debug)]
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
/// This is a convenience type alias for Results with WorkspaceError.
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
#[derive(ThisError, Debug)]
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
