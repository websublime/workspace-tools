use std::fmt;
use std::time::Duration;
use thiserror::Error;

use super::task::{Task, TaskExecution, TaskStatus};

/// Result type for task operations
pub type TaskResult<T> = Result<T, TaskError>;

/// Errors that can occur during task operations
#[derive(Error, Debug)]
pub enum TaskError {
    /// Task execution failed
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    /// Task timed out
    #[error("Task timed out after {0:?}")]
    Timeout(Duration),

    /// Task was cancelled
    #[error("Task was cancelled")]
    Cancelled,

    /// Task dependency failed
    #[error("Dependency {0} failed")]
    DependencyFailed(String),

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Filter error
    #[error("Invalid task filter: {0}")]
    FilterError(String),

    /// Task graph error
    #[error("Task graph error: {0}")]
    GraphError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Command execution error
    #[error("Command error: {0}")]
    CommandError(String),

    /// Standard tools error
    #[error("Standard tools error: {0}")]
    StandardToolsError(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result of executing a specific task with full context
#[derive(Debug, Clone)]
pub struct TaskResultInfo {
    /// The task that was executed
    pub task: Task,
    /// The execution details
    pub execution: TaskExecution,
}

impl TaskResultInfo {
    /// Create a new task result
    pub fn new(task: Task, execution: TaskExecution) -> Self {
        Self { task, execution }
    }

    /// Check if the task succeeded
    pub fn is_success(&self) -> bool {
        self.execution.status == TaskStatus::Success
    }

    /// Check if the task failed
    pub fn is_failure(&self) -> bool {
        match self.execution.status {
            TaskStatus::Failed | TaskStatus::Timeout | TaskStatus::Cancelled => true,
            _ => false,
        }
    }

    /// Get the duration of the task
    pub fn duration(&self) -> Duration {
        self.execution.duration
    }

    /// Get the exit code
    pub fn exit_code(&self) -> i32 {
        self.execution.exit_code
    }

    /// Get the task name
    pub fn name(&self) -> &str {
        &self.task.name
    }
}

impl fmt::Display for TaskResultInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task {} (status: {:?}, exit code: {}, duration: {:?})",
            self.task.name, self.execution.status, self.execution.exit_code, self.execution.duration
        )
    }
} 