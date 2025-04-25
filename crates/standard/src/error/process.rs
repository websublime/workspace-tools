//! Process management error type.
//!
//! What:
//! Defines specific error types for process management failures, providing
//! detailed error information for process-related operations.
//!
//! Who:
//! Used by developers who need to:
//! - Handle process management failures
//! - Track process lifecycle errors
//! - Implement custom process error handling
//!
//! Why:
//! Process management requires specific error handling to provide proper
//! context about what went wrong during process lifecycle operations.

use std::io;
use thiserror::Error;

/// Error type for process management failures.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::error::ProcessError;
/// use std::io;
///
/// let error = ProcessError::SpawnFailed { source: io::Error::new(io::ErrorKind::NotFound, "not found") };
/// assert!(error.to_string().contains("Failed to spawn process"));
/// ```
#[derive(Error, Debug)]
pub enum ProcessError {
    /// Failed to spawn a new process.
    #[error("Failed to spawn process: {source}")]
    SpawnFailed {
        /// The underlying I/O error that caused the spawn failure.
        #[source]
        source: io::Error,
    },

    /// Failed to kill a running process.
    #[error("Failed to kill process (PID: {pid:?}): {source}")]
    KillFailed {
        /// The process identifier that couldn't be killed, if available.
        pid: Option<u32>,
        /// The underlying I/O error that caused the kill failure.
        #[source]
        source: io::Error,
    },

    /// Process exited with a non-zero status code.
    #[error("Process exited with error (code: {code:?})")]
    ExitError {
        /// The exit code returned by the process, if available.
        code: Option<i32>,
    },

    /// Failed to wait for a process to complete.
    #[error("Failed to wait for process: {source}")]
    WaitFailed {
        /// The underlying I/O error that caused the wait failure.
        #[source]
        source: io::Error,
    },

    /// An error occurred within a process pool.
    #[error("Process pool error: {description}")]
    PoolError {
        /// A description of what went wrong in the process pool.
        description: String,
    },

    /// Process ID (PID) was expected but not found.
    #[error("Process ID not available")]
    PidUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_error_display() {
        let spawn_error = ProcessError::SpawnFailed {
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };
        assert_eq!(spawn_error.to_string(), "Failed to spawn process: not found");

        let exit_error = ProcessError::ExitError { code: Some(1) };
        assert_eq!(exit_error.to_string(), "Process exited with error (code: Some(1))");
    }
}
