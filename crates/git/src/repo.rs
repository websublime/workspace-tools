use git2::{BranchType, Error as Git2Error, IndexAddOption, Repository, RepositoryInitOptions};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use thiserror::Error;

fn canonicalize_path(path: &str) -> Result<String, RepoError> {
    let location = PathBuf::from(path);
    let path = canonicalize(location.as_os_str()).map_err(RepoError::CanonicPathFailure)?;
    Ok(path.display().to_string())
}

pub struct Repo {
    repo: Repository,
    local_path: PathBuf,
}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Failed to canonicalize path: {0}")]
    CanonicPathFailure(#[source] std::io::Error),

    #[error("Failed to execute git: {0}")]
    GitFailure(#[source] Git2Error),

    #[error("Failed to create repository: {0}")]
    CreateRepoFailure(#[source] Git2Error),

    #[error("Failed to open repository: {0}")]
    OpenRepoFailure(#[source] Git2Error),

    #[error("Failed to clone repository: {0}")]
    CloneRepoFailure(#[source] Git2Error),

    #[error("Git configuration error: {0}")]
    ConfigError(#[source] Git2Error),

    #[error("Failed to get repository configuration entries: {0}")]
    ConfigEntriesError(#[source] Git2Error),

    #[error("Failed to get repository head: {0}")]
    HeadError(#[source] Git2Error),

    #[error("Failed to peel to commit: {0}")]
    PeelError(#[source] Git2Error),

    #[error("Failed to create branch: {0}")]
    BranchError(#[source] Git2Error),

    #[error("Failed to get repository signature: {0}")]
    SignatureError(#[source] Git2Error),

    #[error("Failed to map index: {0}")]
    IndexError(#[source] Git2Error),

    #[error("Failed to add files to index: {0}")]
    AddFilesError(#[source] Git2Error),

    #[error("Failed to write index: {0}")]
    WriteIndexError(#[source] Git2Error),

    #[error("Failed to find tree: {0}")]
    TreeError(#[source] Git2Error),

    #[error("Failed to commit: {0}")]
    CommitError(#[source] Git2Error),

    #[error("Failed to write tree: {0}")]
    WriteTreeError(#[source] Git2Error),

    #[error("Failed to list branches: {0}")]
    BranchListError(#[source] Git2Error),

    #[error("Failed to get branch name: {0}")]
    BranchNameError(#[source] Git2Error),

    #[error("Failed to checkout branch: {0}")]
    CheckoutBranchError(#[source] Git2Error),

    #[error("Failed to get last tag: {0}")]
    LastTagError(#[source] Git2Error),

    #[error("Failed to create tag: {0}")]
    CreateTagError(#[source] Git2Error),
}

impl From<Git2Error> for RepoError {
    fn from(err: Git2Error) -> Self {
        // You might want to match on error code to create specific errors
        // For simplicity, we'll use a default case here
        RepoError::GitFailure(err)
    }
}

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
        }
    }
}

impl Repo {
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
        let result = Self { repo, local_path: location_buf };

        // Now make the initial commit using our new instance
        result.make_initial_commit()?;

        Ok(result)
    }

    pub fn open(path: &str) -> Result<Self, RepoError> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::open(path).map_err(RepoError::OpenRepoFailure)?;

        Ok(Self { repo, local_path: PathBuf::from(local_path) })
    }

    pub fn clone(url: &str, path: &str) -> Result<Self, RepoError> {
        let local_path = canonicalize_path(path)?;
        let repo = Repository::clone(url, path).map_err(RepoError::CloneRepoFailure)?;

        Ok(Self { repo, local_path: PathBuf::from(local_path) })
    }

    pub fn get_repo_path(&self) -> &Path {
        self.local_path.as_path()
    }

    pub fn config(&self, username: &str, email: &str) -> Result<(), RepoError> {
        let mut config = self.repo.config().map_err(RepoError::ConfigError)?;
        config.set_str("user.name", username)?;
        config.set_str("user.email", email)?;
        config.set_bool("core.safecrlf", true)?;
        config.set_str("core.autocrlf", "input")?;
        config.set_bool("core.filemode", false)?;
        Ok(())
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<&Self, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        self.repo.branch(branch_name, &commit, false).map_err(RepoError::BranchError)?;
        Ok(self)
    }

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

    pub fn list_config(&self) -> Result<Vec<String>, RepoError> {
        let config = self.repo.config().map_err(RepoError::ConfigError)?;
        let mut config_names = Vec::new();

        let mut entries = config.entries(None).map_err(RepoError::ConfigEntriesError)?;
        while let Some(entry) = entries.next() {
            match entry {
                Ok(entry) => {
                    if let Some(name) = entry.name() {
                        config_names.push(name.to_string());
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(config_names)
    }

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

    pub fn get_current_branch(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let branch = head.shorthand().ok_or_else(|| {
            RepoError::BranchNameError(Git2Error::from_str("Invalid branch reference"))
        })?;

        Ok(branch.to_string())
    }

    pub fn create_tag(&self, tag: &str, message: Option<String>) -> Result<(), RepoError> {
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

        Ok(())
    }

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

    pub fn get_last_tag(&self) -> Result<String, RepoError> {
        let tags = self.repo.tag_names(None).map_err(RepoError::LastTagError)?;

        let last_tag = tags.iter().flatten().max_by_key(|&tag| tag.parse::<u64>().unwrap_or(0));

        last_tag
            .map(std::string::ToString::to_string)
            .ok_or_else(|| RepoError::LastTagError(Git2Error::from_str("No tags found")))
    }

    pub fn get_current_sha(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let target = head
            .target()
            .ok_or_else(|| RepoError::HeadError(Git2Error::from_str("No target found")));
        let sha = target.map_err(|_| RepoError::HeadError(Git2Error::from_str("No OID found")))?;

        Ok(sha.to_string())
    }

    pub fn get_previous_sha(&self) -> Result<String, RepoError> {
        let head = self.repo.head().map_err(RepoError::HeadError)?;
        let head_commit = head.peel_to_commit().map_err(RepoError::PeelError)?;

        // Get the parent commit (the previous commit)
        let parent = head_commit.parent(0).map_err(|e| {
            RepoError::GitFailure(Git2Error::from_str(&format!("Failed to get parent commit: {e}")))
        })?;

        // Get the SHA of the parent commit
        let previous_sha = parent.id().to_string();

        Ok(previous_sha)
    }

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
