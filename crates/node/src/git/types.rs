use napi::{Error, Status};

use std::cell::RefCell;
use std::rc::Rc;
use sublime_git_tools::{Repo, RepoError};

#[allow(dead_code)]
#[napi(js_name = "MonorepoRepository")]
pub struct MonorepoRepository {
    pub(crate) repo_instance: Rc<RefCell<Repo>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MonorepoRepositoryError {
    /// Error originating from the Node-API layer
    NapiError(Error<Status>),
    /// Error originating from the git repository
    RepoError(RepoError),
}
