//! Task executor implementation
//!
//! The `TaskExecutor` handles the actual execution of tasks, including command
//! execution, package script running, and result collection.

// TODO: Remove after Phase 4 - currently simplified implementation, full async integration pending
#![allow(clippy::unused_async)]

use super::{
    manager::ExecutionContext,
    types::{TaskCommand, TaskCommandCore},
    PackageScript, TaskDefinition, TaskError, TaskErrorCode, TaskExecutionLog, TaskExecutionResult,
    TaskOutput, TaskScope,
};
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use sublime_standard_tools::command::{Command, CommandPriority, CommandQueue};

/// Executor for running tasks with various scopes and configurations
pub struct TaskExecutor {
    /// Reference to the monorepo project
    project: Arc<MonorepoProject>,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self> {
        // DRY: No CommandQueue created here - will be created when needed
        Ok(Self { project })
    }

    /// Execute a task with specified scope
    pub async fn execute_task(
        &self,
        task: &TaskDefinition,
        scope: &TaskScope,
    ) -> Result<TaskExecutionResult> {
        let context = ExecutionContext::default();
        self.execute_task_with_context(task, scope, &context).await
    }

    /// Execute a task with specific context
    pub async fn execute_task_with_context(
        &self,
        task: &TaskDefinition,
        scope: &TaskScope,
        context: &ExecutionContext,
    ) -> Result<TaskExecutionResult> {
        let mut result = TaskExecutionResult::new(&task.name);
        result.mark_started();

        // Determine packages to execute on
        let target_packages = self.resolve_target_packages(scope, context)?;
        result.affected_packages.clone_from(&target_packages);

        // Add initial log
        result.add_log(TaskExecutionLog::info(format!(
            "Starting task '{}' for {} packages",
            task.name,
            target_packages.len()
        )));

        // Execute commands and package scripts
        let mut all_successful = true;

        // Execute regular commands
        for command in &task.commands {
            match self.execute_command_for_packages(command, &target_packages, context).await {
                Ok(mut outputs) => {
                    result.outputs.append(&mut outputs);
                }
                Err(e) => {
                    all_successful = false;
                    result.add_error(
                        TaskError::new(
                            TaskErrorCode::ExecutionFailed,
                            format!("Command execution failed: {e}"),
                        )
                        .with_command(&command.command.program),
                    );

                    if !task.continue_on_error {
                        break;
                    }
                }
            }
        }

        // Execute package scripts
        for script in &task.package_scripts {
            match self.execute_package_script(script, &target_packages, context).await {
                Ok(mut outputs) => {
                    result.outputs.append(&mut outputs);
                }
                Err(e) => {
                    all_successful = false;
                    result.add_error(
                        TaskError::new(
                            TaskErrorCode::ExecutionFailed,
                            format!("Package script execution failed: {e}"),
                        )
                        .with_command(&script.script_name),
                    );

                    if !task.continue_on_error {
                        break;
                    }
                }
            }
        }

        // Update statistics
        result.stats.commands_executed = task.commands.len() + task.package_scripts.len();
        result.stats.commands_succeeded = result.outputs.iter().filter(|o| o.is_success()).count();
        result.stats.commands_failed =
            result.stats.commands_executed - result.stats.commands_succeeded;
        result.stats.packages_processed = target_packages.len();
        result.stats.stdout_bytes = result.outputs.iter().map(|o| o.stdout.len()).sum();
        result.stats.stderr_bytes = result.outputs.iter().map(|o| o.stderr.len()).sum();

        // Mark completion
        result.mark_completed(all_successful && result.errors.is_empty());

        result.add_log(TaskExecutionLog::info(format!(
            "Task '{}' completed with status: {:?}",
            task.name, result.status
        )));

        Ok(result)
    }

    // Private helper methods

    /// Resolve target packages based on scope and context
    fn resolve_target_packages(
        &self,
        scope: &TaskScope,
        context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        match scope {
            TaskScope::Global => Ok(vec!["__global__".to_string()]),

            TaskScope::Package(package_name) => {
                // Validate package exists
                if self.project.get_package(package_name).is_some() {
                    Ok(vec![package_name.clone()])
                } else {
                    Err(Error::task(format!("Package not found: {package_name}")))
                }
            }

            TaskScope::AffectedPackages => Ok(context.affected_packages.clone()),

            TaskScope::AllPackages => {
                Ok(self.project.packages.iter().map(|pkg| pkg.name().to_string()).collect())
            }

            TaskScope::PackagesMatching { pattern } => {
                let matching_packages = self
                    .project
                    .packages
                    .iter()
                    .filter(|pkg| self.matches_pattern(pkg.name(), pattern))
                    .map(|pkg| pkg.name().to_string())
                    .collect();

                Ok(matching_packages)
            }

            TaskScope::Custom { filter } => {
                // Execute the custom filter function
                let mut matching_packages = Vec::new();
                
                for package in &self.project.packages {
                    // Parse the filter as a simple expression
                    // For now, support basic property access like "package.name.includes('@myorg/')"
                    if self.evaluate_custom_filter(filter, package.name(), context) {
                        matching_packages.push(package.name().to_string());
                    }
                }
                
                Ok(matching_packages)
            }
        }
    }

