use std::ops::Deref;

use napi::bindgen_prelude::*;
use sublime_git_tools::{Repo, RepoError};

#[napi]
#[allow(dead_code)]
pub struct GitRepository {
    pub(crate) inner: Repo,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum JsGitRepositoryError {
    /// Error originating from the Node-API layer
    NapiError(Error),
    /// Error originating from the git repository
    RepoError(RepoError),
}

impl From<RepoError> for JsGitRepositoryError {
    fn from(err: RepoError) -> Self {
        JsGitRepositoryError::RepoError(err)
    }
}

impl AsRef<str> for JsGitRepositoryError {
    fn as_ref(&self) -> &str {
        match self {
            JsGitRepositoryError::NapiError(e) => e.status.as_ref(),
            JsGitRepositoryError::RepoError(e) => e.as_ref(),
        }
    }
}

fn repo_format_napi_error(err: RepoError) -> Error<JsGitRepositoryError> {
    Error::new(err.clone().into(), err.to_string())
}

#[napi]
impl GitRepository {
    #[napi(factory)]
    pub fn open(root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner = Repo::open(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    #[napi(factory)]
    pub fn create(root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner = Repo::create(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    #[napi(factory)]
    pub fn clone(url: String, root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner =
            Repo::clone(url.as_str(), root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.get_repo_path().display().to_string()
    }

    #[napi]
    pub fn set_config(&self, username: String, email: String) -> Result<(), JsGitRepositoryError> {
        self.inner.config(username.as_str(), email.as_str()).map_err(repo_format_napi_error)?;
        Ok(())
    }
}
