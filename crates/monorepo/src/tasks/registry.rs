//! Task registry for managing task definitions
//!
//! The TaskRegistry provides storage, retrieval, and organization of task definitions,
//! with support for filtering, searching, and validation.

use super::{TaskDefinition, TaskScope};
use crate::error::{Error, Result};
use std::collections::HashMap;

/// Registry for storing and managing task definitions
#[derive(Debug, Clone)]
pub struct TaskRegistry {
    /// All registered tasks indexed by name
    tasks: HashMap<String, TaskDefinition>,

    /// Tasks organized by scope for quick filtering
    scope_index: HashMap<TaskScope, Vec<String>>,

    /// Tasks organized by priority
    priority_index: HashMap<u32, Vec<String>>,
}

impl TaskRegistry {
    /// Create a new empty task registry
    pub fn new() -> Self {
        Self { tasks: HashMap::new(), scope_index: HashMap::new(), priority_index: HashMap::new() }
    }

    /// Register a new task
    pub fn register_task(&mut self, task: TaskDefinition) -> Result<()> {
        // Validate task definition
        self.validate_task(&task)?;

        let task_name = task.name.clone();
        let task_scope = task.scope.clone();
        let task_priority = self.get_priority_value(&task.priority);

        // Check for duplicate names
        if self.tasks.contains_key(&task_name) {
            return Err(Error::task(format!("Task already registered: {task_name}")));
        }

        // Add to scope index
        self.scope_index.entry(task_scope).or_default().push(task_name.clone());

        // Add to priority index
        self.priority_index.entry(task_priority).or_default().push(task_name.clone());

        // Store the task
        self.tasks.insert(task_name, task);

        Ok(())
    }

    /// Get a task by name
    pub fn get_task(&self, name: &str) -> Option<&TaskDefinition> {
        self.tasks.get(name)
    }

    /// Get all tasks
    pub fn list_tasks(&self) -> Vec<&TaskDefinition> {
        self.tasks.values().collect()
    }

    /// Get tasks by scope
    pub fn get_tasks_for_scope(&self, scope: &TaskScope) -> Vec<&TaskDefinition> {
        self.scope_index
            .get(scope)
            .map(|task_names| task_names.iter().filter_map(|name| self.tasks.get(name)).collect())
            .unwrap_or_default()
    }

    /// Get tasks by priority (sorted by priority, highest first)
    pub fn get_tasks_by_priority(&self) -> Vec<&TaskDefinition> {
        let mut priorities: Vec<_> = self.priority_index.keys().collect();
        priorities.sort_by(|a, b| b.cmp(a)); // Highest priority first

        let mut tasks = Vec::new();
        for priority in priorities {
            if let Some(task_names) = self.priority_index.get(priority) {
                for task_name in task_names {
                    if let Some(task) = self.tasks.get(task_name) {
                        tasks.push(task);
                    }
                }
            }
        }

        tasks
    }

    /// Get tasks that have package scripts
    pub fn get_package_script_tasks(&self) -> Vec<&TaskDefinition> {
        self.tasks.values().filter(|task| !task.package_scripts.is_empty()).collect()
    }

    /// Get tasks that match a pattern
    pub fn find_tasks_by_pattern(&self, pattern: &str) -> Vec<&TaskDefinition> {
        self.tasks
            .values()
            .filter(|task| task.name.contains(pattern) || task.description.contains(pattern))
            .collect()
    }

    /// Get tasks that depend on a specific task
    pub fn get_dependent_tasks(&self, task_name: &str) -> Vec<&TaskDefinition> {
        self.tasks
            .values()
            .filter(|task| task.dependencies.contains(&task_name.to_string()))
            .collect()
    }

    /// Get tasks for a specific package
    pub fn get_package_tasks(&self, package_name: &str) -> Vec<&TaskDefinition> {
        self.tasks
            .values()
            .filter(|task| match &task.scope {
                TaskScope::Package(pkg) => pkg == package_name,
                _ => task
                    .package_scripts
                    .iter()
                    .any(|script| script.package_name.as_deref() == Some(package_name)),
            })
            .collect()
    }

