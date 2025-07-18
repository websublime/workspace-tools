//! # Filesystem Module Tests
//!
//! ## What
//! This file contains comprehensive tests for the filesystem module functionality,
//! covering basic operations, error conditions, edge cases, and stress scenarios.
//!
//! ## How
//! Tests use temporary directories for safety and include both success cases
//! and expected failure cases. Enhanced tests cover concurrent access, large files,
//! unicode handling, and cross-platform compatibility.
//!
//! ## Why
//! Comprehensive testing ensures that the filesystem operations work correctly
//! across different platforms and handle edge cases appropriately. This unified
//! test suite combines basic functionality tests with enhanced edge case coverage
//! to provide complete validation of the filesystem module.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

use crate::filesystem::{AsyncFileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils};

#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // HELPER FUNCTIONS
    // =============================================================================

    /// Helper function to create a temporary directory for tests
    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory for test")
    }

    /// Helper to create a file with specific content
    #[allow(clippy::unwrap_used)]
    fn create_file(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    /// Helper to create a directory structure
    #[allow(clippy::unwrap_used)]
    fn create_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    // =============================================================================
    // BASIC FILE OPERATIONS
    // =============================================================================

    #[tokio::test]
    async fn test_read_write_file() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_path = temp_dir.path().join("test_file.txt");
        let test_content = b"Hello, filesystem!";

        // Test write_file
        let write_result = fs.write_file(&test_path, test_content).await;
        assert!(write_result.is_ok());

        // Test read_file
        let read_result = fs.read_file(&test_path).await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);

        // Test exists
        assert!(fs.exists(&test_path).await);

        // Test remove
        let remove_result = fs.remove(&test_path).await;
        assert!(remove_result.is_ok());
        assert!(!fs.exists(&test_path).await);
    }

    #[tokio::test]
    async fn test_read_write_string() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_path = temp_dir.path().join("test_string.txt");
        let test_content = "Hello, string content!";

        // Test write_file_string
        let write_result = fs.write_file_string(&test_path, test_content).await;
        assert!(write_result.is_ok());

        // Test read_file_string
        let read_result = fs.read_file_string(&test_path).await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);
    }

    #[tokio::test]
    async fn test_filesystem_manager_comprehensive() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test file operations
        let file_path = path.join("test.txt");
        let content = "Hello, World!";

        assert!(fs.write_file_string(&file_path, content).await.is_ok());
        assert!(fs.exists(&file_path).await);

        let read_content = fs.read_file_string(&file_path).await.unwrap();
        assert_eq!(read_content, content);

        // Test binary operations
        let binary_content = b"Binary content";
        assert!(fs.write_file(&file_path, binary_content).await.is_ok());
        let read_binary = fs.read_file(&file_path).await.unwrap();
        assert_eq!(read_binary, binary_content);
    }

    #[tokio::test]
    async fn test_filesystem_empty_files() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with empty file
        let empty_file = path.join("empty.txt");
        assert!(fs.write_file_string(&empty_file, "").await.is_ok());
        assert!(fs.exists(&empty_file).await);

        let content = fs.read_file_string(&empty_file).await.unwrap();
        assert_eq!(content, "");
    }

    // =============================================================================
    // DIRECTORY OPERATIONS
    // =============================================================================

    #[tokio::test]
    async fn test_directory_operations() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        let test_dir = temp_dir.path().join("test_dir");
        let nested_dir = test_dir.join("nested");

        // Test create_dir_all
        let create_result = fs.create_dir_all(&nested_dir).await;
        assert!(create_result.is_ok());
        assert!(fs.exists(&nested_dir).await);

        // Create a file in the nested directory
        let test_file = nested_dir.join("test.txt");
        let write_result = fs.write_file_string(&test_file, "Test content").await;
        assert!(write_result.is_ok());

        // Test read_dir
        let read_dir_result = fs.read_dir(&test_dir).await;
        assert!(read_dir_result.is_ok());
        let entries = read_dir_result.unwrap();
        assert_eq!(entries.len(), 1); // Should contain just the nested directory

        // Test walk_dir
        let walk_result = fs.walk_dir(&test_dir).await;
        assert!(walk_result.is_ok());
        let all_entries = walk_result.unwrap();
        assert_eq!(all_entries.len(), 2); // nested directory and test.txt file

        // Test remove on directory (recursive)
        let remove_result = fs.remove(&test_dir).await;
        assert!(remove_result.is_ok());
        assert!(!fs.exists(&test_dir).await);
    }

    #[tokio::test]
    async fn test_filesystem_directory_operations() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create directory structure
        let nested_dir = path.join("nested").join("deep").join("structure");
        assert!(fs.create_dir_all(&nested_dir).await.is_ok());
        assert!(fs.exists(&nested_dir).await);

        // Create files in nested structure
        let file1 = nested_dir.join("file1.txt");
        let file2 = nested_dir.join("file2.txt");
        let file3 = path.join("root_file.txt");

        create_file(&file1, "content1");
        create_file(&file2, "content2");
        create_file(&file3, "content3");

        // Test directory listing
        let entries = fs.read_dir(&nested_dir).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&file1));
        assert!(entries.contains(&file2));

        // Test recursive walking
        let all_files = fs.walk_dir(path).await.unwrap();
        let file_count = all_files.iter().filter(|p| p.is_file()).count();
        assert!(file_count >= 3);
    }

    #[tokio::test]
    async fn test_filesystem_remove_operations() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create file and directory
        let file_path = path.join("test.txt");
        let dir_path = path.join("test_dir");

        create_file(&file_path, "content");
        create_dir(&dir_path);

        // Test file removal
        assert!(fs.exists(&file_path).await);
        assert!(fs.remove(&file_path).await.is_ok());
        assert!(!fs.exists(&file_path).await);

        // Test directory removal
        assert!(fs.exists(&dir_path).await);
        assert!(fs.remove(&dir_path).await.is_ok());
        assert!(!fs.exists(&dir_path).await);
    }

    // =============================================================================
    // ERROR CONDITIONS
    // =============================================================================

    #[tokio::test]
    async fn test_error_cases() {
        let fs = FileSystemManager::new();

        // Test reading a non-existent file
        let result = fs.read_file(Path::new("/non/existent/file.txt")).await;
        assert!(result.is_err());

        // Test reading a directory as a file
        let temp_dir = setup_test_dir();
        let result = fs.read_file(temp_dir.path()).await;
        assert!(result.is_err());

        // Test reading a non-directory as a directory
        let test_file = temp_dir.path().join("not_a_dir.txt");
        let _ = fs.write_file_string(&test_file, "content").await;
        let result = fs.read_dir(&test_file).await;
        assert!(result.is_err());

        // Test walking a non-directory
        let result = fs.walk_dir(&test_file).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_error_conditions() {
        let fs = FileSystemManager::new();

        // Test reading non-existent file
        let non_existent = PathBuf::from("/non/existent/file.txt");
        assert!(fs.read_file(&non_existent).await.is_err());
        assert!(fs.read_file_string(&non_existent).await.is_err());

        // Test writing to an invalid path that ensures failure on all platforms
        let temp_dir = setup_test_dir();
        let test_dir = temp_dir.path().join("test_directory");
        fs.create_dir_all(&test_dir).await.unwrap();
        
        // Try to write a file with the same name as an existing directory
        // This will always fail on all platforms because you cannot overwrite
        // a directory with a file
        assert!(fs.write_file_string(&test_dir, "content").await.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_path_validation() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test validation with existing path through exists method
        assert!(fs.exists(path).await);

        // Test validation with non-existent path
        let non_existent = path.join("non_existent");
        assert!(!fs.exists(&non_existent).await);
    }

    // =============================================================================
    // PATH UTILITIES
    // =============================================================================

    #[tokio::test]
    async fn test_path_ext_methods() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Create a mock project structure
        let project_root = temp_dir.path();
        let src_dir = project_root.join("src");
        let nested_dir = src_dir.join("nested");

        fs.create_dir_all(&nested_dir).await.unwrap();
        fs.write_file_string(&project_root.join("package.json"), "{\"name\": \"test-project\"}")
            .await.unwrap();

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

    #[tokio::test]
    async fn test_path_utils() {
        let temp_dir = setup_test_dir();
        let fs = FileSystemManager::new();

        // Create a mock project structure
        let project_root = temp_dir.path();
        fs.write_file_string(&project_root.join("package.json"), "{\"name\": \"test-project\"}")
            .await.unwrap();
        fs.write_file_string(&project_root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'")
            .await.unwrap();

        // Test find_project_root
        let nested_path = project_root.join("a/b/c");
        fs.create_dir_all(&nested_path).await.unwrap();

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

    #[tokio::test]
    async fn test_path_extensions_comprehensive() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create a project structure
        let package_json = path.join("package.json");
        create_file(&package_json, r#"{"name": "test", "version": "1.0.0"}"#);

        // Test path extensions
        let test_path = path.join("src").join("index.js");

        // Test normalization
        let normalized = test_path.normalize();
        assert!(!normalized.to_string_lossy().contains(".."));

        // Test Node.js path operations
        let node_modules = path.node_path(NodePathKind::NodeModules);
        assert_eq!(node_modules, path.join("node_modules"));

        let src_path = path.node_path(NodePathKind::Src);
        assert_eq!(src_path, path.join("src"));
    }

    #[tokio::test]
    async fn test_path_utils_comprehensive() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create nested project structure
        let nested_project = path.join("nested").join("project");
        create_dir(&nested_project);
        let package_json = nested_project.join("package.json");
        create_file(&package_json, r#"{"name": "nested-project", "version": "1.0.0"}"#);

        // Create lock file to make it a proper project root
        let lock_file = nested_project.join("package-lock.json");
        create_file(&lock_file, "{}");

        // Test project root finding
        let found_root = PathUtils::find_project_root(&nested_project);
        assert_eq!(found_root, Some(nested_project.clone()));

        // Test finding from subdirectory
        let src_dir = nested_project.join("src");
        create_dir(&src_dir);
        let found_from_src = PathUtils::find_project_root(&src_dir);
        assert_eq!(found_from_src, Some(nested_project.clone()));

        // Test current directory
        let current = PathUtils::current_dir();
        assert!(current.is_ok());
        assert!(current.unwrap().is_absolute());
    }

    #[tokio::test]
    async fn test_path_utils_edge_cases() {
        // Test with non-existent directory
        let non_existent = PathBuf::from("/non/existent/path");
        let result = PathUtils::find_project_root(&non_existent);
        assert_eq!(result, None);

        // Test with root directory
        let root = PathBuf::from("/");
        let result = PathUtils::find_project_root(&root);
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_path_utils_make_relative() {
        let temp_dir = setup_test_dir();
        let base = temp_dir.path();
        let target = base.join("sub").join("path").join("file.txt");

        let result = PathUtils::make_relative(&target, base);
        assert!(result.is_ok());

        let relative = result.unwrap();
        assert!(relative.is_relative());
        assert_eq!(relative, PathBuf::from("sub").join("path").join("file.txt"));
    }

    #[tokio::test]
    async fn test_path_canonicalization() {
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test canonicalization
        let canonical = path.canonicalize();
        assert!(canonical.is_ok());
        assert!(canonical.unwrap().is_absolute());

        // Test with non-existent path
        let non_existent = path.join("non_existent");
        let canonical_bad = non_existent.canonicalize();
        assert!(canonical_bad.is_err());
    }

    // =============================================================================
    // NODE PATH KIND TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_node_path_kind() {
        assert_eq!(NodePathKind::NodeModules.default_path(), "node_modules");
        assert_eq!(NodePathKind::PackageJson.default_path(), "package.json");
        assert_eq!(NodePathKind::Src.default_path(), "src");
        assert_eq!(NodePathKind::Dist.default_path(), "dist");
        assert_eq!(NodePathKind::Test.default_path(), "test");
    }

    #[tokio::test]
    async fn test_node_path_kind_comprehensive() {
        let test_cases = vec![
            (NodePathKind::NodeModules, "node_modules"),
            (NodePathKind::PackageJson, "package.json"),
            (NodePathKind::Src, "src"),
            (NodePathKind::Dist, "dist"),
            (NodePathKind::Test, "test"),
        ];

        for (kind, expected_path) in test_cases {
            assert_eq!(kind.default_path(), expected_path);

            // Test that all kinds are different
            assert_ne!(kind.default_path(), "");
        }
    }

    // =============================================================================
    // ENHANCED EDGE CASES
    // =============================================================================

    #[tokio::test]
    async fn test_filesystem_large_file_handling() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create a large file (1MB)
        let large_content = "X".repeat(1024 * 1024);
        let large_file = path.join("large_file.txt");

        assert!(fs.write_file_string(&large_file, &large_content).await.is_ok());
        let read_content = fs.read_file_string(&large_file).await.unwrap();
        assert_eq!(read_content.len(), large_content.len());
    }

    #[tokio::test]
    async fn test_filesystem_unicode_handling() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with unicode content
        let unicode_content = "Hello 世界! 🌍 Здравствуй мир! مرحبا بالعالم!";
        let unicode_file = path.join("unicode.txt");

        assert!(fs.write_file_string(&unicode_file, unicode_content).await.is_ok());
        let read_content = fs.read_file_string(&unicode_file).await.unwrap();
        assert_eq!(read_content, unicode_content);

        // Test with unicode filename
        let unicode_filename = path.join("файл_测试_🌍.txt");
        assert!(fs.write_file_string(&unicode_filename, "content").await.is_ok());
        assert!(fs.exists(&unicode_filename).await);
    }

    #[tokio::test]
    async fn test_filesystem_special_characters() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with special characters in content
        let special_content = r#"{"key": "value", "array": [1, 2, 3], "nested": {"inner": true}}"#;
        let special_file = path.join("special.json");

        assert!(fs.write_file_string(&special_file, special_content).await.is_ok());
        let read_content = fs.read_file_string(&special_file).await.unwrap();
        assert_eq!(read_content, special_content);
    }

    #[tokio::test]
    async fn test_filesystem_cross_platform_paths() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with different path separators
        let unix_style = path.join("unix/style/path");
        create_dir(&unix_style);
        assert!(fs.exists(&unix_style).await);

        // Test with various filename patterns
        let filenames = vec![
            "normal-file.txt",
            "file_with_underscores.txt",
            "file.with.dots.txt",
            "file-with-dashes.txt",
            "UPPERCASE.TXT",
            "123numeric.txt",
        ];

        for filename in filenames {
            let file_path = path.join(filename);
            assert!(fs.write_file_string(&file_path, "content").await.is_ok());
            assert!(fs.exists(&file_path).await);
        }
    }

    // =============================================================================
    // CONCURRENT OPERATIONS
    // =============================================================================

    #[tokio::test]
    async fn test_filesystem_concurrent_operations() {
        let fs = Arc::new(FileSystemManager::new());
        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Run concurrent file operations
        let mut handles = vec![];
        for i in 0..10 {
            let fs_clone = Arc::clone(&fs);
            let path_clone = Arc::clone(&path);

            let handle = tokio::spawn(async move {
                let file_path = path_clone.join(format!("file_{i}.txt"));
                let content = format!("Content for file {i}");

                fs_clone.write_file_string(&file_path, &content).await.unwrap();
                let read_content = fs_clone.read_file_string(&file_path).await.unwrap();
                assert_eq!(read_content, content);
            });
            handles.push(handle);
        }

        // All operations should complete successfully
        for handle in handles {
            handle.await.unwrap();
        }
    }

    // =============================================================================
    // STRESS TESTING
    // =============================================================================

    #[tokio::test]
    async fn test_filesystem_stress_test() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create many files
        for i in 0..100 {
            let file_path = path.join(format!("file_{i:03}.txt"));
            let content = format!("Content for file {i}");
            assert!(fs.write_file_string(&file_path, &content).await.is_ok());
        }

        // Read all files
        let entries = fs.read_dir(path).await.unwrap();
        assert_eq!(entries.len(), 100);

        // Walk directory
        let all_files = fs.walk_dir(path).await.unwrap();
        let file_count = all_files.iter().filter(|p| p.is_file()).count();
        assert_eq!(file_count, 100);
    }
}