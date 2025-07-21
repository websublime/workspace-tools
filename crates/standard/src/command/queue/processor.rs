//! # Queue Processor Implementation
//!
//! ## What
//! This module implements the QueueProcessor struct and its methods,
//! providing the core logic for processing commands in priority order.
//!
//! ## How
//! The implementation uses async primitives and priority queues to execute
//! commands based on their priority while respecting concurrency limits.
//!
//! ## Why
//! Separating the queue processor logic improves code organization and
//! makes it easier to maintain and test the core command execution logic.

use std::{
    collections::{BinaryHeap, HashMap},
    sync::{Arc, Mutex},
    time::Instant,
};

use tokio::{
    sync::{mpsc::Receiver, Semaphore},
    time::sleep,
};

use super::super::{
    executor::Executor,
    types::{CommandQueueConfig, CommandQueueResult, CommandStatus, QueueMessage, QueueProcessor},
};

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
    pub(crate) fn new(
        config: CommandQueueConfig,
        receiver: Receiver<QueueMessage>,
        executor: Arc<dyn Executor>,
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
            batch_mode: false,
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
    pub(crate) async fn process_queue(mut self) {
        while self.running {
            // First, collect any pending commands
            let mut collected_commands = false;

            // Try to collect messages for a configured window
            let collect_deadline =
                tokio::time::Instant::now() + tokio::time::Duration::from_millis(self.config.collection_window_ms);

            while tokio::time::Instant::now() < collect_deadline {
                // Poll for messages (non-blocking)
                match self.receiver.try_recv() {
                    Ok(QueueMessage::Execute(boxed_cmd)) => {
                        // Add command to the priority queue
                        self.queue.push(*boxed_cmd);
                        collected_commands = true;
                    }
                    Ok(QueueMessage::BatchStart) => {
                        // Enter batch mode - pause processing
                        self.batch_mode = true;
                    }
                    Ok(QueueMessage::BatchEnd) => {
                        // Exit batch mode - resume processing
                        self.batch_mode = false;
                        collected_commands = true; // Force queue processing
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
                tokio::time::sleep(tokio::time::Duration::from_micros(self.config.collection_sleep_us)).await;
            }

            // If we collected any commands or already had some, process the highest priority one
            if !self.batch_mode && (collected_commands || !self.queue.is_empty()) {
                self.process_next_command().await;
            } else if self.running {
                // No commands in queue and channel empty, wait a bit to avoid CPU spin
                // We wait for a configured time here since we're idle
                tokio::time::sleep(tokio::time::Duration::from_millis(self.config.idle_sleep_ms)).await;
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
            let permit = if let Ok(permit) = semaphore.acquire().await {
                permit
            } else {
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
