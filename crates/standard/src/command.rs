use std::{
    ffi::OsStr,
    fs::canonicalize,
    path::Path,
    process::{Command, Output, Stdio},
    str::from_utf8,
};

use super::error::CommandError;
use super::utils::strip_trailing_newline;

pub type ComandResult<T> = Result<T, CommandError>;

pub fn execute<P, I, F, S, R>(cmd: S, path: P, args: I, process: F) -> Result<R, CommandError>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: Fn(&str, &Output) -> ComandResult<R>,
{
    #[cfg(not(windows))]
    let canonic_path = &canonicalize(&path)?;
    #[cfg(windows)]
    let canonic_path = path;
    let output = Command::new(cmd)
        .current_dir(canonic_path.as_path())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    output.wait_with_output().map_err(CommandError::Run).and_then(|output| {
        if output.status.success() {
            if let Ok(message) = from_utf8(&output.stdout) {
                process(strip_trailing_newline(&message.to_string()).as_str(), &output)
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
