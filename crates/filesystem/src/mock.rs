//! # Mock FileSystem Module
//!
//! In-memory implementation of the [`FileSystem`](crate::FileSystem) trait for testing.
//!
//! ## What
//!
//! This module provides [`MockFileSystem`], an in-memory filesystem implementation
//! designed for unit and integration testing. It simulates a complete filesystem
//! without touching the disk, enabling fast, deterministic, and isolated tests.
//!
//! ## How
//!
//! `MockFileSystem` maintains an in-memory data structure (typically a `HashMap` or
//! tree structure) that represents the filesystem state:
//! - Files are stored as byte vectors with associated metadata
//! - Directories are represented as containers of entries
//! - Symlinks store their target paths
//!
//! The implementation supports:
//! - Pre-populating the filesystem with test data
//! - Simulating errors for specific paths (e.g., permission denied)
//! - Inspecting filesystem state after operations
//!
//! Unlike `RealFileSystem`, the mock does not enforce timeouts, as all operations
//! are instantaneous in-memory operations.
//!
//! ## Why
//!
//! A mock filesystem is essential for:
//! - **Speed**: No disk I/O means tests run in microseconds, not milliseconds
//! - **Determinism**: Tests produce the same results regardless of host filesystem state
//! - **Isolation**: Tests cannot interfere with each other or the host system
//! - **Error Simulation**: Can test error handling without complex setup
//! - **CI/CD Compatibility**: Works in containerized environments with read-only filesystems
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileSystem, MockFileSystem};
//! use std::path::Path;
//!
//! #[tokio::test]
//! async fn test_read_config() {
//!     // Create mock filesystem with test data
//!     let mock = MockFileSystem::new();
//!     mock.create_file(
//!         Path::new("config.json"),
//!         r#"{"name": "test-project"}"#,
//!     ).await.unwrap();
//!
//!     // Test the function under test
//!     let content = mock.read_to_string(Path::new("config.json")).await.unwrap();
//!     assert!(content.contains("test-project"));
//! }
//!
//! #[tokio::test]
//! async fn test_file_not_found() {
//!     let mock = MockFileSystem::new();
//!
//!     // Test error handling
//!     let result = mock.read_to_string(Path::new("nonexistent.json")).await;
//!     assert!(result.is_err());
//! }
//! ```

// TODO: will be implemented on epic workspace-node-tools-0ea (MockFileSystem Implementation)
#![allow(clippy::todo)]
