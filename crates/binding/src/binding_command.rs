use napi::{Error, Result, Status};
use ws_std::command::execute;

pub enum CommandError {
    InvalidCommandResult(String),
    NapiError(Error<Status>),
}

impl AsRef<str> for CommandError {
    fn as_ref(&self) -> &str {
        match self {
            Self::InvalidCommandResult(e) => e.as_str(),
            Self::NapiError(e) => e.status.as_ref(),
        }
    }
}

/// Execute a command.
///
/// @param {string} cmd - The command to execute.
/// @param {string} cwd - The command working directory.
/// @param {string[]} args - The command arguments.
/// @returns {string} The command output.
///
/// @throws {Error} The error description.
#[napi(js_name = "executeCmd", ts_return_type = "Result<String>")]
pub fn js_execute_command(
    cmd: String,
    cwd: String,
    args: Option<Vec<String>>,
) -> Result<String, CommandError> {
    let cmd_args = args.unwrap_or_default();

    match execute(cmd, cwd, cmd_args, |message, _| Ok(message.to_string())) {
        Ok(output) => Ok(output),
        Err(e) => Err(Error::new(CommandError::InvalidCommandResult(e.to_string()), e.to_string())),
    }
}
