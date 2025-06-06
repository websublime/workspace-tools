//! # Command Types Definitions
//!
//! ## What
//! This file defines the core types used throughout the command module, including
//! command configurations, execution interfaces, output structures, and queue
//! management types. These types form the foundation for the command execution system.
//!
//! ## How
//! The types are organized into several categories:
//! - Command representation (`Command`, `CommandBuilder`)
//! - Command execution (`CommandExecutor`, `DefaultCommandExecutor`)
//! - Output handling (`CommandOutput`, `StreamOutput`)
//! - Queue management (`CommandQueue`, `QueuedCommand`, `CommandStatus`)
//! - Stream handling (`CommandStream`, `StreamConfig`)
//!
//! ## Why
//! A well-defined type system enables clear separation of concerns and ensures
//! type safety throughout the command execution and management process. These
//! types establish the contracts between different components of the system and
//! provide the necessary abstractions for flexible command processing.

use std::{
    process::Stdio,
    time::{Duration, Instant},
};

use crate::error::{CommandError, Error, Result};

use tokio::{
    process::{Child, Command},
    time::timeout,
};

use super::types::{
    Command as CmdConfig, CommandOutput, CommandStream, DefaultCommandExecutor, StreamConfig,
};

/// Trait for executing commands with various options.
///
/// This trait defines the contract for command executors, allowing
/// for both synchronous execution with full output capture and
/// streaming execution with real-time output access.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::{CommandExecutor, Command, CommandOutput};
/// use sublime_standard_tools::error::Result;
///
/// # struct MyExecutor;
/// # #[async_trait::async_trait]
/// # impl CommandExecutor for MyExecutor {
/// #     async fn execute(&self, command: Command) -> Result<CommandOutput> { unimplemented!() }
/// #     async fn execute_stream(
/// #         &self,
/// #         command: Command,
/// #         stream_config: crate::command::StreamConfig,
/// #     ) -> Result<(crate::command::CommandStream, tokio::process::Child)> { unimplemented!() }
/// # }
///
/// # async fn example() -> Result<()> {
/// let executor = MyExecutor;
/// let cmd = Command {
///     program: "echo".to_string(),
///     args: vec!["Hello".to_string()],
///     env: std::collections::HashMap::new(),
///     current_dir: None,
///     timeout: Some(std::time::Duration::from_secs(5)),
/// };
///
/// let output = executor.execute(cmd).await?;
/// println!("Command output: {}", output.stdout());
/// # Ok(())
/// # }
/// ``
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    /// Executes a command and returns its output.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    ///
    /// # Returns
    ///
    /// * `Ok(CommandOutput)` - If the command executed successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute or times out.
    /// * `Err(Error)` - If the command failed to execute
    async fn execute(&self, command: CmdConfig) -> Result<CommandOutput>;

    /// Executes a command and returns a stream of its output.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `stream_config` - Configuration for the output stream
    ///
    /// # Returns
    ///
    /// * `Ok((CommandStream, Child))` - If the command was started successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start.
    /// * `Err(Error)` - If the command failed to start
    async fn execute_stream(
        &self,
        command: CmdConfig,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, Child)>;
}

