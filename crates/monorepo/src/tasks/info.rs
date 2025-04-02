use super::task::{Task, TaskExecution, TaskStatus};
use std::{fmt, time::Duration};

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
        matches!(
            self.execution.status,
            TaskStatus::Failed | TaskStatus::Timeout | TaskStatus::Cancelled
        )
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
            self.task.name,
            self.execution.status,
            self.execution.exit_code,
            self.execution.duration
        )
    }
}
