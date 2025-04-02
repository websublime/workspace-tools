use petgraph::{algo, graph::DiGraph, visit::Topo, Direction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::error::{TaskError, TaskResult};
use super::task::Task;

/// Represents a task dependency graph
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
    pub fn sorted_tasks(&self, mode: TaskSortMode) -> TaskResult<Vec<Task>> {
        match mode {
            TaskSortMode::Topological => Ok(self.topological_sort()),
            TaskSortMode::Parallel => Ok(self.parallel_sort()),
            TaskSortMode::Random => Ok(self.random_sort()),
        }
    }

    /// Get the task dependency levels (for visualization and parallel execution)
    /// Returns a Vec of Vecs, where each inner Vec contains tasks at the same level
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
    pub fn all_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    /// Get a task by name
    pub fn get_task(&self, name: &str) -> Option<Task> {
        self.tasks.get(name).cloned()
    }

    /// Returns the number of tasks in the graph
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    // Private helper methods for different sorting strategies

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
