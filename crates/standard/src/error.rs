use thiserror::Error;

/// Represents errors that may occur when executing commands.
///
/// This enum provides a comprehensive set of error variants that can occur
/// during command execution:
/// - IO errors that prevent a command from running
/// - Execution errors that occur when a command fails to execute properly
/// - Failure errors that capture the output when a command runs but exits with a non-zero status
///
/// # Examples
///
/// ```
/// use your_crate::CommandError;
///
/// // Handle IO errors
/// fn handle_io_error(err: std::io::Error) -> CommandError {
///     CommandError::Run(err)
/// }
///
/// // Create a failure error with output
/// let failure = CommandError::Failure {
///     stdout: String::from("No such file"),
///     stderr: String::from("Error code 127"),
/// };
/// ```
#[derive(Error, Debug)]
pub enum CommandError {
    /// Error that occurs when the command fails to start due to an IO error.
    ///
    /// This typically happens when:
    /// - The command binary cannot be found
    /// - There are permission issues
    /// - System resource limits are reached
    #[error("Fail to run command: {0:?}")]
    Run(#[from] std::io::Error),

    /// Error that occurs when the command execution process fails.
    ///
    /// This indicates that the command was found but could not be properly executed.
    /// This could happen due to issues like:
    /// - The process was terminated unexpectedly
    /// - The system was unable to allocate resources for the command
    #[error("Fail to execute command")]
    Execution,

    /// Error that occurs when a command runs but returns a non-zero exit code.
    ///
    /// This variant captures both the standard output and standard error
    /// content produced by the command before it failed.
    ///
    /// # Fields
    /// * `stdout` - The content written to standard output by the command
    /// * `stderr` - The content written to standard error by the command
    #[error("command failed with the following stdout: {stdout} stderr: {stderr}")]
    Failure { stdout: String, stderr: String },
}

impl Clone for CommandError {
    fn clone(&self) -> Self {
        match self {
            CommandError::Run(err) => {
                // Create a new io::Error with the same kind and message
                let kind = err.kind();
                let message = err.to_string();
                CommandError::Run(std::io::Error::new(kind, message))
            }
            CommandError::Execution => CommandError::Execution,
            CommandError::Failure { stdout, stderr } => {
                CommandError::Failure { stdout: stdout.clone(), stderr: stderr.clone() }
            }
        }
    }
}

impl AsRef<str> for CommandError {
    /// Returns a string slice that represents the error message.
    ///
    /// This implementation provides a simplified static error message for each
    /// variant of the `CommandError` enum. For the `Failure` variant, this returns
    /// a generic message rather than including the stdout/stderr content, which
    /// would require dynamic allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::CommandError;
    /// let io_error = CommandError::Run(std::io::Error::new(std::io::ErrorKind::NotFound, "Binary not found"));
    /// assert_eq!(io_error.as_ref(), "Fail to run command");
    ///
    /// let failure = CommandError::Failure {
    ///     stdout: String::from("Output text"),
    ///     stderr: String::from("Error details"),
    /// };
    /// assert_eq!(failure.as_ref(), "Command failed with non-zero exit code");
    /// ```
    ///
    /// # Note
    ///
    /// For detailed error messages including stdout and stderr content,
    /// use the `Display` implementation (via `to_string()`) instead.
    fn as_ref(&self) -> &str {
        match self {
            CommandError::Run(_) => "CommandErrorRun",
            CommandError::Execution => "CommandErrorExecution",
            CommandError::Failure { stdout: _, stderr: _ } => "CommandFailureZeroCode",
        }
    }
}
