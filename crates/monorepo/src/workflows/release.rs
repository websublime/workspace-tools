//! Release workflow implementation
//!
//! This module provides the complete release workflow that orchestrates
//! change detection, version management, task execution, and deployment
//! across multiple environments.

use std::time::Instant;

use super::types::{ReleaseOptions, ReleaseResult};
use crate::analysis::ChangeAnalysis;
use crate::analysis::MonorepoAnalyzer;
use crate::core::MonorepoProject;
use crate::error::Error;

/// Release workflow orchestrator
///
/// Manages the complete release process from change detection through
/// deployment, integrating all necessary components to ensure a smooth
/// and reliable release.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use sublime_monorepo_tools::{ReleaseWorkflow, ReleaseOptions, MonorepoProject};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
/// let workflow = ReleaseWorkflow::from_project(project)?;
///
/// let options = ReleaseOptions::default();
/// let result = workflow.execute(options).await?;
///
/// if result.success {
///     println!("Release completed successfully!");
/// }
/// # Ok(())
/// # }
/// ```
// Import struct definition from types module
use crate::workflows::types::ReleaseWorkflow;

/// Configuration for creating a ReleaseWorkflow
///
/// Groups all the dependencies needed to create a release workflow
/// to avoid too_many_arguments clippy warning and improve API usability.
pub struct ReleaseWorkflowConfig<'a> {
    /// Analyzer for detecting changes and affected packages
    pub analyzer: MonorepoAnalyzer<'a>,
    /// Changeset manager for applying production changesets
    pub changeset_manager: crate::changesets::ChangesetManager<'a>,
    /// Task manager for executing release tasks
    pub task_manager: crate::tasks::TaskManager<'a>,
    /// Plugin manager for extensible release functionality
    pub plugin_manager: crate::plugins::PluginManager<'a>,
    /// Direct reference to configuration
    pub config: &'a crate::config::MonorepoConfig,
    /// Direct reference to packages
    pub packages: &'a [crate::core::MonorepoPackageInfo],
    /// Direct reference to git repository
    pub repository: &'a sublime_git_tools::Repo,
    /// Direct reference to file system manager
    pub file_system: &'a sublime_standard_tools::filesystem::FileSystemManager,
    /// Direct reference to root path
    pub root_path: &'a std::path::Path,
}

