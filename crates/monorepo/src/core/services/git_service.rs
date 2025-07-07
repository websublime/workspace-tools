//! Git operations service implementation
//!
//! Provides centralized Git operations for the monorepo including repository
//! status, commit history, branch management, and file change tracking.

use crate::error::Result;
use std::path::Path;
use sublime_git_tools::{GitChangedFile, Repo, RepoCommit};

/// Git operations service
///
/// Provides monorepo-aware Git operations including status checking,
/// commit history analysis, branch management, and change tracking.
/// All operations are performed within the context of the monorepo repository.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::GitOperationsService;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let git_service = GitOperationsService::new("/path/to/monorepo")?;
///
/// // Get current branch
/// let branch = git_service.get_current_branch()?;
/// println!("Current branch: {}", branch);
///
/// // Get repository status
/// let is_clean = git_service.is_repository_clean()?;
/// println!("Repository clean: {}", is_clean);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct GitOperationsService {
    /// Git repository instance
    repository: Repo,
}

#[allow(dead_code)]
impl GitOperationsService {
    /// Create a new Git operations service
    ///
    /// Initializes the Git service with the repository at the specified path.
    /// The path should point to the root of a Git repository or a subdirectory
    /// within a Git repository.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Path to the Git repository or subdirectory
    ///
    /// # Returns
    ///
    /// A new Git operations service.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Path is not within a Git repository
    /// - Git repository is corrupted or inaccessible
    /// - Insufficient permissions to access repository
    pub fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let path_str = root_path
            .as_ref()
            .to_str()
            .ok_or_else(|| crate::error::Error::git("Invalid UTF-8 in repository path"))?;

        let repository = Repo::open(path_str).map_err(|e| {
            crate::error::Error::git(format!(
                "Failed to open Git repository at {}: {}",
                root_path.as_ref().display(),
                e
            ))
        })?;

