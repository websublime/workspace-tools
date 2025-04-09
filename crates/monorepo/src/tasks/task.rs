//! Task definition and execution models.
//!
//! This module provides the core data structures for defining tasks and representing
//! their execution results. Tasks encapsulate commands to be executed within the
//! monorepo, along with their dependencies, configuration, and package context.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Status of a task
///
/// Represents the current state or final result of a task execution.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::TaskStatus;
///
/// let status = TaskStatus::Success;
/// assert!(status == TaskStatus::Success);
///
/// let status = TaskStatus::Failed;
/// assert!(status != TaskStatus::Success);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task has not started
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Success,
    /// Task failed
    Failed,
    /// Task was skipped
    Skipped,
    /// Task timed out
    Timeout,
    /// Task was cancelled
    Cancelled,
}

/// Configuration for a task
///
/// Contains settings that control how a task is executed, including
/// working directory, environment variables, timeouts, and error handling.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use std::time::Duration;
/// use sublime_monorepo_tools::TaskConfig;
///
/// // Default configuration
/// let default_config = TaskConfig::default();
///
/// // Custom configuration
/// let mut config = TaskConfig::default();
/// config.cwd = Some(PathBuf::from("/path/to/dir"));
/// config.timeout = Some(Duration::from_secs(30));
/// config.ignore_error = true;
/// config.env.insert("NODE_ENV".to_string(), "production".to_string());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskConfig {
    /// Working directory for the task
    pub cwd: Option<PathBuf>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Timeout for the task
    pub timeout: Option<Duration>,
    /// Whether to ignore errors
    pub ignore_error: bool,
    /// Whether to log output to console in real-time
    pub live_output: bool,
}

impl Default for TaskConfig {
    /// Creates a default configuration.
    ///
    /// Default settings:
    /// - cwd: None (uses workspace or package directory)
    /// - env: Empty HashMap
    /// - timeout: None (no timeout)
    /// - ignore_error: false (fail on error)
    /// - live_output: true (show output in real-time)
    ///
    /// # Returns
    ///
    /// A default task configuration.
    fn default() -> Self {
        Self {
            cwd: None,
            env: HashMap::new(),
            timeout: None,
            ignore_error: false,
            live_output: true,
        }
    }
}

/// Results from task execution
///
/// Contains the output, status, and timing information from a task execution.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use sublime_monorepo_tools::{TaskExecution, TaskStatus};
///
/// let execution = TaskExecution {
///     exit_code: 0,
///     stdout: "Task completed successfully".to_string(),
///     stderr: "".to_string(),
///     duration: Duration::from_secs(2),
///     status: TaskStatus::Success,
/// };
///
/// assert_eq!(execution.status, TaskStatus::Success);
/// assert_eq!(execution.exit_code, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskExecution {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
    /// Final task status
    pub status: TaskStatus,
}

/// Definition of a task to be executed
///
/// Represents a command to be executed within the monorepo context,
/// with associated metadata like dependencies and configuration.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use sublime_monorepo_tools::Task;
///
/// // Create a basic task
/// let simple_task = Task::new("lint", "npm run lint");
///
/// // Create a task with dependencies and configuration
/// let complex_task = Task::new("test", "npm test")
///     .with_package("ui-components")
///     .with_dependency("build")
///     .with_timeout(Duration::from_secs(60))
///     .ignore_error(true);
/// ``
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Task name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Package context (optional)
    pub package: Option<String>,
    /// Task dependencies (task names)
    pub dependencies: Vec<String>,
    /// Task configuration
    pub config: TaskConfig,
}

impl Task {
    /// Create a new task with default configuration
    ///
    /// # Arguments
    ///
    /// * `name` - Unique name for the task
    /// * `command` - Command to execute
    ///
    /// # Returns
    ///
    /// A new task with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a simple build task
    /// let build = Task::new("build", "npm run build");
    ///
    /// // Create a test task
    /// let test = Task::new("test", "npm test");
    /// ```
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            package: None,
            dependencies: Vec::new(),
            config: TaskConfig::default(),
        }
    }

    /// Set the package for this task
    ///
    /// Associates the task with a specific package in the monorepo.
    /// This affects the default working directory for the task.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a task for the UI package
    /// let ui_build = Task::new("build:ui", "npm run build")
    ///     .with_package("ui-components");
    /// ```
    #[must_use]
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    /// Add a dependency on another task
    ///
    /// Adds a dependency that must be executed before this task.
    ///
    /// # Arguments
    ///
    /// * `dependency` - Name of the dependency task
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a task that depends on the build task
    /// let test = Task::new("test", "npm test")
    ///     .with_dependency("build");
    /// ```
    #[must_use]
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies
    ///
    /// Adds multiple dependencies that must be executed before this task.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Collection of dependency task names
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a task with multiple dependencies
    /// let deploy = Task::new("deploy", "npm run deploy")
    ///     .with_dependencies(vec!["build", "test"]);
    /// ```
    #[must_use]
    pub fn with_dependencies(mut self, dependencies: Vec<impl Into<String>>) -> Self {
        for dep in dependencies {
            self.dependencies.push(dep.into());
        }
        self
    }

    /// Set working directory
    ///
    /// Sets the directory where the task command will be executed.
    ///
    /// # Arguments
    ///
    /// * `cwd` - Working directory path
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Set a specific working directory
    /// let task = Task::new("build", "npm run build")
    ///     .with_cwd("/path/to/directory");
    /// ```
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.config.cwd = Some(cwd.into());
        self
    }

    /// Add environment variable
    ///
    /// Adds an environment variable for the task execution.
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Set environment variables
    /// let task = Task::new("build", "npm run build")
    ///     .with_env("NODE_ENV", "production")
    ///     .with_env("DEBUG", "false");
    /// ```
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }

    /// Set timeout
    ///
    /// Sets a maximum execution time for the task.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum execution duration
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Set a 5-minute timeout
    /// let task = Task::new("build", "npm run build")
    ///     .with_timeout(Duration::from_secs(300));
    /// ```
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    /// Set whether to ignore errors
    ///
    /// When true, the task runner will continue executing subsequent tasks
    /// even if this task fails.
    ///
    /// # Arguments
    ///
    /// * `ignore` - Whether to ignore errors
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a task that won't block the pipeline if it fails
    /// let lint = Task::new("lint", "npm run lint")
    ///     .ignore_error(true);
    /// ```
    #[must_use]
    pub fn ignore_error(mut self, ignore: bool) -> Self {
        self.config.ignore_error = ignore;
        self
    }

    /// Set whether to show live output
    ///
    /// Controls whether the task's output is displayed in real-time
    /// during execution.
    ///
    /// # Arguments
    ///
    /// * `live` - Whether to show live output
    ///
    /// # Returns
    ///
    /// The modified task.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::Task;
    ///
    /// // Create a task that doesn't show output until completion
    /// let task = Task::new("build", "npm run build")
    ///     .live_output(false);
    /// ```
    #[must_use]
    pub fn live_output(mut self, live: bool) -> Self {
        self.config.live_output = live;
        self
    }
}
