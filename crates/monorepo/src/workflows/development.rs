//! Development workflow implementation
//!
//! This module provides development workflow functionality for day-to-day
//! development operations like running tests on affected packages,
//! validating changesets, and providing developer feedback.

use std::sync::Arc;
use std::time::Instant;

use crate::analysis::{AffectedPackagesAnalysis, ChangeAnalysis, MonorepoAnalyzer};
use crate::changes::{ChangeSignificance, PackageChange, PackageChangeType};
use crate::changesets::{types::ChangesetFilter, ChangesetManager};
use crate::core::MonorepoProject;
use crate::error::Error;
use crate::tasks::TaskManager;
use crate::workflows::{
    AffectedPackageInfo, ChangeAnalysisResult, ImpactLevel, PackageChangeFacts,
};
use std::collections::HashMap;
use sublime_git_tools::GitChangedFile;

// Import struct definition from types module
use crate::workflows::types::DevelopmentWorkflow;

impl DevelopmentWorkflow {
    /// Creates a new development workflow
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new `DevelopmentWorkflow` instance ready to execute development operations.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required components cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use sublime_monorepo_tools::{DevelopmentWorkflow, MonorepoProject};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
    /// let workflow = DevelopmentWorkflow::new(project).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self, Error> {
        let analyzer = MonorepoAnalyzer::new(Arc::clone(&project));
        let changeset_manager = ChangesetManager::new(Arc::clone(&project))?;
        let task_manager = TaskManager::new(Arc::clone(&project))?;

