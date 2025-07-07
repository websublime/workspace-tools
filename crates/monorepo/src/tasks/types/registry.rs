//! Task registry type definitions

use super::{TaskDefinition, TaskScope};
use std::collections::HashMap;

/// Registry for storing and managing task definitions
#[derive(Debug, Clone)]
pub struct TaskRegistry {
    /// All registered tasks indexed by name
    pub(crate) tasks: HashMap<String, TaskDefinition>,

    /// Tasks organized by scope for quick filtering
    pub(crate) scope_index: HashMap<TaskScope, Vec<String>>,

    /// Tasks organized by priority
    pub(crate) priority_index: HashMap<u32, Vec<String>>,
}

impl TaskRegistry {
    /// Create a new empty task registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            scope_index: HashMap::new(),
            priority_index: HashMap::new(),
        }
    }

    /// Add a task to the registry (alias for register_task)
    pub fn add_task(&mut self, task: TaskDefinition) -> crate::error::Result<()> {
        self.register_task(task)
    }

    /// Register a new task
    pub fn register_task(&mut self, task: TaskDefinition) -> crate::error::Result<()> {
        if self.tasks.contains_key(&task.name) {
            return Err(crate::error::Error::Task(
                format!("Task '{}' already exists", task.name),
            ));
        }

        let name = task.name.clone();
        let scope = task.scope.clone();
        let priority_value = match task.priority {
            super::TaskPriority::Low => 0,
            super::TaskPriority::Normal => 50,
            super::TaskPriority::High => 100,
            super::TaskPriority::Critical => 200,
            super::TaskPriority::Custom(value) => value,
        };

        // Update indexes
        self.scope_index.entry(scope).or_default().push(name.clone());
        self.priority_index.entry(priority_value).or_default().push(name.clone());

        // Add task
        self.tasks.insert(name, task);
        Ok(())
    }

    /// Get tasks by scope (alias for compatibility)
    #[must_use]
    pub fn get_tasks_for_scope(&self, scope: &TaskScope) -> Vec<&TaskDefinition> {
        self.get_tasks_by_scope(scope)
    }

    /// List all tasks
    #[must_use]
    pub fn list_tasks(&self) -> Vec<&TaskDefinition> {
        self.tasks.values().collect()
    }

    /// Get a task by name
    #[must_use]
    pub fn get_task(&self, name: &str) -> Option<&TaskDefinition> {
        self.tasks.get(name)
    }

    /// Update an existing task
    pub fn update_task(&mut self, task: TaskDefinition) -> crate::error::Result<()> {
        if !self.tasks.contains_key(&task.name) {
            return Err(crate::error::Error::Task(
                format!("Task '{}' does not exist", task.name),
            ));
        }

        // Remove from old indexes
        self.remove_from_indexes(&task.name);

        // Add to new indexes
        let name = task.name.clone();
        let scope = task.scope.clone();
        let priority_value = match task.priority {
            super::TaskPriority::Low => 0,
            super::TaskPriority::Normal => 50,
            super::TaskPriority::High => 100,
            super::TaskPriority::Critical => 200,
            super::TaskPriority::Custom(value) => value,
        };

        self.scope_index.entry(scope).or_default().push(name.clone());
        self.priority_index.entry(priority_value).or_default().push(name.clone());

        // Update task
        self.tasks.insert(name, task);
        Ok(())
    }

    /// Remove a task by name
    pub fn remove_task(&mut self, name: &str) -> crate::error::Result<bool> {
        if self.tasks.remove(name).is_some() {
            self.remove_from_indexes(name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Remove a task by name (manager version that returns unit)
    pub fn remove_task_unit(&mut self, name: &str) -> crate::error::Result<()> {
        self.remove_task(name)?;
        Ok(())
    }

    /// Get tasks by scope
    #[must_use]
    pub fn get_tasks_by_scope(&self, scope: &TaskScope) -> Vec<&TaskDefinition> {
        self.scope_index
            .get(scope)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.tasks.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get tasks by priority
    #[must_use]
    pub fn get_tasks_by_priority(&self, priority: super::TaskPriority) -> Vec<&TaskDefinition> {
        let priority_value = match priority {
            super::TaskPriority::Low => 0,
            super::TaskPriority::Normal => 50,
            super::TaskPriority::High => 100,
            super::TaskPriority::Critical => 200,
            super::TaskPriority::Custom(value) => value,
        };

        self.priority_index
            .get(&priority_value)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.tasks.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get direct dependencies of a task
    #[must_use]
    pub fn get_dependencies(&self, task_name: &str) -> Vec<String> {
        self.tasks
            .get(task_name)
            .map(|task| task.dependencies.clone())
            .unwrap_or_default()
    }

    /// Get all dependencies (transitive) of a task
    #[must_use]
    pub fn get_all_dependencies(&self, task_name: &str) -> Vec<String> {
        let mut all_deps = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_dependencies(task_name, &mut all_deps, &mut visited);
        all_deps
    }

    /// Helper to remove task from indexes
    fn remove_from_indexes(&mut self, name: &str) {
        // Remove from scope index
        for names in self.scope_index.values_mut() {
            names.retain(|n| n != name);
        }
        // Remove from priority index
        for names in self.priority_index.values_mut() {
            names.retain(|n| n != name);
        }
    }

    /// Helper to recursively collect dependencies
    fn collect_dependencies(
        &self,
        task_name: &str,
        all_deps: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(task_name) {
            return;
        }
        visited.insert(task_name.to_string());

        if let Some(task) = self.tasks.get(task_name) {
            for dep in &task.dependencies {
                if !all_deps.contains(dep) {
                    all_deps.push(dep.clone());
                }
                self.collect_dependencies(dep, all_deps, visited);
            }
        }
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
