//! API response wrapper module for Node.js bindings.
//!
//! # What
//!
//! This module provides the `ApiResponse<T>` wrapper type that encapsulates
//! success/failure states for all NAPI function responses. It also provides
//! helper methods for creating responses from various error types including
//! `std::io::Error`, `CliError`, and `ErrorInfo`.
//!
//! Additionally, it re-exports `JsonResponse<T>` from the CLI crate for internal
//! use when parsing CLI output.
//!
//! # How
//!
//! The module provides:
//! - `ApiResponse<T>`: A generic wrapper with `success`, `data`, and `error` fields
//! - Helper methods: `success()`, `failure()`, `failure_from_io()`, `failure_from_cli()`
//! - Extension trait `ApiResponseExt` for `JsonResponse<T>` with NAPI-specific helpers
//! - `result_to_response()` utility for converting `Result<T, ErrorInfo>` to `JsonResponse<T>`
//!
//! The `ApiResponse<T>` struct uses `serde::Serialize` for JSON serialization when
//! returned from async NAPI functions. Note that `#[napi(object)]` cannot be used
//! on generic structs due to napi-rs limitations; however, the concrete inner types
//! (like `ErrorInfo`) do use `#[napi(object)]` to generate proper TypeScript definitions.
//!
//! # Why
//!
//! A consistent response wrapper provides:
//! - Predictable API contract for JavaScript/TypeScript consumers
//! - Type-safe error handling with structured error information
//! - Clear distinction between success and failure states
//! - Familiar patterns for Node.js developers
//!
//! Using the existing `JsonResponse<T>` from CLI ensures consistency between
//! CLI JSON output and NAPI responses when parsing CLI command output internally.
//!
//! # TypeScript Definition
//!
//! The corresponding TypeScript interface for `ApiResponse<T>`:
//!
//! ```typescript
//! export interface ApiResponse<T> {
//!   /** Whether the operation succeeded */
//!   success: boolean;
//!   /** The response data (present when success is true) */
//!   data?: T;
//!   /** Error information (present when success is false) */
//!   error?: ErrorInfo;
//! }
//! ```
//!
//! # Examples
//!
//! ## Creating Successful Responses
//!
//! ```rust,ignore
//! use sublime_node_tools::response::ApiResponse;
//!
//! #[derive(Serialize)]
//! struct StatusData {
//!     packages: Vec<String>,
//! }
//!
//! let data = StatusData { packages: vec!["@scope/pkg1".to_string()] };
//! let response = ApiResponse::success(data);
//! assert!(response.success);
//! assert!(response.data.is_some());
//! assert!(response.error.is_none());
//! ```
//!
//! ## Creating Failure Responses
//!
//! ```rust,ignore
//! use sublime_node_tools::response::ApiResponse;
//! use sublime_node_tools::error::ErrorInfo;
//!
//! let error = ErrorInfo::validation("Invalid package name", Some("packages[0]"));
//! let response: ApiResponse<()> = ApiResponse::failure(error);
//! assert!(!response.success);
//! assert!(response.data.is_none());
//! assert!(response.error.is_some());
//! ```
//!
//! ## JavaScript/TypeScript Usage
//!
//! ```typescript
//! import { status } from '@websublime/workspace-tools';
//!
//! const result = await status({ root: '.' });
//!
//! if (result.success) {
//!   // TypeScript knows result.data is StatusData here
//!   console.log(result.data.packages);
//! } else {
//!   // TypeScript knows result.error is ErrorInfo here
//!   const { code, message, context, kind } = result.error;
//!   console.error(`[${code}] ${message}`);
//! }
//! ```

use serde::Serialize;

use crate::error::ErrorInfo;
use sublime_cli_tools::error::CliError;

// Re-export JsonResponse from CLI for internal parsing consistency
pub(crate) use sublime_cli_tools::output::JsonResponse;

