use std::{collections::HashMap, path::PathBuf};

use napi::{bindgen_prelude::*, JsBoolean, JsUndefined};
use sublime_git_tools::{
    GitChangedFile as RepoGitChangedFile, GitFileStatus as RepoGitFileStatus, Repo, RepoCommit,
    RepoError, RepoTags,
};

/**
 * Represents a Git repository with methods for common Git operations.
 * This class wraps the underlying Rust Git implementation to provide
 * a JavaScript-friendly interface.
 */
#[napi]
#[allow(dead_code)]
pub struct GitRepository {
    pub(crate) inner: Repo,
}

/**
 * Represents the status of a file in a Git repository.
 * - Added: File has been added to the repository
 * - Deleted: File has been deleted from the repository
 * - Modified: File has been modified
 */
#[napi]
pub enum GitFileStatus {
    Added,
    Deleted,
    Modified,
}

/**
 * Represents a changed file in a Git repository.
 * Contains information about the file path and its status.
 */
#[napi]
pub struct GitChangedFile {
    /// The path to the changed file
    pub path: String,
    /// The status of the file (Added, Deleted, or Modified)
    pub status: GitFileStatus,
}

/**
 * Represents a commit in a Git repository.
 * Contains detailed information about the commit, including
 * author details, date, and commit message.
 */
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

/**
 * Represents a tag in a Git repository.
 * Contains the tag name and the commit hash it points to.
 */
