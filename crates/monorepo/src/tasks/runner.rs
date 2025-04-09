//! Task execution engine for running tasks in a workspace.
//!
//! This module provides the primary interface for executing tasks in a monorepo
//! workspace. It handles task discovery, dependency resolution, and execution,
//! either individually or in batches with proper dependency ordering.

use super::error::{TaskError, TaskResult};
use super::filter::TaskFilter;
use super::graph::{TaskGraph, TaskSortMode};
use super::parallel::{ParallelExecutionConfig, ParallelExecutor};
use super::task::{Task, TaskExecution, TaskStatus};
use crate::workspace::workspace::Workspace;
use crate::TaskResultInfo;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use sublime_standard_tools::execute;

/// Task runner for executing tasks in a workspace
///
/// The `TaskRunner` is the main entry point for executing tasks within a workspace.
/// It manages task registration, discovery, dependency resolution, and execution.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use sublime_monorepo_tools::{Task, TaskFilter, TaskRunner, Workspace};
///
/// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
/// // Create a task runner
/// let mut runner = TaskRunner::new(workspace);
///
/// // Add some tasks
/// runner.add_task(Task::new("build", "npm run build"));
/// runner.add_task(Task::new("test", "npm test").with_dependency("build"));
///
/// // Run a single task
/// let result = runner.run_task("test")?;
/// println!("Task {} completed with status: {:?}", result.name(), result.execution.status);
///
/// // Run multiple tasks
/// let results = runner.run_tasks(&["build", "test"])?;
///
/// // Run tasks matching a filter
/// let filter = TaskFilter::new().with_include(vec!["test*"]);
/// let test_results = runner.run_filtered(&filter)?;
/// # Ok(())
/// # }
/// ```
pub struct TaskRunner<'a> {
    /// Reference to the workspace
    workspace: &'a Workspace,

    /// Stored tasks
    tasks: Vec<Task>,
}

impl<'a> Clone for TaskRunner<'a> {
    /// Clones the task runner.
    ///
    /// # Returns
    ///
    /// A clone of the task runner.
    fn clone(&self) -> Self {
        Self { workspace: self.workspace, tasks: self.tasks.clone() }
    }
}

impl<'a> TaskRunner<'a> {
    /// Create a new task runner for a workspace
    ///
    /// # Arguments
    ///
    /// * `workspace` - Reference to the workspace
    ///
    /// # Returns
    ///
    /// A new task runner.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) {
    /// let runner = TaskRunner::new(workspace);
    /// # }
    /// ```
    pub fn new(workspace: &'a Workspace) -> Self {
        Self { workspace, tasks: Vec::new() }
    }

    /// Add a task to the runner
    ///
    /// # Arguments
    ///
    /// * `task` - Task to add
    ///
    /// # Returns
    ///
    /// Reference to the task runner for method chaining.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) {
    /// let mut runner = TaskRunner::new(workspace);
    ///
    /// // Add a simple task
    /// runner.add_task(Task::new("build", "npm run build"));
    ///
    /// // Add a task with dependencies
    /// runner.add_task(
    ///     Task::new("test", "npm test")
    ///         .with_dependency("build")
    ///         .with_package("ui")
    /// );
    /// # }
    /// ```
    pub fn add_task(&mut self, task: Task) -> &mut Self {
        self.tasks.push(task);
        self
    }

    /// Add multiple tasks to the runner
    ///
    /// # Arguments
    ///
    /// * `tasks` - Tasks to add
    ///
    /// # Returns
    ///
    /// Reference to the task runner for method chaining.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) {
    /// let mut runner = TaskRunner::new(workspace);
    ///
    /// // Create some tasks
    /// let build = Task::new("build", "npm run build");
    /// let test = Task::new("test", "npm test").with_dependency("build");
    ///
    /// // Add multiple tasks
    /// runner.add_tasks(vec![build, test]);
    /// # }
    /// ```
    pub fn add_tasks(&mut self, tasks: Vec<Task>) -> &mut Self {
        for task in tasks {
            self.tasks.push(task);
        }
        self
    }

