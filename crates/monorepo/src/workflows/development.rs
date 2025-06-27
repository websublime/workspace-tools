//! Development workflow implementation
//!
//! This module provides development workflow functionality for day-to-day
//! development operations like running tests on affected packages,
//! validating changesets, and providing developer feedback.

use std::sync::Arc;
use std::time::Instant;

use crate::analysis::{AffectedPackagesAnalysis, ChangeAnalysis, MonorepoAnalyzer};
use crate::changes::{ChangeSignificance, PackageChange, PackageChangeType};
use crate::changesets::{types::ChangesetFilter, ChangesetManager, ChangesetStorage};
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
    /// Creates a new development workflow with injected dependencies
    ///
    /// # Arguments
    ///
    /// * `analyzer` - Analyzer for detecting changes and affected packages
    /// * `task_manager` - Task manager for executing development tasks  
    /// * `changeset_manager` - Changeset manager for handling development changesets
    /// * `config_provider` - Configuration provider for accessing settings
    /// * `package_provider` - Package provider for accessing package information
    /// * `git_provider` - Git provider for repository operations
    ///
    /// # Returns
    ///
    /// A new `DevelopmentWorkflow` instance ready to execute development operations.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required components cannot be initialized.
    pub fn new(
        analyzer: MonorepoAnalyzer,
        task_manager: crate::tasks::TaskManager,
        changeset_manager: crate::changesets::ChangesetManager,
        config_provider: Box<dyn crate::core::ConfigProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
        git_provider: Box<dyn crate::core::GitProvider>,
    ) -> Result<Self, Error> {
        Ok(Self { 
            analyzer, 
            task_manager, 
            changeset_manager,
            config_provider,
            package_provider,
            git_provider,
        })
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use sublime_monorepo_tools::{DevelopmentWorkflow, MonorepoProject};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
    /// let workflow = DevelopmentWorkflow::from_project(project)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_project(project: Arc<MonorepoProject>) -> Result<Self, Error> {
        use crate::core::interfaces::DependencyFactory;
        
        // Create analyzer from project to enable change detection functionality
        let analyzer = MonorepoAnalyzer::from_project(Arc::clone(&project));

        // Create task manager for changeset manager
        let task_manager_for_changeset = TaskManager::new(
            DependencyFactory::file_system_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)), // executor_package_provider
            DependencyFactory::config_provider(Arc::clone(&project)), // executor_config_provider
            DependencyFactory::git_provider(Arc::clone(&project)),
            DependencyFactory::config_provider(Arc::clone(&project)), // checker_config_provider
            DependencyFactory::package_provider(Arc::clone(&project)), // checker_package_provider
            DependencyFactory::file_system_provider(Arc::clone(&project)), // checker_file_system_provider
        )?;
        
        // Create task manager for workflow
        let task_manager = TaskManager::new(
            DependencyFactory::file_system_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)), // executor_package_provider
            DependencyFactory::config_provider(Arc::clone(&project)), // executor_config_provider
            DependencyFactory::git_provider(Arc::clone(&project)),
            DependencyFactory::config_provider(Arc::clone(&project)), // checker_config_provider
            DependencyFactory::package_provider(Arc::clone(&project)), // checker_package_provider
            DependencyFactory::file_system_provider(Arc::clone(&project)), // checker_file_system_provider
        )?;

        // Create changeset storage directly with providers
        let changeset_storage = ChangesetStorage::new(
            project.config.changesets.clone(),
            DependencyFactory::file_system_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)),
        );

        // Create changeset manager directly with components and providers
        let changeset_manager = ChangesetManager::new(
            changeset_storage,
            task_manager_for_changeset,
            DependencyFactory::config_provider(Arc::clone(&project)),
            DependencyFactory::file_system_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)),
            DependencyFactory::git_provider(Arc::clone(&project)),
        );

        Self::new(
            analyzer,
            task_manager,
            changeset_manager,
            DependencyFactory::config_provider(Arc::clone(&project)),
            DependencyFactory::package_provider(Arc::clone(&project)),
            DependencyFactory::git_provider(project),
        )
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
        let git_config = &self.config_provider.config().git;
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
                .git_provider
                .current_branch()
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
            .git_provider
            .current_branch()
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
        let thresholds = &self.config_provider.config().tasks.performance.impact_thresholds;

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
        let changesets_required = self.config_provider.config().changesets.required;

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
            let file_path = self.package_provider.root_path().join(&file.path);

            // Find which package this file belongs to by checking all packages
            for package in self.package_provider.packages() {
                let package_path = package.path();
                if file_path.starts_with(package_path) {
                    package_file_groups.entry(package.name().to_string()).or_default().push(file.clone());
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
    fn convert_facts_to_package_changes(
        &self,
        package_changes: &[PackageChange],
    ) -> Result<Vec<crate::changes::PackageChange>, Error> {
        let mut converted_changes = Vec::new();

        for package_change in package_changes {
            // Analyze change type based on file patterns
            let change_type = self.analyze_change_type(&package_change.changed_files);
            
            // Determine significance based on change type and file count
            let significance = self.analyze_change_significance(&package_change.changed_files, &change_type);
            
            // Suggest version bump based on significance and change type
            let suggested_version_bump = self.suggest_version_bump(&significance, &change_type);
            
            // Create metadata with additional information
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("total_files".to_string(), package_change.changed_files.len().to_string());
            metadata.insert("change_analysis_version".to_string(), "1.0".to_string());
            
            // Add file pattern analysis to metadata
            let file_extensions: Vec<String> = package_change.changed_files
                .iter()
                .filter_map(|f| {
                    std::path::Path::new(&f.path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.to_string())
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

    /// Analyzes the type of changes based on file patterns
    ///
    /// Examines file paths and extensions to determine the primary type of change
    /// affecting the package (source code, dependencies, configuration, etc.).
    fn analyze_change_type(&self, changed_files: &[GitChangedFile]) -> PackageChangeType {
        let mut has_source = false;
        let mut has_deps = false;
        let mut has_config = false;
        let mut has_docs = false;
        let mut has_tests = false;

        for file in changed_files {
            let path = &file.path;
            let path_lower = path.to_lowercase();
            
            // Check for dependency files
            if path_lower.contains("cargo.toml") || path_lower.contains("package.json") 
                || path_lower.contains("requirements.txt") || path_lower.contains("go.mod")
                || path_lower.contains("yarn.lock") || path_lower.contains("cargo.lock") {
                has_deps = true;
            }
            // Check for configuration files
            else if path_lower.contains("config") || path_lower.ends_with(".config.js")
                || path_lower.ends_with(".yml") || path_lower.ends_with(".yaml")
                || path_lower.ends_with(".toml") || path_lower.ends_with(".json")
                || path_lower.contains(".env") {
                has_config = true;
            }
            // Check for documentation files
            else if path_lower.contains("readme") || path_lower.contains("doc")
                || path_lower.ends_with(".md") || path_lower.contains("changelog") {
                has_docs = true;
            }
            // Check for test files
            else if path_lower.contains("test") || path_lower.contains("spec")
                || path_lower.contains("__tests__") || path_lower.ends_with("_test.rs")
                || path_lower.ends_with(".test.js") || path_lower.ends_with(".spec.js") {
                has_tests = true;
            }
            // Check for source code files
            else if path_lower.ends_with(".rs") || path_lower.ends_with(".js")
                || path_lower.ends_with(".ts") || path_lower.ends_with(".go")
                || path_lower.ends_with(".py") || path_lower.ends_with(".java")
                || path_lower.ends_with(".cpp") || path_lower.ends_with(".c") {
                has_source = true;
            }
        }

        // Priority order: Dependencies > Source > Configuration > Tests > Documentation
        if has_deps {
            PackageChangeType::Dependencies
        } else if has_source {
            PackageChangeType::SourceCode
        } else if has_config {
            PackageChangeType::Configuration
        } else if has_tests {
            PackageChangeType::Tests
        } else if has_docs {
            PackageChangeType::Documentation
        } else {
            // Default to source code if we can't determine
            PackageChangeType::SourceCode
        }
    }

    /// Analyzes the significance of changes based on files and change type
    ///
    /// Determines the impact level of changes by considering both the number
    /// of files changed and the type of changes made.
    fn analyze_change_significance(
        &self,
        changed_files: &[GitChangedFile],
        change_type: &PackageChangeType,
    ) -> ChangeSignificance {
        let file_count = changed_files.len();
        
        // Get thresholds from configuration
        let thresholds = &self.config_provider.config().tasks.performance.impact_thresholds;
        
        // Base significance on file count
        let base_significance = match file_count {
            files if files > thresholds.high_impact_files => ChangeSignificance::High,
            files if files > thresholds.medium_impact_files => ChangeSignificance::Medium,
            _ => ChangeSignificance::Low,
        };
        
        // Elevate significance based on change type
        match change_type {
            PackageChangeType::Dependencies => {
                // Dependencies changes are always at least medium significance
                match base_significance {
                    ChangeSignificance::Low => ChangeSignificance::Medium,
                    other => other,
                }
            }
            PackageChangeType::SourceCode => base_significance,
            PackageChangeType::Configuration => {
                // Configuration changes can be significant
                match base_significance {
                    ChangeSignificance::Low => ChangeSignificance::Medium,
                    other => other,
                }
            }
            PackageChangeType::Tests | PackageChangeType::Documentation => {
                // Tests and docs are typically low impact unless there are many files
                match base_significance {
                    ChangeSignificance::High => ChangeSignificance::Medium,
                    other => other,
                }
            }
        }
    }

    /// Suggests appropriate version bump based on change significance and type
    ///
    /// Provides semantic versioning recommendations based on the analyzed
    /// significance and type of changes made to the package.
    fn suggest_version_bump(
        &self,
        significance: &ChangeSignificance,
        change_type: &PackageChangeType,
    ) -> crate::config::VersionBumpType {
        match (significance, change_type) {
            // High significance changes suggest minor bumps
            (ChangeSignificance::High, _) => crate::config::VersionBumpType::Minor,
            
            // Dependencies changes suggest at least patch bumps
            (_, PackageChangeType::Dependencies) => match significance {
                ChangeSignificance::Medium | ChangeSignificance::High => crate::config::VersionBumpType::Minor,
                ChangeSignificance::Low => crate::config::VersionBumpType::Patch,
            },
            
            // Source code changes follow standard significance mapping
            (ChangeSignificance::Medium, PackageChangeType::SourceCode) => crate::config::VersionBumpType::Patch,
            (ChangeSignificance::Low, PackageChangeType::SourceCode) => crate::config::VersionBumpType::Patch,
            
            // Configuration changes can be significant
            (ChangeSignificance::Medium, PackageChangeType::Configuration) => crate::config::VersionBumpType::Patch,
            (ChangeSignificance::Low, PackageChangeType::Configuration) => crate::config::VersionBumpType::Patch,
            
            // Tests and documentation typically get patch bumps
            (_, PackageChangeType::Tests) => crate::config::VersionBumpType::Patch,
            (_, PackageChangeType::Documentation) => crate::config::VersionBumpType::Patch,
        }
    }
}
