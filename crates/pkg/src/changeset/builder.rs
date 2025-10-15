//! # Changeset builder for creating changesets from Git commits
//!
//! ## What
//! Provides the `ChangesetBuilder` struct that orchestrates the creation of changesets
//! by analyzing Git commits, detecting affected packages, and calculating version bumps.
//! This is the primary entry point for generating changesets from Git history.
//!
//! ## How
//! - Uses Git as the source of truth for commits and changed files
//! - Leverages `PackageChangeDetector` to map files to packages
//! - Delegates commit message parsing to `ConventionalCommitService`
//! - Orchestrates all components to build complete `Changeset` structures
//!
//! ## Why
//! Following the Git-first architecture ensures correct commit filtering per package
//! and proper separation of concerns between Git operations, package detection,
//! and semantic versioning logic.

#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::path::{Path, PathBuf};

use sublime_git_tools::{GitChangedFile, Repo};
use sublime_standard_tools::filesystem::FileSystemManager;

use crate::{
    changeset::{ChangeEntry, ChangeReason, Changeset, ChangesetPackage, PackageChangeDetector},
    config::PackageToolsConfig,
    conventional::{ConventionalCommit, ConventionalCommitService},
    error::{ChangesetError, ChangesetResult},
    version::VersionBump,
    Package, ResolvedVersion, Version,
};

/// Builder for creating changesets from Git commits.
///
/// Orchestrates the entire changeset creation process by:
/// 1. Getting changed files from Git (source of truth)
/// 2. Detecting affected packages using `PackageChangeDetector`
/// 3. Filtering commits per package using Git file operations
/// 4. Parsing commit messages for semantic information
/// 5. Calculating version bumps and building the final changeset
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::changeset::ChangesetBuilder;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_pkg_tools::conventional::ConventionalCommitService;
/// use sublime_git_tools::Repo;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let repo = Repo::open(".")?;
/// let config = PackageToolsConfig::default();
/// let commit_service = ConventionalCommitService::new(repo, config.clone())?;
///
/// let builder = ChangesetBuilder::new(
///     &repo,
///     &commit_service,
///     PathBuf::from("."),
///     config,
/// )?;
///
/// let changeset = builder
///     .from_commits_since(None, "dev@example.com".to_string(), vec!["prod".to_string()])
///     .await?;
///
/// println!("Created changeset with {} packages", changeset.packages.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ChangesetBuilder<'a> {
    /// Git repository for file and commit operations
    repo: &'a Repo,
    /// Service for parsing conventional commits
    commit_service: &'a ConventionalCommitService,
    /// Root path of the workspace
    workspace_root: PathBuf,
    /// Configuration for package tools
    #[allow(dead_code)]
    config: PackageToolsConfig,
    /// Detector for mapping files to packages
    package_detector: PackageChangeDetector<FileSystemManager>,
}

