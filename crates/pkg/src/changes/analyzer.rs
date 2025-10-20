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

use crate::config::PackageToolsConfig;
use crate::error::{ChangesError, ChangesResult};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait, MonorepoKind};

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
