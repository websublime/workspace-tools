//! # Error Module
//!
//! Defines the unified error type for all filesystem operations in workspace-fs.
//!
//! ## What
//!
//! This module provides a single [`Error`] enum that captures all possible error
//! conditions that can occur during filesystem operations. It uses the `snafu` crate
//! for ergonomic error handling with context.
//!
//! The error type covers:
//! - Path existence errors ([`NotFound`][Error::NotFound], [`AlreadyExists`][Error::AlreadyExists])
//! - Permission errors ([`PermissionDenied`][Error::PermissionDenied])
//! - Type mismatch errors ([`NotAFile`][Error::NotAFile], [`NotADirectory`][Error::NotADirectory])
//! - Content errors ([`NotEmpty`][Error::NotEmpty], [`InvalidUtf8`][Error::InvalidUtf8])
//! - I/O errors with context ([`Io`][Error::Io])
//! - Timeout errors ([`Timeout`][Error::Timeout])
//!
//! ## How
//!
//! The error type is built using `snafu`'s derive macro, which automatically generates:
//! - `std::error::Error` and `Display` implementations
//! - Context selectors for each variant (e.g., `NotFoundSnafu`, `IoSnafu`)
//! - Source error chaining for the `Io` variant
//!
//! Each variant includes the path that caused the error and, where applicable,
//! the operation being performed and the underlying system error. This provides
//! rich debugging information for troubleshooting filesystem issues.
//!
//! Context selectors are generated with `pub(crate)` visibility to keep them
//! internal to the crate while still allowing convenient error construction
//! within the crate's modules.
//!
//! ## Why
//!
//! A unified error type ensures:
//! - **Consistency**: All filesystem operations return the same error type
//! - **Context Preservation**: Every error includes the path and operation that failed
//! - **Type Safety**: Compile-time guarantees about error handling
//! - **Ergonomics**: Easy error conversion with the `?` operator via snafu's `.context()` method
//! - **Thread Safety**: Error is `Send + Sync` for use in async contexts
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{Error, Result};
//! use workspace_fs::error::NotFoundSnafu;
//! use snafu::ResultExt;
//! use std::path::Path;
//!
//! async fn read_config(fs: &impl FileSystem, path: &Path) -> Result<String> {
//!     // Errors automatically include the path that failed
//!     fs.read_to_string(path).await
//! }
//!
//! fn check_path_exists(path: &Path) -> Result<()> {
//!     if !path.exists() {
//!         return Err(Error::NotFound { path: path.to_path_buf() });
//!     }
//!     Ok(())
//! }
//! ```

use snafu::Snafu;
use std::path::PathBuf;
use std::time::Duration;

// =============================================================================
// Type Aliases
// =============================================================================

/// A specialized `Result` type for filesystem operations.
///
/// This type alias provides a convenient shorthand for functions that return
/// filesystem errors, reducing boilerplate in function signatures.
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::Result;
///
/// async fn read_file_contents(path: &Path) -> Result<String> {
///     // Implementation that may fail with filesystem errors
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

// =============================================================================
// Error Enum
// =============================================================================

