use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Fail to initialize git repo")]
    InitializeFailure,
    #[error("Fail to execute command")]
    CommandFailure(#[from] ws_std::error::CommandError),
    #[error("Failure from IO entry")]
    IoFailure(#[from] std::io::Error),
    #[error("Fail to config git user name")]
    ConfigUsernameFailure,
    #[error("Fail to config git user email")]
    ConfigEmailFailure,
    #[error("Fail to create branch")]
    BranchCreationFailure,
    #[error("Fail to checkout branch")]
    BranchCheckoutFailure,
    #[error("Fail to merge branch")]
    BranchMergeFailure,
    #[error("Fail to add all files")]
    AddAllFailure,
}
