//! # Traits Module
//!
//! Defines the core [`FileSystem`] trait that abstracts all filesystem operations.
//!
//! ## What
//!
//! This module provides the `FileSystem` trait, which is the central abstraction of
//! the workspace-fs crate. The trait defines a complete set of asynchronous filesystem
//! operations that can be implemented by different backends.
//!
//! ## How
//!
//! The trait uses `async_trait` to enable async methods in traits, which Rust does not
//! natively support with object safety. Each method:
//! - Takes `&self` to allow shared access (implementations handle internal synchronization)
//! - Accepts paths as `impl AsRef<Path>` for ergonomic use with `&str`, `String`, `PathBuf`, etc.
//! - Returns `Result<T, Error>` using the crate's error type
//!
//! Two implementations are provided:
//! - [`RealFileSystem`](crate::RealFileSystem): Production implementation using `tokio::fs`
//! - [`MockFileSystem`](crate::MockFileSystem): In-memory implementation for testing
//!
//! ## Why
//!
//! A trait-based abstraction enables:
//! - **Dependency Injection**: Pass filesystem as a parameter, not a global
//! - **Testing**: Swap in `MockFileSystem` for fast, deterministic unit tests
//! - **Flexibility**: Could add other implementations (e.g., cached, logged, remote)
//! - **Decoupling**: Application code doesn't depend on specific filesystem implementation
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileSystem, RealFileSystem, MockFileSystem};
//! use std::path::Path;
//!
//! // Generic function works with any FileSystem implementation
//! async fn read_json<FS: FileSystem>(fs: &FS, path: &Path) -> Result<String, workspace_fs::Error> {
//!     fs.read_to_string(path).await
//! }
//!
//! // In production
//! let fs = RealFileSystem::new();
//! let content = read_json(&fs, Path::new("config.json")).await?;
//!
//! // In tests
//! let mock = MockFileSystem::new();
//! mock.write("config.json", r#"{"key": "value"}"#).await?;
//! let content = read_json(&mock, Path::new("config.json")).await?;
//! ```

// TODO: will be implemented on epic workspace-node-tools-hek (FileSystem Trait)
#![allow(clippy::todo)]
