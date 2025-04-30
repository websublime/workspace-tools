//! # Filesystem Operations Module
//!
//! ## What
//! This module provides abstractions for interacting with the filesystem in a safe
//! and consistent manner. It offers a trait-based approach to file operations that
//! can be implemented by different providers or mocked for testing.
//!
//! ## How
//! The module exposes a `FileSystem` trait that defines standard filesystem operations
//! and a concrete `FileSystemManager` implementation that performs real filesystem
//! operations using standard library functions and the `walkdir` crate for directory
//! traversal.
//!
//! ## Why
//! Filesystem operations are error-prone and platform-dependent. This module
//! provides a consistent API with robust error handling to simplify working with
//! files and directories across different platforms while maintaining proper
//! error context and type safety.

mod manager;
mod paths;
mod types;

#[cfg(test)]
mod tests;

pub use types::{FileSystem, FileSystemManager, NodePathKind, PathExt, PathUtils};
