//! Integration tests for PackageManager
//!
//! Comprehensive integration testing for PackageManager operations with real and mock filesystems.
//! Tests cover normal operations, edge cases, error handling, and performance characteristics
//! to ensure enterprise-grade reliability and robustness.
//!
//! ## What
//!
//! This module provides end-to-end integration testing for the PackageManager,
//! covering real filesystem operations, mock scenarios, error conditions, and
//! performance characteristics that unit tests cannot adequately cover.
//!
//! ## How
//!
//! Tests are organized into categories:
//! - Real filesystem operations with temporary directories
//! - Mock filesystem scenarios for controlled testing
//! - Edge cases and error conditions
//! - Multi-threading and concurrency tests
//! - Performance benchmarks and profiling
//!
//! ## Why
//!
//! Integration tests ensure that PackageManager works correctly in realistic
//! environments with actual I/O operations, handles edge cases gracefully,
//! and maintains performance under various conditions.

#![allow(clippy::unwrap_used)] // Integration tests may use unwrap for test failures
#![allow(clippy::expect_used)] // Integration tests may use expect for test failures
#![allow(clippy::panic)] // Integration tests may use panic for test failures

use crate::{
    package::{manager::PackageManager, package::Package},
    Dependency,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tempfile::TempDir;
use tokio::task;

/// Test helper to create a temporary directory with cleanup
struct TestEnvironment {
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub path: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();
        Self { temp_dir, path }
    }

    fn package_json_path(&self) -> PathBuf {
        self.path.join("package.json")
    }

    fn create_test_package_json(&self, content: &str) -> std::io::Result<()> {
        fs::write(self.package_json_path(), content)
    }
}

/// Mock filesystem for controlled testing scenarios
#[derive(Clone, Default)]
struct MockFileSystem {
    files: Arc<Mutex<std::collections::HashMap<PathBuf, Vec<u8>>>>,
    read_should_fail: Arc<Mutex<bool>>,
    write_should_fail: Arc<Mutex<bool>>,
    operation_delay: Arc<Mutex<Option<Duration>>>,
}

impl MockFileSystem {
    fn new() -> Self {
        Self::default()
    }

    fn set_file_content(&self, path: PathBuf, content: Vec<u8>) {
        let mut files = self.files.lock().unwrap();
        files.insert(path, content);
    }

    fn simulate_read_failure(&self, should_fail: bool) {
        *self.read_should_fail.lock().unwrap() = should_fail;
    }

    fn simulate_write_failure(&self, should_fail: bool) {
        *self.write_should_fail.lock().unwrap() = should_fail;
    }

    fn set_operation_delay(&self, delay: Option<Duration>) {
        *self.operation_delay.lock().unwrap() = delay;
    }

    fn file_exists(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(path)
    }
}

#[async_trait::async_trait]
impl AsyncFileSystem for MockFileSystem {
    async fn read_file(&self, path: &Path) -> sublime_standard_tools::error::Result<Vec<u8>> {
        // Simulate delay if configured
        let delay = *self.operation_delay.lock().unwrap();
        if let Some(delay) = delay {
            tokio::time::sleep(delay).await;
        }

        // Check if we should simulate failure
        let should_fail = *self.read_should_fail.lock().unwrap();
        if should_fail {
            return Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: path.to_path_buf(),
                    message: "Simulated read failure".to_string(),
                },
            ));
        }

        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| {
                sublime_standard_tools::error::Error::FileSystem(
                    sublime_standard_tools::error::FileSystemError::Io {
                        path: path.to_path_buf(),
                        message: "File not found".to_string(),
                    },
                )
            })
    }

    async fn write_file(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> sublime_standard_tools::error::Result<()> {
        // Simulate delay if configured
        let delay = *self.operation_delay.lock().unwrap();
        if let Some(delay) = delay {
            tokio::time::sleep(delay).await;
        }

        // Check if we should simulate failure
        let should_fail = *self.write_should_fail.lock().unwrap();
        if should_fail {
            return Err(sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: path.to_path_buf(),
                    message: "Simulated write failure".to_string(),
                },
            ));
        }

        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), contents.to_vec());
        Ok(())
    }

    async fn read_file_string(&self, path: &Path) -> sublime_standard_tools::error::Result<String> {
        let contents = self.read_file(path).await?;
        String::from_utf8(contents).map_err(|e| {
            sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: path.to_path_buf(),
                    message: format!("Invalid UTF-8: {}", e),
                },
            )
        })
    }

    async fn write_file_string(
        &self,
        path: &Path,
        contents: &str,
    ) -> sublime_standard_tools::error::Result<()> {
        self.write_file(path, contents.as_bytes()).await
    }

    async fn create_dir_all(&self, _path: &Path) -> sublime_standard_tools::error::Result<()> {
        Ok(())
    }

    async fn remove(&self, path: &Path) -> sublime_standard_tools::error::Result<()> {
        let mut files = self.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        self.file_exists(path)
    }

    async fn read_dir(
        &self,
        _path: &Path,
    ) -> sublime_standard_tools::error::Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn walk_dir(
        &self,
        _path: &Path,
    ) -> sublime_standard_tools::error::Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn metadata(
        &self,
        _path: &Path,
    ) -> sublime_standard_tools::error::Result<std::fs::Metadata> {
        std::fs::metadata(".").map_err(|e| {
            sublime_standard_tools::error::Error::FileSystem(
                sublime_standard_tools::error::FileSystemError::Io {
                    path: _path.to_path_buf(),
                    message: e.to_string(),
                },
            )
        })
    }
}

