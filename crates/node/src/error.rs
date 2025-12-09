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
//! - `ErrorCode`: An internal enumeration of all possible error codes
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

use serde::Serialize;
use sublime_cli_tools::error::CliError;

/// Node.js-style error codes for categorizing errors.
///
/// These codes follow Node.js conventions and provide a familiar interface
/// for JavaScript/TypeScript developers to handle errors programmatically.
///
/// This enum is used internally to generate the appropriate error code strings.
/// The string representations are exposed to JavaScript via the `ErrorInfo.code` field.
///
/// # Error Code Mapping
///
/// | ErrorCode   | String       | Description                      |
/// |-------------|--------------|----------------------------------|
/// | Config      | "ECONFIG"    | Configuration-related errors     |
/// | Validation  | "EVALIDATION"| Parameter validation errors      |
/// | Execution   | "EEXEC"      | Execution/command errors         |
/// | Git         | "EGIT"       | Git-related errors               |
/// | Package     | "EPKG"       | Package-related errors           |
/// | NotFound    | "ENOENT"     | File or path not found           |
/// | Io          | "EIO"        | Generic I/O errors               |
/// | Network     | "ENETWORK"   | Network-related errors           |
/// | User        | "EUSER"      | User-caused errors               |
/// | Timeout     | "ETIMEOUT"   | Timeout errors                   |
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::error::ErrorCode;
///
/// let code = ErrorCode::Validation;
/// assert_eq!(code.as_str(), "EVALIDATION");
/// ```
// Allow dead_code for Timeout variant - TODO: will be used on story 6.3 (execute command with timeout)
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
    /// ```rust,ignore
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
/// The `#[napi(object)]` attribute enables automatic conversion to JavaScript
/// objects and TypeScript type generation.
///
/// # Fields
///
/// - `code`: Node.js-style error code (e.g., "EVALIDATION", "EGIT")
/// - `message`: Human-readable error message
/// - `context`: Optional additional context (field name, path, etc.)
/// - `kind`: Error category from the CLI layer
///
/// # TypeScript Definition
///
/// ```typescript
/// export interface ErrorInfo {
///   Node.js-style error code (e.g., "EVALIDATION", "EGIT")
///   code: string;
///   Human-readable error message
///   message: string;
///   Optional additional context about the error
///   context?: string;
///   Error category from CLI (for debugging)
///   kind: string;
/// }
/// ```
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
#[napi(object)]
#[derive(Debug, Clone, Serialize)]
pub struct ErrorInfo {
    /// Node.js-style error code (e.g., "EVALIDATION", "EGIT").
    ///
    /// These codes follow Node.js conventions and can be used for
    /// programmatic error handling in JavaScript/TypeScript.
    ///
    /// # Available Codes
    ///
    /// - `ECONFIG`: Configuration errors
    /// - `EVALIDATION`: Validation errors
    /// - `EEXEC`: Execution errors
    /// - `EGIT`: Git errors
    /// - `EPKG`: Package errors
    /// - `ENOENT`: File/path not found
    /// - `EIO`: I/O errors
    /// - `ENETWORK`: Network errors
    /// - `EUSER`: User errors
    /// - `ETIMEOUT`: Timeout errors
    pub code: String,

    /// Human-readable error message.
    ///
    /// This message is suitable for displaying to end users and
    /// provides a clear description of what went wrong.
    pub message: String,

    /// Optional additional context for the error.
    ///
    /// This may contain the field name that caused a validation error,
    /// the path that was not found, or other relevant context information.
    pub context: Option<String>,

