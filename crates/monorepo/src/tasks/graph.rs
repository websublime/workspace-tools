//! Task dependency graph implementation.
//!
//! This module provides functionality for building and manipulating task dependency graphs.
//! It handles topological sorting, cycle detection, and determining execution order
//! to ensure tasks are executed in the correct sequence.

use super::error::{TaskError, TaskResult};
use super::task::Task;
use petgraph::{algo, graph::DiGraph, visit::Topo, Direction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a task dependency graph
///
/// This type maintains the relationships between tasks and provides methods
/// for analyzing and working with the dependency structure.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{Task, TaskGraph, TaskSortMode};
///
/// # fn example(tasks: Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
/// // Create graph from tasks
/// let graph = TaskGraph::from_tasks(&tasks)?;
///
/// // Get topologically sorted tasks (dependencies first)
/// let sorted_tasks = graph.sorted_tasks(TaskSortMode::Topological)?;
///
/// // Get tasks by levels for parallel execution
/// let task_levels = graph.task_levels();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TaskGraph {
    /// The internal graph structure
    graph: DiGraph<String, ()>,

    /// Mapping from task names to their node indices
    node_indices: HashMap<String, petgraph::graph::NodeIndex>,

    /// All tasks by name
    tasks: HashMap<String, Task>,
}

/// Task sorting approach
///
/// Determines how tasks should be sorted when retrieving them from the graph.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::TaskSortMode;
///
/// // Sort tasks topologically (dependencies first)
/// let mode = TaskSortMode::Topological;
///
/// // Sort tasks for maximum parallelism
/// let mode = TaskSortMode::Parallel;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSortMode {
    /// Topological sort - dependencies first
    Topological,
    /// Parallel sort - optimize for parallelism
    Parallel,
    /// Random sort - no particular order
    Random,
}

