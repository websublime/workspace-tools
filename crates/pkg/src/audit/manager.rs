//! Audit manager for orchestrating comprehensive repository and package audits.
//!
//! **What**: Provides the main `AuditManager` that coordinates all audit operations,
//! integrating upgrade detection, changes analysis, dependency graph construction,
//! and health scoring into a unified audit framework.
//!
//! **How**: The manager initializes with a workspace root and configuration, then
//! sets up all necessary subsystems (upgrade manager, changes analyzer, filesystem,
//! monorepo detector) to enable comprehensive auditing across all aspects of the
//! repository.
//!
//! **Why**: To provide a single entry point for all audit operations that handles
//! the complexity of coordinating multiple subsystems while presenting a clean,
//! simple API for users.

use crate::changes::ChangesAnalyzer;
use crate::config::PackageToolsConfig;
use crate::error::{AuditError, AuditResult};
use crate::upgrade::UpgradeManager;
use std::path::PathBuf;
use sublime_git_tools::Repo;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use sublime_standard_tools::monorepo::{MonorepoDetector, MonorepoDetectorTrait};

/// High-level manager for audit and health check operations.
///
/// `AuditManager` is the primary entry point for all audit operations. It orchestrates
/// the complete audit workflow by integrating:
/// - Upgrade manager for detecting available package updates
/// - Changes analyzer for analyzing file and commit changes
/// - Monorepo detector for understanding project structure
/// - Filesystem operations for reading package information
///
/// This provides a unified API that handles all complexity internally, including
/// configuration management, error handling, and coordination between subsystems.
///
/// # Architecture
///
/// The manager follows a composition pattern, aggregating functionality from:
/// - **UpgradeManager**: Detects available dependency upgrades
/// - **ChangesAnalyzer**: Analyzes changes in working directory and commit ranges
/// - **MonorepoDetector**: Detects and manages monorepo structures
/// - **FileSystemManager**: Provides filesystem operations
///
/// # Project Structure Support
///
/// - **Single Package**: Projects with one package.json at the root
/// - **Monorepo**: Projects with multiple packages in workspace structure
///
/// # Examples
///
/// ## Basic initialization
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::AuditManager;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
/// let config = PackageToolsConfig::default();
///
/// let manager = AuditManager::new(workspace_root, config).await?;
/// println!("Audit manager initialized");
/// # Ok(())
/// # }
/// ```
///
/// ## With custom configuration
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::AuditManager;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace_root = PathBuf::from(".");
///
/// // Load configuration from file or use custom settings
/// let mut config = PackageToolsConfig::default();
/// config.audit.enabled = true;
/// config.audit.min_severity = "warning".to_string();
///
/// let manager = AuditManager::new(workspace_root, config).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Future usage (to be implemented in subsequent stories)
///
/// ```rust,ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # use sublime_pkg_tools::audit::AuditManager;
/// # use sublime_pkg_tools::config::PackageToolsConfig;
/// # use std::path::PathBuf;
/// # let workspace_root = PathBuf::from(".");
/// # let config = PackageToolsConfig::default();
/// let manager = AuditManager::new(workspace_root, config).await?;
///
/// // Run complete audit (TODO: will be implemented in story 10.2+)
/// // let report = manager.run_audit().await?;
/// // println!("Health score: {}/100", report.health_score);
/// # Ok(())
/// # }
/// ```
pub struct AuditManager {
    /// Root directory of the workspace being audited.
    workspace_root: PathBuf,

    /// Upgrade manager for detecting available package updates.
    upgrade_manager: UpgradeManager,

    /// Changes analyzer for analyzing file and commit changes.
    changes_analyzer: ChangesAnalyzer<FileSystemManager>,

    /// Filesystem abstraction for file operations.
    fs: FileSystemManager,

    /// Monorepo detector for understanding project structure.
    monorepo_detector: MonorepoDetector<FileSystemManager>,

    /// Configuration for audit operations.
    config: PackageToolsConfig,
}

