//! # workspace-fs
//!
//! A unified filesystem abstraction layer for the workspace-node-tools ecosystem.
//!
//! ## What
//!
//! This crate provides a trait-based abstraction over filesystem operations, enabling:
//! - Consistent filesystem access across the entire workspace-node-tools ecosystem
//! - Comprehensive testing through mock filesystem implementations
//! - High-performance asynchronous I/O for large monorepo operations
//!
//! ## How
//!
//! The crate defines a `FileSystem` trait that abstracts all filesystem operations.
//! Two implementations are provided:
//! - `RealFileSystem`: Production implementation using `tokio::fs`
//! - `MockFileSystem`: In-memory implementation for testing
//!
//! ## Why
//!
//! Centralizing filesystem operations in a single crate provides:
//! - **Testability**: Mock filesystems enable fast, deterministic unit tests
//! - **Consistency**: All crates use the same filesystem semantics
//! - **Performance**: Async operations optimize I/O-bound workloads
//! - **Portability**: Abstract away platform-specific filesystem quirks
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{FileSystem, RealFileSystem};
//!
//! async fn read_config(fs: &impl FileSystem) -> Result<String, workspace_fs::Error> {
//!     fs.read_to_string("config.json").await
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let fs = RealFileSystem::new();
//!     match read_config(&fs).await {
//!         Ok(content) => println!("Config: {}", content),
//!         Err(e) => eprintln!("Failed to read config: {}", e),
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
