use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::types::{MonorepoRepository, MonorepoRepositoryError};
use napi::{Error, Result};
use sublime_git_tools::{Repo, RepoError};

fn repo_format_napi_error(err: RepoError) -> Error<MonorepoRepositoryError> {
    Error::new(err.clone().into(), err.to_string())
}

impl AsRef<str> for MonorepoRepositoryError {
    fn as_ref(&self) -> &str {
        match self {
            MonorepoRepositoryError::NapiError(e) => e.status.as_ref(),
            MonorepoRepositoryError::RepoError(e) => e.as_ref(),
        }
    }
}

impl From<RepoError> for MonorepoRepositoryError {
    fn from(err: RepoError) -> Self {
        MonorepoRepositoryError::RepoError(err)
    }
}

#[napi]
impl MonorepoRepository {
    /// Opens an existing Git repository at the specified path.
    ///
    /// @param root_path - The path to the existing repository
    /// @returns A new MonorepoRepository instance
    /// @throws If the repository cannot be opened
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-project');
    /// console.log(`Opened repository at: ${repo.path}`);
    /// ```
    #[napi(factory)]
    pub fn open(root_path: String) -> Result<Self, MonorepoRepositoryError> {
        let inner = Repo::open(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(MonorepoRepository { repo_instance: Rc::new(RefCell::new(inner)) })
    }

    /// Creates a new Git repository at the specified path.
    /// This initializes a new Git repository with an initial commit on the 'main' branch.
    ///
    /// @param root_path - The path where the repository should be created
    /// @returns A new MonorepoRepository instance
    /// @throws If the repository cannot be created
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.create('./new-project');
    /// console.log(`Created repository at: ${repo.path}`);
    /// ```
    #[napi(factory)]
    pub fn create(root_path: String) -> Result<Self, MonorepoRepositoryError> {
        let inner = Repo::create(root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(MonorepoRepository { repo_instance: Rc::new(RefCell::new(inner)) })
    }

    /// Clones a Git repository from a URL to a local path.
    ///
    /// @param url - The URL of the repository to clone
    /// @param root_path - The local path where the repository should be cloned
    /// @returns A new MonorepoRepository instance
    /// @throws If the repository cannot be cloned
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.clone('https://github.com/example/repo.git', './cloned-repo');
    /// console.log(`Cloned repository to: ${repo.path}`);
    /// ```
    #[napi(factory)]
    pub fn clone(url: String, root_path: String) -> Result<Self, MonorepoRepositoryError> {
        let inner =
            Repo::clone(url.as_str(), root_path.as_str()).map_err(repo_format_napi_error)?;
        Ok(MonorepoRepository { repo_instance: Rc::new(RefCell::new(inner)) })
    }

    /// Gets the local path of the repository.
    ///
    /// @returns The path to the repository
    ///
    /// @example
    /// ```js
    /// const repo = GitRepository.open('./my-repo');
    /// console.log(`Repository path: ${repo.path}`);
    /// ```
    #[napi(getter)]
    pub fn path(&self) -> String {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.get_repo_path().display().to_string()
    }

    /// Configures the repository with user information.
    ///
    /// @param username - The Git user name
    /// @param email - The Git user email
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If the configuration cannot be set
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.setConfig('Jane Doe', 'jane@example.com');
    /// ```
    #[napi]
    pub fn set_config(
        &self,
        username: String,
        email: String,
    ) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.config(username.as_str(), email.as_str()).map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Creates a new branch based on the current HEAD.
    ///
    /// @param name - The name for the new branch
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If the branch cannot be created
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.createBranch('feature/new-feature');
    /// ```
    #[napi]
    pub fn create_branch(&self, name: String) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.create_branch(name.as_str()).map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Lists all local branches in the repository.
    ///
    /// @returns A list of branch names
    /// @throws If branches cannot be listed
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// const branches = repo.branches;
    /// for (const branch of branches) {
    ///   console.log(`Branch: ${branch}`);
    /// }
    /// ```
    #[napi(getter)]
    pub fn branches(&self) -> Result<Vec<String>, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let branches = repo_instance.list_branches().map_err(repo_format_napi_error)?;

        Ok(branches)
    }

    /// Lists all configuration entries for the repository.
    ///
    /// @returns A map of config keys to values
    /// @throws If config cannot be retrieved
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// const config = repo.config;
    /// for (const [key, value] of Object.entries(config)) {
    ///   console.log(`${key} = ${value}`);
    /// }
    /// ```
    #[napi(getter)]
    pub fn config(&self) -> Result<HashMap<String, String>, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let config = repo_instance.list_config().map_err(repo_format_napi_error)?;

        Ok(config)
    }

