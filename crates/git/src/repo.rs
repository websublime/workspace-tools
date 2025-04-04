use git2::{
    build::CheckoutBuilder, BranchType, Commit, Cred, CredentialType, Delta, DiffOptions,
    Direction, Error as Git2Error, FetchOptions, FetchPrune, IndexAddOption, MergeOptions, Oid,
    PushOptions, RemoteCallbacks, Repository, RepositoryInitOptions, StatusOptions, TreeWalkMode,
    TreeWalkResult,
};
use std::collections::HashMap;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

/// Canonicalizes a path string to its absolute form
///
/// # Arguments
///
/// * `path` - The path to canonicalize
///
/// # Returns
///
/// * `Result<String, RepoError>` - The canonicalized path as a string, or an error
///
/// # Examples
///
/// ```
/// use git::repo::canonicalize_path;
///
/// let path = canonicalize_path("./src").expect("Failed to canonicalize path");
/// println!("Canonical path: {}", path);
/// ```
fn canonicalize_path(path: &str) -> Result<String, RepoError> {
    let location = PathBuf::from(path);
    let path = canonicalize(location.as_os_str()).map_err(RepoError::CanonicPathFailure)?;
    Ok(path.display().to_string())
}

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
    repo: Rc<Repository>,
    local_path: PathBuf,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    CanonicPathFailure(#[source] std::io::Error),

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

impl From<Git2Error> for RepoError {
    fn from(err: Git2Error) -> Self {
        // You might want to match on error code to create specific errors
        // For simplicity, we'll use a default case here
        RepoError::GitFailure(err)
    }
}

#[allow(clippy::too_many_lines)]
impl Clone for RepoError {
    fn clone(&self) -> Self {
        match self {
            RepoError::CanonicPathFailure(_) => {
                // We'll create a new IO error with the same message
                let io_err = std::io::Error::new(std::io::ErrorKind::Other, format!("{self}"));
                RepoError::CanonicPathFailure(io_err)
            }
            RepoError::GitFailure(_) => {
                // Create a new Git2Error with the same message
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::GitFailure(git_err)
            }
            RepoError::CreateRepoFailure(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CreateRepoFailure(git_err)
            }
            RepoError::OpenRepoFailure(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::OpenRepoFailure(git_err)
            }
            RepoError::CloneRepoFailure(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CloneRepoFailure(git_err)
            }
            RepoError::ConfigError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::ConfigError(git_err)
            }
            RepoError::ConfigEntriesError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::ConfigEntriesError(git_err)
            }
            RepoError::HeadError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::HeadError(git_err)
            }
            RepoError::PeelError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::PeelError(git_err)
            }
            RepoError::BranchError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::BranchError(git_err)
            }
            RepoError::SignatureError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::SignatureError(git_err)
            }
            RepoError::IndexError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::IndexError(git_err)
            }
            RepoError::AddFilesError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::AddFilesError(git_err)
            }
            RepoError::WriteIndexError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::WriteIndexError(git_err)
            }
            RepoError::TreeError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::TreeError(git_err)
            }
            RepoError::CommitError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CommitError(git_err)
            }
            RepoError::WriteTreeError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::WriteTreeError(git_err)
            }
            RepoError::BranchListError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::BranchListError(git_err)
            }
            RepoError::BranchNameError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::BranchNameError(git_err)
            }
            RepoError::CheckoutBranchError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CheckoutBranchError(git_err)
            }
            RepoError::LastTagError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::LastTagError(git_err)
            }
            RepoError::CreateTagError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CreateTagError(git_err)
            }
            RepoError::StatusError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::StatusError(git_err)
            }
            RepoError::CommitOidError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CommitOidError(git_err)
            }
            RepoError::GraphError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::GraphError(git_err)
            }
            RepoError::PushError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::PushError(git_err)
            }
            RepoError::RemoteError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::RemoteError(git_err)
            }
            RepoError::ReferenceError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::ReferenceError(git_err)
            }
            RepoError::DiffError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::DiffError(git_err)
            }
            RepoError::RevWalkError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::RevWalkError(git_err)
            }
            RepoError::TagError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::TagError(git_err)
            }
            RepoError::MergeError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::MergeError(git_err)
            }
            RepoError::CheckoutError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::CheckoutError(git_err)
            }
            RepoError::MergeConflictError(_) => {
                let git_err = Git2Error::from_str(&format!("{self}"));
                RepoError::MergeConflictError(git_err)
            }
        }
    }
}

impl AsRef<str> for RepoError {
    fn as_ref(&self) -> &str {
        match self {
            RepoError::CreateRepoFailure(_) => "CreateRepoFailure",
            RepoError::OpenRepoFailure(_) => "OpenRepoFailure",
            RepoError::CloneRepoFailure(_) => "CloneRepoFailure",
            RepoError::ConfigError(_) => "ConfigError",
            RepoError::ConfigEntriesError(_) => "ConfigEntriesError",
            RepoError::HeadError(_) => "HeadError",
            RepoError::PeelError(_) => "PeelError",
            RepoError::BranchError(_) => "BranchError",
            RepoError::GitFailure(_) => "GitFailure",
            RepoError::CanonicPathFailure(_) => "CanonicPathFailure",
            RepoError::SignatureError(_) => "SignatureError",
            RepoError::IndexError(_) => "IndexError",
            RepoError::AddFilesError(_) => "AddFilesError",
            RepoError::WriteIndexError(_) => "WriteIndexError",
            RepoError::TreeError(_) => "TreeError",
            RepoError::CommitError(_) => "CommitError",
            RepoError::WriteTreeError(_) => "WriteTreeError",
            RepoError::BranchListError(_) => "BranchListError",
            RepoError::BranchNameError(_) => "BranchNameError",
            RepoError::CheckoutBranchError(_) => "CheckoutBranchError",
            RepoError::LastTagError(_) => "LastTagError",
            RepoError::CreateTagError(_) => "CreateTagError",
            RepoError::StatusError(_) => "StatusError",
            RepoError::CommitOidError(_) => "CommitOidError",
            RepoError::GraphError(_) => "GraphError",
            RepoError::PushError(_) => "PushError",
            RepoError::RemoteError(_) => "RemoteError",
            RepoError::ReferenceError(_) => "ReferenceError",
            RepoError::DiffError(_) => "DiffError",
            RepoError::RevWalkError(_) => "RevWalkError",
            RepoError::TagError(_) => "TagError",
            RepoError::MergeError(_) => "MergeError",
            RepoError::CheckoutError(_) => "CheckoutError",
            RepoError::MergeConflictError(_) => "MergeConflictError",
        }
    }
}

// Implementation of Debug that avoids using the Repository field since it doesn't implement Debug
impl std::fmt::Debug for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repo")
            .field("local_path", &self.local_path)
            // Skip printing the repo field as Repository doesn't implement Debug
            .finish_non_exhaustive()
    }
}

impl Repo {
    /// Creates a new Git repository at the specified path
    ///
    /// This initializes a new Git repository with an initial commit on the 'main' branch.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the repository should be created
    ///
    /// # Returns
    ///
    /// * `Result<Self, RepoError>` - A new `Repo` instance or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::create("/tmp/new-repo").expect("Failed to create repository");
    /// println!("Repository created at: {}", repo.get_repo_path().display());
    /// ```
    pub fn create(path: &str) -> Result<Self, RepoError> {
        let location = canonicalize_path(path)?;
        let location_buf = PathBuf::from(location);

        // Initialize the repository
        let repo = Repository::init_opts(
            location_buf.as_path(),
            RepositoryInitOptions::new().initial_head("main"),
        )
        .map_err(RepoError::CreateRepoFailure)?;

        // Just return the repo without making any commits
        let result = Self { repo: Rc::new(repo), local_path: location_buf };

        // Now make the initial commit using our new instance
        result.make_initial_commit()?;

        Ok(result)
    }

