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
    //!
    //! This module contains unit tests for:
    //! - `FileType` enum and its methods
    //! - `Metadata` struct and its methods
    //! - `DirEntry` struct and its methods

    use crate::types::FileType;

    // =========================================================================
    // FileType Enum Tests (FR-5.2.1 - FR-5.2.5)
    // =========================================================================

    // -------------------------------------------------------------------------
    // Variant Creation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_file_type_file_variant() {
        let ft = FileType::File;
        assert!(ft.is_file());
        assert!(!ft.is_dir());
        assert!(!ft.is_symlink());
    }

    #[test]
    fn test_file_type_dir_variant() {
        let ft = FileType::Dir;
        assert!(!ft.is_file());
        assert!(ft.is_dir());
        assert!(!ft.is_symlink());
    }

    #[test]
    fn test_file_type_symlink_variant() {
        let ft = FileType::Symlink;
        assert!(!ft.is_file());
        assert!(!ft.is_dir());
        assert!(ft.is_symlink());
    }

    // -------------------------------------------------------------------------
    // Method Tests (FR-5.2.2 - FR-5.2.4)
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_file_returns_true_only_for_file() {
        assert!(FileType::File.is_file());
        assert!(!FileType::Dir.is_file());
        assert!(!FileType::Symlink.is_file());
    }

    #[test]
    fn test_is_dir_returns_true_only_for_dir() {
        assert!(!FileType::File.is_dir());
        assert!(FileType::Dir.is_dir());
        assert!(!FileType::Symlink.is_dir());
    }

    #[test]
    fn test_is_symlink_returns_true_only_for_symlink() {
        assert!(!FileType::File.is_symlink());
        assert!(!FileType::Dir.is_symlink());
        assert!(FileType::Symlink.is_symlink());
    }

    // -------------------------------------------------------------------------
    // Trait Implementation Tests (FR-5.2.5)
    // -------------------------------------------------------------------------

    #[test]
    fn test_file_type_debug() {
        let file = FileType::File;
        let debug_str = format!("{file:?}");
        assert_eq!(debug_str, "File");

        let dir = FileType::Dir;
        let debug_str = format!("{dir:?}");
        assert_eq!(debug_str, "Dir");

        let symlink = FileType::Symlink;
        let debug_str = format!("{symlink:?}");
        assert_eq!(debug_str, "Symlink");
    }

    #[test]
    fn test_file_type_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<FileType>();

        // For Copy types, clone is equivalent to copy
        let original = FileType::File;
        let cloned: FileType = Clone::clone(&original);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_file_type_copy() {
        let original = FileType::Dir;
        let copied = original; // Copy, not move
        assert_eq!(original, copied);
        // Both can still be used (proving Copy works)
        assert!(original.is_dir());
        assert!(copied.is_dir());
    }

    #[test]
    fn test_file_type_partial_eq() {
        assert_eq!(FileType::File, FileType::File);
        assert_eq!(FileType::Dir, FileType::Dir);
        assert_eq!(FileType::Symlink, FileType::Symlink);

        assert_ne!(FileType::File, FileType::Dir);
        assert_ne!(FileType::File, FileType::Symlink);
        assert_ne!(FileType::Dir, FileType::Symlink);
    }

    #[test]
    fn test_file_type_eq() {
        // Eq is a marker trait, we just verify it's implemented
        fn assert_eq_impl<T: Eq>() {}
        assert_eq_impl::<FileType>();
    }

    // -------------------------------------------------------------------------
    // Trait Bound Verification Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_file_type_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<FileType>();
    }

    #[test]
    fn test_file_type_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<FileType>();
    }

    // -------------------------------------------------------------------------
    // From<std::fs::FileType> Tests
    // -------------------------------------------------------------------------

    // Note: Testing From<std::fs::FileType> directly requires actual filesystem
    // access to obtain a std::fs::FileType instance. These tests are covered
    // in integration tests. Here we verify the trait is implemented.

    #[test]
    fn test_file_type_from_trait_is_implemented() {
        fn assert_from<T: From<std::fs::FileType>>() {}
        assert_from::<FileType>();
    }

    // =========================================================================
    // Metadata Struct Tests (FR-5.3.1 - FR-5.3.6)
    // =========================================================================

    use crate::types::Metadata;

    // -------------------------------------------------------------------------
    // Constructor Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_new_file() {
        let metadata = Metadata::new(FileType::File, 1024);
        assert!(metadata.is_file());
        assert_eq!(metadata.len(), 1024);
    }

    #[test]
    fn test_metadata_new_dir() {
        let metadata = Metadata::new(FileType::Dir, 0);
        assert!(metadata.is_dir());
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn test_metadata_new_symlink() {
        let metadata = Metadata::new(FileType::Symlink, 42);
        assert!(metadata.is_symlink());
        assert_eq!(metadata.len(), 42);
    }

    // -------------------------------------------------------------------------
    // len() Method Tests (FR-5.3.1)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_len_zero() {
        let metadata = Metadata::new(FileType::File, 0);
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn test_metadata_len_small_file() {
        let metadata = Metadata::new(FileType::File, 100);
        assert_eq!(metadata.len(), 100);
    }

    #[test]
    fn test_metadata_len_large_file() {
        let large_size: u64 = 10 * 1024 * 1024 * 1024; // 10 GB
        let metadata = Metadata::new(FileType::File, large_size);
        assert_eq!(metadata.len(), large_size);
    }

    #[test]
    fn test_metadata_len_max_u64() {
        let metadata = Metadata::new(FileType::File, u64::MAX);
        assert_eq!(metadata.len(), u64::MAX);
    }

    // -------------------------------------------------------------------------
    // is_empty() Method Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_is_empty_true() {
        let metadata = Metadata::new(FileType::File, 0);
        assert!(metadata.is_empty());
    }

    #[test]
    fn test_metadata_is_empty_false() {
        let metadata = Metadata::new(FileType::File, 1);
        assert!(!metadata.is_empty());
    }

    #[test]
    fn test_metadata_is_empty_large_file() {
        let metadata = Metadata::new(FileType::File, 1_000_000);
        assert!(!metadata.is_empty());
    }

    // -------------------------------------------------------------------------
    // file_type() Method Tests (FR-5.3.5)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_file_type_file() {
        let metadata = Metadata::new(FileType::File, 100);
        assert_eq!(metadata.file_type(), FileType::File);
    }

    #[test]
    fn test_metadata_file_type_dir() {
        let metadata = Metadata::new(FileType::Dir, 0);
        assert_eq!(metadata.file_type(), FileType::Dir);
    }

    #[test]
    fn test_metadata_file_type_symlink() {
        let metadata = Metadata::new(FileType::Symlink, 0);
        assert_eq!(metadata.file_type(), FileType::Symlink);
    }

    // -------------------------------------------------------------------------
    // is_file() Method Tests (FR-5.3.2)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_is_file_true() {
        let metadata = Metadata::new(FileType::File, 100);
        assert!(metadata.is_file());
    }

    #[test]
    fn test_metadata_is_file_false_for_dir() {
        let metadata = Metadata::new(FileType::Dir, 0);
        assert!(!metadata.is_file());
    }

    #[test]
    fn test_metadata_is_file_false_for_symlink() {
        let metadata = Metadata::new(FileType::Symlink, 0);
        assert!(!metadata.is_file());
    }

    // -------------------------------------------------------------------------
    // is_dir() Method Tests (FR-5.3.3)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_is_dir_true() {
        let metadata = Metadata::new(FileType::Dir, 0);
        assert!(metadata.is_dir());
    }

    #[test]
    fn test_metadata_is_dir_false_for_file() {
        let metadata = Metadata::new(FileType::File, 100);
        assert!(!metadata.is_dir());
    }

    #[test]
    fn test_metadata_is_dir_false_for_symlink() {
        let metadata = Metadata::new(FileType::Symlink, 0);
        assert!(!metadata.is_dir());
    }

    // -------------------------------------------------------------------------
    // is_symlink() Method Tests (FR-5.3.4)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_is_symlink_true() {
        let metadata = Metadata::new(FileType::Symlink, 0);
        assert!(metadata.is_symlink());
    }

    #[test]
    fn test_metadata_is_symlink_false_for_file() {
        let metadata = Metadata::new(FileType::File, 100);
        assert!(!metadata.is_symlink());
    }

    #[test]
    fn test_metadata_is_symlink_false_for_dir() {
        let metadata = Metadata::new(FileType::Dir, 0);
        assert!(!metadata.is_symlink());
    }

    // -------------------------------------------------------------------------
    // Trait Implementation Tests (FR-5.3.6)
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_debug() {
        let metadata = Metadata::new(FileType::File, 1024);
        let debug_str = format!("{metadata:?}");
        assert!(debug_str.contains("Metadata"));
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("1024"));
    }

    #[test]
    fn test_metadata_clone() {
        let original = Metadata::new(FileType::File, 2048);
        let cloned = original.clone();
        assert_eq!(original.file_type(), cloned.file_type());
        assert_eq!(original.len(), cloned.len());
    }

    #[test]
    fn test_metadata_clone_independence() {
        let original = Metadata::new(FileType::Dir, 0);
        let cloned = original.clone();
        // Both can be used independently
        assert!(original.is_dir());
        assert!(cloned.is_dir());
    }

    // -------------------------------------------------------------------------
    // Trait Bound Verification Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_metadata_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Metadata>();
    }

    #[test]
    fn test_metadata_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Metadata>();
    }

    // -------------------------------------------------------------------------
    // From<std::fs::Metadata> Tests
    // -------------------------------------------------------------------------

    // Note: Testing From<std::fs::Metadata> directly requires actual filesystem
    // access to obtain a std::fs::Metadata instance. These tests are covered
    // in integration tests. Here we verify the trait is implemented.

    #[test]
    fn test_metadata_from_trait_is_implemented() {
        fn assert_from<T: From<std::fs::Metadata>>() {}
        assert_from::<Metadata>();
    }

    // =========================================================================
    // DirEntry Struct Tests (FR-5.1.1 - FR-5.1.4)
    // =========================================================================

    use crate::types::DirEntry;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    // -------------------------------------------------------------------------
    // Constructor Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_new_file() {
        let path = PathBuf::from("/home/user/file.txt");
        let entry = DirEntry::new(path.clone(), FileType::File);
        assert_eq!(entry.path(), path.as_path());
        assert!(entry.file_type().is_file());
    }

    #[test]
    fn test_dir_entry_new_dir() {
        let path = PathBuf::from("/home/user/documents");
        let entry = DirEntry::new(path.clone(), FileType::Dir);
        assert_eq!(entry.path(), path.as_path());
        assert!(entry.file_type().is_dir());
    }

    #[test]
    fn test_dir_entry_new_symlink() {
        let path = PathBuf::from("/home/user/link");
        let entry = DirEntry::new(path.clone(), FileType::Symlink);
        assert_eq!(entry.path(), path.as_path());
        assert!(entry.file_type().is_symlink());
    }

    // -------------------------------------------------------------------------
    // path() Method Tests (FR-5.1.1)
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_path_absolute() {
        let path = PathBuf::from("/absolute/path/to/file.txt");
        let entry = DirEntry::new(path.clone(), FileType::File);
        assert_eq!(entry.path(), path.as_path());
    }

    #[test]
    fn test_dir_entry_path_relative() {
        let path = PathBuf::from("relative/path/file.txt");
        let entry = DirEntry::new(path.clone(), FileType::File);
        assert_eq!(entry.path(), path.as_path());
    }

    #[test]
    fn test_dir_entry_path_with_special_chars() {
        let path = PathBuf::from("/path/with spaces/and-dashes/file_name.txt");
        let entry = DirEntry::new(path.clone(), FileType::File);
        assert_eq!(entry.path(), path.as_path());
    }

    #[test]
    fn test_dir_entry_path_unicode() {
        let path = PathBuf::from("/путь/文件/αρχείο.txt");
        let entry = DirEntry::new(path.clone(), FileType::File);
        assert_eq!(entry.path(), path.as_path());
    }

    // -------------------------------------------------------------------------
    // file_name() Method Tests (FR-5.1.2)
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_file_name_simple() {
        let entry = DirEntry::new(PathBuf::from("/home/user/document.pdf"), FileType::File);
        assert_eq!(entry.file_name(), OsStr::new("document.pdf"));
    }

    #[test]
    fn test_dir_entry_file_name_directory() {
        let entry = DirEntry::new(PathBuf::from("/var/log"), FileType::Dir);
        assert_eq!(entry.file_name(), OsStr::new("log"));
    }

    #[test]
    fn test_dir_entry_file_name_with_extension() {
        let entry = DirEntry::new(PathBuf::from("/path/to/archive.tar.gz"), FileType::File);
        assert_eq!(entry.file_name(), OsStr::new("archive.tar.gz"));
    }

    #[test]
    fn test_dir_entry_file_name_hidden_file() {
        let entry = DirEntry::new(PathBuf::from("/home/user/.bashrc"), FileType::File);
        assert_eq!(entry.file_name(), OsStr::new(".bashrc"));
    }

    #[test]
    fn test_dir_entry_file_name_root_path() {
        // Root path has no file name, should return empty OsStr
        let entry = DirEntry::new(PathBuf::from("/"), FileType::Dir);
        assert_eq!(entry.file_name(), OsStr::new(""));
    }

    #[test]
    fn test_dir_entry_file_name_dot_dot() {
        // Parent directory reference has no file name
        let entry = DirEntry::new(PathBuf::from(".."), FileType::Dir);
        assert_eq!(entry.file_name(), OsStr::new(""));
    }

    #[test]
    fn test_dir_entry_file_name_relative() {
        let entry = DirEntry::new(PathBuf::from("relative/path/file.txt"), FileType::File);
        assert_eq!(entry.file_name(), OsStr::new("file.txt"));
    }

    // -------------------------------------------------------------------------
    // file_type() Method Tests (FR-5.1.3)
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_file_type_file() {
        let entry = DirEntry::new(PathBuf::from("/tmp/file.txt"), FileType::File);
        assert_eq!(entry.file_type(), FileType::File);
        assert!(entry.file_type().is_file());
        assert!(!entry.file_type().is_dir());
        assert!(!entry.file_type().is_symlink());
    }

    #[test]
    fn test_dir_entry_file_type_dir() {
        let entry = DirEntry::new(PathBuf::from("/tmp/subdir"), FileType::Dir);
        assert_eq!(entry.file_type(), FileType::Dir);
        assert!(!entry.file_type().is_file());
        assert!(entry.file_type().is_dir());
        assert!(!entry.file_type().is_symlink());
    }

    #[test]
    fn test_dir_entry_file_type_symlink() {
        let entry = DirEntry::new(PathBuf::from("/tmp/link"), FileType::Symlink);
        assert_eq!(entry.file_type(), FileType::Symlink);
        assert!(!entry.file_type().is_file());
        assert!(!entry.file_type().is_dir());
        assert!(entry.file_type().is_symlink());
    }

    // -------------------------------------------------------------------------
    // Trait Implementation Tests (FR-5.1.4)
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_debug() {
        let entry = DirEntry::new(PathBuf::from("/test/path.txt"), FileType::File);
        let debug_str = format!("{entry:?}");
        assert!(debug_str.contains("DirEntry"));
        assert!(debug_str.contains("path"));
        assert!(debug_str.contains("file_type"));
    }

    #[test]
    fn test_dir_entry_clone() {
        let original = DirEntry::new(PathBuf::from("/original/path.txt"), FileType::File);
        let cloned = original.clone();
        assert_eq!(original.path(), cloned.path());
        assert_eq!(original.file_type(), cloned.file_type());
    }

    #[test]
    fn test_dir_entry_clone_independence() {
        let original = DirEntry::new(PathBuf::from("/some/path"), FileType::Dir);
        let cloned = original.clone();
        // Both can be used independently
        assert!(original.file_type().is_dir());
        assert!(cloned.file_type().is_dir());
        assert_eq!(original.file_name(), cloned.file_name());
    }

    // -------------------------------------------------------------------------
    // Trait Bound Verification Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<DirEntry>();
    }

    #[test]
    fn test_dir_entry_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<DirEntry>();
    }

    // -------------------------------------------------------------------------
    // Edge Case Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dir_entry_empty_path() {
        let entry = DirEntry::new(PathBuf::new(), FileType::File);
        assert_eq!(entry.path(), PathBuf::new().as_path());
        assert_eq!(entry.file_name(), OsStr::new(""));
    }

    #[test]
    fn test_dir_entry_single_component() {
        let entry = DirEntry::new(PathBuf::from("filename.txt"), FileType::File);
        assert_eq!(entry.file_name(), OsStr::new("filename.txt"));
    }

    #[test]
    fn test_dir_entry_trailing_slash() {
        // PathBuf normalizes trailing slashes
        let entry = DirEntry::new(PathBuf::from("/path/to/dir/"), FileType::Dir);
        // Note: PathBuf may preserve or remove trailing slash depending on platform
        assert!(entry.file_type().is_dir());
    }
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
