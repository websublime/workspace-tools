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

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]
#![allow(clippy::unimplemented)] // Temporarily allow during phased implementation
#![deny(clippy::panic)]

// Public modules
pub mod analysis;
pub mod changes;
pub mod config;
pub mod core;
pub mod error;

// Re-exports
use std::sync::Arc;

pub use crate::analysis::{
    MonorepoAnalysisResult, MonorepoAnalyzer, 
    DiffAnalyzer, BranchComparisonResult, ChangeAnalysis, AffectedPackagesAnalysis,
    ChangeSignificanceResult,
};
pub use crate::changes::{
    ChangeDetector, ChangeDetectionEngine, ChangeDetectionRules,
    PackageChange, PackageChangeType, ChangeSignificance,
    ChangeTypeRule, SignificanceRule, VersionBumpRule,
    FilePattern, PatternType,
};
pub use crate::config::{ConfigManager, Environment, MonorepoConfig, VersionBumpType};
pub use crate::core::{
    Changeset, ChangesetStatus, MonorepoPackageInfo, MonorepoProject, VersionStatus,
    VersionManager, VersioningStrategy, DefaultVersioningStrategy, 
    ConservativeVersioningStrategy, AggressiveVersioningStrategy,
    VersioningResult, PackageVersionUpdate, VersionImpactAnalysis,
    VersioningPlan, VersioningPlanStep,
};
pub use crate::error::{Error, Result};

// Main entry point - will be implemented in later phases
/// The main orchestrator for monorepo tools functionality
pub struct MonorepoTools {
    project: std::sync::Arc<MonorepoProject>,
}

impl MonorepoTools {
    /// Initialize MonorepoTools by detecting and opening a monorepo at the given path
    pub fn initialize(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::initialize will be implemented in Phase 6")
    }

    /// Get a reference to the monorepo analyzer
    pub fn analyzer(&self) -> &MonorepoAnalyzer {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::analyzer will be implemented in Phase 6")
    }

    /// Get a reference to the diff analyzer (Phase 2 functionality)
    pub fn diff_analyzer(&self) -> DiffAnalyzer {
        DiffAnalyzer::new(Arc::clone(&self.project))
    }

    /// Get a reference to the version manager (Phase 2 functionality)
    pub fn version_manager(&self) -> VersionManager {
        VersionManager::new(Arc::clone(&self.project))
    }

    /// Get a version manager with custom strategy (Phase 2 functionality)
    pub fn version_manager_with_strategy(&self, strategy: Box<dyn VersioningStrategy>) -> VersionManager {
        VersionManager::with_strategy(Arc::clone(&self.project), strategy)
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
        let version_manager = self.version_manager();

        if let Some(plan) = plan {
            // Execute provided plan
            let result = version_manager.execute_versioning_plan(&plan)?;
            Ok(VersioningWorkflowResult {
                versioning_result: result,
                plan_executed: Some(plan),
                duration: std::time::Duration::from_secs(0), // Would be measured in real implementation
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
                duration: std::time::Duration::from_secs(0), // Would be measured in real implementation
            })
        }
    }

    /// Run the development workflow
    #[allow(clippy::unused_async)]
    pub async fn development_workflow(&self, _since: Option<&str>) -> Result<DevelopmentResult> {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::development_workflow will be implemented in Phase 6")
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
