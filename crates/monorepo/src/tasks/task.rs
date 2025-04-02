use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Status of a task
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
    #[must_use]
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    /// Add a dependency on another task
    #[must_use]
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Add multiple dependencies
    #[must_use]
    pub fn with_dependencies(mut self, dependencies: Vec<impl Into<String>>) -> Self {
        for dep in dependencies {
            self.dependencies.push(dep.into());
        }
        self
    }

    /// Set working directory
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.config.cwd = Some(cwd.into());
        self
    }

    /// Add environment variable
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }

    /// Set whether to ignore errors
    #[must_use]
    pub fn ignore_error(mut self, ignore: bool) -> Self {
        self.config.ignore_error = ignore;
        self
    }

    /// Set whether to show live output
    #[must_use]
    pub fn live_output(mut self, live: bool) -> Self {
        self.config.live_output = live;
        self
    }
}