/// Standard API response wrapper for all NAPI functions.
///
/// This structure provides a consistent response format for all NAPI functions,
/// encapsulating both success and failure states. All async NAPI functions should
/// return this type (with an appropriate data type `T`).
///
/// # Design Notes
///
/// This struct uses `serde::Serialize` instead of `#[napi(object)]` because napi-rs
/// does not support generic structs with the `#[napi(object)]` derive. When returned
/// from async NAPI functions, napi-rs automatically serializes the struct to a
/// JavaScript object. The concrete inner types (`ErrorInfo`) use `#[napi(object)]`
/// to ensure proper TypeScript type generation.
///
/// # Type Parameters
///
/// * `T` - The type of the response data. Must implement `Serialize`.
///
/// # Fields
///
/// * `success` - `true` if the operation succeeded, `false` otherwise
/// * `data` - The response data (present when `success` is `true`)
/// * `error` - Error information (present when `success` is `false`)
///
/// # Invariants
///
/// While not enforced at the type level, the following invariants should hold:
/// - When `success` is `true`, `data` should be `Some` and `error` should be `None`
/// - When `success` is `false`, `data` should be `None` and `error` should be `Some`
///
/// The helper methods `success()` and `failure()` ensure these invariants.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::response::ApiResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyData {
///     value: String,
/// }
///
/// // Success case
/// let success_response = ApiResponse::success(MyData { value: "test".to_string() });
/// assert!(success_response.success);
///
/// // Failure case
/// use sublime_node_tools::error::ErrorInfo;
/// let error = ErrorInfo::validation("Invalid input", None::<String>);
/// let failure_response: ApiResponse<MyData> = ApiResponse::failure(error);
/// assert!(!failure_response.success);
/// ```
// Used by status command (Story 3.2) and all subsequent command implementations
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    /// Whether the operation succeeded.
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

    /// Error information (only present when `success` is `false`).
    ///
    /// This field contains structured error information including a Node.js-style
    /// error code, message, optional context, and error kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[allow(dead_code)]
impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Creates a successful response with data.
    ///
    /// This is the primary constructor for success cases. It sets `success` to `true`,
    /// provides the data, and sets `error` to `None`.
    ///
    /// # Arguments
    ///
    /// * `data` - The response data to include
    ///
    /// # Returns
    ///
    /// A new `ApiResponse` with `success = true` and the provided data.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
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
    /// let response = ApiResponse::success(info);
    /// assert!(response.success);
    /// assert!(response.data.is_some());
    /// assert!(response.error.is_none());
    /// ```
    #[must_use]
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates a failed response with error information.
    ///
    /// This is the primary constructor for failure cases. It sets `success` to `false`,
    /// sets `data` to `None`, and includes the provided error information.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to include
    ///
    /// # Returns
    ///
    /// A new `ApiResponse` with `success = false` and the provided error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let error = ErrorInfo::validation("Invalid input", Some("field_name"));
    /// let response: ApiResponse<String> = ApiResponse::failure(error);
    ///
    /// assert!(!response.success);
    /// assert!(response.data.is_none());
    /// assert!(response.error.is_some());
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    /// ```
    #[must_use]
    pub fn failure(error: ErrorInfo) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    /// Creates a failed response from an `std::io::Error`.
    ///
    /// This helper method converts a standard I/O error into an appropriate
    /// `ApiResponse` with the correct error code. The error code is determined
    /// by the `ErrorKind`:
    /// - `NotFound` → `ENOENT`
    /// - All others → `EIO`
    ///
    /// # Arguments
    ///
    /// * `error` - The I/O error to convert
    ///
    /// # Returns
    ///
    /// A new `ApiResponse` with `success = false` and the converted error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use std::io::{Error, ErrorKind};
    ///
    /// // Not found error becomes ENOENT
    /// let io_error = Error::new(ErrorKind::NotFound, "File not found: config.json");
    /// let response: ApiResponse<()> = ApiResponse::failure_from_io(io_error);
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));
    ///
    /// // Permission denied becomes EIO
    /// let io_error = Error::new(ErrorKind::PermissionDenied, "Access denied");
    /// let response: ApiResponse<()> = ApiResponse::failure_from_io(io_error);
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EIO"));
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn failure_from_io(error: std::io::Error) -> Self {
        let error_info = match error.kind() {
            std::io::ErrorKind::NotFound => ErrorInfo::not_found(error.to_string(), None::<String>),
            std::io::ErrorKind::PermissionDenied => {
                ErrorInfo::io(format!("Permission denied: {error}"), None::<String>)
            }
            std::io::ErrorKind::AlreadyExists => {
                ErrorInfo::io(format!("Already exists: {error}"), None::<String>)
            }
            std::io::ErrorKind::InvalidInput => {
                ErrorInfo::validation(error.to_string(), None::<String>)
            }
            std::io::ErrorKind::InvalidData => {
                ErrorInfo::validation(format!("Invalid data: {error}"), None::<String>)
            }
            std::io::ErrorKind::TimedOut => ErrorInfo::timeout(error.to_string()),
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected => ErrorInfo::network(error.to_string()),
            _ => ErrorInfo::io(error.to_string(), None::<String>),
        };

        Self::failure(error_info)
    }

    /// Creates a failed response from a `CliError`.
    ///
    /// This helper method converts a CLI error into an appropriate `ApiResponse`.
    /// The conversion uses the `From<CliError> for ErrorInfo` implementation
    /// to map CLI error variants to Node.js-style error codes.
    ///
    /// # Arguments
    ///
    /// * `error` - The CLI error to convert
    ///
    /// # Returns
    ///
    /// A new `ApiResponse` with `success = false` and the converted error.
    ///
    /// # Error Code Mapping
    ///
    /// | CliError Variant | Error Code    |
    /// |------------------|---------------|
    /// | Configuration    | ECONFIG       |
    /// | Validation       | EVALIDATION   |
    /// | Execution        | EEXEC         |
    /// | Git              | EGIT          |
    /// | Package          | EPKG          |
    /// | Io (not found)   | ENOENT        |
    /// | Io (other)       | EIO           |
    /// | Network          | ENETWORK      |
    /// | User             | EUSER         |
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let cli_error = CliError::validation("Invalid package name");
    /// let response: ApiResponse<()> = ApiResponse::failure_from_cli(cli_error);
    ///
    /// assert!(!response.success);
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    /// assert_eq!(response.error.as_ref().map(|e| e.kind.as_str()), Some("Validation"));
    /// ```
    #[must_use]
    pub fn failure_from_cli(error: CliError) -> Self {
        Self::failure(ErrorInfo::from(error))
    }

    /// Returns whether this response represents a success.
    ///
    /// # Returns
    ///
    /// `true` if the operation succeeded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let success_response = ApiResponse::success("data".to_string());
    /// assert!(success_response.is_success());
    ///
    /// let error = ErrorInfo::validation("error", None::<String>);
    /// let failure_response: ApiResponse<String> = ApiResponse::failure(error);
    /// assert!(!failure_response.is_success());
    /// ```
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Returns whether this response represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the operation failed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let success_response = ApiResponse::success("data".to_string());
    /// assert!(!success_response.is_failure());
    ///
    /// let error = ErrorInfo::validation("error", None::<String>);
    /// let failure_response: ApiResponse<String> = ApiResponse::failure(error);
    /// assert!(failure_response.is_failure());
    /// ```
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Maps the data of a successful response to a new type.
    ///
    /// If this is a success response, applies the function to the data and returns
    /// a new success response with the transformed data. If this is a failure
    /// response, returns a failure response with the same error.
    ///
    /// # Type Parameters
    ///
    /// * `U` - The target data type
    /// * `F` - The mapping function type
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms the data from type `T` to type `U`
    ///
    /// # Returns
    ///
    /// A new `ApiResponse<U>` with either the transformed data or the original error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    ///
    /// let response = ApiResponse::success(42i32);
    /// let mapped = response.map(|n| n.to_string());
    ///
    /// assert!(mapped.is_success());
    /// assert_eq!(mapped.data, Some("42".to_string()));
    /// ```
    #[must_use]
    pub fn map<U, F>(self, f: F) -> ApiResponse<U>
    where
        U: Serialize,
        F: FnOnce(T) -> U,
    {
        match self.data {
            Some(data) if self.success => ApiResponse::success(f(data)),
            _ => ApiResponse { success: false, data: None, error: self.error },
        }
    }

    /// Converts the response into a `Result<T, ErrorInfo>`.
    ///
    /// This method provides interoperability with Rust's standard `Result` type,
    /// allowing `ApiResponse` to be used with the `?` operator in functions that
    /// return `Result<T, ErrorInfo>`.
    ///
    /// # Returns
    ///
    /// - `Ok(data)` if the response is successful and contains data
    /// - `Err(error)` if the response is a failure and contains an error
    /// - `Err(ErrorInfo::execution("..."))` if the response is malformed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// fn process() -> Result<String, ErrorInfo> {
    ///     let response = ApiResponse::success("data".to_string());
    ///     response.into_result()
    /// }
    ///
    /// assert_eq!(process().ok(), Some("data".to_string()));
    /// ```
    pub fn into_result(self) -> Result<T, ErrorInfo> {
        if self.success {
            self.data.ok_or_else(|| {
                ErrorInfo::execution("Malformed response: success=true but data is None")
            })
        } else {
            Err(self.error.unwrap_or_else(|| {
                ErrorInfo::execution("Malformed response: success=false but error is None")
            }))
        }
    }
}

