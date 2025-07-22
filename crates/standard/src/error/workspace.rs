//! # Workspace error types
//!
//! ## What
//! This module defines errors that can occur during workspace operations,
//! specifically related to parsing and working with monorepo workspace configurations.
//!
//! ## How
//! The `WorkspaceError` enum provides specific variants for different workspace
//! operation failures, such as invalid formats, missing packages, or configuration issues.
//!
//! ## Why
//! Separating workspace errors enables targeted error handling for operations
//! involving monorepo workspace management and package resolution.

use core::result::Result as CoreResult;
use thiserror::Error as ThisError;

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

impl AsRef<str> for WorkspaceError {
    fn as_ref(&self) -> &str {
        match self {
            WorkspaceError::InvalidPackageJson(_) => "WorkspaceError::InvalidPackageJson",
            WorkspaceError::InvalidWorkspacesPattern(_) => "WorkspaceError::InvalidWorkspacesPattern",
            WorkspaceError::InvalidPnpmWorkspace(_) => "WorkspaceError::InvalidPnpmWorkspace",
            WorkspaceError::PackageNotFound(_) => "WorkspaceError::PackageNotFound",
            WorkspaceError::WorkspaceNotFound(_) => "WorkspaceError::WorkspaceNotFound",
            WorkspaceError::WorkspaceConfigMissing(_) => "WorkspaceError::WorkspaceConfigMissing",
        }
    }
}