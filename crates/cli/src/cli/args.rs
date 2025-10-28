//! Shared argument types for CLI commands.
//!
//! This module defines common argument types used across multiple commands
//! and provides validation and conversion utilities.
//!
//! # What
//!
//! Provides:
//! - `LogLevel` enum for controlling logging verbosity
//! - `OutputFormatArg` wrapper for output format parsing
//! - Shared validation logic for common argument patterns
//! - Conversion utilities between CLI args and internal types
//!
//! # How
//!
//! Uses Clap's `ValueEnum` trait for automatic parsing and validation.
//! Types implement Display and FromStr for flexible usage.
//!
//! # Why
//!
//! Centralizes argument type definitions to ensure consistency across
//! all commands and provide a single source of truth for valid values.
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::cli::LogLevel;
//! use std::str::FromStr;
//!
//! let level = LogLevel::from_str("debug").unwrap();
//! assert_eq!(level, LogLevel::Debug);
//! ```

use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

use crate::output::OutputFormat;

/// Logging level for controlling verbosity.
///
/// Controls what logs are written to stderr. This is completely independent
/// of the output format (stdout).
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::LogLevel;
///
/// let level = LogLevel::Info;
/// let tracing_level = level.to_tracing_level();
/// assert_eq!(tracing_level, tracing::Level::INFO);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum LogLevel {
    /// No logs at all.
    ///
    /// Complete silence on stderr. Useful for automation where only
    /// stdout output is desired.
    Silent,

    /// Only critical errors.
    ///
    /// Shows only errors that prevent command execution.
    Error,

    /// Errors and warnings.
    ///
    /// Shows errors and important warnings.
    Warn,

    /// General progress information.
    ///
    /// Shows high-level progress and completion messages.
    /// This is the default level.
    Info,

    /// Detailed operation logs.
    ///
    /// Shows internal operations and decision points.
    /// Useful for troubleshooting.
    Debug,

    /// Very verbose debugging.
    ///
    /// Shows all internal operations including low-level details.
    /// Use for deep debugging only.
    Trace,
}

impl LogLevel {
    /// Converts to a tracing Level.
    ///
    /// Note: Silent is mapped to ERROR but should be filtered out
    /// during logger initialization.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
    /// assert_eq!(LogLevel::Debug.to_tracing_level(), tracing::Level::DEBUG);
    /// assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
    /// ```
    #[must_use]
    pub const fn to_tracing_level(&self) -> tracing::Level {
        match self {
            Self::Silent | Self::Error => tracing::Level::ERROR,
            Self::Warn => tracing::Level::WARN,
            Self::Info => tracing::Level::INFO,
            Self::Debug => tracing::Level::DEBUG,
            Self::Trace => tracing::Level::TRACE,
        }
    }

    /// Returns true if this is silent mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Silent.is_silent());
    /// assert!(!LogLevel::Info.is_silent());
    /// ```
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        matches!(self, Self::Silent)
    }

    /// Returns true if this level includes error logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Error.includes_errors());
    /// assert!(LogLevel::Info.includes_errors());
    /// assert!(!LogLevel::Silent.includes_errors());
    /// ```
    #[must_use]
    pub const fn includes_errors(&self) -> bool {
        !matches!(self, Self::Silent)
    }

    /// Returns true if this level includes warning logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Warn.includes_warnings());
    /// assert!(LogLevel::Info.includes_warnings());
    /// assert!(!LogLevel::Error.includes_warnings());
    /// ```
    #[must_use]
    pub const fn includes_warnings(&self) -> bool {
        matches!(self, Self::Warn | Self::Info | Self::Debug | Self::Trace)
    }

    /// Returns true if this level includes info logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Info.includes_info());
    /// assert!(LogLevel::Debug.includes_info());
    /// assert!(!LogLevel::Warn.includes_info());
    /// ```
    #[must_use]
    pub const fn includes_info(&self) -> bool {
        matches!(self, Self::Info | Self::Debug | Self::Trace)
    }

    /// Returns true if this level includes debug logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Debug.includes_debug());
    /// assert!(LogLevel::Trace.includes_debug());
    /// assert!(!LogLevel::Info.includes_debug());
    /// ```
    #[must_use]
    pub const fn includes_debug(&self) -> bool {
        matches!(self, Self::Debug | Self::Trace)
    }

    /// Returns true if this level includes trace logs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::LogLevel;
    ///
    /// assert!(LogLevel::Trace.includes_trace());
    /// assert!(!LogLevel::Debug.includes_trace());
    /// ```
    #[must_use]
    pub const fn includes_trace(&self) -> bool {
        matches!(self, Self::Trace)
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => write!(f, "silent"),
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "silent" => Ok(Self::Silent),
            "error" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(format!(
                "Invalid log level '{s}'. Valid options: silent, error, warn, info, debug, trace"
            )),
        }
    }
}

/// Wrapper for OutputFormat to implement Clap traits.
///
/// This type wraps `OutputFormat` to provide Clap integration while keeping
/// the core `OutputFormat` type independent of CLI concerns.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::cli::OutputFormatArg;
/// use sublime_cli_tools::output::OutputFormat;
/// use std::str::FromStr;
///
/// let arg = OutputFormatArg::from_str("json").unwrap();
/// assert_eq!(arg.0, OutputFormat::Json);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OutputFormatArg(pub OutputFormat);

impl OutputFormatArg {
    /// Creates a new OutputFormatArg from OutputFormat.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::OutputFormatArg;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// let arg = OutputFormatArg::new(OutputFormat::Json);
    /// assert_eq!(arg.0, OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn new(format: OutputFormat) -> Self {
        Self(format)
    }

    /// Returns the inner OutputFormat.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::OutputFormatArg;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// let arg = OutputFormatArg::new(OutputFormat::Json);
    /// assert_eq!(arg.into_inner(), OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn into_inner(self) -> OutputFormat {
        self.0
    }

    /// Returns a reference to the inner OutputFormat.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::cli::OutputFormatArg;
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// let arg = OutputFormatArg::new(OutputFormat::Json);
    /// assert_eq!(*arg.as_inner(), OutputFormat::Json);
    /// ```
    #[must_use]
    pub const fn as_inner(&self) -> &OutputFormat {
        &self.0
    }
}

impl fmt::Display for OutputFormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for OutputFormatArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OutputFormat::from_str(s).map(Self)
    }
}

impl ValueEnum for OutputFormatArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self(OutputFormat::Human),
            Self(OutputFormat::Json),
            Self(OutputFormat::JsonCompact),
            Self(OutputFormat::Quiet),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self.0 {
            OutputFormat::Human => clap::builder::PossibleValue::new("human")
                .help("Human-readable output with colors and tables"),
            OutputFormat::Json => {
                clap::builder::PossibleValue::new("json").help("Pretty-printed JSON output")
            }
            OutputFormat::JsonCompact => clap::builder::PossibleValue::new("json-compact")
                .help("Compact JSON output (single line)"),
            OutputFormat::Quiet => {
                clap::builder::PossibleValue::new("quiet").help("Minimal output")
            }
        })
    }
}

impl From<OutputFormat> for OutputFormatArg {
    fn from(format: OutputFormat) -> Self {
        Self(format)
    }
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        arg.0
    }
}
