use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Fail to initialize git repo")]
    InitializeFailure,
    #[error("Fail to execute command")]
    CommandFailure(#[from] ws_std::error::CommandError),
    #[error("Failure from IO entry")]
    IoFailure(#[from] std::io::Error),
}
