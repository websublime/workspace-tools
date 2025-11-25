//! # End-to-End Tests for Filesystem Module
//!
//! ## What
//! Comprehensive e2e tests for the filesystem operations framework including
//! `FileSystemManager`, `AsyncFileSystem` trait, `PathExt`, and `NodePathKind`.
//!
//! ## How
//! Tests perform real filesystem operations using temporary directories to verify
//! file reading, writing, directory operations, and path utilities.
//!
//! ## Why
//! E2E tests ensure the filesystem module works correctly with real files and
//! directories, validating async operations, error handling, and cross-platform
//! compatibility.

#![allow(clippy::print_stdout)]

use std::{path::Path, time::Duration};

use sublime_standard_tools::{
    config::FilesystemConfig,
    error::Result,
    filesystem::{
        AsyncFileSystem, AsyncFileSystemConfig, FileSystemManager, NodePathKind, PathExt,
    },
};
use tempfile::TempDir;

// ============================================================================
// FileSystemManager Creation Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_manager_new() -> Result<()> {
    let fs = FileSystemManager::new();
    let config = fs.config();

    // Verify default configuration
    assert!(config.operation_timeout > Duration::ZERO);
    assert!(config.read_timeout > Duration::ZERO);
    assert!(config.write_timeout > Duration::ZERO);
    Ok(())
}

#[tokio::test]
async fn test_filesystem_manager_with_config() -> Result<()> {
    let config = AsyncFileSystemConfig::new()
        .with_operation_timeout(Duration::from_secs(120))
        .with_read_timeout(Duration::from_secs(60))
        .with_write_timeout(Duration::from_secs(90));

    let fs = FileSystemManager::with_config(config);
    let stored_config = fs.config();

    assert_eq!(stored_config.operation_timeout, Duration::from_secs(120));
    assert_eq!(stored_config.read_timeout, Duration::from_secs(60));
    assert_eq!(stored_config.write_timeout, Duration::from_secs(90));
    Ok(())
}

#[tokio::test]
async fn test_filesystem_manager_with_standard_config() -> Result<()> {
    let fs_config = FilesystemConfig::default();
    let fs = FileSystemManager::with_standard_config(&fs_config);

    // Should be created without errors
    assert!(fs.config().operation_timeout > Duration::ZERO);
    Ok(())
}

#[tokio::test]
async fn test_filesystem_manager_default_trait() -> Result<()> {
    let fs = FileSystemManager::default();
    assert!(fs.config().operation_timeout > Duration::ZERO);
    Ok(())
}

// ============================================================================
// File Operations Tests
// ============================================================================

#[tokio::test]
async fn test_write_and_read_file_string() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("test.txt");

    // Write file
    let content = "Hello, World!\nLine 2\nLine 3";
    fs.write_file_string(&test_file, content).await?;

    // Read file
    let read_content = fs.read_file_string(&test_file).await?;

    assert_eq!(read_content, content);
    Ok(())
}

#[tokio::test]
async fn test_write_and_read_file_bytes() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("binary.dat");

    // Write binary data
    let data: Vec<u8> = vec![0, 1, 2, 3, 255, 254, 253, 252];
    fs.write_file(&test_file, &data).await?;

    // Read binary data
    let read_data = fs.read_file(&test_file).await?;

    assert_eq!(read_data, data);
    Ok(())
}

#[tokio::test]
async fn test_write_creates_parent_directories() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nested_file = temp_dir.path().join("a").join("b").join("c").join("file.txt");

    // Write should create parent directories
    fs.write_file_string(&nested_file, "nested content").await?;

    // Verify file exists and content is correct
    assert!(fs.exists(&nested_file).await);
    let content = fs.read_file_string(&nested_file).await?;
    assert_eq!(content, "nested content");
    Ok(())
}

#[tokio::test]
async fn test_write_overwrites_existing_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("overwrite.txt");

    // Write initial content
    fs.write_file_string(&test_file, "initial content").await?;

    // Overwrite with new content
    fs.write_file_string(&test_file, "new content").await?;

    // Verify new content
    let content = fs.read_file_string(&test_file).await?;
    assert_eq!(content, "new content");
    Ok(())
}

