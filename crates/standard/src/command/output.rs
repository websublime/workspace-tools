//! # Command Builder Implementation
//!
//! ## What
//! This file implements the `CommandBuilder` struct, providing a fluent interface
//! for constructing Command objects with various configuration options.
//!
//! ## How
//! The implementation uses the builder pattern to provide a clean, method-chaining
//! API for setting command properties like arguments, environment variables,
//! working directory, and timeout values. Each method returns the builder itself
//! to enable chaining, and the `build()` method finalizes the configuration into a
//! Command object.
//!
//! ## Why
//! Creating commands often requires setting multiple configuration options, which
//! can lead to verbose and error-prone code. The builder pattern implemented here
//! provides a more ergonomic and readable approach to command construction while
//! ensuring all necessary properties are properly initialized.

use super::types::CommandOutput;
use std::time::Duration;

impl CommandOutput {
    /// Creates a new `CommandOutput` instance.
    ///
    /// # Arguments
    ///
    /// * `status` - The command's exit status code
    /// * `stdout` - The command's standard output
    /// * `stderr` - The command's standard error output
    /// * `duration` - The time taken to execute the command
    ///
    /// # Returns
    ///
    /// A new `CommandOutput` with the provided values
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::command::CommandOutput;
    /// use std::time::Duration;
    ///
    /// let output = CommandOutput::new(
    ///     0,
    ///     "Operation successful".to_string(),
    ///     "".to_string(),
    ///     Duration::from_millis(425)
    /// );
    /// ```
    #[must_use]
    pub fn new(status: i32, stdout: String, stderr: String, duration: Duration) -> Self {
        Self { status, stdout, stderr, duration }
    }

    /// Returns the exit status code.
    ///
    /// # Returns
    ///
    /// The command's exit status code
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::CommandOutput;
    /// # use std::time::Duration;
    /// #
    /// let output = CommandOutput::new(
    ///     0,
    ///     "success".to_string(),
    ///     "".to_string(),
    ///     Duration::from_secs(1)
    /// );
    ///
    /// assert_eq!(output.status(), 0);
    /// ```
    #[must_use]
    pub fn status(&self) -> i32 {
        self.status
    }

    /// Returns the standard output content.
    ///
    /// # Returns
    ///
    /// The command's stdout content as a string slice
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::CommandOutput;
    /// # use std::time::Duration;
    /// #
    /// let output = CommandOutput::new(
    ///     0,
    ///     "Hello, world!".to_string(),
    ///     "".to_string(),
    ///     Duration::from_secs(1)
    /// );
    ///
    /// assert_eq!(output.stdout(), "Hello, world!");
    /// ```
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Returns the standard error content.
    ///
    /// # Returns
    ///
    /// The command's stderr content as a string slice
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::CommandOutput;
    /// # use std::time::Duration;
    /// #
    /// let output = CommandOutput::new(
    ///     1,
    ///     "".to_string(),
    ///     "Error: file not found".to_string(),
    ///     Duration::from_secs(1)
    /// );
    ///
    /// assert_eq!(output.stderr(), "Error: file not found");
    /// ```
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Returns the command execution duration.
    ///
    /// # Returns
    ///
    /// The time taken to execute the command
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::CommandOutput;
    /// # use std::time::Duration;
    /// #
    /// let duration = Duration::from_secs(2);
    /// let output = CommandOutput::new(
    ///     0,
    ///     "".to_string(),
    ///     "".to_string(),
    ///     duration
    /// );
    ///
    /// assert_eq!(output.duration(), duration);
    /// ```
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns true if the command was successful (exit code 0).
    ///
    /// # Returns
    ///
    /// True if the command exited with status code 0
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::command::CommandOutput;
    /// # use std::time::Duration;
    /// #
    /// let success = CommandOutput::new(
    ///     0,
    ///     "Success".to_string(),
    ///     "".to_string(),
    ///     Duration::from_secs(1)
    /// );
    /// assert!(success.success());
    ///
    /// let failure = CommandOutput::new(
    ///     1,
    ///     "".to_string(),
    ///     "Failed".to_string(),
    ///     Duration::from_secs(1)
    /// );
    /// assert!(!failure.success());
    /// ```
    #[must_use]
    pub fn success(&self) -> bool {
        self.status == 0
    }
}