    /// Opens an existing Git repository at the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the existing repository
    ///
    /// # Returns
    ///
    /// * `Result<Self, RepoError>` - A `Repo` instance or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-project").expect("Failed to open repository");
    /// let branch = repo.get_current_branch().expect("Failed to get current branch");
    /// println!("Current branch: {}", branch);
    /// ```
    pub fn open(path: &str) -> Result<Self, RepoError> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::open(path).map_err(RepoError::OpenRepoFailure)?;

        Ok(Self { repo: Rc::new(repo), local_path: PathBuf::from(local_path) })
    }

    /// Clones a Git repository from a URL to a local path
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the repository to clone
    /// * `path` - The local path where the repository should be cloned
    ///
    /// # Returns
    ///
    /// * `Result<Self, RepoError>` - A `Repo` instance or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::clone("https://github.com/example/repo.git", "./cloned-repo")
    ///     .expect("Failed to clone repository");
    /// ```
    pub fn clone(url: &str, path: &str) -> Result<Self, RepoError> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::clone(url, path).map_err(RepoError::CloneRepoFailure)?;

        Ok(Self { repo: Rc::new(repo), local_path: PathBuf::from(local_path) })
    }

    /// Gets the local path of the repository
    ///
    /// # Returns
    ///
    /// * `&Path` - The path to the repository
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// println!("Repository path: {}", repo.get_repo_path().display());
    /// ```
    pub fn get_repo_path(&self) -> &Path {
        self.local_path.as_path()
    }

    /// Configures the repository with user information and core settings
    ///
    /// # Arguments
    ///
    /// * `username` - The Git user name
    /// * `email` - The Git user email
    ///
    /// # Returns
    ///
    /// * `Result<&Self, RepoError>` - A reference to self for method chaining, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.config("Jane Doe", "jane@example.com").expect("Failed to configure repository");
    /// ```
    pub fn config(&self, username: &str, email: &str) -> Result<&Self, RepoError> {
        let mut config = self.repo.config().map_err(RepoError::ConfigError)?;
        config.set_str("user.name", username)?;
        config.set_str("user.email", email)?;
        config.set_bool("core.safecrlf", true)?;
        config.set_str("core.autocrlf", "input")?;
        config.set_bool("core.filemode", false)?;
        Ok(self)
    }

    /// Creates a new branch based on the current HEAD
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name for the new branch
    ///
    /// # Returns
    ///
    /// * `Result<&Self, RepoError>` - A reference to self for method chaining, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.create_branch("feature/new-feature").expect("Failed to create branch");
    /// ```
    pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        self.repo.branch(branch_name, &commit, false).map_err(RepoError::BranchError)?;
        Ok(self)
    }

    /// Lists all local branches in the repository
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, RepoError>` - A list of branch names, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let branches = repo.list_branches().expect("Failed to list branches");
    /// for branch in branches {
    ///     println!("Branch: {}", branch);
    /// }
    /// ```
    pub fn list_branches(&self) -> Result<Vec<String>, RepoError> {
        let branches = self
            .repo
            .branches(Some(git2::BranchType::Local))
            .map_err(RepoError::BranchListError)?;
        let mut branch_names = Vec::new();

        for branch in branches {
            let (branch, _branch_type) = branch?;
            let branch_name = branch.name().map_err(RepoError::BranchNameError)?;

            if let Some(name) = branch_name {
                branch_names.push(name.to_string());
            }
        }

        Ok(branch_names)
    }

    /// Lists all configuration entries for the repository
    ///
    /// # Returns
    ///
    /// * `Result<HashMap<String, String>, RepoError>` - A map of config keys to values, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let config = repo.list_config().expect("Failed to list config");
    /// for (key, value) in config {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn list_config(&self) -> Result<HashMap<String, String>, RepoError> {
        let config = self.repo.config().map_err(RepoError::ConfigError)?;
        let mut config_map = HashMap::new();

        let mut entries = config.entries(None).map_err(RepoError::ConfigEntriesError)?;
        while let Some(entry_result) = entries.next() {
            if let Ok(entry) = entry_result {
                if let Some(name) = entry.name() {
                    // Try to get the value as a string
                    if let Ok(value) = config.get_string(name) {
                        config_map.insert(name.to_string(), value);
                    }
                }
            }
        }

        Ok(config_map)
    }

    /// Checks out a local branch
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch to checkout
    ///
    /// # Returns
    ///
    /// * `Result<&Self, RepoError>` - A reference to self for method chaining, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.checkout("feature-branch").expect("Failed to checkout branch");
    /// ```
    pub fn checkout(&self, branch_name: &str) -> Result<&Self, RepoError> {
        let branch = self
            .repo
            .find_branch(branch_name, BranchType::Local)
            .map_err(RepoError::CheckoutBranchError)?;

        // Get the reference name from the branch
        let branch_ref = branch.get().name().ok_or_else(|| {
            RepoError::BranchNameError(Git2Error::from_str(
                format!("Invalid branch reference: {branch_name}").as_str(),
            ))
        })?;

        // Set head to the reference name
        self.repo.set_head(branch_ref).map_err(RepoError::CheckoutBranchError)?;

        Ok(self)
    }

    /// Gets the name of the currently checked out branch
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The current branch name, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let branch = repo.get_current_branch().expect("Failed to get current branch");
    /// println!("Current branch: {}", branch);
    /// ```
    pub fn get_current_branch(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let branch = head.shorthand().ok_or_else(|| {
            RepoError::BranchNameError(Git2Error::from_str("Invalid branch reference"))
        })?;

        Ok(branch.to_string())
    }

    /// Creates a new tag at the current HEAD
    ///
    /// # Arguments
    ///
    /// * `tag` - The name for the new tag
    /// * `message` - Optional message for the tag
    ///
    /// # Returns
    ///
    /// * `Result<(), RepoError>` - Success or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.create_tag("v1.0.0", Some("Version 1.0.0 release".to_string()))
    ///     .expect("Failed to create tag");
    /// ```
    pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<&Self, RepoError> {
        let signature = self.repo.signature().map_err(RepoError::SignatureError)?;
        let tag_message = match message {
            Some(msg) => msg,
            None => format!("chore: tag creation: {tag}"),
        };

        // Get the HEAD reference
        let head = self.repo.head().map_err(RepoError::HeadError)?;

        // Get the target OID (object ID) from the reference
        let target_oid = head
            .target()
            .ok_or_else(|| RepoError::CreateTagError(Git2Error::from_str("Invalid tag target")))?;

        // Look up the object from the OID
        let target_object =
            self.repo.find_object(target_oid, None).map_err(RepoError::CreateTagError)?;

        self.repo
            .tag(tag, &target_object, &signature, &tag_message, false)
            .map_err(RepoError::CreateTagError)?;

        Ok(self)
    }

    /// Adds a file to the Git index
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file to add
    ///
    /// # Returns
    ///
    /// * `Result<&Self, RepoError>` - A reference to self for method chaining, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.add("src/main.rs").expect("Failed to add file");
    /// ```
    pub fn add(&self, file_path: &str) -> Result<&Self, RepoError> {
        let mut index = self.repo.index().map_err(RepoError::IndexError)?;
        let path = Path::new(file_path);
        // get the relative path of the file_path
        let relative_path = path.strip_prefix(self.local_path.as_path()).unwrap_or(path);
        // Add the file to the index
        index.add_path(relative_path).map_err(RepoError::IndexError)?;

        // Write the index to disk
        index.write().map_err(RepoError::IndexError)?;

        Ok(self)
    }

    /// Adds all changed files to the Git index
    ///
    /// # Returns
    ///
    /// * `Result<&Self, RepoError>` - A reference to self for method chaining, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// repo.add_all().expect("Failed to add all changes");
    /// ```
    pub fn add_all(&self) -> Result<&Self, RepoError> {
        let mut index = self.repo.index().map_err(RepoError::IndexError)?;
        // Add all files to the index
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(RepoError::IndexError)?;

        // Write the index to disk
        index.write().map_err(RepoError::IndexError)?;

        Ok(self)
    }

    /// Gets the name of the last tag in the repository
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The last tag name, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// match repo.get_last_tag() {
    ///     Ok(tag) => println!("Last tag: {}", tag),
    ///     Err(e) => println!("No tags found or error: {}", e),
    /// }
    /// ```
    pub fn get_last_tag(&self) -> Result<String, RepoError> {
        let tags = self.repo.tag_names(None).map_err(RepoError::LastTagError)?;

        let last_tag = tags.iter().flatten().max_by_key(|&tag| tag.parse::<u64>().unwrap_or(0));

        last_tag
            .map(std::string::ToString::to_string)
            .ok_or_else(|| RepoError::LastTagError(Git2Error::from_str("No tags found")))
    }

    /// Gets the SHA of the current HEAD commit
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The current commit SHA, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let sha = repo.get_current_sha().expect("Failed to get current SHA");
    /// println!("Current commit: {}", sha);
    /// ```
    pub fn get_current_sha(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let target = head
            .target()
            .ok_or_else(|| RepoError::HeadError(Git2Error::from_str("No target found")));
        let sha = target.map_err(|_| RepoError::HeadError(Git2Error::from_str("No OID found")))?;

        Ok(sha.to_string())
    }

    /// Gets the SHA of the parent of the current HEAD commit
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The previous commit SHA, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let prev_sha = repo.get_previous_sha().expect("Failed to get previous SHA");
    /// println!("Previous commit: {}", prev_sha);
    /// ```
    pub fn get_previous_sha(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        // Check if this commit has parents (the initial commit won't have any)
        if head_commit.parent_count() == 0 {
            // Return the current commit SHA if there's no parent
            return Ok(head_commit.id().to_string());
        }

        // Get the parent commit (the previous commit)
        let parent = head_commit.parent(0).map_err(|e| {
            RepoError::GitFailure(Git2Error::from_str(&format!("Failed to get parent commit: {e}")))
        })?;

        // Get the SHA of the parent commit
        let previous_sha = parent.id().to_string();

        Ok(previous_sha)
    }

    /// Creates a new commit with the current index
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The new commit's SHA, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// // First add some files
    /// repo.add("src/main.rs").expect("Failed to add file");
    /// // Then commit
    /// let commit_id = repo.commit("fix: update main.rs").expect("Failed to commit");
    /// println!("Created commit: {}", commit_id);
    /// ```
    pub fn commit(&self, message: &str) -> Result<String, RepoError> {
        let signature = self.repo.signature().map_err(RepoError::SignatureError)?;
        let head_ref = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head_ref.peel_to_commit().map_err(RepoError::PeelError)?;

        let tree_id = {
            let mut index = self.repo.index().map_err(RepoError::IndexError)?;
            index.write_tree().map_err(RepoError::WriteTreeError)?
        };

        let tree = self.repo.find_tree(tree_id).map_err(RepoError::TreeError)?;

        let commit_id = self
            .repo
            .commit(Some("HEAD"), &signature, &signature, message, &tree, &[&head_commit])
            .map_err(RepoError::CommitError)?;

        Ok(commit_id.to_string())
    }

    /// Adds all changes and creates a new commit
    ///
    /// This method performs both add_all() and commit() in one step.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The new commit's SHA, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let commit_id = repo.commit_changes("feat: add new feature").expect("Failed to commit changes");
    /// println!("Created commit: {}", commit_id);
    /// ```
    pub fn commit_changes(&self, message: &str) -> Result<String, RepoError> {
        let signature = self.repo.signature().map_err(RepoError::SignatureError)?;
        let head_ref = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head_ref.peel_to_commit().map_err(RepoError::PeelError)?;

        let tree_id = {
            let mut index = self.repo.index().map_err(RepoError::IndexError)?;
            index
                .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
                .map_err(RepoError::AddFilesError)?;
            index.write_tree().map_err(RepoError::WriteTreeError)?
        };

        let tree = self.repo.find_tree(tree_id).map_err(RepoError::TreeError)?;

        let commit_id = self
            .repo
            .commit(Some("HEAD"), &signature, &signature, message, &tree, &[&head_commit])
            .map_err(RepoError::CommitError)?;

        Ok(commit_id.to_string())
    }

    /// Gets the status of the repository in porcelain format
    ///
    /// Returns a list of changed file paths.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, RepoError>` - List of changed file paths, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let status = repo.status_porcelain().expect("Failed to get status");
    /// for file in status {
    ///     println!("Changed file: {}", file);
    /// }
    /// ```
    pub fn status_porcelain(&self) -> Result<Vec<String>, RepoError> {
        let mut status_options = StatusOptions::new();
        status_options
            .include_untracked(true)
            .include_ignored(false)
            .include_unmodified(false)
            .recurse_untracked_dirs(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        let statuses =
            self.repo.statuses(Some(&mut status_options)).map_err(RepoError::StatusError)?;

        let mut result = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("");

            result.push(path.to_string());
        }

        Ok(result)
    }

    /// Finds the branch that contains a specific commit
    ///
    /// # Arguments
    ///
    /// * `sha` - The commit SHA to find
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, RepoError>` - The branch name if found, None if not found, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let commit_sha = repo.get_current_sha().expect("Failed to get current SHA");
    ///
    /// match repo.get_branch_from_commit(&commit_sha) {
    ///     Ok(Some(branch)) => println!("Commit {} is in branch: {}", commit_sha, branch),
    ///     Ok(None) => println!("Commit {} is not in any branch", commit_sha),
    ///     Err(e) => println!("Error: {}", e),
    /// }
    /// ```
    pub fn get_branch_from_commit(&self, sha: &str) -> Result<Option<String>, RepoError> {
        // Parse the SHA string into an OID (Object ID)
        let oid = Oid::from_str(sha).map_err(RepoError::CommitOidError)?;

        // Find the commit object for this SHA
        let commit = self.repo.find_commit(oid).map_err(RepoError::CommitOidError)?;

        // Get all branches in the repository
        let branches =
            self.repo.branches(Some(BranchType::Local)).map_err(RepoError::BranchListError)?;

        // Iterate through all branches to find which one contains the commit
        for branch_result in branches {
            let (branch, _) = branch_result.map_err(RepoError::BranchListError)?;

            // Get the branch name
            let branch_name = branch
                .name()
                .map_err(RepoError::BranchNameError)?
                .ok_or_else(|| {
                    RepoError::BranchNameError(Git2Error::from_str("Invalid branch name"))
                })?
                .to_string();

            // Get the commit that the branch points to
            let branch_commit = branch.get().peel_to_commit().map_err(RepoError::PeelError)?;

            // Check if our commit is an ancestor of the branch's head commit
            // or if it's the same commit
            if commit.id() == branch_commit.id()
                || self
                    .repo
                    .graph_descendant_of(branch_commit.id(), commit.id())
                    .map_err(RepoError::GraphError)?
            {
                return Ok(Some(branch_name));
            }
        }

        // If we get here, no branch contains this commit
        Ok(None)
    }

    /// Finds all branches that contain a specific commit
    ///
    /// # Arguments
    ///
    /// * `sha` - The commit SHA to find
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, RepoError>` - List of branch names containing the commit, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let commit_sha = repo.get_current_sha().expect("Failed to get current SHA");
    ///
    /// let branches = repo.get_branches_containing_commit(&commit_sha)
    ///     .expect("Failed to find branches");
    /// for branch in branches {
    ///     println!("Branch contains commit: {}", branch);
    /// }
    /// ```
    pub fn get_branches_containing_commit(&self, sha: &str) -> Result<Vec<String>, RepoError> {
        // Parse the SHA string into an OID (Object ID)
        let oid = Oid::from_str(sha).map_err(RepoError::CommitOidError)?;

        // Find the commit object for this SHA
        let commit = self.repo.find_commit(oid).map_err(RepoError::CommitOidError)?;

        // Get all branches in the repository
        let branches =
            self.repo.branches(Some(BranchType::Local)).map_err(RepoError::BranchListError)?;

        let mut containing_branches = Vec::new();

        // Iterate through all branches to find which ones contain the commit
        for branch_result in branches {
            let (branch, _) = branch_result.map_err(RepoError::BranchListError)?;

            // Get the branch name
            let branch_name = branch
                .name()
                .map_err(RepoError::BranchNameError)?
                .ok_or_else(|| {
                    RepoError::BranchNameError(Git2Error::from_str("Invalid branch name"))
                })?
                .to_string();

            // Get the commit that the branch points to
            let branch_commit = branch.get().peel_to_commit().map_err(RepoError::PeelError)?;

            // Check if our commit is an ancestor of the branch's head commit
            // or if it's the same commit
            if commit.id() == branch_commit.id()
                || self
                    .repo
                    .graph_descendant_of(branch_commit.id(), commit.id())
                    .map_err(RepoError::GraphError)?
            {
                containing_branches.push(branch_name);
            }
        }

        Ok(containing_branches)
    }

    /// Merges the specified branch into the current HEAD
    ///
    /// This function attempts to merge the given `branch_name` into the currently
    /// checked out branch. It handles fast-forward merges and normal merges.
    /// If merge conflicts occur, it returns a `MergeConflictError`.
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch to merge into the current branch.
    ///
    /// # Returns
    ///
    /// * `Result<(), RepoError>` - Success or an error, including `MergeConflictError`.
    ///
    /// # Examples
    ///
    /// ```no_run // Example needs a repo setup
    /// use git::repo::Repo;
    ///
    /// # fn example() -> Result<(), git::repo::RepoError> {
    /// let repo = Repo::open("./my-repo")?;
    /// repo.checkout("main")?; // Ensure we are on the target branch
    /// repo.merge("feature-branch")?; // Merge feature-branch into main
    /// println!("Merge successful!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn merge(&self, branch_name: &str) -> Result<(), RepoError> {
        // 1. Get HEAD commit (the branch we are merging INTO)
        let head_ref = self.repo.head().map_err(RepoError::HeadError)?;
        let head_oid = head_ref.target().ok_or_else(|| {
            RepoError::HeadError(Git2Error::from_str("HEAD reference has no target OID"))
        })?;
        let head_commit = self.repo.find_commit(head_oid).map_err(RepoError::PeelError)?;

        // 2. Resolve the branch to merge (the branch we are merging FROM)
        // Use revparse to handle branch names, tags, or commit SHAs flexibly
        let source_object =
            self.repo.revparse_single(branch_name).map_err(RepoError::ReferenceError)?;
        let source_commit = source_object.peel_to_commit().map_err(RepoError::PeelError)?;

        // We need an AnnotatedCommit for merge operations
        let annotated_commit =
            self.repo.find_annotated_commit(source_commit.id()).map_err(RepoError::CommitError)?;

        // 3. Perform Merge Analysis
        let (analysis, preference) =
            self.repo.merge_analysis(&[&annotated_commit]).map_err(RepoError::MergeError)?;

        // 4. Handle Merge Scenarios

        // -- Up-to-date --
        if analysis.is_up_to_date() {
            // Nothing to do
            return Ok(());
        }

        // -- Fast-forward --
        if analysis.is_fast_forward() && preference.is_fastforward_only() {
            // Perform fast-forward
            let target_oid = annotated_commit.id();
            let ref_name = head_ref.name().ok_or_else(|| {
                RepoError::HeadError(Git2Error::from_str("Cannot get HEAD reference name"))
            })?;

            // Update the reference directly
            let mut reference = self.repo.find_reference(ref_name).map_err(RepoError::HeadError)?;
            reference
                .set_target(target_oid, &format!("Fast-forward {branch_name} into HEAD"))
                .map_err(RepoError::ReferenceError)?;

            // Update the working directory to match the new HEAD
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force(); // Use force to sync working dir with index/HEAD
            self.repo
                .checkout_head(Some(&mut checkout_builder))
                .map_err(RepoError::CheckoutError)?;

            return Ok(());
        }

        // -- Normal Merge --
        if analysis.is_normal() {
            // Set up merge options
            let mut merge_opts = MergeOptions::new();
            merge_opts.fail_on_conflict(true); // Fail if conflicts can't be automatically resolved

            // Set up checkout options for the merge (how to handle the working dir)
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder
                .allow_conflicts(true) // Allow checkout to proceed even if conflicts exist
                .conflict_style_merge(true); // Create standard <<< === >>> conflict markers

            // Perform the merge operation (this updates the index)
            self.repo
                .merge(
                    &[&annotated_commit],
                    Some(&mut merge_opts),
                    Some(&mut checkout_builder), // Checkout during merge
                )
                .map_err(RepoError::MergeError)?;

            // Check index for conflicts after merge attempt
            let mut index = self.repo.index().map_err(RepoError::IndexError)?;
            if index.has_conflicts() {
                // Conflicts detected. The working dir and index are in a conflicted state.
                // We *could* try to clean up, but it's often better to let the user resolve.
                // The repo state is MERGING. `cleanup_state` would abort the merge.
                // We return a specific error.
                return Err(RepoError::MergeConflictError(Git2Error::from_str(&format!(
                        "Merge conflict detected when merging '{branch_name}'. Resolve conflicts and commit."
                    ))));
            }

            // --- No Conflicts - Create Merge Commit ---

            // Write the index state to a tree
            let tree_oid = index.write_tree().map_err(RepoError::WriteTreeError)?;
            let tree = self.repo.find_tree(tree_oid).map_err(RepoError::TreeError)?;

            // Get the signature for the committer
            let signature = self.repo.signature().map_err(RepoError::SignatureError)?;

            // Create the merge commit message
            let msg = format!("chore: merge branch '{branch_name}'");

            // Create the merge commit with two parents: HEAD and the merged commit
            self.repo
                .commit(
                    Some("HEAD"),                    // Update the HEAD reference
                    &signature,                      // Author
                    &signature,                      // Committer
                    &msg,                            // Commit message
                    &tree,                           // Tree representing the merged state
                    &[&head_commit, &source_commit], // Parents
                )
                .map_err(RepoError::CommitError)?;

            // Merge successful, clean up the MERGE_HEAD state
            self.repo.cleanup_state().map_err(RepoError::MergeError)?;

            return Ok(());
        }

        // -- Handle other analysis results if necessary (e.g., unborn HEAD) --
        if analysis.is_unborn() {
            return Err(RepoError::MergeError(Git2Error::from_str(
                    "Cannot merge into an unborn HEAD (repository might be empty or branch doesn't exist)",
                )));
        }

        // Default error if none of the above conditions were met (shouldn't usually happen)
        Err(RepoError::MergeError(Git2Error::from_str("Unhandled merge analysis result")))
    }

    /// Pushes the current branch to a remote repository
    ///
    /// # Arguments
    ///
    /// * `remote_name` - The name of the remote (e.g., "origin")
    /// * `follow_tags` - Whether to also push tags
    ///
    /// # Returns
    ///
    /// * `Result<bool, RepoError>` - Success indicator or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// // Push current branch with tags
    /// repo.push("origin", Some(true)).expect("Failed to push");
    /// ```
    pub fn push(&self, remote_name: &str, follow_tags: Option<bool>) -> Result<bool, RepoError> {
        // Get the current branch name
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let branch_name = head.shorthand().ok_or_else(|| {
            RepoError::BranchNameError(Git2Error::from_str("Invalid branch reference"))
        })?;

        // Get the remote
        let mut remote = self.repo.find_remote(remote_name).map_err(RepoError::RemoteError)?;

        // Create callbacks for credentials, progress reporting, etc.
        let mut callbacks = RemoteCallbacks::new();

        // Setup SSH authentication with default key paths
        callbacks.credentials(|url, username_from_url, allowed_types| {
            self.create_ssh_credentials(url, username_from_url, allowed_types, None)
        });

        // Add progress reporting
        callbacks.push_update_reference(|refname, status| {
            if let Some(error_msg) = status {
                return Err(Git2Error::from_str(&format!(
                    "Failed to update {refname} with error: {error_msg}",
                )));
            }
            Ok(())
        });

        // Setup push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Create refspecs
        let mut refspecs = Vec::new();

        // Add the branch refspec (e.g., "refs/heads/main:refs/heads/main")
        let branch_refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
        refspecs.push(branch_refspec);

        // Add tags refspec if follow_tags is enabled
        if follow_tags.unwrap_or(false) {
            // Push all tags
            refspecs.push("refs/tags/*:refs/tags/*".to_string());
        }

        // Perform the push operation
        remote.push(&refspecs, Some(&mut push_options)).map_err(RepoError::RemoteError)?;

        Ok(true)
    }

    /// Fetches changes from a remote repository
    ///
    /// # Arguments
    ///
    /// * `remote_name` - The name of the remote (e.g., "origin")
    /// * `refspecs` - Optional reference specs to fetch
    /// * `prune` - Whether to prune deleted references
    ///
    /// # Returns
    ///
    /// * `Result<bool, RepoError>` - Success indicator or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// // Fetch with default refspecs and no pruning
    /// repo.fetch("origin", None, false).expect("Failed to fetch");
    ///
    /// // Fetch a specific branch and prune
    /// repo.fetch("origin", Some(&["refs/heads/main:refs/remotes/origin/main"]), true)
    ///     .expect("Failed to fetch specific branch");
    /// ```
    pub fn fetch(
        &self,
        remote_name: &str,
        refspecs: Option<&[&str]>,
        prune: bool,
    ) -> Result<bool, RepoError> {
        // Find the remote
        let mut remote = self.repo.find_remote(remote_name).map_err(RepoError::RemoteError)?;

        // Set up fetch options
        let mut fetch_opts = FetchOptions::new();

        // Configure authentication
        let mut callbacks = RemoteCallbacks::new();

        // Setup SSH authentication with default key paths
        callbacks.credentials(|url, username_from_url, allowed_types| {
            self.create_ssh_credentials(url, username_from_url, allowed_types, None)
        });

        // Apply the callbacks
        fetch_opts.remote_callbacks(callbacks);

        // Set prune option if requested
        if prune {
            fetch_opts.prune(FetchPrune::On);
        }

        // Determine refspecs to use
        let refspecs_owned: Vec<String> = if let Some(specs) = refspecs {
            // Convert provided refspecs to owned Strings
            specs.iter().map(|&s| s.to_string()).collect()
        } else {
            // Get default refspecs from remote
            let fetch_refspecs = remote.fetch_refspecs().map_err(RepoError::RemoteError)?;

            if fetch_refspecs.is_empty() {
                // If no default refspecs, use standard one
                vec!["refs/heads/*:refs/remotes/origin/*".to_string()]
            } else {
                // Convert OsString to String
                fetch_refspecs
                    .iter()
                    .filter_map(|s| s.as_ref().map(std::string::ToString::to_string))
                    .collect()
            }
        };

        // If we ended up with no refspecs, use the default
        let refspecs_owned = if refspecs_owned.is_empty() {
            vec!["refs/heads/*:refs/remotes/origin/*".to_string()]
        } else {
            refspecs_owned
        };

        // Convert owned strings to &str for the fetch call
        let refspec_refs: Vec<&str> =
            refspecs_owned.iter().map(std::string::String::as_str).collect();

        // Perform the fetch
        remote
            .fetch(
                &refspec_refs,
                Some(&mut fetch_opts),
                None, // log message
            )
            .map_err(RepoError::RemoteError)?;

        Ok(true)
    }

    /// Pulls changes from a remote repository
    ///
    /// This fetches from the remote and merges the changes into the current branch.
    ///
    /// # Arguments
    ///
    /// * `remote_name` - The name of the remote (e.g., "origin")
    /// * `branch_name` - Optional branch name to pull from (defaults to tracking branch)
    ///
    /// # Returns
    ///
    /// * `Result<bool, RepoError>` - Success indicator or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// // Pull from the tracking branch
    /// repo.pull("origin", None).expect("Failed to pull");
    ///
    /// // Pull from a specific branch
    /// repo.pull("origin", Some("feature-branch")).expect("Failed to pull from feature branch");
    /// ```
    pub fn pull(&self, remote_name: &str, branch_name: Option<&str>) -> Result<bool, RepoError> {
        // First, fetch from remote
        self.fetch(remote_name, None, false)?;

        // Get current branch
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let Some(current_branch) = head.shorthand() else {
            return Err(RepoError::BranchNameError(Git2Error::from_str(
                "HEAD is not a valid branch",
            )));
        };

        // Determine target branch name
        let target_branch = branch_name.unwrap_or(current_branch);

        // Build remote branch name
        let remote_ref = format!("{remote_name}/{target_branch}");

        // Get remote branch's commit
        let remote_branch = self
            .repo
            .find_reference(&format!("refs/remotes/{remote_ref}"))
            .map_err(RepoError::RemoteError)?;

        let remote_commit = remote_branch.peel_to_commit().map_err(RepoError::PeelError)?;

        // Get local commit
        let local_commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        // Create an annotated commit for merge
        let annotated_commit =
            self.repo.find_annotated_commit(remote_commit.id()).map_err(RepoError::CommitError)?;

        // Perform the merge analysis
        let (merge_analysis, _) =
            self.repo.merge_analysis(&[&annotated_commit]).map_err(RepoError::MergeError)?;

        if merge_analysis.is_up_to_date() {
            return Ok(true);
        }

        if merge_analysis.is_fast_forward() {
            // Fast-forward
            let mut reference = head;
            reference
                .set_target(remote_commit.id(), "pull: Fast-forward")
                .map_err(RepoError::ReferenceError)?;

            // Update working directory
            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();

            self.repo.checkout_head(Some(&mut checkout_opts)).map_err(RepoError::CheckoutError)?;

            return Ok(true);
        }

        // Normal merge (not fast-forward)
        let mut merge_opts = MergeOptions::new();
        merge_opts.fail_on_conflict(false);

        // Perform the merge
        self.repo
            .merge(&[&annotated_commit], Some(&mut merge_opts), None)
            .map_err(RepoError::MergeError)?;

        // Check for conflicts
        let mut index = self.repo.index().map_err(RepoError::IndexError)?;
        if index.has_conflicts() {
            //errors out with message
            return Err(RepoError::MergeConflictError(Git2Error::from_str(
                "Merge conflicts detected. Please resolve conflicts and commit the result.",
            )));
        }

        // Create the merge commit
        let tree_id = index.write_tree().map_err(RepoError::WriteTreeError)?;
        let tree = self.repo.find_tree(tree_id).map_err(RepoError::TreeError)?;

        let signature = self.repo.signature().map_err(RepoError::SignatureError)?;
        let message = format!("Merge branch '{target_branch}' of {remote_ref}");

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&local_commit, &remote_commit],
            )
            .map_err(RepoError::CommitError)?;

        // Clean up merge state
        self.repo.cleanup_state().map_err(RepoError::MergeError)?;

        Ok(true)
    }

    /// Pushes the current branch to a remote repository with custom SSH key paths
    ///
    /// # Arguments
    ///
    /// * `remote_name` - The name of the remote (e.g., "origin")
    /// * `follow_tags` - Whether to also push tags
    /// * `ssh_key_paths` - Paths to SSH keys to try for authentication
    ///
    /// # Returns
    ///
    /// * `Result<bool, RepoError>` - Success indicator or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    /// use std::path::PathBuf;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let key_paths = vec![
    ///     PathBuf::from("/custom/path/to/id_ed25519"),
    ///     PathBuf::from("/custom/path/to/id_rsa"),
    /// ];
    /// repo.push_with_ssh_config("origin", Some(true), key_paths).expect("Failed to push");
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn push_with_ssh_config(
        &self,
        remote_name: &str,
        follow_tags: Option<bool>,
        ssh_key_paths: Vec<PathBuf>,
    ) -> Result<bool, RepoError> {
        // Get the current branch name
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let branch_name = head.shorthand().ok_or_else(|| {
            RepoError::BranchNameError(Git2Error::from_str("Invalid branch reference"))
        })?;

        // Get the remote
        let mut remote = self.repo.find_remote(remote_name).map_err(RepoError::RemoteError)?;

        // Create callbacks with custom key paths
        let mut callbacks = RemoteCallbacks::new();
        let key_paths = ssh_key_paths.clone(); // Clone for the closure

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            self.create_ssh_credentials(url, username_from_url, allowed_types, Some(&key_paths))
        });

        // Rest of push implementation is the same...
        // [implementation continues as in the regular push method]

        // Setup push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Create refspecs
        let mut refspecs = Vec::new();

        // Add the branch refspec (e.g., "refs/heads/main:refs/heads/main")
        let branch_refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
        refspecs.push(branch_refspec);

        // Add tags refspec if follow_tags is enabled
        if follow_tags.unwrap_or(false) {
            // Push all tags
            refspecs.push("refs/tags/*:refs/tags/*".to_string());
        }

        // Perform the push operation
        remote.push(&refspecs, Some(&mut push_options)).map_err(RepoError::RemoteError)?;

        Ok(true)
    }

    /// Finds the common ancestor (merge base) between HEAD and another reference
    ///
    /// # Arguments
    ///
    /// * `git_ref` - The reference to compare with HEAD (branch name, tag, or commit SHA)
    ///
    /// # Returns
    ///
    /// * `Result<String, RepoError>` - The SHA of the common ancestor commit, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let merge_base = repo.get_diverged_commit("feature-branch")
    ///     .expect("Failed to find common ancestor");
    /// println!("Common ancestor commit: {}", merge_base);
    /// ```
    pub fn get_diverged_commit(&self, git_ref: &str) -> Result<String, RepoError> {
        // Resolve the git reference to an object
        let object = self.repo.revparse_single(git_ref).map_err(RepoError::ReferenceError)?;

        // Convert to a commit (peeling if needed)
        let commit = object.peel_to_commit().map_err(RepoError::PeelError)?;

        // Get the HEAD commit
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        // Find the merge base (common ancestor) between the commit and HEAD
        let merge_base_oid =
            self.repo.merge_base(commit.id(), head_commit.id()).map_err(RepoError::MergeError)?;

        // Convert the merge base OID to a string
        Ok(merge_base_oid.to_string())
    }

    /// Gets all files changed since a specific reference with their status
    ///
    /// # Arguments
    ///
    /// * `git_ref` - The reference to compare with HEAD (branch name, tag, or commit SHA)
    ///
    /// # Returns
    ///
    /// * `Result<Vec<GitChangedFile>, RepoError>` - List of changed files with status, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let changed_files = repo.get_all_files_changed_since_sha_with_status("v1.0.0")
    ///     .expect("Failed to get changed files");
    ///
    /// for file in changed_files {
    ///     println!("File: {} - {:?}", file.path, file.status);
    /// }
    /// ```
    pub fn get_all_files_changed_since_sha_with_status(
        &self,
        git_ref: &str,
    ) -> Result<Vec<GitChangedFile>, RepoError> {
        // Revparse the reference to get the actual commit object
        let object = self.repo.revparse_single(git_ref).map_err(RepoError::ReferenceError)?;

        // Convert to a commit (peeling if needed)
        let old_commit = object.peel_to_commit().map_err(RepoError::PeelError)?;

        // Get HEAD commit
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        // Store changed files here
        let mut changed_files = Vec::new();

        // Use git command directly via libgit2's advanced functionality
        let mut revwalk = self.repo.revwalk().map_err(RepoError::GitFailure)?;
        revwalk.push(head_commit.id()).map_err(RepoError::GitFailure)?;
        revwalk.hide(old_commit.id()).map_err(RepoError::GitFailure)?;

        // Collect all commit IDs between old_commit and head_commit
        let commit_ids: Vec<_> = revwalk.filter_map(Result::ok).collect();

        // Get all commits
        let commits: Vec<_> =
            commit_ids.iter().filter_map(|id| self.repo.find_commit(*id).ok()).collect();

        // Process each commit to find changes
        for commit in commits {
            // If commit has a parent, diff against it
            if let Ok(parent) = commit.parent(0) {
                let parent_tree = parent.tree().map_err(RepoError::GitFailure)?;
                let commit_tree = commit.tree().map_err(RepoError::GitFailure)?;

                // Create diff
                let diff = self
                    .repo
                    .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
                    .map_err(RepoError::DiffError)?;

                // Process each file in the diff
                for delta in diff.deltas() {
                    let status = match delta.status() {
                        Delta::Added => GitFileStatus::Added,
                        Delta::Deleted => GitFileStatus::Deleted,
                        _ => GitFileStatus::Modified,
                    };

                    // Get appropriate path based on status
                    let path_buf = if status == GitFileStatus::Deleted {
                        if let Some(path) = delta.old_file().path() {
                            self.local_path.join(path)
                        } else {
                            continue;
                        }
                    } else if let Some(path) = delta.new_file().path() {
                        self.local_path.join(path)
                    } else {
                        continue;
                    };

                    // Convert to string
                    if let Some(path_str) = path_buf.to_str() {
                        // Check if we already have this file in our list
                        let file_exists =
                            changed_files.iter().any(|f: &GitChangedFile| f.path == path_str);

                        // Only add if we haven't seen it yet
                        if !file_exists {
                            changed_files
                                .push(GitChangedFile { path: path_str.to_string(), status });
                        }
                    }
                }
            }
        }

        // If still no deleted files, try a more direct approach
        let has_deleted = changed_files.iter().any(|f| f.status == GitFileStatus::Deleted);

        if !has_deleted {
            // Get trees
            let old_tree = old_commit.tree().map_err(RepoError::GitFailure)?;
            let head_tree = head_commit.tree().map_err(RepoError::GitFailure)?;

            // Find files in old tree that aren't in the new tree
            let mut old_files = std::collections::HashMap::new();

            old_tree
                .walk(TreeWalkMode::PreOrder, |dir, entry| {
                    if let Some(name) = entry.name() {
                        let path =
                            if dir.is_empty() { name.to_string() } else { format!("{dir}{name}") };
                        old_files.insert(path, entry.id());
                    }
                    TreeWalkResult::Ok
                })
                .map_err(RepoError::DiffError)?;

            // Remove files that exist in head tree
            head_tree
                .walk(TreeWalkMode::PreOrder, |dir, entry| {
                    if let Some(name) = entry.name() {
                        let path =
                            if dir.is_empty() { name.to_string() } else { format!("{dir}{name}") };
                        old_files.remove(&path);
                    }
                    TreeWalkResult::Ok
                })
                .map_err(RepoError::DiffError)?;

            // Any remaining files in old_files were deleted
            for (path, _) in old_files {
                let full_path = self.local_path.join(&path);
                if let Some(path_str) = full_path.to_str() {
                    // Make sure we don't add duplicates
                    let file_exists = changed_files.iter().any(|f| f.path == path_str);
                    if !file_exists {
                        changed_files.push(GitChangedFile {
                            path: path_str.to_string(),
                            status: GitFileStatus::Deleted,
                        });
                    }
                }
            }
        }

        Ok(changed_files)
    }

    /// Gets all files changed since a specific reference
    ///
    /// # Arguments
    ///
    /// * `git_ref` - The reference to compare with HEAD (branch name, tag, or commit SHA)
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, RepoError>` - List of changed file paths, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let changed_files = repo.get_all_files_changed_since_sha("v1.0.0")
    ///     .expect("Failed to get changed files");
    ///
    /// for file in changed_files {
    ///     println!("Changed file: {}", file);
    /// }
    /// ```
    pub fn get_all_files_changed_since_sha(&self, git_ref: &str) -> Result<Vec<String>, RepoError> {
        let changed_files_with_status =
            self.get_all_files_changed_since_sha_with_status(git_ref)?;

        let paths =
            changed_files_with_status.into_iter().map(|changed_file| changed_file.path).collect();

        Ok(paths)
    }

    /// Gets all files changed since a specific branch within specified package paths
    ///
    /// # Arguments
    ///
    /// * `packages_paths` - List of package paths to filter by
    /// * `branch` - The branch to compare against
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, RepoError>` - List of changed file paths within the packages, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    /// let packages = vec!["packages/pkg1".to_string(), "packages/pkg2".to_string()];
    /// let changed_files = repo.get_all_files_changed_since_branch(&packages, "main")
    ///     .expect("Failed to get changed files");
    ///
    /// for file in changed_files {
    ///     println!("Changed file: {}", file);
    /// }
    /// ```
    pub fn get_all_files_changed_since_branch(
        &self,
        packages_paths: &[String],
        branch: &str,
    ) -> Result<Vec<String>, RepoError> {
        // Get all files changed since the specified branch
        let files = self.get_all_files_changed_since_sha(branch)?;

        // Pre-compute canonical package paths to avoid repeating the work
        let canonical_pkg_paths: Vec<(String, String)> = packages_paths
            .iter()
            .filter_map(|path| match canonicalize_path(path) {
                Ok(canonical) => Some((path.clone(), canonical)),
                Err(_) => None,
            })
            .collect();

        // Create a hashset to ensure we don't add duplicate files
        let mut unique_files = std::collections::HashSet::new();

        // Process each file and check against all package paths
        for file in &files {
            let Ok(canonical_file) = canonicalize_path(file) else { continue };

            // Check if the file belongs to any of our package paths
            for (original_path, canonical_path) in &canonical_pkg_paths {
                if canonical_file.starts_with(canonical_path) {
                    unique_files.insert(file.clone());
                    break; // File matched a package, no need to check other packages
                } else if file.starts_with(original_path) {
                    // Fallback to simple prefix check
                    unique_files.insert(file.clone());
                    break;
                }
            }
        }

        // Convert the hashset to a vector for the result
        Ok(unique_files.into_iter().collect())
    }

    /// Gets commits made since a specific reference or from the beginning
    ///
    /// # Arguments
    ///
    /// * `since` - Optional reference to start from (branch, tag, or commit SHA)
    /// * `relative` - Optional path to filter commits by (only commits touching this path)
    ///
    /// # Returns
    ///
    /// * `Result<Vec<RepoCommit>, RepoError>` - List of commits, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    ///
    /// // Get all commits since v1.0.0
    /// let commits = repo.get_commits_since(
    ///     Some("v1.0.0".to_string()),
    ///     &None
    /// ).expect("Failed to get commits");
    ///
    /// // Get all commits that touched a specific file
    /// let file_commits = repo.get_commits_since(
    ///     None,
    ///     &Some("src/main.rs".to_string())
    /// ).expect("Failed to get commits");
    ///
    /// for commit in commits {
    ///     println!("{}: {} ({})",
    ///         commit.hash,
    ///         commit.message,
    ///         commit.author_name
    ///     );
    /// }
    /// ```
    pub fn get_commits_since(
        &self,
        since: Option<String>,
        relative: &Option<String>,
    ) -> Result<Vec<RepoCommit>, RepoError> {
        // Start a revwalk to iterate through commits
        let mut revwalk = self.repo.revwalk().map_err(RepoError::GitFailure)?;

        // Configure the revwalk based on the inputs
        if let Some(since) = since {
            // Resolve the 'since' reference to an OID
            let obj = self.repo.revparse_single(&since).map_err(RepoError::ReferenceError)?;
            let since_commit = obj.peel_to_commit().map_err(RepoError::PeelError)?;

            // Push HEAD as the end point
            revwalk.push_head().map_err(RepoError::RevWalkError)?;

            // Hide any commits reachable from 'since'
            // This effectively gives us commits that are in HEAD but not in 'since'
            revwalk.hide(since_commit.id()).map_err(RepoError::CommitError)?;
        } else {
            // If no 'since' is provided, just walk from HEAD
            revwalk.push_head().map_err(RepoError::RevWalkError)?;
        }

        // Set sorting (newest first, like 'git log')
        revwalk.set_sorting(git2::Sort::TIME).map_err(RepoError::RevWalkError)?;

        // Collect commits
        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = oid_result.map_err(RepoError::CommitOidError)?;
            let commit = self.repo.find_commit(oid).map_err(RepoError::CommitError)?;

            // If relative path is provided, check if commit touches this path
            if let Some(rel_path) = &relative {
                let rel_path_buf = PathBuf::from(rel_path);

                // Skip this commit if it doesn't touch the specified path
                if !self.commit_touches_path(&commit, &rel_path_buf)? {
                    continue;
                }
            }

            // Format the commit date
            let time = commit.time();
            let offset = time.offset_minutes();
            let sign = if offset < 0 { '-' } else { '+' };
            let offset_hours = offset.abs() / 60;
            let offset_minutes = offset.abs() % 60;

            // Create a DateTime object
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(time.seconds(), 0)
                .unwrap_or_else(chrono::Utc::now);

            // Format in RFC2822 format (to match git log --date=rfc2822)
            let date_str = format!(
                "{} {}{:02}:{:02}",
                datetime.format("%a, %d %b %Y %H:%M:%S"),
                sign,
                offset_hours,
                offset_minutes
            );

            // Get author information
            let author = commit.author();
            let name = author.name().unwrap_or("").to_string();
            let email = author.email().unwrap_or("").to_string();

            // Get commit message
            let message = commit.message().unwrap_or("").to_string();

            // Create and add the repository commit
            commits.push(RepoCommit {
                hash: commit.id().to_string(),
                author_name: name,
                author_email: email,
                author_date: date_str,
                message,
            });
        }

        Ok(commits)
    }

    /// Gets tags from either local repository or remote
    ///
    /// # Arguments
    ///
    /// * `local` - If Some(true), gets local tags; if Some(false) or None, gets remote tags
    ///
    /// # Returns
    ///
    /// * `Result<Vec<RepoTags>, RepoError>` - List of tags, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use git::repo::Repo;
    ///
    /// let repo = Repo::open("./my-repo").expect("Failed to open repository");
    ///
    /// // Get local tags
    /// let local_tags = repo.get_remote_or_local_tags(Some(true))
    ///     .expect("Failed to get local tags");
    ///
    /// // Get remote tags (default)
    /// let remote_tags = repo.get_remote_or_local_tags(None)
    ///     .expect("Failed to get remote tags");
    ///
    /// for tag in local_tags {
    ///     println!("Tag: {} ({})", tag.tag, tag.hash);
    /// }
    /// ```
    pub fn get_remote_or_local_tags(
        &self,
        local: Option<bool>,
    ) -> Result<Vec<RepoTags>, RepoError> {
        match local {
            Some(true) => self.get_local_tags(),
            Some(false) | None => self.get_remote_tags(),
        }
    }

    /// Creates SSH credentials for Git operations
    ///
    /// This is an internal helper method used by push, fetch, etc.
    ///
    /// # Arguments
    ///
    /// * `_url` - The remote URL
    /// * `username_from_url` - Optional username extracted from the URL
    /// * `_allowed_types` - Types of credentials allowed by the remote
    /// * `custom_key_paths` - Optional custom SSH key paths to try
    ///
    /// # Returns
    ///
    /// * `Result<Cred, Git2Error>` - SSH credentials or an error
    #[allow(clippy::unused_self)]
    fn create_ssh_credentials(
        &self,
        _url: &str,
        username_from_url: Option<&str>,
        _allowed_types: CredentialType,
        custom_key_paths: Option<&Vec<PathBuf>>,
    ) -> Result<Cred, Git2Error> {
        // Get the list of key paths to try
        let key_paths = match custom_key_paths {
            Some(paths) => paths.clone(),
            None => {
                // Use default paths based on home directory (cross-platform)
                if let Some(home_dir) = dirs::home_dir() {
                    vec![
                        home_dir.join(".ssh").join("id_ed25519"), // Ed25519 (preferred by GitHub)
                        home_dir.join(".ssh").join("id_rsa"),     // RSA (widely used)
                        home_dir.join(".ssh").join("id_ecdsa"),   // ECDSA
                        home_dir.join(".ssh").join("id_dsa"),     // DSA (legacy)
                    ]
                } else {
                    // Fallback if we can't find home directory
                    return Err(Git2Error::from_str(
                        "Could not determine home directory for SSH keys",
                    ));
                }
            }
        };

        // Try to find a username from public key if username wasn't provided in URL
        let username = match username_from_url {
            Some(name) => name.to_string(),
            None => {
                // Try to extract username from the public key files
                for key_path in &key_paths {
                    let pub_key_path = key_path.with_extension("pub");
                    if let Ok(content) = std::fs::read_to_string(&pub_key_path) {
                        // Public key format is typically: ssh-xxx AAAAB3Nza... username@host
                        if let Some(username_part) = content.split_whitespace().nth(2) {
                            if let Some(username) = username_part.split('@').next() {
                                return Cred::ssh_key(username, None, key_path, None);
                            }
                        }
                    }
                }

                // Fallback to environment user or "git"
                match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                    Ok(name) => name,
                    Err(_) => "git".to_string(),
                }
            }
        };

        // Try each key in order until one works
        for key_path in key_paths {
            if key_path.exists() {
                match Cred::ssh_key(
                    &username, None, // public key path (None to use default)
                    &key_path, None, // passphrase (None if no passphrase)
                ) {
                    Ok(cred) => return Ok(cred),
                    Err(_) => continue, // Try the next key if this one fails
                }
            }
        }

        // If all specific attempts fail, try the SSH agent
        Cred::ssh_key_from_agent(&username)
    }

    /// Checks if a commit touches a specific path
    ///
    /// This is an internal helper method used by get_commits_since.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit to check
    /// * `path` - The path to check for changes
    ///
    /// # Returns
    ///
    /// * `Result<bool, RepoError>` - True if the commit touches the path, false otherwise, or an error
    fn commit_touches_path(&self, commit: &Commit, path: &PathBuf) -> Result<bool, RepoError> {
        if commit.parent_count() == 0 {
            // For initial commit, check if the path exists in the tree
            let tree = commit.tree().map_err(RepoError::GitFailure)?;
            return Ok(tree.get_path(path).is_ok());
        }

        // For non-initial commits, check diff with parent
        let parent = commit.parent(0).map_err(RepoError::GitFailure)?;
        let parent_tree = parent.tree().map_err(RepoError::GitFailure)?;
        let commit_tree = commit.tree().map_err(RepoError::GitFailure)?;

        // Create diff options
        let mut diff_opts = DiffOptions::new();

        // If path is not at repository root, add it as a pathspec filter
        if !path.as_os_str().is_empty() {
            diff_opts.pathspec(path);
        }

        // Create diff
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut diff_opts))
            .map_err(RepoError::DiffError)?;

        // If the diff has any deltas, this commit touches the path
        Ok(diff.deltas().count() > 0)
    }

    /// Gets all local tags in the repository
    ///
    /// This is an internal helper method used by get_remote_or_local_tags.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<RepoTags>, RepoError>` - List of local tags, or an error
    fn get_local_tags(&self) -> Result<Vec<RepoTags>, RepoError> {
        let mut tags = Vec::new();

        // Get all references matching "refs/tags/*"
        let tag_refs =
            self.repo.references_glob("refs/tags/*").map_err(RepoError::ReferenceError)?;

        // Iterate through tag references
        for tag_ref_result in tag_refs {
            let tag_ref = tag_ref_result.map_err(RepoError::TagError)?;

            // Get the reference name (e.g., "refs/tags/v1.0.0")
            let ref_name = tag_ref.name().ok_or_else(|| {
                RepoError::TagError(Git2Error::from_str("Invalid reference name"))
            })?;

            // Extract the tag name from the full reference path
            let tag_name = ref_name.strip_prefix("refs/tags/").ok_or_else(|| {
                RepoError::TagError(Git2Error::from_str("Invalid tag reference format"))
            })?;

            // Get the target OID for this reference
            let target_oid = tag_ref.target().ok_or_else(|| {
                RepoError::TagError(Git2Error::from_str("Reference has no target"))
            })?;

            // If the reference is a tag object (annotated tag), dereference it to get the commit
            let final_oid = if let Ok(tag) = self.repo.find_tag(target_oid) {
                tag.target_id()
            } else {
                target_oid
            };

            tags.push(RepoTags { hash: final_oid.to_string(), tag: tag_name.to_string() });
        }

        Ok(tags)
    }

    /// Gets all remote tags from the 'origin' remote
    ///
    /// This is an internal helper method used by get_remote_or_local_tags.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<RepoTags>, RepoError>` - List of remote tags, or an error
    fn get_remote_tags(&self) -> Result<Vec<RepoTags>, RepoError> {
        let mut tags = Vec::new();

        // Find the "origin" remote
        let mut remote = self.repo.find_remote("origin").map_err(RepoError::RemoteError)?;

        // Connect to the remote
        remote.connect(Direction::Fetch).map_err(RepoError::RemoteError)?;

        // List all references on the remote
        let remote_refs = remote.list().map_err(RepoError::RemoteError)?;

        // Filter and extract tag references
        for remote_ref in remote_refs {
            let ref_name = remote_ref.name();

            // Check if this is a tag reference
            if let Some(tag_name) = ref_name.strip_prefix("refs/tags/") {
                // Skip peeled tags (^{}) which are duplicates
                if tag_name.ends_with("^{}") {
                    continue;
                }

                tags.push(RepoTags {
                    hash: remote_ref.oid().to_string(),
                    tag: tag_name.to_string(),
                });
            }
        }

        Ok(tags)
    }

    /// Creates an initial commit in a new repository
    ///
    /// This is an internal helper method used when creating a new repository.
    ///
    /// # Returns
    ///
    /// * `Result<(), RepoError>` - Success or an error
    fn make_initial_commit(&self) -> Result<(), RepoError> {
        let signature = self.repo.signature().map_err(RepoError::SignatureError)?;

        let tree_id = {
            let mut index = self.repo.index().map_err(RepoError::IndexError)?;
            index
                .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
                .map_err(RepoError::AddFilesError)?;
            index.write_tree().map_err(RepoError::WriteTreeError)?
        };

        let tree = self.repo.find_tree(tree_id).map_err(RepoError::TreeError)?;

        self.repo
            .commit(Some("HEAD"), &signature, &signature, "chore: initial commit", &tree, &[])
            .map_err(RepoError::CommitError)?;

        Ok(())
    }
}