impl DefaultCommandExecutor {
    /// Creates a new `DefaultCommandExecutor`.
    ///
    /// # Returns
    ///
    /// A new executor instance ready for executing commands
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::DefaultCommandExecutor;
    ///
    /// let executor = DefaultCommandExecutor::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Builds a tokio Command from our command configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The command configuration
    ///
    /// # Returns
    ///
    /// A tokio `Command` configured based on the inputs
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::{DefaultCommandExecutor, Command};
    /// #
    /// # fn example() {
    /// let config = Command {
    ///     program: "echo".to_string(),
    ///     args: vec!["hello".to_string()],
    ///     env: std::collections::HashMap::new(),
    ///     current_dir: None,
    ///     timeout: Some(std::time::Duration::from_secs(5)),
    /// };
    ///
    /// let tokio_cmd = DefaultCommandExecutor::build_command(&config);
    /// # }
    /// ```
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
impl Executor for DefaultCommandExecutor {
    /// Executes a command and returns its complete output.
    ///
    /// This method spawns the command, waits for it to complete with
    /// a timeout, and returns its output including stdout, stderr,
    /// exit code, and execution duration.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    ///
    /// # Returns
    ///
    /// * `Ok(CommandOutput)` - Command output if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute, times out, or returns a non-zero exit code.
    /// * `Err(Error)` - If the command failed to execute
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::{DefaultCommandExecutor, CommandExecutor, CommandBuilder};
    /// # use sublime_standard_tools::error::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// let executor = DefaultCommandExecutor::new();
    ///
    /// let cmd = CommandBuilder::new("echo")
    ///     .arg("Hello, world!")
    ///     .timeout(std::time::Duration::from_secs(5))
    ///     .build();
    ///
    /// let output = executor.execute(cmd).await?;
    /// println!("Command output: {}", output.stdout());
    /// # Ok(())
    /// # }
    /// ```
    async fn execute(&self, command: CmdConfig) -> Result<CommandOutput> {
        let start_time = Instant::now();
        let mut cmd = Self::build_command(&command);
        let cmd_str = command.program.clone(); // For error reporting

        let timeout_duration = command.timeout.unwrap_or(Duration::from_secs(30));
        let child = cmd.spawn().map_err(|e| {
            Error::Command(CommandError::SpawnFailed { cmd: cmd_str.clone(), source: e })
        })?;

        // Get PID before potentially consuming child with wait_with_output
        let child_pid = child.id();

        // Wait for completion with timeout
        let output_result = timeout(timeout_duration, child.wait_with_output()).await;

        let output = match output_result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(Error::Command(CommandError::ExecutionFailed {
                    cmd: cmd_str,
                    source: Some(e),
                }));
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
                return Err(Error::Command(CommandError::Timeout { duration: timeout_duration }));
            }
        };

        let duration = start_time.elapsed();
        let code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if !output.status.success() {
            return Err(Error::Command(CommandError::NonZeroExitCode {
                cmd: cmd_str,
                code,
                stderr,
            }));
        }

        Ok(CommandOutput::new(code.unwrap_or(0), stdout, stderr, duration))
    }

    /// Executes a command and provides a stream of its output.
    ///
    /// This method spawns the command and returns a stream that allows
    /// reading stdout and stderr lines as they are produced, rather than
    /// waiting for the command to complete.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `stream_config` - Configuration for the output stream
    ///
    /// # Returns
    ///
    /// * `Ok((CommandStream, Child))` - The output stream and process handle
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start or if stdout/stderr cannot be captured.
    /// * `Err(Error)` - If the command failed to start
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::{DefaultCommandExecutor, CommandExecutor, CommandBuilder, StreamConfig};
    /// # use std::time::Duration;
    /// # use sublime_standard_tools::error::Result;
    /// #
    /// # async fn example() -> Result<()> {
    /// let executor = DefaultCommandExecutor::new();
    /// let config = StreamConfig::default();
    ///
    /// let cmd = CommandBuilder::new("ls")
    ///     .arg("-la")
    ///     .build();
    ///
    /// let (mut stream, mut child) = executor.execute_stream(cmd, config).await?;
    ///
    /// // Read output lines as they arrive
    /// while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
    ///     println!("Got: {:?}", output);
    /// }
    ///
    /// // Wait for the process to complete
    /// let status = child.wait().await?;
    /// println!("Process exited with: {}", status);
    /// # Ok(())
    /// # }
    /// ```
    async fn execute_stream(
        &self,
        command: CmdConfig,
        stream_config: StreamConfig,
    ) -> Result<(CommandStream, Child)> {
        let mut cmd = Self::build_command(&command);
        let cmd_str = command.program.clone(); // For error reporting

        let mut child = cmd
            .spawn()
            .map_err(|e| Error::Command(CommandError::SpawnFailed { cmd: cmd_str, source: e }))?;

        let stdout = child
            .stdout
            .take()
            .ok_or(Error::Command(CommandError::CaptureFailed { stream: "stdout".to_string() }))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(Error::Command(CommandError::CaptureFailed { stream: "stderr".to_string() }))?;

        let stream = CommandStream::new(stdout, stderr, &stream_config);

        Ok((stream, child)) // Return the child process handle along with the stream
    }
}
