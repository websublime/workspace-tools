//! Release workflow implementation
//!
//! This module provides the complete release workflow that orchestrates
//! change detection, version management, task execution, and deployment
//! across multiple environments.

use std::sync::Arc;
use std::time::Instant;

use super::types::{ReleaseOptions, ReleaseResult};
use crate::analysis::MonorepoAnalyzer;
use crate::changesets::ChangesetManager;
use crate::core::MonorepoProject;
use crate::error::Error;
use crate::tasks::TaskManager;
use crate::analysis::ChangeAnalysis;

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
/// let workflow = ReleaseWorkflow::new(project).await?;
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

impl ReleaseWorkflow {
    /// Creates a new release workflow
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
    /// use std::sync::Arc;
    /// use sublime_monorepo_tools::{ReleaseWorkflow, MonorepoProject};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
    /// let workflow = ReleaseWorkflow::new(project)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self, Error> {
        let analyzer = MonorepoAnalyzer::new(Arc::clone(&project));
        let changeset_manager = ChangesetManager::new(Arc::clone(&project))?;
        let task_manager = TaskManager::new(Arc::clone(&project))?;
        let version_manager = crate::core::VersionManager::new(Arc::clone(&project));

        Ok(Self { project, analyzer, version_manager, changeset_manager, task_manager })
    }

    /// Executes the complete release workflow
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
    /// # async fn example(workflow: &ReleaseWorkflow) -> Result<(), Box<dyn std::error::Error>> {
    /// let options = ReleaseOptions {
    ///     dry_run: false,
    ///     skip_tests: false,
    ///     target_environments: vec!["production".to_string()],
    ///     ..Default::default()
    /// };
    ///
    /// let result = workflow.execute(options).await?;
    /// println!("Release success: {}", result.success);
    /// # Ok(())
    /// # }
    /// ```
    // TODO: Consider breaking this method into smaller parts for better readability and maintainability
    #[allow(clippy::too_many_lines)]
    pub async fn execute(&self, options: ReleaseOptions) -> Result<ReleaseResult, Error> {
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

        // Step 3: Execute release tasks
        let tasks = if options.skip_tests {
            warnings.push("Skipping tests as requested".to_string());
            Vec::new()
        } else {
            match self.execute_release_tasks(&changes, &options).await {
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
                        errors.push(format!("Release tasks failed: {} tasks", failed_tasks.len()));
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
            match self.deploy_to_environments(&options.target_environments).await {
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

    /// Detects changes since the last release
    fn detect_changes_since_last_release(&self) -> Result<ChangeAnalysis, Error> {
        // Get the last release tag
        let last_tag = self
            .project
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
            .project
            .repository
            .get_current_branch()
            .map_err(|e| Error::workflow(format!("Failed to get current branch: {e}")))?;

        // Apply changesets for this branch
        self.changeset_manager.apply_changesets_on_merge(&current_branch)
    }

    /// Executes all release-related tasks
    async fn execute_release_tasks(
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

        // Execute tasks for affected packages
        self.task_manager.execute_tasks_for_affected_packages(&affected_packages).await
    }

    /// Deploys to the specified environments
    async fn deploy_to_environments(&self, environments: &[String]) -> Result<(), Error> {
        // Execute deployment tasks for each environment using TaskManager
        for env in environments {
            // Get deployment tasks for this environment
            let deployment_tasks = self.get_deployment_tasks_for_environment(env)?;

            if deployment_tasks.is_empty() {
                log::info!("No deployment tasks configured for environment: {}", env);
                continue;
            }

            // Execute deployment tasks using TaskManager
            let task_results = self.task_manager.execute_tasks_batch(&deployment_tasks).await?;

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
    fn get_deployment_tasks_for_environment(
        &self,
        environment: &str,
    ) -> Result<Vec<String>, Error> {
        // Parse environment string to Environment enum if possible
        let environment_enum = self.parse_environment_string(environment);

        // Try to get deployment tasks from project configuration first
        let configured_tasks = self.project.config.tasks.deployment_tasks.get(&environment_enum);

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
    // TODO: implement when changelog manager is available (fase 5)
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::unused_self)]
    fn generate_release_changelogs(&self, _changes: &ChangeAnalysis) -> Result<(), Error> {
        // This would generate changelogs using the changelog manager
        // For now, simulate success
        Ok(())
    }
}