    /// Execute a command for target packages
    async fn execute_command_for_packages(
        &self,
        command: &TaskCommand,
        target_packages: &[String],
        context: &ExecutionContext,
    ) -> Result<Vec<TaskOutput>> {
        let mut outputs = Vec::new();

        // For global scope, execute once
        if target_packages.contains(&"__global__".to_string()) {
            let output = self.execute_command(command, None, context).await?;
            outputs.push(output);
        } else {
            // Execute for each package
            for package_name in target_packages {
                let output = self.execute_command(command, Some(package_name), context).await?;
                outputs.push(output);
            }
        }

        Ok(outputs)
    }

    /// Execute a command instance
    async fn execute_command(
        &self,
        command: &TaskCommand,
        package_name: Option<&str>,
        context: &ExecutionContext,
    ) -> Result<TaskOutput> {
        // DRY: Convert TaskCommand to standard Command and use CommandQueue
        let working_dir = self.resolve_working_directory(command, package_name, context)?;

        // Create standard command from task command with resolved working directory
        let mut task_command_core = command.command.clone();
        task_command_core.current_dir = Some(working_dir.clone());
        let std_command: Command = task_command_core.into();

        let start_time = SystemTime::now();

        // DRY: Create CommandQueue for this execution (lazy initialization)
        let command_queue = CommandQueue::new()
            .start()
            .map_err(|e| Error::task(format!("Failed to start command queue: {e}")))?;

        let command_id = command_queue
            .enqueue(std_command, CommandPriority::Normal)
            .await
            .map_err(|e| Error::task(format!("Failed to enqueue command: {e}")))?;

        // Wait for command completion
        let result = command_queue
            .wait_for_command(&command_id, Duration::from_secs(300))
            .await
            .map_err(|e| Error::task(format!("Command execution failed: {e}")))?;

        let duration = start_time.elapsed().unwrap_or_default();

        // Convert result to TaskOutput
        let task_output = if result.is_successful() {
            let output =
                result.output.ok_or_else(|| Error::task("Command result missing output"))?;
            TaskOutput {
                command: format!("{} {}", command.command.program, command.command.args.join(" ")),
                working_dir,
                exit_code: Some(output.status()),
                stdout: output.stdout().to_string(),
                stderr: output.stderr().to_string(),
                duration,
                environment: command.command.env.clone(),
            }
        } else {
            // Command failed
            let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
            TaskOutput {
                command: format!("{} {}", command.command.program, command.command.args.join(" ")),
                working_dir,
                exit_code: Some(-1),
                stdout: String::new(),
                stderr: error_msg,
                duration,
                environment: command.command.env.clone(),
            }
        };

        Ok(task_output)
    }

    /// Execute a package script
    async fn execute_package_script(
        &self,
        script: &PackageScript,
        target_packages: &[String],
        context: &ExecutionContext,
    ) -> Result<Vec<TaskOutput>> {
        let mut outputs = Vec::new();

        // Determine which packages to run the script on
        let script_packages = if let Some(package_name) = &script.package_name {
            vec![package_name.clone()]
        } else {
            target_packages.to_vec()
        };

        for package_name in script_packages {
            let output = self.execute_single_package_script(script, &package_name, context).await?;
            outputs.push(output);
        }

        Ok(outputs)
    }

    /// Execute a single package script
    async fn execute_single_package_script(
        &self,
        script: &PackageScript,
        package_name: &str,
        context: &ExecutionContext,
    ) -> Result<TaskOutput> {
        // Get package info
        let package_info = self
            .project
            .get_package(package_name)
            .ok_or_else(|| Error::task(format!("Package not found: {package_name}")))?;

        // DRY: Use PackageScript -> Command conversion
        let mut script_with_working_dir = script.clone();
        if script_with_working_dir.working_directory.is_none() {
            script_with_working_dir.working_directory = Some(package_info.path().clone());
        }

        // DRY: Use PackageScript -> Command conversion, then recreate TaskCommandWrapper
        // Build the TaskCommandWrapper directly (simpler than round-trip conversion)
        let manager = script.package_manager.as_deref().unwrap_or("npm");
        let mut args = vec!["run".to_string(), script.script_name.clone()];
        if !script.extra_args.is_empty() {
            args.push("--".to_string());
            args.extend(script.extra_args.clone());
        }

        let task_command = TaskCommand {
            command: TaskCommandCore {
                program: manager.to_string(),
                args,
                current_dir: script_with_working_dir.working_directory,
                env: HashMap::new(),
                timeout: None,
            },
            shell: false,
            expected_exit_codes: vec![0],
        };

        self.execute_command(&task_command, Some(package_name), context).await
    }

