//! # Error Module
//!
//! Defines the unified error type for all filesystem operations in workspace-fs.
//!
//! ## What
//!
//! This module provides a single [`Error`] enum that captures all possible error
//! conditions that can occur during filesystem operations. It uses the `snafu` crate
//! for ergonomic error handling with context.
//!
//! ## How
//!
//! The error type is built using `snafu`'s derive macro, which automatically generates:
//! - `Error` and `Display` implementations
//! - Context selectors for each variant
//! - Backtrace capture (when enabled)
//!
//! Each variant includes the path that caused the error and, where applicable, the
//! underlying system error. This provides rich debugging information.
//!
//! ## Why
//!
//! A unified error type ensures:
//! - **Consistency**: All filesystem operations return the same error type
//! - **Context Preservation**: Every error includes the path and operation that failed
//! - **Type Safety**: Compile-time guarantees about error handling
//! - **Ergonomics**: Easy error conversion with the `?` operator
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::{Error, Result};
//! use std::path::Path;
//!
//! async fn read_config(path: &Path) -> Result<String> {
//!     // Errors automatically include the path that failed
//!     fs.read_to_string(path).await
//! }
//! ```

// TODO: will be implemented on epic workspace-node-tools-906 (Error Module)
#![allow(clippy::todo)]
