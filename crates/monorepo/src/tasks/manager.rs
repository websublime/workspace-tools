//! Task manager implementation
//!
//! The `TaskManager` orchestrates task execution, manages dependencies,
//! and coordinates with other monorepo systems for comprehensive workflow management.

// TODO: Remove after Phase 4 - currently some methods don't need async but will in complete implementation
#![allow(clippy::unused_async)]

use super::{
    ConditionChecker, TaskDefinition, TaskExecutionResult, TaskExecutor, TaskRegistry, TaskScope,
};
use crate::analysis::ChangeAnalysis;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Central manager for task execution and coordination
pub struct TaskManager {
    /// Reference to the monorepo project
    project: Arc<MonorepoProject>,

    /// Task registry for storing and managing task definitions
    registry: TaskRegistry,

    /// Task executor for running tasks
    executor: TaskExecutor,

    /// Condition checker for evaluating task conditions
    condition_checker: ConditionChecker,

    /// Current execution context
    execution_context: ExecutionContext,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new(project: Arc<MonorepoProject>) -> Result<Self> {
        let registry = TaskRegistry::new();
        let executor = TaskExecutor::new(Arc::clone(&project))?;
        let condition_checker = ConditionChecker::new(Arc::clone(&project));

        Ok(Self {
            project,
            registry,
            executor,
            condition_checker,
            execution_context: ExecutionContext::default(),
        })
    }

    /// Execute a single task by name
    pub async fn execute_task(
        &self,
        task_name: &str,
        scope: Option<TaskScope>,
    ) -> Result<TaskExecutionResult> {
        let task_definition = self
            .registry
            .get_task(task_name)
            .ok_or_else(|| Error::task(format!("Task not found: {task_name}")))?;

        let effective_scope = scope.unwrap_or_else(|| task_definition.scope.clone());

        // Check conditions before execution
        if !self.condition_checker.check_conditions(&task_definition.conditions).await? {
            return Ok(
                TaskExecutionResult::new(task_name).with_status_skipped("Conditions not met")
            );
        }

        // Execute task with the specified scope
        self.executor.execute_task(task_definition, &effective_scope).await
    }