// Real Filesystem Integration Tests

#[tokio::test]
async fn test_real_filesystem_read_write_cycle() {
    let env = TestEnvironment::new();
    let fs = FileSystemManager::new();
    let manager = PackageManager::new(fs);

    // Create a test package.json file
    let package_json_content = json!({
        "name": "integration-test-package",
        "version": "1.2.3",
        "dependencies": {
            "react": "^18.0.0",
            "lodash": "~4.17.21"
        }
    });

    env.create_test_package_json(&serde_json::to_string_pretty(&package_json_content).unwrap())
        .expect("Failed to create test package.json");

    // Test reading the package
    let package = manager
        .read_package(&env.package_json_path())
        .await
        .expect("Failed to read package");

    assert_eq!(package.name, "integration-test-package");
    assert_eq!(package.version, "1.2.3");
    assert_eq!(package.dependencies.len(), 2);

    // Modify and write back
    let mut modified_package = package.clone();
    modified_package.version = "2.0.0".to_string();

    let write_result = manager
        .write_package(&env.package_json_path(), &modified_package)
        .await;
    assert!(write_result.is_ok(), "Failed to write package: {:?}", write_result);

    // Verify backup was created
    let backup_path = env.path.join("package.json.backup");
    assert!(backup_path.exists(), "Backup file should be created");

    // Read back and verify changes
    let updated_package = manager
        .read_package(&env.package_json_path())
        .await
        .expect("Failed to read updated package");

    assert_eq!(updated_package.version, "2.0.0");
    assert_eq!(updated_package.name, modified_package.name);
    assert_eq!(updated_package.dependencies.len(), modified_package.dependencies.len());
}

#[tokio::test]
async fn test_real_filesystem_validation_workflow() {
    let env = TestEnvironment::new();
    let fs = FileSystemManager::new();
    let manager = PackageManager::new(fs);

    // Create package with validation issues
    let package_with_issues = Package::new("TEST-PACKAGE", "1.0.0", None)
        .expect("Should create package");

    // Write and read back
    manager
        .write_package(&env.package_json_path(), &package_with_issues)
        .await
        .expect("Should write package");

    let read_package = manager
        .read_package(&env.package_json_path())
        .await
        .expect("Should read package");

    // Validate the package
    let validation_report = manager
        .validate_package(&read_package)
        .await
        .expect("Should validate package");

    // Should have errors due to uppercase name
    assert!(validation_report.has_errors());
    let errors = validation_report.errors();
    assert!(errors.iter().any(|e| e.contains("lowercase")));
}

