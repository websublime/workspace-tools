//! # Conventional commit error types and implementations
//!
//! ## What
//! This module provides error types specific to conventional commit parsing operations,
//! including format validation, type recognition, and breaking change detection.
//!
//! ## How
//! Provides detailed error types for conventional commit parsing failures with specific
//! context for different parsing stages and validation rules.
//!
//! ## Why
//! Conventional commits are essential for automatic version bump calculation and
//! changelog generation, requiring precise error handling to provide clear feedback
//! about parsing failures and format violations.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for conventional commit operations.
///
/// This is a convenience type alias for Results with `ConventionalCommitError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{ConventionalCommitResult, ConventionalCommitError};
///
/// fn parse_commit_message(message: &str) -> ConventionalCommitResult<String> {
///     if message.is_empty() {
///         return Err(ConventionalCommitError::InvalidFormat {
///             commit: message.to_string(),
///             reason: "Empty commit message".to_string(),
///         });
///     }
///     Ok(message.to_string())
/// }
/// ```
pub type ConventionalCommitResult<T> = StdResult<T, ConventionalCommitError>;

/// Conventional commit parsing error types.
///
/// Handles errors in conventional commit parsing including format validation,
/// type recognition, and breaking change detection.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::ConventionalCommitError;
///
/// let error = ConventionalCommitError::InvalidFormat {
///     commit: "bad commit message".to_string(),
///     reason: "Missing type and scope".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Invalid conventional commit format for 'bad commit message': Missing type and scope
/// ```
#[derive(Error, Debug, Clone)]
pub enum ConventionalCommitError {
    /// Invalid conventional commit format
    #[error("Invalid conventional commit format for '{commit}': {reason}")]
    InvalidFormat {
        /// Commit message
        commit: String,
        /// Reason why format is invalid
        reason: String,
    },

    /// Unknown commit type
    #[error("Unknown commit type '{commit_type}' in commit '{commit}'")]
    UnknownType {
        /// Unknown commit type
        commit_type: String,
        /// Full commit message
        commit: String,
    },

    /// Commit parsing failed
    #[error("Failed to parse commit '{commit}': {reason}")]
    ParseFailed {
        /// Commit message
        commit: String,
        /// Reason for parsing failure
        reason: String,
    },

    /// Breaking change detection failed
    #[error("Failed to detect breaking changes in commit '{commit}': {reason}")]
    BreakingChangeDetectionFailed {
        /// Commit message
        commit: String,
        /// Reason for detection failure
        reason: String,
    },
}

/// Commit type parsing error types.
///
/// Handles errors specific to commit type parsing and validation.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::CommitTypeParseError;
///
/// let error = CommitTypeParseError::Empty;
/// println!("Error: {}", error);
/// // Output: Empty commit type string
/// ```
#[derive(Error, Debug, Clone)]
pub enum CommitTypeParseError {
    /// Empty string provided
    #[error("Empty commit type string")]
    Empty,
    /// Invalid commit type format
    #[error("Invalid commit type format: '{0}'")]
    InvalidFormat(String),
}

impl ConventionalCommitError {
    /// Creates an invalid format error.
    ///
    /// # Arguments
    ///
    /// * `commit` - Commit message
    /// * `reason` - Why format is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::invalid_format(
    ///     "bad commit",
    ///     "Missing conventional format"
    /// );
    /// assert!(error.to_string().contains("Invalid conventional commit format"));
    /// ```
    #[must_use]
    pub fn invalid_format(commit: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidFormat { commit: commit.into(), reason: reason.into() }
    }