    /// Resolve working directory for command execution
    #[allow(clippy::unnecessary_wraps)]
    fn resolve_working_directory(
        &self,
        command: &TaskCommand,
        package_name: Option<&str>,
        context: &ExecutionContext,
    ) -> Result<PathBuf> {
        // Priority: command current_dir > package path > context working_dir > project root
        if let Some(working_dir) = &command.command.current_dir {
            if working_dir.is_absolute() {
                Ok(working_dir.clone())
            } else {
                // Relative to project root
                Ok(self.project.root_path().join(working_dir))
            }
        } else if let Some(package_name) = package_name {
            if let Some(package_info) = self.project.get_package(package_name) {
                Ok(package_info.path().clone())
            } else {
                Ok(self.project.root_path().to_path_buf())
            }
        } else if let Some(working_dir) = &context.working_directory {
            Ok(working_dir.clone())
        } else {
            Ok(self.project.root_path().to_path_buf())
        }
    }

    /// Check if a string matches a pattern using glob-style matching
    ///
    /// Uses the glob crate for proper pattern matching support including:
    /// - `*` matches any sequence of characters
    /// - `?` matches any single character  
    /// - `[seq]` matches any character in seq
    /// - `[!seq]` matches any character not in seq
    ///
    /// # Arguments
    ///
    /// * `text` - The text to match against
    /// * `pattern` - The glob pattern to use
    ///
    /// # Returns
    ///
    /// True if the text matches the pattern, false otherwise
    #[allow(clippy::unused_self)]
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        use glob::Pattern;
        
        // Create the glob pattern
        match Pattern::new(pattern) {
            Ok(glob_pattern) => glob_pattern.matches(text),
            Err(_) => {
                // If pattern is invalid, fall back to exact match
                text == pattern
            }
        }
    }

    /// Evaluate a custom filter expression for a package
    ///
    /// Supports simple expressions like:
    /// - `package.name.includes('@scope/')`
    /// - `package.name.startsWith('@myorg/')`
    /// - `context.affected.includes(package.name)`
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter expression to evaluate
    /// * `package_name` - Name of the package being evaluated
    /// * `context` - Execution context with affected packages
    ///
    /// # Returns
    ///
    /// True if the package matches the filter, false otherwise
    #[allow(clippy::unused_self)]
    fn evaluate_custom_filter(&self, filter: &str, package_name: &str, context: &ExecutionContext) -> bool {
        // Parse common filter patterns
        let filter = filter.trim();
        
        // Handle package.name.includes('...') pattern
        if let Some(search_str) = extract_string_argument(filter, "package.name.includes(") {
            return package_name.contains(&search_str);
        }
        
        // Handle package.name.startsWith('...') pattern
        if let Some(prefix) = extract_string_argument(filter, "package.name.startsWith(") {
            return package_name.starts_with(&prefix);
        }
        
        // Handle package.name.endsWith('...') pattern
        if let Some(suffix) = extract_string_argument(filter, "package.name.endsWith(") {
            return package_name.ends_with(&suffix);
        }
        
        // Handle context.affected.includes(package.name) pattern
        if filter == "context.affected.includes(package.name)" {
            return context.affected_packages.contains(&package_name.to_string());
        }
        
        // Handle package.name === '...' pattern
        if let Some(exact_match) = extract_string_argument(filter, "package.name === ") {
            return package_name == exact_match;
        }
        
        // Default to false for unsupported expressions
        false
    }
}

/// Helper function to extract string argument from filter expressions
///
/// Extracts the string argument from patterns like `method('arg')` or `method("arg")`
///
/// # Arguments
///
/// * `filter` - The full filter expression
/// * `prefix` - The method prefix to match (e.g., "package.name.includes(")
///
/// # Returns
///
/// The extracted string argument if found, None otherwise
fn extract_string_argument(filter: &str, prefix: &str) -> Option<String> {
    if !filter.starts_with(prefix) {
        return None;
    }
    
    let remaining = &filter[prefix.len()..];
    
    // Find the closing parenthesis
    if let Some(close_idx) = remaining.rfind(')') {
        let arg_part = &remaining[..close_idx].trim();
        
        // Remove quotes if present
        if (arg_part.starts_with('\'') && arg_part.ends_with('\'')) ||
           (arg_part.starts_with('"') && arg_part.ends_with('"')) {
            let unquoted = &arg_part[1..arg_part.len()-1];
            return Some(unquoted.to_string());
        }
    }
    
    None
}