    /// Execute tasks for affected packages based on change analysis
    pub async fn execute_tasks_for_affected_packages(
        &self,
        affected_packages: &[String],
    ) -> Result<Vec<TaskExecutionResult>> {
        let mut results = Vec::new();

        // Get tasks that should run for affected packages
        let applicable_tasks = self.registry.get_tasks_for_scope(&TaskScope::AffectedPackages);

        for task in applicable_tasks {
            // Update execution context with affected packages
            let mut context = self.execution_context.clone();
            context.affected_packages = affected_packages.to_vec();

            // Execute task
            let result = self.execute_task_with_context(task, &context).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute tasks based on change analysis
    pub async fn execute_tasks_for_changes(
        &self,
        changes: &ChangeAnalysis,
    ) -> Result<Vec<TaskExecutionResult>> {
        let mut results = Vec::new();

        // Extract affected packages from changes
        let affected_packages: Vec<String> =
            changes.package_changes.iter().map(|pc| pc.package_name.clone()).collect();

        // Update execution context
        let mut context = self.execution_context.clone();
        context.affected_packages = affected_packages;
        context.changed_files.clone_from(&changes.changed_files);

        // Get tasks that match the change conditions
        let applicable_tasks = self.get_tasks_for_changes(changes).await?;

        for task in applicable_tasks {
            let result = self.execute_task_with_context(&task, &context).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute multiple tasks in batch with dependency resolution
    pub async fn execute_tasks_batch(
        &self,
        task_names: &[String],
    ) -> Result<Vec<TaskExecutionResult>> {
        // Resolve task dependencies and create execution order
        let execution_order = self.resolve_task_dependencies(task_names)?;
        let mut results = Vec::new();

        for task_name in execution_order {
            let result = self.execute_task(&task_name, None).await?;

            // If task failed and it's not configured to continue on error, stop
            if result.is_failure() {
                let task = self
                    .registry
                    .get_task(&task_name)
                    .ok_or_else(|| Error::task(format!("Task not found: {task_name}")))?;
                if !task.continue_on_error {
                    results.push(result);
                    break;
                }
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Resolve package tasks from package.json scripts
    pub async fn resolve_package_tasks(&self, package_name: &str) -> Result<Vec<TaskDefinition>> {
        let package_info = self
            .project
            .get_package(package_name)
            .ok_or_else(|| Error::task(format!("Package not found: {package_name}")))?;

        // Read package.json to extract scripts
        let package_json_path = package_info.path().join("package.json");
        let package_json_content = std::fs::read_to_string(&package_json_path)
            .map_err(|e| Error::task(format!("Failed to read package.json: {e}")))?;

        let package_json: serde_json::Value = serde_json::from_str(&package_json_content)
            .map_err(|e| Error::task(format!("Failed to parse package.json: {e}")))?;

        let mut tasks = Vec::new();

        if let Some(scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
            for (script_name, _script_command) in scripts {
                let task = TaskDefinition::new(
                    format!("{package_name}:{script_name}"),
                    format!("Run {script_name} script for {package_name}"),
                )
                .with_package_script(
                    super::PackageScript::new(script_name).for_package(package_name),
                )
                .with_scope(TaskScope::Package(package_name.to_string()));

                tasks.push(task);
            }
        }

        Ok(tasks)
    }

    /// Register a new task
    pub fn register_task(&mut self, task: TaskDefinition) -> Result<()> {
        self.registry.register_task(task)
    }

    /// Register multiple tasks
    pub fn register_tasks(&mut self, tasks: Vec<TaskDefinition>) -> Result<()> {
        for task in tasks {
            self.register_task(task)?;
        }
        Ok(())
    }

    /// Get all registered tasks
    #[must_use] pub fn list_tasks(&self) -> Vec<&TaskDefinition> {
        self.registry.list_tasks()
    }

    /// Get task by name
    #[must_use] pub fn get_task(&self, name: &str) -> Option<&TaskDefinition> {
        self.registry.get_task(name)
    }

    /// Remove a task
    pub fn remove_task(&mut self, name: &str) -> Result<()> {
        self.registry.remove_task(name)
    }

    /// Update execution context
    pub fn update_context(&mut self, context: ExecutionContext) {
        self.execution_context = context;
    }

    // Private helper methods

    /// Execute task with specific context
    async fn execute_task_with_context(
        &self,
        task: &TaskDefinition,
        context: &ExecutionContext,
    ) -> Result<TaskExecutionResult> {
        // Check conditions with context
        if !self.condition_checker.check_conditions_with_context(&task.conditions, context).await? {
            return Ok(
                TaskExecutionResult::new(&task.name).with_status_skipped("Conditions not met")
            );
        }

        // Execute with context
        self.executor.execute_task_with_context(task, &task.scope, context).await
    }

    /// Get tasks that should run for given changes
    async fn get_tasks_for_changes(&self, changes: &ChangeAnalysis) -> Result<Vec<TaskDefinition>> {
        let mut applicable_tasks = Vec::new();

        for task in self.registry.list_tasks() {
            // Check if task conditions match the changes
            let matches = self.condition_checker.task_matches_changes(task, changes).await?;
            if matches {
                applicable_tasks.push(task.clone());
            }
        }

        Ok(applicable_tasks)
    }

    /// Resolve task dependencies and return execution order
    fn resolve_task_dependencies(&self, task_names: &[String]) -> Result<Vec<String>> {
        let mut execution_order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for task_name in task_names {
            self.visit_task_dependencies(
                task_name,
                &mut execution_order,
                &mut visited,
                &mut visiting,
            )?;
        }

        Ok(execution_order)
    }

    /// Depth-first search for dependency resolution
    fn visit_task_dependencies(
        &self,
        task_name: &str,
        execution_order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(task_name) {
            return Ok(());
        }

        if visiting.contains(task_name) {
            return Err(Error::task(format!("Circular dependency detected: {task_name}")));
        }

        visiting.insert(task_name.to_string());

        if let Some(task) = self.registry.get_task(task_name) {
            for dependency in &task.dependencies {
                self.visit_task_dependencies(dependency, execution_order, visited, visiting)?;
            }
        }

        visiting.remove(task_name);
        visited.insert(task_name.to_string());
        execution_order.push(task_name.to_string());

        Ok(())
    }
}

/// Execution context for tasks
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Packages that are affected by changes
    pub affected_packages: Vec<String>,

    /// Files that have changed
    pub changed_files: Vec<sublime_git_tools::GitChangedFile>,

    /// Current branch
    pub current_branch: Option<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Working directory
    pub working_directory: Option<std::path::PathBuf>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set affected packages
    pub fn with_affected_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    /// Set changed files
    pub fn with_changed_files(mut self, files: Vec<sublime_git_tools::GitChangedFile>) -> Self {
        self.changed_files = files;
        self
    }

    /// Set current branch
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.current_branch = Some(branch.into());
        self
    }
}

impl TaskExecutionResult {
    /// Create result with skipped status
    #[must_use]
    pub fn with_status_skipped(mut self, reason: impl Into<String>) -> Self {
        self.status = super::TaskStatus::Skipped { reason: reason.into() };
        self
    }
}
