//! # Filesystem Operations Module - Async Only
//!
//! ## What
//! This module provides abstractions for interacting with the filesystem using async operations.
//! It offers a trait-based approach to file operations with a unified async-only architecture.
//!
//! ## How
//! The module exposes an `AsyncFileSystem` trait that defines async filesystem operations
//! and a concrete `FileSystemManager` implementation that performs real filesystem
//! operations using tokio::fs for maximum performance.
//!
//! ## Why
//! Async filesystem operations are essential for performance in large repositories.
//! This unified async-only approach eliminates confusion between sync and async operations
//! and provides a consistent API with robust error handling across all platforms.

mod manager;
mod paths;
mod types;

#[cfg(test)]
mod tests;

pub use manager::FileSystemManager;
pub use types::{AsyncFileSystem, AsyncFileSystemConfig, NodePathKind, PathExt, PathUtils};