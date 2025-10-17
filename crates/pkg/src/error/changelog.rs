//! Changelog error types for package tools.
//!
//! **What**: Defines error types specific to changelog generation, parsing, formatting,
//! and version detection operations.
//!
//! **How**: Uses `thiserror` for error definitions with rich context information including
//! file paths, version strings, commit references, and parsing details. Implements
//! `AsRef<str>` for string conversion.
//!
//! **Why**: To provide clear, actionable error messages for changelog operations, enabling
//! users to quickly identify and fix issues with changelog generation, conventional commit
//! parsing, and version detection.
//!
//! # Examples
//!
//! ```rust
//! use sublime_pkg_tools::error::{ChangelogError, ChangelogResult};
//! use std::path::PathBuf;
//!
//! fn generate_changelog(path: &str) -> ChangelogResult<String> {
//!     if path.is_empty() {
//!         return Err(ChangelogError::InvalidPath {
//!             path: PathBuf::from(path),
//!             reason: "Changelog path cannot be empty".to_string(),
//!         });
//!     }
//!     Ok("changelog-content".to_string())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for changelog operations.
///
/// This type alias simplifies error handling in changelog-related functions
/// by defaulting to `ChangelogError` as the error type.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{ChangelogError, ChangelogResult};
///
/// fn parse_changelog() -> ChangelogResult<String> {
///     Ok("parsed-content".to_string())
/// }
/// ```
pub type ChangelogResult<T> = Result<T, ChangelogError>;

/// Errors that can occur during changelog operations.
///
/// This enum covers all possible error scenarios when working with changelogs,
/// including generation, parsing, formatting, conventional commit parsing, and
/// version detection.
///
/// # Examples
///
/// ## Handling changelog errors
///
/// ```rust
/// use sublime_pkg_tools::error::ChangelogError;
/// use std::path::PathBuf;
///
/// fn handle_changelog_error(error: ChangelogError) {
///     match error {
///         ChangelogError::NotFound { path } => {
///             eprintln!("Changelog not found: {}", path.display());
///         }
///         ChangelogError::ParseError { line, reason } => {
///             eprintln!("Parse error at line {}: {}", line, reason);
///         }
///         _ => eprintln!("Changelog error: {}", error),
///     }
/// }
/// ```
///
/// ## Converting from string representation
///
/// ```rust
/// use sublime_pkg_tools::error::ChangelogError;
/// use std::path::PathBuf;
///
/// let error = ChangelogError::NotFound {
///     path: PathBuf::from("CHANGELOG.md"),
/// };
///
/// let error_msg: &str = error.as_ref();
/// assert!(error_msg.contains("not found"));
/// ```
#[derive(Debug, Error, Clone)]
pub enum ChangelogError {
    /// Changelog file not found.
    ///
    /// This error occurs when attempting to read or update a changelog file
    /// that does not exist at the specified path.
    #[error("Changelog file not found: {path}")]
    NotFound {
        /// Path to the missing changelog file.
        path: PathBuf,
    },

    /// Failed to parse changelog content.
    ///
    /// This error occurs when changelog content cannot be parsed according
    /// to the expected format (Keep a Changelog, Conventional Commits, etc.).
    #[error("Failed to parse changelog at line {line}: {reason}")]
    ParseError {
        /// Line number where parsing failed.
        line: usize,
        /// Description of the parsing error.
        reason: String,
    },

    /// Invalid changelog format.
    ///
    /// This error occurs when the changelog format is not recognized or
    /// is inconsistent with the configured format.
    #[error("Invalid changelog format: expected {expected}, found {actual}")]
    InvalidFormat {
        /// Expected changelog format.
        expected: String,
        /// Actual format detected.
        actual: String,
    },

    /// Failed to generate changelog content.
    ///
    /// This error occurs when changelog generation fails due to missing
    /// data, invalid templates, or formatting errors.
    #[error("Failed to generate changelog for version '{version}': {reason}")]
    GenerationFailed {
        /// Version for which generation failed.
        version: String,
        /// Description of why generation failed.
        reason: String,
    },

    /// Conventional commit parsing failed.
    ///
    /// This error occurs when a commit message does not conform to the
    /// Conventional Commits specification.
    #[error("Failed to parse conventional commit '{commit}': {reason}")]
    ConventionalCommitParseError {
        /// The commit hash or short hash.
        commit: String,
        /// Description of the parsing error.
        reason: String,
    },

    /// Git operation failed during changelog operations.
    ///
    /// This error occurs when git commands fail during version detection,
    /// commit retrieval, or tag operations.
    #[error("Git operation failed: {operation} - {reason}")]
    GitError {
        /// Description of the git operation that failed.
        operation: String,
        /// Detailed error message from git.
        reason: String,
    },

