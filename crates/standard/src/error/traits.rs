//! # Error handling traits
//!
//! ## What
//! This module provides traits for enhancing error handling capabilities,
//! particularly for adding contextual information to errors.
//!
//! ## How
//! The `ErrorContext` trait extends results with methods to add static or
//! dynamic context to errors, making error messages more informative.
//!
//! ## Why
//! Adding context to errors improves debugging and error reporting by providing
//! information about the operation that failed and the circumstances of the failure.

use core::result::Result as CoreResult;
use std::fmt::Display;

use super::{Error, Result};

/// Error context trait for adding contextual information to errors.
///
/// This trait provides methods to add context to any error that can be converted
/// to our standard Error type, making debugging easier by providing operation context.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{ErrorContext, FileSystemError, Result};
/// use std::path::PathBuf;
///
/// fn read_project_config() -> Result<String> {
///     let error = FileSystemError::NotFound { path: PathBuf::from("/missing/config.toml") };
///     Err(error)
///         .context("Failed to load project configuration")
/// }
/// ```
pub trait ErrorContext<T> {
    /// Adds static context to an error.
    fn context<C: Display>(self, context: C) -> Result<T>;

    /// Adds dynamic context to an error using a closure.
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T, E> ErrorContext<T> for CoreResult<T, E>
where
    E: Into<Error>,
{
    fn context<C: Display>(self, context: C) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            Error::operation(format!("{context}: {base_error}"))
        })
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            Error::operation(format!("{context}: {base_error}"))
        })
    }
}
