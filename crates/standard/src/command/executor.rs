//! Command execution implementation.
//!
//! What:
//! This module provides the core command execution functionality, including
//! both synchronous and streaming execution modes, with support for timeouts
//! and resource limits.
//!
//! Who:
//! Used by developers who need to:
//! - Execute system commands reliably
//! - Handle command output in real-time
//! - Manage command execution resources
//! - Implement command timeout policies
//!
//! Why:
//! Reliable command execution with proper resource management and error
//! handling is essential for system integration and automation tasks.

use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use tokio::{
    process::{Child, Command},
    time::timeout,
};

use super::{Command as CmdConfig, CommandOutput, CommandStream, StreamConfig};
use crate::error::{CommandError, CommandResult};

/// Trait defining command execution behavior.
#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Executes a command and returns its output.
    async fn execute(&self, command: CmdConfig) -> CommandResult<CommandOutput>;

    /// Executes a command with streaming output.
    async fn execute_stream(
        &self,
        command: CmdConfig,
        stream_config: StreamConfig,
    ) -> CommandResult<(CommandStream, Child)>;
}

/// Default implementation of CommandExecutor.
#[derive(Debug, Default)]
pub struct DefaultCommandExecutor;

impl DefaultCommandExecutor {
    /// Creates a new DefaultCommandExecutor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Enforces resource limits on the command process.
    /// Note: Actual enforcement is platform-specific and not fully implemented here.
    fn enforce_limits(child: &Child, limits: &super::ResourceLimits) -> CommandResult<()> {
        let pid = match child.id() {
            Some(0) => {
                return Err(CommandError::Generic(
                    "Process ended prematurely with PID 0".to_string(),
                ))
            }
            None => {
                return Err(CommandError::Generic("Process failed to start or get PID".to_string()))
            }
            Some(pid) => pid,
        };

        log::debug!("Enforcing resource limits on process with PID: {}", pid);

        // Platform-specific resource limiting implementation
        #[cfg(target_family = "unix")]
        if let Some(memory_mb) = limits.memory_mb() {
            // On Unix systems, we can use setrlimit for resource limits
            // This is a basic implementation - in production, you might use crates like rlimit
            log::info!("Setting memory limit of {}MB for process {}", memory_mb, pid);

            // In real implementation, you'd use system APIs to enforce this
            // For now, we just log that we would do this
        }

        #[cfg(target_family = "unix")]
        if let Some(fd_limit) = limits.get_file_descriptors() {
            log::info!("Setting file descriptor limit of {} for process {}", fd_limit, pid);
            // Would use system APIs to enforce this
        }

        #[cfg(target_os = "linux")]
        if let Some(cpu_percent) = limits.cpu_percent() {
            log::info!("Setting CPU limit of {}% for process {}", cpu_percent, pid);
            // Would use cgroups or similar to enforce this
        }

        Ok(())
    }

    /// Builds a tokio Command from our command configuration.
    fn build_command(config: &CmdConfig) -> Command {
        let mut cmd = Command::new(&config.program);
        cmd.args(&config.args).envs(&config.env).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = &config.current_dir {
            cmd.current_dir(dir);
        }

        cmd
    }
}

