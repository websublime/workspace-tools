//! Command execution error type.
//!
//! What:
//! Defines specific error types for command execution failures, providing
//! detailed error information for command-related operations.
//!
//! Who:
//! Used by developers who need to:
//! - Handle command execution failures
//! - Provide detailed error information about command failures
//! - Implement custom command error handling
//!
//! Why:
//! Command execution requires specific error handling to provide proper
//! context about what went wrong during command execution.

use std::io;
use std::time::Duration;
use thiserror::Error;

/// Error type for command execution failures.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::error::CommandError;
/// use std::time::Duration;
///
/// let error = CommandError::Timeout { duration: Duration::from_secs(5) };
/// assert!(error.to_string().contains("timed out"));
/// ```
#[derive(Error, Debug)]
pub enum CommandError {
    /// The command failed to start (e.g., not found).
    #[error("Failed to spawn command '{cmd}': {source}")]
    SpawnFailed {
        /// The command that failed to start
        cmd: String,
        /// The underlying IO error that caused the spawn failure
        #[source]
        source: io::Error,
    },

    /// The command execution process itself failed (e.g., internal I/O error).
    #[error("Command execution failed for '{cmd}': {source:?}")]
    ExecutionFailed {
        /// The command that failed during execution
        cmd: String,
        /// The optional IO error that caused the execution failure
        #[source]
        source: Option<io::Error>,
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

    /// Resource limits were exceeded during execution.
    #[error("Resource limits exceeded: {limit}")]
    ResourceExceeded {
        /// Description of the resource limit that was exceeded
        limit: String,
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
    #[error("Error reading {stream} stream: {source}")]
    StreamReadError {
        /// Name of the stream that encountered a read error
        stream: String,
        /// The underlying IO error that caused the read failure
        #[source]
        source: io::Error,
    },

    /// Generic error during command processing.
    #[error("Command processing error: {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_display() {
        let timeout_error = CommandError::Timeout { duration: Duration::from_secs(5) };
        assert_eq!(timeout_error.to_string(), "Command timed out after 5s");

        let exit_error = CommandError::NonZeroExitCode {
            cmd: "test".to_string(),
            code: Some(1),
            stderr: "error message".to_string(),
        };
        assert_eq!(
            exit_error.to_string(),
            "Command 'test' failed with exit code Some(1). Stderr: error message"
        );

        let spawn_error = CommandError::SpawnFailed {
            cmd: "test".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };
        assert_eq!(spawn_error.to_string(), "Failed to spawn command 'test': not found");
    }
}
