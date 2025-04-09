//! Error types for task operations.
//!
//! This module defines the error types and result type aliases used throughout the task system.
//! It provides structured error reporting for various failure modes that can occur during task
//! execution, dependency resolution, and graph construction.
//!
//! # Examples
//!
//! ```
//! use sublime_monorepo_tools::{TaskError, TaskResult};
//!
//! fn example_operation() -> TaskResult<()> {
//!     // Operation that might fail
//!     Err(TaskError::TaskNotFound("build".to_string()))
//! }
//!
//! let result = example_operation();
//! assert!(result.is_err());
//! ```

use std::time::Duration;
use thiserror::Error;

/// Result type for task operations
pub type TaskResult<T> = Result<T, TaskError>;

/// Errors that can occur during task operations
///
/// This enum represents all possible errors that can occur when working with tasks
/// in the monorepo, including execution failures, timeouts, dependency issues, and more.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::TaskError;
///
/// // Create a task not found error
/// let error = TaskError::TaskNotFound("build".to_string());
///
/// // Handle different error types
/// match error {
///     TaskError::TaskNotFound(name) => println!("Task {} not found", name),
///     TaskError::Timeout(duration) => println!("Task timed out after {:?}", duration),
///     TaskError::ExecutionFailed(reason) => println!("Task failed: {}", reason),
///     _ => println!("Other error occurred"),
/// }
/// ```
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
    /// Gets a string representation of the error type.
    ///
    /// This method provides a simple way to get the error variant name without
    /// the associated values, useful for categorizing errors.
    ///
    /// # Returns
    ///
    /// A string slice representing the error type.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_monorepo_tools::TaskError;
    ///
    /// let error = TaskError::Cancelled;
    /// assert_eq!(error.as_ref(), "Cancelled");
    /// ``
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
