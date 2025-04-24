use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Daemon error: {0}")]
    Daemon(String),
    
    #[error("IPC error: {0}")]
    Ipc(String),
    
    #[error("Workspace error: {0}")]
    Workspace(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type CliResult<T> = Result<T, CliError>;

// Common result extension traits
pub trait ResultExt<T> {
    fn with_context<S: AsRef<str>>(self, context: S) -> CliResult<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn with_context<S: AsRef<str>>(self, context: S) -> CliResult<T> {
        self.map_err(|e| {
            CliError::Other(anyhow::Error::new(e).context(context.as_ref().to_string()))
        })
    }
}