    /// Error category from the CLI layer.
    ///
    /// This corresponds to the `CliError` variant name (e.g., "Configuration",
    /// "Validation", "Git") and can be used for logging and debugging.
    pub kind: String,
}

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
    /// ```rust,ignore
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
    pub fn new(
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
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::validation("Package name cannot be empty", Some("packages"));
    /// assert_eq!(error.code, "EVALIDATION");
    /// ```
    #[must_use]
    pub fn validation(message: impl Into<String>, field: Option<impl Into<String>>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::configuration("Config file not found");
    /// assert_eq!(error.code, "ECONFIG");
    /// ```
    #[must_use]
    // TODO: will be used on story 7.2 (configShow command)
    #[allow(dead_code)]
    pub fn configuration(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::not_found("File not found", Some("/path/to/file"));
    /// assert_eq!(error.code, "ENOENT");
    /// ```
    #[must_use]
    pub fn not_found(message: impl Into<String>, path: Option<impl Into<String>>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::git("Repository not found");
    /// assert_eq!(error.code, "EGIT");
    /// ```
    #[must_use]
    // TODO: will be used on story 3.2 (status command)
    #[allow(dead_code)]
    pub fn git(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::execution("Command failed to execute");
    /// assert_eq!(error.code, "EEXEC");
    /// ```
    #[must_use]
    // TODO: will be used on story 6.3 (execute command)
    #[allow(dead_code)]
    pub fn execution(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::package("Package not found in workspace");
    /// assert_eq!(error.code, "EPKG");
    /// ```
    #[must_use]
    // TODO: will be used on story 4.2 (changesetAdd command)
    #[allow(dead_code)]
    pub fn package(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::io("Permission denied", Some("/etc/passwd"));
    /// assert_eq!(error.code, "EIO");
    /// ```
    #[must_use]
    // TODO: will be used on story 3.2 (status command)
    #[allow(dead_code)]
    pub fn io(message: impl Into<String>, path: Option<impl Into<String>>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::network("Registry unreachable");
    /// assert_eq!(error.code, "ENETWORK");
    /// ```
    #[must_use]
    // TODO: will be used on story 8.2 (upgradeCheck command)
    #[allow(dead_code)]
    pub fn network(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::user("Operation cancelled by user");
    /// assert_eq!(error.code, "EUSER");
    /// ```
    #[must_use]
    // TODO: will be used on story 4.6 (changesetRemove command)
    #[allow(dead_code)]
    pub fn user(message: impl Into<String>) -> Self {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::timeout("Operation exceeded 30 second timeout");
    /// assert_eq!(error.code, "ETIMEOUT");
    /// ```
    #[must_use]
    // TODO: will be used on story 6.3 (execute command with timeout)
    #[allow(dead_code)]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout.as_str(), message, None::<String>, "Timeout")
    }
}

/// Determines if an I/O error message indicates a "not found" condition.
///
/// This function checks for common patterns in error messages that indicate
/// a file, path, or resource was not found. This is used to distinguish
/// between ENOENT (not found) and EIO (generic I/O error) codes.
///
/// # Arguments
///
/// * `message` - The error message to check
///
/// # Returns
///
/// `true` if the message indicates a "not found" condition, `false` otherwise.
///
/// # Patterns Detected
///
/// - "not found" (case insensitive)
/// - "No such file" (case insensitive)
/// - "does not exist" (case insensitive)
/// - "doesn't exist" (case insensitive)
fn is_not_found_error(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("not found")
        || lower.contains("no such file")
        || lower.contains("does not exist")
        || lower.contains("doesn't exist")
}

impl From<&CliError> for ErrorInfo {
    /// Converts a `CliError` reference to an `ErrorInfo`.
    ///
    /// This conversion maps CLI error variants to appropriate Node.js-style
    /// error codes while preserving the original error message and kind.
    ///
    /// # Mapping
    ///
    /// | CliError Variant | ErrorCode   | Notes                                    |
    /// |------------------|-------------|------------------------------------------|
    /// | Configuration    | ECONFIG     | Configuration file/parsing errors        |
    /// | Validation       | EVALIDATION | Invalid arguments/parameters             |
    /// | Execution        | EEXEC       | Command execution failures               |
    /// | Git              | EGIT        | Git repository/operation errors          |
    /// | Package          | EPKG        | Package.json/dependency errors           |
    /// | Io               | ENOENT/EIO  | ENOENT if "not found", otherwise EIO     |
    /// | Network          | ENETWORK    | Network/registry errors                  |
    /// | User             | EUSER       | User-caused/cancelled errors             |
    ///
    /// # I/O Error Differentiation
    ///
    /// The `Io` variant is specially handled to differentiate between:
    /// - `ENOENT`: When the error message contains "not found", "No such file",
    ///   "does not exist", or "doesn't exist"
    /// - `EIO`: For all other I/O errors (permission denied, disk full, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_cli_tools::error::CliError;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let cli_error = CliError::validation("Invalid version format");
    /// let error_info = ErrorInfo::from(&cli_error);
    /// assert_eq!(error_info.code, "EVALIDATION");
    /// assert_eq!(error_info.kind, "Validation");
    ///
    /// // I/O not found errors map to ENOENT
    /// let io_error = CliError::io("File not found: /path/to/file");
    /// let error_info = ErrorInfo::from(&io_error);
    /// assert_eq!(error_info.code, "ENOENT");
    ///
    /// // Other I/O errors map to EIO
    /// let io_error = CliError::io("Permission denied");
    /// let error_info = ErrorInfo::from(&io_error);
    /// assert_eq!(error_info.code, "EIO");
    /// ```
    fn from(error: &CliError) -> Self {
        let (code, kind) = match error {
            CliError::Configuration(_) => (ErrorCode::Config, "Configuration"),
            CliError::Validation(_) => (ErrorCode::Validation, "Validation"),
            CliError::Execution(_) => (ErrorCode::Execution, "Execution"),
            CliError::Git(_) => (ErrorCode::Git, "Git"),
            CliError::Package(_) => (ErrorCode::Package, "Package"),
            CliError::Io(msg) => {
                if is_not_found_error(msg) {
                    (ErrorCode::NotFound, "Io")
                } else {
                    (ErrorCode::Io, "Io")
                }
            }
            CliError::Network(_) => (ErrorCode::Network, "Network"),
            CliError::User(_) => (ErrorCode::User, "User"),
        };

        Self::new(code.as_str(), error.to_string(), None::<String>, kind)
    }
}

impl From<CliError> for ErrorInfo {
    /// Converts a `CliError` to an `ErrorInfo`.
    ///
    /// This is a convenience implementation that delegates to the reference
    /// conversion implementation.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_cli_tools::error::CliError;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let cli_error = CliError::git("Repository not found");
    /// let error_info = ErrorInfo::from(cli_error);
    /// assert_eq!(error_info.code, "EGIT");
    /// ```
    fn from(error: CliError) -> Self {
        Self::from(&error)
    }
}
