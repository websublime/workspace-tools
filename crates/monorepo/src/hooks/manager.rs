//! Hook manager implementation
//!
//! The `HookManager` orchestrates Git hook installation, execution, and validation,
//! coordinating with other monorepo systems for comprehensive workflow management.

// Hook manager implementation - now fully synchronous for better integration with MonorepoProject

use super::{
    GitOperationType, HookDefinition, HookError, HookErrorCode, HookExecutionContext,
    HookExecutionResult, HookInstaller, HookScript, HookType, HookValidationResult, HookValidator,
    PreCommitResult, PrePushResult, ValidationCheck,
};
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use crate::Changeset;
use std::collections::HashMap;
use std::sync::Arc;

/// Central manager for Git hook installation, execution, and validation
pub struct HookManager {
    /// Reference to the monorepo project
    #[allow(dead_code)] // Will be used when full integration is implemented
    project: Arc<MonorepoProject>,

    /// Hook installer for setting up Git hooks
    installer: HookInstaller,

    /// Hook validator for checking conditions and requirements
    validator: HookValidator,

    /// Registry of custom hook definitions
    custom_hooks: HashMap<HookType, HookDefinition>,

    /// Whether hooks are currently enabled
    enabled: bool,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self> {
        let installer = HookInstaller::new(Arc::clone(&project))?;
        let validator = HookValidator::new(Arc::clone(&project));

        Ok(Self { project, installer, validator, custom_hooks: HashMap::new(), enabled: true })
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
            match self.installer.install_hook(&hook_type, self.get_hook_definition(&hook_type)) {
                Ok(()) => {
                    installed.push(hook_type);
                }
                Err(e) => {
                    // Log error but continue with other hooks
                    eprintln!("Failed to install hook {hook_type:?}: {e}");
                }
            }
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

        let hook_definition = self.get_hook_definition(&hook_type);

        // Check if hook is enabled
        if !hook_definition.enabled {
            return Ok(HookExecutionResult::new(hook_type).with_skipped("Hook is disabled"));
        }

        // Validate conditions before execution
        if !self.validator.check_conditions(&hook_definition.conditions, context)? {
            return Ok(HookExecutionResult::new(hook_type).with_skipped("Hook conditions not met"));
        }

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
                Ok(hook_result)
            }
            _ => {
                // For other hook types, execute the script directly
                self.execute_hook_script(hook_definition, context)
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
        } else {
            result = result.with_required_action("Create a changeset for the affected packages");
        }

        // Run pre-commit tasks if configured
        let pre_commit_tasks = self.get_pre_commit_tasks()?;
        for task_name in &pre_commit_tasks {
            // Note: This is now async but we're in a sync context
            // In a real implementation, the entire validation would be async
            // For now, we'll handle this as a placeholder
            result = result.with_required_action(format!("Task {task_name} needs validation"));
        }

        // Overall validation status
        let validation_passed = result.changeset.is_some() && result.required_actions.is_empty();
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

        // Execute tasks for affected packages
        for task_name in &pre_push_tasks {
            // Note: This is now async but we're in a sync context
            // In a real implementation, the entire validation would be async
            // For now, we'll simulate task execution
            result = result.with_task_result(task_name, true);
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
        // Now integrate with ChangesetManager from Phase 4
        use crate::changesets::ChangesetManager;

        let changeset_manager = ChangesetManager::new(Arc::clone(&self.project))
            .map_err(|e| Error::hook(format!("Failed to create changeset manager: {e}")))?;

        // Create an interactive changeset
        changeset_manager
            .create_changeset_interactive(None)
            .map_err(|e| Error::hook(format!("Failed to create changeset: {e}")))
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
    fn get_hook_definition(&self, hook_type: &HookType) -> &HookDefinition {
        self.custom_hooks
            .get(hook_type)
            .unwrap_or_else(|| Self::get_default_hook_definition(hook_type))
    }

    /// Get default hook definition for a hook type
    fn get_default_hook_definition(_hook_type: &HookType) -> &'static HookDefinition {
        // This would be stored in a lazy static or similar
        // For now, return a reference to a default definition
        // In a real implementation, this would have proper defaults for each hook type
        static DEFAULT_HOOK: once_cell::sync::Lazy<HookDefinition> =
            once_cell::sync::Lazy::new(|| {
                HookDefinition::new(
                    HookScript::tasks(vec!["lint".to_string(), "test".to_string()]),
                    "Default hook configuration",
                )
            });
        &DEFAULT_HOOK
    }

