use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use sublime_standard_tools::execute;

use crate::workspace::workspace::Workspace;

use super::error::{TaskError, TaskResult};
use super::filter::TaskFilter;
use super::graph::{TaskGraph, TaskSortMode};
use super::parallel::{ParallelExecutionConfig, ParallelExecutor};
use super::task::{Task, TaskExecution, TaskStatus};

/// Task runner for executing tasks in a workspace
pub struct TaskRunner<'a> {
    /// Reference to the workspace
    workspace: &'a Workspace,
    
    /// Stored tasks
    tasks: Vec<Task>,
}

impl<'a> Clone for TaskRunner<'a> {
    fn clone(&self) -> Self {
        Self {
            workspace: self.workspace,
            tasks: self.tasks.clone(),
        }
    }
}

impl<'a> TaskRunner<'a> {
    /// Create a new task runner for a workspace
    pub fn new(workspace: &'a Workspace) -> Self {
        Self {
            workspace,
            tasks: Vec::new(),
        }
    }
    
    /// Add a task to the runner
    pub fn add_task(&mut self, task: Task) -> &mut Self {
        self.tasks.push(task);
        self
    }
    
    /// Add multiple tasks to the runner
    pub fn add_tasks(&mut self, tasks: Vec<Task>) -> &mut Self {
        for task in tasks {
            self.tasks.push(task);
        }
        self
    }
    
    /// Get all registered tasks
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }
    
    /// Get a task by name
    pub fn get_task(&self, name: &str) -> Option<Task> {
        for task in &self.tasks {
            if task.name == name {
                return Some(task.clone());
            }
        }
        None
    }
    
    /// Load tasks from a configuration file
    pub fn load_tasks_from_config(&mut self, path: &Path) -> TaskResult<&mut Self> {
        // Read the config file
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| TaskError::IoError(e))?;
            
        // Parse JSON or YAML depending on file extension
        let tasks: Vec<Task> = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&config_str)
                .map_err(|e| TaskError::Other(format!("Failed to parse JSON config: {}", e)))?
        } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            serde_yaml::from_str(&config_str)
                .map_err(|e| TaskError::Other(format!("Failed to parse YAML config: {}", e)))?
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
    pub fn run_task(&self, task_name: &str) -> TaskResult<super::error::TaskResultInfo> {
        let task = self.get_task(task_name)
            .ok_or_else(|| TaskError::TaskNotFound(task_name.to_string()))?;
            
        let execution = self.execute_task(&task)?;
        
        Ok(super::error::TaskResultInfo {
            task,
            execution,
        })
    }
    
    /// Run multiple tasks
    pub fn run_tasks(&self, task_names: &[&str]) -> TaskResult<Vec<super::error::TaskResultInfo>> {
        // Get all the tasks
        let mut tasks = Vec::new();
        for name in task_names {
            let task = self.get_task(name)
                .ok_or_else(|| TaskError::TaskNotFound(name.to_string()))?;
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
    pub fn run_filtered(&self, filter: TaskFilter) -> TaskResult<Vec<super::error::TaskResultInfo>> {
        // Apply the filter
        let filtered_tasks = filter.apply(&self.tasks)?;
        
        if filtered_tasks.len() == 0 {
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
    pub fn build_task_graph(&self) -> TaskResult<TaskGraph> {
        TaskGraph::from_tasks(&self.tasks)
    }
    
    /// Execute a task
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
                    "Package not found for task {}: {}", task.name, package
                )));
            }
        } else {
            // Use the workspace root
            self.workspace.root_path.clone()
        };
        
        // Check for timeout
        let timeout = task.config.timeout.unwrap_or(Duration::from_secs(3600)); // Default 1 hour
        
        // Execute the command
        let result = self.execute_command(&task.command, &cwd, &task.config.env, timeout);
        
        match result {
            Ok((stdout, stderr, exit_code)) => {
                let status = if exit_code == 0 {
                    TaskStatus::Success
                } else {
                    TaskStatus::Failed
                };
                
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
                    stderr: format!("Error: {}", err),
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
    
    // Helper method to execute a command
    fn execute_command(
        &self,
        cmd: &str,
        cwd: &Path,
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<(String, String, i32), TaskError> {
        // Split the command into program and args
        let mut parts = cmd.split(|c: char| c.is_whitespace());
        let program = parts.next().ok_or_else(|| {
            TaskError::CommandError("Empty command".to_string())
        })?;
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
                self.manual_execute_command(cmd, cwd, env, timeout)
            }
        }
    }
    
    // Fallback execution method
    fn manual_execute_command(
        &self,
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
        let mut child = command
            .spawn()
            .map_err(|e| TaskError::IoError(e))?;
        
        // Read stdout and stderr
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");
        
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);
        
        let stdout_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stdout_reader.lines() {
                if let Ok(line) = line {
                    lines.push(line);
                }
            }
            lines
        });
        
        let stderr_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            for line in stderr_reader.lines() {
                if let Ok(line) = line {
                    lines.push(line);
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
                    
                    return Ok((
                        stdout_str,
                        stderr_str,
                        status.code().unwrap_or(1),
                    ));
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