impl<T> From<Result<T, ErrorInfo>> for ApiResponse<T>
where
    T: Serialize,
{
    /// Converts a `Result<T, ErrorInfo>` into an `ApiResponse<T>`.
    ///
    /// This provides a convenient way to convert from Rust's `Result` pattern
    /// to the `ApiResponse` format used by NAPI functions.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to convert
    ///
    /// # Returns
    ///
    /// An `ApiResponse` representing the success or failure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_node_tools::error::ErrorInfo;
    ///
    /// let ok_result: Result<String, ErrorInfo> = Ok("success".to_string());
    /// let response: ApiResponse<String> = ok_result.into();
    /// assert!(response.is_success());
    ///
    /// let err_result: Result<String, ErrorInfo> = Err(ErrorInfo::validation("error", None::<String>));
    /// let response: ApiResponse<String> = err_result.into();
    /// assert!(response.is_failure());
    /// ```
    fn from(result: Result<T, ErrorInfo>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::failure(error),
        }
    }
}

impl<T> From<Result<T, CliError>> for ApiResponse<T>
where
    T: Serialize,
{
    /// Converts a `Result<T, CliError>` into an `ApiResponse<T>`.
    ///
    /// This provides a convenient way to convert from CLI operation results
    /// to the `ApiResponse` format used by NAPI functions.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to convert
    ///
    /// # Returns
    ///
    /// An `ApiResponse` representing the success or failure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use sublime_cli_tools::error::CliError;
    ///
    /// let ok_result: Result<String, CliError> = Ok("success".to_string());
    /// let response: ApiResponse<String> = ok_result.into();
    /// assert!(response.is_success());
    ///
    /// let err_result: Result<String, CliError> = Err(CliError::validation("error"));
    /// let response: ApiResponse<String> = err_result.into();
    /// assert!(response.is_failure());
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("EVALIDATION"));
    /// ```
    fn from(result: Result<T, CliError>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::failure_from_cli(error),
        }
    }
}

impl<T> From<Result<T, std::io::Error>> for ApiResponse<T>
where
    T: Serialize,
{
    /// Converts a `Result<T, std::io::Error>` into an `ApiResponse<T>`.
    ///
    /// This provides a convenient way to convert from I/O operation results
    /// to the `ApiResponse` format used by NAPI functions.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to convert
    ///
    /// # Returns
    ///
    /// An `ApiResponse` representing the success or failure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_node_tools::response::ApiResponse;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let ok_result: Result<String, Error> = Ok("success".to_string());
    /// let response: ApiResponse<String> = ok_result.into();
    /// assert!(response.is_success());
    ///
    /// let err_result: Result<String, Error> = Err(Error::new(ErrorKind::NotFound, "file not found"));
    /// let response: ApiResponse<String> = err_result.into();
    /// assert!(response.is_failure());
    /// assert_eq!(response.error.as_ref().map(|e| e.code.as_str()), Some("ENOENT"));
    /// ```
    fn from(result: Result<T, std::io::Error>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::failure_from_io(error),
        }
    }
}