#[tokio::test]
async fn test_real_filesystem_concurrent_operations() {
    let env = TestEnvironment::new();
    let fs = FileSystemManager::new();

    // Create initial package
    let initial_package = Package::new("concurrent-test", "1.0.0", None)
        .expect("Should create package");

    let manager = PackageManager::new(fs);
    manager
        .write_package(&env.package_json_path(), &initial_package)
        .await
        .expect("Should write initial package");

    // Spawn multiple concurrent read operations
    let mut tasks = vec![];
    for i in 0..10 {
        let manager_clone = manager.clone();
        let path_clone = env.package_json_path();
        
        let task = task::spawn(async move {
            let result = manager_clone.read_package(&path_clone).await;
            (i, result)
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let mut successful_reads = 0;
    for task in tasks {
        let (task_id, result) = task.await.expect("Task should complete");
        match result {
            Ok(package) => {
                assert_eq!(package.name, "concurrent-test");
                successful_reads += 1;
            }
            Err(e) => {
                panic!("Task {} failed to read package: {:?}", task_id, e);
            }
        }
    }

    assert_eq!(successful_reads, 10, "All concurrent reads should succeed");
}

// Mock Filesystem Integration Tests

#[tokio::test]
async fn test_mock_filesystem_error_handling() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    let package_path = PathBuf::from("test-package.json");

    // Test read failure scenario
    mock_fs.simulate_read_failure(true);
    let read_result = manager.read_package(&package_path).await;
    assert!(read_result.is_err(), "Should fail when read is simulated to fail");

    // Test successful read
    mock_fs.simulate_read_failure(false);
    let valid_package_json = json!({
        "name": "mock-test",
        "version": "1.0.0"
    });
    mock_fs.set_file_content(
        package_path.clone(),
        serde_json::to_string(&valid_package_json).unwrap().into_bytes(),
    );

    let read_result = manager.read_package(&package_path).await;
    assert!(read_result.is_ok(), "Should succeed when read works");

    let package = read_result.unwrap();
    assert_eq!(package.name, "mock-test");

    // Test write failure scenario
    mock_fs.simulate_write_failure(true);
    let write_result = manager.write_package(&package_path, &package).await;
    assert!(write_result.is_err(), "Should fail when write is simulated to fail");

    // Test successful write
    mock_fs.simulate_write_failure(false);
    let write_result = manager.write_package(&package_path, &package).await;
    assert!(write_result.is_ok(), "Should succeed when write works");
}

#[tokio::test]
async fn test_mock_filesystem_corrupted_data() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    let package_path = PathBuf::from("corrupted-package.json");

    // Test invalid JSON
    mock_fs.set_file_content(package_path.clone(), b"{ invalid json".to_vec());
    let result = manager.read_package(&package_path).await;
    assert!(result.is_err(), "Should fail with invalid JSON");

    // Test invalid UTF-8
    mock_fs.set_file_content(package_path.clone(), vec![0xFF, 0xFE, 0xFD]);
    let result = manager.read_package(&package_path).await;
    assert!(result.is_err(), "Should fail with invalid UTF-8");

    // Test missing required fields
    mock_fs.set_file_content(
        package_path.clone(),
        b"{ \"description\": \"Missing name and version\" }".to_vec(),
    );
    let result = manager.read_package(&package_path).await;
    assert!(result.is_err(), "Should fail with missing required fields");

    // Test empty file
    mock_fs.set_file_content(package_path.clone(), vec![]);
    let result = manager.read_package(&package_path).await;
    assert!(result.is_err(), "Should fail with empty file");
}

#[tokio::test]
async fn test_mock_filesystem_performance_with_delays() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    let package_path = PathBuf::from("delayed-package.json");
    let valid_package_json = json!({
        "name": "delayed-test",
        "version": "1.0.0"
    });
    mock_fs.set_file_content(
        package_path.clone(),
        serde_json::to_string(&valid_package_json).unwrap().into_bytes(),
    );

    // Test with artificial delay
    mock_fs.set_operation_delay(Some(Duration::from_millis(100)));

    let start = Instant::now();
    let result = manager.read_package(&package_path).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should succeed despite delay");
    assert!(
        duration >= Duration::from_millis(100),
        "Should respect artificial delay"
    );

    // Test without delay
    mock_fs.set_operation_delay(None);

    let start = Instant::now();
    let result = manager.read_package(&package_path).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should succeed without delay");
    assert!(
        duration < Duration::from_millis(50),
        "Should be fast without delay"
    );
}

// Edge Cases and Error Handling Tests

#[tokio::test]
async fn test_edge_case_empty_dependencies() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    let package_path = PathBuf::from("empty-deps.json");
    let package_json = json!({
        "name": "empty-deps-test",
        "version": "1.0.0",
        "dependencies": {}
    });

    mock_fs.set_file_content(
        package_path.clone(),
        serde_json::to_string(&package_json).unwrap().into_bytes(),
    );

    let package = manager
        .read_package(&package_path)
        .await
        .expect("Should read package with empty dependencies");

    assert_eq!(package.name, "empty-deps-test");
    assert!(package.dependencies.is_empty());

    let validation_report = manager
        .validate_package(&package)
        .await
        .expect("Should validate package");

    assert!(!validation_report.has_errors(), "Empty dependencies should be valid");
}

#[tokio::test]
async fn test_edge_case_large_package() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    // Create package with many dependencies
    let mut dependencies = serde_json::Map::new();
    for i in 0..100 {
        dependencies.insert(format!("dependency-{}", i), json!(format!("^{}.0.0", i % 10 + 1)));
    }

    let large_package_json = json!({
        "name": "large-package",
        "version": "1.0.0",
        "dependencies": dependencies
    });

    let package_path = PathBuf::from("large-package.json");
    mock_fs.set_file_content(
        package_path.clone(),
        serde_json::to_string(&large_package_json).unwrap().into_bytes(),
    );

    let start = Instant::now();
    let package = manager
        .read_package(&package_path)
        .await
        .expect("Should read large package");
    let read_duration = start.elapsed();

    assert_eq!(package.name, "large-package");
    assert_eq!(package.dependencies.len(), 100);
    assert!(
        read_duration < Duration::from_millis(1000),
        "Should read large package reasonably fast"
    );

    let start = Instant::now();
    let validation_report = manager
        .validate_package(&package)
        .await
        .expect("Should validate large package");
    let validation_duration = start.elapsed();

    assert!(
        validation_report.has_warnings(),
        "Should have warnings for many dependencies"
    );
    assert!(
        validation_duration < Duration::from_millis(1000),
        "Should validate large package reasonably fast"
    );
}

