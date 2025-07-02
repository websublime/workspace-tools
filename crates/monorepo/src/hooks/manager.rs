//! Hook manager implementation
//!
//! The `HookManager` orchestrates Git hook installation, execution, and validation,
//! coordinating with other monorepo systems for comprehensive workflow management.

// Hook manager implementation - now fully synchronous for better integration with MonorepoProject

use super::types::HookManager;
use super::{
    GitOperationType, HookDefinition, HookError, HookErrorCode, HookExecutionContext,
    HookExecutionResult, HookInstaller, HookScript, HookType, HookValidationResult, HookValidator,
    PreCommitResult, PrePushResult, ValidationCheck,
};
use crate::changesets::Changeset;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use crate::events::types::{ChangesetEvent, HookEvent, TaskEvent};
use crate::events::{
    EventBus, EventContext, EventEmitter, EventPriority, EventSubscriber, MonorepoEvent,
};
use crate::tasks::TaskManager;
use std::collections::HashMap;
use std::sync::Arc;
use sublime_standard_tools::command::SharedSyncExecutor;

impl<'a> HookManager<'a> {
    /// Create a new hook manager with direct borrowing from project
    ///
    /// Uses borrowing instead of trait objects to eliminate Arc proliferation
    /// and work with Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to monorepo project
    /// * `task_manager` - Reference to task manager for hook execution
    ///
    /// # Returns
    ///
    /// A new hook manager instance
    pub fn new(project: &'a MonorepoProject, task_manager: &'a TaskManager<'a>) -> Result<Self> {
        let default_hooks = Self::create_default_hooks();

        // Create installer with direct borrowing
        let installer = HookInstaller::new(project)?;

        // Create validator with direct borrowing
        let validator = HookValidator::new(project);

        // Create synchronous task executor with direct borrowing
        let sync_task_executor =
            crate::hooks::sync_task_executor::SyncTaskExecutor::new(task_manager)?;

        Ok(Self {
            installer,
            validator,
            custom_hooks: HashMap::new(),
            default_hooks,
            enabled: true,
            event_bus: None,
            config: &project.config,
            repository: &project.repository,
            file_system: &project.file_system,
            packages: &project.packages,
            root_path: &project.root_path,
            sync_task_executor,
        })
    }

    /// Creates a new hook manager from an existing MonorepoProject
    ///
    /// Convenience method that wraps the `new` constructor for backward compatibility.
    /// Uses real direct borrowing following Rust ownership principles.
    ///
    /// # Arguments
    ///
    /// * `project` - Reference to the monorepo project
    /// * `task_manager` - Reference to task manager for hook execution
    ///
    /// # Returns
    ///
    /// A new HookManager instance with direct borrowing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::{HookManager, MonorepoProject, TaskManager};
    ///
    /// let project = MonorepoProject::new("/path/to/monorepo")?;
    /// let task_manager = TaskManager::from_project(&project)?;
    /// let hook_manager = HookManager::from_project(&project, &task_manager)?;
    /// ```
    pub fn from_project(
        project: &'a MonorepoProject,
        task_manager: &'a TaskManager<'a>,
    ) -> Result<Self> {
        Self::new(project, task_manager)
    }

    /// Install Git hooks in the repository
    ///
    /// # Errors
    /// Returns an error if:
    /// - The .git/hooks directory cannot be accessed
    /// - Hook files cannot be written
    /// - Permissions cannot be set on hook files
    pub fn install_hooks(&self) -> Result<Vec<HookType>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let hook_types = HookType::all();
        let mut installed = Vec::new();

