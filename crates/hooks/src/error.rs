use thiserror::Error;

use sublime_git_tools::RepoError;
use sublime_monorepo_tools::ChangeError;
use sublime_package_tools::VersionError;

/// Errors that can occur during hook operations.
#[derive(Error, Debug)]
pub enum HookError {
    /// Error when interacting with git
    #[error("Git error: {0}")]
    Git(#[from] RepoError),
    
    /// Error when managing changes
    #[error("Change error: {0}")]
    Change(#[from] ChangeError),
    
    /// Error when managing versions
    #[error("Version error: {0}")]
    Version(#[from] VersionError),
    
    /// Hook specific errors
    #[error("Hook error: {0}")]
    Hook(String),
}

impl AsRef<str> for HookError {
    fn as_ref(&self) -> &str {
        match self {
            HookError::Git(_) => "HookErrorGit",
            HookError::Change(_) => "HookErrorChange",
            HookError::Version(_) => "HookErrorVersion",
            HookError::Hook(_) => "HookErrorHook",
        }
    }
}

pub type HookResult<T> = Result<T, HookError>; 