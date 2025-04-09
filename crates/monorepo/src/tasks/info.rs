//! Task result information.
//!
//! This module provides the `TaskResultInfo` type, which encapsulates the result
//! of executing a task, including status, output, and timing information.
//! It's used for reporting and analyzing task execution results.

use super::task::{Task, TaskExecution, TaskStatus};
use std::{fmt, time::Duration};

/// Result of executing a specific task with full context
///
/// This struct combines a task definition with its execution results,
/// providing a complete picture of a task execution.
///
/// # Examples
///
/// ```
/// use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
/// use std::time::Duration;
///
/// // Create a task and its execution result
/// let task = Task::new("build", "npm run build");
/// let execution = TaskExecution {
///     exit_code: 0,
///     stdout: "Build successful".to_string(),
///     stderr: "".to_string(),
///     duration: Duration::from_secs(5),
///     status: TaskStatus::Success,
/// };
///
/// // Create task result info
/// let result = TaskResultInfo::new(task, execution);
///
/// // Check if successful
/// assert!(result.is_success());
///
/// // Get duration
/// assert_eq!(result.duration(), Duration::from_secs(5));
/// ```
#[derive(Debug, Clone)]
pub struct TaskResultInfo {
    /// The task that was executed
    pub task: Task,
    /// The execution details
    pub execution: TaskExecution,
}

impl TaskResultInfo {
    /// Create a new task result
    ///
    /// # Arguments
    ///
    /// * `task` - The task that was executed
    /// * `execution` - The execution details
    ///
    /// # Returns
    ///
    /// A new `TaskResultInfo` instance.
    pub fn new(task: Task, execution: TaskExecution) -> Self {
        Self { task, execution }
    }

    /// Check if the task succeeded
    ///
    /// # Returns
    ///
    /// `true` if the task succeeded, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
    /// # use std::time::Duration;
    /// #
    /// # let task = Task::new("test", "npm test");
    /// # let execution = TaskExecution {
    /// #     exit_code: 0,
    /// #     stdout: "All tests passed".to_string(),
    /// #     stderr: "".to_string(),
    /// #     duration: Duration::from_secs(2),
    /// #     status: TaskStatus::Success,
    /// # };
    /// #
    /// let result = TaskResultInfo::new(task, execution);
    ///
    /// if result.is_success() {
    ///     println!("Task completed successfully");
    /// } else {
    ///     println!("Task failed");
    /// }
    /// ```
    pub fn is_success(&self) -> bool {
        self.execution.status == TaskStatus::Success
    }

    /// Check if the task failed
    ///
    /// Failure includes tasks that failed, timed out, or were cancelled.
    ///
    /// # Returns
    ///
    /// `true` if the task failed, timed out, or was cancelled, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
    /// # use std::time::Duration;
    /// #
    /// # let task = Task::new("test", "npm test");
    /// # let execution = TaskExecution {
    /// #     exit_code: 1,
    /// #     stdout: "".to_string(),
    /// #     stderr: "Test failed".to_string(),
    /// #     duration: Duration::from_secs(2),
    /// #     status: TaskStatus::Failed,
    /// # };
    /// #
    /// let result = TaskResultInfo::new(task, execution);
    ///
    /// if result.is_failure() {
    ///     println!("Task failed with exit code: {}", result.exit_code());
    /// }
    /// ```
    pub fn is_failure(&self) -> bool {
        matches!(
            self.execution.status,
            TaskStatus::Failed | TaskStatus::Timeout | TaskStatus::Cancelled
        )
    }

    /// Get the duration of the task
    ///
    /// # Returns
    ///
    /// The duration of the task execution.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
    /// # use std::time::Duration;
    /// #
    /// # let task = Task::new("build", "npm build");
    /// # let execution = TaskExecution {
    /// #     exit_code: 0,
    /// #     stdout: "Build successful".to_string(),
    /// #     stderr: "".to_string(),
    /// #     duration: Duration::from_secs(10),
    /// #     status: TaskStatus::Success,
    /// # };
    /// #
    /// let result = TaskResultInfo::new(task, execution);
    ///
    /// println!("Task took: {:?}", result.duration());
    /// ```
    pub fn duration(&self) -> Duration {
        self.execution.duration
    }

    /// Get the exit code
    ///
    /// # Returns
    ///
    /// The process exit code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
    /// # use std::time::Duration;
    /// #
    /// # let task = Task::new("test", "npm test");
    /// # let execution = TaskExecution {
    /// #     exit_code: 2,
    /// #     stdout: "".to_string(),
    /// #     stderr: "Error code 2".to_string(),
    /// #     duration: Duration::from_secs(1),
    /// #     status: TaskStatus::Failed,
    /// # };
    /// #
    /// let result = TaskResultInfo::new(task, execution);
    ///
    /// println!("Task failed with exit code: {}", result.exit_code());
    /// ```
    pub fn exit_code(&self) -> i32 {
        self.execution.exit_code
    }

    /// Get the task name
    ///
    /// # Returns
    ///
    /// The name of the executed task.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_monorepo_tools::{Task, TaskExecution, TaskResultInfo, TaskStatus};
    /// # use std::time::Duration;
    /// #
    /// # let task = Task::new("deploy", "npm run deploy");
    /// # let execution = TaskExecution {
    /// #     exit_code: 0,
    /// #     stdout: "Deployed successfully".to_string(),
    /// #     stderr: "".to_string(),
    /// #     duration: Duration::from_secs(30),
    /// #     status: TaskStatus::Success,
    /// # };
    /// #
    /// let result = TaskResultInfo::new(task, execution);
    ///
    /// println!("Task '{}' completed", result.name());
    /// ```
    pub fn name(&self) -> &str {
        &self.task.name
    }
}

impl fmt::Display for TaskResultInfo {
    /// Formats the task result info for display.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter
    ///
    /// # Returns
    ///
    /// Format result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task {} (status: {:?}, exit code: {}, duration: {:?})",
            self.task.name,
            self.execution.status,
            self.execution.exit_code,
            self.execution.duration
        )
    }
}
