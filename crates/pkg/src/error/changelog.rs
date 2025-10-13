//! # Changelog error types and implementations
//!
//! ## What
//! This module provides error types specific to changelog generation operations,
//! including template processing, file writing, and format validation.
//!
//! ## How
//! Provides detailed error types for changelog-related failures with specific
//! context for different stages of changelog generation and template rendering.
//!
//! ## Why
//! Changelog generation is an important part of the release process that
//! requires precise error handling to provide clear feedback about template
//! issues, file system problems, and formatting errors.

use std::path::PathBuf;
use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for changelog operations.
///
/// This is a convenience type alias for Results with `ChangelogError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{ChangelogResult, ChangelogError};
/// use std::path::PathBuf;
///
/// fn generate_changelog(path: &PathBuf) -> ChangelogResult<String> {
///     if !path.exists() {
///         return Err(ChangelogError::TemplateNotFound {
///             template_path: path.clone(),
///         });
///     }
///     Ok("# Changelog\n".to_string())
/// }
/// ```
pub type ChangelogResult<T> = StdResult<T, ChangelogError>;

/// Changelog-related error types.
///
/// Handles errors in changelog generation including template processing,
/// file operations, and format validation.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::ChangelogError;
///
/// let error = ChangelogError::GenerationFailed {
///     reason: "No release notes found".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Changelog generation failed: No release notes found
/// ```
#[derive(Error, Debug, Clone)]
pub enum ChangelogError {
    /// Changelog generation failed
    #[error("Changelog generation failed: {reason}")]
    GenerationFailed {
        /// Reason for generation failure
        reason: String,
    },

    /// Changelog template not found
    #[error("Changelog template not found: {template_path}")]
    TemplateNotFound {
        /// Path to the missing template file
        template_path: PathBuf,
    },

    /// Template rendering failed
    #[error("Template rendering failed: {reason}")]
    TemplateRenderingFailed {
        /// Reason for rendering failure
        reason: String,
    },

    /// Failed to write changelog file
    #[error("Failed to write changelog to '{path}': {reason}")]
    WriteFileFailed {
        /// Path where writing failed
        path: PathBuf,
        /// Reason for write failure
        reason: String,
    },

    /// Invalid changelog format
    #[error("Invalid changelog format in '{path}': {reason}")]
    InvalidFormat {
        /// Path to the invalid changelog file
        path: PathBuf,
        /// Reason why format is invalid
        reason: String,
    },
}

impl ChangelogError {
    /// Creates a generation failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why generation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let error = ChangelogError::generation_failed("No commits found");
    /// assert!(error.to_string().contains("Changelog generation failed"));
    /// ```
    #[must_use]
    pub fn generation_failed(reason: impl Into<String>) -> Self {
        Self::GenerationFailed { reason: reason.into() }
    }

    /// Creates a template not found error.
    ///
    /// # Arguments
    ///
    /// * `template_path` - Path to the missing template
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::template_not_found(PathBuf::from("templates/changelog.hbs"));
    /// assert!(error.to_string().contains("template not found"));
    /// ```
    #[must_use]
    pub fn template_not_found(template_path: PathBuf) -> Self {
        Self::TemplateNotFound { template_path }
    }

    /// Creates a template rendering failed error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why rendering failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let error = ChangelogError::template_rendering_failed("Invalid syntax");
    /// assert!(error.to_string().contains("Template rendering failed"));
    /// ```
    #[must_use]
    pub fn template_rendering_failed(reason: impl Into<String>) -> Self {
        Self::TemplateRenderingFailed { reason: reason.into() }
    }

    /// Creates a write file failed error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where writing failed
    /// * `reason` - Why writing failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::write_file_failed(
    ///     PathBuf::from("CHANGELOG.md"),
    ///     "Permission denied"
    /// );
    /// assert!(error.to_string().contains("Failed to write changelog"));
    /// ```
    #[must_use]
    pub fn write_file_failed(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::WriteFileFailed { path, reason: reason.into() }
    }

