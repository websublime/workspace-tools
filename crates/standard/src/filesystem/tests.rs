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

    use crate::filesystem::{FileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils};

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

    #[test]
    fn test_path_ext_methods() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Create a mock project structure
        let project_root = temp_dir.path();
        let src_dir = project_root.join("src");
        let nested_dir = src_dir.join("nested");

        fs.create_dir_all(&nested_dir).unwrap();
        fs.write_file_string(&project_root.join("package.json"), "{\"name\": \"test-project\"}")
            .unwrap();

        // Test normalize
        let path = Path::new("/a/b/../c/./d");
        let normalized = path.normalize();
        assert_eq!(normalized, Path::new("/a/c/d"));

        // Test is_in_project
        assert!(project_root.is_in_project());
        assert!(src_dir.is_in_project());
        assert!(nested_dir.is_in_project());

        // Test relative_to_project
        let relative = nested_dir.relative_to_project().unwrap();
        assert_eq!(relative, Path::new("src/nested"));

        // Test node_path
        let node_modules = project_root.node_path(NodePathKind::NodeModules);
        assert_eq!(node_modules, project_root.join("node_modules"));

        let package_json = project_root.node_path(NodePathKind::PackageJson);
        assert_eq!(package_json, project_root.join("package.json"));
    }

    #[test]
    fn test_path_utils() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Create a mock project structure
        let project_root = temp_dir.path();
        fs.write_file_string(&project_root.join("package.json"), "{\"name\": \"test-project\"}")
            .unwrap();
        fs.write_file_string(&project_root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'")
            .unwrap();

        // Test find_project_root
        let nested_path = project_root.join("a/b/c");
        fs.create_dir_all(&nested_path).unwrap();

        let found_root = PathUtils::find_project_root(&nested_path);
        assert!(found_root.is_some());
        assert_eq!(found_root.unwrap(), project_root);

        // Test make_relative
        let result = PathUtils::make_relative(&nested_path, project_root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("a/b/c"));

        // Test current_dir
        let current = PathUtils::current_dir();
        assert!(current.is_ok());
    }

    #[test]
    fn test_node_path_kind() {
        assert_eq!(NodePathKind::NodeModules.default_path(), "node_modules");
        assert_eq!(NodePathKind::PackageJson.default_path(), "package.json");
        assert_eq!(NodePathKind::Src.default_path(), "src");
        assert_eq!(NodePathKind::Dist.default_path(), "dist");
        assert_eq!(NodePathKind::Test.default_path(), "test");
    }
}
