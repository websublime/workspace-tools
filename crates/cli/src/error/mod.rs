//! Error handling module for the CLI.
//!
//! This module defines all error types used in the CLI and provides user-friendly
//! error messages and exit codes following the sysexits convention.
//!
//! # What
//!
//! Provides:
//! - `CliError` enum for all CLI error cases
//! - User-friendly error messages with suggestions
//! - Exit code mapping following sysexits standards
//! - Error context and conversion utilities
//! - Enhanced error display with colors
//!
//! # How
//!
//! Wraps errors from internal crates (git, package, standard tools) and system
//! operations into a unified `CliError` type that can be displayed to users with
//! helpful context and suggestions. Exit codes follow the sysexits convention for
//! proper shell integration.
//!
//! # Why
//!
//! Centralized error handling ensures consistent error messages and exit codes
//! across all commands, improving user experience and making the CLI more
//! predictable in scripts and automation.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::error::{CliError, Result};
//!
//! fn example_operation() -> Result<()> {
//!     Err(CliError::configuration("Configuration file not found"))
//! }
//! ```
//!
//! Converting from library errors:
//!
//! ```rust
//! use sublime_cli_tools::error::{CliError, Result};
//! use sublime_git_tools::RepoError;
//!
//! fn git_operation() -> Result<()> {
//!     // Library errors are automatically converted
//!     let repo = sublime_git_tools::Repo::open("/path/to/repo")?;
//!     Ok(())
//! }
//! ```
//!
//! Enhanced error display:
//!
//! ```rust
//! use sublime_cli_tools::error::{CliError, ErrorDisplay};
//!
//! let error = CliError::configuration("Config file not found");
//! let display = ErrorDisplay::new(&error, true);
//! println!("{}", display.format());
//! ```

mod cli_error;
mod display;
mod exit_codes;

#[cfg(test)]
mod tests;

// Public re-exports
pub use cli_error::{CliError, Result};
pub use display::ErrorDisplay;
pub use exit_codes::ExitCode;
