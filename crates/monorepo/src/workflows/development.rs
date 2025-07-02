//! Development workflow implementation
//!
//! This module provides development workflow functionality for day-to-day
//! development operations like running tests on affected packages,
//! validating changesets, and providing developer feedback.

use std::time::Instant;

use crate::analysis::{AffectedPackagesAnalysis, ChangeAnalysis, MonorepoAnalyzer};
use crate::changes::{
    ChangeDecisionSource, ChangeSignificance, ConventionalCommitParser, PackageChange,
    PackageChangeType, VersionBumpType,
};
use crate::changesets::types::ChangesetFilter;
use crate::core::MonorepoProject;
use crate::error::Error;
use crate::workflows::{
    AffectedPackageInfo, ChangeAnalysisResult, ImpactLevel, PackageChangeFacts,
};
use std::collections::HashMap;
use sublime_git_tools::GitChangedFile;

// Import struct definition from types module
use crate::workflows::types::DevelopmentWorkflow;

/// Configuration for creating a DevelopmentWorkflow
///
/// Groups all the dependencies needed to create a development workflow
/// to avoid too_many_arguments clippy warning and improve API usability.
pub struct DevelopmentWorkflowConfig<'a> {
    /// Analyzer for detecting changes and affected packages
    pub analyzer: MonorepoAnalyzer<'a>,
    /// Task manager for executing development tasks
    pub task_manager: crate::tasks::TaskManager<'a>,
    /// Changeset manager for handling development changesets
    pub changeset_manager: crate::changesets::ChangesetManager<'a>,
    /// Direct reference to configuration
    pub config: &'a crate::config::MonorepoConfig,
    /// Direct reference to packages
    pub packages: &'a [crate::core::MonorepoPackageInfo],
    /// Direct reference to git repository
    pub repository: &'a sublime_git_tools::Repo,
    /// Direct reference to root path
    pub root_path: &'a std::path::Path,
}

