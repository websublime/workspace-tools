//! Error handling module for Node.js bindings.
//!
//! # What
//!
//! This module provides error types and conversion utilities for the NAPI layer.
//! It defines the `ErrorInfo` structure that represents errors in a Node.js-friendly
//! format with error codes following Node.js conventions (e.g., EVALIDATION, EGIT, ENOENT).
//!
//! # How
//!
//! The module provides:
//! - `ErrorInfo`: A NAPI-compatible struct with error code, message, context, and kind
//! - `ErrorCode`: An enumeration of all possible error codes
//! - Conversion traits from `CliError` to `ErrorInfo`
//!
//! Error codes follow Node.js conventions:
//! - `ECONFIG`: Configuration errors
//! - `EVALIDATION`: Parameter validation errors
//! - `EEXEC`: Execution errors
//! - `EGIT`: Git-related errors
//! - `EPKG`: Package-related errors
//! - `ENOENT`: File/path not found
//! - `EIO`: I/O errors
//! - `ENETWORK`: Network errors
//! - `EUSER`: User-caused errors
//! - `ETIMEOUT`: Timeout errors
//!
//! # Why
//!
//! Node.js developers expect error codes in a specific format. This module bridges
//! the gap between Rust's error handling and JavaScript's error conventions, providing
//! a consistent and familiar error interface for Node.js consumers.
//!
//! # Examples
//!
//! ```typescript
//! import { status } from '@websublime/workspace-tools';
//!
//! const result = await status({ root: '/invalid/path' });
//! if (!result.success) {
//!   console.error(`Error [${result.error.code}]: ${result.error.message}`);
//!   // Output: Error [ENOENT]: Path not found: /invalid/path
//! }
//! ```

use sublime_cli_tools::error::CliError;

/// Node.js-style error codes for categorizing errors.
///
/// These codes follow Node.js conventions and provide a familiar interface
/// for JavaScript/TypeScript developers to handle errors programmatically.
///
/// # Examples
///
/// ```rust
/// use sublime_node_tools::error::ErrorCode;
///
/// let code = ErrorCode::Validation;
/// assert_eq!(code.as_str(), "EVALIDATION");
/// ```
// Allow dead code for placeholder types that will be used in future stories
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorCode {
    /// Configuration-related errors (invalid config, config not found).
    Config,
    /// Parameter validation errors (invalid arguments, invalid state).
    Validation,
    /// Execution errors (command failed, operation failed).
    Execution,
    /// Git-related errors (repository not found, git operation failed).
    Git,
    /// Package-related errors (package not found, invalid package.json).
    Package,
    /// File or path not found errors.
    NotFound,
    /// I/O errors (permission denied, disk full).
    Io,
    /// Network errors (registry unreachable, download failed).
    Network,
    /// User-caused errors (invalid input, cancelled operation).
    User,
    /// Timeout errors (operation timed out).
    Timeout,
}

impl ErrorCode {
    /// Returns the string representation of the error code.
    ///
    /// These strings follow Node.js error code conventions (e.g., ENOENT, EIO).
    ///
    /// # Returns
    ///
    /// A static string slice containing the error code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_node_tools::error::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::NotFound.as_str(), "ENOENT");
    /// assert_eq!(ErrorCode::Io.as_str(), "EIO");
    /// assert_eq!(ErrorCode::Validation.as_str(), "EVALIDATION");
    /// ```
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Config => "ECONFIG",
            Self::Validation => "EVALIDATION",
            Self::Execution => "EEXEC",
            Self::Git => "EGIT",
            Self::Package => "EPKG",
            Self::NotFound => "ENOENT",
            Self::Io => "EIO",
            Self::Network => "ENETWORK",
            Self::User => "EUSER",
            Self::Timeout => "ETIMEOUT",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error information structure for Node.js bindings.
///
/// This structure is exposed to JavaScript/TypeScript via napi-rs and provides
/// detailed error information in a format familiar to Node.js developers.
///
/// # Fields
///
/// - `code`: Node.js-style error code (e.g., "EVALIDATION", "EGIT")
/// - `message`: Human-readable error message
/// - `context`: Optional additional context (field name, path, etc.)
/// - `kind`: Error category from the CLI layer
///
/// # Examples
///
/// ```typescript
/// // In JavaScript/TypeScript:
/// if (!result.success) {
///   const { code, message, context, kind } = result.error;
///   console.error(`[${code}] ${message}`);
///   if (context) {
///     console.error(`Context: ${context}`);
///   }
/// }
/// ```
// TODO: will be implemented on story 2.1
// The #[napi(object)] attribute will be added when implementing the full ErrorInfo
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ErrorInfo {
    /// Node.js-style error code (e.g., "EVALIDATION", "EGIT").
    ///
    /// These codes follow Node.js conventions and can be used for
    /// programmatic error handling in JavaScript/TypeScript.
    pub(crate) code: String,

    /// Human-readable error message.
    ///
    /// This message is suitable for displaying to end users and
    /// provides a clear description of what went wrong.
    pub(crate) message: String,

    /// Optional additional context for the error.
    ///
    /// This may contain the field name that caused a validation error,
    /// the path that was not found, or other relevant context information.
    pub(crate) context: Option<String>,

    /// Error category from the CLI layer.
    ///
    /// This corresponds to the `CliError` variant name (e.g., "Configuration",
    /// "Validation", "Git") and can be used for logging and debugging.
    pub(crate) kind: String,
}

#[allow(dead_code)]
impl ErrorInfo {
    /// Creates a new `ErrorInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `code` - The Node.js-style error code
    /// * `message` - Human-readable error message
    /// * `context` - Optional additional context
    /// * `kind` - Error category from CLI
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::new(
    ///     "EVALIDATION",
    ///     "Invalid package name",
    ///     Some("packages[0]"),
    ///     "Validation",
    /// );
    /// ```
    #[must_use]
    pub(crate) fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        context: Option<impl Into<String>>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: context.map(Into::into),
            kind: kind.into(),
        }
    }

