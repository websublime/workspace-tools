//! Parallel task execution functionality.
//!
//! This module provides the infrastructure to execute tasks in parallel,
//! respecting their dependency relationships. It manages the scheduling
//! and coordination of tasks to maximize throughput while ensuring
//! dependencies are satisfied.
//!
//! Note: Despite the name, the current implementation is sequential but
//! provides the API for future parallel execution capabilities.

use super::error::TaskResult;
use super::graph::TaskGraph;
use super::runner::TaskRunner;
use super::task::{Task, TaskExecution, TaskStatus};
use crate::TaskResultInfo;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Configuration for parallel task execution
///
/// Controls the behavior of the parallel task executor, including
/// concurrency limits and failure handling.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::ParallelExecutionConfig;
///
/// // Default configuration
/// let default_config = ParallelExecutionConfig::default();
///
/// // Custom configuration
/// let custom_config = ParallelExecutionConfig {
///     max_parallel: 4,         // Run up to 4 tasks in parallel
///     fail_fast: true,         // Stop on first failure
///     show_progress: true,     // Show execution progress
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    /// Maximum number of parallel tasks
    pub max_parallel: usize,

    /// Whether to stop on first failure
    pub fail_fast: bool,

    /// Whether to output progress
    pub show_progress: bool,
}

impl Default for ParallelExecutionConfig {
    /// Creates a default configuration.
    ///
    /// Defaults:
    /// - max_parallel: Number of CPU cores
    /// - fail_fast: false
    /// - show_progress: true
    ///
    /// # Returns
    ///
    /// A default configuration.
    fn default() -> Self {
        Self { max_parallel: num_cpus::get(), fail_fast: false, show_progress: true }
    }
}

/// Execution engine for tasks (named "Parallel" for API compatibility, but currently sequential)
///
/// This executor runs tasks in a sequence that respects their dependencies.
/// While the name suggests parallel execution, the current implementation
/// is sequential but maintained for API compatibility.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::{
///     ParallelExecutionConfig, ParallelExecutor, Task, TaskRunner
/// };
///
/// # fn example<'a>(runner: &'a TaskRunner<'a>, tasks: Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
/// // Create executor with default config
/// let config = ParallelExecutionConfig::default();
/// let executor = ParallelExecutor::new(runner, config);
///
/// // Execute tasks
/// let results = executor.execute(&tasks)?;
/// println!("Executed {} tasks", results.len());
/// # Ok(())
/// # }
/// ```
pub struct ParallelExecutor<'a> {
    task_runner: &'a TaskRunner<'a>,
    config: ParallelExecutionConfig,
}

impl<'a> ParallelExecutor<'a> {
    /// Create a new executor
    ///
    /// # Arguments
    ///
    /// * `task_runner` - The task runner to use for executing individual tasks
    /// * `config` - Configuration for parallel execution
    ///
    /// # Returns
    ///
    /// A new parallel executor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{
    ///     ParallelExecutionConfig, ParallelExecutor, TaskRunner
    /// };
    ///
    /// # fn example<'a>(runner: &'a TaskRunner<'a>) {
    /// // Create config with custom settings
    /// let config = ParallelExecutionConfig {
    ///     max_parallel: 8,
    ///     fail_fast: true,
    ///     show_progress: true,
    /// };
    ///
    /// // Create executor
    /// let executor = ParallelExecutor::new(runner, config);
    /// # }
    /// ```
    pub fn new(task_runner: &'a TaskRunner<'a>, config: ParallelExecutionConfig) -> Self {
        Self { task_runner, config }
    }

    /// Execute tasks in sequence, respecting the dependency graph
    ///
    /// Executes the given tasks in a sequence that respects their dependencies.
    /// Tasks are executed level by level, where each level contains tasks
    /// that have all their dependencies satisfied.
    ///
    /// # Arguments
    ///
    /// * `tasks` - List of tasks to execute
    ///
    /// # Returns
    ///
    /// A list of task execution results.
    ///
    /// # Errors
    ///
    /// Returns an error if task graph construction fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{
    ///     default_parallel_config, ParallelExecutor, Task, TaskRunner
    /// };
    ///
    /// # fn example<'a>(runner: &'a TaskRunner<'a>, tasks: Vec<Task>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Create executor with default config
    /// let config = default_parallel_config();
    /// let executor = ParallelExecutor::new(runner, config);
    ///
    /// // Execute tasks
    /// let results = executor.execute(&tasks)?;
    ///
    /// // Process results
    /// for result in results {
    ///     println!("{}: {}", result.name(), if result.is_success() { "Success" } else { "Failed" });
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute(&self, tasks: &[Task]) -> TaskResult<Vec<TaskResultInfo>> {
        // Build task graph
        let graph = TaskGraph::from_tasks(tasks)?;

        // Get task levels for execution
        let levels = graph.task_levels();

        // Prepare result collection
        let mut results = Vec::new();
        let mut failures = HashSet::new();

        // Execute each level in sequence
        for (level_idx, level) in levels.iter().enumerate() {
            if self.config.show_progress {
                // Report progress
                log::info!(
                    "Executing level {} of {} ({} tasks)",
                    level_idx + 1,
                    levels.len(),
                    level.len()
                );
            }

            // Check if any failures have occurred and we should stop
            if self.config.fail_fast && !failures.is_empty() {
                break;
            }

            // Execute all tasks in this level
            self.execute_level(level, &mut results, &mut failures);
        }

        Ok(results)
    }

