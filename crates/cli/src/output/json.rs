//! JSON response structures and utilities.
//!
//! This module provides the standard JSON response format used by all CLI commands
//! when `--format json` is specified. It ensures consistency across the entire CLI.
//!
//! # What
//!
//! Provides:
//! - `JsonResponse<T>` - Standard response wrapper for all commands
//! - Helper functions for creating success/error responses
//! - Consistent field naming and structure
//!
//! # How
//!
//! All commands that support JSON output must use `JsonResponse<T>` to wrap their
//! data. The structure always includes a `success` field, and conditionally includes
//! `data` (on success) or `error` (on failure).
//!
//! # Why
//!
//! Consistency in JSON output makes the CLI more predictable and easier to integrate
//! with scripts and tools. The standard format allows downstream tools to handle
//! responses uniformly without needing command-specific parsing logic.
//!
//! # Examples
//!
//! Success response:
//!
//! ```rust
//! use sublime_cli_tools::output::JsonResponse;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct CommandResult {
//!     files_processed: usize,
//!     packages: Vec<String>,
//! }
//!
//! let result = CommandResult {
//!     files_processed: 5,
//!     packages: vec!["@org/core".to_string()],
//! };
//!
//! let response = JsonResponse::success(result);
//! assert!(response.success);
//! ```
//!
//! Error response:
//!
//! ```rust
//! use sublime_cli_tools::output::JsonResponse;
//!
//! let response: JsonResponse<()> = JsonResponse::error("Configuration not found".to_string());
//! assert!(!response.success);
//! assert!(response.error.is_some());
//! ```

use serde::Serialize;

/// Standard JSON response structure for all CLI commands.
///
/// This structure provides a consistent format for all command outputs when
/// JSON format is requested. Every command must wrap its output data in this
/// structure to ensure API consistency.
///
/// # Structure
///
/// ```json
/// {
///   "success": true,
///   "data": { ... }
/// }
/// ```
///
/// Or on error:
///
/// ```json
/// {
///   "success": false,
///   "error": "Error message"
/// }
/// ```
///
/// # Examples
///
/// Creating a success response:
///
/// ```rust
/// use sublime_cli_tools::output::JsonResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct InitResult {
///     config_file: String,
///     strategy: String,
/// }
///
/// let result = InitResult {
///     config_file: "repo.config.yaml".to_string(),
///     strategy: "independent".to_string(),
/// };
///
/// let response = JsonResponse::success(result);
/// assert_eq!(response.success, true);
/// assert!(response.data.is_some());
/// assert!(response.error.is_none());
/// ```
///
/// Creating an error response:
///
/// ```rust
/// use sublime_cli_tools::output::JsonResponse;
///
/// let response: JsonResponse<()> = JsonResponse::error(
///     "Failed to initialize: directory is not empty".to_string()
/// );
///
/// assert_eq!(response.success, false);
/// assert!(response.data.is_none());
/// assert!(response.error.is_some());
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JsonResponse<T> {
    /// Indicates whether the operation was successful.
    ///
    /// - `true`: Operation completed successfully, `data` field will be present
    /// - `false`: Operation failed, `error` field will be present
    pub success: bool,

    /// The response data (only present when `success` is `true`).
    ///
    /// This field contains the command-specific output data. It is omitted
    /// from the JSON output when `None` to keep responses clean.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// The error message (only present when `success` is `false`).
    ///
    /// This field contains a human-readable error message explaining what
    /// went wrong. It is omitted from the JSON output when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> JsonResponse<T> {
    /// Creates a successful response with data.
    ///
    /// The resulting JSON will include `success: true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct PackageInfo {
    ///     name: String,
    ///     version: String,
    /// }
    ///
    /// let info = PackageInfo {
    ///     name: "@org/core".to_string(),
    ///     version: "1.2.3".to_string(),
    /// };
    ///
    /// let response = JsonResponse::success(info);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// assert!(response.error.is_none());
    /// ```
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates an error response with a message.
    ///
    /// The resulting JSON will include `success: false` and the provided error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let response: JsonResponse<String> = JsonResponse::error(
    ///     "Package not found in workspace".to_string()
    /// );
    ///
    /// assert!(!response.success);
    /// assert!(response.data.is_none());
    /// assert_eq!(response.error, Some("Package not found in workspace".to_string()));
    /// ```
    pub fn error(message: String) -> Self {
        Self { success: false, data: None, error: Some(message) }
    }

    /// Returns whether this response represents a success.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let success_response = JsonResponse::success("data");
    /// assert!(success_response.is_success());
    ///
    /// let error_response: JsonResponse<String> = JsonResponse::error("failed".to_string());
    /// assert!(!error_response.is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let success_response = JsonResponse::success("data");
    /// assert!(!success_response.is_error());
    ///
    /// let error_response: JsonResponse<String> = JsonResponse::error("failed".to_string());
    /// assert!(error_response.is_error());
    /// ```
    pub fn is_error(&self) -> bool {
        !self.success
    }
}

impl<T> Default for JsonResponse<T> {
    /// Creates a default error response.
    ///
    /// This is primarily used for initialization purposes and represents
    /// an error state with a generic message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let response: JsonResponse<String> = JsonResponse::default();
    /// assert!(!response.success);
    /// assert_eq!(response.error, Some("Unknown error".to_string()));
    /// ```
    fn default() -> Self {
        Self { success: false, data: None, error: Some("Unknown error".to_string()) }
    }
}
