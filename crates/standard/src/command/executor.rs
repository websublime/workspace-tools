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
    time::Instant,
    path::Path,
};

use crate::config::{ConfigManager, StandardConfig, traits::Configurable};
use crate::error::{CommandError, Error, Result};
use crate::filesystem::{FileSystemManager, AsyncFileSystem};

use tokio::{
    process::{Child, Command},
    time::timeout,
};

use super::types::{
    Command as CmdConfig, CommandOutput, CommandStream, DefaultCommandExecutor,
    GlobalExecutorState, SharedSyncExecutor, StreamConfig, SyncCommandExecutor,
};
use crate::config::CommandConfig;

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
/// ```
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
    /// Creates a new `DefaultCommandExecutor` with default configuration.
    ///
    /// # Returns
    ///
    /// A new executor instance using default command configuration
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
        Self {
            config: CommandConfig::default(),
        }
    }

    /// Creates a new `DefaultCommandExecutor` with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The command configuration to use
    ///
    /// # Returns
    ///
    /// A new executor instance using the provided configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::DefaultCommandExecutor;
    /// use sublime_standard_tools::config::CommandConfig;
    /// use std::time::Duration;
    ///
    /// let config = CommandConfig { 
    ///     default_timeout: Duration::from_secs(120), 
    ///     ..CommandConfig::default()
    /// };
    /// let executor = DefaultCommandExecutor::new_with_config(config);
    /// ```
    #[must_use]
    pub fn new_with_config(config: CommandConfig) -> Self {
        Self { config }
    }

    /// Creates a new `DefaultCommandExecutor` that automatically loads configuration from project files.
    ///
    /// This method searches for configuration files (repo.config.*) in the specified path and
    /// loads the command configuration from them. If no config files are found, it uses
    /// default configuration with environment variable overrides.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The path to search for configuration files
    ///
    /// # Returns
    ///
    /// * `Ok(DefaultCommandExecutor)` - An executor with loaded configuration
    /// * `Err(Error)` - If configuration loading fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::DefaultCommandExecutor;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let executor = DefaultCommandExecutor::new_with_project_config(Path::new(".")).await?;
    /// // Configuration loaded from repo.config.toml/yml/json or defaults + env vars
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files exist but cannot be parsed.
    pub async fn new_with_project_config(project_root: &Path) -> Result<Self> {
        let config = Self::load_project_config(project_root, None).await?;
        
        Ok(Self {
            config: config.commands,
        })
    }

    /// Loads configuration from project files in the specified directory.
    ///
    /// This method searches for configuration files in the following order:
    /// - repo.config.toml
    /// - repo.config.yml/yaml  
    /// - repo.config.json
    ///
    /// # Arguments
    ///
    /// * `project_root` - The directory to search for configuration files
    /// * `base_config` - Optional base configuration to merge with
    ///
    /// # Returns
    ///
    /// * `Ok(StandardConfig)` - The loaded and merged configuration
    /// * `Err(Error)` - If configuration loading fails
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files exist but cannot be parsed.
    async fn load_project_config(
        project_root: &Path,
        base_config: Option<StandardConfig>,
    ) -> Result<StandardConfig> {
        let fs = FileSystemManager::new();
        let mut builder = ConfigManager::<StandardConfig>::builder().with_defaults();

        // Check for repo.config.* files in order of preference
        let config_files = [
            project_root.join("repo.config.toml"),
            project_root.join("repo.config.yml"), 
            project_root.join("repo.config.yaml"),
            project_root.join("repo.config.json"),
        ];

        // Add existing config files to the builder
        for config_file in &config_files {
            if fs.exists(config_file).await {
                builder = builder.with_file(config_file);
            }
        }

        let manager = builder.build(fs).map_err(|e| {
            Error::operation(format!("Failed to create config manager: {e}"))
        })?;

        let mut config = manager.load().await.map_err(|e| {
            Error::operation(format!("Failed to load configuration: {e}"))
        })?;

        // Merge with base config if provided
        if let Some(base) = base_config {
            config.merge_with(base).map_err(|e| {
                Error::operation(format!("Failed to merge configurations: {e}"))
            })?;
        }

        Ok(config)
    }

    /// Gets the current command configuration.
    ///
    /// # Returns
    ///
    /// A reference to the command configuration
    #[must_use]
    pub fn config(&self) -> &CommandConfig {
        &self.config
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

        let timeout_duration = command.timeout.unwrap_or(self.config.default_timeout);
        let child = cmd.spawn().map_err(|e| {
            Error::Command(CommandError::SpawnFailed {
                cmd: cmd_str.clone(),
                message: e.to_string(),
            })
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
                    message: e.to_string(),
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

        let mut child = cmd.spawn().map_err(|e| {
            Error::Command(CommandError::SpawnFailed { cmd: cmd_str, message: e.to_string() })
        })?;

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

/// Implementation for synchronous command executor
impl SyncCommandExecutor {
    /// Create a new synchronous command executor with default configuration
    ///
    /// Creates a dedicated Tokio runtime for async operations.
    /// This runtime is isolated and doesn't interfere with other
    /// async contexts in the application.
    ///
    /// # Returns
    ///
    /// A new synchronous command executor with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created.
    pub fn new() -> Result<Self> {
        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
                Error::Command(CommandError::Generic(format!(
                    "Failed to create runtime for sync executor: {e}"
                )))
            })?;

        Ok(Self { runtime, executor: DefaultCommandExecutor::new() })
    }

    /// Create a new synchronous command executor with custom configuration
    ///
    /// Creates a dedicated Tokio runtime for async operations using the
    /// provided command configuration for timeout and execution settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The command configuration to use
    ///
    /// # Returns
    ///
    /// A new synchronous command executor with the provided configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created.
    pub fn new_with_config(config: CommandConfig) -> Result<Self> {
        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
                Error::Command(CommandError::Generic(format!(
                    "Failed to create runtime for sync executor: {e}"
                )))
            })?;

        Ok(Self { runtime, executor: DefaultCommandExecutor::new_with_config(config) })
    }

    /// Create a new synchronous command executor that automatically loads configuration from project files.
    ///
    /// This method searches for configuration files (repo.config.*) in the specified path and
    /// loads the command configuration from them. If no config files are found, it uses
    /// default configuration with environment variable overrides.
    ///
    /// # Arguments
    ///
    /// * `project_root` - The path to search for configuration files
    ///
    /// # Returns
    ///
    /// A new synchronous command executor with loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the Tokio runtime cannot be created or configuration loading fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::SyncCommandExecutor;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let executor = SyncCommandExecutor::new_with_project_config(Path::new("."))?;
    /// // Configuration loaded from repo.config.toml/yml/json or defaults + env vars
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_project_config(project_root: &Path) -> Result<Self> {
        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
                Error::Command(CommandError::Generic(format!(
                    "Failed to create runtime for sync executor: {e}"
                )))
            })?;

        // Load configuration using the runtime
        let config = runtime.block_on(async {
            DefaultCommandExecutor::load_project_config(project_root, None).await
        })?;

        Ok(Self { 
            runtime, 
            executor: DefaultCommandExecutor::new_with_config(config.commands) 
        })
    }

    /// Execute a command synchronously
    ///
    /// Executes the command using the underlying async executor
    /// but blocks until completion, providing a synchronous interface.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    ///
    /// # Returns
    ///
    /// Command output with stdout, stderr, and exit status.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::{SyncCommandExecutor, CommandBuilder};
    /// use sublime_standard_tools::error::Result;
    ///
    /// # fn example() -> Result<()> {
    /// let executor = SyncCommandExecutor::new()?;
    /// let command = CommandBuilder::new("git").args(&["status", "--porcelain"]).build();
    /// let output = executor.execute_sync(command)?;
    ///
    /// if output.status() == 0 {
    ///     println!("Git status: {}", output.stdout());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_sync(&self, command: CmdConfig) -> Result<CommandOutput> {
        self.runtime.block_on(self.executor.execute(command))
    }

    /// Execute a command synchronously with timeout
    ///
    /// Similar to `execute_sync` but with an explicit timeout.
    /// The command will be terminated if it exceeds the timeout.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    /// * `timeout` - Maximum execution time
    ///
    /// # Returns
    ///
    /// Command output or timeout error.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails or times out.
    pub fn execute_sync_with_timeout(
        &self,
        command: CmdConfig,
        timeout: std::time::Duration,
    ) -> Result<CommandOutput> {
        self.runtime.block_on(async {
            tokio::time::timeout(timeout, self.executor.execute(command))
                .await
                .map_err(|_| Error::Command(CommandError::Timeout { duration: timeout }))?
        })
    }

    /// Get runtime handle for advanced usage
    ///
    /// Provides access to the underlying runtime handle for cases
    /// where manual async/sync bridging is needed.
    ///
    /// # Returns
    ///
    /// Handle to the underlying Tokio runtime.
    pub fn runtime_handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }
}

