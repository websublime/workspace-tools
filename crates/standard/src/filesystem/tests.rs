//! # Filesystem Tests
//!
//! ## What
//! This file contains tests for the filesystem module functionality.
//!
//! ## How
//! Tests use temporary directories for safety and include both success cases
//! and expected failure cases.
//!
//! ## Why
//! Comprehensive testing ensures that the filesystem operations work correctly
//! across different platforms and handle edge cases appropriately.

#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::path::Path;
    use tempfile::TempDir;

    use crate::filesystem::{FileSystem, FileSystemManager};

    // Helper function to create a temporary directory for tests
    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory for test")
    }

    #[test]
    fn test_read_write_file() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_path = temp_dir.path().join("test_file.txt");
        let test_content = b"Hello, filesystem!";

        // Test write_file
        let write_result = fs.write_file(&test_path, test_content);
        assert!(write_result.is_ok());

        // Test read_file
        let read_result = fs.read_file(&test_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);

        // Test exists
        assert!(fs.exists(&test_path));

        // Test remove
        let remove_result = fs.remove(&test_path);
        assert!(remove_result.is_ok());
        assert!(!fs.exists(&test_path));
    }

    #[test]
    fn test_read_write_string() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_path = temp_dir.path().join("test_string.txt");
        let test_content = "Hello, string content!";

        // Test write_file_string
        let write_result = fs.write_file_string(&test_path, test_content);
        assert!(write_result.is_ok());

        // Test read_file_string
        let read_result = fs.read_file_string(&test_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);
    }

    #[test]
    fn test_directory_operations() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_dir = temp_dir.path().join("test_dir");
        let nested_dir = test_dir.join("nested");

        // Test create_dir_all
        let create_result = fs.create_dir_all(&nested_dir);
        assert!(create_result.is_ok());
        assert!(fs.exists(&nested_dir));

        // Create a file in the nested directory
        let test_file = nested_dir.join("test.txt");
        let write_result = fs.write_file_string(&test_file, "Test content");
        assert!(write_result.is_ok());

        // Test read_dir
        let read_dir_result = fs.read_dir(&test_dir);
        assert!(read_dir_result.is_ok());
        let entries = read_dir_result.unwrap();
        assert_eq!(entries.len(), 1); // Should contain just the nested directory

        // Test walk_dir
        let walk_result = fs.walk_dir(&test_dir);
        assert!(walk_result.is_ok());
        let all_entries = walk_result.unwrap();
        assert_eq!(all_entries.len(), 3); // test_dir, nested, and test.txt

        // Test remove on directory (recursive)
        let remove_result = fs.remove(&test_dir);
        assert!(remove_result.is_ok());
        assert!(!fs.exists(&test_dir));
    }

    #[test]
    fn test_error_cases() {
        let fs = FileSystemManager::new();

        // Test reading a non-existent file
        let result = fs.read_file(Path::new("/non/existent/file.txt"));
        assert!(result.is_err());

        // Test reading a directory as a file
        let temp_dir = setup_test_dir();
        let result = fs.read_file(temp_dir.path());
        assert!(result.is_err());

        // Test reading a non-directory as a directory
        let test_file = temp_dir.path().join("not_a_dir.txt");
        let _ = fs.write_file_string(&test_file, "content");
        let result = fs.read_dir(&test_file);
        assert!(result.is_err());

        // Test walking a non-directory
        let result = fs.walk_dir(&test_file);
        assert!(result.is_err());
    }
}
