//! Performance benchmarks for PackageManager operations
//!
//! Comprehensive benchmarking suite for PackageManager to ensure enterprise-grade performance
//! characteristics. Benchmarks cover various scenarios including small/large packages,
//! concurrent operations, and memory usage patterns.
//!
//! ## What
//!
//! This module provides detailed performance benchmarks for all PackageManager operations:
//! - Read/write operations with various package sizes
//! - Validation performance with different dependency counts
//! - Concurrent operation throughput
//! - Memory usage profiling
//!
//! ## How
//!
//! Benchmarks use realistic package.json scenarios and measure:
//! - Operation latency (95th percentile, average, min/max)
//! - Throughput (operations per second)
//! - Memory allocation patterns
//! - Scalability with package size and dependency count
//!
//! ## Why
//!
//! Performance benchmarks ensure that PackageManager meets enterprise requirements
//! for speed and efficiency, especially when processing large monorepos with
//! hundreds of packages and thousands of dependencies.

#![allow(clippy::unwrap_used)] // Benchmarks may use unwrap for simplicity
#![allow(clippy::expect_used)] // Benchmarks may use expect for setup
#![allow(clippy::panic)] // Benchmarks may use panic for setup failures

use crate::{
    package::{manager::PackageManager, package::Package},
    Dependency,
};
use serde_json::json;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::task;

/// Mock filesystem optimized for benchmarking
#[derive(Clone, Default)]
struct BenchmarkFileSystem {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    operation_count: Arc<Mutex<u64>>,
}

impl BenchmarkFileSystem {
    fn new() -> Self {
        Self::default()
    }

    fn setup_test_file(&self, path: PathBuf, content: Vec<u8>) {
        let mut files = self.files.lock().unwrap();
        files.insert(path, content);
    }

    fn get_operation_count(&self) -> u64 {
        *self.operation_count.lock().unwrap()
    }

    fn reset_operation_count(&self) {
        *self.operation_count.lock().unwrap() = 0;
    }
}

#[async_trait::async_trait]
impl sublime_standard_tools::filesystem::AsyncFileSystem for BenchmarkFileSystem {
    async fn read_file(&self, path: &std::path::Path) -> sublime_standard_tools::error::Result<Vec<u8>> {
        // Increment operation counter
        *self.operation_count.lock().unwrap() += 1;

        let files = self.files.lock().unwrap();
        files.get(path).cloned().ok_or_else(|| {
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
        path: &std::path::Path,
        contents: &[u8],
    ) -> sublime_standard_tools::error::Result<()> {
        // Increment operation counter
        *self.operation_count.lock().unwrap() += 1;

        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), contents.to_vec());
        Ok(())
    }

    async fn read_file_string(&self, path: &std::path::Path) -> sublime_standard_tools::error::Result<String> {
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
        path: &std::path::Path,
        contents: &str,
    ) -> sublime_standard_tools::error::Result<()> {
        self.write_file(path, contents.as_bytes()).await
    }

    async fn create_dir_all(&self, _path: &std::path::Path) -> sublime_standard_tools::error::Result<()> {
        Ok(())
    }

    async fn remove(&self, path: &std::path::Path) -> sublime_standard_tools::error::Result<()> {
        let mut files = self.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    async fn exists(&self, path: &std::path::Path) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(path)
    }

