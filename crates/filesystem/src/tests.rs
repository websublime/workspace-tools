//! # Tests Module
//!
//! Unit tests for all workspace-fs modules, organized by module.
//!
//! ## What
//!
//! This module contains the unit test suite for the workspace-fs crate. Tests are
//! organized into submodules that mirror the main crate structure, ensuring each
//! module's functionality is thoroughly validated.
//!
//! ## How
//!
//! Tests are structured following these conventions:
//! - Each source module has a corresponding test submodule (e.g., `mod error`, `mod config`)
//! - Tests use `#[tokio::test]` for async test functions
//! - The `MockFileSystem` is used for testing filesystem operations without disk I/O
//! - Tests follow the Arrange-Act-Assert pattern for clarity
//!
//! Test submodules:
//! - `error`: Tests for error type construction and display
//! - `config`: Tests for configuration builder and defaults
//! - `types`: Tests for `FileType`, `DirEntry`, and `Metadata`
//! - `path_ext`: Tests for path normalization and utilities
//! - `traits`: Tests for `FileSystem` trait behavior via mock
//! - `real`: Integration tests for `RealFileSystem` (uses tempdir)
//! - `mock`: Tests for `MockFileSystem` behavior
//!
//! ## Why
//!
//! A dedicated tests module provides:
//! - **Organization**: All unit tests in one place, separate from production code
//! - **Discoverability**: Easy to find and run tests for specific functionality
//! - **Maintainability**: Test code doesn't clutter implementation files
//! - **Coverage Tracking**: Clear mapping between modules and their tests
//!
//! ## Example
//!
//! ```rust,ignore
//! // Run all workspace-fs tests
//! cargo test -p workspace-fs
//!
//! // Run tests for a specific module
//! cargo test -p workspace-fs error
//!
//! // Run a specific test
//! cargo test -p workspace-fs test_read_error_display
//! ```

#[cfg(test)]
mod error {
    //! Tests for the error module.
    //!
    //! This module contains unit tests that verify:
    //! - Display messages for all error variants
    //! - Error trait implementation
    //! - Send + Sync trait bounds
    //! - Error source chaining for Io variant

    use crate::error::Error;
    use std::error::Error as StdError;
    use std::io;
    use std::path::PathBuf;
    use std::time::Duration;

    // =========================================================================
    // Display Message Tests (FR-6.2.1 - FR-6.2.9)
    // =========================================================================

    #[test]
    fn test_not_found_display() {
        let path = PathBuf::from("/missing/file.txt");
        let err = Error::NotFound { path };
        let display = format!("{err}");
        assert_eq!(display, "path not found: /missing/file.txt");
    }

    #[test]
    fn test_permission_denied_display() {
        let path = PathBuf::from("/root/secret.txt");
        let err = Error::PermissionDenied { path };
        let display = format!("{err}");
        assert_eq!(display, "permission denied: /root/secret.txt");
    }

    #[test]
    fn test_already_exists_display() {
        let path = PathBuf::from("/existing/file.txt");
        let err = Error::AlreadyExists { path };
        let display = format!("{err}");
        assert_eq!(display, "path already exists: /existing/file.txt");
    }

    #[test]
    fn test_not_a_file_display() {
        let path = PathBuf::from("/some/directory");
        let err = Error::NotAFile { path };
        let display = format!("{err}");
        assert_eq!(display, "expected file, found directory: /some/directory");
    }

    #[test]
    fn test_not_a_directory_display() {
        let path = PathBuf::from("/some/file.txt");
        let err = Error::NotADirectory { path };
        let display = format!("{err}");
        assert_eq!(display, "expected directory, found file: /some/file.txt");
    }

    #[test]
    fn test_not_empty_display() {
        let path = PathBuf::from("/non/empty/directory");
        let err = Error::NotEmpty { path };
        let display = format!("{err}");
        assert_eq!(display, "directory not empty: /non/empty/directory");
    }

    #[test]
    fn test_invalid_utf8_display() {
        let path = PathBuf::from("/binary/file.bin");
        let err = Error::InvalidUtf8 { path };
        let display = format!("{err}");
        assert_eq!(display, "invalid UTF-8 content in file: /binary/file.bin");
    }

    #[test]
    fn test_io_error_display() {
        let path = PathBuf::from("/failed/operation.txt");
        let io_error = io::Error::other("disk full");
        let err = Error::Io { path, operation: "write".to_string(), source: io_error };
        let display = format!("{err}");
        assert_eq!(display, "write failed for '/failed/operation.txt': disk full");
    }

