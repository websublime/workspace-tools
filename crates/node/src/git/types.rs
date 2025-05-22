use napi::{Error, Status};

use std::cell::RefCell;
use std::rc::Rc;
use sublime_git_tools::{Repo, RepoError};

#[allow(dead_code)]
#[napi(js_name = "MonorepoRepository")]
pub struct MonorepoRepository {
    pub(crate) repo_instance: Rc<RefCell<Repo>>,
}

/// Represents the status of a file in a Git repository.
/// - Added: File has been added to the repository
/// - Deleted: File has been deleted from the repository
/// - Modified: File has been modified
#[napi]
pub enum GitFileStatus {
    Added,
    Deleted,
    Modified,
}

/// Represents a changed file in a Git repository.
/// Contains information about the file path and its status.
#[napi]
pub struct GitChangedFile {
    /// The path to the changed file
    pub path: String,
    /// The status of the file (Added, Deleted, or Modified)
    pub status: GitFileStatus,
}

/// Represents a commit in a Git repository.
/// Contains detailed information about the commit, including
/// author details, date, and commit message.
#[napi]
pub struct GitCommit {
    /// The commit hash (SHA)
    pub hash: String,
    /// The name of the commit author
    pub author_name: String,
    /// The email of the commit author
    pub author_email: String,
    /// The date of the commit in RFC2822 format
    pub author_date: String,
    /// The commit message
    pub message: String,
}

/// Represents a tag in a Git repository.
/// Contains the tag name and the commit hash it points to.
#[napi]
pub struct GitTag {
    /// The hash of the commit that the tag points to
    pub hash: String,
    /// The name of the tag
    pub tag: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MonorepoRepositoryError {
    /// Error originating from the Node-API layer
    NapiError(Error<Status>),
    /// Error originating from the git repository
    RepoError(RepoError),
}