    /// Creates an invalid format error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the invalid changelog file
    /// * `reason` - Why format is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::invalid_format(
    ///     PathBuf::from("CHANGELOG.md"),
    ///     "Missing version header"
    /// );
    /// assert!(error.to_string().contains("Invalid changelog format"));
    /// ```
    #[must_use]
    pub fn invalid_format(path: PathBuf, reason: impl Into<String>) -> Self {
        Self::InvalidFormat { path, reason: reason.into() }
    }

    /// Checks if this is a template error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::template_not_found(PathBuf::from("template.hbs"));
    /// assert!(error.is_template_error());
    ///
    /// let error = ChangelogError::template_rendering_failed("reason");
    /// assert!(error.is_template_error());
    /// ```
    #[must_use]
    pub fn is_template_error(&self) -> bool {
        matches!(self, Self::TemplateNotFound { .. } | Self::TemplateRenderingFailed { .. })
    }

    /// Checks if this is a file system error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::write_file_failed(PathBuf::from("file"), "reason");
    /// assert!(error.is_filesystem_error());
    ///
    /// let error = ChangelogError::invalid_format(PathBuf::from("file"), "reason");
    /// assert!(error.is_filesystem_error());
    /// ```
    #[must_use]
    pub fn is_filesystem_error(&self) -> bool {
        matches!(
            self,
            Self::TemplateNotFound { .. }
                | Self::WriteFileFailed { .. }
                | Self::InvalidFormat { .. }
        )
    }

    /// Checks if this is a generation error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let error = ChangelogError::generation_failed("reason");
    /// assert!(error.is_generation_error());
    ///
    /// let error = ChangelogError::template_rendering_failed("reason");
    /// assert!(!error.is_generation_error());
    /// ```
    #[must_use]
    pub fn is_generation_error(&self) -> bool {
        matches!(self, Self::GenerationFailed { .. })
    }

    /// Gets the file path from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let path = PathBuf::from("CHANGELOG.md");
    /// let error = ChangelogError::write_file_failed(path.clone(), "reason");
    /// assert_eq!(error.file_path(), Some(&path));
    ///
    /// let error = ChangelogError::generation_failed("reason");
    /// assert_eq!(error.file_path(), None);
    /// ```
    #[must_use]
    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            Self::TemplateNotFound { template_path } => Some(template_path),
            Self::WriteFileFailed { path, .. } | Self::InvalidFormat { path, .. } => Some(path),
            Self::GenerationFailed { .. } | Self::TemplateRenderingFailed { .. } => None,
        }
    }

    /// Gets the reason from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let error = ChangelogError::generation_failed("No commits found");
    /// assert_eq!(error.reason(), Some("No commits found"));
    ///
    /// let error = ChangelogError::template_not_found(std::path::PathBuf::from("template"));
    /// assert_eq!(error.reason(), None);
    /// ```
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::GenerationFailed { reason }
            | Self::TemplateRenderingFailed { reason }
            | Self::WriteFileFailed { reason, .. }
            | Self::InvalidFormat { reason, .. } => Some(reason),
            Self::TemplateNotFound { .. } => None,
        }
    }
}

impl AsRef<str> for ChangelogError {
    fn as_ref(&self) -> &str {
        match self {
            ChangelogError::GenerationFailed { .. } => "ChangelogError::GenerationFailed",
            ChangelogError::TemplateNotFound { .. } => "ChangelogError::TemplateNotFound",
            ChangelogError::TemplateRenderingFailed { .. } => {
                "ChangelogError::TemplateRenderingFailed"
            }
            ChangelogError::WriteFileFailed { .. } => "ChangelogError::WriteFileFailed",
            ChangelogError::InvalidFormat { .. } => "ChangelogError::InvalidFormat",
        }
    }
}