    /// Creates an unknown type error.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - Unknown commit type
    /// * `commit` - Full commit message
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::unknown_type(
    ///     "unknown",
    ///     "unknown: some message"
    /// );
    /// assert!(error.to_string().contains("Unknown commit type"));
    /// ```
    #[must_use]
    pub fn unknown_type(commit_type: impl Into<String>, commit: impl Into<String>) -> Self {
        Self::UnknownType { commit_type: commit_type.into(), commit: commit.into() }
    }

    /// Creates a parse failed error.
    ///
    /// # Arguments
    ///
    /// * `commit` - Commit message
    /// * `reason` - Why parsing failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::parse_failed(
    ///     "malformed: commit",
    ///     "Regex match failed"
    /// );
    /// assert!(error.to_string().contains("Failed to parse commit"));
    /// ```
    #[must_use]
    pub fn parse_failed(commit: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ParseFailed { commit: commit.into(), reason: reason.into() }
    }

    /// Creates a breaking change detection failed error.
    ///
    /// # Arguments
    ///
    /// * `commit` - Commit message
    /// * `reason` - Why detection failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::breaking_change_detection_failed(
    ///     "feat!: new feature",
    ///     "Failed to parse footer"
    /// );
    /// assert!(error.to_string().contains("Failed to detect breaking changes"));
    /// ```
    #[must_use]
    pub fn breaking_change_detection_failed(
        commit: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::BreakingChangeDetectionFailed { commit: commit.into(), reason: reason.into() }
    }

    /// Checks if this is a format error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::invalid_format("commit", "reason");
    /// assert!(error.is_format_error());
    ///
    /// let error = ConventionalCommitError::parse_failed("commit", "reason");
    /// assert!(!error.is_format_error());
    /// ```
    #[must_use]
    pub fn is_format_error(&self) -> bool {
        matches!(self, Self::InvalidFormat { .. })
    }

    /// Checks if this is a parsing error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::parse_failed("commit", "reason");
    /// assert!(error.is_parsing_error());
    ///
    /// let error = ConventionalCommitError::unknown_type("type", "commit");
    /// assert!(error.is_parsing_error());
    /// ```
    #[must_use]
    pub fn is_parsing_error(&self) -> bool {
        matches!(
            self,
            Self::ParseFailed { .. }
                | Self::UnknownType { .. }
                | Self::BreakingChangeDetectionFailed { .. }
        )
    }

    /// Gets the commit message from all error variants.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::invalid_format("my commit", "reason");
    /// assert_eq!(error.commit_message(), "my commit");
    ///
    /// let error = ConventionalCommitError::unknown_type("type", "my commit");
    /// assert_eq!(error.commit_message(), "my commit");
    /// ```
    #[must_use]
    pub fn commit_message(&self) -> &str {
        match self {
            Self::InvalidFormat { commit, .. }
            | Self::UnknownType { commit, .. }
            | Self::ParseFailed { commit, .. }
            | Self::BreakingChangeDetectionFailed { commit, .. } => commit,
        }
    }

    /// Gets the commit type from unknown type errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConventionalCommitError;
    ///
    /// let error = ConventionalCommitError::unknown_type("custom", "custom: message");
    /// assert_eq!(error.commit_type(), Some("custom"));
    ///
    /// let error = ConventionalCommitError::invalid_format("commit", "reason");
    /// assert_eq!(error.commit_type(), None);
    /// ```
    #[must_use]
    pub fn commit_type(&self) -> Option<&str> {
        match self {
            Self::UnknownType { commit_type, .. } => Some(commit_type),
            _ => None,
        }
    }
}

impl CommitTypeParseError {
    /// Creates an empty error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::CommitTypeParseError;
    ///
    /// let error = CommitTypeParseError::empty();
    /// assert!(error.to_string().contains("Empty"));
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Creates an invalid format error.
    ///
    /// # Arguments
    ///
    /// * `format` - Invalid format string
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::CommitTypeParseError;
    ///
    /// let error = CommitTypeParseError::invalid_format("invalid-type!");
    /// assert!(error.to_string().contains("Invalid commit type format"));
    /// ```
    #[must_use]
    pub fn invalid_format(format: impl Into<String>) -> Self {
        Self::InvalidFormat(format.into())
    }

    /// Checks if this is an empty error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::CommitTypeParseError;
    ///
    /// let error = CommitTypeParseError::Empty;
    /// assert!(error.is_empty());
    ///
    /// let error = CommitTypeParseError::InvalidFormat("format".to_string());
    /// assert!(!error.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Gets the invalid format string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::CommitTypeParseError;
    ///
    /// let error = CommitTypeParseError::InvalidFormat("bad-format".to_string());
    /// assert_eq!(error.format_string(), Some("bad-format"));
    ///
    /// let error = CommitTypeParseError::Empty;
    /// assert_eq!(error.format_string(), None);
    /// ```
    #[must_use]
    pub fn format_string(&self) -> Option<&str> {
        match self {
            Self::InvalidFormat(format) => Some(format),
            Self::Empty => None,
        }
    }
}
