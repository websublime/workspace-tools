//! # Internal Queue Types
//!
//! ## What
//! This module contains internal types used by the command queue system
//! for managing command execution and processing.
//!
//! ## How
//! These types provide the internal structure for queue processing,
//! including the main queue struct and internal processor.
//!
//! ## Why
//! Internal queue types enable sophisticated command queue management
//! with proper separation between public and internal APIs.

use super::{
    queue::{CommandQueueConfig, CommandQueueResult, QueuedCommand, QueueMessage},
    priority::CommandStatus,
};
use crate::command::executor::Executor;
use std::{
    collections::{BinaryHeap, HashMap},
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Semaphore,
    },
    task::JoinHandle,
};

/// A queue for managing command execution with priorities and concurrency control.
///
/// The `CommandQueue` provides an asynchronous interface for queuing up commands
/// to be executed with specific priorities, managing concurrency limits, and
/// collecting results.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandQueue, CommandQueueConfig};
/// use std::time::Duration;
///
/// // Create a command queue with default configuration
/// let mut queue = CommandQueue::new();
///
/// // Queue up some commands
/// let cmd1_id = queue.enqueue("npm", &["install"]);
/// let cmd2_id = queue.enqueue("cargo", &["build", "--release"]);
///
/// // Wait for all commands to complete
/// queue.wait_all(Some(Duration::from_secs(300)));
///
/// // Get results
/// let results = queue.get_all_results();
/// for result in results {
///     println!("Command {} completed with status: {:?}", result.id, result.status);
/// }
///
/// // Shutdown the queue when done
/// queue.shutdown();
/// ```
pub struct CommandQueue {
    /// Queue configuration
    pub(crate) config: CommandQueueConfig,
    /// Command executor for running commands
    pub(crate) executor: Arc<dyn Executor>,
    /// Sender for the command queue
    pub(crate) queue_sender: Option<Sender<QueueMessage>>,
    /// Status of queued commands
    pub(crate) statuses: Arc<Mutex<HashMap<String, CommandStatus>>>,
    /// Results of completed commands
    pub(crate) results: Arc<Mutex<HashMap<String, CommandQueueResult>>>,
    /// Handle to the queue processing task
    pub(crate) processor_handle: Option<JoinHandle<()>>,
    /// Command counter for generating IDs
    pub(crate) command_counter: Arc<Mutex<usize>>,
}

/// Internal processor for the command queue.
///
/// Manages the execution of queued commands based on priority, concurrency limits,
/// and rate limiting. This is an internal structure used by `CommandQueue`.
pub(crate) struct QueueProcessor {
    /// Queue configuration
    pub(crate) config: CommandQueueConfig,
    /// Receiver for command queue messages
    pub(crate) receiver: Receiver<QueueMessage>,
    /// Command executor for running commands
    pub(crate) executor: Arc<dyn Executor>,
    /// Command queue for prioritizing commands
    pub(crate) queue: BinaryHeap<QueuedCommand>,
    /// Semaphore to limit concurrent commands
    pub(crate) concurrency_semaphore: Arc<Semaphore>,
    /// Command statuses
    pub(crate) statuses: Arc<Mutex<HashMap<String, CommandStatus>>>,
    /// Command results
    pub(crate) results: Arc<Mutex<HashMap<String, CommandQueueResult>>>,
    /// Last command execution time (for rate limiting)
    pub(crate) last_execution: Option<Instant>,
    /// Whether the processor is running
    pub(crate) running: bool,
    /// Flag indicating if we're in batch mode
    pub(crate) batch_mode: bool,
}