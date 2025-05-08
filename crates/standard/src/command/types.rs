//! # Command execution type definitions
//!
//! ## What
//! This module contains the core type definitions for the command execution system.
//! It defines structures for representing commands, command execution results,
//! command queues, and streaming output.
//!
//! ## How
//! Types are organized into several categories:
//! - Command representation (Command, CommandBuilder)
//! - Command execution results (CommandOutput)
//! - Command queuing and prioritization (CommandQueue, QueuedCommand, CommandPriority)
//! - Stream handling for command output (StreamOutput, CommandStream)
//!
//! ## Why
//! These types provide a robust foundation for executing shell commands with
//! various execution strategies (synchronous, asynchronous, queued) while handling
//! timeouts, priorities, concurrency, and output streaming in a consistent way.

use super::executor::CommandExecutor;
use std::{
    collections::{BinaryHeap, HashMap},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Semaphore,
    },
    task::JoinHandle,
};

/// Result of executing a command.
///
/// Contains the exit status, captured stdout and stderr output, and the duration
/// of the command execution.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::CommandOutput;
/// use std::time::Duration;
///
/// let output = CommandOutput {
///     status: 0,
///     stdout: "Hello, world!".to_string(),
///     stderr: "".to_string(),
///     duration: Duration::from_millis(50),
/// };
///
/// assert_eq!(output.status, 0);
/// assert_eq!(output.stdout, "Hello, world!");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandOutput {
    /// Exit status code
    pub(crate) status: i32,
    /// Standard output content
    pub(crate) stdout: String,
    /// Standard error content
    pub(crate) stderr: String,
    /// Command execution duration
    pub(crate) duration: Duration,
}

/// Represents a command to be executed.
///
/// Contains all the information needed to execute a command, including the program
/// to run, its arguments, environment variables, working directory, and timeout.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{Command, CommandBuilder};
/// use std::collections::HashMap;
/// use std::time::Duration;
///
/// // Create a command using CommandBuilder
/// let command = CommandBuilder::new("npm")
///     .arg("install")
///     .env("NODE_ENV", "production")
///     .timeout(Duration::from_secs(60))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct Command {
    /// Program to execute
    pub(crate) program: String,
    /// Command arguments
    pub(crate) args: Vec<String>,
    /// Environment variables
    pub(crate) env: HashMap<String, String>,
    /// Working directory
    pub(crate) current_dir: Option<PathBuf>,
    /// Execution timeout
    pub(crate) timeout: Option<Duration>,
}

/// Builder for creating Command instances.
///
/// Provides a fluent interface for configuring command parameters before execution.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::CommandBuilder;
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let command = CommandBuilder::new("cargo")
///     .arg("test")
///     .args(&["--all-features", "--no-fail-fast"])
///     .env("RUST_BACKTRACE", "1")
///     .current_dir(PathBuf::from("./my-project"))
///     .timeout(Duration::from_secs(120))
///     .build();
/// ```
#[derive(Debug)]
pub struct CommandBuilder {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) current_dir: Option<PathBuf>,
    pub(crate) timeout: Option<Duration>,
}

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

/// Default implementation of the CommandExecutor trait.
///
/// Provides a standard implementation for executing commands directly without
/// any custom behavior.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{DefaultCommandExecutor, CommandExecutor};
/// use sublime_standard_tools::command::types::Command;
///
/// let executor = DefaultCommandExecutor::default();
/// // Use the executor to run commands
/// ```
#[derive(Debug, Default)]
pub struct DefaultCommandExecutor;

#[derive(Debug)]
pub(crate) struct QueuedCommand {
    /// Unique identifier for the command
    pub(crate) id: String,
    /// Command to execute
    pub(crate) command: Command,
    /// Execution priority
    pub(crate) priority: CommandPriority,
    /// When the command was added to the queue
    pub(crate) enqueued_at: Instant,
}

/// Message type for the command queue's internal communication.
///
/// Used for sending commands to be executed or signaling queue shutdown.
///
/// This is an internal enum used by the CommandQueue implementation.
#[derive(Debug)]
pub(crate) enum QueueMessage {
    /// Execute a command
    Execute(Box<QueuedCommand>),
    /// Shutdown the queue
    Shutdown,
}

/// A queue for managing command execution with priorities and concurrency control.
///
/// The CommandQueue provides an asynchronous interface for queuing up commands
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
    pub(crate) executor: Arc<dyn CommandExecutor>,
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

/// Type of output from a command stream.
///
/// Used to differentiate between standard output, standard error, and stream end markers.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandExecutor, StreamOutput};
///
/// async fn process_output(output: StreamOutput) {
///     match output {
///         StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
///         StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
///         StreamOutput::End => println!("Stream ended"),
///     }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StreamOutput {
    /// Standard output line
    Stdout(String),
    /// Standard error line
    Stderr(String),
    /// Stream has ended
    End,
}

/// Configuration for command output streaming.
///
/// Defines parameters for how command output streams are handled,
/// including buffer sizes and timeouts.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{StreamConfig};
/// use std::time::Duration;
///
/// let config = StreamConfig {
///     buffer_size: 1024,
///     read_timeout: Duration::from_millis(100),
/// };
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for output channel
    pub(crate) buffer_size: usize,
    /// Read timeout for each line
    pub(crate) read_timeout: Duration,
}

/// Stream of output from a running command.
///
/// Provides an asynchronous stream of stdout and stderr from a command,
/// allowing real-time processing of command output.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{CommandExecutor, StreamOutput};
/// use tokio::stream::StreamExt;
///
/// async fn stream_command(executor: impl CommandExecutor) {
///     let mut stream = executor.execute_streaming("ls", &["-la"]).await.unwrap();
///
///     while let Some(output) = stream.next().await {
///         match output {
///             StreamOutput::Stdout(line) => println!("STDOUT: {}", line),
///             StreamOutput::Stderr(line) => eprintln!("STDERR: {}", line),
///             StreamOutput::End => break,
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct CommandStream {
    /// Channel receiver for output
    pub(crate) rx: mpsc::Receiver<StreamOutput>,
    /// Flag for cancellation
    pub(crate) cancel: Arc<AtomicBool>,
}

/// Internal processor for the command queue.
///
/// Manages the execution of queued commands based on priority, concurrency limits,
/// and rate limiting. This is an internal structure used by CommandQueue.
pub(crate) struct QueueProcessor {
    /// Queue configuration
    pub(crate) config: CommandQueueConfig,
    /// Receiver for command queue messages
    pub(crate) receiver: Receiver<QueueMessage>,
    /// Command executor for running commands
    pub(crate) executor: Arc<dyn CommandExecutor>,
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
}
