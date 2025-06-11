//! MonorepoTools implementation - main orchestrator for monorepo functionality

use crate::analysis::{BranchComparisonResult, ChangeAnalysis, DiffAnalyzer, MonorepoAnalyzer};
use crate::core::{MonorepoProject, VersionManager, VersioningPlan, VersioningResult, VersioningStrategy};
use crate::error::{Error, Result};
use crate::hooks::HookManager;
use crate::tasks::TaskManager;
use crate::workflows::{DevelopmentResult, DevelopmentWorkflow};
use std::sync::Arc;

/// The main orchestrator for monorepo tools functionality
pub struct MonorepoTools {
    project: Arc<MonorepoProject>,
    analyzer: MonorepoAnalyzer,
}

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
        let analyzer = MonorepoAnalyzer::new(Arc::clone(&project));

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
        DiffAnalyzer::new(Arc::clone(&self.project))
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
        TaskManager::new(Arc::clone(&self.project))
    }

    /// Get a reference to the hook manager (Phase 3 functionality)
    pub fn hook_manager(&self) -> Result<HookManager> {
        HookManager::new(Arc::clone(&self.project))
    }

    /// Analyze changes between branches (Phase 2 functionality)
    #[allow(clippy::unused_async)]
    pub async fn analyze_changes_workflow(
        &self,
        from_branch: &str,
        to_branch: Option<&str>,
    ) -> Result<super::ChangeAnalysisWorkflowResult> {
        let diff_analyzer = self.diff_analyzer();

        let analysis = if let Some(to_branch) = to_branch {
            // Compare between specific branches
            let comparison = diff_analyzer.compare_branches(from_branch, to_branch)?;
            super::ChangeAnalysisWorkflowResult::BranchComparison(comparison)
        } else {
            // Analyze changes since a reference
            let analysis = diff_analyzer.detect_changes_since(from_branch, None)?;
            super::ChangeAnalysisWorkflowResult::ChangeAnalysis(analysis)
        };

        Ok(analysis)
    }

    /// Execute a complete versioning workflow (Phase 2 functionality)
    #[allow(clippy::unused_async)]
    pub async fn versioning_workflow(
        &self,
        plan: Option<VersioningPlan>,
    ) -> Result<super::VersioningWorkflowResult> {
        let start_time = std::time::Instant::now();
        let version_manager = self.version_manager();

        if let Some(plan) = plan {
            // Execute provided plan
            let result = version_manager.execute_versioning_plan(&plan)?;
            Ok(super::VersioningWorkflowResult {
                versioning_result: result,
                plan_executed: Some(plan),
                duration: start_time.elapsed(),
            })
        } else {
            // Create plan from current changes
            let diff_analyzer = self.diff_analyzer();
            let changes = diff_analyzer.detect_changes_since("HEAD~1", None)?;
            let plan = version_manager.create_versioning_plan(&changes)?;
            let result = version_manager.execute_versioning_plan(&plan)?;

            Ok(super::VersioningWorkflowResult {
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
    /// * `since` - Optional reference point for change detection (defaults to "HEAD~1")
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
        let workflow = DevelopmentWorkflow::new(Arc::clone(&self.project))?;
        workflow.execute(since).await
    }
}