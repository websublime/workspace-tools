#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

//! # Sublime Monorepo Tools
//!
//! A comprehensive library that unifies functionality from base crates to provide complete
//! Node.js monorepo workflows including versioning, diff analysis, task management, and more.
//!
//! ## Features
//!
//! - **Versioning**: Major, Minor, Patch, Snapshot with automatic propagation to dependents
//! - **Diffs**: Recognize differences between branches and affected packages
//! - **Tasks**: Package.json scripts organized as tasks executed based on changes
//! - **Monorepo Analysis**: Package manager detection, dependency graph, internal/external packages
//! - **Monorepo as Project**: Aggregate complete monorepo information
//! - **Changelogs**: Based on conventional commits with customizable templates
//! - **Hooks**: Git hooks (pre-commit, pre-push) with validations
//! - **Changesets**: Change management with deployment environments
//! - **Plugins**: Extensible system for customization
//!
//! ## Architecture
//!
//! This crate is built on top of three foundational crates:
//! - `sublime_git_tools`: Git operations and repository management
//! - `sublime_standard_tools`: File system, command execution, and monorepo detection
//! - `sublime_package_tools`: Package management, dependencies, and version handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::MonorepoTools;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize monorepo tools
//! let tools = MonorepoTools::initialize(Path::new("."))?;
//!
//! // Analyze the monorepo
//! let analysis = tools.analyzer().detect_monorepo_info(Path::new("."))?;
//! println!("Found {} packages", analysis.packages.internal_packages.len());
//!
//! // Run development workflow
//! let result = tools.development_workflow(None).await?;
//! println!("Affected tasks: {}", result.affected_tasks.len());
//! # Ok(())
//! # }
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]
#![allow(clippy::unimplemented)] // Temporarily allow during phased implementation
#![deny(clippy::panic)]

// Public modules
pub mod analysis;
pub mod changes;
pub mod changesets;
pub mod config;
pub mod core;
pub mod error;
pub mod hooks;
pub mod tasks;
pub mod workflows;

// Re-exports
use std::sync::Arc;

pub use crate::analysis::{
    AffectedPackagesAnalysis, BranchComparisonResult, ChangeAnalysis, ChangeSignificanceResult,
    DiffAnalyzer, MonorepoAnalysisResult, MonorepoAnalyzer,
};
pub use crate::changes::{
    ChangeDetectionEngine, ChangeDetectionRules, ChangeDetector, ChangeSignificance,
    ChangeTypeRule, FilePattern as ChangeFilePattern, PackageChange, PackageChangeType,
    PatternType, SignificanceRule, VersionBumpRule,
};
pub use crate::changesets::{
    Changeset, ChangesetApplication, ChangesetFilter, ChangesetManager, ChangesetSpec,
    ChangesetStatus, DeploymentResult, EnvironmentDeploymentResult, ValidationResult,
};
pub use crate::config::{ConfigManager, Environment, MonorepoConfig, VersionBumpType};
pub use crate::core::{
    AggressiveVersioningStrategy, ConservativeVersioningStrategy, DefaultVersioningStrategy,
    MonorepoPackageInfo, MonorepoProject, PackageVersionUpdate, VersionImpactAnalysis,
    VersionManager, VersionStatus, VersioningPlan, VersioningPlanStep, VersioningResult,
    VersioningStrategy,
};
pub use crate::error::{Error, Result};
pub use crate::hooks::{
    CommitInfo, GitOperationType, HookCondition, HookDefinition, HookError, HookErrorCode,
    HookExecutionContext, HookExecutionResult, HookInstaller, HookManager, HookScript, HookStatus,
    HookType, HookValidationResult, HookValidator, PostCommitResult, PreCommitResult,
    PrePushResult, RemoteInfo, ValidationCheck,
};
pub use crate::tasks::{
    FilePattern, FilePatternType, PackageScript, TaskCommand, TaskCondition, TaskDefinition,
    TaskError, TaskExecutionLog, TaskExecutionResult, TaskExecutionStats, TaskExecutor,
    TaskManager, TaskOutput, TaskPriority, TaskRegistry, TaskScope, TaskStatus, TaskTrigger,
};
pub use crate::workflows::{
    AffectedPackageInfo, ChangeAnalysisResult, ChangesetHookIntegration, ConfidenceLevel,
    DevelopmentResult as WorkflowDevelopmentResult, DevelopmentWorkflow, ImpactLevel,
    ReleaseOptions, ReleaseResult, ReleaseWorkflow, VersionRecommendation, WorkflowProgress,
    WorkflowStatus,
};

// Main entry point - will be implemented in later phases
/// The main orchestrator for monorepo tools functionality
pub struct MonorepoTools {
    project: std::sync::Arc<MonorepoProject>,
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
    #[allow(clippy::unused_async)]
    pub async fn versioning_workflow(
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
            let changes = diff_analyzer.detect_changes_since("HEAD~1", None)?;
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
    pub async fn development_workflow(
        &self,
        since: Option<&str>,
    ) -> Result<crate::workflows::DevelopmentResult> {
        use crate::workflows::DevelopmentWorkflow;

        let workflow = DevelopmentWorkflow::new(Arc::clone(&self.project))?;
        workflow.execute(since).await
    }
}

/// Result of a development workflow execution
pub struct DevelopmentResult {
    /// Tasks that were executed
    pub affected_tasks: Vec<String>,
}

/// Result of a change analysis workflow (Phase 2)
#[derive(Debug)]
pub enum ChangeAnalysisWorkflowResult {
    /// Result of branch comparison
    BranchComparison(BranchComparisonResult),
    /// Result of change analysis since a reference
    ChangeAnalysis(ChangeAnalysis),
}

/// Result of a versioning workflow execution (Phase 2)
#[derive(Debug)]
pub struct VersioningWorkflowResult {
    /// The versioning operation result
    pub versioning_result: VersioningResult,
    /// The plan that was executed
    pub plan_executed: Option<VersioningPlan>,
    /// Duration of the workflow execution
    pub duration: std::time::Duration,
}
