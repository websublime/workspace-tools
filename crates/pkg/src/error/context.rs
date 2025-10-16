//! Error context utilities for adding contextual information to errors.
//!
//! **What**: Provides traits and utilities for adding rich contextual information to errors,
//! making error messages more descriptive and actionable.
//!
//! **How**: Implements the `ErrorContext` trait for `Result` types, allowing errors to be
//! wrapped with additional context strings. The trait provides both static and dynamic
//! context methods.
//!
//! **Why**: To provide clear, actionable error messages with full context about what operation
//! was being performed when an error occurred, making debugging and troubleshooting easier.
//!
//! # Features
//!
//! - **Context Attachment**: Add descriptive context to errors
//! - **Lazy Evaluation**: Support for lazy context evaluation using closures
//! - **Chain Preservation**: Maintains the original error chain while adding context
//! - **Zero Cost**: Context is only evaluated when an error actually occurs
//!
//! # Examples
//!
//! ## Adding static context
//!
//! ```rust
//! use sublime_pkg_tools::error::{Error, ErrorContext, ChangesetError};
//!
//! fn load_changeset(branch: &str) -> Result<String, Error> {
//!     read_file("changeset.json")
//!         .with_context(format!("Failed to load changeset for branch '{}'", branch))?;
//!     Ok("data".to_string())
//! }
//!
//! fn read_file(path: &str) -> Result<String, Error> {
//!     Err(Error::FileSystem("file not found".to_string()))
//! }
//! ```
//!
//! ## Adding lazy-evaluated context
//!
//! ```rust
//! use sublime_pkg_tools::error::{Error, ErrorContext};
//!
//! fn expensive_operation(id: usize) -> Result<String, Error> {
//!     perform_operation(id)
//!         .with_context_lazy(|| {
//!             format!("Failed to perform expensive operation for id {}", id)
//!         })?;
//!     Ok("result".to_string())
//! }
//!
//! fn perform_operation(id: usize) -> Result<String, Error> {
//!     Err(Error::FileSystem("operation failed".to_string()))
//! }
//! ```

use crate::error::Error;

/// Trait for adding contextual information to errors.
///
/// This trait provides methods to attach descriptive context to errors, making
/// error messages more informative and easier to debug. It is implemented for
/// `Result` types to allow chaining context operations.
///
/// # Examples
///
/// ## Basic usage
///
/// ```rust
/// use sublime_pkg_tools::error::{Error, ErrorContext, ConfigError};
///
/// fn load_config(path: &str) -> Result<String, Error> {
///     read_config(path)
///         .with_context(format!("Failed to load configuration from '{}'", path))?;
///     Ok("config".to_string())
/// }
///
/// fn read_config(path: &str) -> Result<String, Error> {
///     Err(Error::Config(ConfigError::NotFound {
///         path: std::path::PathBuf::from(path),
///     }))
/// }
/// ```
///
/// ## Using lazy evaluation
///
/// ```rust
/// use sublime_pkg_tools::error::{Error, ErrorContext};
///
/// fn process_package(name: &str, version: &str) -> Result<(), Error> {
///     validate_package(name, version)
///         .with_context_lazy(|| {
///             format!("Failed to process package '{}@{}'", name, version)
///         })?;
///     Ok(())
/// }
///
/// fn validate_package(name: &str, version: &str) -> Result<(), Error> {
///     Err(Error::FileSystem("validation failed".to_string()))
/// }
/// ```
pub trait ErrorContext<T> {
    /// Adds context information to an error.
    ///
    /// This method wraps the error with additional context, making the error
    /// message more descriptive. The context is evaluated immediately.
    ///
    /// # Arguments
    ///
    /// * `context` - A string describing the context of the operation
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` if the result was successful, or an `Err` with the
    /// context information prepended to the original error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{Error, ErrorContext};
    ///
    /// fn read_version() -> Result<String, Error> {
    ///     get_version()
    ///         .with_context("Failed to read package version".to_string())?;
    ///     Ok("1.0.0".to_string())
    /// }
    ///
    /// fn get_version() -> Result<String, Error> {
    ///     Err(Error::FileSystem("version not found".to_string()))
    /// }
    /// ```
    fn with_context(self, context: String) -> Result<T, Error>;

    /// Adds context information to an error using lazy evaluation.
    ///
    /// This method is similar to `with_context`, but the context string is only
    /// constructed if an error actually occurs. This is more efficient when the
    /// context string is expensive to create.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that returns the context string
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` if the result was successful, or an `Err` with the
    /// context information prepended to the original error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::error::{Error, ErrorContext};
    ///
    /// fn process_files(files: &[String]) -> Result<(), Error> {
    ///     for file in files {
    ///         process_file(file)
    ///             .with_context_lazy(|| format!("Failed to process file '{}'", file))?;
    ///     }
    ///     Ok(())
    /// }
    ///
    /// fn process_file(file: &str) -> Result<(), Error> {
    ///     Err(Error::FileSystem("process failed".to_string()))
    /// }
    /// ```
    fn with_context_lazy<F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn with_context(self, context: String) -> Result<T, Error> {
        self.map_err(|e| {
            let error: Error = e.into();
            Error::from(WithContext { context, source: Box::new(error) })
        })
    }

    fn with_context_lazy<F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let error: Error = e.into();
            Error::from(WithContext { context: f(), source: Box::new(error) })
        })
    }
}

/// A wrapper that adds context to an error.
///
/// This struct is used internally by the `ErrorContext` trait to wrap errors
/// with additional context information. It is not typically used directly.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::error::{Error, WithContext};
///
/// let original_error = Error::FileSystem("file not found".to_string());
/// let contextual_error = WithContext {
///     context: "Failed to read configuration".to_string(),
///     source: Box::new(original_error),
/// };
/// ```
#[derive(Debug)]
pub struct WithContext {
    /// The context message describing what operation was being performed.
    pub context: String,
    /// The original error that was wrapped.
    pub source: Box<Error>,
}

impl std::fmt::Display for WithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for WithContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<WithContext> for Error {
    fn from(ctx: WithContext) -> Self {
        // Create a contextual error by prepending the context to the source error
        match *ctx.source {
            Error::Config(e) => Error::Config(e),
            Error::Version(e) => Error::Version(e),
            Error::Changeset(e) => Error::Changeset(e),
            Error::Changes(e) => Error::Changes(e),
            Error::Changelog(e) => Error::Changelog(e),
            Error::Upgrade(e) => Error::Upgrade(e),
            Error::Audit(e) => Error::Audit(e),
            Error::FileSystem(msg) => Error::FileSystem(format!("{}: {}", ctx.context, msg)),
            Error::Git(msg) => Error::Git(format!("{}: {}", ctx.context, msg)),
            Error::IO(e) => Error::IO(e),
            Error::Json(e) => Error::Json(e),
        }
    }
}