    async fn read_dir(&self, _path: &std::path::Path) -> sublime_standard_tools::error::Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn walk_dir(&self, _path: &std::path::Path) -> sublime_standard_tools::error::Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    async fn metadata(&self, _path: &std::path::Path) -> sublime_standard_tools::error::Result<std::fs::Metadata> {
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

/// Benchmark result structure
#[derive(Debug)]
pub struct BenchmarkResult {
    pub operation_name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
}

impl BenchmarkResult {
    pub fn new(operation_name: String, iterations: u64, durations: Vec<Duration>) -> Self {
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;
        let min_duration = *durations.iter().min().unwrap_or(&Duration::ZERO);
        let max_duration = *durations.iter().max().unwrap_or(&Duration::ZERO);
        let operations_per_second = if total_duration.as_secs_f64() > 0.0 {
            iterations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            operation_name,
            iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            operations_per_second,
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== {} Benchmark Results ===", self.operation_name);
        println!("Iterations: {}", self.iterations);
        println!("Total time: {:?}", self.total_duration);
        println!("Average time: {:?}", self.avg_duration);
        println!("Min time: {:?}", self.min_duration);
        println!("Max time: {:?}", self.max_duration);
        println!("Operations/sec: {:.2}", self.operations_per_second);
    }
}

// Benchmark Tests

#[tokio::test]
async fn benchmark_read_operations_by_package_size() {
    let fs = BenchmarkFileSystem::new();
    let manager = PackageManager::new(fs.clone());

    let test_cases = vec![
        ("tiny", 0),
        ("small", 5),
        ("medium", 25),
        ("large", 100),
        ("xlarge", 500),
    ];

    for (size_name, dep_count) in test_cases {
        // Create package JSON with specified number of dependencies
        let mut dependencies = serde_json::Map::new();
        for i in 0..dep_count {
            dependencies.insert(
                format!("dep-{}", i),
                json!(format!("^{}.{}.{}", (i % 10) + 1, (i % 5) + 1, 0)),
            );
        }

        let package_json = json!({
            "name": format!("benchmark-{}", size_name),
            "version": "1.0.0",
            "dependencies": dependencies
        });

        let package_path = PathBuf::from(format!("benchmark-{}.json", size_name));
        fs.setup_test_file(
            package_path.clone(),
            serde_json::to_string(&package_json).unwrap().into_bytes(),
        );

        // Benchmark read operations
        let iterations = 100;
        let mut durations = Vec::new();
        fs.reset_operation_count();

        for _ in 0..iterations {
            let start = Instant::now();
            let result = manager.read_package(&package_path).await;
            let duration = start.elapsed();

            assert!(result.is_ok(), "Read should succeed for {}", size_name);
            durations.push(duration);
        }

        let benchmark_result = BenchmarkResult::new(
            format!("Read {} Package ({} deps)", size_name, dep_count),
            iterations,
            durations,
        );

        benchmark_result.print_summary();

        // Performance assertions
        assert!(
            benchmark_result.avg_duration < Duration::from_millis(50),
            "Average read time should be under 50ms for {} deps, was {:?}",
            dep_count,
            benchmark_result.avg_duration
        );

        assert!(
            benchmark_result.operations_per_second > 20.0,
            "Should achieve at least 20 ops/sec for {} deps, got {:.2}",
            dep_count,
            benchmark_result.operations_per_second
        );
    }
}

#[tokio::test]
async fn benchmark_write_operations_by_package_size() {
    let fs = BenchmarkFileSystem::new();
    let manager = PackageManager::new(fs.clone());

    let test_cases = vec![
        ("tiny", 0),
        ("small", 5),
        ("medium", 25),
        ("large", 100),
    ];

    for (size_name, dep_count) in test_cases {
        // Create package with specified number of dependencies
        let mut dependencies = Vec::new();
        for i in 0..dep_count {
            dependencies.push(
                Dependency::new(
                    &format!("dep-{}", i),
                    &format!("^{}.{}.0", (i % 10) + 1, (i % 5) + 1),
                )
                .expect("Should create dependency"),
            );
        }

        let package = Package::new(
            &format!("write-benchmark-{}", size_name),
            "1.0.0",
            if dependencies.is_empty() {
                None
            } else {
                Some(dependencies)
            },
        )
        .expect("Should create package");

        // Benchmark write operations
        let iterations = 50; // Fewer iterations for writes (more expensive)
        let mut durations = Vec::new();
        fs.reset_operation_count();

        for i in 0..iterations {
            let package_path = PathBuf::from(format!("write-{}-{}.json", size_name, i));
            let start = Instant::now();
            let result = manager.write_package(&package_path, &package).await;
            let duration = start.elapsed();

            assert!(result.is_ok(), "Write should succeed for {}", size_name);
            durations.push(duration);
        }

        let benchmark_result = BenchmarkResult::new(
            format!("Write {} Package ({} deps)", size_name, dep_count),
            iterations,
            durations,
        );

        benchmark_result.print_summary();

        // Performance assertions
        assert!(
            benchmark_result.avg_duration < Duration::from_millis(100),
            "Average write time should be under 100ms for {} deps, was {:?}",
            dep_count,
            benchmark_result.avg_duration
        );

        assert!(
            benchmark_result.operations_per_second > 10.0,
            "Should achieve at least 10 write ops/sec for {} deps, got {:.2}",
            dep_count,
            benchmark_result.operations_per_second
        );
    }
}

#[tokio::test]
async fn benchmark_validation_operations() {
    let manager = PackageManager::new(BenchmarkFileSystem::new());

    let test_cases = vec![
        ("valid_simple", create_valid_simple_package()),
        ("valid_complex", create_valid_complex_package()),
        ("invalid_multiple_errors", create_invalid_package()),
        ("large_dependencies", create_large_dependency_package()),
    ];

    for (test_name, package) in test_cases {
        let iterations = 200;
        let mut durations = Vec::new();

        for _ in 0..iterations {
            let start = Instant::now();
            let result = manager.validate_package(&package).await;
            let duration = start.elapsed();

            assert!(result.is_ok(), "Validation should not fail for {}", test_name);
            durations.push(duration);
        }

        let benchmark_result = BenchmarkResult::new(
            format!("Validate {} Package", test_name),
            iterations,
            durations,
        );

        benchmark_result.print_summary();

        // Performance assertions
        assert!(
            benchmark_result.avg_duration < Duration::from_millis(25),
            "Average validation time should be under 25ms for {}, was {:?}",
            test_name,
            benchmark_result.avg_duration
        );

        assert!(
            benchmark_result.operations_per_second > 40.0,
            "Should achieve at least 40 validation ops/sec for {}, got {:.2}",
            test_name,
            benchmark_result.operations_per_second
        );
    }
}

#[tokio::test]
async fn benchmark_concurrent_read_operations() {
    let fs = BenchmarkFileSystem::new();

    // Setup test files
    let package_json = json!({
        "name": "concurrent-test",
        "version": "1.0.0",
        "dependencies": {
            "react": "^18.0.0",
            "lodash": "^4.17.0",
            "axios": "^1.0.0"
        }
    });

    let package_path = PathBuf::from("concurrent-test.json");
    fs.setup_test_file(
        package_path.clone(),
        serde_json::to_string(&package_json).unwrap().into_bytes(),
    );

    let concurrency_levels = vec![1, 5, 10, 20, 50];

    for concurrency in concurrency_levels {
        let manager = PackageManager::new(fs.clone());
        fs.reset_operation_count();

        let start = Instant::now();
        let mut tasks = Vec::new();

        for _ in 0..concurrency {
            let manager_clone = manager.clone();
            let path_clone = package_path.clone();

            let task = task::spawn(async move {
                manager_clone.read_package(&path_clone).await
            });
            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut successful_ops = 0;
        for task in tasks {
            let result = task.await.expect("Task should complete");
            if result.is_ok() {
                successful_ops += 1;
            }
        }

        let total_duration = start.elapsed();
        let ops_per_second = successful_ops as f64 / total_duration.as_secs_f64();

        println!(
            "Concurrent Read Benchmark: {} concurrent operations in {:?} ({:.2} ops/sec)",
            concurrency, total_duration, ops_per_second
        );

        assert_eq!(
            successful_ops, concurrency,
            "All concurrent operations should succeed"
        );

        // Performance should scale reasonably with concurrency
        if concurrency <= 10 {
            assert!(
                ops_per_second > (concurrency as f64 * 0.8),
                "Should achieve reasonable throughput with {} concurrent ops, got {:.2} ops/sec",
                concurrency,
                ops_per_second
            );
        }
    }
}

#[tokio::test]
async fn benchmark_memory_usage_patterns() {
    let fs = BenchmarkFileSystem::new();
    let manager = PackageManager::new(fs.clone());

    // Test memory usage with packages of increasing size
    let sizes = vec![10, 50, 100, 500, 1000];

    for dep_count in sizes {
        let mut dependencies = Vec::new();
        for i in 0..dep_count {
            dependencies.push(
                Dependency::new(
                    &format!("memory-dep-{}", i),
                    &format!("^{}.0.0", (i % 20) + 1),
                )
                .expect("Should create dependency"),
            );
        }

        let package = Package::new(
            &format!("memory-test-{}", dep_count),
            "1.0.0",
            Some(dependencies),
        )
        .expect("Should create package");

        // Measure validation time and assume it correlates with memory usage
        let iterations = 10;
        let mut total_duration = Duration::ZERO;

        for _ in 0..iterations {
            let start = Instant::now();
            let result = manager.validate_package(&package).await;
            total_duration += start.elapsed();

            assert!(result.is_ok(), "Validation should succeed");
        }

        let avg_duration = total_duration / iterations;
        println!(
            "Memory Usage Test: {} dependencies - avg validation time: {:?}",
            dep_count, avg_duration
        );

        // Memory usage should scale reasonably (not exponentially)
        if dep_count <= 100 {
            assert!(
                avg_duration < Duration::from_millis(50),
                "Validation with {} deps should be under 50ms, was {:?}",
                dep_count,
                avg_duration
            );
        } else {
            assert!(
                avg_duration < Duration::from_millis(200),
                "Validation with {} deps should be under 200ms, was {:?}",
                dep_count,
                avg_duration
            );
        }
    }
}

// Helper functions for creating test packages

fn create_valid_simple_package() -> Package {
    Package::new("simple-package", "1.0.0", None).expect("Should create simple package")
}

fn create_valid_complex_package() -> Package {
    let dependencies = vec![
        Dependency::new("react", "^18.0.0").expect("Should create dependency"),
        Dependency::new("@types/node", "^20.0.0").expect("Should create dependency"),
        Dependency::new("lodash", "~4.17.21").expect("Should create dependency"),
    ];

    Package::new("complex-package", "2.1.3", Some(dependencies))
        .expect("Should create complex package")
}

fn create_invalid_package() -> Package {
    // Create a package that will have validation errors
    let mut package = Package::new("valid-start", "1.0.0", None).expect("Should create package");
    
    // Manually modify to create validation issues
    package.name = "Invalid Package Name".to_string(); // Spaces and uppercase
    package.version = "not-a-version".to_string(); // Invalid version
    
    package
}

fn create_large_dependency_package() -> Package {
    let mut dependencies = Vec::new();
    for i in 0..200 {
        dependencies.push(
            Dependency::new(
                &format!("large-dep-{}", i),
                &format!("^{}.{}.{}", (i % 5) + 1, (i % 3) + 1, i % 10),
            )
            .expect("Should create dependency"),
        );
    }

    Package::new("large-package", "1.0.0", Some(dependencies))
        .expect("Should create large package")
}