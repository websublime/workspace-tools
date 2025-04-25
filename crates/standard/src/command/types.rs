//! Type definitions for command execution.
//!
//! What:
//! This module defines the core types used for command execution, including
//! command configuration, output handling, and resource management.
//!
//! Who:
//! Used by developers working with the command execution system who need to:
//! - Configure command execution parameters
//! - Handle command output
//! - Manage resource limits
//!
//! Why:
//! Well-defined types ensure type safety and provide a clear interface for
//! command execution functionality.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

/// Resource limits for command execution.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::command::ResourceLimits;
///
/// let limits = ResourceLimits::new()
///     .memory_limit(1024) // 1GB
///     .cpu_limit(50)      // 50%
///     .file_descriptors(1000);
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceLimits {
    /// Maximum memory usage in megabytes
    memory_mb: Option<u64>,
    /// Maximum CPU usage percentage (0-100)
    cpu_percent: Option<u8>,
    /// Maximum number of file descriptors
    file_descriptors: Option<u32>,
}

impl ResourceLimits {
    /// Creates a new ResourceLimits instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::ResourceLimits;
    ///
    /// let limits = ResourceLimits::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { memory_mb: None, cpu_percent: None, file_descriptors: None }
    }

    /// Sets the memory limit in megabytes.
    ///
    /// # Arguments
    ///
    /// * `limit` - Memory limit in megabytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::ResourceLimits;
    ///
    /// let limits = ResourceLimits::new().memory_limit(1024); // 1GB limit
    /// ```
    #[must_use]
    pub fn memory_limit(mut self, limit: u64) -> Self {
        self.memory_mb = Some(limit);
        self
    }

    /// Sets the CPU usage limit as a percentage.
    ///
    /// # Arguments
    ///
    /// * `limit` - CPU usage limit (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::ResourceLimits;
    ///
    /// let limits = ResourceLimits::new().cpu_limit(50); // 50% CPU limit
    /// ```
    #[must_use]
    pub fn cpu_limit(mut self, limit: u8) -> Self {
        assert!(limit <= 100, "CPU limit must be between 0 and 100");
        self.cpu_percent = Some(limit);
        self
    }

    /// Sets the file descriptor limit.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of file descriptors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::ResourceLimits;
    ///
    /// let limits = ResourceLimits::new().file_descriptors(1000);
    /// ```
    #[must_use]
    pub fn file_descriptors(mut self, limit: u32) -> Self {
        self.file_descriptors = Some(limit);
        self
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Command execution output.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::command::CommandOutput;
/// use std::time::Duration;
///
/// let output = CommandOutput::new(
///     0,
///     "stdout content".to_string(),
///     "stderr content".to_string(),
///     Duration::from_secs(1),
/// );
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandOutput {
    /// Exit status code
    status: i32,
    /// Standard output content
    stdout: String,
    /// Standard error content
    stderr: String,
    /// Command execution duration
    duration: Duration,
}

impl CommandOutput {
    /// Creates a new CommandOutput instance.
    ///
    /// # Arguments
    ///
    /// * `status` - Exit status code
    /// * `stdout` - Standard output content
    /// * `stderr` - Standard error content
    /// * `duration` - Command execution duration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandOutput;
    /// use std::time::Duration;
    ///
    /// let output = CommandOutput::new(
    ///     0,
    ///     "command output".to_string(),
    ///     "".to_string(),
    ///     Duration::from_secs(1),
    /// );
    /// ```
    #[must_use]
    pub fn new(status: i32, stdout: String, stderr: String, duration: Duration) -> Self {
        Self { status, stdout, stderr, duration }
    }

    /// Returns the exit status code.
    #[must_use]
    pub fn status(&self) -> i32 {
        self.status
    }

    /// Returns the standard output content.
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Returns the standard error content.
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Returns the command execution duration.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns true if the command was successful (exit code 0).
    #[must_use]
    pub fn success(&self) -> bool {
        self.status == 0
    }
}

/// Command configuration builder.
///
/// # Examples
///
/// ```rust
/// use sublime_standard_tools::command::{Command, CommandBuilder};
/// use std::time::Duration;
///
/// let cmd = CommandBuilder::new("npm")
///     .arg("install")
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
    /// Resource limits
    pub(crate) resource_limits: Option<ResourceLimits>,
}

/// Builder for Command configuration.
#[derive(Debug)]
pub struct CommandBuilder {
    program: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    current_dir: Option<PathBuf>,
    timeout: Option<Duration>,
    resource_limits: Option<ResourceLimits>,
}

impl CommandBuilder {
    /// Creates a new CommandBuilder instance.
    ///
    /// # Arguments
    ///
    /// * `program` - The program to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    ///
    /// let builder = CommandBuilder::new("npm");
    /// ```
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
            current_dir: None,
            timeout: None,
            resource_limits: None,
        }
    }

    /// Builds the final Command instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("npm")
    ///     .arg("install")
    ///     .build();
    /// ```
    pub fn build(self) -> Command {
        Command {
            program: self.program,
            args: self.args,
            env: self.env,
            current_dir: self.current_dir,
            timeout: self.timeout,
            resource_limits: self.resource_limits,
        }
    }

    /// Adds an argument to the command.
    ///
    /// # Arguments
    ///
    /// * `arg` - Command argument to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    ///
    /// let builder = CommandBuilder::new("npm").arg("install");
    /// ```
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Sets the command timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Command execution timeout duration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = CommandBuilder::new("npm")
    ///     .timeout(Duration::from_secs(60));
    /// ```
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the working directory for the command.
    ///
    /// # Arguments
    ///
    /// * `path` - Working directory path
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    ///
    /// let builder = CommandBuilder::new("npm")
    ///     .current_dir("/path/to/project");
    /// ```
    #[must_use]
    pub fn current_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.current_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the resource limits for the command.
    ///
    /// # Arguments
    ///
    /// * `limits` - Resource limits configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::{CommandBuilder, ResourceLimits};
    ///
    /// let limits = ResourceLimits::new().memory_limit(1024);
    /// let builder = CommandBuilder::new("npm")
    ///     .resource_limits(limits);
    /// ```
    #[must_use]
    pub fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = Some(limits);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::new().memory_limit(1024).cpu_limit(50).file_descriptors(1000);

        assert_eq!(limits.memory_mb, Some(1024));
        assert_eq!(limits.cpu_percent, Some(50));
        assert_eq!(limits.file_descriptors, Some(1000));
    }

    #[test]
    #[should_panic(expected = "CPU limit must be between 0 and 100")]
    fn test_resource_limits_cpu_validation() {
        let _ = ResourceLimits::new().cpu_limit(101);
    }

    #[test]
    fn test_command_output() {
        let duration = Duration::from_secs(1);
        let output = CommandOutput::new(0, "stdout".to_string(), "stderr".to_string(), duration);

        assert!(output.success());
        assert_eq!(output.stdout(), "stdout");
        assert_eq!(output.stderr(), "stderr");
        assert_eq!(output.duration(), duration);
    }

    #[test]
    fn test_command_builder() {
        let cmd = CommandBuilder::new("npm")
            .arg("install")
            .current_dir("/tmp")
            .timeout(Duration::from_secs(60))
            .resource_limits(ResourceLimits::new().memory_limit(1024))
            .build();

        assert_eq!(cmd.program, "npm");
        assert_eq!(cmd.args, vec!["install"]);
        assert_eq!(cmd.current_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(cmd.timeout, Some(Duration::from_secs(60)));
        assert!(cmd.resource_limits.is_some());
    }
}
