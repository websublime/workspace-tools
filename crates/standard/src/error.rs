use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Fail to run command")]
    Run(#[from] std::io::Error),
    #[error("Fail to execute command")]
    Execution,
    #[error("command failed with the following stdout: {stdout} stderr: {stderr}")]
    Failure { stdout: String, stderr: String },
}