    #[test]
    fn test_timeout_display() {
        let path = PathBuf::from("/slow/file.txt");
        let err = Error::Timeout {
            path,
            operation: "read".to_string(),
            duration: Duration::from_secs(30),
        };
        let display = format!("{err}");
        assert_eq!(display, "read timed out after 30s for '/slow/file.txt'");
    }

    #[test]
    fn test_timeout_display_with_millis() {
        let path = PathBuf::from("/slow/file.txt");
        let err = Error::Timeout {
            path,
            operation: "metadata".to_string(),
            duration: Duration::from_millis(500),
        };
        let display = format!("{err}");
        assert_eq!(display, "metadata timed out after 500ms for '/slow/file.txt'");
    }

    // =========================================================================
    // std::error::Error Trait Tests (FR-6.1.3)
    // =========================================================================

    #[test]
    fn test_error_implements_std_error() {
        fn assert_std_error<T: std::error::Error>() {}
        assert_std_error::<Error>();
    }

    #[test]
    fn test_io_error_has_source() {
        let path = PathBuf::from("/test/file.txt");
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io { path, operation: "read".to_string(), source: io_error };

        // Verify the source is accessible through std::error::Error
        let source = err.source();
        assert!(source.is_some());
    }

    #[test]
    fn test_non_io_errors_have_no_source() {
        let not_found = Error::NotFound { path: PathBuf::from("/test") };
        assert!(not_found.source().is_none());

        let permission_denied = Error::PermissionDenied { path: PathBuf::from("/test") };
        assert!(permission_denied.source().is_none());

        let already_exists = Error::AlreadyExists { path: PathBuf::from("/test") };
        assert!(already_exists.source().is_none());

        let not_a_file = Error::NotAFile { path: PathBuf::from("/test") };
        assert!(not_a_file.source().is_none());

        let not_a_directory = Error::NotADirectory { path: PathBuf::from("/test") };
        assert!(not_a_directory.source().is_none());

        let not_empty = Error::NotEmpty { path: PathBuf::from("/test") };
        assert!(not_empty.source().is_none());

        let invalid_utf8 = Error::InvalidUtf8 { path: PathBuf::from("/test") };
        assert!(invalid_utf8.source().is_none());

        let timeout = Error::Timeout {
            path: PathBuf::from("/test"),
            operation: "read".to_string(),
            duration: Duration::from_secs(1),
        };
        assert!(timeout.source().is_none());
    }

    // =========================================================================
    // Send + Sync Tests (FR-6.1.4)
    // =========================================================================

