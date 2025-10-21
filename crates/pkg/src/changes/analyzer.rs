//! Changes analyzer for detecting and analyzing file and commit changes.
//!
//! **What**: Provides the main `ChangesAnalyzer` that orchestrates change analysis operations,
//! integrating with Git for change detection and monorepo detection for package mapping.
//!
//! **How**: The analyzer initializes with a workspace root, Git repository, filesystem, and
//! configuration. It validates the Git repository and detects the project structure (single-package
//! vs monorepo) to enable accurate change analysis.
//!
//! **Why**: To provide a centralized, robust analyzer that can handle both simple projects and
//! complex monorepo structures, enabling accurate detection of which packages are affected by
//! changes.

use crate::changes::PackageMapper;
use crate::config::PackageToolsConfig;
use crate::error::{ChangesError, ChangesResult};

use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{
    MonorepoDetector, MonorepoDetectorTrait, MonorepoKind, WorkspacePackage,
};

/// Main analyzer for detecting and analyzing changes in a workspace.
///
/// The `ChangesAnalyzer` is the primary entry point for all change analysis operations.
/// It integrates with Git to detect file changes, with the filesystem to read package
/// information, and with monorepo detection to understand the project structure.
///
/// # Architecture
///
/// - **Git Integration**: Uses `sublime_git_tools::Repo` for all Git operations
/// - **Monorepo Detection**: Uses `sublime_standard_tools::monorepo::MonorepoDetector`
/// - **Filesystem**: Uses `sublime_standard_tools::filesystem::AsyncFileSystem` trait
/// - **Configuration**: Loads settings from `PackageToolsConfig`
///
/// # Project Structure Support
///
/// - **Single Package**: Projects with one package.json at the root
/// - **Monorepo**: Projects with multiple packages in workspace structure
///   - npm workspaces
///   - yarn workspaces
///   - pnpm workspaces
///   - bun workspaces
///   - Lerna
///   - Other custom workspace configurations
///
/// # Examples
///
/// ## Basic initialization
///
/// ```rust,ignore
/// use sublime_pkg_tools::changes::ChangesAnalyzer;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_git_tools::Repo;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let git_repo = Repo::open(".")?;
/// let fs = FileSystemManager::new();
/// let config = PackageToolsConfig::default();
///
/// let analyzer = ChangesAnalyzer::new(
///     workspace_root,
///     git_repo,
///     fs,
///     config
/// ).await?;
///
/// println!("Analyzer initialized for workspace");
/// # Ok(())
/// # }
/// ```
///
/// ## With custom configuration
///
/// ```rust,ignore
/// use sublime_pkg_tools::changes::ChangesAnalyzer;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_git_tools::Repo;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let git_repo = Repo::open(".")?;
/// let fs = FileSystemManager::new();
///
/// // Load configuration from file
/// let config = PackageToolsConfig::default(); // In real usage, load from file
///
/// let analyzer = ChangesAnalyzer::new(
///     workspace_root,
///     git_repo,
///     fs,
///     config
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ChangesAnalyzer<F = FileSystemManager>
where
    F: AsyncFileSystem + Clone + Send + Sync + 'static,
{
    /// Root directory of the workspace being analyzed.
    workspace_root: PathBuf,

    /// Git repository instance for change detection.
    git_repo: Rc<Repo>,

    /// Monorepo detector for understanding project structure.
    monorepo_detector: MonorepoDetector<F>,

    /// Filesystem abstraction for file operations.
    fs: F,

    /// Configuration for changes analysis.
    config: PackageToolsConfig,

    /// Detected monorepo kind (if any).
    monorepo_kind: Option<MonorepoKind>,
}

impl ChangesAnalyzer<FileSystemManager> {
    /// Creates a new `ChangesAnalyzer` with the default filesystem.
    ///
    /// This constructor initializes the analyzer with a Git repository, validates
    /// the workspace, and detects the project structure (monorepo vs single-package).
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace to analyze
    /// * `git_repo` - Git repository instance
    /// * `fs` - Filesystem instance for file operations
    /// * `config` - Configuration for analysis behavior
    ///
    /// # Returns
    ///
    /// Returns a configured `ChangesAnalyzer` ready for change analysis operations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace root doesn't exist or is invalid
    /// - The Git repository is invalid or corrupted
    /// - Monorepo detection fails
    /// - Required configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let analyzer = ChangesAnalyzer::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config
    /// ).await?;
    ///
    /// println!("Analyzer ready for: {}", analyzer.workspace_root().display());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        workspace_root: PathBuf,
        git_repo: Repo,
        fs: FileSystemManager,
        config: PackageToolsConfig,
    ) -> ChangesResult<Self> {
        Self::new_impl(workspace_root, git_repo, fs, config).await
    }
}