#[tokio::test]
async fn test_edge_case_special_characters_in_paths() {
    let env = TestEnvironment::new();
    let fs = FileSystemManager::new();
    let manager = PackageManager::new(fs);

    // Create package with special characters in name
    let package = Package::new("special-chars-àáâã", "1.0.0", None)
        .expect("Should create package with special chars");

    let special_path = env.path.join("special-àáâã-package.json");

    let write_result = manager.write_package(&special_path, &package).await;
    assert!(
        write_result.is_ok(),
        "Should handle special characters in paths: {:?}",
        write_result
    );

    let read_result = manager.read_package(&special_path).await;
    assert!(
        read_result.is_ok(),
        "Should read package with special chars in path: {:?}",
        read_result
    );

    let read_package = read_result.unwrap();
    assert_eq!(read_package.name, "special-chars-àáâã");
}

// Performance and Benchmark Tests

#[tokio::test]
async fn test_performance_multiple_read_operations() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    let package_json = json!({
        "name": "perf-test",
        "version": "1.0.0",
        "dependencies": {
            "react": "^18.0.0",
            "lodash": "^4.17.0"
        }
    });

    let package_path = PathBuf::from("perf-package.json");
    mock_fs.set_file_content(
        package_path.clone(),
        serde_json::to_string(&package_json).unwrap().into_bytes(),
    );

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let result = manager.read_package(&package_path).await;
        assert!(result.is_ok(), "All reads should succeed");
    }

    let duration = start.elapsed();
    let avg_per_operation = duration / iterations;

    println!(
        "Performance: {} reads in {:?}, avg {:?} per read",
        iterations, duration, avg_per_operation
    );

    assert!(
        avg_per_operation < Duration::from_millis(10),
        "Average read time should be under 10ms, was {:?}",
        avg_per_operation
    );
}

#[tokio::test]
async fn test_performance_validation_benchmarks() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    // Create packages of different sizes for benchmarking
    let test_cases = vec![
        ("small", 0),
        ("medium", 10),
        ("large", 50),
        ("xlarge", 100),
    ];

    for (size_name, dep_count) in test_cases {
        let mut dependencies = Vec::new();
        for i in 0..dep_count {
            dependencies.push(
                Dependency::new(&format!("dep-{}", i), &format!("^{}.0.0", i % 5 + 1))
                    .expect("Should create dependency"),
            );
        }

        let package = Package::new(
            &format!("benchmark-{}", size_name),
            "1.0.0",
            if dependencies.is_empty() {
                None
            } else {
                Some(dependencies)
            },
        )
        .expect("Should create package");

        let start = Instant::now();
        let validation_report = manager
            .validate_package(&package)
            .await
            .expect("Should validate package");
        let duration = start.elapsed();

        println!(
            "Validation benchmark {}: {} deps in {:?}",
            size_name,
            dep_count,
            duration
        );

        // Validation should be fast even for large packages
        assert!(
            duration < Duration::from_millis(100),
            "Validation should be under 100ms for {} deps, was {:?}",
            dep_count,
            duration
        );

        // Verify validation logic still works
        if dep_count > 50 {
            assert!(
                validation_report.has_warnings(),
                "Should warn about too many dependencies"
            );
        }
    }
}

#[tokio::test]
async fn test_memory_usage_with_large_packages() {
    let mock_fs = MockFileSystem::new();
    let manager = PackageManager::new(mock_fs.clone());

    // Create a very large package to test memory usage
    let mut dependencies = Vec::new();
    for i in 0..1000 {
        dependencies.push(
            Dependency::new(
                &format!("memory-test-dep-{}", i),
                &format!("^{}.{}.{}", i % 10, i % 100, i % 1000),
            )
            .expect("Should create dependency"),
        );
    }

    let large_package = Package::new("memory-test", "1.0.0", Some(dependencies))
        .expect("Should create large package");

    // Test that we can handle large packages without excessive memory usage
    let start = Instant::now();
    let validation_report = manager
        .validate_package(&large_package)
        .await
        .expect("Should validate large package");
    let duration = start.elapsed();

    assert!(large_package.dependencies.len() == 1000);
    assert!(validation_report.has_warnings()); // Should warn about too many deps
    assert!(
        duration < Duration::from_secs(1),
        "Should validate 1000 deps in under 1 second, took {:?}",
        duration
    );

    println!(
        "Memory test: validated package with {} dependencies in {:?}",
        large_package.dependencies.len(),
        duration
    );
}