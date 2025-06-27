//! MonorepoTools implementation - main orchestrator for monorepo functionality

use crate::analysis::{DiffAnalyzer, MonorepoAnalyzer};
use crate::core::{MonorepoProject, VersionManager, VersioningPlan, VersioningStrategy};
use crate::core::types::MonorepoTools;
use crate::error::{Error, Result};
use crate::hooks::HookManager;
use crate::tasks::TaskManager;
use crate::workflows::{DevelopmentResult, DevelopmentWorkflow};
use crate::workflows::{ChangeAnalysisWorkflowResult, VersioningWorkflowResult};
use crate::workflows::{ReleaseWorkflow, ReleaseResult, ReleaseOptions};
use crate::plugins::PluginManager;
use std::sync::Arc;

impl MonorepoTools {
    /// Initialize `MonorepoTools` by detecting and opening a monorepo at the given path
    ///
    /// This function detects the type of monorepo at the given path, loads its configuration,
    /// and initializes all necessary components for monorepo management.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the monorepo root directory
    ///
    /// # Returns
    ///
    /// A configured `MonorepoTools` instance ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist or is not accessible
    /// - No valid monorepo configuration is found
    /// - Git repository is not found or invalid
    /// - Required dependencies are missing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let analyzer = tools.analyzer()?;
    /// ```
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn initialize(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();

        // Validate path exists and is a directory
        if !path.exists() {
            return Err(Error::workflow(format!("Path does not exist: {}", path.display())));
        }

        if !path.is_dir() {
            return Err(Error::workflow(format!("Path is not a directory: {}", path.display())));
        }

        // Initialize the monorepo project
        let project = Arc::new(MonorepoProject::new(path)?);

        // Initialize the analyzer
        let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

        // Validate that this is actually a monorepo
        let packages = &project.packages;
        if packages.is_empty() {
            return Err(Error::workflow(format!(
                "No packages found in monorepo at {}. Please ensure this is a valid monorepo with packages.",
                path.display()
            )));
        }

        log::info!(
            "Initialized monorepo tools for {} with {} packages",
            path.display(),
            packages.len()
        );

        Ok(Self { project, analyzer })
    }

    /// Get a reference to the monorepo analyzer
    ///
    /// Returns a reference to the initialized `MonorepoAnalyzer` that can be used
    /// for analyzing the monorepo structure, dependencies, and changes.
    ///
    /// # Returns
    ///
    /// A reference to the `MonorepoAnalyzer` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let analyzer = tools.analyzer()?;
    /// let packages = analyzer.get_packages()?;
    /// ```
    pub fn analyzer(&self) -> Result<&MonorepoAnalyzer> {
        Ok(&self.analyzer)
    }

    /// Get a reference to the diff analyzer (Phase 2 functionality)
    #[must_use]
    pub fn diff_analyzer(&self) -> DiffAnalyzer {
        DiffAnalyzer::from_project(Arc::clone(&self.project))
    }

    /// Get a reference to the version manager (Phase 2 functionality)
    #[must_use]
    pub fn version_manager(&self) -> VersionManager {
        VersionManager::new(Arc::clone(&self.project))
    }

    /// Get a version manager with custom strategy (Phase 2 functionality)
    #[must_use]
    pub fn version_manager_with_strategy(
        &self,
        strategy: Box<dyn VersioningStrategy>,
    ) -> VersionManager {
        VersionManager::with_strategy(Arc::clone(&self.project), strategy)
    }

    /// Get a reference to the task manager (Phase 3 functionality)
    pub fn task_manager(&self) -> Result<TaskManager> {
        TaskManager::from_project(Arc::clone(&self.project))
    }

    /// Get a reference to the hook manager (Phase 3 functionality)
    pub fn hook_manager(&self) -> Result<HookManager> {
        HookManager::from_project(Arc::clone(&self.project))
    }

