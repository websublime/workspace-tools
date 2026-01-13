//! # Real FileSystem Module
//!
//! Production implementation of the [`FileSystem`](crate::FileSystem) trait using `tokio::fs`.
//!
//! ## What
//!
//! This module provides [`RealFileSystem`], the production-ready filesystem implementation
//! that performs actual I/O operations on the host's filesystem. It wraps `tokio::fs` to
//! provide async filesystem access with configurable timeout support.
//!
//! ## How
//!
//! `RealFileSystem` implements the `FileSystem` trait by:
//! 1. Delegating each operation to the corresponding `tokio::fs` function
//! 2. Wrapping operations in `tokio::time::timeout` using configured durations
//! 3. Converting `std::io::Error` into the crate's `Error` type with path context
//! 4. Logging operations at appropriate levels (trace for entry, debug for results)
//!
//! The implementation is stateless (configuration is stored but not mutated), making it
//! safe to share across tasks via `Arc<RealFileSystem>`.
//!
//! ## Why
//!
//! A dedicated real filesystem implementation provides:
//! - **Async I/O**: Non-blocking operations for better concurrency in large monorepos
//! - **Timeouts**: Prevent hanging on slow or unresponsive filesystems (e.g., network mounts)
//! - **Error Context**: Every error includes the path and operation that failed
//! - **Logging**: Integrated logging for debugging and observability
//! - **Testability**: Can be swapped for `MockFileSystem` in tests
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileSystem, RealFileSystem, FileSystemConfig};
//! use std::path::Path;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), workspace_fs::Error> {
//!     // Default configuration with 30s read/write, 60s operation timeouts
//!     let fs = RealFileSystem::new();
//!
//!     // Custom configuration
//!     let config = FileSystemConfig::builder()
//!         .with_read_timeout(Duration::from_secs(10))
//!         .build();
//!     let fs = RealFileSystem::with_config(config);
//!
//!     // Use the filesystem
//!     let content = fs.read_to_string(Path::new("package.json")).await?;
//!     println!("Content: {}", content);
//!
//!     Ok(())
//! }
//! ```

// TODO: will be implemented on epic workspace-node-tools-1gx (RealFileSystem Implementation)
#![allow(clippy::todo)]