    /// Get all registered tasks
    ///
    /// # Returns
    ///
    /// A slice of all registered tasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build", "npm run build"));
    ///
    /// // Get all tasks
    /// let tasks = runner.get_tasks();
    /// println!("Runner has {} tasks", tasks.len());
    /// # }
    /// ```
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get a task by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the task to retrieve
    ///
    /// # Returns
    ///
    /// The task if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build", "npm run build"));
    ///
    /// // Look up a task
    /// if let Some(task) = runner.get_task("build") {
    ///     println!("Found task: {}", task.name);
    /// } else {
    ///     println!("Task not found");
    /// }
    /// # }
    ///
    /// ```
    pub fn get_task(&self, name: &str) -> Option<Task> {
        for task in &self.tasks {
            if task.name == name {
                return Some(task.clone());
            }
        }
        None
    }

    /// Load tasks from a configuration file
    ///
    /// Loads tasks from a JSON or YAML configuration file. The file format
    /// is determined by the file extension (.json or .yaml/.yml).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// Reference to the task runner for method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use sublime_monorepo_tools::{TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runner = TaskRunner::new(workspace);
    ///
    /// // Load tasks from a JSON file
    /// runner.load_tasks_from_config(Path::new("tasks.json"))?;
    ///
    /// // Load tasks from a YAML file
    /// runner.load_tasks_from_config(Path::new("tasks.yaml"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_tasks_from_config(&mut self, path: &Path) -> TaskResult<&mut Self> {
        // Read the config file
        let config_str = std::fs::read_to_string(path).map_err(TaskError::IoError)?;