    /// Analyze changes between branches (Phase 2 functionality)
    pub fn analyze_changes_workflow(
        &self,
        from_branch: &str,
        to_branch: Option<&str>,
    ) -> Result<ChangeAnalysisWorkflowResult> {
        let diff_analyzer = self.diff_analyzer();

        let analysis = if let Some(to_branch) = to_branch {
            // Compare between specific branches
            let comparison = diff_analyzer.compare_branches(from_branch, to_branch)?;
            ChangeAnalysisWorkflowResult::BranchComparison(comparison)
        } else {
            // Analyze changes since a reference
            let analysis = diff_analyzer.detect_changes_since(from_branch, None)?;
            ChangeAnalysisWorkflowResult::ChangeAnalysis(analysis)
        };

        Ok(analysis)
    }

    /// Execute a complete versioning workflow (Phase 2 functionality)
    pub fn versioning_workflow(
        &self,
        plan: Option<VersioningPlan>,
    ) -> Result<VersioningWorkflowResult> {
        let start_time = std::time::Instant::now();
        let version_manager = self.version_manager();

        if let Some(plan) = plan {
            // Execute provided plan
            let result = version_manager.execute_versioning_plan(&plan)?;
            Ok(VersioningWorkflowResult {
                versioning_result: result,
                plan_executed: Some(plan),
                duration: start_time.elapsed(),
            })
        } else {
            // Create plan from current changes
            let diff_analyzer = self.diff_analyzer();
            let git_config = &self.project.config.git;
            let changes = diff_analyzer.detect_changes_since(&git_config.default_since_ref, None)?;
            let plan = version_manager.create_versioning_plan(&changes)?;
            let result = version_manager.execute_versioning_plan(&plan)?;

            Ok(VersioningWorkflowResult {
                versioning_result: result,
                plan_executed: Some(plan),
                duration: start_time.elapsed(),
            })
        }
    }

    /// Run the development workflow
    ///
    /// Executes the complete development workflow including change analysis,
    /// affected package detection, task execution, and validation.
    ///
    /// # Arguments
    ///
    /// * `since` - Optional reference point for change detection (defaults to configured git.default_since_ref)
    ///
    /// # Returns
    ///
    /// A `DevelopmentResult` containing analysis results, executed tasks, and recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow cannot be completed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let result = tools.development_workflow(Some("main")).await?;
    /// println!("Development checks passed: {}", result.checks_passed);
    /// ```
    pub async fn development_workflow(&self, since: Option<&str>) -> Result<DevelopmentResult> {
        let workflow = DevelopmentWorkflow::from_project(Arc::clone(&self.project))?;
        workflow.execute(since).await
    }

    /// Execute a complete release workflow
    ///
    /// This orchestrates the entire release process including change detection,
    /// version management, task execution, and deployment across multiple environments.
    ///
    /// # Arguments
    ///
    /// * `options` - Release configuration options including target environments and version bump preferences
    ///
    /// # Returns
    ///
    /// A comprehensive result containing information about the release process,
    /// including success status, affected packages, and any errors or warnings.
    ///
    /// # Errors
    ///
    /// Returns an error if any critical step of the release workflow fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{MonorepoTools, ReleaseOptions};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let options = ReleaseOptions::default();
    /// let result = tools.release_workflow(options).await?;
    /// 
    /// if result.success {
    ///     println!("Release completed successfully!");
    ///     println!("Affected packages: {}", result.affected_packages.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn release_workflow(&self, options: ReleaseOptions) -> Result<ReleaseResult> {
        let workflow = ReleaseWorkflow::from_project(Arc::clone(&self.project))?;
        workflow.execute(options).await
    }

    /// Create a plugin manager for this monorepo
    ///
    /// Creates a plugin manager instance that can be used to load and execute
    /// plugins for extending monorepo functionality.
    ///
    /// # Returns
    ///
    /// A configured plugin manager ready for use
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin manager cannot be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::MonorepoTools;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tools = MonorepoTools::initialize("/path/to/monorepo")?;
    /// let mut plugin_manager = tools.plugin_manager()?;
    ///
    /// // Load built-in plugins
    /// plugin_manager.load_builtin_plugins()?;
    ///
    /// // Execute plugin command
    /// let result = plugin_manager.execute_plugin_command("analyzer", "analyze-dependencies", &[])?;
    /// println!("Plugin result: {}", result.success);
    /// # Ok(())
    /// # }
    /// ```
    pub fn plugin_manager(&self) -> Result<PluginManager> {
        PluginManager::from_project(Arc::clone(&self.project))
    }
}
