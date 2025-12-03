//! API response wrapper module for Node.js bindings.
//!
//! # What
//!
//! This module provides response utilities for the NAPI layer. Instead of duplicating
//! the `JsonResponse<T>` type that already exists in `sublime_cli_tools::output`,
//! this module re-exports it and provides additional NAPI-specific extensions.
//!
//! # How
//!
//! The module provides:
//! - Re-export of `JsonResponse<T>` from the CLI crate
//! - Extension trait `ApiResponseExt` with NAPI-specific helpers
//! - Conversion utilities for creating responses from CLI errors
//!
//! The response structure follows the CLI pattern:
//! ```typescript
//! interface JsonResponse<T> {
//!   success: boolean;
//!   data?: T;      // Present when success is true
//!   error?: string;  // Present when success is false
//! }
//! ```
//!
//! # Why
//!
//! Using the existing `JsonResponse<T>` from CLI ensures consistency between
//! CLI JSON output and NAPI responses. This means JavaScript consumers get
//! the same response format whether using the CLI with `--format json` or
//! calling NAPI functions directly.
//!
//! # Examples
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
//!   // TypeScript knows result.error is string here
//!   console.error(`Error: ${result.error}`);
//! }
//! ```

use crate::error::ErrorInfo;

// Re-export JsonResponse from CLI for consistency
pub(crate) use sublime_cli_tools::output::JsonResponse;

/// Extension trait for `JsonResponse` with NAPI-specific helpers.
///
/// This trait provides additional methods useful in the NAPI context,
/// such as creating responses from `ErrorInfo` and other NAPI types.
// Allow dead code for placeholder trait that will be used in future stories
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
/// Rust's standard Result pattern to the JsonResponse format.
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
#[allow(dead_code)]
pub(crate) fn result_to_response<T>(result: Result<T, ErrorInfo>) -> JsonResponse<T> {
    match result {
        Ok(data) => JsonResponse::success(data),
        Err(error) => JsonResponse::from_error_info(error),
    }
}
