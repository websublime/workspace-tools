use std::{
    ffi::OsStr,
    fs::canonicalize,
    path::Path,
    process::{Command, Output, Stdio},
    str::from_utf8,
};

use super::error::CommandError;
use super::utils::strip_trailing_newline;

/// Result type alias for command execution functions that return a `Result<T, CommandError>`.
///
/// This type is used to simplify function signatures and provide a consistent
/// error handling pattern for command execution.
pub type ComandResult<T> = Result<T, CommandError>;

/// Executes an external command in a specified directory with the given arguments.
///
/// # Parameters
///
/// * `cmd` - The command to execute (e.g., "git", "npm", etc.)
/// * `path` - The directory path in which to execute the command
/// * `args` - An iterator of arguments to pass to the command
/// * `process` - A function that processes the command's output if successful
///
/// # Type Parameters
///
/// * `P` - A type that can be converted to a `Path` (e.g., `&str`, `PathBuf`)
/// * `I` - An iterator type whose items can be converted to `OsStr`
/// * `F` - A function type that processes command output
/// * `S` - A type that can be converted to `OsStr` (for command name and arguments)
/// * `R` - The return type of the process function
///
/// # Returns
///
/// * `Result<R, CommandError>` - The processed result of the command execution or an error
///
/// # Error Handling
///
/// This function can return several types of errors:
/// * Path canonicalization errors when the provided path is invalid
/// * Command execution errors when the command fails to start
/// * Command failure errors with captured stdout/stderr when the command returns a non-zero exit code
/// * Execution errors when the output cannot be properly decoded as UTF-8
///
/// # Example
///
/// ```no_run
/// use std::process::Output;
/// use my_crate::command::{execute, ComandResult};
///
/// fn process_git_output(stdout: &str, _output: &Output) -> ComandResult<String> {
///     Ok(stdout.to_string())
/// }
///
/// let result = execute("git", ".", ["status"], process_git_output);
/// ```
pub fn execute<P, I, F, S, R>(cmd: S, path: P, args: I, process: F) -> Result<R, CommandError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: Fn(&str, &Output) -> ComandResult<R>,
{
    // Convert the provided path to a canonical path
    let canonic_path = &canonicalize(&path)?;

    // Create and configure the command process
    let output = Command::new(cmd)
        .current_dir(canonic_path.as_path()) // Set working directory
        .args(args) // Add command arguments
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()) // Capture stderr
        .spawn() // Start the process
        .map_err(CommandError::Run)?; // Convert error to CommandError

    // Wait for the process to complete and handle its output
    output.wait_with_output().map_err(CommandError::Run).and_then(|output| {
        if output.status.success() {
            // Command executed successfully
            if let Ok(message) = from_utf8(&output.stdout) {
                // Process the output using the provided callback function
                process(strip_trailing_newline(&message.to_string()).as_str(), &output)
            } else {
                // Failed to decode stdout as UTF-8
                Err(CommandError::Execution)
            }
        } else if let Ok(message) = from_utf8(&output.stdout) {
            // Command failed (non-zero exit code)
            if let Ok(err) = from_utf8(&output.stderr) {
                // Create a detailed error with both stdout and stderr
                Err(CommandError::Failure { stdout: message.to_string(), stderr: err.to_string() })
            } else {
                // Failed to decode stderr as UTF-8
                Err(CommandError::Execution)
            }
        } else {
            // Failed to decode any output as UTF-8
            Err(CommandError::Execution)
        }
    })
}
