//! # Command Queue Implementation
//!
//! ## What
//! This module implements the main CommandQueue struct and its methods,
//! providing a high-level interface for managing command execution.
//!
//! ## How
//! The implementation provides methods for creating, starting, and managing
//! command queues with priority-based execution and resource management.
//!
//! ## Why
//! Separating the CommandQueue implementation into its own module improves
//! code organization and makes it easier to maintain and test.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::{
    sync::mpsc::{self},
    time::sleep,
};

use crate::error::{Error, Result};

use super::super::{
    executor::Executor,
    types::{
        CommandPriority, CommandQueue, CommandQueueConfig, CommandQueueResult, CommandStatus,
        DefaultCommandExecutor, QueueMessage, QueueProcessor, QueuedCommand,
    },
    Command,
};

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
    pub fn with_executor<E: Executor + 'static>(executor: E) -> Self {
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
    /// # Errors
    ///
    /// Returns an error if the queue is already started.
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
    /// # Errors
    ///
    /// Returns an error if the queue is not started or if the channel is closed.
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

    /// Enqueues multiple commands as a batch, ensuring proper priority ordering.
    ///
    /// This method guarantees that all commands in the batch are enqueued and
    /// properly prioritized before any of them start executing, solving race
    /// conditions with priority ordering.
    ///
    /// # Arguments
    ///
    /// * `commands` - Vector of (Command, `CommandPriority`) tuples to enqueue
    ///
    /// # Returns
    ///
    /// Vector of command IDs in the same order as the input commands
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::{CommandQueue, Command, CommandPriority};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let queue = CommandQueue::new().start()?;
    ///
    /// let mut commands = Vec::new();
    /// commands.push((Command::new("echo", &["Critical task"]), CommandPriority::Critical));
    /// commands.push((Command::new("echo", &["High task"]), CommandPriority::High));
    /// commands.push((Command::new("echo", &["Normal task"]), CommandPriority::Normal));
    ///
    /// // Enqueue all commands as a batch, ensuring proper priority ordering
    /// let command_ids = queue.enqueue_batch(commands).await?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the queue is not started or if the channel is closed.
    pub async fn enqueue_batch(
        &self,
        commands: Vec<(Command, CommandPriority)>,
    ) -> Result<Vec<String>> {
        if commands.is_empty() {
            return Ok(Vec::new());
        }

        let Some(sender) = &self.queue_sender else {
            return Err(Error::operation("Command queue not started".to_string()));
        };

        // Signal the start of a batch operation
        sender.send(QueueMessage::BatchStart).await.map_err(|_| {
            Error::operation(
                "Failed to start batch operation, queue processor has shut down".to_string(),
            )
        })?;

        // Generate command IDs for all commands
        let ids = {
            let mut counter = self
                .command_counter
                .lock()
                .map_err(|e| Error::operation(format!("Failed to lock command counter: {e}")))?;

            let mut ids = Vec::with_capacity(commands.len());
            for _i in 0..commands.len() {
                *counter += 1;
                ids.push(format!("cmd-{counter}"));
            }
            ids
        };

        // Update all command statuses to Queued
        {
            let mut statuses = self
                .statuses
                .lock()
                .map_err(|e| Error::operation(format!("Failed to lock command statuses: {e}")))?;

            for id in &ids {
                statuses.insert(id.clone(), CommandStatus::Queued);
            }
        }

        // Prepare all queued commands
        let now = Instant::now();
        let mut queued_commands = Vec::with_capacity(commands.len());

        for ((command, priority), id) in commands.into_iter().zip(ids.iter()) {
            let queued_command =
                QueuedCommand { id: id.clone(), command, priority, enqueued_at: now };
            queued_commands.push(queued_command);
        }

        // Send all commands to queue
        for cmd in queued_commands {
            sender.send(QueueMessage::Execute(Box::new(cmd))).await.map_err(|_| {
                Error::operation(
                    "Failed to enqueue batch, queue processor has shut down".to_string(),
                )
            })?;
        }

        // Signal the end of batch operation, allowing processing to begin
        sender.send(QueueMessage::BatchEnd).await.map_err(|_| {
            Error::operation(
                "Failed to end batch operation, queue processor has shut down".to_string(),
            )
        })?;

        Ok(ids)
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
    /// # Errors
    ///
    /// Returns an error if the command is not found or if the timeout is reached.
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
    /// # Errors
    ///
    /// Returns an error if unable to access command statuses or if timeout is reached.
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
    /// # Errors
    ///
    /// Returns an error if the queue was not started.
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

impl std::fmt::Debug for CommandQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandQueue")
            .field("config", &self.config)
            .field("executor", &"Arc<dyn CommandExecutor>")
            .field("queue_sender", &self.queue_sender.as_ref().map(|_| "Sender<QueueMessage>"))
            .field("statuses", &self.statuses)
            .field("results", &self.results)
            .field("processor_handle", &self.processor_handle.as_ref().map(|_| "JoinHandle<()>"))
            .field("command_counter", &self.command_counter)
            .finish()
    }
}