    /// Version not found in git history.
    ///
    /// This error occurs when attempting to detect a previous version from
    /// git tags but no version tag is found.
    #[error("Version not found in git history: {reason}")]
    VersionNotFound {
        /// Description of why the version was not found.
        reason: String,
    },

    /// Invalid version string.
    ///
    /// This error occurs when a version string does not conform to semantic
    /// versioning or the expected version format.
    #[error("Invalid version '{version}': {reason}")]
    InvalidVersion {
        /// The invalid version string.
        version: String,
        /// Description of why the version is invalid.
        reason: String,
    },

    /// Version tag parsing failed.
    ///
    /// This error occurs when attempting to extract version information from
    /// a git tag but the tag format is invalid.
    #[error("Failed to parse version from tag '{tag}': {reason}")]
    VersionTagParseError {
        /// The git tag that failed to parse.
        tag: String,
        /// Description of the parsing error.
        reason: String,
    },

    /// File system error during changelog operations.
    ///
    /// This error occurs when filesystem operations (read, write, update) fail
    /// during changelog management.
    #[error("Filesystem error at '{path}': {reason}")]
    FileSystemError {
        /// Path where the error occurred.
        path: PathBuf,
        /// Description of the filesystem error.
        reason: String,
    },

    /// Invalid changelog path.
    ///
    /// This error occurs when a changelog path is malformed, contains invalid
    /// characters, or points to an invalid location.
    #[error("Invalid changelog path '{path}': {reason}")]
    InvalidPath {
        /// The invalid path.
        path: PathBuf,
        /// Description of why the path is invalid.
        reason: String,
    },

    /// Template rendering failed.
    ///
    /// This error occurs when custom changelog templates fail to render,
    /// possibly due to invalid syntax or missing variables.
    #[error("Template rendering failed: {reason}")]
    TemplateError {
        /// Description of the template error.
        reason: String,
    },

    /// Empty changelog section.
    ///
    /// This error occurs when attempting to generate a changelog but no
    /// entries are available for any section.
    #[error("Empty changelog for version '{version}': no commits to include")]
    EmptyChangelog {
        /// Version for which the changelog is empty.
        version: String,
    },

    /// Invalid configuration for changelog generation.
    ///
    /// This error occurs when the changelog configuration is invalid,
    /// incomplete, or contains conflicting settings.
    #[error("Invalid changelog configuration: {reason}")]
    InvalidConfig {
        /// Description of the configuration problem.
        reason: String,
    },

    /// Package not found for changelog generation.
    ///
    /// This error occurs when attempting to generate a changelog for a
    /// package that does not exist in the workspace.
    #[error("Package '{package}' not found in workspace")]
    PackageNotFound {
        /// Name of the package that was not found.
        package: String,
    },

    /// Failed to merge changelog sections.
    ///
    /// This error occurs when merging multiple changelog sections or
    /// combining changelogs from multiple packages fails.
    #[error("Failed to merge changelog sections: {reason}")]
    MergeError {
        /// Description of the merge error.
        reason: String,
    },

    /// Commit range is invalid or empty.
    ///
    /// This error occurs when the commit range for changelog generation
    /// contains no commits or is invalid.
    #[error("Invalid commit range for changelog: {reason}")]
    InvalidCommitRange {
        /// Description of why the range is invalid.
        reason: String,
    },

    /// Reference extraction failed.
    ///
    /// This error occurs when extracting issue/PR references from commit
    /// messages fails due to invalid patterns or parsing errors.
    #[error("Failed to extract references from commit message: {reason}")]
    ReferenceExtractionError {
        /// Description of the extraction error.
        reason: String,
    },

    /// Repository URL not configured.
    ///
    /// This error occurs when attempting to generate commit or issue links
    /// but no repository URL is configured.
    #[error("Repository URL not configured: cannot generate {link_type} links")]
    RepositoryUrlMissing {
        /// Type of link that cannot be generated (e.g., "commit", "issue").
        link_type: String,
    },

    /// Changelog update failed.
    ///
    /// This error occurs when updating an existing changelog file fails,
    /// possibly due to permission issues or file corruption.
    #[error("Failed to update changelog at '{path}': {reason}")]
    UpdateFailed {
        /// Path to the changelog file.
        path: PathBuf,
        /// Description of why the update failed.
        reason: String,
    },

    /// Unsupported changelog format.
    ///
    /// This error occurs when an unsupported or custom changelog format
    /// is requested without proper configuration.
    #[error("Unsupported changelog format: {format}")]
    UnsupportedFormat {
        /// The unsupported format name.
        format: String,
    },

    /// Breaking change detection failed.
    ///
    /// This error occurs when analyzing commits for breaking changes but
    /// detection fails due to parsing or pattern matching errors.
    #[error("Failed to detect breaking changes: {reason}")]
    BreakingChangeDetectionError {
        /// Description of the detection error.
        reason: String,
    },

