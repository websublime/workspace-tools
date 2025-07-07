//! # Command Module Tests
//!
//! ## What
//! This file contains unit tests for the command module functionality,
//! ensuring all components work correctly independently and together.
//!
//! ## How
//! Tests are organized into sections covering CommandBuilder, CommandExecutor,
//! CommandOutput, CommandQueue, and CommandStream. Each test focuses on a specific
//! aspect of functionality with clear assertions.
//!
//! ## Why
//! Comprehensive testing ensures that command execution, output handling, and queuing
//! work correctly across different scenarios and edge cases, providing confidence
//! in the reliability of the module.

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use crate::command::types::QueuedCommand;
    use crate::command::{
        Command, CommandBuilder, CommandOutput, CommandPriority, CommandQueue, CommandQueueConfig,
        CommandStatus, DefaultCommandExecutor, Executor as CommandExecutor, StreamConfig,
        StreamOutput,
    };
    use crate::error::{CommandError, Error, Result};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::time::Instant;
    use std::{path::PathBuf, sync::Arc, time::Duration};

    #[test]
    fn test_command_builder() {
        let cmd = CommandBuilder::new("echo")
            .arg("hello")
            .arg("world")
            .timeout(Duration::from_secs(5))
            .current_dir("/tmp")
            .build();

        assert_eq!(cmd.program, "echo");
        assert_eq!(cmd.args, vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(cmd.timeout, Some(Duration::from_secs(5)));
        assert_eq!(cmd.current_dir, Some(PathBuf::from("/tmp")));
        assert!(cmd.env.is_empty());
    }

    #[test]
    fn test_command_output() {
        let output = CommandOutput::new(
            0,
            "stdout content".to_string(),
            "stderr content".to_string(),
            Duration::from_secs(1),
        );

        assert_eq!(output.status(), 0);
        assert_eq!(output.stdout(), "stdout content");
        assert_eq!(output.stderr(), "stderr content");
        assert_eq!(output.duration(), Duration::from_secs(1));
        assert!(output.success());

        let failed_output =
            CommandOutput::new(1, String::new(), "error".to_string(), Duration::from_secs(1));
        assert!(!failed_output.success());
    }

    #[tokio::test]
    async fn test_default_executor_execute() {
        let executor = DefaultCommandExecutor::new();
        let cmd = CommandBuilder::new("echo").arg("test message").build();

        let result = executor.execute(cmd).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.status(), 0);
        assert!(output.stdout().contains("test message"));
    }

    #[test]
    fn test_command_status() {
        assert!(CommandStatus::Completed.is_completed());
        assert!(CommandStatus::Failed.is_completed());
        assert!(CommandStatus::Cancelled.is_completed());
        assert!(!CommandStatus::Queued.is_completed());
        assert!(!CommandStatus::Running.is_completed());

        assert!(CommandStatus::Completed.is_successful());
        assert!(!CommandStatus::Failed.is_successful());
        assert!(!CommandStatus::Cancelled.is_successful());
        assert!(!CommandStatus::Queued.is_successful());
        assert!(!CommandStatus::Running.is_successful());
    }

    // Mock CommandExecutor for testing CommandQueue
    struct MockCommandExecutor;

    #[async_trait]
    impl CommandExecutor for MockCommandExecutor {
        async fn execute(&self, command: Command) -> Result<CommandOutput> {
            // Simulate command execution with predictable output
            Ok(CommandOutput::new(
                0,
                format!("Mock stdout for: {}", command.program),
                String::new(),
                Duration::from_millis(10),
            ))
        }

        async fn execute_stream(
            &self,
            _command: Command,
            _stream_config: StreamConfig,
        ) -> Result<(crate::command::CommandStream, tokio::process::Child)> {
            Err(Error::Command(CommandError::Generic(
                "Mock executor doesn't support streaming".to_string(),
            )))
        }
    }

    #[tokio::test]
    async fn test_command_queue() {
        // Create a command queue with mock executor
        let queue = CommandQueue {
            config: CommandQueueConfig {
                max_concurrent_commands: 2,
                rate_limit: Some(Duration::from_millis(10)),
                default_timeout: Duration::from_secs(1),
                shutdown_timeout: Duration::from_secs(1),
            },
            executor: Arc::new(MockCommandExecutor),
            queue_sender: None,
            statuses: Arc::default(),
            results: Arc::default(),
            processor_handle: None,
            command_counter: Arc::default(),
        };

        // Start the queue
        let mut queue = queue.start().expect("Failed to start queue");

        // Enqueue commands
        let cmd1 = CommandBuilder::new("test1").build();
        let cmd2 = CommandBuilder::new("test2").build();

        let id1 =
            queue.enqueue(cmd1, CommandPriority::Normal).await.expect("Failed to enqueue cmd1");
        let id2 = queue.enqueue(cmd2, CommandPriority::High).await.expect("Failed to enqueue cmd2");

        // Wait for commands to complete
        let result1 = queue.wait_for_command(&id1, Duration::from_secs(2)).await;
        let result2 = queue.wait_for_command(&id2, Duration::from_secs(2)).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        assert!(result1.unwrap().is_successful());
        assert!(result2.unwrap().is_successful());

        // Verify correct execution via get_result
        let output1 = queue.get_result(&id1).expect("Result should be available");
        assert!(output1.output.as_ref().unwrap().stdout().contains("test1"));

        // Shutdown the queue
        queue.shutdown().await.expect("Failed to shutdown queue");
    }

    #[tokio::test]
    async fn test_stream_config() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.read_timeout, Duration::from_secs(1));

        let custom_config = StreamConfig::new(2048, Duration::from_secs(2));
        assert_eq!(custom_config.buffer_size, 2048);
        assert_eq!(custom_config.read_timeout, Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_command_stream() {
        let executor = DefaultCommandExecutor::new();
        let stream_config = StreamConfig::default();

        // Create a command that outputs to both stdout and stderr in a cross-platform way
        #[cfg(target_family = "unix")]
        let command = CommandBuilder::new("sh")
            .arg("-c")
            .arg("echo 'stdout line 1'; echo 'stderr line' >&2; echo 'stdout line 2'")
            .build();

        #[cfg(target_family = "windows")]
        let command = CommandBuilder::new("cmd")
            .arg("/C")
            .arg("echo stdout line 1 && echo stderr line 1>&2 && echo stdout line 2")
            .build();

        // Execute with streaming
        let result = executor.execute_stream(command, stream_config).await;
        assert!(result.is_ok());

        let (mut stream, mut child) = result.unwrap();

        // Read from the stream and verify output
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        // Read up to 3 lines (we expect 3 total)
        for _ in 0..3 {
            match stream.next_timeout(Duration::from_secs(1)).await {
                Ok(Some(StreamOutput::Stdout(line))) => {
                    stdout_lines.push(line);
                }
                Ok(Some(StreamOutput::Stderr(line))) => {
                    stderr_lines.push(line);
                }
                Ok(Some(StreamOutput::End) | None) | Err(_) => break,
            }
        }

        // Clean up the process
        let _ = child.kill().await;

        // Verify we got output from both streams
        assert!(!stdout_lines.is_empty(), "Should have received stdout output");
        assert!(!stderr_lines.is_empty(), "Should have received stderr output");

        // Test cancellation
        stream.cancel();
    }

    #[allow(clippy::panic)]
    #[tokio::test]
    async fn test_command_not_found() {
        let executor = DefaultCommandExecutor::new();
        let cmd = CommandBuilder::new("non_existent_command_12345").build();

        let result = executor.execute(cmd).await;
        assert!(result.is_err());

        match result {
            Err(Error::Command(CommandError::SpawnFailed { cmd: _, message: _ })) => {
                // This is the expected error
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[allow(clippy::panic)]
    #[tokio::test]
    async fn test_command_timeout() {
        let executor = DefaultCommandExecutor::new();

        // On Unix systems, use "sleep" command; for Windows compatibility, the actual command might need to be adjusted
        #[cfg(target_family = "unix")]
        let cmd = CommandBuilder::new("sleep")
            .arg("2") // Sleep for 2 seconds
            .timeout(Duration::from_millis(100)) // But timeout after 100ms
            .build();

        #[cfg(target_family = "windows")]
        let cmd = CommandBuilder::new("ping")
            .arg("-n")
            .arg("10") // Ping 10 times with default 1-second interval
            .arg("127.0.0.1") // Localhost
            .timeout(Duration::from_millis(100)) // But timeout after 100ms
            .build();

        let start_time = Instant::now();
        let result = executor.execute(cmd).await;
        let elapsed = start_time.elapsed();

        // Should fail with timeout
        assert!(result.is_err());
        assert!(elapsed < Duration::from_secs(2));

        match result {
            Err(Error::Command(CommandError::Timeout { duration: _ })) => {
                // This is the expected error
            }
            Err(e) => panic!("Unexpected error type: {e}"),
            Ok(_) => panic!("Expected timeout error but command succeeded"),
        }
    }

    // Test the CommandQueue ordering by priority
    #[tokio::test]
    async fn test_command_priority_ordering() {
        // Test the specific ordering mechanism used in QueuedCommand

        use std::collections::BinaryHeap;
        use std::time::{Duration, Instant};

        // Create a binary heap (which is a max-heap)
        let mut queue = BinaryHeap::new();

        // Add commands in reverse priority order
        let now = Instant::now();

        // Add normal priority
        queue.push(QueuedCommand {
            id: "normal".to_string(),
            command: Command {
                program: "normal".to_string(),
                args: vec![],
                env: HashMap::default(),
                current_dir: None,
                timeout: None,
            },
            priority: CommandPriority::Normal,
            enqueued_at: now,
        });

        // Add low priority
        queue.push(QueuedCommand {
            id: "low".to_string(),
            command: Command {
                program: "low".to_string(),
                args: vec![],
                env: HashMap::default(),
                current_dir: None,
                timeout: None,
            },
            priority: CommandPriority::Low,
            enqueued_at: now + Duration::from_nanos(1),
        });

        // Add high priority
        queue.push(QueuedCommand {
            id: "high".to_string(),
            command: Command {
                program: "high".to_string(),
                args: vec![],
                env: HashMap::default(),
                current_dir: None,
                timeout: None,
            },
            priority: CommandPriority::High,
            enqueued_at: now + Duration::from_nanos(2),
        });

        // Add critical priority
        queue.push(QueuedCommand {
            id: "critical".to_string(),
            command: Command {
                program: "critical".to_string(),
                args: vec![],
                env: HashMap::default(),
                current_dir: None,
                timeout: None,
            },
            priority: CommandPriority::Critical,
            enqueued_at: now + Duration::from_nanos(3),
        });

        // Now pop them off and verify order
        let mut order = Vec::new();
        while let Some(cmd) = queue.pop() {
            order.push(cmd.command.program);
        }

        // Should be in priority order (critical, high, normal, low)
        assert_eq!(order.len(), 4, "Expected to get 4 commands");
        assert_eq!(order[0], "critical", "Critical should be first");
        assert_eq!(order[1], "high", "High should be second");
        assert_eq!(order[2], "normal", "Normal should be third");
        assert_eq!(order[3], "low", "Low should be last");
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::items_after_statements)]
    #[allow(clippy::print_stdout)]
    #[allow(clippy::assertions_on_constants)]
    #[allow(clippy::vec_init_then_push)]
    #[tokio::test]
    async fn test_command_queue_priority_execution() {
        // Set up a simple executor that records execution order
        let execution_order = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        struct RecordingExecutor {
            order: Arc<tokio::sync::Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl CommandExecutor for RecordingExecutor {
            async fn execute(&self, command: Command) -> Result<CommandOutput> {
                // Record execution order
                {
                    let mut order = self.order.lock().await;
                    order.push(command.program.clone());
                    println!("Executing: {} (current order: {:?})", command.program, *order);
                }

                // Ensure different commands have different timestamps
                tokio::time::sleep(Duration::from_millis(10)).await;

                Ok(CommandOutput::new(
                    0,
                    format!("Executed: {}", command.program),
                    String::new(),
                    Duration::from_millis(10),
                ))
            }

            async fn execute_stream(
                &self,
                _command: Command,
                _stream_config: StreamConfig,
            ) -> Result<(crate::command::CommandStream, tokio::process::Child)> {
                Err(Error::Command(CommandError::Generic("Not implemented".to_string())))
            }
        }

        // Create a queue with our recording executor
        let executor = Arc::new(RecordingExecutor { order: Arc::clone(&execution_order) });

        // Create a queue with a configuration that ensures predictable ordering
        let config = CommandQueueConfig {
            max_concurrent_commands: 1, // Process one at a time
            rate_limit: Some(Duration::from_millis(50)),
            default_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(5),
        };

        let queue = CommandQueue {
            config,
            executor,
            queue_sender: None,
            statuses: Arc::default(),
            results: Arc::default(),
            processor_handle: None,
            command_counter: Arc::default(),
        };

        // Start the queue
        let mut queue = queue.start().expect("Failed to start queue");

        // Wait to ensure queue is ready
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Prepare commands in specific order to test priority handling
        let cmd_low = Command {
            program: "low".to_string(),
            args: vec![],
            env: HashMap::default(),
            current_dir: None,
            timeout: None,
        };

        let cmd_normal = Command {
            program: "normal".to_string(),
            args: vec![],
            env: HashMap::default(),
            current_dir: None,
            timeout: None,
        };

        let cmd_high = Command {
            program: "high".to_string(),
            args: vec![],
            env: HashMap::default(),
            current_dir: None,
            timeout: None,
        };

        let cmd_critical = Command {
            program: "critical".to_string(),
            args: vec![],
            env: HashMap::default(),
            current_dir: None,
            timeout: None,
        };

        let mut batch = Vec::new();
        batch.push((cmd_normal, CommandPriority::Normal));
        batch.push((cmd_low, CommandPriority::Low));
        batch.push((cmd_high, CommandPriority::High));
        batch.push((cmd_critical, CommandPriority::Critical));

        println!("Enqueueing commands as a batch");
        let ids = queue.enqueue_batch(batch).await.expect("Failed to enqueue batch");

        let id_normal = &ids[0];
        let id_low = &ids[1];
        let id_high = &ids[2];
        let id_critical = &ids[3];

        // Enqueue them in specific order: normal, low, high, critical
        /*println!("Enqueueing normal priority command");
        let id_normal = queue
            .enqueue(cmd_normal, CommandPriority::Normal)
            .await
            .expect("Failed to enqueue normal command");

        println!("Enqueueing low priority command");
        let id_low = queue
            .enqueue(cmd_low, CommandPriority::Low)
            .await
            .expect("Failed to enqueue low command");

        println!("Enqueueing high priority command");
        let id_high = queue
            .enqueue(cmd_high, CommandPriority::High)
            .await
            .expect("Failed to enqueue high command");

        println!("Enqueueing critical priority command");
        let id_critical = queue
            .enqueue(cmd_critical, CommandPriority::Critical)
            .await
            .expect("Failed to enqueue critical command");

        // Wait for all commands to finish*/
        println!("Waiting for completion");
        queue.wait_for_completion().await.expect("Failed waiting for completion");

        // Check execution order
        let final_order = execution_order.lock().await;
        println!("Final execution order: {:?}", *final_order);

        assert_eq!(final_order.len(), 4, "Expected 4 commands to be executed");

        if final_order[0] != "critical"
            || final_order[1] != "high"
            || final_order[2] != "normal"
            || final_order[3] != "low"
        {
            println!("Incorrect order detected: {:?}", *final_order);
            println!("Expected: [critical, high, normal, low]");

            // Provide more diagnostic info
            for i in 0..4 {
                let status = match i {
                    0 => queue.get_status(id_critical),
                    1 => queue.get_status(id_high),
                    2 => queue.get_status(id_normal),
                    3 => queue.get_status(id_low),
                    _ => None,
                };
                println!("Command {i} status: {status:?}");
            }

            // Failing assertion with clear message
            assert!(false, "Commands were not executed in priority order");
        }

        // Shutdown
        queue.shutdown().await.expect("Failed to shutdown queue");
    }
}
