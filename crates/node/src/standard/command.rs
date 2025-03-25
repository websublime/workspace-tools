use napi::bindgen_prelude::*;
use sublime_standard_tools::{execute, CommandError};

/// Custom error type for handling JavaScript-facing command execution errors
///
/// This enum represents different types of errors that can occur during command execution:
/// - `NapiError`: Errors from the Node-API layer
/// - `CommandError`: Errors from the underlying command execution
#[derive(Debug)]
#[allow(dead_code)]
pub enum JsCommandError {
    /// Error originating from the Node-API layer
    NapiError(Error),
    /// Error originating from the command execution
    CommandError(CommandError),
}

impl AsRef<str> for JsCommandError {
    fn as_ref(&self) -> &str {
        match self {
            JsCommandError::NapiError(e) => e.status.as_ref(),
            JsCommandError::CommandError(e) => e.as_ref(),
        }
    }
}

impl From<CommandError> for JsCommandError {
    fn from(err: CommandError) -> Self {
        JsCommandError::CommandError(err)
    }
}

impl From<JsCommandError> for Error {
    fn from(err: JsCommandError) -> Self {
        match err {
            JsCommandError::NapiError(e) => e,
            JsCommandError::CommandError(e) => {
                let message = match &e {
                    CommandError::Run(io_err) => format!("Failed to run command: {}", io_err),
                    CommandError::Execution => "Failed to execute command".to_string(),
                    CommandError::Failure { stdout, stderr } => {
                        format!("Command failed with stdout: {} stderr: {}", stdout, stderr)
                    }
                };
                Error::new(Status::GenericFailure, message)
            }
        }
    }
}

fn command_format_napi_error(err: CommandError) -> Error<JsCommandError> {
    Error::new(err.clone().into(), err.to_string())
}

/**
 * Executes a shell command and returns its output as a string.
 *
 * This function provides a bridge between JavaScript and Rust for executing shell commands.
 * It runs the specified command in the given working directory with the provided arguments,
 * capturing and returning the standard output.
 *
 * @param {string} cmd - The command to execute (e.g., "git", "npm", "ls")
 * @param {string} path - The working directory where the command will be executed
 * @param {string[]} args - An array of arguments to pass to the command
 * @returns {string} Output the command's standard output as a string
 * @throws {Error} If the command fails to run, execute, or returns a non-zero exit code.
 *                 The error message will contain details about the failure, including
 *                 stdout and stderr content for non-zero exit codes.
 *
 * @example
 * // Execute 'git status' in the current project directory
 * const output = executeCommand('git', '/path/to/project', ['status']);
 * console.log(output);
 */
#[napi(js_name = "executeCommand", ts_return_type = "string")]
pub fn js_execute(cmd: String, path: String, args: Vec<String>) -> Result<String, JsCommandError> {
    execute(cmd, path, args, |stdout, _| Ok(stdout.to_string())).map_err(command_format_napi_error)
}
