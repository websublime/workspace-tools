//! Task manager implementation
//!
//! The `TaskManager` orchestrates task execution, manages dependencies,
//! and coordinates with other monorepo systems for comprehensive workflow management.

use super::{
    types::{ConditionChecker, ExecutionContext, TaskExecutor, TaskManager, TaskRegistry},
    TaskDefinition, TaskExecutionResult, TaskScope,
};
use crate::analysis::ChangeAnalysis;
use crate::core::MonorepoProject;
use crate::error::{Error, Result};
use std::collections::HashSet;
use std::sync::Arc;

impl TaskManager {
    /// Create a new task manager with injected dependencies
    pub fn new(
        file_system_provider: Box<dyn crate::core::FileSystemProvider>,
        package_provider: Box<dyn crate::core::PackageProvider>,
        executor_package_provider: Box<dyn crate::core::PackageProvider>,
        executor_config_provider: Box<dyn crate::core::ConfigProvider>,
        git_provider: Box<dyn crate::core::GitProvider>,
        checker_config_provider: Box<dyn crate::core::ConfigProvider>,
        checker_package_provider: Box<dyn crate::core::PackageProvider>,
        checker_file_system_provider: Box<dyn crate::core::FileSystemProvider>,
    ) -> Result<Self> {
        let registry = TaskRegistry::new();
        
        let executor = TaskExecutor::new(
            executor_package_provider,
            executor_config_provider,
        )?;
        
        let condition_checker = ConditionChecker::new(
            git_provider,
            checker_config_provider,
            checker_package_provider,
            checker_file_system_provider,
        );

        Ok(Self {
            file_system: file_system_provider,
            package_provider,
            registry,
            executor,
            condition_checker,
            execution_context: ExecutionContext::default(),
        })
    }

    /// Create a new task manager from project (convenience method)
    /// 
    /// NOTE: This convenience method creates multiple provider instances from the same project.
    /// For better performance and memory usage, prefer using the `new()` method with 
    /// pre-created providers when creating multiple components.
    pub fn from_project(project: Arc<MonorepoProject>) -> Result<Self> {
        use crate::core::interfaces::DependencyFactory;
        
        // Create providers once and reuse them to avoid multiple Arc clones
        let file_system_provider = DependencyFactory::file_system_provider(Arc::clone(&project));
        let package_provider1 = DependencyFactory::package_provider(Arc::clone(&project));
        let package_provider2 = DependencyFactory::package_provider(Arc::clone(&project));
        let package_provider3 = DependencyFactory::package_provider(Arc::clone(&project));
        let config_provider1 = DependencyFactory::config_provider(Arc::clone(&project));
        let config_provider2 = DependencyFactory::config_provider(Arc::clone(&project));
        let git_provider = DependencyFactory::git_provider(Arc::clone(&project));
        
        Self::new(
            file_system_provider,
            package_provider1,
            package_provider2, // for executor
            config_provider1,  // for executor
            git_provider,
            config_provider2,  // for checker
            package_provider3, // for checker
            DependencyFactory::file_system_provider(project), // for checker
        )
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
        if !self.condition_checker.check_conditions(&task_definition.conditions)? {
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
        let applicable_tasks = self.get_tasks_for_changes(changes)?;

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
    pub fn resolve_package_tasks(&self, package_name: &str) -> Result<Vec<TaskDefinition>> {
        let package_info = self
            .package_provider
            .get_package(package_name)
            .ok_or_else(|| Error::task(format!("Package not found: {package_name}")))?;

        // Read package.json to extract scripts using FileSystem trait
        let package_json_path = package_info.path().join("package.json");
        let package_json_content = self.file_system.read_file_string(&package_json_path)
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
    #[must_use]
    pub fn list_tasks(&self) -> Vec<&TaskDefinition> {
        self.registry.list_tasks()
    }

    /// Get task by name
    #[must_use]
    pub fn get_task(&self, name: &str) -> Option<&TaskDefinition> {
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
        if !self.condition_checker.check_conditions_with_context(&task.conditions, context)? {
            return Ok(
                TaskExecutionResult::new(&task.name).with_status_skipped("Conditions not met")
            );
        }

        // Execute with context
        self.executor.execute_task_with_context(task, &task.scope, context).await
    }

    /// Get tasks that should run for given changes
    fn get_tasks_for_changes(&self, changes: &ChangeAnalysis) -> Result<Vec<TaskDefinition>> {
        let mut applicable_tasks = Vec::new();

        for task in self.registry.list_tasks() {
            // Check if task conditions match the changes
            let matches = self.condition_checker.task_matches_changes(task, changes)?;
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

impl TaskExecutionResult {
    /// Create result with skipped status
    #[must_use]
    pub fn with_status_skipped(mut self, reason: impl Into<String>) -> Self {
        self.status = super::TaskStatus::Skipped { reason: reason.into() };
        self
    }
}
