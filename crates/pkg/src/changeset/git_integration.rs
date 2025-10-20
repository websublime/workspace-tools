//! Git integration for detecting packages affected by commits.
//!
//! **What**: Provides functionality to analyze Git commits and determine which packages
//! in a workspace are affected by file changes, supporting both monorepo and single-package
//! repositories.
//!
//! **How**: The `PackageDetector` uses `sublime_git_tools` to retrieve commit information
//! and file changes, then uses `sublime_standard_tools` to detect the workspace structure
//! (monorepo vs single package) and map changed files to their corresponding packages.
//!
//! **Why**: Automating the detection of affected packages from Git commits enables
//! developers to quickly identify which packages need version bumps and should be included
//! in a changeset, reducing manual work and potential errors in the release process.

use crate::error::{ChangesetError, ChangesetResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use sublime_git_tools::{Repo, RepoCommit};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, WorkspacePackage};

/// Detects packages affected by Git commits.
///
/// The `PackageDetector` analyzes commits in a Git repository to determine which packages
/// are affected by the changes. It handles both monorepo and single-package repositories,
/// automatically detecting the workspace structure and mapping file changes to packages.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changeset::PackageDetector;
/// use sublime_git_tools::Repo;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let repo = Repo::open(".")?;
/// let fs = FileSystemManager::new();
///
/// let detector = PackageDetector::new(workspace_root, repo, fs);
///
/// // Detect affected packages from specific commits
/// let commit_ids = vec!["abc123".to_string(), "def456".to_string()];
/// let packages = detector.detect_affected_packages(&commit_ids).await?;
///
/// println!("Affected packages: {:?}", packages);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PackageDetector<'a> {
    /// Root directory of the workspace.
    workspace_root: PathBuf,
    /// Git repository instance.
    repo: &'a Repo,
    /// Filesystem manager for file operations.
    fs: FileSystemManager,
    /// Monorepo detector for workspace analysis.
    monorepo_detector: MonorepoDetector<FileSystemManager>,
}

impl<'a> PackageDetector<'a> {
    /// Creates a new `PackageDetector`.
    ///
    /// # Parameters
    ///
    /// * `workspace_root` - The root directory of the workspace
    /// * `repo` - Git repository instance
    /// * `fs` - Filesystem manager
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changeset::PackageDetector;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = PackageDetector::new(
    ///     PathBuf::from("."),
    ///     Repo::open(".")?,
    ///     FileSystemManager::new(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new(workspace_root: impl Into<PathBuf>, repo: &'a Repo, fs: FileSystemManager) -> Self {
        let workspace_root = workspace_root.into();
        let monorepo_detector = MonorepoDetector::with_filesystem(fs.clone());
        Self { workspace_root, repo, fs, monorepo_detector }
    }

