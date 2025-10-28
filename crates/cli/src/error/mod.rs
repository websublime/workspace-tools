//! Error handling module for the CLI.
//!
//! This module defines all error types used in the CLI and provides user-friendly
//! error messages and exit codes following the sysexits convention.
//!
//! # What
//!
//! Provides:
//! - `CliError` enum for all CLI error cases
//! - User-friendly error messages
//! - Exit code mapping following sysexits standards
//! - Error context and conversion utilities
//!
//! # How
//!
//! Wraps errors from internal crates and system operations into a unified
//! `CliError` type that can be displayed to users with helpful context and
//! suggestions. Exit codes follow the sysexits convention for proper shell
//! integration.
//!
//! # Why
//!
//! Centralized error handling ensures consistent error messages and exit codes
//! across all commands, improving user experience and making the CLI more
//! predictable in scripts and automation.
//!
//! ## Examples
//!
//! ```rust
//! use sublime_cli::error::{CliError, Result};
//!
//! fn example_operation() -> Result<()> {
//!     Err(CliError::Configuration(
//!         "Configuration file not found".to_string()
//!     ))
//! }
//! ```

use std::fmt;

/// Result type alias for CLI operations.
///
/// This is the standard result type used throughout the CLI, wrapping
/// the `CliError` enum for error cases.
pub type Result<T> = std::result::Result<T, CliError>;

/// Main error type for CLI operations.
///
/// This enum represents all possible error conditions that can occur
/// during CLI execution. Each variant provides context-specific information
/// and maps to an appropriate exit code.
///
/// # Examples
///
/// ```rust
/// use sublime_cli::error::CliError;
///
/// let error = CliError::Configuration("Invalid config".to_string());
/// assert_eq!(error.exit_code(), 78);
/// ```
#[derive(Debug)]
pub enum CliError {
    /// Configuration-related errors (invalid, not found, parsing failed)
    ///
    /// Exit code: 78 (EX_CONFIG)
    Configuration(String),

    /// Validation errors (invalid arguments, invalid state)
    ///
    /// Exit code: 65 (EX_DATAERR)
    Validation(String),

    /// Execution errors (command failed, operation failed)
    ///
    /// Exit code: 70 (EX_SOFTWARE)
    Execution(String),

    /// Git-related errors (repository not found, git operation failed)
    ///
    /// Exit code: 70 (EX_SOFTWARE)
    Git(String),

    /// Package-related errors (package not found, package.json invalid)
    ///
    /// Exit code: 65 (EX_DATAERR)
    Package(String),

    /// I/O errors (file not found, permission denied)
    ///
    /// Exit code: 74 (EX_IOERR)
    Io(String),

    /// Network errors (registry unreachable, download failed)
    ///
    /// Exit code: 69 (EX_UNAVAILABLE)
    Network(String),

    /// User-caused errors (invalid input, cancelled operation)
    ///
    /// Exit code: 64 (EX_USAGE)
    User(String),
}

impl CliError {
    /// Returns the exit code for this error following sysexits convention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli::error::CliError;
    ///
    /// let error = CliError::Configuration("Invalid config".to_string());
    /// assert_eq!(error.exit_code(), 78);
    /// ```
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Configuration(_) => 78,                 // EX_CONFIG
            Self::Validation(_) | Self::Package(_) => 65, // EX_DATAERR
            Self::Execution(_) | Self::Git(_) => 70,      // EX_SOFTWARE
            Self::Io(_) => 74,                            // EX_IOERR
            Self::Network(_) => 69,                       // EX_UNAVAILABLE
            Self::User(_) => 64,                          // EX_USAGE
        }
    }

    /// Returns a user-friendly error message.
    ///
    /// This message is displayed to the user and should provide clear
    /// information about what went wrong and how to fix it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli::error::CliError;
    ///
    /// let error = CliError::Configuration("File not found".to_string());
    /// let message = error.user_message();
    /// assert!(message.contains("Configuration error"));
    /// ```
    pub fn user_message(&self) -> String {
        match self {
            Self::Configuration(msg) => format!("Configuration error: {msg}"),
            Self::Validation(msg) => format!("Validation error: {msg}"),
            Self::Execution(msg) => format!("Execution error: {msg}"),
            Self::Git(msg) => format!("Git error: {msg}"),
            Self::Package(msg) => format!("Package error: {msg}"),
            Self::Io(msg) => format!("I/O error: {msg}"),
            Self::Network(msg) => format!("Network error: {msg}"),
            Self::User(msg) => format!("Error: {msg}"),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for CliError {}

// Error conversions from internal crates
// These will be implemented as we integrate with the internal crates

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Execution(format!("JSON serialization error: {error}"))
    }
}

impl From<toml::de::Error> for CliError {
    fn from(error: toml::de::Error) -> Self {
        Self::Configuration(format!("TOML parsing error: {error}"))
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Configuration(format!("YAML parsing error: {error}"))
    }
}

// TODO: will be implemented in story 1.3
// Additional modules for error handling:
// - display.rs: Enhanced error display with colors and suggestions
// - exit_codes.rs: Exit code constants and utilities
// - tests.rs: Error handling tests