        // Parse JSON or YAML depending on file extension
        let tasks: Vec<Task> = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&config_str)
                .map_err(|e| TaskError::Other(format!("Failed to parse JSON config: {e}")))?
        } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            serde_yaml::from_str(&config_str)
                .map_err(|e| TaskError::Other(format!("Failed to parse YAML config: {e}")))?
        } else {
            return Err(TaskError::Other(format!(
                "Unsupported config file format: {:?}",
                path.extension().unwrap_or_default()
            )));
        };

        // Add the tasks
        self.add_tasks(tasks);

        Ok(self)
    }

    /// Execute a single task
    ///
    /// Runs a task by name, including its dependencies.
    ///
    /// # Arguments
    ///
    /// * `task_name` - Name of the task to run
    ///
    /// # Returns
    ///
    /// The result of the task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the task is not found or execution fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build", "npm run build"));
    ///
    /// // Run the task
    /// let result = runner.run_task("build")?;
    /// println!("Task exited with code: {}", result.exit_code());
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_task(&self, task_name: &str) -> TaskResult<TaskResultInfo> {
        let task = self
            .get_task(task_name)
            .ok_or_else(|| TaskError::TaskNotFound(task_name.to_string()))?;

        let execution = self.execute_task(&task)?;

        Ok(TaskResultInfo { task, execution })
    }

    /// Run multiple tasks
    ///
    /// Runs multiple tasks with proper dependency resolution, ensuring dependencies
    /// are executed before the tasks that depend on them.
    ///
    /// # Arguments
    ///
    /// * `task_names` - Names of the tasks to run
    ///
    /// # Returns
    ///
    /// Results of the task executions.
    ///
    /// # Errors
    ///
    /// Returns an error if any task is not found, or if there are dependency issues.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build", "npm run build"));
    /// runner.add_task(Task::new("test", "npm test").with_dependency("build"));
    ///
    /// // Run multiple tasks
    /// let results = runner.run_tasks(&["build", "test"])?;
    /// println!("Executed {} tasks", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_tasks(&self, task_names: &[&str]) -> TaskResult<Vec<TaskResultInfo>> {
        // Get all the tasks
        let mut tasks = Vec::new();
        for name in task_names {
            let task =
                self.get_task(name).ok_or_else(|| TaskError::TaskNotFound((*name).to_string()))?;
            tasks.push(task);
        }

        // Build the task graph to get dependencies
        let graph = TaskGraph::from_tasks(&tasks)?;

        // Get sorted tasks (dependencies first)
        let sorted_tasks = graph.sorted_tasks(TaskSortMode::Topological)?;

        // Execute the tasks
        let config = ParallelExecutionConfig::default();
        let executor = ParallelExecutor::new(self, config);
        executor.execute(&sorted_tasks)
    }

    /// Run tasks matching a filter
    ///
    /// Runs tasks that match the specified filter criteria.
    ///
    /// # Arguments
    ///
    /// * `filter` - Filter to select tasks
    ///
    /// # Returns
    ///
    /// Results of the task executions.
    ///
    /// # Errors
    ///
    /// Returns an error if there are dependency issues or execution fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskFilter, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build:ui", "npm run build").with_package("ui"));
    /// runner.add_task(Task::new("test:ui", "npm test").with_package("ui"));
    /// runner.add_task(Task::new("build:api", "npm run build").with_package("api"));
    ///
    /// // Create a filter for UI tasks
    /// let filter = TaskFilter::new()
    ///     .with_packages(vec!["ui"])
    ///     .with_include(vec!["build*"]);
    ///
    /// // Run filtered tasks
    /// let results = runner.run_filtered(&filter)?;
    /// println!("Ran {} tasks", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_filtered(&self, filter: &TaskFilter) -> TaskResult<Vec<TaskResultInfo>> {
        // Apply the filter
        let filtered_tasks = filter.apply(&self.tasks)?;

        if filtered_tasks.is_empty() {
            return Ok(Vec::new());
        }

        // Build the task graph
        let graph = TaskGraph::from_tasks(&filtered_tasks)?;

        // Get sorted tasks (dependencies first)
        let sorted_tasks = graph.sorted_tasks(TaskSortMode::Topological)?;

        // Execute the tasks
        let config = ParallelExecutionConfig::default();
        let executor = ParallelExecutor::new(self, config);
        executor.execute(&sorted_tasks)
    }

    /// Build task graph for visualization
    ///
    /// Creates a graph representation of task dependencies, which can be used
    /// for visualization or analysis.
    ///
    /// # Returns
    ///
    /// A task dependency graph.
    ///
    /// # Errors
    ///
    /// Returns an error if there are circular dependencies or missing task references.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut runner = TaskRunner::new(workspace);
    /// runner.add_task(Task::new("build", "npm run build"));
    /// runner.add_task(Task::new("test", "npm test").with_dependency("build"));
    ///
    /// // Build the task graph
    /// let graph = runner.build_task_graph()?;
    ///
    /// // Examine the graph
    /// println!("Graph contains {} tasks", graph.task_count());
    /// println!("Task 'test' depends on {} tasks", graph.dependencies_of("test")?.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_task_graph(&self) -> TaskResult<TaskGraph> {
        TaskGraph::from_tasks(&self.tasks)
    }

    /// Execute a task
    ///
    /// Executes a single task with its specified configuration.
    ///
    /// # Arguments
    ///
    /// * `task` - Task to execute
    ///
    /// # Returns
    ///
    /// Result of the task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails or times out.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_monorepo_tools::{Task, TaskRunner, Workspace};
    ///
    /// # fn example(workspace: &Workspace, runner: &TaskRunner) -> Result<(), Box<dyn std::error::Error>> {
    /// let task = Task::new("build", "npm run build");
    /// let execution = runner.execute_task(&task)?;
    /// println!("Exit code: {}", execution.exit_code);
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_task(&self, task: &Task) -> TaskResult<TaskExecution> {
        let start_time = Instant::now();

        // Determine the working directory
        let cwd = if let Some(cwd) = &task.config.cwd {
            cwd.clone()
        } else if let Some(package) = &task.package {
            // Use the package directory
            if let Some(pkg_info) = self.workspace.get_package(package) {
                PathBuf::from(&pkg_info.borrow().package_path)
            } else {
                return Err(TaskError::TaskNotFound(format!(
                    "Package not found for task {}: {}",
                    task.name, package
                )));
            }
        } else {
            // Use the workspace root
            self.workspace.root_path().to_path_buf()
        };

        // Check for timeout
        let timeout = task.config.timeout.unwrap_or(Duration::from_secs(3600)); // Default 1 hour

        // Execute the command
        let result = TaskRunner::execute_command(&task.command, &cwd, &task.config.env, timeout);

        match result {
            Ok((stdout, stderr, exit_code)) => {
                let status = if exit_code == 0 { TaskStatus::Success } else { TaskStatus::Failed };

                let execution = TaskExecution {
                    exit_code,
                    stdout,
                    stderr,
                    duration: start_time.elapsed(),
                    status,
                };

                Ok(execution)
            }
            Err(err) => {
                let status = if err.to_string().contains("timed out") {
                    TaskStatus::Timeout
                } else {
                    TaskStatus::Failed
                };

                let execution = TaskExecution {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!("Error: {err}"),
                    duration: start_time.elapsed(),
                    status,
                };

                // Only return an error for timeout, otherwise return the execution
                if status == TaskStatus::Timeout {
                    Err(TaskError::Timeout(timeout))
                } else {
                    Ok(execution)
                }
            }
        }
    }

    /// Executes a command string with environment variables and timeout.
    ///
    /// This method first attempts to use the `sublime_standard_tools::execute`
    /// function, falling back to a manual implementation if that fails.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command string to execute
    /// * `cwd` - Working directory
    /// * `env` - Environment variables
    /// * `timeout` - Maximum execution time
    ///
    /// # Returns
    ///
    /// Tuple of (stdout, stderr, exit_code) on success.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails or times out.
    fn execute_command(
        cmd: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<(String, String, i32), TaskError> {
        // Split the command into program and args
        let mut parts = cmd.split(|c: char| c.is_whitespace());
        let program =
            parts.next().ok_or_else(|| TaskError::CommandError("Empty command".to_string()))?;
        let args: Vec<&str> = parts.collect();

        // Try using sublime_standard_tools execute first
        let result = execute(program, cwd, args, |stdout, output| {
            Ok((
                stdout.to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
                output.status.code().unwrap_or(1),
            ))
        });

        // Handle the result
        match result {
            Ok(result) => Ok(result),
            Err(_err) => {
                // Fall back to manually running the command
                TaskRunner::manual_execute_command(cmd, cwd, env, timeout)
            }
        }
    }

    /// Manual command execution implementation.
    ///
    /// This is a fallback method for executing commands when the standard
    /// tools execution fails.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command string to execute
    /// * `cwd` - Working directory
    /// * `env` - Environment variables
    /// * `timeout` - Maximum execution time
    ///
    /// # Returns
    ///
    /// Tuple of (stdout, stderr, exit_code) on success.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails or times out.
    fn manual_execute_command(
        cmd: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<(String, String, i32), TaskError> {
        // Create the command
        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", cmd]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", cmd]);
            c
        };

        // Set working directory and environment
        command.current_dir(cwd);
        for (key, value) in env {
            command.env(key, value);
        }

        // Set up pipes
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Start the command
        let mut child = command.spawn().map_err(TaskError::IoError)?;

        // Read stdout and stderr
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let stdout_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stdout_reader.lines() {
                match line {
                    Ok(line) => {
                        lines.push(line);
                    }
                    _ => (),
                }
            }
            lines
        });

        let stderr_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stderr_reader.lines() {
                match line {
                    Ok(line) => {
                        lines.push(line);
                    }
                    _ => (),
                }
            }
            lines
        });

        // Wait for command to complete with timeout
        let start_time = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Command completed
                    let stdout_lines = stdout_handle.join().unwrap_or_default();
                    let stderr_lines = stderr_handle.join().unwrap_or_default();

                    // Join stdout and stderr
                    let mut stdout_str = String::new();
                    for line in &stdout_lines {
                        stdout_str.push_str(line);
                        stdout_str.push('\n');
                    }

                    let mut stderr_str = String::new();
                    for line in &stderr_lines {
                        stderr_str.push_str(line);
                        stderr_str.push('\n');
                    }

                    return Ok((stdout_str, stderr_str, status.code().unwrap_or(1)));
                }
                Ok(None) => {
                    // Command still running, check timeout
                    if start_time.elapsed() >= timeout {
                        // Kill the process
                        let _ = child.kill();
                        return Err(TaskError::Timeout(timeout));
                    }

                    // Sleep a bit before checking again
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(TaskError::IoError(e));
                }
            }
        }
    }
}
