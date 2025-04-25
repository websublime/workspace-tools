//! Error handling for the sublime_standard_tools crate.
//!
//! What:
//! This module provides a comprehensive error handling system for the sublime_standard_tools crate.
//! It defines a set of error types that cover all possible failure scenarios across the crate's
//! functionality.
//!
//! Who:
//! This module is used by both internal crate developers and external consumers who need to
//! handle errors from the crate's operations.
//!
//! Why:
//! A robust error handling system is essential for providing clear, actionable error messages
//! and enabling proper error recovery strategies.

mod command;
mod fs;
mod process;
mod standard;

pub use command::CommandError;
pub use fs::FileSystemError;
pub use process::ProcessError;
pub use standard::StandardError;

/// Result type alias for StandardError
pub type StandardResult<T> = Result<T, StandardError>;

/// Result type alias for FileSystemError
pub type FileSystemResult<T> = Result<T, FileSystemError>;

/// Result type alias for CommandError
pub type CommandResult<T> = Result<T, CommandError>;

/// Result type alias for ProcessError
pub type ProcessResult<T> = Result<T, ProcessError>;