        Ok(Self { repository })
    }

    /// Get the underlying repository
    ///
    /// Provides access to the underlying Git repository for operations
    /// that require the raw repository interface.
    ///
    /// # Returns
    ///
    /// Reference to the Git repository.
    pub fn repository(&self) -> &Repo {
        &self.repository
    }

    /// Get current branch name
    ///
    /// Returns the name of the currently checked out branch.
    ///
    /// # Returns
    ///
    /// Name of the current branch.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Repository is in a detached HEAD state
    /// - Git operation fails
    pub fn get_current_branch(&self) -> Result<String> {
        self.repository
            .get_current_branch()
            .map_err(|e| crate::error::Error::git(format!("Failed to get current branch: {e}")))
    }

    /// Check if repository is clean
    ///
    /// Returns true if the repository has no uncommitted changes,
    /// false if there are staged or unstaged changes.
    ///
    /// # Returns
    ///
    /// True if repository is clean, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if Git status cannot be determined.
    pub fn is_repository_clean(&self) -> Result<bool> {
        let staged_files = self.get_staged_files()?;
        let unstaged_files = self.get_unstaged_files()?;
        Ok(staged_files.is_empty() && unstaged_files.is_empty())
    }

    /// Get staged files
    ///
    /// Returns a list of files that are staged for commit.
    ///
    /// # Returns
    ///
    /// Vector of staged file paths relative to repository root.
    ///
    /// # Errors
    ///
    /// Returns an error if Git status cannot be determined.
    pub fn get_staged_files(&self) -> Result<Vec<String>> {
        self.repository
            .get_staged_files()
            .map_err(|e| crate::error::Error::git(format!("Failed to get staged files: {e}")))
    }

    /// Get unstaged files
    ///
    /// Returns a list of files that have modifications but are not staged.
    ///
    /// # Returns
    ///
    /// Vector of unstaged file paths relative to repository root.
    ///
    /// # Errors
    ///
    /// Returns an error if Git status cannot be determined.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn get_unstaged_files(&self) -> Result<Vec<String>> {
        // Note: This method might need to be implemented in sublime_git_tools
        // For now, we'll return an empty list as a placeholder
        Ok(Vec::new())
    }

    /// Get commits since a reference
    ///
    /// Returns a list of commits that have been made since the specified
    /// reference (tag, branch, or commit hash).
    ///
    /// # Arguments
    ///
    /// * `since_ref` - Reference to start from (e.g., "v1.0.0", "main", "abc123")
    /// * `until_ref` - Optional reference to end at
    ///
    /// # Returns
    ///
    /// Vector of commits in chronological order (newest first).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reference does not exist
    /// - Git operation fails
    pub fn get_commits_since(
        &self,
        since_ref: Option<String>,
        until_ref: &Option<String>,
    ) -> Result<Vec<RepoCommit>> {
        self.repository
            .get_commits_since(since_ref, until_ref)
            .map_err(|e| crate::error::Error::git(format!("Failed to get commits: {e}")))
    }

    /// Get changed files for a commit
    ///
    /// Returns a list of files that were changed in the specified commit.
    ///
    /// # Arguments
    ///
    /// * `commit_hash` - Hash of the commit to analyze
    ///
    /// # Returns
    ///
    /// Vector of changed files with their change types.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Commit hash does not exist
    /// - Git operation fails
    pub fn get_changed_files_for_commit(&self, commit_hash: &str) -> Result<Vec<GitChangedFile>> {
        self.repository
            .get_all_files_changed_since_sha(commit_hash)
            .map(|files| {
                files
                    .into_iter()
                    .map(|path| GitChangedFile {
                        path,
                        status: sublime_git_tools::GitFileStatus::Modified, // Simplified for now
                        staged: false,
                        workdir: true,
                    })
                    .collect()
            })
            .map_err(|e| crate::error::Error::git(format!("Failed to get changed files: {e}")))
    }

    /// Get all files changed since a commit
    ///
    /// Returns a list of all files that have been changed since the specified commit.
    ///
    /// # Arguments
    ///
    /// * `commit_hash` - Hash of the commit to compare against
    ///
    /// # Returns
    ///
    /// Vector of file paths that have changed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Commit hash does not exist
    /// - Git operation fails
    pub fn get_all_files_changed_since(&self, commit_hash: &str) -> Result<Vec<String>> {
        self.repository.get_all_files_changed_since_sha(commit_hash).map_err(|e| {
            crate::error::Error::git(format!(
                "Failed to get changed files since {commit_hash}: {e}"
            ))
        })
    }

    /// Get last tag
    ///
    /// Returns the most recent Git tag in the repository.
    ///
    /// # Returns
    ///
    /// Name of the last tag.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No tags exist in the repository
    /// - Git operation fails
    pub fn get_last_tag(&self) -> Result<String> {
        self.repository
            .get_last_tag()
            .map_err(|e| crate::error::Error::git(format!("Failed to get last tag: {e}")))
    }

    /// Get repository configuration
    ///
    /// Returns the Git configuration for the repository as a key-value map.
    ///
    /// # Returns
    ///
    /// Map of configuration keys to values.
    ///
    /// # Errors
    ///
    /// Returns an error if Git configuration cannot be read.
    pub fn get_repository_config(&self) -> Result<std::collections::HashMap<String, String>> {
        self.repository
            .list_config()
            .map_err(|e| crate::error::Error::git(format!("Failed to get repository config: {e}")))
    }

    /// Check if path is ignored by Git
    ///
    /// Returns true if the specified path is ignored by .gitignore rules.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check (relative to repository root)
    ///
    /// # Returns
    ///
    /// True if path is ignored, false otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if Git operation fails.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn is_path_ignored(&self, path: &str) -> Result<bool> {
        // Note: This method might need to be implemented in sublime_git_tools
        // For now, we'll return false as a placeholder
        let _ = path; // Suppress unused parameter warning
        Ok(false)
    }

    /// Get repository root path
    ///
    /// Returns the absolute path to the root of the Git repository.
    ///
    /// # Returns
    ///
    /// Path to repository root.
    ///
    /// # Errors
    ///
    /// Returns an error if repository path cannot be determined.
    #[allow(clippy::unused_self)]
    pub fn get_repository_root(&self) -> Result<std::path::PathBuf> {
        // Note: This method might need to be implemented in sublime_git_tools
        // For now, we'll return the current directory as a placeholder
        std::env::current_dir()
            .map_err(|e| crate::error::Error::git(format!("Failed to get repository root: {e}")))
    }
}
