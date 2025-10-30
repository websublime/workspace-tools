//! Output formatting and logging module.
//!
//! This module provides output formatting for different modes (human-readable, JSON)
//! and logging functionality that is independent of output formatting.
//!
//! # What
//!
//! Provides:
//! - `OutputFormat` enum for different output modes
//! - `Output` struct for consistent formatting across all commands
//! - JSON response structure for API-like output
//! - Styling and color utilities
//! - Table rendering capabilities
//! - Progress indicators
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
//! Creating an output instance:
//!
//! ```rust
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! output.success("Operation completed successfully").unwrap();
//! ```
//!
//! Using JSON output:
//!
//! ```rust
//! use sublime_cli_tools::output::{Output, OutputFormat, JsonResponse};
//! use std::io;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyData {
//!     value: String,
//! }
//!
//! let output = Output::new(OutputFormat::Json, io::stdout(), false);
//! let data = MyData { value: "test".to_string() };
//! let response = JsonResponse::success(data);
//! output.json(&response).unwrap();
//! ```

mod json;
mod style;
pub mod table;

#[cfg(test)]
mod tests;

use crate::error::{CliError, Result};
use serde::Serialize;
use std::cell::RefCell;
use std::io::Write;

// Public re-exports
pub use json::JsonResponse;
pub use style::{Style, StyledText};

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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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

/// Main output handler for CLI commands.
///
/// Provides consistent formatting across all commands with support for multiple
/// output formats (human-readable, JSON, quiet). All output goes to stdout,
/// while logs go to stderr independently.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// output.success("Operation completed").unwrap();
/// output.info("Processing 3 files").unwrap();
/// output.warning("Deprecated option used").unwrap();
/// ```
///
/// JSON output:
///
/// ```rust
/// use sublime_cli_tools::output::{Output, OutputFormat, JsonResponse};
/// use std::io;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Result {
///     count: usize,
/// }
///
/// let output = Output::new(OutputFormat::Json, io::stdout(), false);
/// let response = JsonResponse::success(Result { count: 42 });
/// output.json(&response).unwrap();
/// ```
pub struct Output {
    format: OutputFormat,
    writer: RefCell<Box<dyn Write + Send>>,
    no_color: bool,
}

