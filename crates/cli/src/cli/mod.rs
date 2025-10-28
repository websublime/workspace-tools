//! CLI framework module.
//!
//! This module defines the CLI structure, command parsing, and global options.
//!
//! # What
//!
//! Provides the core CLI framework including:
//! - Command-line argument definitions using Clap
//! - Global options (root, log-level, format, no-color, config)
//! - Command enumeration and routing
//! - Argument parsing and validation
//!
//! # How
//!
//! Uses Clap's derive macros to define a structured CLI with global options
//! that apply to all commands and command-specific arguments. The framework
//! separates concerns between:
//! - CLI parsing (this module)
//! - Command execution (commands module)
//! - Output formatting (output module)
//! - Error handling (error module)
//!
//! # Why
//!
//! Centralizes CLI definition for consistency, maintainability, and automatic
//! help generation. Global options ensure consistent behavior across all commands.
//!
//! # Examples
//!
//! ```rust,no_run
//! use clap::Parser;
//! use sublime_cli_tools::cli::Cli;
//!
//! // Parse CLI arguments
//! let cli = Cli::parse();
//!
//! // Access global options
//! let format = cli.format;
//! let log_level = cli.log_level();
//! ```

mod args;
mod commands;
pub mod completions;
mod dispatch;

#[cfg(test)]
mod tests;

use clap::Parser;
use std::path::PathBuf;

pub use args::{LogLevel, OutputFormatArg};
pub use commands::Commands;
pub use completions::generate_completions;
pub use dispatch::dispatch_command;

use crate::output::OutputFormat;

/// Workspace Node Tools - Changeset-based version management.
///
/// This CLI provides comprehensive tools for managing Node.js workspaces using
/// a changeset-based workflow. It supports both single-package and monorepo
/// projects with independent or unified versioning strategies.
///
/// # Global Options
///
/// All global options apply to ALL subcommands and control behavior across
/// the entire application:
///
/// - `--root`: Changes working directory before executing commands
/// - `--log-level`: Controls logging verbosity (stderr only)
/// - `--format`: Controls output format (stdout only)
/// - `--no-color`: Disables ANSI colors in output and logs
/// - `--config`: Override default config file location
///
/// # Stream Separation
///
/// The CLI maintains strict separation between:
/// - **stderr**: Logs only (controlled by `--log-level`)
/// - **stdout**: Command output only (controlled by `--format`)
///
/// This ensures JSON output is never contaminated with logs, enabling
/// reliable piping and parsing in scripts.
///
/// # Examples
///
/// ```bash
/// # Initialize a new project
/// wnt init
///
/// # Add a changeset
/// wnt changeset add
///
/// # Preview version bump
/// wnt bump --dry-run
///
/// # JSON output with no logs (clean JSON for automation)
/// wnt --format json --log-level silent bump --dry-run
///
/// # Debug logging with text output
/// wnt --log-level debug changeset list
/// ```
#[derive(Debug, Parser)]
#[command(name = "wnt")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Workspace Node Tools - Changeset-based version management")]
#[command(long_about = None)]
#[command(author = "Sublime Labs")]
#[command(help_template = "\
{before-help}{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Project root directory.
    ///
    /// Changes working directory before executing the command.
    /// All file operations will be relative to this path.
    ///
    /// Default: Current directory
    #[arg(global = true, short = 'r', long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Logging level.
    ///
    /// Controls verbosity of operation logs written to stderr.
    /// Does NOT affect command output (stdout).
    ///
    /// Levels:
    /// - silent: No logs at all
    /// - error: Only critical errors
    /// - warn: Errors + warnings
    /// - info: General progress (default)
    /// - debug: Detailed operations
    /// - trace: Very verbose debugging
    ///
    /// Default: info
    #[arg(global = true, short = 'l', long, value_name = "LEVEL", default_value = "info")]
    pub log_level: LogLevel,

    /// Output format.
    ///
    /// Controls format of command output written to stdout.
    /// Does NOT affect logging (stderr).
    ///
    /// Formats:
    /// - human: Human-readable with colors and tables (default)
    /// - json: Pretty-printed JSON
    /// - json-compact: Compact JSON (single line)
    /// - quiet: Minimal output
    ///
    /// Default: human
    #[arg(global = true, short = 'f', long, value_name = "FORMAT", default_value = "human")]
    pub format: OutputFormatArg,

    /// Disable colored output.
    ///
    /// Removes ANSI color codes from both logs (stderr) and output (stdout).
    /// Also respects the NO_COLOR environment variable.
    ///
    /// Useful for CI/CD environments and file redirection.
    #[arg(global = true, long)]
    pub no_color: bool,

    /// Path to config file.
    ///
    /// Override default config file location.
    /// Path can be relative or absolute.
    ///
    /// Default: Auto-detect (.changesets.{toml,json,yaml,yml})
    #[arg(global = true, short = 'c', long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

impl Cli {
    /// Returns the log level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::{Cli, LogLevel};
    ///
    /// let cli = Cli::parse_from(["wnt", "--log-level", "debug", "version"]);
    /// assert_eq!(cli.log_level(), LogLevel::Debug);
    /// ```
    #[must_use]
    pub const fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Returns the output format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// let cli = Cli::parse_from(["wnt", "--format", "json", "version"]);
    /// assert_eq!(cli.output_format(), OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn output_format(&self) -> OutputFormat {
        self.format.0
    }

    /// Returns whether color output is disabled.
    ///
    /// Also checks the NO_COLOR environment variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    ///
    /// let cli = Cli::parse_from(["wnt", "--no-color", "version"]);
    /// assert!(cli.is_color_disabled());
    /// ```
    #[must_use]
    pub fn is_color_disabled(&self) -> bool {
        self.no_color || std::env::var("NO_COLOR").is_ok()
    }

    /// Returns the root directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    /// use std::path::PathBuf;
    ///
    /// let cli = Cli::parse_from(["wnt", "--root", "/tmp", "version"]);
    /// assert_eq!(cli.root(), Some(&PathBuf::from("/tmp")));
    /// ```
    #[must_use]
    pub const fn root(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

    /// Returns the config file path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    /// use std::path::PathBuf;
    ///
    /// let cli = Cli::parse_from(["wnt", "--config", "custom.toml", "version"]);
    /// assert_eq!(cli.config_path(), Some(&PathBuf::from("custom.toml")));
    /// ```
    #[must_use]
    pub const fn config_path(&self) -> Option<&PathBuf> {
        self.config.as_ref()
    }
}