/// Extension trait for `JsonResponse` with NAPI-specific helpers.
///
/// This trait provides additional methods useful in the NAPI context,
/// such as creating responses from `ErrorInfo` and other NAPI types.
///
/// This trait is used for internal processing when parsing CLI command output
/// that uses `JsonResponse<T>` format. It bridges the gap between CLI's string-based
/// errors and NAPI's structured `ErrorInfo` errors.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::response::{ApiResponseExt, JsonResponse};
/// use sublime_node_tools::error::ErrorInfo;
///
/// let error = ErrorInfo::validation("Invalid input", Some("field"));
/// let response: JsonResponse<String> = JsonResponse::from_error_info(error);
/// assert!(response.is_error());
/// ```
// Used by status command (Story 3.2) for parsing CLI JSON output
#[allow(dead_code)]
pub(crate) trait ApiResponseExt<T> {
    /// Creates an error response from an `ErrorInfo`.
    ///
    /// This converts the structured `ErrorInfo` into the string format
    /// expected by `JsonResponse`.
    ///
    /// # Arguments
    ///
    /// * `error` - The error information to convert
    ///
    /// # Returns
    ///
    /// A `JsonResponse` with `success = false` and the error message.
    fn from_error_info(error: ErrorInfo) -> Self;

    /// Creates an error response from a validation error.
    ///
    /// Convenience method for creating validation error responses.
    ///
    /// # Arguments
    ///
    /// * `message` - The validation error message
    /// * `field` - Optional field name that caused the error
    ///
    /// # Returns
    ///
    /// A `JsonResponse` with `success = false` and the formatted error.
    fn validation_error(message: impl Into<String>, field: Option<&str>) -> Self;
}

impl<T> ApiResponseExt<T> for JsonResponse<T> {
    fn from_error_info(error: ErrorInfo) -> Self {
        // Format error with code for consistency
        let message = format!("[{}] {}", error.code, error.message);
        Self::error(message)
    }

    fn validation_error(message: impl Into<String>, field: Option<&str>) -> Self {
        let msg = message.into();
        let formatted = match field {
            Some(f) => format!("[EVALIDATION] {f}: {msg}"),
            None => format!("[EVALIDATION] {msg}"),
        };
        Self::error(formatted)
    }
}

/// Converts a `Result<T, ErrorInfo>` into a `JsonResponse<T>`.
///
/// This utility function provides a convenient way to convert from
/// Rust's standard Result pattern to the JsonResponse format used
/// for internal CLI output parsing.
///
/// # Arguments
///
/// * `result` - The result to convert
///
/// # Returns
///
/// A `JsonResponse` representing success or failure.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_node_tools::response::{result_to_response, JsonResponse};
/// use sublime_node_tools::error::ErrorInfo;
///
/// let ok_result: Result<String, ErrorInfo> = Ok("success".to_string());
/// let response = result_to_response(ok_result);
/// assert!(response.is_success());
///
/// let err_result: Result<String, ErrorInfo> = Err(ErrorInfo::validation("invalid", None::<String>));
/// let response = result_to_response(err_result);
/// assert!(response.is_error());
/// ```
// Used by status command (Story 3.2) for parsing CLI JSON output
#[allow(dead_code)]
pub(crate) fn result_to_response<T>(result: Result<T, ErrorInfo>) -> JsonResponse<T> {
    match result {
        Ok(data) => JsonResponse::success(data),
        Err(error) => JsonResponse::from_error_info(error),
    }
}