impl Output {
    /// Creates a new Output instance.
    ///
    /// # Arguments
    ///
    /// * `format` - The output format to use
    /// * `writer` - The writer to output to (typically stdout)
    /// * `no_color` - Whether to disable color output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// ```
    pub fn new<W: Write + Send + 'static>(format: OutputFormat, writer: W, no_color: bool) -> Self {
        Self { format, writer: RefCell::new(Box::new(writer)), no_color }
    }

    /// Returns the current output format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Json, io::stdout(), false);
    /// assert_eq!(output.format(), OutputFormat::Json);
    /// ```
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Returns whether color output is disabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), true);
    /// assert!(output.no_color());
    /// ```
    pub fn no_color(&self) -> bool {
        self.no_color
    }

    /// Outputs a success message.
    ///
    /// In human mode, this is displayed with a green checkmark.
    /// In JSON mode, this is ignored (use `json()` instead).
    /// In quiet mode, this is suppressed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.success("Configuration saved successfully").unwrap();
    /// ```
    pub fn success(&self, message: &str) -> Result<()> {
        match self.format {
            OutputFormat::Human => {
                let styled = if self.no_color {
                    format!("✓ {message}")
                } else {
                    Style::success(&format!("✓ {message}"))
                };
                writeln!(self.writer.borrow_mut(), "{styled}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                // In JSON mode, success is part of the final JSON response
                Ok(())
            }
            OutputFormat::Quiet => Ok(()),
        }
    }

    /// Outputs an error message.
    ///
    /// In human mode, this is displayed with a red X.
    /// In JSON mode, this is ignored (use `json()` with error response).
    /// In quiet mode, this is still displayed as errors are critical.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.error("Failed to read configuration file").unwrap();
    /// ```
    pub fn error(&self, message: &str) -> Result<()> {
        match self.format {
            OutputFormat::Human => {
                let styled = if self.no_color {
                    format!("✗ {message}")
                } else {
                    Style::error(&format!("✗ {message}"))
                };
                writeln!(self.writer.borrow_mut(), "{styled}")?;
                Ok(())
            }
            OutputFormat::Quiet => {
                // Errors are always shown even in quiet mode
                writeln!(self.writer.borrow_mut(), "Error: {message}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                // In JSON mode, errors are part of the final JSON response
                Ok(())
            }
        }
    }

    /// Outputs a warning message.
    ///
    /// In human mode, this is displayed with a yellow warning symbol.
    /// In JSON mode, this is ignored (use `json()` instead).
    /// In quiet mode, this is suppressed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.warning("Deprecated configuration option detected").unwrap();
    /// ```
    pub fn warning(&self, message: &str) -> Result<()> {
        match self.format {
            OutputFormat::Human => {
                let styled = if self.no_color {
                    format!("⚠ {message}")
                } else {
                    Style::warning(&format!("⚠ {message}"))
                };
                writeln!(self.writer.borrow_mut(), "{styled}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact | OutputFormat::Quiet => Ok(()),
        }
    }

    /// Outputs an informational message.
    ///
    /// In human mode, this is displayed with a blue info symbol.
    /// In JSON mode, this is ignored (use `json()` instead).
    /// In quiet mode, this is suppressed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.info("Found 3 packages in workspace").unwrap();
    /// ```
    pub fn info(&self, message: &str) -> Result<()> {
        match self.format {
            OutputFormat::Human => {
                let styled = if self.no_color {
                    format!("ℹ {message}")
                } else {
                    Style::info(&format!("ℹ {message}"))
                };
                writeln!(self.writer.borrow_mut(), "{styled}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact | OutputFormat::Quiet => Ok(()),
        }
    }

    /// Outputs plain text without formatting.
    ///
    /// This is useful for outputting raw data or content that should
    /// not be modified. In JSON mode, this is ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.plain("Package: @org/core").unwrap();
    /// output.plain("Version: 1.2.3").unwrap();
    /// ```
    pub fn plain(&self, message: &str) -> Result<()> {
        match self.format {
            OutputFormat::Human | OutputFormat::Quiet => {
                writeln!(self.writer.borrow_mut(), "{message}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                // In JSON mode, plain text is ignored
                Ok(())
            }
        }
    }

    /// Outputs data as JSON.
    ///
    /// This is the primary method for outputting structured data in JSON mode.
    /// The data is serialized according to the current output format:
    /// - `Json`: Pretty-printed with indentation
    /// - `JsonCompact`: Single line, no whitespace
    /// - `Human`/`Quiet`: Ignored (use other methods for human output)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat, JsonResponse};
    /// use std::io;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyData {
    ///     packages: Vec<String>,
    ///     count: usize,
    /// }
    ///
    /// let output = Output::new(OutputFormat::Json, io::stdout(), false);
    /// let data = MyData {
    ///     packages: vec!["@org/core".to_string()],
    ///     count: 1,
    /// };
    /// let response = JsonResponse::success(data);
    /// output.json(&response).unwrap();
    /// ```
    pub fn json<T: Serialize>(&self, data: &T) -> Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(data)
                    .map_err(|e| CliError::execution(format!("JSON serialization failed: {e}")))?;
                writeln!(self.writer.borrow_mut(), "{json}")?;
                Ok(())
            }
            OutputFormat::JsonCompact => {
                let json = serde_json::to_string(data)
                    .map_err(|e| CliError::execution(format!("JSON serialization failed: {e}")))?;
                writeln!(self.writer.borrow_mut(), "{json}")?;
                Ok(())
            }
            OutputFormat::Human | OutputFormat::Quiet => {
                // In non-JSON modes, json() is ignored
                Ok(())
            }
        }
    }

    /// Renders and outputs a table.
    ///
    /// In human mode, displays a formatted table.
    /// In JSON mode, this is ignored (use `json()` with structured data instead).
    /// In quiet mode, this is suppressed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat, table::TableBuilder};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// let mut table = TableBuilder::new()
    ///     .columns(&["Package", "Version"])
    ///     .build();
    /// table.add_row(&["typescript", "5.3.3"]);
    /// output.table(&mut table).unwrap();
    /// ```
    pub fn table(&self, table: &mut table::Table) -> Result<()> {
        match self.format {
            OutputFormat::Human => {
                let rendered = table.render(self.no_color);
                writeln!(self.writer.borrow_mut(), "{rendered}")?;
                Ok(())
            }
            OutputFormat::Json | OutputFormat::JsonCompact | OutputFormat::Quiet => {
                // In JSON/quiet modes, tables are ignored
                Ok(())
            }
        }
    }

    /// Writes raw bytes to the output stream.
    ///
    /// This is a low-level method that bypasses all formatting.
    /// Use with caution and prefer the higher-level methods when possible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.write_raw(b"Raw data\n").unwrap();
    /// ```
    pub fn write_raw(&self, data: &[u8]) -> Result<()> {
        self.writer.borrow_mut().write_all(data)?;
        Ok(())
    }

    /// Flushes the output buffer.
    ///
    /// Ensures all buffered output is written to the underlying stream.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_cli_tools::output::{Output, OutputFormat};
    /// use std::io;
    ///
    /// let output = Output::new(OutputFormat::Human, io::stdout(), false);
    /// output.info("Processing...").unwrap();
    /// output.flush().unwrap();
    /// ```
    pub fn flush(&self) -> Result<()> {
        self.writer.borrow_mut().flush()?;
        Ok(())
    }
}

// TODO: will be implemented in story 3.2 (Table Rendering)
// Table rendering utilities will be added in the next story

// TODO: will be implemented in story 3.3 (Progress Indicators)
// Progress bar functionality will be added in a subsequent story