    /// Executes all tasks in a single level of the dependency graph.
    ///
    /// Tasks at the same level can in theory be executed in parallel,
    /// but the current implementation runs them sequentially.
    ///
    /// # Arguments
    ///
    /// * `level` - List of tasks at the current level
    /// * `results` - Collection to store results in
    /// * `failures` - Set of failed task names
    fn execute_level(
        &self,
        level: &[Task],
        results: &mut Vec<TaskResultInfo>,
        failures: &mut HashSet<String>,
    ) {
        if !level.is_empty() {
            // Process each task in the level
            for task in level {
                // Skip task if any of its dependencies failed
                let should_skip = task.dependencies.iter().any(|dep| failures.contains(dep));

                if should_skip {
                    // Add a skipped result
                    let execution = TaskExecution {
                        exit_code: 0,
                        stdout: String::new(),
                        stderr: String::from("Skipped because dependency failed"),
                        duration: Duration::from_secs(0),
                        status: TaskStatus::Skipped,
                    };

                    let task_result = TaskResultInfo { task: task.clone(), execution };

                    results.push(task_result);

                    // Mark this task as failed for its dependents
                    failures.insert(task.name.clone());

                    continue;
                }

                let start_time = Instant::now();

                // Execute the task
                match self.task_runner.execute_task(task) {
                    Ok(execution) => {
                        // Create result
                        let task_result =
                            TaskResultInfo { task: task.clone(), execution: execution.clone() };

                        // Store result
                        results.push(task_result);

                        // Check if it failed, and add to failures if so
                        if execution.status != TaskStatus::Success && !task.config.ignore_error {
                            failures.insert(task.name.clone());
                        }
                    }
                    Err(err) => {
                        // Create a failed execution
                        let execution = TaskExecution {
                            exit_code: 1,
                            stdout: String::new(),
                            stderr: format!("Error: {err}"),
                            duration: start_time.elapsed(),
                            status: TaskStatus::Failed,
                        };

                        // Create result
                        let task_result = TaskResultInfo { task: task.clone(), execution };

                        // Store result
                        results.push(task_result);

                        // Mark as failed
                        failures.insert(task.name.clone());
                    }
                }

                // Check if we should stop due to failure
                if self.config.fail_fast && !failures.is_empty() {
                    break;
                }
            }
        }
    }
}

/// Creates a default parallel executor config
///
/// # Returns
///
/// A default configuration for parallel task execution.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{default_parallel_config, ParallelExecutor, TaskRunner};
///
/// # fn example<'a>(runner: &'a TaskRunner<'a>) {
/// let config = default_parallel_config();
/// let executor = ParallelExecutor::new(runner, config);
/// # }
/// ```
pub fn default_parallel_config() -> ParallelExecutionConfig {
    ParallelExecutionConfig::default()
}

/// Creates a parallel config with specific concurrency
///
/// # Arguments
///
/// * `concurrency` - Maximum number of parallel tasks
///
/// # Returns
///
/// A parallel execution configuration with the specified concurrency.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{parallel_config_with_concurrency, ParallelExecutor, TaskRunner};
///
/// # fn example<'a>(runner: &'a TaskRunner<'a>) {
/// // Create config with 4 parallel tasks
/// let config = parallel_config_with_concurrency(4);
/// let executor = ParallelExecutor::new(runner, config);
/// # }
/// ```
pub fn parallel_config_with_concurrency(concurrency: usize) -> ParallelExecutionConfig {
    ParallelExecutionConfig { max_parallel: concurrency, ..ParallelExecutionConfig::default() }
}

/// Creates a fail-fast parallel config
///
/// Creates a configuration that stops execution on the first failure.
///
/// # Returns
///
/// A parallel execution configuration with fail-fast enabled.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{fail_fast_parallel_config, ParallelExecutor, TaskRunner};
///
/// # fn example<'a>(runner: &'a TaskRunner<'a>) {
/// let config = fail_fast_parallel_config();
/// let executor = ParallelExecutor::new(runner, config);
/// # }
/// ```
pub fn fail_fast_parallel_config() -> ParallelExecutionConfig {
    ParallelExecutionConfig { fail_fast: true, ..ParallelExecutionConfig::default() }
}