        Ok(Self { project, analyzer, changeset_manager, task_manager })
    }

    /// Executes the development workflow
    ///
    /// This method performs development-time checks:
    /// 1. Detects changes since the specified reference
    /// 2. Identifies affected packages
    /// 3. Runs tests and linting on affected packages
    /// 4. Validates changeset requirements
    /// 5. Provides recommendations to the developer
    ///
    /// # Arguments
    ///
    /// * `since` - Optional reference to compare changes against (defaults to configured git.default_since_ref)
    ///
    /// # Returns
    ///
    /// Development result with check status and recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow cannot be executed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(workflow: &DevelopmentWorkflow) -> Result<(), Box<dyn std::error::Error>> {
    /// // Check changes since last commit
    /// let result = workflow.execute(Some("HEAD~1")).await?;
    ///
    /// for recommendation in &result.recommendations {
    ///     println!("Recommendation: {}", recommendation);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, since: Option<&str>) -> Result<super::DevelopmentResult, Error> {
        let start_time = Instant::now();

        // Default to comparing against configured git reference if no reference provided
        let git_config = &self.project.config.git;
        let since_ref = since.unwrap_or(&git_config.default_since_ref);

        // Step 1: Detect changes
        let changes = self.analyzer.detect_changes_since(since_ref, None)?;

        // Step 2: Execute tasks for affected packages
        let affected_packages: Vec<String> =
            changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

        let affected_tasks = if affected_packages.is_empty() {
            Vec::new()
        } else {
            self.task_manager.execute_tasks_for_affected_packages(&affected_packages).await?
        };

        // Step 3: Check if tasks passed
        let tasks_passed = affected_tasks
            .iter()
            .all(|task| matches!(task.status, crate::tasks::types::results::TaskStatus::Success));

        // Step 4: Generate recommendations
        let recommendations = self.generate_recommendations(&changes, &affected_tasks)?;

        Ok(super::DevelopmentResult {
            changes,
            affected_tasks,
            recommendations,
            checks_passed: tasks_passed,
            duration: start_time.elapsed(),
        })
    }

    /// Analyzes changes and provides detailed analysis
    ///
    /// This method provides comprehensive analysis of changes including
    /// impact assessment, version recommendations, and changeset requirements.
    ///
    /// # Arguments
    ///
    /// * `from_branch` - Source branch for comparison
    /// * `to_branch` - Target branch for comparison (defaults to current branch)
    ///
    /// # Returns
    ///
    /// Detailed change analysis with recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if the analysis cannot be performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(workflow: &DevelopmentWorkflow) -> Result<(), Box<dyn std::error::Error>> {
    /// let analysis = workflow.analyze_changes("main", None).await?;
    ///
    /// for package in &analysis.affected_packages {
    ///     println!("Package {} has {:?} impact", package.name, package.impact_level);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_changes(
        &self,
        from_branch: &str,
        to_branch: Option<&str>,
    ) -> Result<ChangeAnalysisResult, Error> {
        let start_time = Instant::now();

        // Get current branch if to_branch not specified
        let target_branch = match to_branch {
            Some(branch) => branch.to_string(),
            None => self
                .project
                .repository
                .get_current_branch()
                .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?,
        };

        // Analyze changes
        let comparison = self.analyzer.compare_branches(from_branch, &target_branch)?;

        // Map changed files to package changes
        let changes_package_changes =
            self.map_files_to_package_changes(&comparison.changed_files)?;

        // For now, use empty package changes since we have a type mismatch
        // TODO: Properly convert between the two PackageChange types in a future phase
        let package_changes: Vec<crate::changes::PackageChange> = Vec::new();

        // Create a complete ChangeAnalysis from the comparison
        let analysis = ChangeAnalysis {
            from_ref: from_branch.to_string(),
            to_ref: target_branch.to_string(),
            changed_files: comparison.changed_files.clone(),
            package_changes,
            affected_packages: AffectedPackagesAnalysis {
                directly_affected: comparison.affected_packages.clone(),
                dependents_affected: Vec::new(),
                change_propagation_graph: std::collections::HashMap::new(),
                impact_scores: std::collections::HashMap::new(),
                total_affected_count: comparison.affected_packages.len(),
            },
            significance_analysis: Vec::new(),
        };

        // Build affected package information with actual package changes
        let affected_packages = self.build_affected_package_info(&changes_package_changes);

        // Generate version recommendations based on actual package changes
        let version_recommendations =
            self.generate_version_recommendations(&changes_package_changes);

        // Check if changesets are required
        let changesets_required = self.check_changesets_required(&affected_packages);

        Ok(ChangeAnalysisResult {
            analysis,
            affected_packages,
            version_recommendations,
            changesets_required,
            duration: start_time.elapsed(),
        })
    }

    /// Generates development recommendations based on analysis
    pub fn generate_recommendations(
        &self,
        changes: &ChangeAnalysis,
        task_results: &[crate::tasks::types::results::TaskExecutionResult],
    ) -> Result<Vec<String>, Error> {
        let mut recommendations = Vec::new();

        // Check if there are any changes
        if changes.changed_files.is_empty() {
            recommendations.push("No changes detected. You're up to date!".to_string());
            return Ok(recommendations);
        }

        // Check for failed tasks
        let failed_tasks: Vec<_> = task_results
            .iter()
            .filter(|result| {
                !matches!(result.status, crate::tasks::types::results::TaskStatus::Success)
            })
            .collect();

        if !failed_tasks.is_empty() {
            recommendations.push(format!(
                "❌ {} task(s) failed. Please fix the issues before proceeding.",
                failed_tasks.len()
            ));

            for failed_task in failed_tasks {
                recommendations.push(format!(
                    "  - {}: {}",
                    failed_task.task_name,
                    failed_task.errors.first().map_or("Unknown error", |e| e.message.as_str())
                ));
            }
        } else if !task_results.is_empty() {
            recommendations.push("✅ All tests and checks passed!".to_string());
        }

        // Check changeset requirements
        let current_branch = self
            .project
            .repository
            .get_current_branch()
            .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?;

        let changeset_filter =
            ChangesetFilter { branch: Some(current_branch.clone()), ..Default::default() };

        let existing_changesets = self.changeset_manager.list_changesets(&changeset_filter)?;

        if existing_changesets.is_empty() && !changes.package_changes.is_empty() {
            recommendations.push("💡 Consider creating a changeset for your changes:".to_string());
            recommendations.push(
                "   Run the changeset creation command to document your changes.".to_string(),
            );
        }

        // Package-specific recommendations
        for package_change in &changes.package_changes {
            if package_change.change_type == crate::changes::PackageChangeType::Dependencies {
                recommendations.push(format!(
                    "📦 Dependencies changed in {}: Consider updating version locks",
                    package_change.package_name
                ));
            }
        }

        Ok(recommendations)
    }

    /// Builds detailed information about affected packages
    fn build_affected_package_info(
        &self,
        package_changes: &[PackageChange],
    ) -> Vec<AffectedPackageInfo> {
        let mut affected_packages = Vec::new();

        for package_change in package_changes {
            let impact_level = self.determine_impact_level(package_change);

            let changed_files: Vec<String> =
                package_change.changed_files.iter().map(|f| f.path.clone()).collect();

            // For now, simulate dependents (would normally query dependency graph)
            let dependents = Vec::new();

            affected_packages.push(AffectedPackageInfo {
                name: package_change.package_name.clone(),
                impact_level,
                changed_files,
                dependents,
            });
        }

        affected_packages
    }

    /// Determines the impact level based on facts using configurable thresholds
    pub fn determine_impact_level(&self, package_change: &PackageChange) -> ImpactLevel {
        // Use actual changed files count
        let total_files = package_change.changed_files.len();

        // Get thresholds from configuration
        let thresholds = &self.project.config.tasks.performance.impact_thresholds;

        // Use configurable thresholds for impact level determination
        match total_files {
            files if files > thresholds.high_impact_files => ImpactLevel::High,
            files if files > thresholds.medium_impact_files => ImpactLevel::Medium,
            _ => ImpactLevel::Low,
        }
    }

    /// Generates version bump recommendations based on facts
    fn generate_version_recommendations(
        &self,
        package_changes: &[PackageChange],
    ) -> Vec<super::types::VersionRecommendation> {
        let mut recommendations = Vec::new();

        for package_change in package_changes {
            let reason = self.generate_recommendation_reason(package_change);

            // Get impact level from facts
            let impact = self.determine_impact_level(package_change);

            // Simple bump suggestion based on impact scale
            let recommended_bump = match impact {
                ImpactLevel::High | ImpactLevel::Critical => crate::VersionBumpType::Minor,
                ImpactLevel::Medium | ImpactLevel::Low => crate::VersionBumpType::Patch,
            };

            recommendations.push(super::types::VersionRecommendation {
                package: package_change.package_name.clone(),
                recommended_bump,
                reason,
                confidence: super::types::ConfidenceLevel::Medium, // Always medium - we're just suggesting
            });
        }

        recommendations
    }

    /// Generates reason for version recommendation based on facts
    #[allow(clippy::unused_self)]
    #[allow(clippy::if_not_else)]
    fn generate_recommendation_reason(&self, package_change: &PackageChange) -> String {
        let total_files = package_change.changed_files.len();

        let example_files = if !package_change.changed_files.is_empty() {
            package_change.changed_files[0].path.clone()
        } else {
            "No files".to_string()
        };

        format!("Changes detected: {total_files} files modified. Examples: {example_files}")
    }

    /// Checks if changesets are required for the affected packages
    fn check_changesets_required(&self, affected_packages: &[AffectedPackageInfo]) -> bool {
        // Check configuration to see if changesets are required
        let changesets_required = self.project.config.changesets.required;

        if !changesets_required {
            return false;
        }

        // Check if any packages have medium or high impact
        affected_packages.iter().any(|pkg| {
            matches!(
                pkg.impact_level,
                ImpactLevel::Medium | ImpactLevel::High | ImpactLevel::Critical
            )
        })
    }

    /// Map changed files to package changes using pure facts approach
    ///
    /// Groups files by package and creates simple fact-based reports - no decisions made.
    #[allow(clippy::unnecessary_wraps)]
    fn map_files_to_package_changes(
        &self,
        changed_files: &[sublime_git_tools::GitChangedFile],
    ) -> Result<Vec<PackageChange>, Error> {
        // Group files by package
        let mut package_file_groups: HashMap<String, Vec<sublime_git_tools::GitChangedFile>> =
            HashMap::new();

        for file in changed_files {
            // Find which package this file belongs to
            let file_path = self.project.root_path().join(&file.path);

            if let Some(package) = self.project.descriptor.find_package_for_path(&file_path) {
                package_file_groups.entry(package.name.clone()).or_default().push(file.clone());
            }
        }

        // Convert to PackageChange objects using facts-only approach
        let mut package_changes = Vec::new();

        for (package_name, files) in package_file_groups {
            let facts = self.create_change_facts(&files);

            // Store facts in metadata - no decisions made about significance or type
            // Convert string file paths to GitChangedFile format
            let changed_files = facts
                .files_changed
                .iter()
                .map(|file_path| GitChangedFile {
                    path: file_path.clone(),
                    status: sublime_git_tools::GitFileStatus::Modified,
                    staged: false,
                    workdir: true,
                })
                .collect();

            package_changes.push(PackageChange {
                package_name,
                change_type: PackageChangeType::SourceCode, // Always the same - no decisions
                significance: ChangeSignificance::Low,      // Always the same - no decisions
                changed_files,
                suggested_version_bump: crate::config::VersionBumpType::Patch,
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(package_changes)
    }

    /// Creates a simple facts-based change report for a package
    ///
    /// No decisions made - just pure facts about what files changed
    #[allow(clippy::unused_self)]
    fn create_change_facts(
        &self,
        files: &[sublime_git_tools::GitChangedFile],
    ) -> PackageChangeFacts {
        let total_files = files.len();
        let files_changed: Vec<String> = files.iter().map(|f| f.path.clone()).collect();

        PackageChangeFacts { total_files, files_changed }
    }
}
