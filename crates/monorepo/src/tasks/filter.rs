//! Task filtering functionality.
//!
//! This module provides the ability to filter tasks based on names, patterns, and package
//! associations. It's used to select specific tasks to run, enabling workflows like:
//! - Running tasks for a specific package
//! - Running tasks that match certain patterns
//! - Selectively including or excluding dependencies
//!
//! The filtering system supports glob patterns for flexible matching of task names.

use super::error::{TaskError, TaskResult};
use super::task::Task;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Filter for selecting tasks to run
///
/// The `TaskFilter` allows for granular selection of tasks to execute based on
/// glob patterns, package names, and dependency relationships. It's used to limit
/// the scope of task execution in a workspace.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{TaskFilter, Task};
///
/// // Create a filter for build tasks in the UI package
/// let filter = TaskFilter::new()
///     .with_include(vec!["build:*"])
///     .with_packages(vec!["ui"])
///     .include_dependencies(true);
///
/// // Create a filter for test tasks, excluding coverage
/// let test_filter = TaskFilter::new()
///     .with_include(vec!["test:*"])
///     .with_exclude(vec!["*:coverage"])
///     .include_dependencies(true)
///     .include_dependents(false);
///
/// // Apply filter to tasks
/// let filtered_tasks = filter.apply(&all_tasks)?;
/// ```
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
    /// Creates a new filter with default settings.
    ///
    /// Default settings:
    /// - No include patterns (matches all tasks)
    /// - No exclude patterns
    /// - No package filters
    /// - Dependencies included
    /// - Dependents not included
    ///
    /// # Returns
    ///
    /// A new filter with default settings.
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
    ///
    /// Creates a filter with default settings.
    ///
    /// # Returns
    ///
    /// A new filter with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// let filter = TaskFilter::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Add patterns to include
    ///
    /// Adds one or more glob patterns to match task names that should be included.
    /// If no include patterns are specified, all tasks match by default.
    ///
    /// # Arguments
    ///
    /// * `patterns` - List of glob patterns to include
    ///
    /// # Returns
    ///
    /// The modified filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// // Include all test and lint tasks
    /// let filter = TaskFilter::new()
    ///     .with_include(vec!["test*", "lint*"]);
    /// ```
    #[must_use]
    pub fn with_include(mut self, patterns: Vec<impl Into<String>>) -> Self {
        for pattern in patterns {
            self.include.push(pattern.into());
        }
        self
    }

    /// Add patterns to exclude
    ///
    /// Adds one or more glob patterns to match task names that should be excluded.
    /// Exclude patterns take precedence over include patterns.
    ///
    /// # Arguments
    ///
    /// * `patterns` - List of glob patterns to exclude
    ///
    /// # Returns
    ///
    /// The modified filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// // Include all tasks except slow ones
    /// let filter = TaskFilter::new()
    ///     .with_exclude(vec!["*slow*", "*integration*"]);
    /// ```
    #[must_use]
    pub fn with_exclude(mut self, patterns: Vec<impl Into<String>>) -> Self {
        for pattern in patterns {
            self.exclude.push(pattern.into());
        }
        self
    }

    /// Add packages to filter by
    ///
    /// Limit tasks to those associated with specific packages.
    ///
    /// # Arguments
    ///
    /// * `packages` - List of package names
    ///
    /// # Returns
    ///
    /// The modified filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// // Only run tasks for the ui and api packages
    /// let filter = TaskFilter::new()
    ///     .with_packages(vec!["ui", "api"]);
    /// ```
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<impl Into<String>>) -> Self {
        for package in packages {
            self.packages.push(package.into());
        }
        self
    }

    /// Set whether to include dependencies
    ///
    /// When true, dependencies of matched tasks will also be included in the result.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include dependencies
    ///
    /// # Returns
    ///
    /// The modified filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// // Include dependencies of matched tasks
    /// let filter = TaskFilter::new()
    ///     .with_include(vec!["build"])
    ///     .include_dependencies(true);
    ///
    /// // Don't include dependencies
    /// let filter = TaskFilter::new()
    ///     .with_include(vec!["test"])
    ///     .include_dependencies(false);
    /// ```
    #[must_use]
    pub fn include_dependencies(mut self, include: bool) -> Self {
        self.include_dependencies = include;
        self
    }

    /// Set whether to include dependents
    ///
    /// When true, tasks that depend on matched tasks will also be included in the result.
    ///
    /// # Arguments
    ///
    /// * `include` - Whether to include dependents
    ///
    /// # Returns
    ///
    /// The modified filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskFilter;
    ///
    /// // Include dependents of matched tasks (downstream impact)
    /// let filter = TaskFilter::new()
    ///     .with_include(vec!["core"])
    ///     .include_dependents(true);
    /// ```
    #[must_use]
    pub fn include_dependents(mut self, include: bool) -> Self {
        self.include_dependents = include;
        self
    }

    /// Apply the filter to a list of tasks
    ///
    /// Filters the provided tasks according to the filter's criteria.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The list of tasks to filter
    ///
    /// # Returns
    ///
    /// A list of tasks that match the filter criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if glob patterns are invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{TaskFilter, Task};
    ///
    /// # fn example(tasks: Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
    /// let filter = TaskFilter::new().with_include(vec!["test*"]);
    /// let filtered_tasks = filter.apply(&tasks)?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Checks if a task matches the filter criteria.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to check
    /// * `include_globs` - Set of include patterns
    /// * `exclude_globs` - Set of exclude patterns
    ///
    /// # Returns
    ///
    /// `true` if the task matches the filter, `false` otherwise.
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

    /// Builds a glob set from patterns.
    ///
    /// # Arguments
    ///
    /// * `patterns` - List of glob patterns
    ///
    /// # Returns
    ///
    /// A compiled glob set.
    ///
    /// # Errors
    ///
    /// Returns an error if any pattern is invalid.
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

    /// Recursively collects dependencies of a task.
    ///
    /// # Arguments
    ///
    /// * `all_tasks` - All available tasks
    /// * `dependencies` - Set to collect dependencies into
    /// * `task` - Task to collect dependencies for
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
