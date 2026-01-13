//! # Path Extension Module
//!
//! Provides utility extensions for working with filesystem paths.
//!
//! ## What
//!
//! This module defines the [`PathExt`] trait, which adds convenience methods to
//! [`std::path::Path`] for common path manipulation tasks. These are pure functions
//! that operate on path data without performing any I/O operations.
//!
//! ## How
//!
//! The trait is implemented for `Path` and provides methods such as:
//! - Path normalization (resolving `.` and `..` components)
//! - Cross-platform path handling
//! - Extension manipulation utilities
//!
//! All methods are synchronous because they operate only on in-memory path data,
//! not on the filesystem. This makes them safe to use in any context.
//!
//! ## Why
//!
//! Path extension utilities are valuable for:
//! - **Normalization**: Ensure consistent path representation across platforms
//! - **Safety**: Validate paths before filesystem operations
//! - **Convenience**: Common operations available as methods, not functions
//! - **Portability**: Abstract over platform-specific path separators
//!
//! ## Example
//!
//! ```rust,ignore
//! use workspace_fs::PathExt;
//! use std::path::Path;
//!
//! let path = Path::new("/foo/bar/../baz/./qux");
//!
//! // Normalize path without touching filesystem
//! let normalized = path.normalize();
//! assert_eq!(normalized, Path::new("/foo/baz/qux"));
//! ```

// TODO: will be implemented on epic workspace-node-tools-60y (PathExt Module)
#![allow(clippy::todo)]