    /// Checks out a local branch.
    ///
    /// @param branch_name - The name of the branch to checkout
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If checkout fails
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.checkout('feature-branch');
    /// ```
    #[napi]
    pub fn checkout(&self, branch_name: String) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.checkout(branch_name.as_str()).map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Gets the name of the currently checked out branch.
    ///
    /// @returns The current branch name
    /// @throws If the current branch cannot be determined
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// console.log(`Current branch: ${repo.currentBranch}`);
    /// ```
    #[napi(getter)]
    pub fn current_branch(&self) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let branch = repo_instance.get_current_branch().map_err(repo_format_napi_error)?;
        Ok(branch)
    }

    /// Creates a new tag at the current HEAD.
    ///
    /// @param name - The name for the new tag
    /// @param message - Optional message for the tag
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If the tag cannot be created
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.createTag('v1.0.0', 'Version 1.0.0 release');
    /// ```
    #[napi(ts_args_type = "name: string, message?: string")]
    pub fn create_tag(
        &self,
        name: String,
        message: Option<String>,
    ) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.create_tag(name.as_str(), message).map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Adds a file to the Git index.
    ///
    /// @param file_path - The path to the file to add
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If the file cannot be added
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.add('src/main.js');
    /// ```
    #[napi]
    pub fn add(&self, file_path: String) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.add(file_path.as_str()).map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Adds all changed files to the Git index.
    ///
    /// @returns The MonorepoRepository instance for method chaining
    /// @throws If files cannot be added
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.addAll();
    /// ```
    #[napi]
    pub fn add_all(&self) -> Result<Self, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        repo_instance.add_all().map_err(repo_format_napi_error)?;

        Ok(MonorepoRepository { repo_instance: Rc::clone(&self.repo_instance) })
    }

    /// Gets the name of the last tag in the repository.
    ///
    /// @returns The last tag name
    /// @throws If no tags are found or an error occurs
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// console.log(`Last tag: ${repo.last_tag}`);
    /// ```
    #[napi(getter)]
    pub fn last_tag(&self) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let tag = repo_instance.get_last_tag().map_err(repo_format_napi_error)?;

        Ok(tag)
    }

    /// Gets the SHA of the current HEAD commit.
    ///
    /// @returns The current commit SHA
    /// @throws If the SHA cannot be retrieved
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// console.log(`Current commit: ${repo.currentSha}`);
    /// ```
    #[napi(getter)]
    pub fn current_sha(&self) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let sha = repo_instance.get_current_sha().map_err(repo_format_napi_error)?;

        Ok(sha)
    }

    /// Gets the SHA of the parent of the current HEAD commit.
    ///
    /// @returns The previous commit SHA
    /// @throws If the SHA cannot be retrieved
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// console.log(`Previous commit: ${repo.previousSha}`);
    /// ```
    #[napi(getter)]
    pub fn previous_sha(&self) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let sha = repo_instance.get_previous_sha().map_err(repo_format_napi_error)?;

        Ok(sha)
    }

    /// Creates a new commit with the current index.
    ///
    /// @param message - The commit message
    /// @returns The new commit's SHA
    /// @throws If the commit cannot be created
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// repo.add('src/main.js');
    /// const commitId = repo.commit('fix: update main.js');
    /// console.log(`Created commit: ${commitId}`);
    /// ```
    #[napi]
    pub fn commit(&self, message: String) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let oid = repo_instance.commit(message.as_str()).map_err(repo_format_napi_error)?;

        Ok(oid)
    }

    /// Adds all changes and creates a new commit.
    /// This method performs both addAll() and commit() in one step.
    ///
    /// @param message - The commit message
    /// @returns The new commit's SHA
    /// @throws If the commit cannot be created
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// const commitId = repo.commitChanges('feat: add new feature');
    /// console.log(`Created commit: ${commitId}`);
    /// ```
    #[napi]
    pub fn commit_changes(&self, message: String) -> Result<String, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let oid = repo_instance.commit_changes(message.as_str()).map_err(repo_format_napi_error)?;

        Ok(oid)
    }

    /// Gets the status of the repository in porcelain format.
    /// Returns a list of changed file paths.
    ///
    /// @returns List of changed file paths
    /// @throws If status cannot be retrieved
    ///
    /// @example
    /// ```js
    /// const repo = MonorepoRepository.open('./my-repo');
    /// const status = repo.status;
    /// for (const file of status) {
    ///   console.log(`Changed file: ${file}`);
    /// }
    /// ```
    #[napi(getter)]
    pub fn status(&self) -> Result<Vec<String>, MonorepoRepositoryError> {
        let repo_instance = self.repo_instance.borrow();
        let status = repo_instance.status_porcelain().map_err(repo_format_napi_error)?;

        Ok(status)
    }
}
