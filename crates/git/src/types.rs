use git2::{Error as Git2Error, Repository};
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

/// Represents a Git repository with high-level operation methods
///
/// This struct wraps the libgit2 `Repository` type and provides simplified methods
/// for common Git operations.
///
/// # Examples
///
/// ```
/// use git::repo::Repo;
///
/// // Open an existing repository
/// let repo = Repo::open("./my-repo").expect("Failed to open repository");
///
/// // Create a new branch
/// repo.create_branch("feature-branch").expect("Failed to create branch");
///
/// // Make changes and commit them
/// repo.add_all().expect("Failed to stage changes");
/// let commit_id = repo.commit("feat: add new feature").expect("Failed to commit");
/// ```
#[derive(Clone)]
pub struct Repo {
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) repo: Arc<Repository>,
    pub(crate) local_path: PathBuf,
}

/// Represents the status of a file in Git
///
/// # Examples
///
/// ```
/// use git::repo::GitFileStatus;
///
/// let status = GitFileStatus::Modified;
/// match status {
///     GitFileStatus::Added => println!("File was added"),
///     GitFileStatus::Modified => println!("File was modified"),
///     GitFileStatus::Deleted => println!("File was deleted"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GitFileStatus {
    /// File has been added to the repository
    Added,
    /// File has been modified
    Modified,
    /// File has been deleted
    Deleted,
}

/// Represents a changed file in the Git repository
///
/// # Examples
///
/// ```
/// use git::repo::{GitChangedFile, GitFileStatus};
///
/// let file = GitChangedFile {
///     path: "src/main.rs".to_string(),
///     status: GitFileStatus::Modified,
/// };
///
/// println!("Changed file: {} ({})", file.path,
///     match file.status {
///         GitFileStatus::Added => "added",
///         GitFileStatus::Modified => "modified",
///         GitFileStatus::Deleted => "deleted",
///     }
/// );
/// ```
#[derive(Debug, Clone)]
pub struct GitChangedFile {
    /// The path to the changed file
    pub path: String,
    /// The status of the file (Added, Modified, or Deleted)
    pub status: GitFileStatus,
}

