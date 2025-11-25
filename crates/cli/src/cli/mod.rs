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
pub mod branding;
pub mod commands;
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

/// Workspace Tools - Changeset-based version management.
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
/// workspace init
///
/// # Add a changeset
/// workspace changeset add
///
/// # Preview version bump
/// workspace bump --dry-run
///
/// # JSON output with no logs (clean JSON for automation)
/// workspace --format json --log-level silent bump --dry-run
///
/// # Debug logging with text output
/// workspace --log-level debug changeset list
/// ```
#[derive(Debug, Parser)]
#[command(name = "workspace")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Workspace Tools - Changeset-based version management")]
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

    /// Quiet mode.
    ///
    /// Minimizes both logs (stderr) and output (stdout).
    /// Equivalent to `--log-level silent --format quiet`.
    ///
    /// Useful for scripts and automation where only exit code matters.
    /// Cannot be combined with --log-level, --format, or --verbose.
    #[arg(global = true, short = 'q', long, conflicts_with_all = ["log_level", "format", "verbose"])]
    pub quiet: bool,

    /// Verbose output mode.
    ///
    /// Increases detail level in command output (stdout) and enables debug logs (stderr).
    /// Equivalent to `--log-level debug`.
    ///
    /// Different from --log-level which only controls operational logs.
    /// Cannot be combined with --log-level or --quiet.
    #[arg(global = true, short = 'v', long, conflicts_with_all = ["log_level", "quiet"])]
    pub verbose: bool,
}

impl Cli {
    /// Returns the raw log level from command line.
    ///
    /// Note: Prefer `effective_log_level()` which accounts for --quiet and --verbose flags.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::{Cli, LogLevel};
    ///
    /// let cli = Cli::parse_from(["workspace", "--log-level", "debug", "version"]);
    /// assert_eq!(cli.log_level(), LogLevel::Debug);
    /// ```
    #[must_use]
    pub const fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Returns the effective log level, accounting for --quiet and --verbose flags.
    ///
    /// Priority:
    /// 1. --quiet: Returns `Silent`
    /// 2. --verbose: Returns `Debug`
    /// 3. Otherwise: Returns the value from --log-level
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::{Cli, LogLevel};
    ///
    /// // --quiet overrides to silent
    /// let cli = Cli::parse_from(["workspace", "--quiet", "version"]);
    /// assert_eq!(cli.effective_log_level(), LogLevel::Silent);
    ///
    /// // --verbose sets debug level
    /// let cli = Cli::parse_from(["workspace", "--verbose", "version"]);
    /// assert_eq!(cli.effective_log_level(), LogLevel::Debug);
    ///
    /// // Default uses --log-level value
    /// let cli = Cli::parse_from(["workspace", "--log-level", "warn", "version"]);
    /// assert_eq!(cli.effective_log_level(), LogLevel::Warn);
    /// ```
    #[must_use]
    pub const fn effective_log_level(&self) -> LogLevel {
        if self.quiet {
            LogLevel::Silent
        } else if self.verbose {
            LogLevel::Debug
        } else {
            self.log_level
        }
    }

    /// Returns the raw output format from command line.
    ///
    /// Note: Prefer `effective_output_format()` which accounts for --quiet flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// let cli = Cli::parse_from(["workspace", "--format", "json", "version"]);
    /// assert_eq!(cli.output_format(), OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn output_format(&self) -> OutputFormat {
        self.format.0
    }

    /// Returns the effective output format, accounting for --quiet flag.
    ///
    /// Priority:
    /// 1. --quiet: Returns `Quiet`
    /// 2. Otherwise: Returns the value from --format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use clap::Parser;
    /// use sublime_cli_tools::cli::Cli;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// // --quiet overrides to quiet format
    /// let cli = Cli::parse_from(["workspace", "--quiet", "version"]);
    /// assert_eq!(cli.effective_output_format(), OutputFormat::Quiet);
    ///
    /// // Default uses --format value
    /// let cli = Cli::parse_from(["workspace", "--format", "json", "version"]);
    /// assert_eq!(cli.effective_output_format(), OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn effective_output_format(&self) -> OutputFormat {
        if self.quiet { OutputFormat::Quiet } else { self.format.0 }
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
    /// let cli = Cli::parse_from(["workspace", "--no-color", "version"]);
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
    /// let cli = Cli::parse_from(["workspace", "--root", "/tmp", "version"]);
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
    /// let cli = Cli::parse_from(["workspace", "--config", "custom.toml", "version"]);
    /// assert_eq!(cli.config_path(), Some(&PathBuf::from("custom.toml")));
    /// ```
    #[must_use]
    pub const fn config_path(&self) -> Option<&PathBuf> {
        self.config.as_ref()
    }
}
