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
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use crate::events::types::{ChangesetEvent, HookEvent, TaskEvent};
use crate::events::{
    EventBus, EventContext, EventEmitter, EventPriority, EventSubscriber, MonorepoEvent,
};
use crate::tasks::TaskManager;
use crate::changesets::Changeset;
use std::collections::HashMap;
use std::sync::Arc;

impl HookManager {
    /// Create a new hook manager with injected dependencies
    pub fn new(
        installer: HookInstaller,
        validator: HookValidator,
        task_manager: TaskManager,
        config_provider: Box<dyn crate::core::ConfigProvider>,
        git_provider: Box<dyn crate::core::GitProvider>,
        file_system_provider: Box<dyn crate::core::FileSystemProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
    ) -> Result<Self> {
        let default_hooks = Self::create_default_hooks();
        
        // Create synchronous task executor with dedicated runtime
        let sync_task_executor = crate::hooks::sync_task_executor::SyncTaskExecutor::new(task_manager)?;

        Ok(Self {
            installer,
            validator,
            custom_hooks: HashMap::new(),
            default_hooks,
            enabled: true,
            event_bus: None,
            config_provider,
            git_provider,
            file_system_provider,
            package_provider,
            sync_task_executor,
        })
    }

    /// Create a new hook manager from project (convenience method)
    ///
    /// NOTE: This convenience method creates all components and provider instances from the project.
    /// For better performance and memory usage, prefer using the `new()` method with
    /// pre-created components and providers when creating multiple managers.
    pub fn from_project(project: Arc<MonorepoProject>) -> Result<Self> {
        use crate::core::interfaces::DependencyFactory;

        // Create providers efficiently to avoid multiple Arc clones
        let git_provider1 = DependencyFactory::git_provider(Arc::clone(&project));
        let git_provider2 = DependencyFactory::git_provider(Arc::clone(&project));
        let git_provider3 = DependencyFactory::git_provider(Arc::clone(&project));
        let file_system_provider1 = DependencyFactory::file_system_provider(Arc::clone(&project));
        let file_system_provider2 = DependencyFactory::file_system_provider(Arc::clone(&project));
        let package_provider1 = DependencyFactory::package_provider(Arc::clone(&project));
        let package_provider2 = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider1 = DependencyFactory::config_provider(Arc::clone(&project));
        let config_provider2 = DependencyFactory::config_provider(Arc::clone(&project));

        // Create installer directly with providers
        let installer = HookInstaller::new(git_provider1, file_system_provider1)?;

        // Create validator directly with providers
        let validator = HookValidator::new(git_provider2, package_provider1, config_provider1);

        // Create task manager with proper dependency injection
        let task_manager = TaskManager::from_project(Arc::clone(&project))?;

        Self::new(
            installer,
            validator,
            task_manager,
            config_provider2,
            git_provider3,
            file_system_provider2,
            package_provider2,
        )
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
            match self.get_hook_definition(&hook_type) {
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

        let hook_definition = self.get_hook_definition(&hook_type)?;

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
                let mut hook_result = HookExecutionResult::new(hook_type.clone());

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
                let mut hook_result = HookExecutionResult::new(hook_type.clone());

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
        } else if self.config_provider.config().changesets.required {
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
        let changeset_manager = self.create_changeset_manager()
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
        use crate::changesets::ChangesetManager;
        use crate::core::MonorepoProject;

        // Get project root path from package provider
        let root_path = self.package_provider.root_path();

        // Create a new MonorepoProject instance from the root path
        // This is safe since the project structure is the same
        let project = Arc::new(MonorepoProject::new(root_path)
            .map_err(|e| Error::hook(format!("Failed to create project instance for changesets: {e}")))?);

        // Use the convenient from_project method which handles all dependency injection
        let changeset_manager = ChangesetManager::from_project(project)
            .map_err(|e| Error::hook(format!("Failed to create changeset manager from project: {e}")))?;

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
        self.custom_hooks.keys().cloned().collect()
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
    fn get_hook_definition(&self, hook_type: &HookType) -> Result<&HookDefinition> {
        if let Some(custom_hook) = self.custom_hooks.get(hook_type) {
            Ok(custom_hook)
        } else if let Some(default_hook) = self.default_hooks.get(hook_type) {
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

                let mut command_builder =
                    CommandBuilder::new(cmd).current_dir(self.package_provider.root_path());

                // Add each argument individually
                for arg in args {
                    command_builder = command_builder.arg(arg);
                }

                let command = command_builder.build();
                let executor = DefaultExecutor;

                // Execute synchronously using a bridge
                let output = self.execute_command_sync(executor, command, cmd)?;

                if output.status() == 0 {
                    Ok(result.with_success().with_stdout(output.stdout()))
                } else {
                    Ok(result.with_failure(HookError::new(
                        HookErrorCode::ExecutionFailed,
                        format!("Command '{cmd}' failed: {}", output.stderr()),
                    )))
                }
            }
            HookScript::ScriptFile { path, args } => {
                // Execute script file using sublime-standard-tools command system
                use sublime_standard_tools::command::{CommandBuilder, DefaultExecutor};

                let script_path = self.package_provider.root_path().join(path);
                if !script_path.exists() {
                    return Ok(result.with_failure(HookError::new(
                        HookErrorCode::SystemError,
                        format!("Script file not found: {}", script_path.display()),
                    )));
                }

                let mut command_builder = CommandBuilder::new(script_path.to_string_lossy())
                    .current_dir(self.package_provider.root_path());

                // Add each argument individually
                for arg in args {
                    command_builder = command_builder.arg(arg);
                }

                let command = command_builder.build();
                let executor = DefaultExecutor;

                // Execute synchronously using a bridge
                let output = self.execute_script_sync(executor, command, &script_path)?;

                if output.status() == 0 {
                    Ok(result.with_success().with_stdout(output.stdout()))
                } else {
                    Ok(result.with_failure(HookError::new(
                        HookErrorCode::ExecutionFailed,
                        format!("Script '{}' failed: {}", script_path.display(), output.stderr()),
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
        let staged_files = self.git_provider.repository().get_staged_files()?;
        Ok(staged_files)
    }

    /// Map files to affected packages
    #[allow(clippy::unnecessary_wraps)]
    fn map_files_to_packages(&self, files: &[String]) -> Result<Vec<String>> {
        let mut affected_packages = Vec::new();

        for file_path in files {
            // Convert relative file path to absolute path
            let full_path = self.package_provider.root_path().join(file_path);

            // Find which package this file belongs to by checking all packages
            for package in self.package_provider.packages() {
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
        let config = self.config_provider.config();
        if config.hooks.enabled && config.hooks.pre_commit.enabled {
            Ok(config.hooks.pre_commit.run_tasks.clone())
        } else {
            // If hooks are disabled, return empty list
            Ok(Vec::new())
        }
    }

    /// Get pre-push tasks from configuration
    #[allow(clippy::unnecessary_wraps)]
    fn get_pre_push_tasks(&self) -> Result<Vec<String>> {
        // Read from configuration - use the configured tasks if hooks are enabled
        let config = self.config_provider.config();
        if config.hooks.enabled && config.hooks.pre_push.enabled {
            Ok(config.hooks.pre_push.run_tasks.clone())
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
                reason: format!("Task '{}' execution failed", task_name) 
            }
        };

        let result = crate::tasks::types::TaskExecutionResult::new(task_name.to_string())
            .with_status(task_status);

        // Emit task completion event
        let completion_context =
            EventContext::new("HookManager").with_priority(EventPriority::High);

        let completion_event =
            TaskEvent::Completed { context: completion_context, result };

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
            match self.git_provider.repository().get_all_files_changed_since_sha(commit_hash) {
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
        &self,
        executor: sublime_standard_tools::command::DefaultExecutor,
        command: sublime_standard_tools::command::Command,
        cmd_name: &str,
    ) -> Result<sublime_standard_tools::command::CommandOutput> {
        log::debug!("Executing command: {}", cmd_name);
        
        // Create a dedicated runtime for command execution to avoid nested runtime issues
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::hook(format!("Failed to create runtime for command execution: {e}")))?;
        
        // Execute the actual command asynchronously in the dedicated runtime
        runtime.block_on(async {
            use sublime_standard_tools::command::Executor;
            executor.execute(command).await
        })
        .map_err(|e| Error::hook(format!("Command '{cmd_name}' execution failed: {e}")))
    }

    /// Execute script synchronously using real script execution
    fn execute_script_sync(
        &self,
        executor: sublime_standard_tools::command::DefaultExecutor,
        command: sublime_standard_tools::command::Command,
        script_path: &std::path::Path,
    ) -> Result<sublime_standard_tools::command::CommandOutput> {
        // Delegate to command execution since it's the same underlying mechanism
        self.execute_command_sync(executor, command, &script_path.display().to_string())
    }
}

impl EventEmitter for HookManager {
    fn emit_event(&self, event: MonorepoEvent) -> Result<()> {
        if let Some(_event_bus) = &self.event_bus {
            // For synchronous hook execution, we skip async event emission to avoid runtime issues
            // In a full implementation, this would use proper async handling or a background thread pool
            log::debug!("Event emission skipped in synchronous context: {:?}", event);
            Ok(())
        } else {
            // No event bus configured, skip event emission
            Ok(())
        }
    }
}

impl EventSubscriber for HookManager {
    fn subscribe_to_events(&mut self, _event_bus: &mut EventBus) -> Result<()> {
        // For synchronous hook execution, we skip event subscription to avoid runtime issues
        // In a full implementation, this would use proper async handling
        log::debug!("Event subscription skipped in synchronous context");
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