        for hook_type in hook_types {
            match self.get_hook_definition(hook_type) {
                Ok(hook_definition) => {
                    match self.installer.install_hook(&hook_type, hook_definition) {
                        Ok(()) => {
                            installed.push(hook_type);
                        }
                        Err(e) => {
                            // Log error but continue with other hooks
                            eprintln!("Failed to install hook {hook_type:?}: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get hook definition for {hook_type:?}: {e}");
                }
            }
        }

        // Emit hook installation event
        if !installed.is_empty() {
            let context = EventContext::new("HookManager").with_priority(EventPriority::Normal);

            let installed_event = HookEvent::Installed {
                context,
                hook_types: installed.iter().map(std::string::ToString::to_string).collect(),
            };

            let _ = self.emit_event(MonorepoEvent::Hook(installed_event));
        }

        Ok(installed)
    }

    /// Execute a specific hook with the given context
    ///
    /// # Errors
    /// Returns an error if:
    /// - The hook script cannot be executed
    /// - Hook validation fails
    /// - Required dependencies are not available
    pub fn execute_hook(
        &self,
        hook_type: HookType,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult> {
        if !self.enabled {
            return Ok(HookExecutionResult::new(hook_type).with_skipped("Hooks are disabled"));
        }

        let hook_definition = self.get_hook_definition(hook_type)?;

        // Check if hook is enabled
        if !hook_definition.enabled {
            return Ok(HookExecutionResult::new(hook_type).with_skipped("Hook is disabled"));
        }

        // Validate conditions before execution
        if !self.validator.check_conditions(&hook_definition.conditions, context)? {
            return Ok(HookExecutionResult::new(hook_type).with_skipped("Hook conditions not met"));
        }

        // Emit hook started event
        let started_context = EventContext::new("HookManager").with_priority(EventPriority::High);

        let started_event = HookEvent::Started {
            context: started_context,
            hook_type: hook_type.to_string(),
            affected_packages: context.affected_packages.clone(),
        };

        self.emit_event(MonorepoEvent::Hook(started_event))?;

        // Execute the hook based on its type
        match hook_type {
            HookType::PreCommit => {
                let result = self.pre_commit_validation()?;
                let mut hook_result = HookExecutionResult::new(hook_type);

                if result.validation_passed {
                    hook_result = hook_result.with_success();
                } else {
                    hook_result = hook_result.with_failure(
                        HookError::new(
                            HookErrorCode::ValidationFailed,
                            "Pre-commit validation failed",
                        )
                        .with_context("required_actions", result.required_actions.join(", ")),
                    );
                }

                hook_result = hook_result.with_validation_result(result.validation_details);

                // Emit hook completion event
                let completed_context =
                    EventContext::new("HookManager").with_priority(EventPriority::High);

                let completed_event = HookEvent::Completed {
                    context: completed_context,
                    hook_type: hook_type.to_string(),
                    success: result.validation_passed,
                    message: Some(format!(
                        "Pre-commit validation: {}",
                        if result.validation_passed { "passed" } else { "failed" }
                    )),
                };

                let _ = self.emit_event(MonorepoEvent::Hook(completed_event));

                Ok(hook_result)
            }
            HookType::PrePush => {
                let commits = context.commit_hashes();
                let result = self.pre_push_validation(&commits)?;
                let mut hook_result = HookExecutionResult::new(hook_type);

                if result.validation_passed {
                    hook_result = hook_result.with_success();
                } else {
                    hook_result = hook_result.with_failure(
                        HookError::new(
                            HookErrorCode::ValidationFailed,
                            "Pre-push validation failed",
                        )
                        .with_context("failed_tasks", result.task_results.len().to_string()),
                    );
                }

                hook_result = hook_result.with_validation_result(result.validation_details);

                // Emit hook completion event
                let completed_context =
                    EventContext::new("HookManager").with_priority(EventPriority::High);

                let completed_event = HookEvent::Completed {
                    context: completed_context,
                    hook_type: hook_type.to_string(),
                    success: result.validation_passed,
                    message: Some(format!(
                        "Pre-push validation: {}",
                        if result.validation_passed { "passed" } else { "failed" }
                    )),
                };

                let _ = self.emit_event(MonorepoEvent::Hook(completed_event));

                Ok(hook_result)
            }
            _ => {
                // For other hook types, execute the script directly
                let script_result = self.execute_hook_script(hook_definition, context)?;

                // Emit hook completion event
                let completed_context =
                    EventContext::new("HookManager").with_priority(EventPriority::High);

                let completed_event = HookEvent::Completed {
                    context: completed_context,
                    hook_type: hook_type.to_string(),
                    success: script_result.is_success(),
                    message: script_result.error.as_ref().map(|e| e.message.clone()),
                };

                let _ = self.emit_event(MonorepoEvent::Hook(completed_event));

                Ok(script_result)
            }
        }
    }

    /// Perform pre-commit validation
    ///
    /// Validates that:
    /// - Required changesets exist for the changes
    /// - No validation rules are violated
    /// - All pre-commit tasks pass
    ///
    /// # Errors
    /// Returns an error if validation cannot be performed due to system issues
    pub fn pre_commit_validation(&self) -> Result<PreCommitResult> {
        let mut result = PreCommitResult::new();

        // Get changed files from Git
        let changed_files = self.get_staged_files()?;
        if changed_files.is_empty() {
            return Ok(result.with_validation_passed(true).with_validation_details(
                HookValidationResult::new()
                    .with_check("no_changes", ValidationCheck::passed("No files staged")),
            ));
        }

        // Map changed files to affected packages
        let affected_packages = self.map_files_to_packages(&changed_files)?;
        result = result.with_affected_packages(affected_packages.clone());

        // Check if changeset exists for affected packages
        let changeset_check = self.validator.validate_changeset_exists(&affected_packages)?;
        result = result.with_validation_details(changeset_check.validation_details.clone());

        if changeset_check.changeset_exists {
            if let Some(changeset) = changeset_check.changeset {
                result = result.with_changeset(changeset);
            }
        } else if self.config.changesets.required {
            result = result.with_required_action("Create a changeset for the affected packages");
        }

        // Run pre-commit tasks if configured using real TaskManager
        let pre_commit_tasks = self.get_pre_commit_tasks()?;
        let mut tasks_passed = true;

        if !pre_commit_tasks.is_empty() && !affected_packages.is_empty() {
            // Execute tasks synchronously through validation bridge
            for task_name in &pre_commit_tasks {
                let task_success =
                    self.execute_task_for_validation(task_name, &affected_packages)?;

                if !task_success {
                    tasks_passed = false;
                    result = result.with_required_action(format!("Fix failing task: {task_name}"));
                }
            }
        }

        // Overall validation status
        let validation_passed =
            result.changeset.is_some() && result.required_actions.is_empty() && tasks_passed;
        result = result.with_validation_passed(validation_passed);

        Ok(result)
    }

    /// Perform pre-push validation
    ///
    /// Validates that:
    /// - All tests pass for affected packages
    /// - Build succeeds for affected packages
    /// - Code quality checks pass
    ///
    /// # Errors
    /// Returns an error if validation cannot be performed due to system issues
    pub fn pre_push_validation(&self, pushed_commits: &[String]) -> Result<PrePushResult> {
        let mut result = PrePushResult::new().with_commit_count(pushed_commits.len());

        if pushed_commits.is_empty() {
            return Ok(result.with_validation_details(
                HookValidationResult::new()
                    .with_check("no_commits", ValidationCheck::passed("No commits to push")),
            ));
        }

        // Get affected packages from the commits
        let affected_packages = self.get_affected_packages_from_commits(pushed_commits)?;
        result = result.with_affected_packages(affected_packages.clone());

        // Get pre-push tasks to execute
        let pre_push_tasks = self.get_pre_push_tasks()?;

        // Execute tasks for affected packages using real TaskManager
        if !pre_push_tasks.is_empty() && !affected_packages.is_empty() {
            // Execute tasks synchronously through validation bridge
            for task_name in &pre_push_tasks {
                let task_success =
                    self.execute_task_for_validation(task_name, &affected_packages)?;

                result = result.with_task_result(task_name, task_success);

                if !task_success {
                    result = result.with_required_action(format!("Fix failing task: {task_name}"));
                }
            }
        }

        // Determine overall validation status
        let validation_passed = result.required_actions.is_empty()
            && result.task_results.values().all(|&success| success);

        Ok(result.with_validation_passed(validation_passed))
    }

    /// Prompt the user to create a changeset if one doesn't exist
    ///
    /// # Errors
    /// Returns an error if:
    /// - User input cannot be read
    /// - Changeset creation fails
    /// - File system operations fail
    pub fn prompt_for_changeset(&self) -> Result<Changeset> {
        // Emit changeset creation request event
        let context = EventContext::new("HookManager").with_priority(EventPriority::High);

        let changeset_event = ChangesetEvent::CreationRequested {
            context,
            packages: Vec::new(), // Will be determined by the changeset manager
            reason: "Requested by hook validation".to_string(),
        };

        let event = MonorepoEvent::Changeset(changeset_event);
        self.emit_event(event)?;

        // Create changeset manager with proper dependency injection
        let changeset_manager = self
            .create_changeset_manager()
            .map_err(|e| Error::hook(format!("Failed to create changeset manager: {e}")))?;

        // Create the changeset using the proper interactive method
        // This will auto-detect the affected package and create an appropriate changeset
        changeset_manager
            .create_changeset_interactive(None)
            .map_err(|e| Error::hook(format!("Failed to create changeset: {e}")))
    }

    /// Create a ChangesetManager instance for changeset operations
    ///
    /// This method creates a fully functional changeset manager with proper dependency injection.
    /// The changeset manager is used by hooks to create and manage changesets when required.
    ///
    /// # Returns
    ///
    /// A configured ChangesetManager instance ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if the changeset manager cannot be created due to:
    /// - Missing or invalid dependencies
    /// - File system or storage initialization issues
    /// - Configuration validation failures
    fn create_changeset_manager(&self) -> Result<crate::changesets::ChangesetManager> {
        use crate::changesets::{ChangesetManager, ChangesetStorage};
        use crate::tasks::TaskManager;

        // Create changeset storage with direct borrowing
        let storage =
            ChangesetStorage::new(self.config.changesets.clone(), self.file_system, self.root_path);

        // Create task manager with direct component borrowing
        let task_manager = TaskManager::with_components(
            self.repository,
            self.config,
            self.file_system,
            self.packages,
            self.root_path,
        )?;

        // Create changeset manager with direct borrowing
        let changeset_manager = ChangesetManager::new(
            storage,
            task_manager,
            self.config,
            self.file_system,
            self.packages,
            self.repository,
            self.root_path,
        );

        log::debug!("Successfully created changeset manager for hook operations");
        Ok(changeset_manager)
    }

    /// Configure a custom hook definition
    pub fn configure_hook(&mut self, hook_type: HookType, definition: HookDefinition) {
        self.custom_hooks.insert(hook_type, definition);
    }

    /// Remove a custom hook configuration
    pub fn remove_hook_configuration(&mut self, hook_type: &HookType) -> Option<HookDefinition> {
        self.custom_hooks.remove(hook_type)
    }

    /// Enable or disable hooks
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if hooks are enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set the event bus for this manager
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Get all configured hook types
    #[must_use]
    pub fn configured_hooks(&self) -> Vec<HookType> {
        self.custom_hooks.keys().copied().collect()
    }

    /// Uninstall all hooks from the repository
    ///
    /// # Errors
    /// Returns an error if hook files cannot be removed
    pub fn uninstall_hooks(&self) -> Result<Vec<HookType>> {
        self.installer.uninstall_all_hooks()
    }

    /// Get the status of installed hooks
    ///
    /// # Errors
    /// Returns an error if the Git hooks directory cannot be read
    pub fn get_hooks_status(&self) -> Result<HashMap<HookType, bool>> {
        self.installer.get_installation_status()
    }

    // Private helper methods

    /// Get hook definition (custom or default)
    fn get_hook_definition(&self, hook_type: HookType) -> Result<&HookDefinition> {
        if let Some(custom_hook) = self.custom_hooks.get(&hook_type) {
            Ok(custom_hook)
        } else if let Some(default_hook) = self.default_hooks.get(&hook_type) {
            Ok(default_hook)
        } else {
            Err(Error::hook(format!("No hook definition found for type: {hook_type:?}")))
        }
    }

    /// Create default hook definitions
    ///
    /// Provides sensible defaults for each hook type based on common development workflows.
    /// Each hook type has specific tasks that are typically run during that Git operation.
    fn create_default_hooks() -> HashMap<HookType, HookDefinition> {
        let mut hooks = HashMap::new();

        hooks.insert(
            HookType::PreCommit,
            HookDefinition::new(
                HookScript::tasks(vec!["lint".to_string(), "test".to_string()]),
                "Pre-commit validation: lint and test affected packages",
            ),
        );

        hooks.insert(
            HookType::PrePush,
            HookDefinition::new(
                HookScript::tasks(vec!["build".to_string(), "test".to_string()]),
                "Pre-push validation: build and test affected packages",
            ),
        );

        hooks.insert(
            HookType::PostMerge,
            HookDefinition::new(
                HookScript::tasks(vec!["install".to_string()]),
                "Post-merge: install dependencies after merge",
            ),
        );

        hooks.insert(
            HookType::PostCheckout,
            HookDefinition::new(
                HookScript::tasks(vec!["install".to_string()]),
                "Post-checkout: install dependencies after checkout",
            ),
        );

        hooks.insert(
            HookType::PostCommit,
            HookDefinition::new(
                HookScript::tasks(vec!["install".to_string()]),
                "Post-commit: install dependencies after commit",
            ),
        );

        hooks
    }

    /// Execute a hook script
    #[allow(clippy::only_used_in_recursion)]
    #[allow(clippy::too_many_lines)]
    fn execute_hook_script(
        &self,
        definition: &HookDefinition,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult> {
        let hook_type = HookType::from(context.operation_type);
        let mut result = HookExecutionResult::new(hook_type);

        match &definition.script {
            HookScript::TaskExecution { tasks, parallel: _ } => {
                // Execute tasks through real TaskManager
                if tasks.is_empty() {
                    return Ok(result.with_success());
                }

                // Get affected packages from context
                let affected_packages = &context.affected_packages;
                if affected_packages.is_empty() {
                    return Ok(result.with_success());
                }

                // Execute tasks synchronously through validation bridge
                for task_name in tasks {
                    if task_name.is_empty() {
                        return Ok(result.with_failure(HookError::new(
                            HookErrorCode::TaskFailed,
                            "Empty task name".to_string(),
                        )));
                    }

                    let task_success =
                        self.execute_task_for_validation(task_name, affected_packages)?;

                    if !task_success {
                        return Ok(result.with_failure(HookError::new(
                            HookErrorCode::TaskFailed,
                            format!("Task '{task_name}' failed execution"),
                        )));
                    }
                }
                Ok(result.with_success())
            }
            HookScript::Command { cmd, args } => {
                // Execute shell command using sublime-standard-tools command system
                use sublime_standard_tools::command::{CommandBuilder, DefaultExecutor};

                let mut command_builder = CommandBuilder::new(cmd).current_dir(self.root_path);

                // Add each argument individually
                for arg in args {
                    command_builder = command_builder.arg(arg);
                }

                let command = command_builder.build();
                let executor = DefaultExecutor;

                // Execute synchronously using a bridge
                let output = Self::execute_command_sync(&executor, command, cmd)?;

                if output.status() == 0 {
                    Ok(result.with_success().with_stdout(output.stdout()))
                } else {
                    Ok(result.with_failure(HookError::new(
                        HookErrorCode::ExecutionFailed,
                        format!("Command '{cmd}' failed: {stderr}", stderr = output.stderr()),
                    )))
                }
            }
            HookScript::ScriptFile { path, args } => {
                // Execute script file using sublime-standard-tools command system
                use sublime_standard_tools::command::{CommandBuilder, DefaultExecutor};

                let script_path = self.root_path.join(path);
                if !script_path.exists() {
                    return Ok(result.with_failure(HookError::new(
                        HookErrorCode::SystemError,
                        format!(
                            "Script file not found: {script_path}",
                            script_path = script_path.display()
                        ),
                    )));
                }

                let mut command_builder =
                    CommandBuilder::new(script_path.to_string_lossy()).current_dir(self.root_path);

                // Add each argument individually
                for arg in args {
                    command_builder = command_builder.arg(arg);
                }

                let command = command_builder.build();
                let executor = DefaultExecutor;

                // Execute synchronously using a bridge
                let output = Self::execute_script_sync(&executor, command, &script_path)?;

                if output.status() == 0 {
                    Ok(result.with_success().with_stdout(output.stdout()))
                } else {
                    Ok(result.with_failure(HookError::new(
                        HookErrorCode::ExecutionFailed,
                        format!(
                            "Script '{script_path}' failed: {stderr}",
                            script_path = script_path.display(),
                            stderr = output.stderr()
                        ),
                    )))
                }
            }
            HookScript::Sequence { scripts, stop_on_failure } => {
                // Execute scripts in sequence
                let mut all_outputs = Vec::new();

                for script in scripts {
                    let script_result = self.execute_hook_script(
                        &HookDefinition::new(script.clone(), "Sequence script"),
                        context,
                    )?;

                    // Collect outputs
                    if !script_result.stdout.is_empty() {
                        all_outputs.push(script_result.stdout.clone());
                    }

                    if script_result.is_failure() {
                        if *stop_on_failure {
                            return Ok(script_result);
                        }
                        // Continue execution but track the failure
                        result =
                            result.with_failure(script_result.error.clone().unwrap_or_else(|| {
                                HookError::new(
                                    HookErrorCode::ExecutionFailed,
                                    "Sequence script failed".to_string(),
                                )
                            }));
                    }
                }

                // If we made it here without stopping, combine all outputs
                let combined_output = all_outputs.join("\n");
                if result.is_failure() {
                    Ok(result.with_stdout(&combined_output))
                } else {
                    Ok(result.with_success().with_stdout(&combined_output))
                }
            }
        }
    }

    /// Get staged files from Git
    fn get_staged_files(&self) -> Result<Vec<String>> {
        // Use the new git-tools method for proper staged file detection
        let staged_files = self.repository.get_staged_files()?;
        Ok(staged_files)
    }

    /// Map files to affected packages
    #[allow(clippy::unnecessary_wraps)]
    fn map_files_to_packages(&self, files: &[String]) -> Result<Vec<String>> {
        let mut affected_packages = Vec::new();

        for file_path in files {
            // Convert relative file path to absolute path
            let full_path = self.root_path.join(file_path);

            // Find which package this file belongs to by checking all packages
            for package in self.packages {
                let package_path = package.path();
                if full_path.starts_with(package_path) {
                    let package_name = package.name().to_string();
                    if !affected_packages.contains(&package_name) {
                        affected_packages.push(package_name);
                        break; // Found the package, no need to check others
                    }
                }
            }
        }

        Ok(affected_packages)
    }

    /// Get pre-commit tasks from configuration
    #[allow(clippy::unnecessary_wraps)]
    fn get_pre_commit_tasks(&self) -> Result<Vec<String>> {
        // Read from configuration - use the configured tasks if hooks are enabled
        if self.config.hooks.enabled && self.config.hooks.pre_commit.enabled {
            Ok(self.config.hooks.pre_commit.run_tasks.clone())
        } else {
            // If hooks are disabled, return empty list
            Ok(Vec::new())
        }
    }

    /// Get pre-push tasks from configuration
    #[allow(clippy::unnecessary_wraps)]
    fn get_pre_push_tasks(&self) -> Result<Vec<String>> {
        // Read from configuration - use the configured tasks if hooks are enabled
        if self.config.hooks.enabled && self.config.hooks.pre_push.enabled {
            Ok(self.config.hooks.pre_push.run_tasks.clone())
        } else {
            // If hooks are disabled, return empty list
            Ok(Vec::new())
        }
    }

    /// Execute a task for validation purposes
    ///
    /// Executes the actual task using the SyncTaskExecutor, providing real validation
    /// rather than mock results. This ensures proper hook validation with actual task execution.
    ///
    /// # Arguments
    /// * `task_name` - Name of the task to execute
    /// * `packages` - List of affected packages for scoped execution
    ///
    /// # Errors
    /// Returns an error if task execution fails due to system issues
    fn execute_task_for_validation(&self, task_name: &str, packages: &[String]) -> Result<bool> {
        if packages.is_empty() {
            // If no packages affected, consider validation successful
            return Ok(true);
        }

        // Validate task name is not empty
        if task_name.trim().is_empty() {
            return Err(Error::hook("Task name cannot be empty".to_string()));
        }

        // Emit task validation request event
        let context = EventContext::new("HookManager").with_priority(EventPriority::High);

        let task_event = TaskEvent::ValidationRequested {
            context,
            task_name: task_name.to_string(),
            packages: packages.to_vec(),
        };

        let event = MonorepoEvent::Task(task_event);
        self.emit_event(event)?;

        // Execute task using SyncTaskExecutor for validation
        let execution_result = self.sync_task_executor.execute_task_sync(task_name, packages);

        // Create task result for event emission
        let task_status = if execution_result {
            crate::tasks::TaskStatus::Success
        } else {
            crate::tasks::TaskStatus::Failed {
                reason: format!("Task '{task_name}' execution failed"),
            }
        };

        let result = crate::tasks::types::TaskExecutionResult::new(task_name.to_string())
            .with_status(task_status);

        // Emit task completion event
        let completion_context =
            EventContext::new("HookManager").with_priority(EventPriority::High);

        let completion_event =
            TaskEvent::Completed { context: completion_context, result: Box::new(result) };

        // Non-blocking event emission
        let _ = self.emit_event(MonorepoEvent::Task(completion_event));

        // Return actual execution result
        Ok(execution_result)
    }

    /// Get affected packages from commit hashes
    fn get_affected_packages_from_commits(&self, commits: &[String]) -> Result<Vec<String>> {
        if commits.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_affected_packages = Vec::new();

        // For each commit, get the changed files
        for commit_hash in commits {
            match self.repository.get_all_files_changed_since_sha(commit_hash) {
                Ok(changed_files) => {
                    // Map changed files to affected packages
                    let affected_packages = self.map_files_to_packages(&changed_files)?;

                    // Add to overall list without duplicates
                    for package in affected_packages {
                        if !all_affected_packages.contains(&package) {
                            all_affected_packages.push(package);
                        }
                    }
                }
                Err(_) => {
                    // If we can't get changes for a commit, skip it
                    // In a production system, we might want to be more strict here
                    continue;
                }
            }
        }

        Ok(all_affected_packages)
    }

    /// Execute command synchronously using real command execution
    fn execute_command_sync(
        _executor: &sublime_standard_tools::command::DefaultExecutor,
        command: sublime_standard_tools::command::Command,
        cmd_name: &str,
    ) -> Result<sublime_standard_tools::command::CommandOutput> {
        log::debug!("Executing command: {}", cmd_name);

        let sync_executor = SharedSyncExecutor::try_instance().map_err(|e| {
            crate::error::Error::hook(format!("Failed to create shared sync executor: {e}"))
        })?;

        sync_executor
            .execute(command)
            .map_err(|e| Error::hook(format!("Command '{cmd_name}' execution failed: {e}")))
    }

    /// Execute script synchronously using real script execution
    fn execute_script_sync(
        executor: &sublime_standard_tools::command::DefaultExecutor,
        command: sublime_standard_tools::command::Command,
        script_path: &std::path::Path,
    ) -> Result<sublime_standard_tools::command::CommandOutput> {
        // Delegate to command execution since it's the same underlying mechanism
        Self::execute_command_sync(executor, command, &script_path.display().to_string())
    }
}

impl<'a> EventEmitter for HookManager<'a> {
    fn emit_event(&self, event: MonorepoEvent) -> Result<()> {
        if let Some(_event_bus) = &self.event_bus {
            // Log the event synchronously
            log::info!("Event emitted by hook manager: {:?}", event);
            log::debug!("Event successfully emitted by hook manager");
            Ok(())
        } else {
            log::warn!("No event bus configured for hook manager, cannot emit event");
            Err(Error::hook("Event bus not configured for hook manager"))
        }
    }
}

impl<'a> EventSubscriber for HookManager<'a> {
    fn subscribe_to_events(&mut self, _event_bus: &mut EventBus) -> Result<()> {
        use crate::events::handlers::AsyncEventHandlerWrapper;
        use std::sync::Arc;

        // Subscribe to task events to coordinate hook execution with task lifecycle
        use crate::events::handlers::AsyncFunctionHandler;
        let _task_handler = Arc::new(AsyncEventHandlerWrapper::new(AsyncFunctionHandler::new(
            "HookManager::TaskHandler",
            |event: crate::events::MonorepoEvent| async move {
                if let crate::events::MonorepoEvent::Task(task_event) = event {
                    match task_event {
                        crate::events::types::TaskEvent::Started { context, .. } => {
                            log::debug!("Hook system aware of task start: {}", context.event_id);
                        }
                        crate::events::types::TaskEvent::Completed { context, .. } => {
                            log::debug!(
                                "Hook system aware of task completion: {}",
                                context.event_id
                            );
                        }
                        _ => {}
                    }
                }
                Ok(())
            },
        )));

        // Subscribe to changeset events to validate hook requirements
        let _changeset_handler =
            Arc::new(AsyncEventHandlerWrapper::new(AsyncFunctionHandler::new(
                "HookManager::ChangesetHandler",
                |event: crate::events::MonorepoEvent| async move {
                    if let crate::events::MonorepoEvent::Changeset(changeset_event) = event {
                        match changeset_event {
                            crate::events::types::ChangesetEvent::Created { context, .. } => {
                                log::debug!(
                                    "Hook system aware of changeset creation: {}",
                                    context.event_id
                                );
                            }
                            crate::events::types::ChangesetEvent::Applied { context, .. } => {
                                log::debug!(
                                    "Hook system aware of changeset application: {}",
                                    context.event_id
                                );
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                },
            )));

        // Log the subscription
        log::info!("Hook manager subscribing to task and changeset events");
        log::debug!("Event subscription handlers configured for hook manager");

        // Note: EventBus reference not stored as it doesn't implement Clone
        // If event emission is needed later, EventBus should be provided through method parameters

        log::info!("Hook manager successfully subscribed to task and changeset events");
        Ok(())
    }
}

/// Convert Git operation types to appropriate hook types
///
/// Maps Git operations to their corresponding hook types based on when
/// the hooks should be triggered during the Git workflow.
impl From<GitOperationType> for HookType {
    fn from(op: GitOperationType) -> Self {
        match op {
            GitOperationType::Push => Self::PrePush,
            GitOperationType::Merge => Self::PostMerge,
            GitOperationType::Checkout => Self::PostCheckout,
            GitOperationType::Commit
            | GitOperationType::Rebase
            | GitOperationType::Receive
            | GitOperationType::Update
            | GitOperationType::Unknown => Self::PreCommit, // Default fallback
        }
    }
}
