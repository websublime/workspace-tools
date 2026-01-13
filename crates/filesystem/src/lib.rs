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

// =============================================================================
// Safety: This crate contains no unsafe code (NFR-4.1)
// =============================================================================
#![forbid(unsafe_code)]
// =============================================================================
// Documentation lints
// =============================================================================
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
// =============================================================================
// Code quality lints (deny unwrap/expect/todo/unimplemented/panic in production)
// =============================================================================
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

// =============================================================================
// Module Declarations
// =============================================================================

/// Error types for filesystem operations.
///
/// Provides a unified [`Error`](error::Error) enum that captures all possible error
/// conditions with path context for debugging.
pub mod error;

/// Configuration types for filesystem behavior.
///
/// Provides [`FileSystemConfig`](config) with timeout settings and a builder
/// pattern for customization.
pub(crate) mod config;

/// Core data types for filesystem entries.
///
/// Provides [`FileType`](types), [`DirEntry`](types), and [`Metadata`](types)
/// types used throughout the crate.
pub(crate) mod types;

/// The core filesystem trait definition.
///
/// Defines the [`FileSystem`](traits) trait that abstracts all filesystem
/// operations for dependency injection and testing.
pub(crate) mod traits;

/// Path utility extensions.
///
/// Provides the [`PathExt`](path_ext) trait with synchronous path manipulation
/// utilities that don't perform I/O.
pub(crate) mod path_ext;

/// Real filesystem implementation using tokio::fs.
///
/// Provides [`RealFileSystem`](real) for production use with async I/O
/// and configurable timeout support.
pub(crate) mod real;

/// Mock filesystem implementation for testing.
///
/// Provides [`MockFileSystem`](mock) for fast, deterministic unit tests
/// without touching the disk.
pub(crate) mod mock;

/// Unit tests organized by module.
#[cfg(test)]
mod tests;

// =============================================================================
// Public Re-exports
// =============================================================================

// Error types
pub use error::{Error, Result};

// TODO: Re-exports will be added as modules are implemented
// pub use config::{FileSystemConfig, FileSystemConfigBuilder};
// pub use mock::MockFileSystem;
// pub use path_ext::PathExt;
// pub use real::RealFileSystem;
// pub use traits::FileSystem;
// pub use types::{DirEntry, FileType, Metadata};