    /// Remove a task
    pub fn remove_task(&mut self, name: &str) -> Result<()> {
        let task = self
            .tasks
            .remove(name)
            .ok_or_else(|| Error::task(format!("Task not found: {name}")))?;

        // Remove from scope index
        if let Some(scope_tasks) = self.scope_index.get_mut(&task.scope) {
            scope_tasks.retain(|task_name| task_name != name);
            if scope_tasks.is_empty() {
                self.scope_index.remove(&task.scope);
            }
        }

        // Remove from priority index
        let priority = self.get_priority_value(&task.priority);
        if let Some(priority_tasks) = self.priority_index.get_mut(&priority) {
            priority_tasks.retain(|task_name| task_name != name);
            if priority_tasks.is_empty() {
                self.priority_index.remove(&priority);
            }
        }

        Ok(())
    }

    /// Update a task
    pub fn update_task(&mut self, task: TaskDefinition) -> Result<()> {
        let task_name = task.name.clone();

        // Remove old task if it exists
        if self.tasks.contains_key(&task_name) {
            self.remove_task(&task_name)?;
        }

        // Register the updated task
        self.register_task(task)
    }

    /// Clear all tasks
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.scope_index.clear();
        self.priority_index.clear();
    }

    /// Get task count
    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Get task names
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Validate all registered tasks
    pub fn validate_all_tasks(&self) -> Result<()> {
        for task in self.tasks.values() {
            self.validate_task(task)?;
        }
        Ok(())
    }

    /// Get tasks with circular dependencies
    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for task_name in self.tasks.keys() {
            if !visited.contains(task_name) {
                if let Some(cycle) =
                    self.find_cycle(task_name, &mut visited, &mut rec_stack, &mut Vec::new())
                {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    // Private helper methods

    /// Validate a task definition
    fn validate_task(&self, task: &TaskDefinition) -> Result<()> {
        // Check task name
        if task.name.is_empty() {
            return Err(Error::task("Task name cannot be empty"));
        }

        // Check that dependencies exist
        for dependency in &task.dependencies {
            if !self.tasks.contains_key(dependency) && dependency != &task.name {
                return Err(Error::task(format!("Task dependency not found: {dependency}")));
            }
        }

        // Check for self-dependency
        if task.dependencies.contains(&task.name) {
            return Err(Error::task(format!("Task cannot depend on itself: {}", task.name)));
        }

        // Validate commands
        for command in &task.commands {
            if command.command.program.is_empty() {
                return Err(Error::task("Command program cannot be empty"));
            }
        }

        // Validate package scripts
        for script in &task.package_scripts {
            if script.script_name.is_empty() {
                return Err(Error::task("Package script name cannot be empty"));
            }
        }

        Ok(())
    }

    /// Get numeric priority value
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::unused_self)]
    fn get_priority_value(&self, priority: &super::TaskPriority) -> u32 {
        match priority {
            super::TaskPriority::Low => 0,
            super::TaskPriority::Normal => 50,
            super::TaskPriority::High => 100,
            super::TaskPriority::Critical => 200,
            super::TaskPriority::Custom(value) => *value,
        }
    }

    /// Find circular dependency cycles
    fn find_cycle(
        &self,
        task_name: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(task_name.to_string());
        rec_stack.insert(task_name.to_string());
        path.push(task_name.to_string());

        if let Some(task) = self.tasks.get(task_name) {
            for dependency in &task.dependencies {
                if !visited.contains(dependency) {
                    if let Some(cycle) = self.find_cycle(dependency, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dependency) {
                    // Found cycle
                    if let Some(cycle_start) = path.iter().position(|t| t == dependency) {
                        return Some(path[cycle_start..].to_vec());
                    }
                    // If we can't find the position (which shouldn't happen in a well-formed graph),
                    // return the entire path as the cycle
                    return Some(path.clone());
                }
            }
        }

        path.pop();
        rec_stack.remove(task_name);
        None
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
