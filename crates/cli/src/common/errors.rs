use thiserror::Error;
use std::path::PathBuf;

/// CLI-specific error types
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Failed to read configuration: {0}")]
    ConfigError(String),

    #[error("Failed to find workspace root: {0}")]
    WorkspaceError(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Daemon error: {0}")]
    DaemonError(String),

    #[error("Subcommand not found: {0}")]
    SubcommandNotFound(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Operation not permitted: {0}")]
    PermissionError(String),
}