    /// Creates an `ErrorInfo` for a validation error.
    ///
    /// # Arguments
    ///
    /// * `message` - The validation error message
    /// * `field` - Optional field name that caused the validation error
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EVALIDATION".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::validation("Package name cannot be empty", Some("packages"));
    /// assert_eq!(error.code, "EVALIDATION");
    /// ```
    #[must_use]
    pub(crate) fn validation(message: impl Into<String>, field: Option<impl Into<String>>) -> Self {
        Self::new(ErrorCode::Validation.as_str(), message, field, "Validation")
    }

    /// Creates an `ErrorInfo` for a configuration error.
    ///
    /// # Arguments
    ///
    /// * `message` - The configuration error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "ECONFIG".
    #[must_use]
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Config.as_str(), message, None::<String>, "Configuration")
    }

    /// Creates an `ErrorInfo` for a not found error.
    ///
    /// # Arguments
    ///
    /// * `message` - The not found error message
    /// * `path` - Optional path that was not found
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "ENOENT".
    #[must_use]
    pub(crate) fn not_found(message: impl Into<String>, path: Option<impl Into<String>>) -> Self {
        Self::new(ErrorCode::NotFound.as_str(), message, path, "Io")
    }

    /// Creates an `ErrorInfo` for a git error.
    ///
    /// # Arguments
    ///
    /// * `message` - The git error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EGIT".
    #[must_use]
    pub(crate) fn git(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Git.as_str(), message, None::<String>, "Git")
    }

    /// Creates an `ErrorInfo` for an execution error.
    ///
    /// # Arguments
    ///
    /// * `message` - The execution error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EEXEC".
    #[must_use]
    pub(crate) fn execution(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Execution.as_str(), message, None::<String>, "Execution")
    }

    /// Creates an `ErrorInfo` for a package error.
    ///
    /// # Arguments
    ///
    /// * `message` - The package error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EPKG".
    #[must_use]
    pub(crate) fn package(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Package.as_str(), message, None::<String>, "Package")
    }

    /// Creates an `ErrorInfo` for an I/O error.
    ///
    /// # Arguments
    ///
    /// * `message` - The I/O error message
    /// * `path` - Optional path related to the I/O error
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EIO".
    #[must_use]
    pub(crate) fn io(message: impl Into<String>, path: Option<impl Into<String>>) -> Self {
        Self::new(ErrorCode::Io.as_str(), message, path, "Io")
    }

    /// Creates an `ErrorInfo` for a network error.
    ///
    /// # Arguments
    ///
    /// * `message` - The network error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "ENETWORK".
    #[must_use]
    pub(crate) fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Network.as_str(), message, None::<String>, "Network")
    }

    /// Creates an `ErrorInfo` for a user error.
    ///
    /// # Arguments
    ///
    /// * `message` - The user error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "EUSER".
    #[must_use]
    pub(crate) fn user(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::User.as_str(), message, None::<String>, "User")
    }

    /// Creates an `ErrorInfo` for a timeout error.
    ///
    /// # Arguments
    ///
    /// * `message` - The timeout error message
    ///
    /// # Returns
    ///
    /// A new `ErrorInfo` with code "ETIMEOUT".
    #[must_use]
    pub(crate) fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout.as_str(), message, None::<String>, "Timeout")
    }
}

impl From<&CliError> for ErrorInfo {
    /// Converts a `CliError` reference to an `ErrorInfo`.
    ///
    /// This conversion maps CLI error variants to appropriate Node.js-style
    /// error codes while preserving the original error message and kind.
    ///
    /// # Mapping
    ///
    /// | CliError Variant | ErrorCode |
    /// |------------------|-----------|
    /// | Configuration    | ECONFIG   |
    /// | Validation       | EVALIDATION |
    /// | Execution        | EEXEC     |
    /// | Git              | EGIT      |
    /// | Package          | EPKG      |
    /// | Io               | EIO       |
    /// | Network          | ENETWORK  |
    /// | User             | EUSER     |
    fn from(error: &CliError) -> Self {
        let code = match error {
            CliError::Configuration(_) => ErrorCode::Config,
            CliError::Validation(_) => ErrorCode::Validation,
            CliError::Execution(_) => ErrorCode::Execution,
            CliError::Git(_) => ErrorCode::Git,
            CliError::Package(_) => ErrorCode::Package,
            CliError::Io(_) => ErrorCode::Io,
            CliError::Network(_) => ErrorCode::Network,
            CliError::User(_) => ErrorCode::User,
        };

        Self::new(code.as_str(), error.to_string(), None::<String>, error.kind())
    }
}

impl From<CliError> for ErrorInfo {
    /// Converts a `CliError` to an `ErrorInfo`.
    ///
    /// This is a convenience implementation that delegates to the reference
    /// conversion implementation.
    fn from(error: CliError) -> Self {
        Self::from(&error)
    }
}