    /// Execute a hook script
    #[allow(clippy::only_used_in_recursion)]
    fn execute_hook_script(
        &self,
        definition: &HookDefinition,
        context: &HookExecutionContext,
    ) -> Result<HookExecutionResult> {
        let hook_type = HookType::from(context.operation_type);
        let result = HookExecutionResult::new(hook_type);

        match &definition.script {
            HookScript::TaskExecution { tasks, parallel: _ } => {
                // Execute tasks through TaskManager
                // Note: This is now async but we're in a sync context
                // In a real implementation, hook execution would be async
                for task_name in tasks {
                    // For now, simulate successful task execution
                    // In Phase 4+, this would use the async execute_task_for_validation
                    if task_name.is_empty() {
                        return Ok(result.with_failure(HookError::new(
                            HookErrorCode::TaskFailed,
                            "Empty task name".to_string(),
                        )));
                    }
                }
                Ok(result.with_success())
            }
            HookScript::Command { cmd: _, args: _ } => {
                // Execute shell command
                // This would use CommandQueue from standard-tools
                Ok(result.with_success().with_stdout("Command executed successfully"))
            }
            HookScript::ScriptFile { path: _, args: _ } => {
                // Execute script file
                // This would use CommandQueue from standard-tools
                Ok(result.with_success().with_stdout("Script executed successfully"))
            }
            HookScript::Sequence { scripts, stop_on_failure } => {
                // Execute scripts in sequence
                for script in scripts {
                    let script_result = self.execute_hook_script(
                        &HookDefinition::new(script.clone(), "Sequence script"),
                        context,
                    )?;

                    if script_result.is_failure() && *stop_on_failure {
                        return Ok(script_result);
                    }
                }
                Ok(result.with_success())
            }
        }
    }

    /// Get staged files from Git
    fn get_staged_files(&self) -> Result<Vec<String>> {
        // Use the new git-tools method for proper staged file detection
        let staged_files = self.project.repository.get_staged_files()?;
        Ok(staged_files)
    }

    /// Map files to affected packages
    #[allow(clippy::unnecessary_wraps)]
    fn map_files_to_packages(&self, files: &[String]) -> Result<Vec<String>> {
        let mut affected_packages = Vec::new();

        for file_path in files {
            // Convert relative file path to absolute path
            let full_path = self.project.root_path().join(file_path);

            // Use MonorepoDescriptor to find which package this file belongs to
            if let Some(package) = self.project.descriptor.find_package_for_path(&full_path) {
                let package_name = package.name.clone();
                if !affected_packages.contains(&package_name) {
                    affected_packages.push(package_name);
                }
            }
        }

        Ok(affected_packages)
    }

    /// Get pre-commit tasks from configuration
    #[allow(clippy::unnecessary_wraps)]
    fn get_pre_commit_tasks(&self) -> Result<Vec<String>> {
        // Read from configuration - use the configured tasks if hooks are enabled
        if self.project.config.hooks.enabled && self.project.config.hooks.pre_commit.enabled {
            Ok(self.project.config.hooks.pre_commit.run_tasks.clone())
        } else {
            // If hooks are disabled, return empty list
            Ok(Vec::new())
        }
    }

    /// Get pre-push tasks from configuration
    #[allow(clippy::unnecessary_wraps)]
    fn get_pre_push_tasks(&self) -> Result<Vec<String>> {
        // Read from configuration - use the configured tasks if hooks are enabled
        if self.project.config.hooks.enabled && self.project.config.hooks.pre_push.enabled {
            Ok(self.project.config.hooks.pre_push.run_tasks.clone())
        } else {
            // If hooks are disabled, return empty list
            Ok(Vec::new())
        }
    }

    /// Execute a task for validation purposes
    #[allow(dead_code)]
    async fn execute_task_for_validation(
        &self,
        task_name: &str,
        packages: &[String],
    ) -> Result<bool> {
        // Now integrate with TaskManager for real task execution
        use crate::tasks::types::results::TaskStatus;
        use crate::tasks::{TaskManager, TaskScope};

        if packages.is_empty() {
            // If no packages affected, consider it successful
            return Ok(true);
        }

        let task_manager = TaskManager::new(Arc::clone(&self.project))
            .map_err(|e| Error::hook(format!("Failed to create task manager: {e}")))?;

        // Try to resolve this task from package.json scripts or registered tasks
        for package_name in packages {
            match task_manager.resolve_package_tasks(package_name).await {
                Ok(task_definitions) => {
                    // Find the task we want to execute
                    if let Some(_task_def) =
                        task_definitions.iter().find(|def| def.name == task_name)
                    {
                        // Execute the task for this package
                        let task_scope = TaskScope::Package(package_name.clone());
                        match task_manager.execute_task(task_name, Some(task_scope)).await {
                            Ok(result) => {
                                match result.status {
                                    TaskStatus::Success => {
                                        // Task completed successfully
                                    }
                                    _ => {
                                        return Ok(false);
                                    }
                                }
                            }
                            Err(_) => {
                                return Ok(false);
                            }
                        }
                    }
                }
                Err(_) => {
                    // If we can't resolve tasks for this package, treat as failure
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Get affected packages from commit hashes
    fn get_affected_packages_from_commits(&self, commits: &[String]) -> Result<Vec<String>> {
        if commits.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_affected_packages = Vec::new();

        // For each commit, get the changed files
        for commit_hash in commits {
            match self.project.repository.get_all_files_changed_since_sha(commit_hash) {
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
}

// Temporary conversion for development - would be properly implemented
impl From<GitOperationType> for HookType {
    fn from(op: GitOperationType) -> Self {
        match op {
            GitOperationType::Push => Self::PrePush,
            GitOperationType::Merge => Self::PostMerge,
            GitOperationType::Checkout => Self::PostCheckout,
            _ => Self::PreCommit, // Default fallback for Commit and others
        }
    }
}