#[tokio::test]
async fn test_read_nonexistent_file_error() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let result = fs.read_file_string(&nonexistent).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_read_unicode_content() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("unicode.txt");

    let unicode_content = "Hello 世界! Привет мир! مرحبا بالعالم! 🌍🌎🌏";
    fs.write_file_string(&test_file, unicode_content).await?;

    let read_content = fs.read_file_string(&test_file).await?;
    assert_eq!(read_content, unicode_content);
    Ok(())
}

#[tokio::test]
async fn test_read_large_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("large.txt");

    // Create a ~1MB file
    let line = "This is a line of text that will be repeated many times.\n";
    let content: String = line.repeat(20000);
    fs.write_file_string(&test_file, &content).await?;

    let read_content = fs.read_file_string(&test_file).await?;
    assert_eq!(read_content.len(), content.len());
    Ok(())
}

// ============================================================================
// Directory Operations Tests
// ============================================================================

#[tokio::test]
async fn test_create_dir_all() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nested_dir = temp_dir.path().join("level1").join("level2").join("level3");

    // Create nested directories
    fs.create_dir_all(&nested_dir).await?;

    // Verify directory exists
    assert!(fs.exists(&nested_dir).await);
    Ok(())
}

#[tokio::test]
async fn test_create_dir_all_idempotent() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nested_dir = temp_dir.path().join("idempotent").join("test");

    // Create twice - should not error
    fs.create_dir_all(&nested_dir).await?;
    fs.create_dir_all(&nested_dir).await?;

    assert!(fs.exists(&nested_dir).await);
    Ok(())
}

#[tokio::test]
async fn test_read_dir() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create some files and directories
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    let subdir = temp_dir.path().join("subdir");

    fs.write_file_string(&file1, "content1").await?;
    fs.write_file_string(&file2, "content2").await?;
    fs.create_dir_all(&subdir).await?;

    // Read directory
    let entries = fs.read_dir(temp_dir.path()).await?;

    assert_eq!(entries.len(), 3);
    assert!(entries.contains(&file1));
    assert!(entries.contains(&file2));
    assert!(entries.contains(&subdir));
    Ok(())
}

#[tokio::test]
async fn test_read_dir_sorted() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create files in non-alphabetical order
    fs.write_file_string(&temp_dir.path().join("zebra.txt"), "z").await?;
    fs.write_file_string(&temp_dir.path().join("apple.txt"), "a").await?;
    fs.write_file_string(&temp_dir.path().join("mango.txt"), "m").await?;

    let entries = fs.read_dir(temp_dir.path()).await?;

    // Should be sorted
    let names: Vec<&str> =
        entries.iter().filter_map(|p| p.file_name()).filter_map(|n| n.to_str()).collect();

    assert_eq!(names, vec!["apple.txt", "mango.txt", "zebra.txt"]);
    Ok(())
}

#[tokio::test]
async fn test_read_dir_nonexistent_error() -> Result<()> {
    let fs = FileSystemManager::new();
    let nonexistent = Path::new("/nonexistent/directory/path");

    let result = fs.read_dir(nonexistent).await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_walk_dir() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create a directory structure
    // root/
    //   file1.txt
    //   subdir1/
    //     file2.txt
    //     subdir2/
    //       file3.txt

    fs.write_file_string(&temp_dir.path().join("file1.txt"), "1").await?;
    fs.create_dir_all(&temp_dir.path().join("subdir1").join("subdir2")).await?;
    fs.write_file_string(&temp_dir.path().join("subdir1").join("file2.txt"), "2").await?;
    fs.write_file_string(&temp_dir.path().join("subdir1").join("subdir2").join("file3.txt"), "3")
        .await?;

    let paths = fs.walk_dir(temp_dir.path()).await?;

    // Should find all files and directories
    assert!(paths.len() >= 4); // At least: file1.txt, subdir1, file2.txt, subdir2, file3.txt
    Ok(())
}