    #[test]
    fn test_error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn test_error_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }

    // =========================================================================
    // AsRef<str> Trait Tests (Variant Name Introspection)
    // =========================================================================

    #[test]
    fn test_as_ref_not_found() {
        let err = Error::NotFound { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::NotFound");
    }

    #[test]
    fn test_as_ref_permission_denied() {
        let err = Error::PermissionDenied { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::PermissionDenied");
    }

    #[test]
    fn test_as_ref_already_exists() {
        let err = Error::AlreadyExists { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::AlreadyExists");
    }

    #[test]
    fn test_as_ref_not_a_file() {
        let err = Error::NotAFile { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::NotAFile");
    }

    #[test]
    fn test_as_ref_not_a_directory() {
        let err = Error::NotADirectory { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::NotADirectory");
    }

    #[test]
    fn test_as_ref_not_empty() {
        let err = Error::NotEmpty { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::NotEmpty");
    }

    #[test]
    fn test_as_ref_invalid_utf8() {
        let err = Error::InvalidUtf8 { path: PathBuf::from("/test") };
        assert_eq!(err.as_ref(), "Error::InvalidUtf8");
    }

    #[test]
    fn test_as_ref_io() {
        let io_error = io::Error::other("test");
        let err = Error::Io {
            path: PathBuf::from("/test"),
            operation: "read".to_string(),
            source: io_error,
        };
        assert_eq!(err.as_ref(), "Error::Io");
    }

    #[test]
    fn test_as_ref_timeout() {
        let err = Error::Timeout {
            path: PathBuf::from("/test"),
            operation: "read".to_string(),
            duration: Duration::from_secs(30),
        };
        assert_eq!(err.as_ref(), "Error::Timeout");
    }

    #[test]
    fn test_as_ref_returns_static_str() {
        // Verify that as_ref returns a &str that can be used for pattern matching
        let err = Error::NotFound { path: PathBuf::from("/test") };
        let variant_name: &str = err.as_ref();
        assert!(variant_name.starts_with("Error::"));
    }

    // =========================================================================
    // Debug Trait Tests
    // =========================================================================

    #[test]
    fn test_error_debug_format() {
        let err = Error::NotFound { path: PathBuf::from("/test/file.txt") };
        let debug = format!("{err:?}");
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("/test/file.txt"));
    }

    // =========================================================================
    // Result Type Alias Tests
    // =========================================================================

    #[test]
    fn test_result_type_alias_ok() {
        let result: crate::error::Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: crate::error::Result<i32> =
            Err(Error::NotFound { path: PathBuf::from("/test") });
        assert!(result.is_err());
    }

    // =========================================================================
    // Context Selector Tests (snafu integration)
    // =========================================================================

    #[test]
    fn test_context_selectors_are_available() {
        use crate::error::{
            AlreadyExistsSnafu, InvalidUtf8Snafu, IoSnafu, NotADirectorySnafu, NotAFileSnafu,
            NotEmptySnafu, NotFoundSnafu, PermissionDeniedSnafu, TimeoutSnafu,
        };
        use snafu::ResultExt;

        // Verify context selectors can be used
        let io_result: Result<(), io::Error> = Err(io::Error::other("test"));
        let _with_context: Result<(), Error> =
            io_result.context(IoSnafu { path: PathBuf::from("/test"), operation: "test" });

        // Verify simple selectors compile
        let _: crate::error::NotFoundSnafu<PathBuf> =
            NotFoundSnafu { path: PathBuf::from("/test") };
        let _: crate::error::PermissionDeniedSnafu<PathBuf> =
            PermissionDeniedSnafu { path: PathBuf::from("/test") };
        let _: crate::error::AlreadyExistsSnafu<PathBuf> =
            AlreadyExistsSnafu { path: PathBuf::from("/test") };
        let _: crate::error::NotAFileSnafu<PathBuf> =
            NotAFileSnafu { path: PathBuf::from("/test") };
        let _: crate::error::NotADirectorySnafu<PathBuf> =
            NotADirectorySnafu { path: PathBuf::from("/test") };
        let _: crate::error::NotEmptySnafu<PathBuf> =
            NotEmptySnafu { path: PathBuf::from("/test") };
        let _: crate::error::InvalidUtf8Snafu<PathBuf> =
            InvalidUtf8Snafu { path: PathBuf::from("/test") };
        let _: crate::error::TimeoutSnafu<PathBuf, &str, Duration> = TimeoutSnafu {
            path: PathBuf::from("/test"),
            operation: "read",
            duration: Duration::from_secs(1),
        };
    }

    #[test]
    fn test_io_context_with_result_ext() {
        use crate::error::IoSnafu;
        use snafu::ResultExt;

        fn fallible_io_operation() -> Result<String, io::Error> {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }

        let result: crate::error::Result<String> = fallible_io_operation()
            .context(IoSnafu { path: PathBuf::from("/test/file.txt"), operation: "read" });

        assert!(result.is_err());
        // Use match to extract error without unwrap_err
        let Err(err) = result else {
            // This branch is unreachable due to the assert above,
            // but we use return to satisfy the compiler without panic
            return;
        };
        let display = format!("{err}");
        assert!(display.contains("read failed"));
        assert!(display.contains("/test/file.txt"));
        assert!(display.contains("file not found"));
    }
}

#[cfg(test)]
mod config {
    //! Tests for the config module.
    // TODO: will be implemented on epic workspace-node-tools-g2t (Configuration Module)
}

#[cfg(test)]
mod types {
    //! Tests for the types module.
    // TODO: will be implemented on epic workspace-node-tools-3q8 (Types Module)
}

#[cfg(test)]
mod path_ext {
    //! Tests for the path_ext module.
    // TODO: will be implemented on epic workspace-node-tools-60y (PathExt Module)
}

#[cfg(test)]
mod traits {
    //! Tests for the traits module.
    // TODO: will be implemented on epic workspace-node-tools-hek (FileSystem Trait)
}

#[cfg(test)]
mod real {
    //! Tests for the real filesystem module.
    // TODO: will be implemented on epic workspace-node-tools-1gx (RealFileSystem Implementation)
}

#[cfg(test)]
mod mock {
    //! Tests for the mock filesystem module.
    // TODO: will be implemented on epic workspace-node-tools-0ea (MockFileSystem Implementation)
}
