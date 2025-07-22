//! # Command execution error types
//!
//! ## What
//! This module defines errors that can occur during command execution,
//! including spawn failures, timeouts, and non-zero exit codes.
//!
//! ## How
//! The `CommandError` enum provides specific variants for different failure modes
//! during command execution, with descriptive error messages and relevant context.
//!
//! ## Why
//! Separating command errors allows for precise error handling and recovery
//! strategies specific to command execution scenarios.

use core::result::Result as CoreResult;
use std::time::Duration;
use thiserror::Error as ThisError;

/// Errors that can occur during command execution.
///
/// This enum represents the various ways that command execution can fail,
/// from spawn failures to timeouts to non-zero exit codes, with specific
/// variants for common error conditions.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{CommandError, Error};
/// use std::time::Duration;
///
/// // Creating a timeout error
/// let error = CommandError::Timeout {
///     duration: Duration::from_secs(30)
/// };
///
/// // Converting to the general Error type
/// let general_error: Error = error.into();
/// ```
#[derive(ThisError, Debug, Clone)]
pub enum CommandError {
    /// The command failed to start (e.g., not found).
    #[error("Failed to spawn command '{cmd}': {message}")]
    SpawnFailed {
        /// The command that failed to start
        cmd: String,
        /// The spawn failure error message
        message: String,
    },

    /// The command execution process itself failed (e.g., internal I/O error).
    #[error("Command execution failed for '{cmd}': {message}")]
    ExecutionFailed {
        /// The command that failed during execution
        cmd: String,
        /// The execution failure error message
        message: String,
    },

    /// The command executed but returned a non-zero exit code.
    #[error("Command '{cmd}' failed with exit code {code:?}. Stderr: {stderr}")]
    NonZeroExitCode {
        /// The command that returned a non-zero exit code
        cmd: String,
        /// The exit code returned by the command
        code: Option<i32>,
        /// The error output captured from the command
        stderr: String,
    },

    /// The command timed out after the specified duration.
    #[error("Command timed out after {duration:?}")]
    Timeout {
        /// The time period after which the command timed out
        duration: Duration,
    },

    /// The command was killed (e.g., by a signal).
    #[error("Command was killed: {reason}")]
    Killed {
        /// The reason why the command was killed
        reason: String,
    },

    /// Invalid configuration provided for the command.
    #[error("Invalid command configuration: {description}")]
    Configuration {
        /// Description of the configuration error
        description: String,
    },

    /// Failed to capture stdout or stderr.
    #[error("Failed to capture {stream} stream")]
    CaptureFailed {
        /// Name of the stream that failed to capture (stdout/stderr)
        stream: String,
    },

    /// Error occurred while reading stdout or stderr stream.
    #[error("Error reading {stream} stream: {message}")]
    StreamReadError {
        /// Name of the stream that encountered a read error
        stream: String,
        /// The stream read error message
        message: String,
    },

    /// Generic error during command processing.
    #[error("Command processing error: {0}")]
    Generic(String),
}

/// Result type for command operations.
///
/// This is a convenience type alias for Results with `CommandError`.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::error::{CommandResult, CommandError};
///
/// fn execute_build_command(args: &[&str]) -> CommandResult<String> {
///     if args.is_empty() {
///         return Err(CommandError::Configuration {
///             description: "No build arguments provided".to_string(),
///         });
///     }
///     // Actual implementation would execute the command
///     Ok("Build completed successfully".to_string())
/// }
/// ```
pub type CommandResult<T> = CoreResult<T, CommandError>;

impl AsRef<str> for CommandError {
    fn as_ref(&self) -> &str {
        match self {
            CommandError::SpawnFailed { .. } => "CommandError::SpawnFailed",
            CommandError::ExecutionFailed { .. } => "CommandError::ExecutionFailed",
            CommandError::NonZeroExitCode { .. } => "CommandError::NonZeroExitCode",
            CommandError::Timeout { .. } => "CommandError::Timeout",
            CommandError::Killed { .. } => "CommandError::Killed",
            CommandError::Configuration { .. } => "CommandError::Configuration",
            CommandError::CaptureFailed { .. } => "CommandError::CaptureFailed",
            CommandError::StreamReadError { .. } => "CommandError::StreamReadError",
            CommandError::Generic(_) => "CommandError::Generic",
        }
    }
}