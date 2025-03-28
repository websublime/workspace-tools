use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;

use crate::{DiscoveryOptions, Workspace, WorkspaceAnalysis, WorkspaceConfig, WorkspaceError};

/// Main entry point for workspace operations.
#[derive(Debug, Default)]
pub struct WorkspaceManager {}

impl WorkspaceManager {
    /// Creates a new workspace manager.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Discovers a workspace from a directory.
    ///
    /// # Errors
    /// Returns an error if workspace discovery fails.
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

        if workspace.is_empty() {
            return Err(WorkspaceError::NoPackagesFound(workspace.root_path().to_path_buf()));
        }

        Ok(workspace)
    }

    /// Loads a workspace from explicit configuration.
    ///
    /// # Errors
    /// Returns an error if workspace loading fails.
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
    /// # Errors
    /// Returns an error if analysis fails.
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
            cycle_detected: dependency_analysis.cycles_detected,
            missing_dependencies: dependency_analysis.missing_dependencies,
            version_conflicts: dependency_analysis.version_conflicts,
            validation_issues: validation.has_issues(),
        };

        Ok(analysis)
    }
}
