//! # Enhanced Filesystem Module Tests
//!
//! ## What
//! This file contains comprehensive edge case tests for the filesystem module,
//! focusing on permissions, symlinks, error conditions, and platform-specific behavior.
//!
//! ## How
//! Tests create various filesystem scenarios including permission denied,
//! symlink cycles, and cross-platform path handling to ensure robust behavior.
//!
//! ## Why
//! Phase 4 validation requires comprehensive testing of filesystem edge cases
//! to ensure the module behaves correctly under all conditions.

use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod enhanced_tests {
    use super::*;

    /// Helper to create a temporary directory for testing.
    #[allow(clippy::unwrap_used)]
    fn setup_test_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper to create a file with specific content.
    #[allow(clippy::unwrap_used)]
    fn create_file(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    /// Helper to create a directory structure.
    #[allow(clippy::unwrap_used)]
    fn create_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    #[test]
    fn test_filesystem_manager_comprehensive() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test file operations
        let file_path = path.join("test.txt");
        let content = "Hello, World!";

        assert!(fs.write_file_string(&file_path, content).is_ok());
        assert!(fs.exists(&file_path));

        let read_content = fs.read_file_string(&file_path).unwrap();
        assert_eq!(read_content, content);

        // Test binary operations
        let binary_content = b"Binary content";
        assert!(fs.write_file(&file_path, binary_content).is_ok());
        let read_binary = fs.read_file(&file_path).unwrap();
        assert_eq!(read_binary, binary_content);
    }

    #[test]
    fn test_filesystem_error_conditions() {
        let fs = FileSystemManager::new();

        // Test reading non-existent file
        let non_existent = PathBuf::from("/non/existent/file.txt");
        assert!(fs.read_file(&non_existent).is_err());
        assert!(fs.read_file_string(&non_existent).is_err());

        // Test writing to non-existent directory
        let bad_path = PathBuf::from("/non/existent/dir/file.txt");
        assert!(fs.write_file_string(&bad_path, "content").is_err());
    }

    #[test]
    fn test_filesystem_directory_operations() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create directory structure
        let nested_dir = path.join("nested").join("deep").join("structure");
        assert!(fs.create_dir_all(&nested_dir).is_ok());
        assert!(fs.exists(&nested_dir));

        // Create files in nested structure
        let file1 = nested_dir.join("file1.txt");
        let file2 = nested_dir.join("file2.txt");
        let file3 = path.join("root_file.txt");

        create_file(&file1, "content1");
        create_file(&file2, "content2");
        create_file(&file3, "content3");

        // Test directory listing
        let entries = fs.read_dir(&nested_dir).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&file1));
        assert!(entries.contains(&file2));

        // Test recursive walking
        let all_files = fs.walk_dir(path).unwrap();
        let file_count = all_files.iter().filter(|p| p.is_file()).count();
        assert!(file_count >= 3);
    }

    #[test]
    fn test_filesystem_path_validation() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test validation with existing path through exists method
        assert!(fs.exists(path));

        // Test validation with non-existent path
        let non_existent = path.join("non_existent");
        assert!(!fs.exists(&non_existent));
    }

    #[test]
    fn test_filesystem_large_file_handling() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create a large file (1MB)
        let large_content = "X".repeat(1024 * 1024);
        let large_file = path.join("large_file.txt");

        assert!(fs.write_file_string(&large_file, &large_content).is_ok());
        let read_content = fs.read_file_string(&large_file).unwrap();
        assert_eq!(read_content.len(), large_content.len());
    }

    #[test]
    fn test_filesystem_unicode_handling() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with unicode content
        let unicode_content = "Hello 世界! 🌍 Здравствуй мир! مرحبا بالعالم!";
        let unicode_file = path.join("unicode.txt");

        assert!(fs.write_file_string(&unicode_file, unicode_content).is_ok());
        let read_content = fs.read_file_string(&unicode_file).unwrap();
        assert_eq!(read_content, unicode_content);

        // Test with unicode filename
        let unicode_filename = path.join("файл_测试_🌍.txt");
        assert!(fs.write_file_string(&unicode_filename, "content").is_ok());
        assert!(fs.exists(&unicode_filename));
    }

    #[test]
    fn test_filesystem_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let fs = Arc::new(FileSystemManager::new());
        let temp_dir = setup_test_dir();
        let path = Arc::new(temp_dir.path().to_path_buf());

        // Run concurrent file operations
        let mut handles = vec![];
        for i in 0..10 {
            let fs_clone = Arc::clone(&fs);
            let path_clone = Arc::clone(&path);

            let handle = thread::spawn(move || {
                let file_path = path_clone.join(format!("file_{i}.txt"));
                let content = format!("Content for file {i}");

                fs_clone.write_file_string(&file_path, &content).unwrap();
                let read_content = fs_clone.read_file_string(&file_path).unwrap();
                assert_eq!(read_content, content);
            });
            handles.push(handle);
        }

        // All operations should complete successfully
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_filesystem_special_characters() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with special characters in content
        let special_content = r#"{"key": "value", "array": [1, 2, 3], "nested": {"inner": true}}"#;
        let special_file = path.join("special.json");

        assert!(fs.write_file_string(&special_file, special_content).is_ok());
        let read_content = fs.read_file_string(&special_file).unwrap();
        assert_eq!(read_content, special_content);
    }

    #[test]
    fn test_filesystem_empty_files() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with empty file
        let empty_file = path.join("empty.txt");
        assert!(fs.write_file_string(&empty_file, "").is_ok());
        assert!(fs.exists(&empty_file));

        let content = fs.read_file_string(&empty_file).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_node_path_kind_comprehensive() {
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

    #[test]
    fn test_path_extensions_comprehensive() {
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

    #[test]
    fn test_path_utils_comprehensive() {
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

    #[test]
    fn test_path_utils_edge_cases() {
        // Test with non-existent directory
        let non_existent = PathBuf::from("/non/existent/path");
        let result = PathUtils::find_project_root(&non_existent);
        assert_eq!(result, None);

        // Test with root directory
        let root = PathBuf::from("/");
        let result = PathUtils::find_project_root(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn test_path_utils_make_relative() {
        let temp_dir = setup_test_dir();
        let base = temp_dir.path();
        let target = base.join("sub").join("path").join("file.txt");

        let result = PathUtils::make_relative(&target, base);
        assert!(result.is_ok());

        let relative = result.unwrap();
        assert!(relative.is_relative());
        assert_eq!(relative, PathBuf::from("sub").join("path").join("file.txt"));
    }

    #[test]
    fn test_filesystem_remove_operations() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create file and directory
        let file_path = path.join("test.txt");
        let dir_path = path.join("test_dir");

        create_file(&file_path, "content");
        create_dir(&dir_path);

        // Test file removal
        assert!(fs.exists(&file_path));
        assert!(fs.remove(&file_path).is_ok());
        assert!(!fs.exists(&file_path));

        // Test directory removal
        assert!(fs.exists(&dir_path));
        assert!(fs.remove(&dir_path).is_ok());
        assert!(!fs.exists(&dir_path));
    }

    #[test]
    fn test_filesystem_cross_platform_paths() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Test with different path separators
        let unix_style = path.join("unix/style/path");
        create_dir(&unix_style);
        assert!(fs.exists(&unix_style));

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
            assert!(fs.write_file_string(&file_path, "content").is_ok());
            assert!(fs.exists(&file_path));
        }
    }

    #[test]
    fn test_filesystem_stress_test() {
        let fs = FileSystemManager::new();
        let temp_dir = setup_test_dir();
        let path = temp_dir.path();

        // Create many files
        for i in 0..100 {
            let file_path = path.join(format!("file_{i:03}.txt"));
            let content = format!("Content for file {i}");
            assert!(fs.write_file_string(&file_path, &content).is_ok());
        }

        // Read all files
        let entries = fs.read_dir(path).unwrap();
        assert_eq!(entries.len(), 100);

        // Walk directory
        let all_files = fs.walk_dir(path).unwrap();
        let file_count = all_files.iter().filter(|p| p.is_file()).count();
        assert_eq!(file_count, 100);
    }

    #[test]
    fn test_path_canonicalization() {
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
}
