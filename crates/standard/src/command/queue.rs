//! # Command Queue Implementation
//!
//! ## What
//! This file implements a prioritized command queue system that allows for
//! controlled execution of commands with different priority levels. It handles
//! concurrent execution with configurable limits, rate limiting, and provides
//! a clean interface for command status tracking and result retrieval.
//!
//! ## How
//! The implementation uses tokio's asynchronous primitives including channels,
//! semaphores, and tasks to manage command execution. It maintains a priority-based
//! binary heap for scheduling commands and tracks their status and results in
//! thread-safe data structures. The queue processes commands concurrently up to
//! a configured limit, respecting priority levels from Critical to Low.
//!
//! ## Why
//! Many applications need to execute external commands with different importance
//! levels, requiring a way to prioritize critical operations while limiting
//! system resource usage. This queue implementation provides that capability
//! while ensuring safety, proper resource cleanup, and clear visibility into
//! command execution status.

use std::{
    collections::{BinaryHeap, HashMap},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::{
    sync::{
        mpsc::{self, Receiver},
        Semaphore,
    },
    time::sleep,
};

use crate::error::{Error, Result};

use super::{
    executor::CommandExecutor,
    types::{
        CommandOutput, CommandPriority, CommandQueue, CommandQueueConfig, CommandQueueResult,
        CommandStatus, QueueMessage, QueueProcessor, QueuedCommand,
    },
    Command, DefaultCommandExecutor,
};

impl Default for CommandPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for CommandQueueConfig {
    fn default() -> Self {
        Self {
            max_concurrent_commands: 4,
            rate_limit: None,
            default_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(10),
        }
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

impl CommandQueueResult {
    /// Creates a new successful result.
    ///
    /// # Arguments
    ///
    /// * `id` - Command identifier
    /// * `output` - Command execution output
    ///
    /// # Returns
    ///
    /// A new successful command result
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueueResult, CommandOutput};
    /// use std::time::Duration;
    ///
    /// let output = CommandOutput::new(0, "output".to_string(), "".to_string(), Duration::from_secs(1));
    /// let result = CommandQueueResult::success("cmd-123".to_string(), output);
    ///
    /// assert!(result.is_successful());
    /// ```
    #[must_use]
    pub fn success(id: String, output: CommandOutput) -> Self {
        Self { id, status: CommandStatus::Completed, output: Some(output), error: None }
    }

    /// Creates a new failed result.
    ///
    /// # Arguments
    ///
    /// * `id` - Command identifier
    /// * `error` - Error message
    ///
    /// # Returns
    ///
    /// A new failed command result
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandQueueResult;
    ///
    /// let result = CommandQueueResult::failure("cmd-123".to_string(), "Command failed".to_string());
    ///
    /// assert!(!result.is_successful());
    /// assert_eq!(result.error.unwrap(), "Command failed");
    /// ```
    #[must_use]
    pub fn failure(id: String, error: String) -> Self {
        Self { id, status: CommandStatus::Failed, output: None, error: Some(error) }
    }

    /// Creates a new cancelled result.
    ///
    /// # Arguments
    ///
    /// * `id` - Command identifier
    ///
    /// # Returns
    ///
    /// A new cancelled command result
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandQueueResult;
    ///
    /// let result = CommandQueueResult::cancelled("cmd-123".to_string());
    ///
    /// assert!(!result.is_successful());
    /// assert_eq!(result.status, sublime_standard_tools::command::CommandStatus::Cancelled);
    /// ```
    #[must_use]
    pub fn cancelled(id: String) -> Self {
        Self {
            id,
            status: CommandStatus::Cancelled,
            output: None,
            error: Some("Command was cancelled".to_string()),
        }
    }

    /// Returns true if the command was successful.
    ///
    /// # Returns
    ///
    /// True if the command completed successfully
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueueResult, CommandStatus};
    ///
    /// let result = CommandQueueResult {
    ///     id: "cmd-123".to_string(),
    ///     status: CommandStatus::Completed,
    ///     output: None,
    ///     error: None,
    /// };
    ///
    /// assert!(result.is_successful());
    /// ```
    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.status.is_successful()
    }
}

