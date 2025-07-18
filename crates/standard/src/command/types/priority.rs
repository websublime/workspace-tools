//! # Command Priority and Status Types
//!
//! ## What
//! This module defines command priority levels and execution status enums
//! for managing command execution order and lifecycle tracking.
//!
//! ## How
//! CommandPriority uses numeric values to ensure proper ordering in queues,
//! while CommandStatus provides clear lifecycle states for command tracking.
//!
//! ## Why
//! Explicit priority and status types enable sophisticated command queue
//! management and provide clear visibility into command execution states.

/// Priority levels for queued commands.
///
/// Used to determine the order of execution when multiple commands are queued.
/// Higher priority commands are executed before lower priority ones.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandPriority, CommandQueue};
///
/// // Queue a high priority command
/// let queue = CommandQueue::new();
/// let command_id = queue.enqueue_with_priority("npm", &["install"], CommandPriority::High);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    /// Low priority commands, executed after all others
    Low = 0,
    /// Normal priority commands, default level
    Normal = 1,
    /// High priority commands, executed before normal and low
    High = 2,
    /// Critical priority commands, executed before all others
    Critical = 3,
}

/// Status of a command in the execution queue.
///
/// Represents the lifecycle states of a command as it moves through the execution process.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandQueue, CommandStatus};
///
/// let queue = CommandQueue::new();
/// let command_id = queue.enqueue("npm", &["install"]);
///
/// // Check the status of the command
/// let status = queue.get_status(&command_id);
/// if status == CommandStatus::Queued {
///     println!("Command is waiting to be executed");
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// Command is waiting in the queue
    Queued,
    /// Command is currently being executed
    Running,
    /// Command completed successfully
    Completed,
    /// Command failed during execution
    Failed,
    /// Command was cancelled before execution
    Cancelled,
}

impl Default for CommandPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl CommandStatus {
    /// Returns true if the command execution is complete (either successfully or not).
    ///
    /// # Returns
    ///
    /// True if the command has finished executing
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandStatus;
    ///
    /// assert!(CommandStatus::Completed.is_completed());
    /// assert!(CommandStatus::Failed.is_completed());
    /// assert!(CommandStatus::Cancelled.is_completed());
    /// assert!(!CommandStatus::Queued.is_completed());
    /// assert!(!CommandStatus::Running.is_completed());
    /// ```
    #[must_use]
    pub fn is_completed(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Returns true if the command execution was successful.
    ///
    /// # Returns
    ///
    /// True if the command completed successfully
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandStatus;
    ///
    /// assert!(CommandStatus::Completed.is_successful());
    /// assert!(!CommandStatus::Failed.is_successful());
    /// assert!(!CommandStatus::Cancelled.is_successful());
    /// assert!(!CommandStatus::Queued.is_successful());
    /// assert!(!CommandStatus::Running.is_successful());
    /// ```
    #[must_use]
    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }
}