use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::error::{TaskError, TaskResult};
use super::task::Task;

/// Filter for selecting tasks to run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    /// Include tasks matching these patterns
    pub include: Vec<String>,

    /// Exclude tasks matching these patterns
    pub exclude: Vec<String>,

    /// Only package-specific tasks in these packages
    pub packages: Vec<String>,

    /// Whether to include dependencies of matched tasks
    pub include_dependencies: bool,

    /// Whether to include dependents of matched tasks
    pub include_dependents: bool,
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            include: vec![],
            exclude: vec![],
            packages: vec![],
            include_dependencies: true,
            include_dependents: false,
        }
    }
}

impl TaskFilter {
    /// Create a new empty task filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add patterns to include
    #[must_use]
    pub fn with_include(mut self, patterns: Vec<impl Into<String>>) -> Self {
        for pattern in patterns {
            self.include.push(pattern.into());
        }
        self
    }

    /// Add patterns to exclude
    #[must_use]
    pub fn with_exclude(mut self, patterns: Vec<impl Into<String>>) -> Self {
        for pattern in patterns {
            self.exclude.push(pattern.into());
        }
        self
    }

    /// Add packages to filter by
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<impl Into<String>>) -> Self {
        for package in packages {
            self.packages.push(package.into());
        }
        self
    }

    /// Set whether to include dependencies
    #[must_use]
    pub fn include_dependencies(mut self, include: bool) -> Self {
        self.include_dependencies = include;
        self
    }

    /// Set whether to include dependents
    #[must_use]
    pub fn include_dependents(mut self, include: bool) -> Self {
        self.include_dependents = include;
        self
    }

    /// Apply the filter to a list of tasks
    pub fn apply(&self, tasks: &[Task]) -> TaskResult<Vec<Task>> {
        // Build glob patterns for inclusion and exclusion
        let include_globs = TaskFilter::build_glob_set(&self.include)?;
        let exclude_globs = TaskFilter::build_glob_set(&self.exclude)?;

        // Filter tasks
        let mut filtered_tasks = Vec::new();
        let mut included_task_names = HashSet::new();

        for task in tasks {
            if self.matches_task(task, &include_globs, &exclude_globs) {
                filtered_tasks.push(task.clone());
                included_task_names.insert(task.name.clone());
            }
        }

        // Handle dependencies and dependents if needed
        if self.include_dependencies || self.include_dependents {
            // We need to do a second pass to include dependencies/dependents
            let mut dependencies_to_add = HashSet::new();

            // First identify all dependencies and dependents we need to add
            if self.include_dependencies {
                for task in &filtered_tasks {
                    self.collect_dependencies(tasks, &mut dependencies_to_add, task);
                }
            }

            if self.include_dependents {
                for task in tasks {
                    if !included_task_names.contains(&task.name) {
                        // See if any of its dependencies are included
                        let mut has_included_dependency = false;
                        for dep in &task.dependencies {
                            if included_task_names.contains(dep) {
                                has_included_dependency = true;
                                break;
                            }
                        }

                        if has_included_dependency {
                            dependencies_to_add.insert(task.name.clone());
                        }
                    }
                }
            }

            // Add the additional tasks
            for task in tasks {
                if dependencies_to_add.contains(&task.name)
                    && !included_task_names.contains(&task.name)
                {
                    filtered_tasks.push(task.clone());
                    included_task_names.insert(task.name.clone());
                }
            }
        }

        Ok(filtered_tasks)
    }

    // Helper method to check if a task matches the filter
    fn matches_task(&self, task: &Task, include_globs: &GlobSet, exclude_globs: &GlobSet) -> bool {
        // Check package filter first
        if !self.packages.is_empty() {
            if let Some(package) = &task.package {
                let mut package_match = false;
                for p in &self.packages {
                    if p == package {
                        package_match = true;
                        break;
                    }
                }
                if !package_match {
                    return false;
                }
            } else {
                // If task doesn't have a package but we're filtering by package, exclude it
                return false;
            }
        }

        // If no include patterns and no package filters, include by default
        let should_include = self.include.is_empty() || include_globs.is_match(&task.name);

        // Exclude patterns override include patterns
        let should_exclude = !self.exclude.is_empty() && exclude_globs.is_match(&task.name);

        should_include && !should_exclude
    }

    // Helper method to build a GlobSet from patterns
    fn build_glob_set(patterns: &[String]) -> TaskResult<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            match Glob::new(pattern) {
                Ok(glob) => {
                    builder.add(glob);
                }
                Err(err) => {
                    return Err(TaskError::FilterError(format!(
                        "Invalid glob pattern '{pattern}': {err}"
                    )));
                }
            }
        }

        match builder.build() {
            Ok(set) => Ok(set),
            Err(err) => Err(TaskError::FilterError(format!("Failed to build glob set: {err}"))),
        }
    }

    // Helper to recursively collect all dependencies
    #[allow(clippy::only_used_in_recursion)]
    fn collect_dependencies(
        &self,
        all_tasks: &[Task],
        dependencies: &mut HashSet<String>,
        task: &Task,
    ) {
        for dep_name in &task.dependencies {
            if dependencies.insert(dep_name.clone()) {
                // Only continue recursively if this is a new addition
                for t in all_tasks {
                    if &t.name == dep_name {
                        self.collect_dependencies(all_tasks, dependencies, t);
                        break;
                    }
                }
            }
        }
    }
}