impl<F> ChangesAnalyzer<F>
where
    F: AsyncFileSystem + Clone + Send + Sync + 'static,
{
    /// Creates a new `ChangesAnalyzer` with a custom filesystem implementation.
    ///
    /// This allows using mock filesystems for testing or custom implementations
    /// for specialized use cases.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace to analyze
    /// * `git_repo` - Git repository instance
    /// * `fs` - Custom filesystem implementation
    /// * `config` - Configuration for analysis behavior
    ///
    /// # Returns
    ///
    /// Returns a configured `ChangesAnalyzer` with the custom filesystem.
    ///
    /// # Errors
    ///
    /// Same as `new()`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use std::path::PathBuf;
    ///
    /// # async fn example<F>(fs: F) -> Result<(), Box<dyn std::error::Error>>
    /// # where F: AsyncFileSystem + Clone + Send + Sync + 'static
    /// # {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    ///
    /// let analyzer = ChangesAnalyzer::with_filesystem(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_filesystem(
        workspace_root: PathBuf,
        git_repo: Repo,
        fs: F,
        config: PackageToolsConfig,
    ) -> ChangesResult<Self> {
        Self::new_impl(workspace_root, git_repo, fs, config).await
    }

    /// Internal implementation for creating a new analyzer.
    ///
    /// This method performs all the validation and initialization:
    /// 1. Validates the workspace root exists and is a directory
    /// 2. Validates the Git repository is accessible
    /// 3. Initializes the monorepo detector
    /// 4. Detects the project structure
    ///
    /// # Errors
    ///
    /// Returns `ChangesError` variants for various failure conditions.
    async fn new_impl(
        workspace_root: PathBuf,
        git_repo: Repo,
        fs: F,
        config: PackageToolsConfig,
    ) -> ChangesResult<Self> {
        // Validate workspace root exists
        Self::validate_workspace_root(&workspace_root, &fs).await?;

        // Validate Git repository
        Self::validate_git_repo(&git_repo, &workspace_root)?;

        // Initialize monorepo detector
        let monorepo_detector = MonorepoDetector::with_filesystem_and_config(
            fs.clone(),
            config.get_standard_config().monorepo.clone(),
        );

        // Detect monorepo structure
        let monorepo_kind = Self::detect_monorepo_kind(&monorepo_detector, &workspace_root).await?;

        Ok(Self {
            workspace_root,
            git_repo: Rc::new(git_repo),
            monorepo_detector,
            fs,
            config,
            monorepo_kind,
        })
    }

    /// Validates that the workspace root is a valid directory.
    ///
    /// # Errors
    ///
    /// Returns `InvalidWorkspaceRoot` if the path doesn't exist, is not a directory,
    /// or cannot be accessed.
    async fn validate_workspace_root(workspace_root: &Path, fs: &F) -> ChangesResult<()> {
        if !fs.exists(workspace_root).await {
            return Err(ChangesError::InvalidWorkspaceRoot {
                path: workspace_root.to_path_buf(),
                reason: "Directory does not exist".to_string(),
            });
        }

        // Check if it's a directory by checking metadata
        match fs.metadata(workspace_root).await {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    return Err(ChangesError::InvalidWorkspaceRoot {
                        path: workspace_root.to_path_buf(),
                        reason: "Path is not a directory".to_string(),
                    });
                }
            }
            Err(e) => {
                return Err(ChangesError::InvalidWorkspaceRoot {
                    path: workspace_root.to_path_buf(),
                    reason: format!("Cannot read metadata: {}", e),
                });
            }
        }

        Ok(())
    }

    /// Validates that the Git repository is accessible and valid.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryNotFound` if the Git repository cannot be accessed or is invalid.
    fn validate_git_repo(git_repo: &Repo, workspace_root: &Path) -> ChangesResult<()> {
        // Verify we can access the repository path
        let repo_path = git_repo.get_repo_path();
        if repo_path.as_os_str().is_empty() {
            return Err(ChangesError::RepositoryNotFound { path: workspace_root.to_path_buf() });
        }

        Ok(())
    }

    /// Detects the monorepo kind for the workspace.
    ///
    /// This method checks if the workspace is a monorepo and what kind of
    /// monorepo structure it uses (npm workspaces, yarn, pnpm, etc.).
    ///
    /// # Errors
    ///
    /// Returns `MonorepoDetectionFailed` if detection encounters errors.
    async fn detect_monorepo_kind(
        monorepo_detector: &MonorepoDetector<F>,
        workspace_root: &Path,
    ) -> ChangesResult<Option<MonorepoKind>> {
        match monorepo_detector.is_monorepo_root(workspace_root).await {
            Ok(kind) => Ok(kind),
            Err(e) => Err(ChangesError::MonorepoDetectionFailed {
                reason: format!("Failed to detect monorepo: {}", e),
            }),
        }
    }

    /// Returns the workspace root directory.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) {
    /// let root = analyzer.workspace_root();
    /// println!("Workspace: {}", root.display());
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
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = analyzer.git_repo();
    /// let current_branch = repo.get_current_branch()?;
    /// println!("Current branch: {}", current_branch);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn git_repo(&self) -> &Repo {
        &self.git_repo
    }

    /// Returns the detected monorepo kind, if any.
    ///
    /// Returns `None` for single-package projects or if monorepo detection
    /// did not identify a specific monorepo structure.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) {
    /// if let Some(kind) = analyzer.monorepo_kind() {
    ///     println!("Detected monorepo: {}", kind.name());
    /// } else {
    ///     println!("Single-package project");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn monorepo_kind(&self) -> Option<&MonorepoKind> {
        self.monorepo_kind.as_ref()
    }

    /// Returns whether this workspace is a monorepo.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) {
    /// if analyzer.is_monorepo() {
    ///     println!("Analyzing monorepo workspace");
    /// } else {
    ///     println!("Analyzing single-package project");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.monorepo_kind.is_some()
    }

    /// Returns a reference to the monorepo detector.
    ///
    /// This allows access to advanced monorepo operations if needed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) -> Result<(), Box<dyn std::error::Error>> {
    /// let detector = analyzer.monorepo_detector();
    /// let has_multiple = detector.has_multiple_packages(analyzer.workspace_root()).await;
    /// println!("Has multiple packages: {}", has_multiple);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn monorepo_detector(&self) -> &MonorepoDetector<F> {
        &self.monorepo_detector
    }

    /// Returns a reference to the filesystem.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = analyzer.filesystem();
    /// let package_json = analyzer.workspace_root().join("package.json");
    /// if fs.exists(&package_json).await {
    ///     println!("package.json found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn filesystem(&self) -> &F {
        &self.fs
    }

    /// Returns a reference to the configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// # async fn example(analyzer: &ChangesAnalyzer) {
    /// let config = analyzer.config();
    /// println!("Version strategy: {:?}", config.version.strategy);
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }

    /// Analyzes changes in the working directory (uncommitted changes).
    ///
    /// Detects all uncommitted changes (working tree + staging area) and maps them
    /// to affected packages. This method does not include commit information, as the
    /// changes have not been committed yet.
    ///
    /// # Returns
    ///
    /// Returns a `ChangesReport` containing all affected packages and their changes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git status cannot be retrieved
    /// - File-to-package mapping fails
    /// - Package information cannot be loaded
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let analyzer = ChangesAnalyzer::with_filesystem(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config
    /// ).await?;
    ///
    /// let report = analyzer.analyze_working_directory().await?;
    ///
    /// for package in report.packages_with_changes() {
    ///     println!("Package {} has {} files changed",
    ///         package.package_name(),
    ///         package.files.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_working_directory(&self) -> ChangesResult<crate::changes::ChangesReport> {
        use crate::changes::{
            AnalysisMode, ChangesReport, FileChange, FileChangeType, PackageChanges,
        };
        use std::collections::HashMap;

        // Get git status for all files
        let status = self.git_repo.get_status_detailed().map_err(|e| ChangesError::GitError {
            operation: "get_status_detailed".to_string(),
            reason: format!("Failed to get git status: {}", e),
        })?;

        if status.is_empty() {
            // No changes detected, return empty report with all packages
            let mut report = ChangesReport::new(AnalysisMode::WorkingDirectory, self.is_monorepo());

            // Get all packages even if they have no changes
            let packages = self.get_all_packages().await?;
            for package_info in packages {
                report.add_package(PackageChanges::new(package_info));
            }

            return Ok(report);
        }

        // Convert git changed files to paths
        let changed_paths: Vec<PathBuf> = status.iter().map(|f| PathBuf::from(&f.path)).collect();

        // Create package mapper
        let mut package_mapper =
            PackageMapper::with_filesystem(self.workspace_root.clone(), self.fs.clone());

        // Map files to packages (not used directly but ensures cache is populated)
        let _files_by_package = package_mapper.map_files_to_packages(&changed_paths).await?;

        // Build file changes grouped by package
        let mut package_file_changes: HashMap<String, Vec<FileChange>> = HashMap::new();

        for git_file in &status {
            let file_path = PathBuf::from(&git_file.path);

            // Find which package this file belongs to
            if let Some(package_name) = package_mapper.find_package_for_file(&file_path).await? {
                // Get the package info to calculate relative path
                let all_pkgs = self.get_all_packages().await?;
                let package_info = all_pkgs.iter().find(|p| p.name == package_name);

                let package_relative_path = if let Some(pkg) = package_info {
                    file_path.strip_prefix(&pkg.location).unwrap_or(&file_path).to_path_buf()
                } else {
                    file_path.clone()
                };

                // Create FileChange
                let change_type = FileChangeType::from_git_status(&git_file.status);
                let file_change = FileChange::new(file_path, package_relative_path, change_type);

                package_file_changes.entry(package_name).or_default().push(file_change);
            }
        }

        // Get all packages to include those without changes
        let all_packages = self.get_all_packages().await?;

        // Create ChangesReport
        let mut report = ChangesReport::new(AnalysisMode::WorkingDirectory, self.is_monorepo());

        for package_info in all_packages {
            let mut package_changes = PackageChanges::new(package_info.clone());

            // Set current version from package info
            if let Ok(version) = crate::types::Version::parse(&package_info.version) {
                package_changes.current_version = Some(version);
            }

            // Add file changes if any
            if let Some(files) = package_file_changes.get(&package_info.name) {
                for file in files {
                    package_changes.add_file(file.clone());
                }
            }

            report.add_package(package_changes);
        }

        Ok(report)
    }

    /// Analyzes changes in a commit range between two Git references.
    ///
    /// Detects all commits and file changes between `from_ref` and `to_ref`, maps them
    /// to affected packages, and associates commits with the packages they affect.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting Git reference (commit, branch, tag)
    /// * `to_ref` - Ending Git reference (commit, branch, tag)
    ///
    /// # Returns
    ///
    /// Returns a `ChangesReport` containing all affected packages with their file changes
    /// and associated commits.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git references cannot be resolved
    /// - Commit range is invalid or empty
    /// - File-to-package mapping fails
    /// - Package information cannot be loaded
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let analyzer = ChangesAnalyzer::with_filesystem(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config
    /// ).await?;
    ///
    /// // Analyze commits between main and feature branch
    /// let report = analyzer.analyze_commit_range("main", "feature-branch").await?;
    ///
    /// for package in report.packages_with_changes() {
    ///     println!("Package {} has {} commits",
    ///         package.package_name(),
    ///         package.commits.len());
    ///
    ///     for commit in &package.commits {
    ///         println!("  {} - {}", commit.short_hash, commit.message);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_commit_range(
        &self,
        from_ref: &str,
        to_ref: &str,
    ) -> ChangesResult<crate::changes::ChangesReport> {
        use crate::changes::{
            ChangesReport, CommitInfo, FileChange, FileChangeType, PackageChanges,
        };
        use std::collections::{HashMap, HashSet};

        // Get commits in the range
        let commits = self.git_repo.get_commits_between(from_ref, to_ref, &None).map_err(|e| {
            ChangesError::GitError {
                operation: "get_commits_between".to_string(),
                reason: format!("Failed to get commits between {} and {}: {}", from_ref, to_ref, e),
            }
        })?;

        if commits.is_empty() {
            return Err(ChangesError::InvalidCommitRange {
                from: from_ref.to_string(),
                to: to_ref.to_string(),
                reason: "No commits found in range".to_string(),
            });
        }

        // Get changed files between the two refs
        let changed_files =
            self.git_repo.get_files_changed_between(from_ref, to_ref).map_err(|e| {
                ChangesError::GitError {
                    operation: "get_files_changed_between".to_string(),
                    reason: format!("Failed to get changed files: {}", e),
                }
            })?;

        if changed_files.is_empty() {
            return Err(ChangesError::NoChangesDetected {
                scope: format!("commit range {} to {}", from_ref, to_ref),
            });
        }

        // Convert to PathBuf for processing
        let changed_paths: Vec<PathBuf> =
            changed_files.iter().map(|f| PathBuf::from(&f.path)).collect();

        // Create package mapper
        let mut package_mapper =
            PackageMapper::with_filesystem(self.workspace_root.clone(), self.fs.clone());

        // Map files to packages
        let files_by_package = package_mapper.map_files_to_packages(&changed_paths).await?;

        // Build file changes grouped by package with detailed info
        let mut package_file_changes: HashMap<String, Vec<FileChange>> = HashMap::new();

        for git_file in &changed_files {
            let file_path = PathBuf::from(&git_file.path);

            // Find which package this file belongs to
            if let Some(package_name) = package_mapper.find_package_for_file(&file_path).await? {
                // Get the package info to calculate relative path
                let all_pkgs = self.get_all_packages().await?;
                let package_info = all_pkgs.iter().find(|p| p.name == package_name);

                let package_relative_path = if let Some(pkg) = package_info {
                    file_path.strip_prefix(&pkg.location).unwrap_or(&file_path).to_path_buf()
                } else {
                    file_path.clone()
                };

                // Create FileChange
                let change_type = FileChangeType::from_git_status(&git_file.status);
                let mut file_change =
                    FileChange::new(file_path.clone(), package_relative_path, change_type);

                // Note: Line statistics would require diff analysis which is not in scope for this story
                // They will remain None for commit range analysis
                file_change.lines_added = None;
                file_change.lines_deleted = None;

                package_file_changes.entry(package_name).or_default().push(file_change);
            }
        }

        // For each commit, determine which packages it affects based on the files
        // that were already determined to be in the range
        let mut commits_by_package: HashMap<String, HashSet<String>> = HashMap::new();

        for repo_commit in &commits {
            // For each package with changes, check if any of its changed files
            // could have been affected by this commit.
            // Since we don't have a way to get files per commit easily,
            // we'll associate all commits with all packages that have changes.
            // This is conservative but correct for the commit range.
            for package_name in files_by_package.keys() {
                commits_by_package
                    .entry(package_name.clone())
                    .or_default()
                    .insert(repo_commit.hash.clone());
            }
        }

        // Convert commits to CommitInfo and organize by package
        let mut commit_info_by_package: HashMap<String, Vec<CommitInfo>> = HashMap::new();

        for (package_name, commit_hashes) in &commits_by_package {
            let mut package_commits = Vec::new();

            for repo_commit in &commits {
                if commit_hashes.contains(&repo_commit.hash) {
                    // Build list of affected packages for this commit
                    let affected_packages: Vec<String> = commits_by_package
                        .iter()
                        .filter(|(_, hashes)| hashes.contains(&repo_commit.hash))
                        .map(|(name, _)| name.clone())
                        .collect();

                    // Create CommitInfo
                    let mut commit_info =
                        CommitInfo::from_git_commit(repo_commit, affected_packages);

                    // Set files_changed to the number of files in the entire range
                    // This is an approximation since we don't have per-commit file info
                    commit_info.files_changed = changed_files.len();
                    // Line statistics would require diff parsing, left as 0 for now
                    commit_info.lines_added = 0;
                    commit_info.lines_deleted = 0;

                    package_commits.push(commit_info);
                }
            }

            commit_info_by_package.insert(package_name.clone(), package_commits);
        }

        // Add commit hashes to file changes
        // Since we associate all commits with all files in a package, add all commit hashes
        for (package_name, files) in package_file_changes.iter_mut() {
            if let Some(commit_hashes) = commits_by_package.get(package_name) {
                for file in files {
                    file.commits = commit_hashes.iter().cloned().collect();
                }
            }
        }

        // Get all packages to include those without changes
        let all_packages = self.get_all_packages().await?;

        // Create ChangesReport
        let mut report = ChangesReport::new_for_range(from_ref, to_ref, self.is_monorepo());

        for package_info in all_packages {
            let mut package_changes = PackageChanges::new(package_info.clone());

            // Set current version from package info
            if let Ok(version) = crate::types::Version::parse(&package_info.version) {
                package_changes.current_version = Some(version);
            }

            // Add file changes if any
            if let Some(files) = package_file_changes.get(&package_info.name) {
                for file in files {
                    package_changes.add_file(file.clone());
                }
            }

            // Add commits if any
            if let Some(commits) = commit_info_by_package.get(&package_info.name) {
                for commit in commits {
                    package_changes.add_commit(commit.clone());
                }
            }

            report.add_package(package_changes);
        }

        Ok(report)
    }

    /// Analyzes changes with version preview calculation.
    ///
    /// This method performs commit range analysis and enhances the report with next version
    /// calculations based on the provided changeset. Only packages listed in the changeset
    /// will have their versions calculated.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting Git reference (commit, branch, tag)
    /// * `to_ref` - Ending Git reference (commit, branch, tag)
    /// * `changeset` - Changeset containing bump type and affected packages
    ///
    /// # Returns
    ///
    /// Returns a `ChangesReport` with `next_version` and `bump_type` populated for
    /// packages that are included in the changeset.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git references cannot be resolved
    /// - Commit range analysis fails
    /// - Version parsing or bumping fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::ChangesAnalyzer;
    /// use sublime_pkg_tools::types::{Changeset, VersionBump};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let git_repo = Repo::open(".")?;
    /// let fs = FileSystemManager::new();
    /// let config = PackageToolsConfig::default();
    ///
    /// let analyzer = ChangesAnalyzer::new(
    ///     workspace_root,
    ///     git_repo,
    ///     fs,
    ///     config
    /// ).await?;
    ///
    /// // Create or load a changeset
    /// let changeset = Changeset::new(
    ///     "feature-branch",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// // Analyze with version preview
    /// let report = analyzer.analyze_with_versions(
    ///     "main",
    ///     "HEAD",
    ///     &changeset
    /// ).await?;
    ///
    /// for package in report.packages_with_changes() {
    ///     if let (Some(current), Some(next)) = (&package.current_version, &package.next_version) {
    ///         println!("Package {} will be bumped from {} to {}",
    ///             package.package_name(),
    ///             current,
    ///             next);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_with_versions(
        &self,
        from_ref: &str,
        to_ref: &str,
        changeset: &crate::types::Changeset,
    ) -> ChangesResult<crate::changes::ChangesReport> {
        // First, perform the regular commit range analysis
        let mut report = self.analyze_commit_range(from_ref, to_ref).await?;

        // Add version information to packages that are in the changeset
        for package_changes in &mut report.packages {
            // Only calculate versions for packages included in the changeset
            if changeset.packages.contains(&package_changes.package_name) {
                self.add_version_info(package_changes, changeset)?;
            }
        }

        Ok(report)
    }

    /// Calculates the next version for a package based on the bump type.
    ///
    /// This is a helper method that applies the version bump to a current version.
    /// It uses the `Version::bump` method which already handles all bump types correctly.
    ///
    /// # Arguments
    ///
    /// * `current_version` - The current version to bump
    /// * `bump` - The type of version bump to apply
    ///
    /// # Returns
    ///
    /// Returns the next version after applying the bump.
    ///
    /// # Errors
    ///
    /// Returns an error if the version bump fails (e.g., integer overflow).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::types::{Version, VersionBump};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let current = Version::parse("1.2.3")?;
    /// let next = current.bump(VersionBump::Minor)?;
    /// assert_eq!(next.to_string(), "1.3.0");
    /// # Ok(())
    /// # }
    /// ```
    fn calculate_next_version(
        &self,
        current_version: &crate::types::Version,
        bump: crate::types::VersionBump,
    ) -> ChangesResult<crate::types::Version> {
        current_version.bump(bump).map_err(|e| ChangesError::VersionCalculationFailed {
            package: String::from("unknown"), // Will be set by caller
            current_version: current_version.to_string(),
            bump_type: format!("{:?}", bump),
            reason: e.to_string(),
        })
    }

    /// Adds version information to a package changes instance.
    ///
    /// This method populates the `next_version` and `bump_type` fields of a
    /// `PackageChanges` based on the changeset's bump type. It requires that
    /// the package already has a `current_version` set.
    ///
    /// # Arguments
    ///
    /// * `package_changes` - The package changes to update with version info
    /// * `changeset` - The changeset containing the bump type
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The package doesn't have a current version
    /// - Version bumping fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changes::PackageChanges;
    /// use sublime_pkg_tools::types::{Changeset, Version, VersionBump};
    /// use sublime_standard_tools::monorepo::WorkspacePackage;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_pkg = WorkspacePackage {
    ///     name: "@myorg/core".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     location: PathBuf::from("packages/core"),
    ///     absolute_path: PathBuf::from("/workspace/packages/core"),
    ///     workspace_dependencies: Vec::new(),
    ///     workspace_dev_dependencies: Vec::new(),
    /// };
    ///
    /// let mut package_changes = PackageChanges::new(workspace_pkg);
    /// package_changes.current_version = Some(Version::parse("1.0.0")?);
    ///
    /// let changeset = Changeset::new(
    ///     "feature-branch",
    ///     VersionBump::Minor,
    ///     vec!["production".to_string()],
    /// );
    ///
    /// // add_version_info(&mut package_changes, &changeset)?;
    ///
    /// assert_eq!(
    ///     package_changes.next_version.as_ref().map(|v| v.to_string()),
    ///     Some("1.1.0".to_string())
    /// );
    /// assert_eq!(package_changes.bump_type, Some(VersionBump::Minor));
    /// # Ok(())
    /// # }
    /// ```
    fn add_version_info(
        &self,
        package_changes: &mut crate::changes::PackageChanges,
        changeset: &crate::types::Changeset,
    ) -> ChangesResult<()> {
        if let Some(current) = &package_changes.current_version {
            // Calculate next version based on changeset bump
            let next = self.calculate_next_version(current, changeset.bump).map_err(|e| {
                // Enhance error with package name
                if let ChangesError::VersionCalculationFailed {
                    package: _,
                    current_version,
                    bump_type,
                    reason,
                } = e
                {
                    ChangesError::VersionCalculationFailed {
                        package: package_changes.package_name.clone(),
                        current_version,
                        bump_type,
                        reason,
                    }
                } else {
                    e
                }
            })?;

            package_changes.next_version = Some(next);
            package_changes.bump_type = Some(changeset.bump);
        } else {
            // If there's no current version, we can't calculate next version
            // This is not necessarily an error - some packages might not have versions yet
            // We'll just skip setting the next_version
        }

        Ok(())
    }

    /// Gets all packages in the workspace.
    ///
    /// Returns package information for all packages, regardless of whether they
    /// have changes or not.
    ///
    /// # Errors
    ///
    /// Returns an error if package discovery fails or package.json files cannot be read.
    async fn get_all_packages(&self) -> ChangesResult<Vec<WorkspacePackage>> {
        // Check if we have cached monorepo info
        let packages = if let Some(_monorepo_kind) = &self.monorepo_kind {
            // It's a monorepo, get all packages from monorepo detector
            self.monorepo_detector.detect_packages(&self.workspace_root).await.map_err(|e| {
                ChangesError::MonorepoDetectionFailed {
                    reason: format!("Failed to detect workspace packages: {}", e),
                }
            })?
        } else {
            // Single package, create a WorkspacePackage for the root
            let package_json_path = self.workspace_root.join("package.json");

            let content_result = self.fs.read_file_string(&package_json_path).await;

            if content_result.is_err() {
                return Err(ChangesError::NoPackagesFound {
                    workspace_root: self.workspace_root.clone(),
                });
            }

            let content = content_result.map_err(|e| ChangesError::FileSystemError {
                path: package_json_path.clone(),
                reason: format!("Failed to read package.json: {}", e),
            })?;

            let package_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                ChangesError::PackageJsonParseError {
                    path: package_json_path.clone(),
                    reason: format!("Failed to parse package.json: {}", e),
                }
            })?;

            let name = package_json["name"].as_str().unwrap_or("root").to_string();
            let version = package_json["version"].as_str().unwrap_or("0.0.0").to_string();

            // Create WorkspacePackage manually since new() might not exist
            let package = WorkspacePackage {
                name,
                version,
                location: PathBuf::from("."),
                absolute_path: self.workspace_root.clone(),
                workspace_dependencies: Vec::new(),
                workspace_dev_dependencies: Vec::new(),
            };
            vec![package]
        };

        Ok(packages)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use sublime_git_tools::Repo;
    use sublime_standard_tools::filesystem::FileSystemManager;
    use tempfile::TempDir;

    /// Helper to create a temporary Git repository for testing.
    async fn create_test_git_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repository
        Repo::create(repo_path.to_str().unwrap()).unwrap();

        (temp_dir, repo_path)
    }

    /// Helper to create a test workspace with package.json.
    #[allow(dead_code)]
    async fn create_test_workspace(with_package_json: bool) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        if with_package_json {
            let package_json = r#"{
                "name": "test-package",
                "version": "1.0.0"
            }"#;
            fs::write(workspace_path.join("package.json"), package_json).unwrap();
        }

        (temp_dir, workspace_path)
    }

    /// Helper to create a monorepo workspace structure.
    async fn create_test_monorepo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        // Root package.json with workspaces
        let root_package_json = r#"{
            "name": "test-monorepo",
            "version": "1.0.0",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(workspace_path.join("package.json"), root_package_json).unwrap();

        // Create pnpm-workspace.yaml for proper monorepo detection
        let pnpm_workspace = "packages:\n  - 'packages/*'\n";
        fs::write(workspace_path.join("pnpm-workspace.yaml"), pnpm_workspace).unwrap();

        // Create packages directory
        fs::create_dir_all(workspace_path.join("packages/pkg-a")).unwrap();
        fs::create_dir_all(workspace_path.join("packages/pkg-b")).unwrap();

        // Package A
        let pkg_a_json = r#"{
            "name": "@test/pkg-a",
            "version": "1.0.0"
        }"#;
        fs::write(workspace_path.join("packages/pkg-a/package.json"), pkg_a_json).unwrap();

        // Package B
        let pkg_b_json = r#"{
            "name": "@test/pkg-b",
            "version": "1.0.0"
        }"#;
        fs::write(workspace_path.join("packages/pkg-b/package.json"), pkg_b_json).unwrap();

        (temp_dir, workspace_path)
    }

    #[tokio::test]
    async fn test_new_with_valid_single_package_repo() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let result = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await;

        assert!(result.is_ok());
        let analyzer = result.unwrap();
        assert_eq!(analyzer.workspace_root(), repo_path.as_path());
        assert!(!analyzer.is_monorepo());
        assert!(analyzer.monorepo_kind().is_none());
    }

    #[tokio::test]
    async fn test_new_with_valid_monorepo() {
        let (_temp, workspace_path) = create_test_monorepo().await;

        // Initialize git in the workspace
        Repo::create(workspace_path.to_str().unwrap()).unwrap();
        let git_repo = Repo::open(workspace_path.to_str().unwrap()).unwrap();

        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let result = ChangesAnalyzer::new(workspace_path.clone(), git_repo, fs, config).await;

        assert!(result.is_ok());
        let analyzer = result.unwrap();
        assert_eq!(analyzer.workspace_root(), workspace_path.as_path());
        assert!(analyzer.is_monorepo());
        assert!(analyzer.monorepo_kind().is_some());
    }

    #[tokio::test]
    async fn test_new_with_nonexistent_workspace() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/to/workspace");
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let result = ChangesAnalyzer::new(nonexistent_path.clone(), git_repo, fs, config).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ChangesError::InvalidWorkspaceRoot { .. }));
        assert!(err.as_ref().contains("invalid workspace root"));
    }

    #[tokio::test]
    async fn test_new_with_file_instead_of_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test content").unwrap();

        let (_temp_git, repo_path) = create_test_git_repo().await;
        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let result = ChangesAnalyzer::new(file_path.clone(), git_repo, fs, config).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ChangesError::InvalidWorkspaceRoot { .. }));
    }

    #[tokio::test]
    async fn test_workspace_root_accessor() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await.unwrap();

        assert_eq!(analyzer.workspace_root(), repo_path.as_path());
    }

    #[tokio::test]
    async fn test_git_repo_accessor() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await.unwrap();

        let repo = analyzer.git_repo();
        // Verify we can use the repo
        let repo_path_from_accessor = repo.get_repo_path();
        assert!(!repo_path_from_accessor.as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_is_monorepo_single_package() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path, git_repo, fs, config).await.unwrap();

        assert!(!analyzer.is_monorepo());
    }

    #[tokio::test]
    async fn test_is_monorepo_with_workspaces() {
        let (_temp, workspace_path) = create_test_monorepo().await;
        Repo::create(workspace_path.to_str().unwrap()).unwrap();
        let git_repo = Repo::open(workspace_path.to_str().unwrap()).unwrap();

        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(workspace_path, git_repo, fs, config).await.unwrap();

        assert!(analyzer.is_monorepo());
    }

    #[tokio::test]
    async fn test_monorepo_kind_accessor() {
        let (_temp, workspace_path) = create_test_monorepo().await;
        Repo::create(workspace_path.to_str().unwrap()).unwrap();
        let git_repo = Repo::open(workspace_path.to_str().unwrap()).unwrap();

        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(workspace_path, git_repo, fs, config).await.unwrap();

        let kind = analyzer.monorepo_kind();
        assert!(kind.is_some());
    }

    #[tokio::test]
    async fn test_monorepo_detector_accessor() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await.unwrap();

        let detector = analyzer.monorepo_detector();
        let has_multiple = detector.has_multiple_packages(repo_path.as_path()).await;
        assert!(!has_multiple);
    }

    #[tokio::test]
    async fn test_filesystem_accessor() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await.unwrap();

        let filesystem = analyzer.filesystem();
        let package_json_path = repo_path.join("package.json");
        assert!(filesystem.exists(&package_json_path).await);
    }

    #[tokio::test]
    async fn test_config_accessor() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path, git_repo, fs, config).await.unwrap();

        let config = analyzer.config();
        assert_eq!(config.changeset.path, ".changesets");
    }

    #[tokio::test]
    async fn test_with_filesystem_custom() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let result =
            ChangesAnalyzer::with_filesystem(repo_path.clone(), git_repo, fs, config).await;

        assert!(result.is_ok());
        let analyzer = result.unwrap();
        assert_eq!(analyzer.workspace_root(), repo_path.as_path());
    }

    #[tokio::test]
    async fn test_analyzer_clone() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs = FileSystemManager::new();
        let config = PackageToolsConfig::default();

        let analyzer = ChangesAnalyzer::new(repo_path.clone(), git_repo, fs, config).await.unwrap();

        let cloned_analyzer = analyzer.clone();
        assert_eq!(cloned_analyzer.workspace_root(), analyzer.workspace_root());
        assert_eq!(cloned_analyzer.is_monorepo(), analyzer.is_monorepo());
    }

    #[tokio::test]
    async fn test_validate_workspace_root_with_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        let fs = FileSystemManager::new();

        let result =
            ChangesAnalyzer::<FileSystemManager>::validate_workspace_root(&workspace_path, &fs)
                .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_workspace_root_with_nonexistent_path() {
        let nonexistent = PathBuf::from("/this/does/not/exist");
        let fs = FileSystemManager::new();

        let result =
            ChangesAnalyzer::<FileSystemManager>::validate_workspace_root(&nonexistent, &fs).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ChangesError::InvalidWorkspaceRoot { .. }));
    }

    #[tokio::test]
    async fn test_validate_git_repo_with_valid_repo() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let git_repo = Repo::open(repo_path.to_str().unwrap()).unwrap();

        let result = ChangesAnalyzer::<FileSystemManager>::validate_git_repo(&git_repo, &repo_path);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_analyzer_instances() {
        let (_temp_git, repo_path) = create_test_git_repo().await;
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(repo_path.join("package.json"), package_json).unwrap();

        let git_repo1 = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let git_repo2 = Repo::open(repo_path.to_str().unwrap()).unwrap();
        let fs1 = FileSystemManager::new();
        let fs2 = FileSystemManager::new();
        let config1 = PackageToolsConfig::default();
        let config2 = PackageToolsConfig::default();

        let analyzer1 =
            ChangesAnalyzer::new(repo_path.clone(), git_repo1, fs1, config1).await.unwrap();

        let analyzer2 =
            ChangesAnalyzer::new(repo_path.clone(), git_repo2, fs2, config2).await.unwrap();

        assert_eq!(analyzer1.workspace_root(), analyzer2.workspace_root());
        assert_eq!(analyzer1.is_monorepo(), analyzer2.is_monorepo());
    }
}