/// We cannot implement Default for SyncCommandExecutor following NUNCA rules
/// because runtime creation can fail and Default trait cannot return errors.
/// Use SyncCommandExecutor::new() instead for proper error handling.
///
/// This implementation has been removed to enforce proper error handling patterns.

/// We cannot implement Clone for SyncCommandExecutor following NUNCA rules
/// because runtime creation can fail and Clone trait cannot return errors.
/// Use SyncCommandExecutor::new() instead for proper error handling.
///
/// This implementation has been removed to enforce proper error handling patterns.

/// Implementation for shared synchronous executor
impl SharedSyncExecutor {
    /// Get the shared synchronous executor instance with default configuration
    ///
    /// Creates the shared instance on first access using default configuration.
    /// This method properly handles creation errors and returns them instead of
    /// panicking. This follows the NUNCA rules - errors are handled, never
    /// ignored with panics or unsafe code.
    ///
    /// # Returns
    ///
    /// Shared synchronous command executor or creation error.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying SyncCommandExecutor cannot be created.
    pub fn try_instance() -> Result<&'static SharedSyncExecutor> {
        Self::try_instance_with_config(CommandConfig::default())
    }

    /// Get the shared synchronous executor instance with custom configuration
    ///
    /// Creates the shared instance on first access using the provided configuration.
    /// This method properly handles creation errors and returns them instead of
    /// panicking. This follows the NUNCA rules - errors are handled, never
    /// ignored with panics or unsafe code.
    ///
    /// # Arguments
    ///
    /// * `config` - The command configuration to use for the shared instance
    ///
    /// # Returns
    ///
    /// Shared synchronous command executor or creation error.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying SyncCommandExecutor cannot be created.
    pub fn try_instance_with_config(config: CommandConfig) -> Result<&'static SharedSyncExecutor> {
        use std::sync::{Arc, Mutex, OnceLock};

        // Use a global state that stores either success or error
        static GLOBAL_STATE: OnceLock<Mutex<GlobalExecutorState>> = OnceLock::new();

        let state_lock =
            GLOBAL_STATE.get_or_init(|| Mutex::new(GlobalExecutorState::Uninitialized));

        let mut guard = state_lock.lock().map_err(|_| {
            Error::Command(crate::error::CommandError::Generic(
                "Failed to acquire lock for shared sync executor".to_string(),
            ))
        })?;

        match &*guard {
            GlobalExecutorState::Success(executor) => Ok(executor),
            GlobalExecutorState::Error(error) => Err(error.clone()),
            GlobalExecutorState::Uninitialized => {
                // First access - try to create the instance with provided config
                match SyncCommandExecutor::new_with_config(config) {
                    Ok(sync_executor) => {
                        let shared_executor =
                            SharedSyncExecutor { executor: Arc::new(sync_executor) };

                        // Store the success state
                        *guard = GlobalExecutorState::Success(Box::leak(Box::new(shared_executor)));

                        // Return the reference
                        if let GlobalExecutorState::Success(executor) = &*guard {
                            Ok(executor)
                        } else {
                            // This should never happen, but handle it gracefully
                            Err(Error::Command(crate::error::CommandError::Generic(
                                "Internal state corruption in shared sync executor".to_string(),
                            )))
                        }
                    }
                    Err(e) => {
                        // Store the error for future calls
                        *guard = GlobalExecutorState::Error(e.clone());
                        Err(e)
                    }
                }
            }
        }
    }

    /// Get the shared synchronous executor instance (DEPRECATED)
    ///
    /// This method has been removed following NUNCA rules - no panics allowed.
    /// Use `try_instance()` instead for proper error handling.
    ///
    /// This method has been removed to enforce proper error handling patterns.

    /// Execute a command using the shared executor
    ///
    /// Convenience method for executing commands with the shared instance.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    ///
    /// # Returns
    ///
    /// Command output.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    pub fn execute(&self, command: CmdConfig) -> Result<CommandOutput> {
        self.executor.execute_sync(command)
    }

    /// Execute a command with timeout using the shared executor
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    /// * `timeout` - Maximum execution time
    ///
    /// # Returns
    ///
    /// Command output or timeout error.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails or times out.
    pub fn execute_with_timeout(
        &self,
        command: CmdConfig,
        timeout: std::time::Duration,
    ) -> Result<CommandOutput> {
        self.executor.execute_sync_with_timeout(command, timeout)
    }
}
