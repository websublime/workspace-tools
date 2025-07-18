//! # Core Command Types
//!
//! ## What
//! This module defines the fundamental command execution types including
//! Command, CommandBuilder, and CommandOutput structures.
//!
//! ## How
//! These types provide a structured way to define commands with their
//! parameters, build them using a fluent interface, and capture their results.
//!
//! ## Why
//! Centralized command types ensure consistency across the command execution
//! system and provide a clear API for command configuration and result handling.

use std::{
    collections::HashMap,
    path::PathBuf,
    time::Duration,
};

/// Result of executing a command.
///
/// Contains the exit status, captured stdout and stderr output, and the duration
/// of the command execution.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::CommandOutput;
/// use std::time::Duration;
///
/// let output = CommandOutput {
///     status: 0,
///     stdout: "Hello, world!".to_string(),
///     stderr: "".to_string(),
///     duration: Duration::from_millis(50),
/// };
///
/// assert_eq!(output.status, 0);
/// assert_eq!(output.stdout, "Hello, world!");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommandOutput {
    /// Exit status code
    pub(crate) status: i32,
    /// Standard output content
    pub(crate) stdout: String,
    /// Standard error content
    pub(crate) stderr: String,
    /// Command execution duration
    pub(crate) duration: Duration,
}

/// Represents a command to be executed.
///
/// Contains all the information needed to execute a command, including the program
/// to run, its arguments, environment variables, working directory, and timeout.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::{Command, CommandBuilder};
/// use std::collections::HashMap;
/// use std::time::Duration;
///
/// // Create a command using CommandBuilder
/// let command = CommandBuilder::new("npm")
///     .arg("install")
///     .env("NODE_ENV", "production")
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
}

/// Builder for creating Command instances.
///
/// Provides a fluent interface for configuring command parameters before execution.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::command::types::CommandBuilder;
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let command = CommandBuilder::new("cargo")
///     .arg("test")
///     .args(&["--all-features", "--no-fail-fast"])
///     .env("RUST_BACKTRACE", "1")
///     .current_dir(PathBuf::from("./my-project"))
///     .timeout(Duration::from_secs(120))
///     .build();
/// ```
#[derive(Debug)]
pub struct CommandBuilder {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) current_dir: Option<PathBuf>,
    pub(crate) timeout: Option<Duration>,
}