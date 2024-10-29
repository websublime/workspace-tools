use serde::{Deserialize, Serialize};
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};
use ws_std::command::execute;

use super::error::RepositoryError;

impl From<&str> for Repository {
    fn from(root: &str) -> Self {
        let path_buff = PathBuf::from(root);
        let repo_path = &canonicalize(Path::new(path_buff.as_os_str())).expect("Invalid path");

        Repository { location: repo_path.clone() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    location: PathBuf,
}

impl Repository {
    pub fn new(location: &Path) -> Self {
        let root = canonicalize(location.as_os_str()).expect("Invalid path");
        Self { location: root }
    }

    pub fn get_repo_path(&self) -> &Path {
        &self.location
    }

    pub fn init(&self, initial_branch: &str) -> Result<bool, RepositoryError> {
        let inited = execute(
            "git",
            self.location.as_path(),
            ["init", "--initial-branch", initial_branch],
            |_, output| Ok(output.status.success()),
        )?;

        if !inited {
            return Err(RepositoryError::InitializeFailure);
        }

        Ok(inited)
    }
}