impl TaskGraph {
    /// Create a new task graph from a list of tasks
    ///
    /// Builds a directed graph representing the dependencies between tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - List of tasks to build the graph from
    ///
    /// # Returns
    ///
    /// A new task graph.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A dependency references a non-existent task
    /// - There are circular dependencies between tasks
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskGraph};
    ///
    /// # fn example(tasks: Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
    /// let graph = TaskGraph::from_tasks(&tasks)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_tasks(tasks: &[Task]) -> TaskResult<Self> {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();
        let mut task_map = HashMap::new();

        // First pass: add all tasks as nodes
        for task in tasks {
            // Store the task in the map
            task_map.insert(task.name.clone(), task.clone());

            // Add node to graph if it doesn't exist already
            if !node_indices.contains_key(&task.name) {
                let idx = graph.add_node(task.name.clone());
                node_indices.insert(task.name.clone(), idx);
            }
        }

        // Second pass: add all dependencies as edges
        for task in tasks {
            let from_idx = node_indices[&task.name];

            for dep_name in &task.dependencies {
                // Make sure the dependency exists
                if !node_indices.contains_key(dep_name) {
                    return Err(TaskError::TaskNotFound(dep_name.clone()));
                }

                let to_idx = node_indices[dep_name];
                graph.add_edge(to_idx, from_idx, ()); // Dependency points to dependent
            }
        }

        // Check for cycles
        if let Err(cycle) = algo::toposort(&graph, None) {
            let cycle_node = cycle.node_id();
            let cycle_task = graph[cycle_node].clone();

            // Try to get a more complete cycle
            let mut cycle_str = String::new();
            let mut visited = HashSet::new();
            let mut stack = Vec::new();
            stack.push(cycle_task.clone());

            while let Some(task_name) = stack.pop() {
                cycle_str.push_str(&task_name);
                cycle_str.push_str(" -> ");

                if let Some(task_idx) = node_indices.get(&task_name) {
                    for neighbor in graph.neighbors_directed(*task_idx, Direction::Incoming) {
                        let neighbor_name = &graph[neighbor];

                        if neighbor_name == &cycle_task {
                            // Complete the cycle
                            cycle_str.push_str(&cycle_task);
                            break;
                        }

                        if !visited.contains(neighbor_name) {
                            visited.insert(neighbor_name.clone());
                            stack.push(neighbor_name.clone());
                            break;
                        }
                    }
                }

                if stack.is_empty() || cycle_str.contains(&cycle_task) {
                    break;
                }
            }

            return Err(TaskError::CircularDependency(cycle_str));
        }

        Ok(Self { graph, node_indices, tasks: task_map })
    }

    /// Get a sorted list of tasks according to the sort mode
    ///
    /// Returns tasks sorted according to the specified mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The sorting mode to use
    ///
    /// # Returns
    ///
    /// A list of sorted tasks.
    ///
    /// # Errors
    ///
    /// Returns an error if sorting fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{TaskGraph, TaskSortMode};
    ///
    /// # fn example(graph: TaskGraph) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get tasks sorted topologically
    /// let tasks = graph.sorted_tasks(TaskSortMode::Topological)?;
    ///
    /// // Get tasks sorted for parallel execution
    /// let tasks = graph.sorted_tasks(TaskSortMode::Parallel)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sorted_tasks(&self, mode: TaskSortMode) -> TaskResult<Vec<Task>> {
        match mode {
            TaskSortMode::Topological => Ok(self.topological_sort()),
            TaskSortMode::Parallel => Ok(self.parallel_sort()),
            TaskSortMode::Random => Ok(self.random_sort()),
        }
    }

    /// Get the task dependency levels (for visualization and parallel execution)
    ///
    /// Returns tasks grouped by their dependency levels, where tasks at the same level
    /// can be executed in parallel.
    ///
    /// Level 0 contains tasks with no dependencies.
    /// Level 1 contains tasks that only depend on level 0 tasks.
    /// Level n contains tasks that depend on tasks in levels 0 through n-1.
    ///
    /// # Returns
    ///
    /// A list of lists, where each inner list contains tasks at the same level.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) {
    /// let levels = graph.task_levels();
    ///
    /// // Process each level in sequence
    /// for (i, level) in levels.iter().enumerate() {
    ///     println!("Level {}: {} tasks", i, level.len());
    ///     // Tasks in this level can be executed in parallel
    /// }
    /// # }
    /// ```
    pub fn task_levels(&self) -> Vec<Vec<Task>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        // Start with tasks that have no dependencies (roots)
        let mut current_level = Vec::new();
        for task_name in self.tasks.keys() {
            let idx = self.node_indices[task_name];
            let in_degree = self.graph.neighbors_directed(idx, Direction::Outgoing).count();

            if in_degree == 0 {
                if let Some(task) = self.tasks.get(task_name) {
                    current_level.push(task.clone());
                    visited.insert(task_name.clone());
                }
            }
        }

        // Add the first level
        if !current_level.is_empty() {
            result.push(current_level);
        }

        // Keep processing levels until we've visited all tasks
        while visited.len() < self.tasks.len() {
            let mut next_level = Vec::new();

            for task_name in self.tasks.keys() {
                if visited.contains(task_name) {
                    continue;
                }

                let task_idx = self.node_indices[task_name];
                let dependencies = self
                    .graph
                    .neighbors_directed(task_idx, Direction::Outgoing)
                    .map(|idx| self.graph[idx].clone())
                    .collect::<HashSet<_>>();

                // Check if all dependencies are already visited
                let mut all_deps_visited = true;
                for dep in &dependencies {
                    if !visited.contains(dep) {
                        all_deps_visited = false;
                        break;
                    }
                }

                if all_deps_visited {
                    if let Some(task) = self.tasks.get(task_name) {
                        next_level.push(task.clone());
                        visited.insert(task_name.clone());
                    }
                }
            }

            if next_level.is_empty() {
                // This shouldn't happen with a valid DAG, but just in case
                break;
            }

            result.push(next_level);
        }

        result
    }

    /// Get all direct dependencies of a task
    ///
    /// Returns the tasks that the specified task directly depends on.
    ///
    /// # Arguments
    ///
    /// * `task_name` - Name of the task to get dependencies for
    ///
    /// # Returns
    ///
    /// A list of tasks that the specified task depends on.
    ///
    /// # Errors
    ///
    /// Returns an error if the task doesn't exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get dependencies of "build" task
    /// let deps = graph.dependencies_of("build")?;
    /// println!("'build' depends on {} tasks", deps.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn dependencies_of(&self, task_name: &str) -> TaskResult<Vec<Task>> {
        if let Some(task_idx) = self.node_indices.get(task_name) {
            let deps = self
                .graph
                .neighbors_directed(*task_idx, Direction::Outgoing)
                .map(|idx| self.graph[idx].clone())
                .filter_map(|name| self.tasks.get(&name).cloned())
                .collect();
            Ok(deps)
        } else {
            Err(TaskError::TaskNotFound(task_name.to_string()))
        }
    }

    /// Get all direct dependents of a task
    ///
    /// Returns the tasks that directly depend on the specified task.
    ///
    /// # Arguments
    ///
    /// * `task_name` - Name of the task to get dependents for
    ///
    /// # Returns
    ///
    /// A list of tasks that depend on the specified task.
    ///
    /// # Errors
    ///
    /// Returns an error if the task doesn't exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get tasks that depend on "core" task
    /// let dependents = graph.dependents_of("core")?;
    /// println!("{} tasks depend on 'core'", dependents.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn dependents_of(&self, task_name: &str) -> TaskResult<Vec<Task>> {
        if let Some(task_idx) = self.node_indices.get(task_name) {
            let deps = self
                .graph
                .neighbors_directed(*task_idx, Direction::Incoming)
                .map(|idx| self.graph[idx].clone())
                .filter_map(|name| self.tasks.get(&name).cloned())
                .collect();
            Ok(deps)
        } else {
            Err(TaskError::TaskNotFound(task_name.to_string()))
        }
    }

    /// Get all tasks in the graph
    ///
    /// Returns all tasks in the graph, unsorted.
    ///
    /// # Returns
    ///
    /// A list of all tasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) {
    /// let all_tasks = graph.all_tasks();
    /// println!("Graph contains {} tasks", all_tasks.len());
    /// # }
    /// ```
    pub fn all_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    /// Get a task by name
    ///
    /// Retrieves a specific task by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the task to retrieve
    ///
    /// # Returns
    ///
    /// The task if found, or None if not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) {
    /// if let Some(task) = graph.get_task("build") {
    ///     println!("Found task: {}", task.name);
    /// } else {
    ///     println!("Task not found");
    /// }
    /// # }
    /// ```
    pub fn get_task(&self, name: &str) -> Option<Task> {
        self.tasks.get(name).cloned()
    }

    /// Returns the number of tasks in the graph
    ///
    /// # Returns
    ///
    /// The number of tasks in the graph.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::TaskGraph;
    ///
    /// # fn example(graph: TaskGraph) {
    /// println!("Graph contains {} tasks", graph.task_count());
    /// # }
    /// ```
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Sorts tasks topologically.
    ///
    /// Tasks are sorted such that dependencies come before the tasks that depend on them.
    ///
    /// # Returns
    ///
    /// A list of tasks sorted topologically.
    fn topological_sort(&self) -> Vec<Task> {
        let mut sorted = Vec::new();
        let mut topo = Topo::new(&self.graph);

        while let Some(node_idx) = topo.next(&self.graph) {
            let task_name = &self.graph[node_idx];
            if let Some(task) = self.tasks.get(task_name) {
                sorted.push(task.clone());
            }
        }

        // Reverse since we want dependencies first
        let mut reversed = Vec::new();
        for i in (0..sorted.len()).rev() {
            reversed.push(sorted[i].clone());
        }

        reversed
    }

    /// Sorts tasks for parallel execution.
    ///
    /// Tasks are sorted by levels, where tasks at the same level can be executed in parallel.
    ///
    /// # Returns
    ///
    /// A list of tasks sorted for parallel execution.
    fn parallel_sort(&self) -> Vec<Task> {
        // For parallel sort, we return tasks grouped by levels
        let levels = self.task_levels();

        // Flatten the levels into a single list
        let mut result = Vec::new();
        for level in levels {
            for task in level {
                result.push(task);
            }
        }

        result
    }

    fn random_sort(&self) -> Vec<Task> {
        // For simplicity in this implementation, we'll just use topological sort
        // In a real implementation, this would shuffle the tasks while respecting dependencies
        self.topological_sort()
    }
}