impl CommandQueue {
    /// Creates a new command queue with default configuration.
    ///
    /// # Returns
    ///
    /// A new, non-started command queue
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandQueue;
    ///
    /// let queue = CommandQueue::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(CommandQueueConfig::default())
    }

    /// Creates a new command queue with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Queue configuration parameters
    ///
    /// # Returns
    ///
    /// A new, non-started command queue with the specified configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, CommandQueueConfig};
    /// use std::time::Duration;
    ///
    /// let config = CommandQueueConfig {
    ///     max_concurrent_commands: 2,
    ///     rate_limit: Some(Duration::from_millis(100)),
    ///     default_timeout: Duration::from_secs(60),
    ///     shutdown_timeout: Duration::from_secs(5),
    /// };
    ///
    /// let queue = CommandQueue::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: CommandQueueConfig) -> Self {
        Self {
            config,
            executor: Arc::new(DefaultCommandExecutor::new()),
            queue_sender: None,
            statuses: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            processor_handle: None,
            command_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a command queue with a custom executor.
    ///
    /// # Arguments
    ///
    /// * `executor` - Command executor to use for running commands
    ///
    /// # Returns
    ///
    /// A new, non-started command queue with the specified executor
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, DefaultCommandExecutor};
    ///
    /// let executor = DefaultCommandExecutor::new();
    /// let queue = CommandQueue::with_executor(executor);
    /// ```
    #[must_use]
    pub fn with_executor<E: CommandExecutor + 'static>(executor: E) -> Self {
        Self {
            config: CommandQueueConfig::default(),
            executor: Arc::new(executor),
            queue_sender: None,
            statuses: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            processor_handle: None,
            command_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Starts the command queue processor.
    ///
    /// # Returns
    ///
    /// This queue instance if started successfully, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandQueue;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn start(mut self) -> Result<Self> {
        if self.queue_sender.is_some() {
            return Err(Error::operation("Command queue already started".to_string()));
        }

        let (tx, rx) = mpsc::channel(100);
        self.queue_sender = Some(tx);
        let processor = QueueProcessor::new(
            self.config.clone(),
            rx,
            Arc::clone(&self.executor),
            Arc::clone(&self.statuses),
            Arc::clone(&self.results),
        );

        self.processor_handle = Some(tokio::spawn(processor.process_queue()));

        Ok(self)
    }

    /// Enqueues a command for execution.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to enqueue
    /// * `priority` - Priority level for this command
    ///
    /// # Returns
    ///
    /// Command ID that can be used to check status or retrieve results
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// let command = Command::new("echo", &["Hello, world!"]);
    /// let id = queue.enqueue(command, CommandPriority::High).await?;
    ///
    /// println!("Enqueued command with ID: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enqueue(&self, command: Command, priority: CommandPriority) -> Result<String> {
        let Some(sender) = &self.queue_sender else {
            return Err(Error::operation("Command queue not started".to_string()));
        };

        // Generate command ID
        let id = {
            let mut counter = self
                .command_counter
                .lock()
                .map_err(|e| Error::operation(format!("Failed to lock command counter: {e}")))?;
            *counter += 1;
            format!("cmd-{counter}")
        };

        // Create queued command
        let queued_command =
            QueuedCommand { id: id.clone(), command, priority, enqueued_at: Instant::now() };

        // Update command status
        {
            let mut statuses = self
                .statuses
                .lock()
                .map_err(|e| Error::operation(format!("Failed to lock command statuses: {e}")))?;
            statuses.insert(id.clone(), CommandStatus::Queued);
        }

        // Send command to queue
        sender.send(QueueMessage::Execute(Box::new(queued_command))).await.map_err(|_| {
            Error::operation("Failed to enqueue command, queue processor has shut down".to_string())
        })?;

        Ok(id)
    }

    /// Gets the status of a queued command.
    ///
    /// # Arguments
    ///
    /// * `id` - Command ID returned from enqueue
    ///
    /// # Returns
    ///
    /// Command status or None if the command is not found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// let command = Command::new("sleep", &["1"]);
    /// let id = queue.enqueue(command, CommandPriority::Normal).await?;
    ///
    /// let status = queue.get_status(&id);
    /// println!("Command status: {:?}", status);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_status(&self, id: &str) -> Option<CommandStatus> {
        match self.statuses.lock() {
            Ok(statuses) => statuses.get(id).copied(),
            Err(e) => {
                log::error!("Failed to lock command statuses: {}", e);
                None
            }
        }
    }

    /// Gets the result of a completed command.
    ///
    /// # Arguments
    ///
    /// * `id` - Command ID returned from enqueue
    ///
    /// # Returns
    ///
    /// Command result or None if the command is not completed or not found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// let command = Command::new("echo", &["Hello"]);
    /// let id = queue.enqueue(command, CommandPriority::Normal).await?;
    ///
    /// // Wait for completion
    /// tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    ///
    /// if let Some(result) = queue.get_result(&id) {
    ///     if result.is_successful() {
    ///         println!("Command output: {:?}", result.output);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_result(&self, id: &str) -> Option<CommandQueueResult> {
        match self.results.lock() {
            Ok(results) => results.get(id).cloned(),
            Err(e) => {
                log::error!("Failed to lock command results: {}", e);
                None
            }
        }
    }

    /// Waits for a specific command to complete.
    ///
    /// # Arguments
    ///
    /// * `id` - Command ID returned from enqueue
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// Command result or error if timeout occurs or command not found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// let command = Command::new("echo", &["Hello"]);
    /// let id = queue.enqueue(command, CommandPriority::Normal).await?;
    ///
    /// // Wait for this specific command with a timeout
    /// let result = queue.wait_for_command(&id, Duration::from_secs(5)).await?;
    ///
    /// if result.is_successful() {
    ///     println!("Command completed successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_command(
        &self,
        id: &str,
        timeout: Duration,
    ) -> Result<CommandQueueResult> {
        let start_time = Instant::now();
        let id = id.to_string();

        while start_time.elapsed() < timeout {
            // Check if command is completed
            match self.get_status(&id) {
                Some(status) if status.is_completed() => {
                    return match self.get_result(&id) {
                        Some(result) => Ok(result),
                        None => Err(Error::operation(format!(
                            "Command {id} completed but no result found"
                        ))),
                    };
                }
                Some(_) => {
                    // Command exists but not completed, wait a bit
                    sleep(Duration::from_millis(100)).await;
                }
                None => {
                    return Err(Error::operation(format!("Command {id} not found")));
                }
            }
        }

        Err(Error::operation(format!("Timeout waiting for command {id} to complete")))
    }

    /// Waits for all queued commands to complete.
    ///
    /// # Returns
    ///
    /// Success if all commands complete or error if timeout occurs
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// // Enqueue multiple commands
    /// for i in 0..3 {
    ///     let command = Command::new("echo", &[&format!("Command {}", i)]);
    ///     queue.enqueue(command, CommandPriority::Normal).await?;
    /// }
    ///
    /// // Wait for all commands to complete
    /// queue.wait_for_completion().await?;
    /// println!("All commands completed");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_completion(&self) -> Result<()> {
        // Get all command IDs
        let command_ids = {
            let statuses = self
                .statuses
                .lock()
                .map_err(|e| Error::operation(format!("Failed to lock command statuses: {e}")))?;
            statuses.keys().cloned().collect::<Vec<_>>()
        };

        if command_ids.is_empty() {
            return Ok(());
        }

        let timeout = self.config.shutdown_timeout;
        let start_time = Instant::now();

        // Wait for all commands to complete or timeout
        let mut all_completed = false;
        while !all_completed && start_time.elapsed() < timeout {
            all_completed = true; // Assume all completed unless we find an incomplete one

            {
                let statuses = self.statuses.lock().map_err(|e| {
                    Error::operation(format!("Failed to lock command statuses: {e}"))
                })?;

                for id in &command_ids {
                    match statuses.get(id) {
                        Some(status) if status.is_completed() => {
                            // This command is completed, continue checking others
                        }
                        _ => {
                            // This command is not completed, need to wait more
                            all_completed = false;
                            break;
                        }
                    }
                }
            }

            if !all_completed {
                // Don't busy-wait, sleep a bit
                sleep(Duration::from_millis(50)).await;
            }
        }

        if all_completed {
            Ok(())
        } else {
            Err(Error::operation("Timeout waiting for all commands to complete".to_string()))
        }
    }

    /// Shuts down the command queue, cancelling all pending commands.
    ///
    /// # Returns
    ///
    /// Success if shutdown completes or error
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandQueue;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// // ... use queue ...
    ///
    /// // Shutdown the queue when done
    /// queue.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(sender) = self.queue_sender.take() {
            // Send shutdown message
            let _ = sender.send(QueueMessage::Shutdown).await;

            // Wait for processor to finish
            if let Some(handle) = self.processor_handle.take() {
                match tokio::time::timeout(self.config.shutdown_timeout, handle).await {
                    Ok(result) => {
                        if let Err(e) = result {
                            log::error!("Error during queue processor shutdown: {}", e);
                        }
                    }
                    Err(_) => {
                        log::warn!("Timeout during queue processor shutdown");
                    }
                }
            }

            Ok(())
        } else {
            Err(Error::operation("Command queue not started".to_string()))
        }
    }
}

