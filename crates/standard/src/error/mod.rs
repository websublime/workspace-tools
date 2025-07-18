//! # Error handling for `sublime_standard_tools`
//!
//! ## What
//! This module provides comprehensive error types for various operations within
//! the crate. It implements specific error types for different domains, such as
//! filesystem operations, command execution, and project management.
//!
//! ## How
//! Each domain has its own error type (e.g., `FileSystemError`) that implements
//! the `Error` trait from the standard library and uses the `thiserror` crate
//! for concise error definitions. Result type aliases are provided for convenience.
//!
//! ## Why
//! A structured approach to error handling enables callers to handle errors
//! appropriately based on their type and context, improving error reporting and
//! recovery strategies. The consistent pattern makes error handling predictable
//! across the crate.

mod filesystem;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    CommandError, CommandResult, Error, FileSystemError, FileSystemResult, MonorepoError,
    MonorepoResult, Result, WorkspaceError, WorkspaceResult,
};