impl<'a> DevelopmentWorkflow<'a> {
    /// Creates a new development workflow with configuration struct
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles. Accepts a configuration struct
    /// to avoid too_many_arguments issues and improve maintainability.
    ///
    /// # Arguments
    ///
    /// * `config` - Development workflow configuration containing all dependencies
    ///
    /// # Returns
    ///
    /// A new development workflow instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::workflows::{DevelopmentWorkflow, DevelopmentWorkflowConfig};
    /// use sublime_monorepo_tools::analysis::MonorepoAnalyzer;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DevelopmentWorkflowConfig {
    ///     analyzer: MonorepoAnalyzer::new(&project),
    ///     task_manager: task_manager,
    ///     changeset_manager: changeset_manager,
    ///     config: &project.config,
    ///     packages: &project.packages,
    ///     repository: &project.repository,
    ///     root_path: &project.root_path,
    /// };
    /// let workflow = DevelopmentWorkflow::new(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: DevelopmentWorkflowConfig<'a>) -> Self {
        Self {
            analyzer: config.analyzer,
            task_manager: config.task_manager,
            changeset_manager: config.changeset_manager,
            config: config.config,
            packages: config.packages,
            repository: config.repository,
            root_path: config.root_path,
        }
    }


    /// Creates a new development workflow from project (convenience method)
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
    pub fn from_project(project: &'a MonorepoProject) -> Result<Self, Error> {
        use crate::analysis::MonorepoAnalyzer;
        use crate::changesets::{ChangesetManager, ChangesetStorage};
        use crate::tasks::TaskManager;

        // Create analyzer with direct borrowing
        let analyzer = MonorepoAnalyzer::new(project);

        // Create task manager with direct borrowing
        let task_manager = TaskManager::new(project)?;

        // Create changeset storage with direct borrowing
        let storage = ChangesetStorage::new(
            project.config.changesets.clone(),
            &project.file_system,
            &project.root_path,
        );

        // Create changeset manager with direct borrowing
        // Note: We need to create a separate task manager instance for changeset manager
        let changeset_task_manager = TaskManager::new(project)?;
        let changeset_manager = ChangesetManager::new(
            storage,
            changeset_task_manager,
            &project.config,
            &project.file_system,
            &project.packages,
            &project.repository,
            &project.root_path,
        );

        let config = DevelopmentWorkflowConfig {
            analyzer,
            task_manager,
            changeset_manager,
            config: &project.config,
            packages: &project.packages,
            repository: &project.repository,
            root_path: &project.root_path,
        };

        Ok(Self::new(config))
    }

    /// Executes the development workflow synchronously
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
    /// # fn example(workflow: &DevelopmentWorkflow) -> Result<(), Box<dyn std::error::Error>> {
    /// // Check changes since last commit
    /// let result = workflow.execute(Some("HEAD~1"))?;
    ///
    /// for recommendation in &result.recommendations {
    ///     println!("Recommendation: {}", recommendation);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute(&self, since: Option<&str>) -> Result<super::DevelopmentResult, Error> {
        let start_time = Instant::now();

        // Default to comparing against configured git reference if no reference provided
        let git_config = &self.config.git;
        let since_ref = since.unwrap_or(&git_config.default_since_ref);

        // Step 1: Detect changes
        let changes = self.analyzer.detect_changes_since(since_ref, None)?;

        // Step 2: Execute tasks for affected packages
        let affected_packages: Vec<String> =
            changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

        let affected_tasks = if affected_packages.is_empty() {
            Vec::new()
        } else {
            self.task_manager.execute_tasks_for_affected_packages(&affected_packages)?
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
                .repository
                .get_current_branch()
                .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?,
        };

        // Analyze changes
        let comparison = self.analyzer.compare_branches(from_branch, &target_branch)?;

        // Map changed files to package changes
        let changes_package_changes =
            self.map_files_to_package_changes(&comparison.changed_files)?;

        // Convert PackageChangeFacts to full PackageChange objects
        let package_changes = self.convert_facts_to_package_changes(&changes_package_changes)?;

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
        let thresholds = &self.config.tasks.performance.impact_thresholds;

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
        let changesets_required = self.config.changesets.required;

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
            let file_path = self.root_path.join(&file.path);

            // Find which package this file belongs to by checking all packages
            for package in self.packages {
                let package_path = package.path();
                if file_path.starts_with(package_path) {
                    package_file_groups
                        .entry(package.name().to_string())
                        .or_default()
                        .push(file.clone());
                    break; // Found the package, no need to check others
                }
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

    /// Converts PackageChangeFacts to full PackageChange objects with proper analysis
    ///
    /// This function bridges the gap between simple fact collection and comprehensive
    /// change analysis by converting basic file change facts into structured PackageChange
    /// objects with proper classification and metadata.
    ///
    /// # Arguments
    ///
    /// * `package_changes` - Vector of PackageChange objects containing basic change information
    ///
    /// # Returns
    ///
    /// A vector of fully analyzed PackageChange objects with proper change types,
    /// significance levels, and version bump recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion process fails due to invalid data or
    /// configuration issues.
    #[allow(clippy::unnecessary_wraps)]
    fn convert_facts_to_package_changes(
        &self,
        package_changes: &[PackageChange],
    ) -> Result<Vec<crate::changes::PackageChange>, Error> {
        let mut converted_changes = Vec::new();

        for package_change in package_changes {
            // Determine version bump using changeset-first, conventional-commits-fallback approach
            let version_decision = self.determine_version_bump(
                &package_change.package_name,
                &package_change.changed_files,
            )?;
            let suggested_version_bump = version_decision.version_bump();

            // Infer change type from version bump and files (simplified approach)
            let change_type =
                Self::infer_change_type(&package_change.changed_files, suggested_version_bump);

            // Determine significance based on version bump and change type
            let significance = Self::infer_significance_from_version_bump(suggested_version_bump);

            // Create metadata with additional information
            let mut metadata = std::collections::HashMap::new();
            metadata
                .insert("total_files".to_string(), package_change.changed_files.len().to_string());
            metadata.insert("change_analysis_version".to_string(), "1.0".to_string());

            // Add file pattern analysis to metadata
            let file_extensions: Vec<String> = package_change
                .changed_files
                .iter()
                .filter_map(|f| {
                    std::path::Path::new(&f.path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(std::string::ToString::to_string)
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            if !file_extensions.is_empty() {
                metadata.insert("file_extensions".to_string(), file_extensions.join(","));
            }

            converted_changes.push(crate::changes::PackageChange {
                package_name: package_change.package_name.clone(),
                change_type,
                significance,
                changed_files: package_change.changed_files.clone(),
                suggested_version_bump,
                metadata,
            });
        }

        Ok(converted_changes)
    }

    /// Analyzes the significance of changes based on files and change type
    ///

    /// Determines version bump using changeset-first, conventional-commits-fallback approach
    ///
    /// This method implements the intelligent version bump determination logic:
    /// 1. First priority: Check for explicit changesets for this package
    /// 2. Second priority: Analyze conventional commits in the changed files
    /// 3. Final fallback: Conservative patch bump
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to determine version bump for
    /// * `changed_files` - List of files that changed in this package
    ///
    /// # Returns
    ///
    /// `ChangeDecisionSource` containing the version bump and its source
    ///
    /// # Errors
    ///
    /// Returns an error if git operations fail or changeset analysis encounters issues
    fn determine_version_bump(
        &self,
        package_name: &str,
        changed_files: &[GitChangedFile],
    ) -> Result<ChangeDecisionSource, Error> {
        // Step 1: Check for explicit changesets (highest priority)
        if let Some(changeset_bump) = self.find_changeset_for_package(package_name)? {
            log::info!(
                "Found explicit changeset for package '{}': {:?}",
                package_name,
                changeset_bump
            );
            return Ok(ChangeDecisionSource::Changeset(changeset_bump));
        }

        // Step 2: Analyze conventional commits (intelligent fallback)
        if let Some(conventional_bump) =
            self.analyze_conventional_commits_for_files(changed_files)?
        {
            log::debug!(
                "Determined version bump from conventional commits for package '{}': {:?}",
                package_name,
                conventional_bump
            );
            return Ok(ChangeDecisionSource::ConventionalCommit(conventional_bump));
        }

        // Step 3: Conservative fallback
        log::debug!(
            "Using conservative patch fallback for package '{}' (no changesets or conventional commits found)",
            package_name
        );
        Ok(ChangeDecisionSource::Fallback(VersionBumpType::Patch))
    }

    /// Finds explicit changeset for a specific package
    ///
    /// Searches through existing changesets to find explicit version bump decisions
    /// made by developers for this package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to search changesets for
    ///
    /// # Returns
    ///
    /// `Some(VersionBumpType)` if an explicit changeset exists, `None` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if changeset retrieval fails
    fn find_changeset_for_package(
        &self,
        package_name: &str,
    ) -> Result<Option<VersionBumpType>, Error> {
        // Get pending changesets for this package
        let filter = ChangesetFilter {
            package: Some(package_name.to_string()),
            status: Some(crate::changesets::types::ChangesetStatus::Pending),
            environment: None,
            branch: None,
            author: None,
        };

        let changesets = self.changeset_manager.list_changesets(&filter)?;

        // Return the version bump from the most recent changeset
        if let Some(changeset) = changesets.first() {
            Ok(Some(changeset.version_bump))
        } else {
            Ok(None)
        }
    }

    /// Analyzes conventional commits for changed files to suggest version bump
    ///
    /// Gets the commit history that affected these files and parses conventional
    /// commit messages to determine appropriate version bump.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - Files that changed, used to filter relevant commits
    ///
    /// # Returns
    ///
    /// `Some(VersionBumpType)` if conventional commits suggest a bump, `None` otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if git operations fail
    fn analyze_conventional_commits_for_files(
        &self,
        _changed_files: &[GitChangedFile],
    ) -> Result<Option<VersionBumpType>, Error> {
        // For simplicity, get all commits since last tag (the git crate has a simpler API)
        let commits = if let Ok(last_tag) = self.repository.get_last_tag() {
            // Get commits since last tag
            self.repository.get_commits_since(Some(last_tag), &None)
        } else {
            // Fallback: get all recent commits (no relative filter)
            self.repository.get_commits_since(None, &None)
        }
        .map_err(|e| Error::workflow(format!("Failed to get commits: {e}")))?;

        // Filter commits to only those that are relevant (this is a simplified approach)
        // In a more sophisticated implementation, we could check if commits touch the specific files
        // For now, we'll analyze all commits in the range which is reasonable for conventional commits

        // Parse conventional commits
        let parser = ConventionalCommitParser::new();
        Ok(parser.analyze_commits(commits))
    }

    /// Infers change type from files and version bump
    ///
    /// Since we're moving away from hardcoded file pattern analysis, this method
    /// provides a simplified inference based on version bump type and basic file analysis.
    ///
    /// # Arguments
    ///
    /// * `changed_files` - List of changed files
    /// * `version_bump` - The determined version bump type
    ///
    /// # Returns
    ///
    /// Inferred `PackageChangeType` based on available information
    fn infer_change_type(
        changed_files: &[GitChangedFile],
        version_bump: VersionBumpType,
    ) -> PackageChangeType {
        // For major version bumps, assume source code changes (breaking changes)
        if matches!(version_bump, VersionBumpType::Major) {
            return PackageChangeType::SourceCode;
        }

        // Simple heuristic: check for specific well-known dependency files
        for file in changed_files {
            let path_lower = file.path.to_lowercase();
            if path_lower.ends_with("cargo.toml")
                || path_lower.ends_with("package.json")
                || path_lower.ends_with("go.mod")
            {
                return PackageChangeType::Dependencies;
            }
        }

        // For minor bumps, likely new features (source code)
        if matches!(version_bump, VersionBumpType::Minor) {
            return PackageChangeType::SourceCode;
        }

        // Default to source code for patch bumps
        PackageChangeType::SourceCode
    }

    /// Infers change significance from version bump type
    ///
    /// Maps version bump types to change significance levels using semantic versioning logic.
    ///
    /// # Arguments
    ///
    /// * `version_bump` - The version bump type to map
    ///
    /// # Returns
    ///
    /// Appropriate `ChangeSignificance` level for the version bump
    #[must_use]
    fn infer_significance_from_version_bump(version_bump: VersionBumpType) -> ChangeSignificance {
        match version_bump {
            VersionBumpType::Major => ChangeSignificance::High,
            VersionBumpType::Minor => ChangeSignificance::Medium,
            VersionBumpType::Patch | VersionBumpType::Snapshot => ChangeSignificance::Low,
        }
    }
}