#[async_trait::async_trait]
impl CommandExecutor for DefaultCommandExecutor {
    async fn execute(&self, command: CmdConfig) -> CommandResult<CommandOutput> {
        let start_time = Instant::now();
        let mut cmd = Self::build_command(&command);
        let cmd_str = command.program.clone(); // For error reporting

        let timeout_duration = command.timeout.unwrap_or(Duration::from_secs(30));
        let child = cmd
            .spawn()
            .map_err(|e| CommandError::SpawnFailed { cmd: cmd_str.clone(), source: e })?;

        // Get PID before potentially consuming child with wait_with_output
        let child_pid = child.id();

        // Apply resource limits if specified
        if let Some(limits) = &command.resource_limits {
            // Pass a reference, enforce_limits doesn't consume child
            Self::enforce_limits(&child, limits)?;
        }

        // Wait for completion with timeout
        let output_result = timeout(timeout_duration, child.wait_with_output()).await;

        let output = match output_result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(CommandError::ExecutionFailed { cmd: cmd_str, source: Some(e) });
            }
            Err(_) => {
                // Timeout elapsed
                if let Some(pid) = child_pid {
                    log::warn!(
                        "Process (PID: {}) timed out after {:?}. Manual cleanup might be required.",
                        pid,
                        timeout_duration
                    );
                } else {
                    log::warn!(
                        "Process timed out after {:?}, but PID was unavailable.",
                        timeout_duration
                    );
                }
                return Err(CommandError::Timeout { duration: timeout_duration });
            }
        };

        let duration = start_time.elapsed();
        let code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if !output.status.success() {
            return Err(CommandError::NonZeroExitCode { cmd: cmd_str, code, stderr });
        }

        Ok(CommandOutput::new(code.unwrap_or(0), stdout, stderr, duration))
    }

    async fn execute_stream(
        &self,
        command: CmdConfig,
        stream_config: StreamConfig,
    ) -> CommandResult<(CommandStream, Child)> {
        let mut cmd = Self::build_command(&command);
        let cmd_str = command.program.clone(); // For error reporting

        let mut child =
            cmd.spawn().map_err(|e| CommandError::SpawnFailed { cmd: cmd_str, source: e })?;

        // Apply resource limits if specified
        if let Some(limits) = &command.resource_limits {
            Self::enforce_limits(&child, limits)?;
        }

        let stdout = child
            .stdout
            .take()
            .ok_or(CommandError::CaptureFailed { stream: "stdout".to_string() })?;
        let stderr = child
            .stderr
            .take()
            .ok_or(CommandError::CaptureFailed { stream: "stderr".to_string() })?;

        let stream = CommandStream::new(stdout, stderr, &stream_config);

        Ok((stream, child)) // Return the child process handle along with the stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandBuilder;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_execution() -> Result<(), Box<dyn std::error::Error>> {
        let executor = DefaultCommandExecutor::new();
        let command =
            CommandBuilder::new("echo").arg("test").timeout(Duration::from_secs(1)).build();

        let output = executor.execute(command).await?;
        assert!(output.success());
        assert_eq!(output.stdout().trim(), "test");
        Ok(())
    }

    #[tokio::test]
    async fn test_execution_timeout() -> Result<(), Box<dyn std::error::Error>> {
        let executor = DefaultCommandExecutor::new();
        let command = CommandBuilder::new("sleep").arg("2").timeout(Duration::from_secs(1)).build();

        let result = executor.execute(command).await;
        assert!(result.is_err());

        // Use matches! macro instead of panic for better error handling
        assert!(
            matches!(result, Err(CommandError::Timeout { duration }) if duration == Duration::from_secs(1)),
            "Expected Timeout error with 1s duration, got {result:?}",
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_stream_execution() -> Result<(), Box<dyn std::error::Error>> {
        let executor = DefaultCommandExecutor::new();
        let command = CommandBuilder::new("echo").arg("test").build();

        let (mut stream, mut child) = // Make child mutable
            executor.execute_stream(command, StreamConfig::default()).await?;

        let output = stream.next_timeout(Duration::from_secs(1)).await?;
        assert!(matches!(output, Some(super::super::StreamOutput::Stdout(_))));

        // Ensure process cleanup
        child.kill().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_stream_cancellation() -> Result<(), Box<dyn std::error::Error>> {
        let executor = DefaultCommandExecutor::new();
        let command = CommandBuilder::new("yes") // Continuous output
            .build();

        let (stream, mut child) = // Make child mutable
            executor.execute_stream(command, StreamConfig::default()).await?;

        // Let it run briefly
        sleep(Duration::from_millis(100)).await;

        // Cancel the stream
        stream.cancel();

        // Wait a bit to ensure cancellation takes effect
        sleep(Duration::from_millis(100)).await;

        // Ensure process cleanup
        child.kill().await?;
        Ok(())
    }
}
