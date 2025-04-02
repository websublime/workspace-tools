use std::time::Duration;
use thiserror::Error;

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

impl AsRef<str> for TaskError {
    fn as_ref(&self) -> &str {
        match self {
            TaskError::FilterError(_) => "FilterError",
            TaskError::GraphError(_) => "GraphError",
            TaskError::IoError(_) => "IoError",
            TaskError::CommandError(_) => "CommandError",
            TaskError::StandardToolsError(_) => "StandardToolsError",
            TaskError::Other(_) => "Other",
            TaskError::TaskNotFound(_) => "TaskNotFound",
            TaskError::ExecutionFailed(_) => "ExecutionFailed",
            TaskError::Timeout(_) => "Timeout",
            TaskError::Cancelled => "Cancelled",
            TaskError::DependencyFailed(_) => "DependencyFailed",
            TaskError::CircularDependency(_) => "CircularDependency",
        }
    }
}