impl AuditManager {
    /// Creates a new `AuditManager` with the default filesystem.
    ///
    /// This constructor initializes the audit manager with all necessary subsystems:
    /// - Opens and validates the Git repository
    /// - Initializes the upgrade manager for dependency update detection
    /// - Sets up the changes analyzer for change detection
    /// - Configures the monorepo detector for project structure detection
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Root directory of the workspace to audit
    /// * `config` - Configuration for all audit operations
    ///
    /// # Returns
    ///
    /// Returns a configured `AuditManager` ready to perform audit operations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace root doesn't exist or is invalid
    /// - The Git repository cannot be opened or is corrupted
    /// - The upgrade manager cannot be initialized
    /// - The changes analyzer cannot be initialized
    /// - Any configuration validation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::audit::AuditManager;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    /// let config = PackageToolsConfig::default();
    ///
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With disabled auditing
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let workspace_root = PathBuf::from(".");
    ///
    /// let mut config = PackageToolsConfig::default();
    /// config.audit.enabled = false;
    ///
    /// // Manager still initializes, but audit operations will check the flag
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(workspace_root: PathBuf, config: PackageToolsConfig) -> AuditResult<Self> {
        let fs = FileSystemManager::new();
        // Validate workspace root exists
        if !fs.exists(&workspace_root).await {
            return Err(AuditError::InvalidWorkspaceRoot {
                path: workspace_root.clone(),
                reason: "Workspace root does not exist".to_string(),
            });
        }

        // Open Git repository
        let workspace_str =
            workspace_root.to_str().ok_or_else(|| AuditError::InvalidWorkspaceRoot {
                path: workspace_root.clone(),
                reason: "Workspace path contains invalid UTF-8".to_string(),
            })?;

        let git_repo = Repo::open(workspace_str).map_err(|e| AuditError::GitError {
            operation: "open repository".to_string(),
            reason: e.to_string(),
        })?;

        // Initialize monorepo detector
        let monorepo_detector = MonorepoDetector::new();

        // Detect if this is a monorepo to validate the structure
        let _is_monorepo =
            monorepo_detector.is_monorepo_root(&workspace_root).await.map_err(|e| {
                AuditError::WorkspaceAnalysisFailed {
                    reason: format!("Failed to detect monorepo structure: {}", e),
                }
            })?;

        // Initialize upgrade manager
        let upgrade_manager = UpgradeManager::new(workspace_root.clone(), config.upgrade.clone())
            .await
            .map_err(|e| AuditError::UpgradeDetectionFailed {
                reason: format!("Failed to initialize upgrade manager: {}", e),
            })?;

        // Initialize changes analyzer
        let changes_analyzer =
            ChangesAnalyzer::new(workspace_root.clone(), git_repo, fs.clone(), config.clone())
                .await
                .map_err(|e| AuditError::AnalysisFailed {
                    section: "changes".to_string(),
                    reason: format!("Failed to initialize changes analyzer: {}", e),
                })?;

        Ok(Self {
            workspace_root,
            upgrade_manager,
            changes_analyzer,
            fs,
            monorepo_detector,
            config,
        })
    }

    /// Returns a reference to the workspace root path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root.clone(), config).await?;
    /// assert_eq!(manager.workspace_root(), &workspace_root);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }

    /// Returns a reference to the configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// let audit_config = &manager.config().audit;
    /// assert!(audit_config.enabled);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }

    /// Returns a reference to the upgrade manager.
    ///
    /// This provides access to the underlying upgrade manager for direct
    /// upgrade detection operations if needed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// let upgrade_mgr = manager.upgrade_manager();
    /// // Use upgrade manager directly if needed
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn upgrade_manager(&self) -> &UpgradeManager {
        &self.upgrade_manager
    }

    /// Returns a reference to the changes analyzer.
    ///
    /// This provides access to the underlying changes analyzer for direct
    /// change analysis operations if needed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// let analyzer = manager.changes_analyzer();
    /// // Use changes analyzer directly if needed
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn changes_analyzer(&self) -> &ChangesAnalyzer<FileSystemManager> {
        &self.changes_analyzer
    }

    /// Returns a reference to the monorepo detector.
    ///
    /// This provides access to the underlying monorepo detector for direct
    /// project structure queries if needed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// let detector = manager.monorepo_detector();
    /// let is_monorepo = detector.is_monorepo().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn monorepo_detector(&self) -> &MonorepoDetector<FileSystemManager> {
        &self.monorepo_detector
    }

    /// Returns a reference to the filesystem.
    ///
    /// This provides access to the underlying filesystem implementation
    /// for direct file operations if needed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use sublime_pkg_tools::audit::AuditManager;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// # use std::path::PathBuf;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let workspace_root = PathBuf::from(".");
    /// # let config = PackageToolsConfig::default();
    /// let manager = AuditManager::new(workspace_root, config).await?;
    /// let fs = manager.filesystem();
    /// // Use filesystem directly if needed
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn filesystem(&self) -> &FileSystemManager {
        &self.fs
    }

    // Future audit methods will be implemented in subsequent stories:
    // - Story 10.2: audit_upgrades() -> UpgradeAuditSection
    // - Story 10.3: audit_dependencies() -> DependencyAuditSection
    // - Story 10.4: categorize_dependencies() -> DependencyCategorization
    // - Story 10.5: audit_breaking_changes() -> BreakingChangesAuditSection
    // - Story 10.6: audit_version_consistency() -> VersionConsistencyAuditSection
    // - Story 10.7: calculate_health_score() -> u8
    // - Story 10.8: run_audit() -> AuditReport
}