/// Represents a commit in the Git repository
///
/// # Examples
///
/// ```
/// use git::repo::RepoCommit;
///
/// let commit = RepoCommit {
///     hash: "abcdef123456".to_string(),
///     author_name: "John Doe".to_string(),
///     author_email: "john@example.com".to_string(),
///     author_date: "Wed, 01 Jan 2023 12:00:00 +0000".to_string(),
///     message: "feat: add new feature".to_string(),
/// };
///
/// println!("{}: {} ({})", commit.hash, commit.message, commit.author_name);
/// ```
#[derive(Debug, Clone)]
pub struct RepoCommit {
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

/// Represents a tag in the Git repository
///
/// # Examples
///
/// ```
/// use git::repo::RepoTags;
///
/// let tag = RepoTags {
///     hash: "abcdef123456".to_string(),
///     tag: "v1.0.0".to_string(),
/// };
///
/// println!("Tag {} points to commit {}", tag.tag, tag.hash);
/// ```
#[derive(Debug, Clone)]
pub struct RepoTags {
    /// The hash of the commit that the tag points to
    pub hash: String,
    /// The name of the tag
    pub tag: String,
}

/// Errors that can occur when working with Git repositories
///
/// This enum represents all possible errors that can occur when using the `Repo` struct.
/// Each variant provides context about what operation failed and includes the underlying error.
///
/// # Examples
///
/// ```
/// use git::repo::{Repo, RepoError};
///
/// match Repo::open("/non/existent/path") {
///     Ok(_) => println!("Repository opened successfully"),
///     Err(e) => match e {
///         RepoError::OpenRepoFailure(git_err) => println!("Failed to open repo: {}", git_err),
///         _ => println!("Other error: {}", e),
///     },
/// }
/// ```
#[derive(Error, Debug)]
pub enum RepoError {
    /// Failed to canonicalize a path
    #[error("Failed to canonicalize path: {0}")]
    CanonicalPathFailure(#[source] std::io::Error),

    /// Generic Git operation failure
    #[error("Failed to execute git: {0}")]
    GitFailure(#[source] Git2Error),

    /// Failed to create a new repository
    #[error("Failed to create repository: {0}")]
    CreateRepoFailure(#[source] Git2Error),

    /// Failed to open an existing repository
    #[error("Failed to open repository: {0}")]
    OpenRepoFailure(#[source] Git2Error),

    /// Failed to clone a repository
    #[error("Failed to clone repository: {0}")]
    CloneRepoFailure(#[source] Git2Error),

    /// Git configuration error
    #[error("Git configuration error: {0}")]
    ConfigError(#[source] Git2Error),

    /// Failed to retrieve configuration entries
    #[error("Failed to get repository configuration entries: {0}")]
    ConfigEntriesError(#[source] Git2Error),

    /// Failed to get repository HEAD
    #[error("Failed to get repository head: {0}")]
    HeadError(#[source] Git2Error),

    /// Failed to peel a reference to a commit
    #[error("Failed to peel to commit: {0}")]
    PeelError(#[source] Git2Error),

    /// Failed to create or manipulate a branch
    #[error("Failed to create branch: {0}")]
    BranchError(#[source] Git2Error),

    /// Failed to get repository signature
    #[error("Failed to get repository signature: {0}")]
    SignatureError(#[source] Git2Error),

    /// Failed to get or manipulate the index
    #[error("Failed to map index: {0}")]
    IndexError(#[source] Git2Error),

    /// Failed to add files to the index
    #[error("Failed to add files to index: {0}")]
    AddFilesError(#[source] Git2Error),

    /// Failed to write the index
    #[error("Failed to write index: {0}")]
    WriteIndexError(#[source] Git2Error),

    /// Failed to find or manipulate a tree
    #[error("Failed to find tree: {0}")]
    TreeError(#[source] Git2Error),

    /// Failed to create a commit
    #[error("Failed to commit: {0}")]
    CommitError(#[source] Git2Error),

    /// Failed to write a tree
    #[error("Failed to write tree: {0}")]
    WriteTreeError(#[source] Git2Error),

    /// Failed to list branches
    #[error("Failed to list branches: {0}")]
    BranchListError(#[source] Git2Error),

    /// Failed to get a branch name
    #[error("Failed to get branch name: {0}")]
    BranchNameError(#[source] Git2Error),

    /// Failed to checkout a branch
    #[error("Failed to checkout branch: {0}")]
    CheckoutBranchError(#[source] Git2Error),

    /// Failed to checkout
    #[error("Failed to checkout: {0}")]
    CheckoutError(#[source] Git2Error),

    /// Failed to get the last tag
    #[error("Failed to get last tag: {0}")]
    LastTagError(#[source] Git2Error),

    /// Failed to create a tag
    #[error("Failed to create tag: {0}")]
    CreateTagError(#[source] Git2Error),

    /// Failed to get repository status
    #[error("Failed to get status: {0}")]
    StatusError(#[source] Git2Error),

    /// Failed to parse a commit SHA
    #[error("Failed to parse commit sha: {0}")]
    CommitOidError(#[source] Git2Error),

    /// Failed on repository graph operations
    #[error("Failed on graph: {0}")]
    GraphError(#[source] Git2Error),

    /// Failed to push to a remote
    #[error("Failed to push to remote: {0}")]
    PushError(#[source] Git2Error),

    /// Failed on remote operations
    #[error("Failed on remote: {0}")]
    RemoteError(#[source] Git2Error),

    /// Failed on reference parsing
    #[error("Failed on revparse: {0}")]
    ReferenceError(#[source] Git2Error),

    /// Failed on diff operations
    #[error("Failed on diff: {0}")]
    DiffError(#[source] Git2Error),

    /// Failed on revision walking
    #[error("Failed on revwalk: {0}")]
    RevWalkError(#[source] Git2Error),

    /// Failed on tag operations
    #[error("Failed on tag: {0}")]
    TagError(#[source] Git2Error),

    /// Failed on merge operations
    #[error("Failed on merge: {0}")]
    MergeError(#[source] Git2Error),

    /// Failed due to merge conflicts
    #[error("Failed on merge conflict: {0}")]
    MergeConflictError(#[source] Git2Error),
}
