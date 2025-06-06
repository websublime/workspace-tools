use super::types::{Command, CommandBuilder};
use std::{collections::HashMap, path::Path, time::Duration};

impl CommandBuilder {
    /// Creates a new `CommandBuilder` instance.
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
        }
    }

    /// Builds the final `Command` instance.
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
    #[must_use]
    pub fn build(self) -> Command {
        Command {
            program: self.program,
            args: self.args,
            env: self.env,
            current_dir: self.current_dir,
            timeout: self.timeout,
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

    /// Sets an environment variable for the command.
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_standard_tools::command::CommandBuilder;
    ///
    /// let builder = CommandBuilder::new("npm")
    ///     .env("NODE_ENV", "production");
    /// ```
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}