#[napi]
pub struct GitTag {
    /// The hash of the commit that the tag points to
    pub hash: String,
    /// The name of the tag
    pub tag: String,
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

impl From<RepoGitFileStatus> for GitFileStatus {
    fn from(status: RepoGitFileStatus) -> Self {
        match status {
            RepoGitFileStatus::Added => GitFileStatus::Added,
            RepoGitFileStatus::Deleted => GitFileStatus::Deleted,
            RepoGitFileStatus::Modified => GitFileStatus::Modified,
        }
    }
}

impl From<RepoGitChangedFile> for GitChangedFile {
    fn from(file: RepoGitChangedFile) -> Self {
        GitChangedFile { path: file.path, status: file.status.into() }
    }
}

impl From<RepoCommit> for GitCommit {
    fn from(commit: RepoCommit) -> Self {
        GitCommit {
            hash: commit.hash,
            message: commit.message,
            author_date: commit.author_date,
            author_email: commit.author_email,
            author_name: commit.author_name,
        }
    }
}

impl From<RepoTags> for GitTag {
    fn from(tag: RepoTags) -> Self {
        GitTag { hash: tag.hash, tag: tag.tag }
    }
}

fn repo_format_napi_error(err: RepoError) -> Error<JsGitRepositoryError> {
    Error::new(err.clone().into(), err.to_string())
}

#[napi]
impl GitRepository {
    /**
     * Opens an existing Git repository at the specified path.
     *
     * @param root_path - The path to the existing repository
     * @returns A new GitRepository instance
     * @throws If the repository cannot be opened
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-project');
     * console.log(`Opened repository at: ${repo.path}`);
     * ```
     */
    #[napi(factory)]
    pub fn open(root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner = Repo::open(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    /**
     * Creates a new Git repository at the specified path.
     * This initializes a new Git repository with an initial commit on the 'main' branch.
     *
     * @param root_path - The path where the repository should be created
     * @returns A new GitRepository instance
     * @throws If the repository cannot be created
     *
     * @example
     * ```js
     * const repo = GitRepository.create('./new-project');
     * console.log(`Created repository at: ${repo.path}`);
     * ```
     */
    #[napi(factory)]
    pub fn create(root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner = Repo::create(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    /**
     * Clones a Git repository from a URL to a local path.
     *
     * @param url - The URL of the repository to clone
     * @param root_path - The local path where the repository should be cloned
     * @returns A new GitRepository instance
     * @throws If the repository cannot be cloned
     *
     * @example
     * ```js
     * const repo = GitRepository.clone('https://github.com/example/repo.git', './cloned-repo');
     * console.log(`Cloned repository to: ${repo.path}`);
     * ```
     */
    #[napi(factory)]
    pub fn clone(url: String, root_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner =
            Repo::clone(url.as_str(), root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(GitRepository { inner })
    }

    /**
     * Gets the local path of the repository.
     *
     * @returns The path to the repository
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * console.log(`Repository path: ${repo.path}`);
     * ```
     */
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.get_repo_path().display().to_string()
    }

    /**
     * Configures the repository with user information.
     *
     * @param username - The Git user name
     * @param email - The Git user email
     * @returns The GitRepository instance for method chaining
     * @throws If the configuration cannot be set
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.set_config('Jane Doe', 'jane@example.com');
     * ```
     */
    #[napi]
    pub fn set_config(
        &self,
        username: String,
        email: String,
    ) -> Result<Self, JsGitRepositoryError> {
        let inner =
            self.inner.config(username.as_str(), email.as_str()).map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Creates a new branch based on the current HEAD.
     *
     * @param name - The name for the new branch
     * @returns The GitRepository instance for method chaining
     * @throws If the branch cannot be created
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.create_branch('feature/new-feature');
     * ```
     */
    #[napi]
    pub fn create_branch(&self, name: String) -> Result<Self, JsGitRepositoryError> {
        let inner = self.inner.create_branch(name.as_str()).map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Lists all local branches in the repository.
     *
     * @returns A list of branch names
     * @throws If branches cannot be listed
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const branches = repo.branches;
     * for (const branch of branches) {
     *   console.log(`Branch: ${branch}`);
     * }
     * ```
     */
    #[napi(getter)]
    pub fn branches(&self) -> Result<Vec<String>, JsGitRepositoryError> {
        let branches = self.inner.list_branches().map_err(repo_format_napi_error)?;
        Ok(branches)
    }

    /**
     * Lists all configuration entries for the repository.
     *
     * @returns A map of config keys to values
     * @throws If config cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const config = repo.config;
     * for (const [key, value] of Object.entries(config)) {
     *   console.log(`${key} = ${value}`);
     * }
     * ```
     */
    #[napi(getter)]
    pub fn config(&self) -> Result<HashMap<String, String>, JsGitRepositoryError> {
        let config = self.inner.list_config().map_err(repo_format_napi_error)?;
        Ok(config)
    }

    /**
     * Checks out a local branch.
     *
     * @param branch_name - The name of the branch to checkout
     * @returns The GitRepository instance for method chaining
     * @throws If checkout fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.checkout('feature-branch');
     * ```
     */
    #[napi]
    pub fn checkout(&self, branch_name: String) -> Result<Self, JsGitRepositoryError> {
        let inner = self.inner.checkout(branch_name.as_str()).map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Gets the name of the currently checked out branch.
     *
     * @returns The current branch name
     * @throws If the current branch cannot be determined
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * console.log(`Current branch: ${repo.current_branch}`);
     * ```
     */
    #[napi(getter)]
    pub fn current_branch(&self) -> Result<String, JsGitRepositoryError> {
        let branch = self.inner.get_current_branch().map_err(repo_format_napi_error)?;
        Ok(branch)
    }

    /**
     * Creates a new tag at the current HEAD.
     *
     * @param name - The name for the new tag
     * @param message - Optional message for the tag
     * @returns The GitRepository instance for method chaining
     * @throws If the tag cannot be created
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.create_tag('v1.0.0', 'Version 1.0.0 release');
     * ```
     */
    #[napi(ts_args_type = "name: string, message?: string")]
    pub fn create_tag(
        &self,
        name: String,
        message: Option<String>,
    ) -> Result<Self, JsGitRepositoryError> {
        let inner =
            self.inner.create_tag(name.as_str(), message).map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Adds a file to the Git index.
     *
     * @param file_path - The path to the file to add
     * @returns The GitRepository instance for method chaining
     * @throws If the file cannot be added
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.add('src/main.js');
     * ```
     */
    #[napi]
    pub fn add(&self, file_path: String) -> Result<Self, JsGitRepositoryError> {
        let inner = self.inner.add(file_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Adds all changed files to the Git index.
     *
     * @returns The GitRepository instance for method chaining
     * @throws If files cannot be added
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.add_all();
     * ```
     */
    #[napi]
    pub fn add_all(&self) -> Result<Self, JsGitRepositoryError> {
        let inner = self.inner.add_all().map_err(repo_format_napi_error)?;
        Ok(Self { inner: inner.clone() })
    }

    /**
     * Gets the name of the last tag in the repository.
     *
     * @returns The last tag name
     * @throws If no tags are found or an error occurs
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * console.log(`Last tag: ${repo.last_tag}`);
     * ```
     */
    #[napi(getter)]
    pub fn last_tag(&self) -> Result<String, JsGitRepositoryError> {
        let tag = self.inner.get_last_tag().map_err(repo_format_napi_error)?;
        Ok(tag)
    }

    /**
     * Gets the SHA of the current HEAD commit.
     *
     * @returns The current commit SHA
     * @throws If the SHA cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * console.log(`Current commit: ${repo.current_sha}`);
     * ```
     */
    #[napi(getter)]
    pub fn current_sha(&self) -> Result<String, JsGitRepositoryError> {
        let sha = self.inner.get_current_sha().map_err(repo_format_napi_error)?;
        Ok(sha)
    }

    /**
     * Gets the SHA of the parent of the current HEAD commit.
     *
     * @returns The previous commit SHA
     * @throws If the SHA cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * console.log(`Previous commit: ${repo.previous_sha}`);
     * ```
     */
    #[napi(getter)]
    pub fn previous_sha(&self) -> Result<String, JsGitRepositoryError> {
        let sha = self.inner.get_previous_sha().map_err(repo_format_napi_error)?;
        Ok(sha)
    }

    /**
     * Creates a new commit with the current index.
     *
     * @param message - The commit message
     * @returns The new commit's SHA
     * @throws If the commit cannot be created
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * repo.add('src/main.js');
     * const commitId = repo.commit('fix: update main.js');
     * console.log(`Created commit: ${commitId}`);
     * ```
     */
    #[napi]
    pub fn commit(&self, message: String) -> Result<String, JsGitRepositoryError> {
        let oid = self.inner.commit(message.as_str()).map_err(repo_format_napi_error)?;
        Ok(oid)
    }

    /**
     * Adds all changes and creates a new commit.
     * This method performs both add_all() and commit() in one step.
     *
     * @param message - The commit message
     * @returns The new commit's SHA
     * @throws If the commit cannot be created
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const commitId = repo.commit_changes('feat: add new feature');
     * console.log(`Created commit: ${commitId}`);
     * ```
     */
    #[napi]
    pub fn commit_changes(&self, message: String) -> Result<String, JsGitRepositoryError> {
        let oid = self.inner.commit_changes(message.as_str()).map_err(repo_format_napi_error)?;
        Ok(oid)
    }

    /**
     * Gets the status of the repository in porcelain format.
     * Returns a list of changed file paths.
     *
     * @returns List of changed file paths
     * @throws If status cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const status = repo.status;
     * for (const file of status) {
     *   console.log(`Changed file: ${file}`);
     * }
     * ```
     */
    #[napi(getter)]
    pub fn status(&self) -> Result<Vec<String>, JsGitRepositoryError> {
        let status = self.inner.status_porcelain().map_err(repo_format_napi_error)?;
        Ok(status)
    }

    /**
     * Finds the branch that contains a specific commit.
     *
     * @param commit_sha - The commit SHA to find
     * @returns The branch name if found, undefined if not found
     * @throws If the search fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const commitSha = repo.current_sha;
     * const branch = repo.get_branch_from_commit(commitSha);
     * if (branch) {
     *   console.log(`Commit ${commitSha} is in branch: ${branch}`);
     * } else {
     *   console.log(`Commit ${commitSha} is not in any branch`);
     * }
     * ```
     */
    #[napi]
    pub fn get_branch_from_commit(
        &self,
        commit_sha: String,
    ) -> Result<Option<Either<String, JsUndefined>>, JsGitRepositoryError> {
        let branch = self
            .inner
            .get_branch_from_commit(commit_sha.as_str())
            .map_err(repo_format_napi_error)?;
        Ok(branch.map(Either::A))
    }

    /**
     * Finds all branches that contain a specific commit.
     *
     * @param commit_sha - The commit SHA to find
     * @returns List of branch names containing the commit
     * @throws If the search fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const commitSha = repo.current_sha;
     * const branches = repo.get_branches_containing_commit(commitSha);
     * for (const branch of branches) {
     *   console.log(`Branch contains commit: ${branch}`);
     * }
     * ```
     */
    #[napi]
    pub fn get_branches_containing_commit(
        &self,
        commit_sha: String,
    ) -> Result<Vec<String>, JsGitRepositoryError> {
        let branches = self
            .inner
            .get_branches_containing_commit(commit_sha.as_str())
            .map_err(repo_format_napi_error)?;
        Ok(branches)
    }

    /**
     * Pushes the current branch to a remote repository.
     *
     * @param remote_name - The name of the remote (defaults to "origin")
     * @param follow_tags - Whether to also push tags (defaults to false)
     * @returns Success indicator
     * @throws If the push fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * // Push current branch with tags
     * repo.push("origin", true);
     * ```
     */
    #[napi(ts_args_type = "remote_name?: string, follow_tags?: boolean")]
    pub fn push(
        &self,
        remote_name: Option<Either<String, JsUndefined>>,
        follow_tags: Option<Either<bool, JsBoolean>>,
    ) -> Result<bool, JsGitRepositoryError> {
        let remote = match remote_name {
            Some(Either::A(name)) => name,
            Some(Either::B(_)) | None => String::from("origin"),
        };

        let follow = match follow_tags {
            Some(Either::A(follow)) => follow,
            Some(Either::B(_)) | None => false,
        };

        let result = self.inner.push(&remote, Some(follow)).map_err(repo_format_napi_error)?;
        Ok(result)
    }

    /**
     * Pushes the current branch to a remote repository with custom SSH key paths.
     *
     * @param remote_name - The name of the remote (defaults to "origin")
     * @param follow_tags - Whether to also push tags (defaults to false)
     * @param ssh_key_paths - Paths to SSH keys to try for authentication
     * @returns Success indicator
     * @throws If the push fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * // Push with custom SSH keys
     * repo.push_with_ssh_config(
     *   "origin",
     *   true,
     *   ["/home/user/.ssh/id_ed25519", "/home/user/.ssh/id_rsa"]
     * );
     * ```
     */
    #[napi(ts_args_type = "remote_name?: string, follow_tags?: boolean, ssh_key_paths?: [string]")]
    pub fn push_with_ssh_config(
        &self,
        remote_name: Option<Either<String, JsUndefined>>,
        follow_tags: Option<Either<bool, JsBoolean>>,
        ssh_key_paths: Option<Either<Vec<String>, JsUndefined>>,
    ) -> Result<bool, JsGitRepositoryError> {
        let remote = match remote_name {
            Some(Either::A(name)) => name,
            Some(Either::B(_)) | None => String::from("origin"),
        };

        let follow = match follow_tags {
            Some(Either::A(follow)) => follow,
            Some(Either::B(_)) | None => false,
        };

        let key_paths = match ssh_key_paths {
            Some(Either::A(paths)) => paths.into_iter().map(PathBuf::from).collect(),
            Some(Either::B(_)) | None => Vec::new(),
        };

        let result = self
            .inner
            .push_with_ssh_config(&remote, Some(follow), key_paths)
            .map_err(repo_format_napi_error)?;
        Ok(result)
    }

    /**
     * Fetches changes from a remote repository.
     *
     * @param remote_name - The name of the remote (defaults to "origin")
     * @param refspecs - Optional reference specs to fetch
     * @param prune - Whether to prune deleted references (defaults to false)
     * @returns Success indicator
     * @throws If the fetch fails
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * // Fetch with default refspecs and no pruning
     * repo.fetch("origin");
     *
     * // Fetch a specific branch and prune
     * repo.fetch(
     *   "origin",
     *   ["refs/heads/main:refs/remotes/origin/main"],
     *   true
     * );
     * ```
     */
    #[napi(ts_args_type = "remote_name?: string, refspecs?: [string], prune?: boolean")]
    pub fn fetch(
        &self,
        remote_name: Option<Either<String, JsUndefined>>,
        refspecs: Option<Vec<String>>,
        prune: Option<Either<bool, JsBoolean>>,
    ) -> Result<bool, JsGitRepositoryError> {
        let remote = match remote_name {
            Some(Either::A(name)) => name,
            Some(Either::B(_)) | None => String::from("origin"),
        };

        let prune = match prune {
            Some(Either::A(prune)) => prune,
            Some(Either::B(_)) | None => false,
        };

        let result = match refspecs {
            Some(specs) => {
                // Convert Vec<String> to Vec<&str> for the API
                let ref_slices: Vec<&str> = specs.iter().map(|s| s.as_str()).collect();
                self.inner
                    .fetch(&remote, Some(&ref_slices), prune)
                    .map_err(repo_format_napi_error)?
            }
            None => self.inner.fetch(&remote, None, prune).map_err(repo_format_napi_error)?,
        };

        Ok(result)
    }

    /**
     * Pulls changes from a remote repository.
     * This fetches from the remote and merges the changes into the current branch.
     *
     * @param remote_name - The name of the remote (defaults to "origin")
     * @param branch_name - Optional branch name to pull from (defaults to tracking branch)
     * @returns Success indicator
     * @throws If the pull fails or if there are merge conflicts
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * try {
     *   // Pull from the tracking branch
     *   repo.pull("origin");
     *
     *   // Pull from a specific branch
     *   repo.pull("origin", "feature-branch");
     * } catch (error) {
     *   console.error(`Pull failed: ${error.message}`);
     * }
     * ```
     */
    #[napi(ts_args_type = "remote_name?: string, branch_name?: string")]
    pub fn pull(
        &self,
        remote_name: Option<Either<String, JsUndefined>>,
        branch_name: Option<Either<String, JsUndefined>>,
    ) -> Result<bool, JsGitRepositoryError> {
        let remote = match remote_name {
            Some(Either::A(name)) => name,
            Some(Either::B(_)) | None => String::from("origin"),
        };

        let branch_owned = match branch_name {
            Some(Either::A(name)) => Some(name),
            Some(Either::B(_)) | None => None,
        };

        let result = match branch_owned {
            Some(ref branch) => self.inner.pull(&remote, Some(branch.as_str())),
            None => self.inner.pull(&remote, None),
        }
        .map_err(repo_format_napi_error)?;

        Ok(result)
    }

    /**
     * Finds the common ancestor (merge base) between HEAD and another reference.
     *
     * @param git_ref - The reference to compare with HEAD (branch name, tag, or commit SHA)
     * @returns The SHA of the common ancestor commit
     * @throws If the common ancestor cannot be found
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const mergeBase = repo.get_diverged_commits("feature-branch");
     * console.log(`Common ancestor commit: ${mergeBase}`);
     * ```
     */
    #[napi]
    pub fn get_diverged_commits(&self, git_ref: String) -> Result<String, JsGitRepositoryError> {
        let result = self.inner.get_diverged_commit(&git_ref).map_err(repo_format_napi_error)?;

        Ok(result)
    }

    /**
     * Gets all files changed since a specific reference with their status.
     *
     * @param git_ref - The reference to compare with HEAD (branch name, tag, or commit SHA)
     * @returns List of changed files with status
     * @throws If changed files cannot be determined
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const changedFiles = repo.get_all_files_changed_since_sha_with_status("v1.0.0");
     * for (const file of changedFiles) {
     *   console.log(`File: ${file.path} - ${file.status}`);
     * }
     * ```
     */
    #[napi]
    pub fn get_all_files_changed_since_sha_with_status(
        &self,
        git_ref: String,
    ) -> Result<Vec<GitChangedFile>, JsGitRepositoryError> {
        let changed_status_list = self
            .inner
            .get_all_files_changed_since_sha_with_status(&git_ref)
            .map_err(repo_format_napi_error)?;

        let result = changed_status_list
            .into_iter()
            .map(GitChangedFile::from)
            .collect::<Vec<GitChangedFile>>();

        Ok(result)
    }

    /**
     * Gets all files changed since a specific reference.
     *
     * @param git_ref - The reference to compare with HEAD (branch name, tag, or commit SHA)
     * @returns List of changed file paths
     * @throws If changed files cannot be determined
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const changedFiles = repo.get_all_files_changed_since_sha("v1.0.0");
     * for (const file of changedFiles) {
     *   console.log(`Changed file: ${file}`);
     * }
     * ```
     */
    #[napi]
    pub fn get_all_files_changed_since_sha(
        &self,
        git_ref: String,
    ) -> Result<Vec<String>, JsGitRepositoryError> {
        let result =
            self.inner.get_all_files_changed_since_sha(&git_ref).map_err(repo_format_napi_error)?;

        Ok(result)
    }

    /**
     * Gets all files changed since a specific branch within specified package paths.
     *
     * @param packages_paths - List of package paths to filter by
     * @param branch_name - The branch to compare against
     * @returns List of changed file paths within the packages
     * @throws If changed files cannot be determined
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     * const packages = ['packages/pkg1', 'packages/pkg2'];
     * const changedFiles = repo.get_all_files_changed_since_branch(packages, 'main');
     * for (const file of changedFiles) {
     *   console.log(`Changed file: ${file}`);
     * }
     * ```
     */
    #[napi]
    pub fn get_all_files_changed_since_branch(
        &self,
        packages_paths: Vec<String>,
        branch_name: String,
    ) -> Result<Vec<String>, JsGitRepositoryError> {
        let result = self
            .inner
            .get_all_files_changed_since_branch(&packages_paths, &branch_name)
            .map_err(repo_format_napi_error)?;

        Ok(result)
    }

    /**
     * Gets commits made since a specific reference or from the beginning.
     *
     * @param since - Optional reference to start from (branch, tag, or commit SHA)
     * @param relative - Optional path to filter commits by (only commits touching this path)
     * @returns List of commits
     * @throws If commits cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     *
     * // Get all commits since v1.0.0
     * const commits = repo.get_commits_since("v1.0.0");
     *
     * // Get all commits that touched a specific file
     * const fileCommits = repo.get_commits_since("v1.0.0", "src/main.js");
     *
     * for (const commit of commits) {
     *   console.log(`${commit.hash}: ${commit.message} (${commit.author_name})`);
     * }
     * ```
     */
    #[napi(ts_args_type = "since?: string, relative?: string")]
    pub fn get_commits_since(
        &self,
        since: Option<Either<String, JsUndefined>>,
        relative: Option<Either<String, JsUndefined>>,
    ) -> Result<Vec<GitCommit>, JsGitRepositoryError> {
        let since_owned = match since {
            Some(Either::A(since)) => since,
            Some(Either::B(_)) | None => String::from("main"),
        };

        let commits = match relative {
            Some(Either::A(relative)) => self
                .inner
                .get_commits_since(Some(since_owned), &Some(relative))
                .map_err(repo_format_napi_error)?,
            Some(Either::B(_)) | None => self
                .inner
                .get_commits_since(Some(since_owned), &None)
                .map_err(repo_format_napi_error)?,
        };

        let result = commits.into_iter().map(GitCommit::from).collect::<Vec<GitCommit>>();

        Ok(result)
    }

    /**
     * Gets tags from either local repository or remote.
     *
     * @param local - If true, gets local tags; if false or undefined, gets remote tags
     * @returns List of tags
     * @throws If tags cannot be retrieved
     *
     * @example
     * ```js
     * const repo = GitRepository.open('./my-repo');
     *
     * // Get local tags
     * const localTags = repo.get_remote_or_local_tags(true);
     *
     * // Get remote tags (default)
     * const remoteTags = repo.get_remote_or_local_tags();
     *
     * for (const tag of localTags) {
     *   console.log(`Tag: ${tag.tag} (${tag.hash})`);
     * }
     * ```
     */
    #[napi(ts_args_type = "local?: boolean")]
    pub fn get_remote_or_local_tags(
        &self,
        local: Option<Either<bool, JsUndefined>>,
    ) -> Result<Vec<GitTag>, JsGitRepositoryError> {
        let local_tag = match local {
            Some(Either::A(local)) => local,
            Some(Either::B(_)) | None => false,
        };

        let local_remote_tags =
            self.inner.get_remote_or_local_tags(Some(local_tag)).map_err(repo_format_napi_error)?;

        let result = local_remote_tags.into_iter().map(GitTag::from).collect::<Vec<GitTag>>();

        Ok(result)
    }
}