impl<'a> ReleaseWorkflow<'a> {
    /// Creates a new release workflow with configuration struct
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles. Accepts a configuration struct
    /// to avoid too_many_arguments issues and improve maintainability.
    ///
    /// # Arguments
    ///
    /// * `config` - Release workflow configuration containing all dependencies
    ///
    /// # Returns
    ///
    /// A new release workflow instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::workflows::{ReleaseWorkflow, ReleaseWorkflowConfig};
    /// use sublime_monorepo_tools::analysis::MonorepoAnalyzer;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ReleaseWorkflowConfig {
    ///     analyzer: MonorepoAnalyzer::new(&project),
    ///     changeset_manager: changeset_manager,
    ///     task_manager: task_manager,
    ///     config: &project.config,
    ///     packages: &project.packages,
    ///     repository: &project.repository,
    ///     root_path: &project.root_path,
    /// };
    /// let workflow = ReleaseWorkflow::new(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: ReleaseWorkflowConfig<'a>) -> Self {
        Self {
            analyzer: config.analyzer,
            changeset_manager: config.changeset_manager,
            task_manager: config.task_manager,
            plugin_manager: config.plugin_manager,
            config: config.config,
            packages: config.packages,
            repository: config.repository,
            file_system: config.file_system,
            root_path: config.root_path,
        }
    }


    /// Creates a new release workflow from project (convenience method)
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    ///
    /// # Returns
    ///
    /// A new `ReleaseWorkflow` instance ready to execute releases.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required components cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{ReleaseWorkflow, MonorepoProject};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let workflow = ReleaseWorkflow::from_project(&project)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_project(project: &'a MonorepoProject) -> Result<Self, Error> {
        use crate::analysis::MonorepoAnalyzer;
        use crate::changesets::{ChangesetManager, ChangesetStorage};
        use crate::core::VersionManager;
        use crate::tasks::TaskManager;

        // Create analyzer with direct borrowing
        let analyzer = MonorepoAnalyzer::new(project);

        // Create version manager with direct borrowing
        let _version_manager = VersionManager::new(project);
        // Version manager already created above

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

        // Create plugin manager with direct borrowing
        let plugin_manager = crate::plugins::PluginManager::from_project(project)?;

        let config = ReleaseWorkflowConfig {
            analyzer,
            changeset_manager,
            task_manager,
            plugin_manager,
            config: &project.config,
            packages: &project.packages,
            repository: &project.repository,
            file_system: &project.file_system,
            root_path: &project.root_path,
        };

        Ok(Self::new(config))
    }

    /// Executes the complete release workflow synchronously
    ///
    /// This method orchestrates the entire release process:
    /// 1. Detects changes since the last release
    /// 2. Applies pending changesets
    /// 3. Executes release tasks (tests, builds, etc.)
    /// 4. Deploys to target environments
    /// 5. Updates version numbers and generates changelogs
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration options for the release
    ///
    /// # Returns
    ///
    /// Complete release result with success status and details.
    ///
    /// # Errors
    ///
    /// Returns an error if any critical step of the release fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::ReleaseOptions;
    ///
    /// # fn example(workflow: &ReleaseWorkflow) -> Result<(), Box<dyn std::error::Error>> {
    /// let options = ReleaseOptions {
    ///     dry_run: false,
    ///     skip_tests: false,
    ///     target_environments: vec!["production".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let result = workflow.execute(options)?;
    /// println!("Release success: {}", result.success);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// This method orchestrates multiple complex operations in sequence. The current
    /// implementation prioritizes clear step-by-step execution flow over method decomposition
    /// to maintain workflow clarity and reduce callback complexity.
    #[allow(clippy::too_many_lines)]
    pub fn execute(&self, options: &ReleaseOptions) -> Result<ReleaseResult, Error> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut success = true;

        // Step 1: Detect changes since last release
        let changes = match self.detect_changes_since_last_release() {
            Ok(changes) => {
                if changes.changed_files.is_empty() {
                    warnings.push("No changes detected since last release".to_string());
                }
                changes
            }
            Err(e) => {
                errors.push(format!("Failed to detect changes: {e}"));
                // success = false;
                return Ok(ReleaseResult {
                    changes: ChangeAnalysis::default(),
                    tasks: Vec::new(),
                    changesets_applied: Vec::new(),
                    success: false,
                    duration: start_time.elapsed(),
                    errors,
                    warnings,
                });
            }
        };

        // Step 1.5: Execute pre-release plugin hooks
        self.execute_plugin_hooks("pre-release-validation", &changes, options);

        // Step 2: Apply pending changesets
        let changesets_applied = if options.dry_run {
            warnings.push("Dry run: Skipping changeset application".to_string());
            Vec::new()
        } else {
            match self.apply_pending_changesets() {
                Ok(applications) => {
                    if applications.is_empty() {
                        warnings.push("No changesets to apply".to_string());
                    }
                    applications
                }
                Err(e) => {
                    errors.push(format!("Failed to apply changesets: {e}"));
                    success = false;
                    Vec::new()
                }
            }
        };

        // Step 2.5: Execute post-changeset plugin hooks
        self.execute_plugin_hooks("post-changeset-application", &changes, options);

        // Step 3: Execute release tasks
        let tasks = if options.skip_tests {
            warnings.push("Skipping tests as requested".to_string());
            Vec::new()
        } else {
            match self.execute_release_tasks_sync(&changes, options) {
                Ok(task_results) => {
                    // Check if any tasks failed
                    let failed_tasks: Vec<_> = task_results
                        .iter()
                        .filter(|result| {
                            !matches!(
                                result.status,
                                crate::tasks::types::results::TaskStatus::Success
                            )
                        })
                        .collect();

                    if !failed_tasks.is_empty() && !options.force {
                        errors.push(format!(
                            "Release tasks failed: {count} tasks",
                            count = failed_tasks.len()
                        ));
                        success = false;
                    }

                    task_results
                }
                Err(e) => {
                    errors.push(format!("Failed to execute release tasks: {e}"));
                    success = false;
                    Vec::new()
                }
            }
        };

        // Step 4: Deploy to target environments
        if !options.dry_run && success {
            match self.deploy_to_environments_sync(&options.target_environments) {
                Ok(()) => {
                    // Deployment successful
                }
                Err(e) => {
                    errors.push(format!("Deployment failed: {e}"));
                    success = false;
                }
            }
        } else if options.dry_run {
            warnings.push("Dry run: Skipping deployment".to_string());
        }

        // Step 5: Generate changelogs (if not skipped)
        if !options.skip_changelogs && !options.dry_run && success {
            match self.generate_release_changelogs(&changes) {
                Ok(()) => {
                    // Changelog generation successful
                }
                Err(e) => {
                    warnings.push(format!("Changelog generation failed: {e}"));
                    // Not a critical failure for the release
                }
            }
        }

        // Final step: Execute post-release plugin hooks
        self.execute_plugin_hooks("post-release-completion", &changes, options);

        Ok(ReleaseResult {
            changes,
            tasks,
            changesets_applied,
            success,
            duration: start_time.elapsed(),
            errors,
            warnings,
        })
    }

    /// Executes the complete release workflow

    /// Detects changes since the last release
    fn detect_changes_since_last_release(&self) -> Result<ChangeAnalysis, Error> {
        // Get the last release tag
        let last_tag = self
            .repository
            .get_last_tag()
            .map_err(|e| Error::workflow(format!("Failed to get last tag: {e}")))?;

        // Analyze changes since that tag
        self.analyzer.detect_changes_since(&last_tag, None)
    }

    /// Applies all pending changesets for the current branch
    fn apply_pending_changesets(
        &self,
    ) -> Result<Vec<crate::changesets::types::ChangesetApplication>, Error> {
        // Get current branch
        let current_branch = self
            .repository
            .get_current_branch()
            .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?;

        // Apply changesets for this branch
        self.changeset_manager.apply_changesets_on_merge(&current_branch)
    }

    /// Executes all release-related tasks synchronously
    fn execute_release_tasks_sync(
        &self,
        changes: &ChangeAnalysis,
        _options: &ReleaseOptions,
    ) -> Result<Vec<crate::tasks::TaskExecutionResult>, Error> {
        // Get affected packages
        let affected_packages: Vec<String> =
            changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

        if affected_packages.is_empty() {
            return Ok(Vec::new());
        }

        // Execute tasks for affected packages using sync version
        self.task_manager.execute_tasks_for_affected_packages(&affected_packages)
    }

    /// Deploys to the specified environments synchronously
    fn deploy_to_environments_sync(&self, environments: &[String]) -> Result<(), Error> {
        // Execute deployment tasks for each environment using TaskManager
        for env in environments {
            // Get deployment tasks for this environment
            let deployment_tasks = self.get_deployment_tasks_for_environment(env)?;

            if deployment_tasks.is_empty() {
                log::info!("No deployment tasks configured for environment: {}", env);
                continue;
            }

            // Execute deployment tasks using sync TaskManager
            let task_results = self.task_manager.execute_tasks_batch(&deployment_tasks)?;

            // Check if all tasks succeeded
            let failed_tasks: Vec<_> = task_results
                .iter()
                .filter(|result| {
                    !matches!(result.status, crate::tasks::types::results::TaskStatus::Success)
                })
                .collect();

            if !failed_tasks.is_empty() {
                let error_msg = failed_tasks
                    .iter()
                    .map(|task| {
                        format!(
                            "{}: {}",
                            task.task_name,
                            task.errors.first().map_or("Unknown error", |e| &e.message)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(Error::workflow(format!("Deployment to {env} failed: {error_msg}")));
            }

            log::info!("✅ Successfully deployed to environment: {}", env);
        }

        Ok(())
    }

    /// Gets deployment tasks for a specific environment
    ///
    /// Uses real project configuration and validates that tasks actually exist.
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn get_deployment_tasks_for_environment(
        &self,
        environment: &str,
    ) -> Result<Vec<String>, Error> {
        // Parse environment string to Environment enum if possible
        let environment_enum = self.parse_environment_string(environment);

        // Try to get deployment tasks from project configuration first
        let configured_tasks = self.config.tasks.deployment_tasks.get(&environment_enum);

        let candidate_tasks = if let Some(tasks) = configured_tasks {
            // Use configured tasks for this environment
            tasks.clone()
        } else {
            // Fallback to reasonable defaults based on environment name
            self.get_default_deployment_tasks(environment)
        };

        // Get list of actually available tasks from TaskManager
        let available_tasks = self.task_manager.list_tasks();
        let available_task_names: std::collections::HashSet<String> =
            available_tasks.iter().map(|task| task.name.clone()).collect();

        // Filter to only include tasks that actually exist
        let valid_tasks: Vec<String> = candidate_tasks
            .into_iter()
            .filter(|task_name| available_task_names.contains(task_name))
            .collect();

        if valid_tasks.is_empty() {
            log::warn!(
                "No valid deployment tasks found for environment '{}'. Available tasks: {:?}",
                environment,
                available_task_names
            );
        } else {
            log::info!(
                "Found {} valid deployment task(s) for environment '{}': {:?}",
                valid_tasks.len(),
                environment,
                valid_tasks
            );
        }

        Ok(valid_tasks)
    }

    /// Parses environment string to Environment enum
    #[allow(clippy::unused_self)]
    fn parse_environment_string(&self, env_str: &str) -> crate::config::types::Environment {
        match env_str.to_lowercase().as_str() {
            "development" | "dev" => crate::config::types::Environment::Development,
            "staging" | "stage" => crate::config::types::Environment::Staging,
            "integration" | "int" => crate::config::types::Environment::Integration,
            "production" | "prod" => crate::config::types::Environment::Production,
            custom => crate::config::types::Environment::Custom(custom.to_string()),
        }
    }

    /// Gets sensible default tasks when environment is not configured
    #[allow(clippy::unused_self)]
    fn get_default_deployment_tasks(&self, environment: &str) -> Vec<String> {
        match environment.to_lowercase().as_str() {
            "development" | "dev" | "integration" | "int" => {
                vec!["build".to_string(), "test".to_string()]
            }
            "staging" | "stage" | "production" | "prod" => {
                vec!["build".to_string(), "test".to_string(), "lint".to_string()]
            }
            _ => {
                log::info!("Using minimal default tasks for custom environment: {}", environment);
                vec!["build".to_string()]
            }
        }
    }

    /// Generates changelogs for the release
    ///
    /// Creates comprehensive changelogs for all affected packages using conventional commits
    /// and package-specific change analysis. Each package gets its own changelog entry
    /// with proper version information and commit grouping.
    ///
    /// # Arguments
    ///
    /// * `changes` - The change analysis containing package changes and affected files
    ///
    /// # Returns
    ///
    /// Success if all changelogs were generated successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Changelog manager cannot be created
    /// - Commit history cannot be retrieved
    /// - Changelog files cannot be written
    /// - Package information is invalid
    fn generate_release_changelogs(&self, changes: &ChangeAnalysis) -> Result<(), Error> {
        use crate::changelog::{ChangelogManager, ChangelogRequest};

        log::info!("Starting changelog generation for {} packages", changes.package_changes.len());

        // Create changelog manager with direct component borrowing (no Arc/Rc needed)
        let changelog_manager = ChangelogManager::with_components(
            self.config,
            self.packages,
            self.repository,
            self.file_system,
            self.root_path,
        );

        // Generate changelog for each affected package
        for package_change in &changes.package_changes {
            log::info!("Generating changelog for package: {}", package_change.package_name);

            // Determine version based on suggested version bump
            let next_version = self.calculate_next_version(
                &package_change.package_name,
                package_change.suggested_version_bump,
            )?;

            // Create changelog request for this package
            let request = ChangelogRequest {
                package_name: Some(package_change.package_name.clone()),
                version: next_version.clone(),
                since: Some(changes.from_ref.clone()),
                until: Some(changes.to_ref.clone()),
                write_to_file: true,
                include_all_commits: false,
                output_path: None, // Use default path
            };

            // Generate the changelog using sync version - eliminates runtime creation anti-pattern
            match changelog_manager.generate_changelog(request) {
                Ok(changelog_result) => {
                    log::info!(
                        "Successfully generated changelog for {}: {} commits, {} breaking changes",
                        package_change.package_name,
                        changelog_result.commit_count,
                        if changelog_result.has_breaking_changes { "has" } else { "no" }
                    );

                    if let Some(output_path) = &changelog_result.output_path {
                        log::debug!("Changelog written to: {}", output_path);
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to generate changelog for package {}: {}",
                        package_change.package_name,
                        e
                    );
                    return Err(Error::workflow(format!(
                        "Changelog generation failed for package {}: {e}",
                        package_change.package_name
                    )));
                }
            }
        }

        // Generate root changelog if there are multiple packages
        if changes.package_changes.len() > 1 {
            log::info!("Generating root monorepo changelog");

            let root_request = ChangelogRequest {
                package_name: None, // Root changelog
                version: "latest".to_string(),
                since: Some(changes.from_ref.clone()),
                until: Some(changes.to_ref.clone()),
                write_to_file: true,
                include_all_commits: false,
                output_path: None,
            };

            // Generate root changelog using sync version - eliminates runtime creation anti-pattern
            let _result = changelog_manager.generate_changelog(root_request)?;
        }

        log::info!("Changelog generation completed for all packages");
        Ok(())
    }

    /// Calculate the next version for a package based on the version bump type
    ///
    /// This method determines the next semantic version based on the suggested
    /// version bump type from the change analysis.
    pub(crate) fn calculate_next_version(
        &self,
        package_name: &str,
        version_bump: crate::VersionBumpType,
    ) -> Result<String, Error> {
        // Get current version of the package
        let packages = self.packages;
        let package = packages
            .iter()
            .find(|p| p.name() == package_name)
            .ok_or_else(|| Error::workflow(format!("Package '{package_name}' not found")))?;

        let current_version = &package.workspace_package.version;

        // Parse current version to increment appropriately
        let version_parts: Vec<&str> = current_version.split('.').collect();
        if version_parts.len() != 3 {
            return Err(Error::workflow(format!(
                "Invalid version format for package {package_name}: {current_version}"
            )));
        }

        let major: u32 = version_parts[0].parse().map_err(|_| {
            Error::workflow(format!("Invalid major version: {version}", version = version_parts[0]))
        })?;
        let minor: u32 = version_parts[1].parse().map_err(|_| {
            Error::workflow(format!("Invalid minor version: {version}", version = version_parts[1]))
        })?;
        let patch: u32 = version_parts[2].parse().map_err(|_| {
            Error::workflow(format!("Invalid patch version: {version}", version = version_parts[2]))
        })?;

        let next_version = match version_bump {
            crate::VersionBumpType::Major => format!("{major_new}.0.0", major_new = major + 1),
            crate::VersionBumpType::Minor => {
                format!("{major}.{minor_new}.0", minor_new = minor + 1)
            }
            crate::VersionBumpType::Patch => {
                format!("{major}.{minor}.{patch_new}", patch_new = patch + 1)
            }
            crate::VersionBumpType::Snapshot => {
                format!("{major}.{minor}.{patch_new}-SNAPSHOT", patch_new = patch + 1)
            }
        };

        log::debug!(
            "Version bump for {}: {} -> {} ({})",
            package_name,
            current_version,
            next_version,
            match version_bump {
                crate::VersionBumpType::Major => "major",
                crate::VersionBumpType::Minor => "minor",
                crate::VersionBumpType::Patch => "patch",
                crate::VersionBumpType::Snapshot => "snapshot",
            }
        );

        Ok(next_version)
    }

    /// Execute plugin hooks for release workflow extension points
    ///
    /// Provides extensible hook points where plugins can customize release process,
    /// add validation, deployment integration, and post-release actions.
    ///
    /// # Arguments
    ///
    /// * `hook_name` - Name of the hook to execute (e.g., "pre-release-validation")
    /// * `changes` - Change analysis context for plugins
    /// * `options` - Release options for plugin context
    fn execute_plugin_hooks(&self, hook_name: &str, changes: &ChangeAnalysis, options: &ReleaseOptions) {
        // Convert release context to plugin arguments
        let args = vec![
            format!("--changed-files={}", changes.changed_files.len()),
            format!("--packages={}", changes.package_changes.len()),
            format!("--dry-run={}", options.dry_run),
            format!("--skip-tests={}", options.skip_tests),
            format!("--environments={}", options.target_environments.join(",")),
        ];

        // Get all loaded plugins and execute the hook
        let plugin_names: Vec<String> = self.plugin_manager.list_plugins()
            .iter()
            .map(|info| info.name.clone())
            .collect();

        for plugin_name in plugin_names {
            // Execute plugin command with hook name and release context
            match self.plugin_manager.execute_plugin_command(&plugin_name, hook_name, &args) {
                Ok(result) => {
                    if result.success {
                        log::info!(
                            "Plugin '{}' release hook '{}' executed successfully",
                            plugin_name,
                            hook_name
                        );
                        
                        // Process plugin results for specific hooks
                        match hook_name {
                            "pre-release-validation" => {
                                if let Some(validations) = result.data.get("validations") {
                                    log::info!("Plugin '{}' validation results: {}", plugin_name, validations);
                                }
                            }
                            "post-release-completion" => {
                                if let Some(notifications) = result.data.get("notifications") {
                                    log::info!("Plugin '{}' post-release notifications: {}", plugin_name, notifications);
                                }
                            }
                            _ => {
                                // Generic processing for other hooks
                                if !result.data.is_null() {
                                    log::debug!("Plugin '{}' hook '{}' data: {}", plugin_name, hook_name, result.data);
                                }
                            }
                        }
                    } else {
                        log::warn!(
                            "Plugin '{}' release hook '{}' reported failure: {}",
                            plugin_name,
                            hook_name,
                            result.error.unwrap_or_else(|| "No error message".to_string())
                        );
                    }
                }
                Err(e) => {
                    // Log plugin errors but don't fail the entire release
                    log::warn!(
                        "Plugin '{}' failed to execute release hook '{}': {}",
                        plugin_name,
                        hook_name,
                        e
                    );
                }
            }
        }
    }
}