impl QueueProcessor {
    /// Creates a new queue processor.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the queue processor
    /// * `receiver` - Channel receiver for queue messages
    /// * `executor` - Executor for running commands
    /// * `statuses` - Shared map of command statuses
    /// * `results` - Shared map of command results
    ///
    /// # Returns
    ///
    /// A new queue processor instance
    fn new(
        config: CommandQueueConfig,
        receiver: Receiver<QueueMessage>,
        executor: Arc<dyn CommandExecutor>,
        statuses: Arc<Mutex<HashMap<String, CommandStatus>>>,
        results: Arc<Mutex<HashMap<String, CommandQueueResult>>>,
    ) -> Self {
        Self {
            concurrency_semaphore: Arc::new(Semaphore::new(config.max_concurrent_commands)),
            config,
            receiver,
            executor,
            queue: BinaryHeap::new(),
            statuses,
            results,
            last_execution: None,
            running: true,
        }
    }

    /// Processes a command queue, respecting priorities and concurrency limits.
    ///
    /// This method continuously monitors the command queue, executing commands
    /// based on their priority while respecting the configured concurrency limits
    /// and rate limiting rules. It continues until a shutdown message is received
    /// or the channel is closed.
    ///
    /// # Examples
    ///
    /// This method is typically called from inside a spawned task:
    ///
    /// ```no_run
    /// use sublime_standard_tools::command::CommandQueueConfig;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// # use sublime_standard_tools::command::DefaultCommandExecutor;
    /// let processor = QueueProcessor::new(
    ///     CommandQueueConfig::default(),
    ///     receiver,
    ///     Arc::new(DefaultCommandExecutor::new()),
    ///     statuses,
    ///     results
    /// );
    ///
    /// tokio::spawn(processor.process_queue());
    /// # }
    /// ```
    async fn process_queue(mut self) {
        while self.running {
            // First, collect any pending commands
            let mut collected_commands = false;

            // Try to collect messages for a short window (5ms)
            let collect_deadline =
                tokio::time::Instant::now() + tokio::time::Duration::from_millis(5);

            while tokio::time::Instant::now() < collect_deadline {
                // Poll for messages (non-blocking)
                match self.receiver.try_recv() {
                    Ok(QueueMessage::Execute(boxed_cmd)) => {
                        // Add command to the priority queue
                        self.queue.push(*boxed_cmd);
                        collected_commands = true;
                    }
                    Ok(QueueMessage::Shutdown) => {
                        self.running = false;
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        // No more messages at the moment
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        // Channel closed, exit loop
                        self.running = false;
                        break;
                    }
                }

                // Small sleep to prevent CPU spin while still collecting rapidly
                tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
            }

            // If we collected any commands or already had some, process the highest priority one
            if collected_commands || !self.queue.is_empty() {
                self.process_next_command().await;
            } else if self.running {
                // No commands in queue and channel empty, wait a bit to avoid CPU spin
                // We wait for a slightly longer time here since we're idle
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        // Process any remaining commands in the queue
        while !self.queue.is_empty() {
            self.process_next_command().await;
        }

        log::info!("Command queue processor has shut down");
    }

    /// Processes the next command in the queue, respecting concurrency limits.
    ///
    /// This method applies rate limiting, retrieves the highest priority command
    /// from the queue, updates its status to Running, and executes it in a separate
    /// task. The execution permit from the semaphore is held until the command
    /// completes, ensuring the concurrency limit is maintained.
    ///
    /// # Returns
    ///
    /// Nothing, but updates command statuses and results as side effects.
    #[allow(clippy::manual_let_else)]
    async fn process_next_command(&mut self) {
        // Apply rate limit
        if let Some(rate_limit) = self.config.rate_limit {
            if let Some(last_time) = self.last_execution {
                let elapsed = last_time.elapsed();
                if elapsed < rate_limit {
                    sleep(rate_limit - elapsed).await;
                }
            }
        }

        // Get the highest priority command
        let Some(cmd) = self.queue.pop() else {
            return;
        };

        // Get the command ID for logging
        let id = cmd.id.clone();

        // Update status to running - do this before we try to acquire the semaphore
        if let Ok(mut statuses) = self.statuses.lock() {
            statuses.insert(id.clone(), CommandStatus::Running);
        }

        let executor = Arc::clone(&self.executor);
        let statuses = Arc::clone(&self.statuses);
        let results = Arc::clone(&self.results);
        let semaphore = Arc::clone(&self.concurrency_semaphore);

        // Launch the command execution in a separate task
        tokio::spawn(async move {
            // Acquire a permit from the semaphore
            let permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    // Semaphore was closed
                    if let Ok(mut statuses) = statuses.lock() {
                        statuses.insert(id.clone(), CommandStatus::Failed);
                    }

                    if let Ok(mut results_map) = results.lock() {
                        results_map.insert(
                            id.clone(),
                            CommandQueueResult::failure(
                                id,
                                "Failed to acquire execution permit".to_string(),
                            ),
                        );
                    }
                    return;
                }
            };

            // Execute the command and hold the permit until completion
            let result = executor.execute(cmd.command).await;

            // Update status and result based on command execution result
            let (status, queue_result) = match result {
                Ok(output) => {
                    let status = CommandStatus::Completed;
                    let result = CommandQueueResult::success(id.clone(), output);
                    (status, result)
                }
                Err(err) => {
                    let status = CommandStatus::Failed;
                    let result = CommandQueueResult::failure(id.clone(), err.to_string());
                    (status, result)
                }
            };

            // Update status and result
            if let Ok(mut statuses) = statuses.lock() {
                statuses.insert(id.clone(), status);
            }

            if let Ok(mut results_map) = results.lock() {
                results_map.insert(id, queue_result);
            }

            // Explicitly drop the permit to release the semaphore
            drop(permit);
        });

        // Update last execution time for rate limiting
        self.last_execution = Some(Instant::now());
    }
}