    /// Changelog already exists for version.
    ///
    /// This error occurs when attempting to generate a changelog for a
    /// version that already has an entry in the changelog.
    #[error("Changelog already exists for version '{version}' in '{path}'")]
    ChangelogExists {
        /// The version that already exists.
        version: String,
        /// Path to the changelog file.
        path: PathBuf,
    },
}

impl AsRef<str> for ChangelogError {
    /// Returns a string representation of the error.
    ///
    /// This implementation enables the error to be used in contexts that require
    /// string references, such as logging or display operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let error = ChangelogError::NotFound {
    ///     path: PathBuf::from("CHANGELOG.md"),
    /// };
    ///
    /// let msg: &str = error.as_ref();
    /// assert!(msg.contains("not found"));
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Self::NotFound { .. } => "changelog not found",
            Self::ParseError { .. } => "changelog parse error",
            Self::InvalidFormat { .. } => "invalid changelog format",
            Self::GenerationFailed { .. } => "changelog generation failed",
            Self::ConventionalCommitParseError { .. } => "conventional commit parse error",
            Self::GitError { .. } => "git error",
            Self::VersionNotFound { .. } => "version not found",
            Self::InvalidVersion { .. } => "invalid version",
            Self::VersionTagParseError { .. } => "version tag parse error",
            Self::FileSystemError { .. } => "filesystem error",
            Self::InvalidPath { .. } => "invalid path",
            Self::TemplateError { .. } => "template error",
            Self::EmptyChangelog { .. } => "empty changelog",
            Self::InvalidConfig { .. } => "invalid configuration",
            Self::PackageNotFound { .. } => "package not found",
            Self::MergeError { .. } => "merge error",
            Self::InvalidCommitRange { .. } => "invalid commit range",
            Self::ReferenceExtractionError { .. } => "reference extraction error",
            Self::RepositoryUrlMissing { .. } => "repository url missing",
            Self::UpdateFailed { .. } => "update failed",
            Self::UnsupportedFormat { .. } => "unsupported format",
            Self::BreakingChangeDetectionError { .. } => "breaking change detection error",
            Self::ChangelogExists { .. } => "changelog exists",
        }
    }
}

impl ChangelogError {
    /// Returns whether this error is transient and might succeed on retry.
    ///
    /// Some changelog errors (like filesystem errors or git errors) might be
    /// recoverable through retry, while others (like parse errors) are not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangelogError;
    /// use std::path::PathBuf;
    ///
    /// let fs_error = ChangelogError::FileSystemError {
    ///     path: PathBuf::from("CHANGELOG.md"),
    ///     reason: "temporary lock".to_string(),
    /// };
    /// assert!(fs_error.is_transient());
    ///
    /// let parse_error = ChangelogError::ParseError {
    ///     line: 10,
    ///     reason: "invalid syntax".to_string(),
    /// };
    /// assert!(!parse_error.is_transient());
    /// ```
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::FileSystemError { .. } | Self::GitError { .. } | Self::UpdateFailed { .. }
        )
    }

    /// Returns whether this error is related to git operations.
    ///
    /// This helper method identifies errors that originate from git operations,
    /// useful for categorizing and handling git-specific errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let git_error = ChangelogError::GitError {
    ///     operation: "fetch tags".to_string(),
    ///     reason: "network error".to_string(),
    /// };
    /// assert!(git_error.is_git_related());
    ///
    /// let version_error = ChangelogError::VersionNotFound {
    ///     reason: "no tags found".to_string(),
    /// };
    /// assert!(version_error.is_git_related());
    /// ```
    #[must_use]
    pub fn is_git_related(&self) -> bool {
        matches!(
            self,
            Self::GitError { .. }
                | Self::VersionNotFound { .. }
                | Self::VersionTagParseError { .. }
                | Self::InvalidCommitRange { .. }
        )
    }

    /// Returns whether this error is related to parsing operations.
    ///
    /// This helper method identifies errors that occur during parsing of
    /// changelogs, commits, or versions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::ChangelogError;
    ///
    /// let parse_error = ChangelogError::ParseError {
    ///     line: 5,
    ///     reason: "unexpected format".to_string(),
    /// };
    /// assert!(parse_error.is_parse_related());
    ///
    /// let fs_error = ChangelogError::FileSystemError {
    ///     path: std::path::PathBuf::from("test"),
    ///     reason: "io error".to_string(),
    /// };
    /// assert!(!fs_error.is_parse_related());
    /// ```
    #[must_use]
    pub fn is_parse_related(&self) -> bool {
        matches!(
            self,
            Self::ParseError { .. }
                | Self::ConventionalCommitParseError { .. }
                | Self::VersionTagParseError { .. }
                | Self::InvalidFormat { .. }
        )
    }
}
