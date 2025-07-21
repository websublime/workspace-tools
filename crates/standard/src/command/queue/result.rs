//! # Queue Result Implementation
//!
//! ## What
//! This module implements constructor and utility methods for CommandQueueResult,
//! providing convenient ways to create and inspect command execution results.
//!
//! ## How
//! The implementation provides factory methods for creating successful, failed,
//! and cancelled results, along with utility methods for checking success status.
//!
//! ## Why
//! Convenient result creation and inspection methods improve the ergonomics
//! of working with command execution results.

use super::super::types::{CommandOutput, CommandQueueResult, CommandStatus};

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
