//! Output formatting and logging module.
//!
//! This module provides output formatting for different modes (human-readable, JSON)
//! and logging functionality that is independent of output formatting.
//!
//! # What
//!
//! Provides:
//! - `OutputFormat` enum for different output modes
//! - Output utilities for consistent formatting
//! - Logging initialization and utilities
//! - Progress indicators and table rendering
//! - Separation of concerns: logs go to stderr, output goes to stdout
//!
//! # How
//!
//! The module uses:
//! - `comfy-table` for table rendering
//! - `indicatif` for progress bars
//! - `console` for styling and terminal control
//! - `tracing` for structured logging
//! - `serde_json` for JSON output
//!
//! Key principle: `--log-level` controls stderr (logs), `--format` controls stdout (output).
//! These are completely independent - you can have JSON output with debug logs,
//! or text output with no logs, etc.
//!
//! # Why
//!
//! Separating output formatting from logging ensures:
//! - Clean JSON output without log contamination
//! - Consistent output across all commands
//! - Flexibility in choosing output and logging independently
//! - Better tooling integration (parseable JSON + debug logs)
//!
//! ## Examples
//!
//! ```rust
//! use sublime_cli_tools::output::OutputFormat;
//!
//! let format = OutputFormat::Json;
//! assert!(format.is_json());
//! ```

use serde::Serialize;

/// Output format for CLI commands.
///
/// Controls how command results are formatted and displayed to stdout.
/// This is independent of logging (stderr) controlled by `--log-level`.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::OutputFormat;
///
/// let format = OutputFormat::Human;
/// assert!(!format.is_json());
///
/// let json_format = OutputFormat::Json;
/// assert!(json_format.is_json());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable output with colors, tables, and formatting.
    ///
    /// This is the default format for interactive use.
    Human,

    /// Pretty-printed JSON output.
    ///
    /// All command results are wrapped in a standard JSON structure with
    /// `success`, `data`, and `error` fields.
    Json,

    /// Compact JSON output (single line, no whitespace).
    ///
    /// Same structure as Json but minified for piping and processing.
    JsonCompact,

    /// Minimal output mode.
    ///
    /// Only essential information is displayed. Useful for scripting.
    Quiet,
}

impl OutputFormat {
    /// Returns true if this format is JSON (either pretty or compact).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// assert!(OutputFormat::Json.is_json());
    /// assert!(OutputFormat::JsonCompact.is_json());
    /// assert!(!OutputFormat::Human.is_json());
    /// ```
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json | Self::JsonCompact)
    }

    /// Returns true if this format is human-readable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// assert!(OutputFormat::Human.is_human());
    /// assert!(!OutputFormat::Json.is_human());
    /// ```
    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human)
    }

    /// Returns true if this format is quiet mode.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::OutputFormat;
    ///
    /// assert!(OutputFormat::Quiet.is_quiet());
    /// assert!(!OutputFormat::Human.is_quiet());
    /// ```
    pub fn is_quiet(&self) -> bool {
        matches!(self, Self::Quiet)
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Human
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
            Self::JsonCompact => write!(f, "json-compact"),
            Self::Quiet => write!(f, "quiet"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "json-compact" | "compact" => Ok(Self::JsonCompact),
            "quiet" => Ok(Self::Quiet),
            _ => Err(format!(
                "Invalid output format '{s}'. Valid options: human, json, json-compact, quiet"
            )),
        }
    }
}

/// Standard JSON response structure for all commands.
///
/// All commands must use this structure when outputting JSON to ensure
/// consistency across the CLI.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::JsonResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyData {
///     value: String,
/// }
///
/// let success = JsonResponse::success(MyData {
///     value: "test".to_string(),
/// });
/// assert!(success.success);
///
/// let error: JsonResponse<MyData> = JsonResponse::error("Something went wrong".to_string());
/// assert!(!error.success);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct JsonResponse<T> {
    /// Whether the operation succeeded.
    pub success: bool,

    /// The data payload (only present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// The error message (only present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> JsonResponse<T> {
    /// Creates a successful response with data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let response = JsonResponse::success("Hello");
    /// assert!(response.success);
    /// assert_eq!(response.data, Some("Hello"));
    /// assert_eq!(response.error, None);
    /// ```
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    /// Creates an error response with a message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::JsonResponse;
    ///
    /// let response: JsonResponse<()> = JsonResponse::error("Failed".to_string());
    /// assert!(!response.success);
    /// assert_eq!(response.data, None);
    /// assert_eq!(response.error, Some("Failed".to_string()));
    /// ```
    pub fn error(message: String) -> Self {
        Self { success: false, data: None, error: Some(message) }
    }
}

// TODO: will be implemented in story 1.3 (Output Formatting & Logging)
// Additional modules:
// - table.rs: Table rendering utilities using comfy-table
// - json.rs: JSON output formatting utilities
// - progress.rs: Progress indicators using indicatif
// - style.rs: Color and styling using console
// - logger.rs: Logging setup and utilities using tracing
// - context.rs: Global context for output and logging options
// - tests.rs: Output module tests