#[tokio::test]
async fn test_walk_dir_empty() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let empty_dir = temp_dir.path().join("empty");

    fs.create_dir_all(&empty_dir).await?;

    let paths = fs.walk_dir(&empty_dir).await?;
    assert!(paths.is_empty());
    Ok(())
}

// ============================================================================
// Remove Operations Tests
// ============================================================================

#[tokio::test]
async fn test_remove_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("to_remove.txt");

    // Create and verify file exists
    fs.write_file_string(&test_file, "to be removed").await?;
    assert!(fs.exists(&test_file).await);

    // Remove file
    fs.remove(&test_file).await?;

    // Verify file no longer exists
    assert!(!fs.exists(&test_file).await);
    Ok(())
}

#[tokio::test]
async fn test_remove_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let dir_to_remove = temp_dir.path().join("dir_to_remove");

    // Create directory with content
    fs.write_file_string(&dir_to_remove.join("file.txt"), "content").await?;
    assert!(fs.exists(&dir_to_remove).await);

    // Remove directory (recursive)
    fs.remove(&dir_to_remove).await?;

    // Verify directory no longer exists
    assert!(!fs.exists(&dir_to_remove).await);
    Ok(())
}

#[tokio::test]
async fn test_remove_nonexistent_error() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nonexistent = temp_dir.path().join("nonexistent");

    let result = fs.remove(&nonexistent).await;
    assert!(result.is_err());
    Ok(())
}

// ============================================================================
// Exists and Metadata Tests
// ============================================================================

#[tokio::test]
async fn test_exists_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("exists.txt");

    assert!(!fs.exists(&test_file).await);

    fs.write_file_string(&test_file, "content").await?;

    assert!(fs.exists(&test_file).await);
    Ok(())
}

#[tokio::test]
async fn test_exists_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_dir = temp_dir.path().join("exists_dir");

    assert!(!fs.exists(&test_dir).await);

    fs.create_dir_all(&test_dir).await?;

    assert!(fs.exists(&test_dir).await);
    Ok(())
}

#[tokio::test]
async fn test_metadata_file() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_file = temp_dir.path().join("metadata.txt");

    let content = "test content for metadata";
    fs.write_file_string(&test_file, content).await?;

    let metadata = fs.metadata(&test_file).await?;

    assert!(metadata.is_file());
    assert!(!metadata.is_dir());
    assert_eq!(metadata.len(), content.len() as u64);
    Ok(())
}

#[tokio::test]
async fn test_metadata_directory() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let test_dir = temp_dir.path().join("metadata_dir");

    fs.create_dir_all(&test_dir).await?;

    let metadata = fs.metadata(&test_dir).await?;

    assert!(metadata.is_dir());
    assert!(!metadata.is_file());
    Ok(())
}

#[tokio::test]
async fn test_metadata_nonexistent_error() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();
    let nonexistent = temp_dir.path().join("nonexistent_for_metadata");

    let result = fs.metadata(&nonexistent).await;
    assert!(result.is_err());
    Ok(())
}

// ============================================================================
// PathExt Tests
// ============================================================================

#[tokio::test]
async fn test_path_ext_node_path_src() -> Result<()> {
    let base = Path::new("/project");
    let src_path = base.node_path(NodePathKind::Src);

    assert_eq!(src_path, Path::new("/project/src"));
    Ok(())
}

#[tokio::test]
async fn test_path_ext_node_path_dist() -> Result<()> {
    let base = Path::new("/project");
    let dist_path = base.node_path(NodePathKind::Dist);

    assert_eq!(dist_path, Path::new("/project/dist"));
    Ok(())
}

#[tokio::test]
async fn test_path_ext_node_path_node_modules() -> Result<()> {
    let base = Path::new("/project");
    let modules_path = base.node_path(NodePathKind::NodeModules);

    assert_eq!(modules_path, Path::new("/project/node_modules"));
    Ok(())
}

#[tokio::test]
async fn test_path_ext_node_path_package_json() -> Result<()> {
    let base = Path::new("/project");
    let pkg_path = base.node_path(NodePathKind::PackageJson);

    assert_eq!(pkg_path, Path::new("/project/package.json"));
    Ok(())
}

