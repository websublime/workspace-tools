//! # Command Queue Types
//!
//! ## What
//! This module defines types for managing command queues, including queue
//! configuration, command results, and internal queue management structures.
//!
//! ## How
//! The types work together to provide a comprehensive command queue system
//! with priority ordering, concurrency control, and result tracking.
//!
//! ## Why
//! Structured queue management enables sophisticated command execution
//! workflows with proper resource management and result collection.

use super::{command::CommandOutput, priority::{CommandPriority, CommandStatus}};
use std::time::{Duration, Instant};

/// Result of a queued command execution.
///
/// Contains all information about a command's execution, including its status,
/// output (if successful), or error details (if failed).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandQueue, CommandQueueResult, CommandStatus};
///
/// let queue = CommandQueue::new();
/// let command_id = queue.enqueue("npm", &["install"]);
///
/// // Later, when the command has completed
/// if let Some(result) = queue.get_result(&command_id) {
///     match result.status {
///         CommandStatus::Completed => println!("Command output: {}", result.output.unwrap().stdout),
///         CommandStatus::Failed => println!("Command failed: {}", result.error.unwrap()),
///         _ => println!("Command is still in progress"),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CommandQueueResult {
    /// Unique identifier of the command
    pub id: String,
    /// Status of the command execution
    pub status: CommandStatus,
    /// Command output if successful
    pub output: Option<CommandOutput>,
    /// Error information if failed
    pub error: Option<String>,
}

/// Configuration for a command queue.
///
/// Defines the behavior of the command queue, including concurrency limits,
/// rate limiting, and timeouts.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandQueue, CommandQueueConfig};
/// use std::time::Duration;
///
/// let config = CommandQueueConfig {
///     max_concurrent_commands: 4,
///     rate_limit: Some(Duration::from_millis(100)),
///     default_timeout: Duration::from_secs(60),
///     shutdown_timeout: Duration::from_secs(10),
/// };
///
/// let queue = CommandQueue::with_config(config);
/// ```
#[derive(Debug, Clone)]
pub struct CommandQueueConfig {
    /// Maximum number of commands that can run concurrently
    pub max_concurrent_commands: usize,
    /// Minimum time between command executions (for rate limiting)
    pub rate_limit: Option<Duration>,
    /// Default timeout for command execution
    pub default_timeout: Duration,
    /// Timeout when shutting down the queue
    pub shutdown_timeout: Duration,
}

/// Internal structure representing a queued command.
///
/// Contains the command information along with queue-specific metadata
/// like priority and enqueue time.
#[derive(Debug)]
pub(crate) struct QueuedCommand {
    /// Unique identifier for the command
    pub(crate) id: String,
    /// Command to execute
    pub(crate) command: super::command::Command,
    /// Execution priority
    pub(crate) priority: CommandPriority,
    /// When the command was added to the queue
    pub(crate) enqueued_at: Instant,
}

impl PartialEq for QueuedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.enqueued_at == other.enqueued_at
    }
}

impl Eq for QueuedCommand {}

/// Ordering for priority queue to make it a max heap by priority and min heap by time
impl PartialOrd for QueuedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by priority (higher is greater)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Then by enqueue time (earlier is greater)
                other.enqueued_at.cmp(&self.enqueued_at)
            }
            ordering => ordering,
        }
    }
}

impl Default for CommandQueueConfig {
    fn default() -> Self {
        Self {
            max_concurrent_commands: 4,
            rate_limit: None,
            default_timeout: Duration::from_secs(60),
            shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Message type for the command queue's internal communication.
///
/// Used for sending commands to be executed or signaling queue shutdown.
///
/// This is an internal enum used by the `CommandQueue` implementation.
#[derive(Debug)]
pub(crate) enum QueueMessage {
    /// Execute a command
    Execute(Box<QueuedCommand>),
    /// Start a batch operation - pause processing
    BatchStart,
    /// End a batch operation - resume processing with all commands properly prioritized
    BatchEnd,
    /// Shutdown the queue
    Shutdown,
}