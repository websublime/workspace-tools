use std::collections::HashSet;
use std::time::{Duration, Instant};

use super::error::{TaskResult, TaskResultInfo};
use super::graph::TaskGraph;
use super::task::{Task, TaskExecution, TaskStatus};
use super::runner::TaskRunner;

/// Configuration for parallel task execution
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
    fn default() -> Self {
        Self {
            max_parallel: num_cpus::get(),
            fail_fast: false,
            show_progress: true,
        }
    }
}

/// Execution engine for tasks (named "Parallel" for API compatibility, but currently sequential)
pub struct ParallelExecutor<'a> {
    task_runner: &'a TaskRunner<'a>,
    config: ParallelExecutionConfig,
}

impl<'a> ParallelExecutor<'a> {
    /// Create a new executor
    pub fn new(task_runner: &'a TaskRunner<'a>, config: ParallelExecutionConfig) -> Self {
        Self { task_runner, config }
    }
    
    /// Execute tasks in sequence, respecting the dependency graph
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
            self.execute_level(&level, &mut results, &mut failures)?;
        }
        
        Ok(results)
    }
    
    // Execute all tasks in a level
    fn execute_level(
        &self,
        level: &[Task],
        results: &mut Vec<TaskResultInfo>,
        failures: &mut HashSet<String>,
    ) -> TaskResult<()> {
        if level.is_empty() {
            return Ok(());
        }
        
        // Process each task in the level
        for task in level {
            // Skip task if any of its dependencies failed
            let should_skip = task.dependencies.iter().any(|dep| failures.contains(dep));
            
            if should_skip {
                // Add a skipped result
                let execution = TaskExecution {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: format!("Skipped because dependency failed"),
                    duration: Duration::from_secs(0),
                    status: TaskStatus::Skipped,
                };
                
                let task_result = TaskResultInfo {
                    task: task.clone(),
                    execution,
                };
                
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
                    let task_result = TaskResultInfo {
                        task: task.clone(),
                        execution: execution.clone(),
                    };
                    
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
                        stderr: format!("Error: {}", err),
                        duration: start_time.elapsed(),
                        status: TaskStatus::Failed,
                    };
                    
                    // Create result
                    let task_result = TaskResultInfo {
                        task: task.clone(),
                        execution,
                    };
                    
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
        
        Ok(())
    }
}

/// Creates a default parallel executor config
pub fn default_parallel_config() -> ParallelExecutionConfig {
    ParallelExecutionConfig::default()
}

/// Creates a parallel config with specific concurrency
pub fn parallel_config_with_concurrency(concurrency: usize) -> ParallelExecutionConfig {
    ParallelExecutionConfig {
        max_parallel: concurrency,
        ..ParallelExecutionConfig::default()
    }
}

/// Creates a fail-fast parallel config
pub fn fail_fast_parallel_config() -> ParallelExecutionConfig {
    ParallelExecutionConfig {
        fail_fast: true,
        ..ParallelExecutionConfig::default()
    }
} 