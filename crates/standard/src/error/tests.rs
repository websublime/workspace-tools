#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::error::FileSystemError;
    use std::{io, path::PathBuf};

    #[test]
    fn test_filesystem_error_display() {
        let not_found = FileSystemError::NotFound { path: "/test".into() };
        assert_eq!(not_found.to_string(), "Path not found: /test");

        let io_error =
            FileSystemError::from_io(io::Error::new(io::ErrorKind::Other, "disk full"), "/data");
        assert_eq!(io_error.to_string(), "I/O error accessing path '/data': disk full");
    }

    #[test]
    fn test_validation_error() {
        let validation_error = FileSystemError::validation("/a/../b", "Parent traversal");
        assert_eq!(
            validation_error.to_string(),
            "Path validation failed for '/a/../b': Parent traversal"
        );
    }

    #[test]
    fn test_from_io_method() {
        // Test with different io::ErrorKind variants
        let not_found_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let fs_error = FileSystemError::from_io(not_found_error, "/missing/file.txt");
        assert!(
            matches!(fs_error, FileSystemError::NotFound { path } if path == PathBuf::from("/missing/file.txt"))
        );

        let permission_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let fs_error = FileSystemError::from_io(permission_error, "/protected/file.txt");
        assert!(
            matches!(fs_error, FileSystemError::PermissionDenied { path } if path == PathBuf::from("/protected/file.txt"))
        );

        let other_error = io::Error::new(io::ErrorKind::Other, "unknown error");
        let fs_error = FileSystemError::from_io(other_error, "/some/file.txt");
        assert!(
            matches!(fs_error, FileSystemError::Io { path, .. } if path == PathBuf::from("/some/file.txt"))
        );
    }

    #[test]
    fn test_from_io_error_trait_implementation() {
        // Test the From<io::Error> implementation
        let not_found_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let fs_error: FileSystemError = not_found_error.into();
        assert!(
            matches!(fs_error, FileSystemError::NotFound { path } if path == PathBuf::from("<unknown>"))
        );

        let permission_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let fs_error: FileSystemError = permission_error.into();
        assert!(
            matches!(fs_error, FileSystemError::PermissionDenied { path } if path == PathBuf::from("<unknown>"))
        );

        let other_error = io::Error::new(io::ErrorKind::Other, "unknown error");
        let fs_error: FileSystemError = other_error.into();
        assert!(
            matches!(fs_error, FileSystemError::Io { path, .. } if path == PathBuf::from("<unknown>"))
        );
    }

    #[test]
    fn test_as_ref_implementation() {
        // Test the AsRef<str> implementation for all variants
        let not_found = FileSystemError::NotFound { path: "/test".into() };
        assert_eq!(not_found.as_ref(), "FileSystemError::NotFound");

        let permission_denied = FileSystemError::PermissionDenied { path: "/test".into() };
        assert_eq!(permission_denied.as_ref(), "FileSystemError::PermissionDenied");

        let io_error = FileSystemError::Io {
            path: "/test".into(),
            source: io::Error::new(io::ErrorKind::Other, "test error"),
        };
        assert_eq!(io_error.as_ref(), "FileSystemError::Io");

        let not_a_directory = FileSystemError::NotADirectory { path: "/test".into() };
        assert_eq!(not_a_directory.as_ref(), "FileSystemError::NotADirectory");

        let not_a_file = FileSystemError::NotAFile { path: "/test".into() };
        assert_eq!(not_a_file.as_ref(), "FileSystemError::NotAFile");

        // Create a valid FromUtf8Error by trying to convert invalid UTF-8 bytes to a String
        let invalid_utf8 = vec![0xFF, 0xFF]; // Invalid UTF-8 bytes
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let utf8_decode = FileSystemError::Utf8Decode { path: "/test".into(), source: utf8_error };
        assert_eq!(utf8_decode.as_ref(), "FileSystemError::Utf8Decode");

        let validation =
            FileSystemError::Validation { path: "/test".into(), reason: "Invalid path".into() };
        assert_eq!(validation.as_ref(), "FileSystemError::Validation");
    }

    #[test]
    fn test_remaining_error_variants_display() {
        // Test display formatting for variants not covered by existing tests
        let not_a_directory = FileSystemError::NotADirectory { path: "/test/dir".into() };
        assert_eq!(not_a_directory.to_string(), "Expected a directory but found a file: /test/dir");

        let not_a_file = FileSystemError::NotAFile { path: "/test/file".into() };
        assert_eq!(not_a_file.to_string(), "Expected a file but found a directory: /test/file");

        // Create a UTF-8 decode error properly
        let invalid_utf8 = vec![0xFF, 0xFF]; // Invalid UTF-8
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let utf8_decode =
            FileSystemError::Utf8Decode { path: "/test/file.txt".into(), source: utf8_error };
        assert_eq!(
            utf8_decode.to_string(),
            "Failed to decode UTF-8 content in file: /test/file.txt"
        );

        let permission_denied =
            FileSystemError::PermissionDenied { path: "/test/protected".into() };
        assert_eq!(permission_denied.to_string(), "Permission denied for path: /test/protected");
    }
}
