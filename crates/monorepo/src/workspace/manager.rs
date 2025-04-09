//! Main entry point for workspace operations.
//!
//! This module provides the `WorkspaceManager` class, which is the primary
//! interface for creating, loading, and analyzing workspaces.

use crate::{DiscoveryOptions, Workspace, WorkspaceAnalysis, WorkspaceConfig, WorkspaceError};
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;

/// Main entry point for workspace operations.
///
/// The `WorkspaceManager` provides high-level functions for discovering,
/// loading, and analyzing monorepo workspaces.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a workspace manager
/// let manager = WorkspaceManager::new();
///
/// // Discover a workspace
/// let options = DiscoveryOptions::default();
/// let workspace = manager.discover_workspace(".", &options)?;
///
/// // Analyze the workspace
/// let analysis = manager.analyze_workspace(&workspace)?;
///
/// // Check for cycles
/// if !analysis.cycles.is_empty() {
///     println!("Found {} dependency cycles", analysis.cycles.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct WorkspaceManager {}

impl WorkspaceManager {
    /// Creates a new workspace manager.
    ///
    /// # Returns
    ///
    /// A new workspace manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::WorkspaceManager;
    ///
    /// let manager = WorkspaceManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Discovers a workspace from a directory.
    ///
    /// Finds and loads packages in a workspace according to the provided options.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the workspace root or any directory within it
    /// * `options` - Discovery options
    ///
    /// # Returns
    ///
    /// The discovered workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if workspace discovery fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = WorkspaceManager::new();
    ///
    /// // Discover with default options
    /// let workspace = manager.discover_workspace(".", &DiscoveryOptions::default())?;
    ///
    /// // Discover with custom options
    /// let options = DiscoveryOptions::new()
    ///     .include_patterns(vec!["packages/*/package.json"])
    ///     .exclude_patterns(vec!["**/node_modules/**"]);
    /// let workspace = manager.discover_workspace(".", &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_workspace(
        &self,
        path: impl AsRef<Path>,
        options: &DiscoveryOptions,
    ) -> Result<Workspace, WorkspaceError> {
        // Use sublime_standard_tools to find project root if needed
        let root_path = if options.auto_detect_root {
            sublime_standard_tools::get_project_root_path(Some(PathBuf::from(path.as_ref())))
                .ok_or(WorkspaceError::RootNotFound)?
        } else {
            PathBuf::from(path.as_ref())
        };

        // Detect package manager
        let package_manager = if options.detect_package_manager {
            sublime_standard_tools::detect_package_manager(&root_path)
        } else {
            None
        };

        // Create workspace config
        let config = WorkspaceConfig::new(root_path.clone())
            .with_package_manager(package_manager.map(|pm| pm.to_string()));

        // Try to open git repository
        let git_repo = Repo::open(root_path.to_str().unwrap_or(".")).ok();

        // Create and initialize the workspace
        let mut workspace = Workspace::new(root_path, config, git_repo)?;

        // Discover packages using the provided options
        workspace.discover_packages_with_options(options)?;

        // Make sure we have at least one package
        if workspace.is_empty() {
            return Err(WorkspaceError::NoPackagesFound(workspace.root_path().to_path_buf()));
        }

        Ok(workspace)
    }

    /// Loads a workspace from explicit configuration.
    ///
    /// Creates a workspace with the provided configuration and loads
    /// packages according to default discovery options.
    ///
    /// # Arguments
    ///
    /// * `config` - Workspace configuration
    ///
    /// # Returns
    ///
    /// The loaded workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if workspace loading fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::{WorkspaceConfig, WorkspaceManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = WorkspaceManager::new();
    ///
    /// // Create workspace config
    /// let config = WorkspaceConfig::new(PathBuf::from("."))
    ///     .with_packages(vec!["packages/*", "apps/*"]);
    ///
    /// // Load workspace from config
    /// let workspace = manager.load_workspace(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_workspace(&self, config: WorkspaceConfig) -> Result<Workspace, WorkspaceError> {
        let root_path = config.root_path.clone();

        // Try to open git repository
        let git_repo = Repo::open(root_path.to_str().unwrap_or(".")).ok();

        // Create workspace
        let mut workspace = Workspace::new(root_path, config, git_repo)?;

        // Discover packages with default options
        workspace.discover_packages_with_options(&DiscoveryOptions::default())?;

        if workspace.is_empty() {
            return Err(WorkspaceError::NoPackagesFound(workspace.root_path().to_path_buf()));
        }

        Ok(workspace)
    }

    /// Analyzes a workspace for issues.
    ///
    /// Analyzes the dependency structure of a workspace, detecting
    /// cycles, external dependencies, and version conflicts.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace to analyze
    ///
    /// # Returns
    ///
    /// Analysis results.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = WorkspaceManager::new();
    /// let workspace = manager.discover_workspace(".", &DiscoveryOptions::default())?;
    ///
    /// // Analyze the workspace
    /// let analysis = manager.analyze_workspace(&workspace)?;
    ///
    /// // Check for issues
    /// if analysis.validation_issues {
    ///     println!("Workspace has validation issues");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_workspace(
        &self,
        workspace: &Workspace,
    ) -> Result<WorkspaceAnalysis, WorkspaceError> {
        // Analyze dependencies
        let dependency_analysis = workspace.analyze_dependencies()?;

        // Validate workspace with default options
        let validation = workspace.validate()?;

        // Create analysis result
        let analysis = WorkspaceAnalysis {
            cycles: dependency_analysis.cycles,
            external_dependencies: dependency_analysis.external_dependencies,
            version_conflicts: dependency_analysis.version_conflicts,
            validation_issues: validation.has_issues(),
        };

        Ok(analysis)
    }
}
