use std::{
    ffi::OsStr,
    fs::canonicalize,
    path::Path,
    process::{Command, Output},
    str::from_utf8,
};

use super::error::CommandError;

pub type ComandResult<T> = Result<T, CommandError>;

pub fn execute<P, I, F, S, R>(cmd: S, path: P, args: I, process: F) -> Result<R, CommandError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: Fn(&str, &Output) -> ComandResult<R>,
{
    let canonic_path = &canonicalize(&path)?;
    let output = Command::new(cmd).current_dir(canonic_path.as_path()).args(args).output();

    output.map_err(CommandError::Run).and_then(|output| {
        if output.status.success() {
            if let Ok(message) = from_utf8(&output.stdout) {
                process(message, &output)
            } else {
                Err(CommandError::Execution)
            }
        } else if let Ok(message) = from_utf8(&output.stdout) {
            if let Ok(err) = from_utf8(&output.stderr) {
                Err(CommandError::Failure { stdout: message.to_string(), stderr: err.to_string() })
            } else {
                Err(CommandError::Execution)
            }
        } else {
            Err(CommandError::Execution)
        }
    })
}