impl<'a> ChangesetBuilder<'a> {
    /// Creates a new changeset builder.
    ///
    /// # Arguments
    ///
    /// * `repo` - Git repository reference
    /// * `commit_service` - Service for parsing conventional commits
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Configuration for package tools
    ///
    /// # Returns
    ///
    /// A new `ChangesetBuilder` instance ready to build changesets
    ///
    /// # Errors
    ///
    /// Returns error if workspace root is invalid or configuration is malformed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetBuilder;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// use sublime_git_tools::Repo;
    /// use std::path::PathBuf;
    ///
    /// # fn example(repo: &Repo, commit_service: &ConventionalCommitService) -> Result<(), Box<dyn std::error::Error>> {
    /// let config = PackageToolsConfig::default();
    ///
    /// let builder = ChangesetBuilder::new(
    ///     repo,
    ///     commit_service,
    ///     PathBuf::from("."),
    ///     config,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        repo: &'a Repo,
        commit_service: &'a ConventionalCommitService,
        workspace_root: PathBuf,
        config: PackageToolsConfig,
    ) -> ChangesetResult<Self> {
        // Validate workspace root exists
        if !workspace_root.exists() {
            return Err(ChangesetError::CreationFailed {
                branch: "unknown".to_string(),
                reason: format!("Workspace root does not exist: {}", workspace_root.display()),
            });
        }

        let filesystem = FileSystemManager::new();
        let package_detector = PackageChangeDetector::new(workspace_root.clone(), filesystem);

        Ok(Self { repo, commit_service, workspace_root, config, package_detector })
    }

    /// Creates a changeset from commits since a given reference.
    ///
    /// This is the main entry point for building changesets. It analyzes Git history,
    /// detects affected packages, and builds a complete changeset structure.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting reference (commit hash, tag, or branch). If `None`, uses the initial commit
    /// * `author` - Email of the changeset author
    /// * `target_environments` - List of target environments (e.g., "dev", "prod")
    ///
    /// # Returns
    ///
    /// A fully constructed `Changeset` ready to be saved
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Git operations fail
    /// - No packages are affected by the changes
    /// - Package detection fails
    /// - Commit parsing fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::changeset::ChangesetBuilder;
    ///
    /// # async fn example(builder: &ChangesetBuilder<'_>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create changeset from last release tag
    /// let changeset = builder
    ///     .from_commits_since(
    ///         Some("v1.0.0"),
    ///         "dev@example.com".to_string(),
    ///         vec!["prod".to_string()],
    ///     )
    ///     .await?;
    ///
    /// // Create changeset from all commits (initial commit)
    /// let full_changeset = builder
    ///     .from_commits_since(
    ///         None,
    ///         "dev@example.com".to_string(),
    ///         vec!["dev".to_string()],
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_commits_since(
        &self,
        from_ref: Option<&str>,
        author: String,
        target_environments: Vec<String>,
    ) -> ChangesetResult<Changeset> {
        // 1. Get all parsed conventional commits using the service
        let all_parsed_commits = if let Some(ref_str) = from_ref {
            // From a specific ref to HEAD
            self.commit_service
                .get_commits_between(ref_str, "HEAD")
                .await
                .map_err(|e| ChangesetError::GitAnalysisFailed { reason: e.to_string() })?
        } else {
            // From the beginning of repository - use None to get all commits
            self.commit_service
                .get_commits_since(None)
                .await
                .map_err(|e| ChangesetError::GitAnalysisFailed { reason: e.to_string() })?
        };

        if all_parsed_commits.is_empty() {
            return self.create_empty_changeset(author, target_environments);
        }

        // 2. Get all changed files from Git (source of truth)
        // We need to collect all files touched by all commits
        let changed_files = if let Some(ref_str) = from_ref {
            self.get_changed_files_from_git(Some(ref_str), None)?
        } else {
            // For all commits scenario, collect files from each commit
            self.get_all_changed_files_from_commits(&all_parsed_commits).await?
        };

        if changed_files.is_empty() {
            return self.create_empty_changeset(author, target_environments);
        }

        // 3. Detect affected packages using the detector
        let affected_packages =
            self.package_detector.detect_affected_packages(&changed_files).await?;

        if affected_packages.is_empty() {
            return self.create_empty_changeset(author, target_environments);
        }

        // 4. Build changeset packages by analyzing commits per package
        let mut changeset_packages = Vec::new();

        for (package_name, package_files) in affected_packages {
            // 4a. Filter commits that touched this package files
            let package_commits = self
                .filter_conventional_commits_by_files(&all_parsed_commits, &package_files)
                .await?;

            if package_commits.is_empty() {
                continue;
            }

            // 4b. Calculate version bump
            let bump = self.commit_service.calculate_version_bump(&package_commits);

            // 4c. Build change entries
            let changes = self.build_change_entries(&package_commits);

            // 4d. Get package information for versions
            let package_path =
                package_files.first().and_then(|p| p.parent()).unwrap_or(&self.workspace_root);

            let package_info = self.read_package_info(package_path).await;

            let (current_version, next_version) = match package_info {
                Ok(pkg) => {
                    let current = ResolvedVersion::Release(pkg.metadata.version.clone());
                    let next = self.calculate_next_version(&current, &bump);
                    (current, next)
                }
                Err(_) => {
                    // Fallback to 0.0.0 if package.json not found or invalid
                    let current = ResolvedVersion::Release(Version::new(0, 0, 0));
                    let next = self.calculate_next_version(&current, &bump);
                    (current, next)
                }
            };

            // 4e. Create commit hashes for the reason
            let commit_hashes: Vec<String> =
                package_commits.iter().map(|c| c.hash.clone()).collect();

            // 4f. Create changeset package
            let changeset_package = ChangesetPackage {
                name: package_name,
                bump,
                current_version,
                next_version,
                reason: ChangeReason::DirectChanges { commits: commit_hashes },
                dependency: None,
                changes,
            };

            changeset_packages.push(changeset_package);
        }

        // Ensure we have at least one package with changes
        if changeset_packages.is_empty() {
            return Err(ChangesetError::NoChangesDetected {
                since_ref: from_ref.unwrap_or("repository start").to_string(),
            });
        }

        // 5. Build the final changeset
        let branch = self.get_current_branch()?;
        let mut changeset = Changeset::new(branch, author);
        changeset.releases = target_environments;
        changeset.packages = changeset_packages;

        Ok(changeset)
    }

    /// Gets all files changed between two references using Git.
    ///
    /// This is the source of truth for which files have changed.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting reference (required)
    /// * `to_ref` - Ending reference (None = HEAD)
    ///
    /// # Returns
    ///
    /// List of absolute paths to changed files
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail
    fn get_changed_files_from_git(
        &self,
        from_ref: Option<&str>,
        to_ref: Option<&str>,
    ) -> ChangesetResult<Vec<PathBuf>> {
        let from = from_ref.ok_or_else(|| ChangesetError::GitAnalysisFailed {
            reason: "from_ref is required for get_changed_files_from_git".to_string(),
        })?;

        let to = to_ref.unwrap_or("HEAD");

        let repo_root = self.repo.get_repo_path();

        // Get changed files from Git
        let file_changes = self
            .repo
            .get_files_changed_between(from, to)
            .map_err(|e| ChangesetError::GitAnalysisFailed { reason: e.to_string() })?;

        // Convert to absolute paths
        let changed_files: Vec<PathBuf> =
            file_changes.into_iter().map(|fc| repo_root.join(&fc.path)).collect();

        Ok(changed_files)
    }

    /// Gets all files changed across a set of commits.
    ///
    /// # Arguments
    ///
    /// * `commits` - The commits to analyze
    ///
    /// # Returns
    ///
    /// List of unique absolute paths to changed files
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail
    async fn get_all_changed_files_from_commits(
        &self,
        commits: &[ConventionalCommit],
    ) -> ChangesetResult<Vec<PathBuf>> {
        let repo_root = self.repo.get_repo_path();
        let mut all_files = std::collections::HashSet::new();

        for commit in commits {
            let file_changes = self
                .repo
                .get_files_changed_in_commit(&commit.hash)
                .map_err(|e| ChangesetError::GitAnalysisFailed { reason: e.to_string() })?;

            for fc in file_changes {
                all_files.insert(repo_root.join(&fc.path));
            }
        }

        Ok(all_files.into_iter().collect())
    }

    /// Filters conventional commits that touched specific package files.
    ///
    /// Uses Git directly to check which files each commit modified,
    /// ensuring accurate filtering based on actual file changes.
    ///
    /// # Arguments
    ///
    /// * `commits` - All conventional commits to filter
    /// * `package_files` - Files belonging to the package
    ///
    /// # Returns
    ///
    /// List of conventional commits that touched the package
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail
    async fn filter_conventional_commits_by_files(
        &self,
        commits: &[ConventionalCommit],
        package_files: &[PathBuf],
    ) -> ChangesetResult<Vec<ConventionalCommit>> {
        let mut filtered = Vec::new();
        let repo_root = self.repo.get_repo_path();

        // Convert absolute paths to repo-relative paths
        let relative_files: Vec<PathBuf> = package_files
            .iter()
            .filter_map(|p| p.strip_prefix(repo_root).ok().map(|r| r.to_path_buf()))
            .collect();

        if relative_files.is_empty() {
            return Ok(Vec::new());
        }

        for commit in commits {
            // Get files changed in this commit from Git
            let changed_files = self
                .repo
                .get_files_changed_in_commit(&commit.hash)
                .map_err(|e| ChangesetError::GitAnalysisFailed { reason: e.to_string() })?;

            // Check if any changed file belongs to this package
            let touches_package = self.commit_touches_package(&changed_files, &relative_files);

            if touches_package {
                filtered.push(commit.clone());
            }
        }

        Ok(filtered)
    }

    /// Checks if a commit touches any files in a package.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - Files changed in the commit
    /// * `package_files` - Files belonging to the package
    ///
    /// # Returns
    ///
    /// `true` if the commit touches the package
    fn commit_touches_package(
        &self,
        changed_files: &[GitChangedFile],
        package_files: &[PathBuf],
    ) -> bool {
        changed_files.iter().any(|changed| {
            let changed_path = Path::new(&changed.path);
            package_files.iter().any(|pkg_file| {
                // Check exact match
                changed_path == pkg_file
                    // Check if changed file is inside package directory
                    || changed_path.starts_with(pkg_file)
                    // Check if package file is inside changed directory
                    || pkg_file.starts_with(changed_path)
            })
        })
    }

    /// Builds change entries from parsed conventional commits.
    ///
    /// # Arguments
    ///
    /// * `commits` - Parsed conventional commits
    ///
    /// # Returns
    ///
    /// List of change entries for the changeset
    fn build_change_entries(&self, commits: &[ConventionalCommit]) -> Vec<ChangeEntry> {
        commits
            .iter()
            .map(|commit| {
                let change_type = commit.commit_type.as_str().to_string();
                let description = if let Some(scope) = &commit.scope {
                    format!("{}: {}", scope, commit.description)
                } else {
                    commit.description.clone()
                };

                ChangeEntry {
                    change_type,
                    description,
                    breaking: commit.breaking,
                    commit: Some(commit.hash.clone()),
                }
            })
            .collect()
    }

    /// Reads package information from a directory.
    ///
    /// # Arguments
    ///
    /// * `package_path` - Path to the package directory
    ///
    /// # Returns
    ///
    /// Package information
    ///
    /// # Errors
    ///
    /// Returns error if package.json cannot be read
    async fn read_package_info(&self, package_path: &Path) -> ChangesetResult<Package> {
        Package::from_path(&self.package_detector.filesystem, package_path).await.map_err(|e| {
            ChangesetError::PackageJsonReadFailed {
                path: package_path.join("package.json"),
                reason: e.to_string(),
            }
        })
    }

    /// Calculates the next version based on current version and bump type.
    ///
    /// # Arguments
    ///
    /// * `current` - Current version
    /// * `bump` - Version bump type
    ///
    /// # Returns
    ///
    /// The next version after applying the bump
    fn calculate_next_version(
        &self,
        current: &ResolvedVersion,
        bump: &VersionBump,
    ) -> ResolvedVersion {
        let version = match current {
            ResolvedVersion::Release(v) => v.clone(),
            ResolvedVersion::Snapshot(s) => s.base_version.clone(),
        };

        let next = match bump {
            VersionBump::Major => Version::new(version.major() + 1, 0, 0),
            VersionBump::Minor => Version::new(version.major(), version.minor() + 1, 0),
            VersionBump::Patch => {
                Version::new(version.major(), version.minor(), version.patch() + 1)
            }
            VersionBump::None => version,
        };

        ResolvedVersion::Release(next)
    }

    /// Gets the current branch name.
    ///
    /// # Returns
    ///
    /// The current branch name
    ///
    /// # Errors
    ///
    /// Returns error if unable to determine the current branch
    fn get_current_branch(&self) -> ChangesetResult<String> {
        self.repo.get_current_branch().map_err(|e| ChangesetError::GitAnalysisFailed {
            reason: format!("Failed to get current branch: {}", e),
        })
    }

    /// Creates an empty changeset when no changes are detected.
    ///
    /// # Arguments
    ///
    /// * `author` - Changeset author
    /// * `target_environments` - Target environments
    ///
    /// # Returns
    ///
    /// An empty changeset
    fn create_empty_changeset(
        &self,
        author: String,
        target_environments: Vec<String>,
    ) -> ChangesetResult<Changeset> {
        let branch = self.get_current_branch().unwrap_or_else(|_| "unknown".to_string());
        let mut changeset = Changeset::new(branch, author);
        changeset.releases = target_environments;

        Ok(changeset)
    }
}