    /// Detects packages affected by the given commits.
    ///
    /// This method analyzes each commit to determine which files were changed, then maps
    /// those files to packages. It handles both monorepo and single-package repositories.
    ///
    /// # Parameters
    ///
    /// * `commit_ids` - List of commit hashes to analyze
    ///
    /// # Returns
    ///
    /// A vector of unique package names that were affected by the commits.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git operations fail
    /// - Package detection fails
    /// - File system operations fail
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # async fn example(detector: PackageDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// let commits = vec![
    ///     "abc123def456".to_string(),
    ///     "789012ghi345".to_string(),
    /// ];
    ///
    /// let affected = detector.detect_affected_packages(&commits).await?;
    /// println!("Found {} affected packages", affected.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_affected_packages(
        &self,
        commit_ids: &[String],
    ) -> ChangesetResult<Vec<String>> {
        if commit_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Check if this is a monorepo
        let is_monorepo = self.is_monorepo().await?;

        // Get all changed files from commits
        let changed_files = self.get_changed_files_from_commits(commit_ids)?;

        if changed_files.is_empty() {
            return Ok(Vec::new());
        }

        // Map files to packages
        let packages = if is_monorepo {
            self.map_files_to_packages_monorepo(&changed_files).await?
        } else {
            self.map_files_to_packages_single(&changed_files).await?
        };

        Ok(packages)
    }

    /// Checks if the workspace is a monorepo.
    ///
    /// # Returns
    ///
    /// `true` if the workspace is detected as a monorepo, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if monorepo detection fails due to file system issues.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # async fn example(detector: PackageDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// if detector.is_monorepo().await? {
    ///     println!("This is a monorepo workspace");
    /// } else {
    ///     println!("This is a single-package workspace");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_monorepo(&self) -> ChangesetResult<bool> {
        let result = self.monorepo_detector.is_monorepo_root(&self.workspace_root).await;

        match result {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(ChangesetError::GitIntegration {
                operation: "monorepo detection".to_string(),
                reason: format!("Failed to detect monorepo: {}", e),
            }),
        }
    }

    /// Lists all packages in the workspace.
    ///
    /// For monorepos, returns all workspace packages. For single-package repositories,
    /// returns the single package if a package.json exists.
    ///
    /// # Returns
    ///
    /// A vector of package names found in the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package detection fails
    /// - File system operations fail
    /// - package.json parsing fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # async fn example(detector: PackageDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// let packages = detector.list_packages().await?;
    /// for package in packages {
    ///     println!("Found package: {}", package);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_packages(&self) -> ChangesetResult<Vec<String>> {
        let is_monorepo = self.is_monorepo().await?;

        if is_monorepo {
            let packages = self.get_workspace_packages().await?;
            Ok(packages.iter().map(|p| p.name.clone()).collect())
        } else {
            // Single package - get name from package.json
            let package_json_path = self.workspace_root.join("package.json");
            if self.fs.exists(&package_json_path).await {
                let content = self.fs.read_file_string(&package_json_path).await.map_err(|e| {
                    ChangesetError::GitIntegration {
                        operation: "read package.json".to_string(),
                        reason: format!("Failed to read package.json: {}", e),
                    }
                })?;

                let package_json: serde_json::Value =
                    serde_json::from_str(&content).map_err(|e| ChangesetError::GitIntegration {
                        operation: "parse package.json".to_string(),
                        reason: format!("Failed to parse package.json: {}", e),
                    })?;

                if let Some(name) = package_json.get("name").and_then(|n| n.as_str()) {
                    return Ok(vec![name.to_string()]);
                }
            }

            Ok(Vec::new())
        }
    }

    /// Gets the list of commits between two references.
    ///
    /// # Parameters
    ///
    /// * `from_ref` - Starting reference (commits after this)
    /// * `to_ref` - Ending reference (commits up to this)
    ///
    /// # Returns
    ///
    /// A vector of `RepoCommit` instances between the references.
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail or references are invalid.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # fn example(detector: PackageDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// let commits = detector.get_commits_between("v1.0.0", "HEAD")?;
    /// println!("Found {} commits", commits.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_commits_between(
        &self,
        from_ref: &str,
        to_ref: &str,
    ) -> ChangesetResult<Vec<RepoCommit>> {
        self.repo.get_commits_between(from_ref, to_ref, &None).map_err(|e| {
            ChangesetError::GitIntegration {
                operation: format!("get commits between {} and {}", from_ref, to_ref),
                reason: format!("Failed to get commits: {}", e),
            }
        })
    }

    /// Gets commits since a specific reference.
    ///
    /// # Parameters
    ///
    /// * `since` - Optional reference to start from. If `None`, gets all commits.
    ///
    /// # Returns
    ///
    /// A vector of `RepoCommit` instances since the reference.
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail or reference is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # fn example(detector: PackageDetector) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get commits since last tag
    /// let commits = detector.get_commits_since(Some("v1.0.0".to_string()))?;
    ///
    /// // Get all commits
    /// let all_commits = detector.get_commits_since(None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_commits_since(&self, since: Option<String>) -> ChangesetResult<Vec<RepoCommit>> {
        self.repo.get_commits_since(since, &None).map_err(|e| ChangesetError::GitIntegration {
            operation: "get commits since reference".to_string(),
            reason: format!("Failed to get commits: {}", e),
        })
    }

    /// Gets all files changed in the given commits.
    ///
    /// This method retrieves the list of files that were modified, added, or deleted
    /// in each commit. If there's only one commit, it compares against its parent.
    /// For multiple commits, it finds the range and gets all changes.
    fn get_changed_files_from_commits(
        &self,
        commit_ids: &[String],
    ) -> ChangesetResult<Vec<PathBuf>> {
        if commit_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_files = HashSet::new();

        if commit_ids.len() == 1 {
            // Single commit - compare with parent
            let commit_id = &commit_ids[0];
            let parent_id = format!("{}^", commit_id);

            let files =
                self.repo.get_files_changed_between(&parent_id, commit_id).map_err(|e| {
                    ChangesetError::GitIntegration {
                        operation: format!("get files changed in commit {}", commit_id),
                        reason: format!("Failed to get changed files: {}", e),
                    }
                })?;

            all_files.extend(files.into_iter().map(|f| PathBuf::from(f.path)));
        } else {
            // Multiple commits - need to find the range from oldest to newest
            // Commits might come in any order, so we need to determine which is oldest
            // We'll get all changed files for each commit and combine them
            for commit_id in commit_ids {
                let parent_id = format!("{}^", commit_id);

                let files =
                    self.repo.get_files_changed_between(&parent_id, commit_id).map_err(|e| {
                        ChangesetError::GitIntegration {
                            operation: format!("get files changed in commit {}", commit_id),
                            reason: format!("Failed to get changed files: {}", e),
                        }
                    })?;

                all_files.extend(files.into_iter().map(|f| PathBuf::from(f.path)));
            }
        }

        Ok(all_files.into_iter().collect())
    }

    /// Maps changed files to packages in a monorepo.
    ///
    /// This method determines which packages contain the changed files by checking
    /// if each file path is within a package directory.
    async fn map_files_to_packages_monorepo(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<Vec<String>> {
        let packages = self.get_workspace_packages().await?;
        let mut affected_packages = HashSet::new();

        for file in changed_files {
            // Convert file to absolute path
            let file_absolute =
                if file.is_absolute() { file.clone() } else { self.workspace_root.join(file) };

            // Canonicalize the file path if possible, otherwise use as-is
            let file_canonical =
                file_absolute.canonicalize().unwrap_or_else(|_| file_absolute.clone());

            // Check which package contains this file
            for package in &packages {
                // Use the absolute_path field from WorkspacePackage
                let package_canonical = package
                    .absolute_path
                    .canonicalize()
                    .unwrap_or_else(|_| package.absolute_path.clone());

                // Check if file is within this package's directory
                if file_canonical.starts_with(&package_canonical) {
                    affected_packages.insert(package.name.clone());
                    break;
                }
            }
        }

        Ok(affected_packages.into_iter().collect())
    }

    /// Maps changed files to the single package.
    ///
    /// For single-package repositories, any file change (except certain ignored files)
    /// affects the single package.
    async fn map_files_to_packages_single(
        &self,
        changed_files: &[PathBuf],
    ) -> ChangesetResult<Vec<String>> {
        if changed_files.is_empty() {
            return Ok(Vec::new());
        }

        // Get the package name
        let packages = self.list_packages().await?;

        if packages.is_empty() {
            return Ok(Vec::new());
        }

        // In a single package repo, any change affects the package
        Ok(packages)
    }

    /// Gets all workspace packages in a monorepo.
    async fn get_workspace_packages(&self) -> ChangesetResult<Vec<WorkspacePackage>> {
        self.monorepo_detector.detect_packages(&self.workspace_root).await.map_err(|e| {
            ChangesetError::GitIntegration {
                operation: "get workspace packages".to_string(),
                reason: format!("Failed to get workspace packages: {}", e),
            }
        })
    }

    /// Returns a reference to the workspace root.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # fn example(detector: PackageDetector) {
    /// let root = detector.workspace_root();
    /// println!("Workspace root: {:?}", root);
    /// # }
    /// ```
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Returns a reference to the Git repository.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changeset::PackageDetector;
    /// # fn example(detector: PackageDetector) {
    /// let repo = detector.repo();
    /// println!("Repository path: {:?}", repo.get_repo_path());
    /// # }
    /// ```
    #[must_use]
    pub fn repo(&self) -> &Repo {
        self.repo
    }
}