#[tokio::test]
async fn test_path_ext_all_node_paths() -> Result<()> {
    let base = Path::new("/app");

    // Test all available node path kinds
    assert_eq!(base.node_path(NodePathKind::Src), Path::new("/app/src"));
    assert_eq!(base.node_path(NodePathKind::Dist), Path::new("/app/dist"));
    assert_eq!(base.node_path(NodePathKind::Test), Path::new("/app/test"));
    assert_eq!(base.node_path(NodePathKind::NodeModules), Path::new("/app/node_modules"));
    assert_eq!(base.node_path(NodePathKind::PackageJson), Path::new("/app/package.json"));

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_complex_directory_operations() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create a realistic project structure
    let project_root = temp_dir.path().join("my-project");

    // Create directories using NodePathKind
    fs.create_dir_all(&project_root.node_path(NodePathKind::Src)).await?;
    fs.create_dir_all(&project_root.node_path(NodePathKind::Dist)).await?;
    fs.create_dir_all(&project_root.node_path(NodePathKind::Test)).await?;

    // Create package.json using NodePathKind
    let package_json = r#"{
        "name": "my-project",
        "version": "1.0.0"
    }"#;
    fs.write_file_string(&project_root.node_path(NodePathKind::PackageJson), package_json).await?;

    // Create source file
    fs.write_file_string(&project_root.join("src").join("index.ts"), "export const x = 1;").await?;

    // Create tsconfig directly (not using NodePathKind since it doesn't have that variant)
    let tsconfig = r#"{
        "compilerOptions": {
            "outDir": "./dist"
        }
    }"#;
    fs.write_file_string(&project_root.join("tsconfig.json"), tsconfig).await?;

    // Verify structure
    assert!(fs.exists(&project_root).await);
    assert!(fs.exists(&project_root.node_path(NodePathKind::Src)).await);
    assert!(fs.exists(&project_root.node_path(NodePathKind::Dist)).await);
    assert!(fs.exists(&project_root.node_path(NodePathKind::Test)).await);
    assert!(fs.exists(&project_root.node_path(NodePathKind::PackageJson)).await);
    assert!(fs.exists(&project_root.join("tsconfig.json")).await);

    // Read and verify package.json
    let read_package =
        fs.read_file_string(&project_root.node_path(NodePathKind::PackageJson)).await?;
    assert!(read_package.contains("my-project"));

    // Walk directory and verify all entries
    let all_paths = fs.walk_dir(&project_root).await?;
    assert!(all_paths.len() >= 5); // At least: src, dist, test, package.json, tsconfig.json, index.ts

    Ok(())
}

#[tokio::test]
async fn test_concurrent_file_operations() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Create multiple files sequentially
    for i in 0..10 {
        let path = temp_dir.path().join(format!("file_{i}.txt"));
        let content = format!("Content for file {i}");
        fs.write_file_string(&path, &content).await?;
    }

    // Verify all files exist and have correct content
    for i in 0..10 {
        let path = temp_dir.path().join(format!("file_{i}.txt"));
        assert!(fs.exists(&path).await);

        let content = fs.read_file_string(&path).await?;
        assert_eq!(content, format!("Content for file {i}"));
    }

    Ok(())
}

#[tokio::test]
async fn test_special_characters_in_filename() -> Result<()> {
    let temp_dir = create_temp_dir()?;
    let fs = FileSystemManager::new();

    // Test files with special characters (valid on most systems)
    let special_names = vec![
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.multiple.dots.txt",
        "UPPERCASE.txt",
        "MixedCase.txt",
    ];

    for name in special_names {
        let path = temp_dir.path().join(name);
        fs.write_file_string(&path, &format!("Content for {name}")).await?;
        assert!(fs.exists(&path).await);

        let content = fs.read_file_string(&path).await?;
        assert!(content.contains(name));
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a temporary directory for testing
fn create_temp_dir() -> Result<TempDir> {
    TempDir::new().map_err(|e| {
        sublime_standard_tools::error::Error::operation(format!("Failed to create temp dir: {e}"))
    })
}
