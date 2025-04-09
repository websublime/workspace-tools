//! Error types for change tracking.
//!
//! This module defines error types that can occur during change tracking operations,
//! such as reading, writing, parsing, and serializing changesets.

use crate::WorkspaceError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during change tracking operations.
///
/// This enum represents all possible errors that might occur when working
/// with changes and changesets, including file I/O errors, parsing errors,
/// and workspace-related errors.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use sublime_monorepo_tools::ChangeError;
///
/// // Create a read error
/// let read_error = ChangeError::ReadError {
///     path: PathBuf::from("/path/to/changes"),
///     error: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
/// };
///
/// // Get error string representation
/// assert_eq!(read_error.as_ref(), "ReadError");
/// ```
#[derive(Debug, Error)]
pub enum ChangeError {
    /// Failed to read a changeset file
    #[error("Failed to read changeset at {path}: {error}")]
    ReadError { path: PathBuf, error: io::Error },

    /// Failed to write a changeset file
    #[error("Failed to write changeset at {path}: {error}")]
    WriteError { path: PathBuf, error: io::Error },

    /// Failed to parse a changeset file
    #[error("Failed to parse changeset at {path}: {error}")]
    ParseError { path: PathBuf, error: serde_json::Error },

    /// Failed to serialize a changeset
    #[error("Failed to serialize changeset: {0}")]
    SerializeError(serde_json::Error),

    /// Failed to create changeset directory
    #[error("Failed to create changeset directory at {path}: {error}")]
    DirectoryCreationError { path: PathBuf, error: io::Error },

    /// Failed to list changeset files
    #[error("Failed to list changeset files in {path}: {error}")]
    ListError { path: PathBuf, error: io::Error },

    /// No Git repository found
    #[error("No Git repository found")]
    NoGitRepository,

    /// Failed to detect changes
    #[error("Failed to detect changes: {0}")]
    DetectionError(String),

    /// Invalid reference
    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    /// Workspace error
    #[error("Workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),

    /// Git error
    #[error("Git error: {0}")]
    GitError(#[from] sublime_git_tools::RepoError),

    /// No changes found
    #[error("No changes found")]
    NoChangesFound,

    /// Invalid changeset
    #[error("Invalid changeset: {0}")]
    InvalidChangeset(String),

    /// Invalid package
    #[error("Invalid package: {0}")]
    InvalidPackage(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

impl AsRef<str> for ChangeError {
    fn as_ref(&self) -> &str {
        match self {
            ChangeError::ReadError { path: _, error: _ } => "ReadError",
            ChangeError::WriteError { path: _, error: _ } => "WriteError",
            ChangeError::ParseError { path: _, error: _ } => "ParseError",
            ChangeError::SerializeError(_) => "SerializeError",
            ChangeError::DirectoryCreationError { path: _, error: _ } => "DirectoryCreationError",
            ChangeError::ListError { path: _, error: _ } => "ListError",
            ChangeError::NoGitRepository => "NoGitRepository",
            ChangeError::DetectionError(_) => "DetectionError",
            ChangeError::InvalidReference(_) => "InvalidReference",
            ChangeError::WorkspaceError(_) => "WorkspaceError",
            ChangeError::GitError(_) => "GitError",
            ChangeError::NoChangesFound => "NoChangesFound",
            ChangeError::InvalidChangeset(_) => "InvalidChangeset",
            ChangeError::InvalidPackage(_) => "InvalidPackage",
            ChangeError::IoError(_) => "IoError",
        }
    }
}