/// Unified error type for all filesystem operations.
///
/// This enum captures all possible error conditions that can occur during
/// filesystem operations in the `workspace-fs` crate. Each variant includes
/// the path that caused the error for debugging purposes.
///
/// The error type implements:
/// - [`std::error::Error`] for standard error handling
/// - [`std::fmt::Display`] for user-friendly error messages
/// - [`Send`] and [`Sync`] for thread-safe async operations
///
/// # Variants
///
/// | Variant | Description | PRD Reference |
/// |---------|-------------|---------------|
/// | [`NotFound`][Self::NotFound] | Path does not exist | FR-6.2.1 |
/// | [`PermissionDenied`][Self::PermissionDenied] | Insufficient permissions | FR-6.2.2 |
/// | [`AlreadyExists`][Self::AlreadyExists] | Path already exists | FR-6.2.3 |
/// | [`NotAFile`][Self::NotAFile] | Expected file, found directory | FR-6.2.4 |
/// | [`NotADirectory`][Self::NotADirectory] | Expected directory, found file | FR-6.2.5 |
/// | [`NotEmpty`][Self::NotEmpty] | Directory is not empty | FR-6.2.6 |
/// | [`InvalidUtf8`][Self::InvalidUtf8] | Invalid UTF-8 content | FR-6.2.7 |
/// | [`Io`][Self::Io] | Wrapped I/O error | FR-6.2.8 |
/// | [`Timeout`][Self::Timeout] | Operation timed out | FR-6.2.9 |
///
/// # Example
///
/// ```rust,ignore
/// use workspace_fs::Error;
/// use std::path::PathBuf;
///
/// // Creating an error directly
/// let err = Error::NotFound {
///     path: PathBuf::from("/missing/file.txt"),
/// };
/// assert_eq!(format!("{}", err), "path not found: /missing/file.txt");
///
/// // Using context selectors with snafu
/// use snafu::ResultExt;
/// use workspace_fs::error::IoSnafu;
///
/// let result: Result<String, Error> = std::fs::read_to_string("config.json")
///     .context(IoSnafu {
///         path: PathBuf::from("config.json"),
///         operation: "read",
///     });
/// ```
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Path does not exist.
    ///
    /// This error occurs when attempting to access a file or directory
    /// that does not exist on the filesystem.
    ///
    /// # Fields
    ///
    /// * `path` - The path that was not found
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::NotFound {
    ///     path: PathBuf::from("/nonexistent/file.txt"),
    /// };
    /// println!("{}", err); // "path not found: /nonexistent/file.txt"
    /// ```
    #[snafu(display("path not found: {}", path.display()))]
    NotFound {
        /// The path that was not found.
        path: PathBuf,
    },

    /// Insufficient permissions to access path.
    ///
    /// This error occurs when the current process does not have the
    /// required permissions to perform the requested operation on the path.
    ///
    /// # Fields
    ///
    /// * `path` - The path that could not be accessed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::PermissionDenied {
    ///     path: PathBuf::from("/root/secret.txt"),
    /// };
    /// println!("{}", err); // "permission denied: /root/secret.txt"
    /// ```
    #[snafu(display("permission denied: {}", path.display()))]
    PermissionDenied {
        /// The path that access was denied for.
        path: PathBuf,
    },

    /// Path already exists when it shouldn't.
    ///
    /// This error occurs when attempting to create a file or directory
    /// that already exists, when the operation requires it not to exist.
    ///
    /// # Fields
    ///
    /// * `path` - The path that already exists
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::AlreadyExists {
    ///     path: PathBuf::from("/existing/file.txt"),
    /// };
    /// println!("{}", err); // "path already exists: /existing/file.txt"
    /// ```
    #[snafu(display("path already exists: {}", path.display()))]
    AlreadyExists {
        /// The path that already exists.
        path: PathBuf,
    },

    /// Expected a file but found a directory.
    ///
    /// This error occurs when an operation expects a file but the
    /// path points to a directory instead.
    ///
    /// # Fields
    ///
    /// * `path` - The path that is a directory instead of a file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::NotAFile {
    ///     path: PathBuf::from("/some/directory"),
    /// };
    /// println!("{}", err); // "expected file, found directory: /some/directory"
    /// ```
    #[snafu(display("expected file, found directory: {}", path.display()))]
    NotAFile {
        /// The path that is a directory instead of a file.
        path: PathBuf,
    },

    /// Expected a directory but found a file.
    ///
    /// This error occurs when an operation expects a directory but the
    /// path points to a file instead.
    ///
    /// # Fields
    ///
    /// * `path` - The path that is a file instead of a directory
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::NotADirectory {
    ///     path: PathBuf::from("/some/file.txt"),
    /// };
    /// println!("{}", err); // "expected directory, found file: /some/file.txt"
    /// ```
    #[snafu(display("expected directory, found file: {}", path.display()))]
    NotADirectory {
        /// The path that is a file instead of a directory.
        path: PathBuf,
    },

    /// Directory is not empty.
    ///
    /// This error occurs when attempting to remove a directory that
    /// is not empty, when the operation requires an empty directory.
    ///
    /// # Fields
    ///
    /// * `path` - The path to the non-empty directory
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::NotEmpty {
    ///     path: PathBuf::from("/some/directory"),
    /// };
    /// println!("{}", err); // "directory not empty: /some/directory"
    /// ```
    #[snafu(display("directory not empty: {}", path.display()))]
    NotEmpty {
        /// The path to the non-empty directory.
        path: PathBuf,
    },

    /// File content is not valid UTF-8.
    ///
    /// This error occurs when attempting to read a file as a string
    /// but the content contains invalid UTF-8 byte sequences.
    ///
    /// # Fields
    ///
    /// * `path` - The path to the file with invalid UTF-8 content
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    ///
    /// let err = Error::InvalidUtf8 {
    ///     path: PathBuf::from("/binary/file.bin"),
    /// };
    /// println!("{}", err); // "invalid UTF-8 content in file: /binary/file.bin"
    /// ```
    #[snafu(display("invalid UTF-8 content in file: {}", path.display()))]
    InvalidUtf8 {
        /// The path to the file with invalid UTF-8 content.
        path: PathBuf,
    },

    /// Wrapped I/O error with context.
    ///
    /// This error wraps an underlying [`std::io::Error`] with additional
    /// context about which path and operation caused the failure.
    ///
    /// # Fields
    ///
    /// * `path` - The path where the I/O error occurred
    /// * `operation` - A description of the operation being performed
    /// * `source` - The underlying I/O error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::Other, "disk full");
    /// let err = Error::Io {
    ///     path: PathBuf::from("/large/file.bin"),
    ///     operation: "write".to_string(),
    ///     source: io_error,
    /// };
    /// println!("{}", err); // "write failed for '/large/file.bin': disk full"
    /// ```
    #[snafu(display("{} failed for '{}': {}", operation, path.display(), source))]
    Io {
        /// The path where the I/O error occurred.
        path: PathBuf,
        /// The operation that was being performed when the error occurred.
        operation: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Operation timed out.
    ///
    /// This error occurs when a filesystem operation exceeds its
    /// configured timeout duration.
    ///
    /// # Fields
    ///
    /// * `path` - The path involved in the operation
    /// * `operation` - A description of the operation that timed out
    /// * `duration` - The timeout duration that was exceeded
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use workspace_fs::Error;
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    ///
    /// let err = Error::Timeout {
    ///     path: PathBuf::from("/slow/network/file.txt"),
    ///     operation: "read".to_string(),
    ///     duration: Duration::from_secs(30),
    /// };
    /// println!("{}", err); // "read timed out after 30s for '/slow/network/file.txt'"
    /// ```
    #[snafu(display("{} timed out after {:?} for '{}'", operation, duration, path.display()))]
    Timeout {
        /// The path involved in the timed-out operation.
        path: PathBuf,
        /// The operation that timed out.
        operation: String,
        /// The timeout duration that was exceeded.
        duration: Duration,
    },
}

// =============================================================================
// Trait Implementations
// =============================================================================

// Static assertions to ensure Error is Send + Sync (FR-6.1.4)
// These will fail to compile if Error is not Send + Sync
const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_sync<T: Sync>() {}
    assert_send::<Error>();
    assert_sync::<Error>();
};
