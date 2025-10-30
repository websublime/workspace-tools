//! Logging system for CLI operations.
//!
//! This module provides structured logging using the tracing library, with
//! strict separation between logs (stderr) and output (stdout).
//!
//! # What
//!
//! Provides:
//! - Logging initialization based on `LogLevel`
//! - Tracing subscriber configuration
//! - Convenience macros for logging at different levels
//! - RUST_LOG environment variable support
//! - Proper stream separation (stderr for logs, stdout for output)
//!
//! # How
//!
//! Uses `tracing` and `tracing-subscriber` to create a global logging system
//! that writes exclusively to stderr. The logging level is controlled by the
//! `--log-level` CLI flag, which is completely independent of the `--format`
//! flag that controls stdout output.
//!
//! # Why
//!
//! Separating logs (stderr) from output (stdout) enables:
//! - Clean JSON output that's never mixed with logs
//! - Reliable piping and parsing in automation
//! - Debugging without contaminating command output
//! - Independent control of verbosity and format
//!
//! # Examples
//!
//! ```rust
//! use sublime_cli_tools::output::logger::init_logging;
//! use sublime_cli_tools::cli::LogLevel;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize logging at startup
//! init_logging(LogLevel::Info, false)?;
//!
//! // Use logging macros in your code
//! tracing::info!("Operation started");
//! tracing::debug!("Processing item: {}", "example");
//! tracing::warn!("Potential issue detected");
//! # Ok(())
//! # }
//! ```

use crate::cli::LogLevel;
use crate::error::{CliError, Result};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

/// Initializes the global tracing subscriber for logging.
///
/// This function sets up structured logging that writes exclusively to stderr,
/// never contaminating stdout. It respects both the CLI `--log-level` flag
/// and the `RUST_LOG` environment variable.
///
/// # Stream Separation
///
/// **Critical:** All logs go to stderr, never stdout. This ensures:
/// - JSON output on stdout is never mixed with logs
/// - Output can be reliably piped and parsed
/// - Logs can be independently redirected or suppressed
///
/// # Log Level Priority
///
/// 1. If `RUST_LOG` is set, it takes precedence (for debugging)
/// 2. Otherwise, uses the provided `level` from `--log-level` flag
/// 3. Silent mode (`LogLevel::Silent`) completely disables logging
///
/// # Arguments
///
/// * `level` - The log level from CLI arguments (--log-level)
/// * `no_color` - Whether to disable ANSI colors (from --no-color or NO_COLOR)
///
/// # Errors
///
/// Returns `CliError::Execution` if:
/// - The tracing subscriber is already initialized
/// - Environment variable parsing fails
/// - Subscriber configuration fails
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::logger::init_logging;
/// use sublime_cli_tools::cli::LogLevel;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Initialize with info level and colors enabled
/// init_logging(LogLevel::Info, false)?;
///
/// // Now logging is active
/// tracing::info!("Application started");
/// tracing::debug!("This won't be shown (level is Info)");
/// # Ok(())
/// # }
/// ```
///
/// # Silent Mode
///
/// ```rust
/// use sublime_cli_tools::output::logger::init_logging;
/// use sublime_cli_tools::cli::LogLevel;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Silent mode - no logs at all
/// init_logging(LogLevel::Silent, false)?;
///
/// // These produce no output
/// tracing::error!("Even errors are suppressed");
/// tracing::info!("Silent is silent");
/// # Ok(())
/// # }
/// ```
pub fn init_logging(level: LogLevel, no_color: bool) -> Result<()> {
    // Silent mode means no logging at all
    if level.is_silent() {
        return Ok(());
    }

    // Build environment filter
    // RUST_LOG takes precedence if set, otherwise use CLI level
    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        // Create filter based on log level
        let level_filter = level.to_tracing_level();

        // Filter out noisy dependencies but allow our crates
        EnvFilter::new(format!(
            "sublime_cli_tools={level_filter},sublime_pkg_tools={level_filter},sublime_standard_tools={level_filter},sublime_git_tools={level_filter}"
        ))
    };

    // Configure the subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr) // CRITICAL: Always stderr, never stdout
        .with_ansi(!no_color) // Respect NO_COLOR
        .with_target(level.includes_debug()) // Show target in debug/trace
        .with_line_number(level.includes_trace()) // Show line numbers in trace
        .with_file(level.includes_trace()) // Show file names in trace
        .with_span_events(if level.includes_trace() {
            FmtSpan::ENTER | FmtSpan::EXIT
        } else {
            FmtSpan::NONE
        })
        .with_level(true) // Always show level
        .with_thread_ids(level.includes_trace()) // Show thread IDs in trace
        .with_thread_names(level.includes_debug()) // Show thread names in debug
        .compact() // Use compact format
        .finish();

    // Set the global default subscriber
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| CliError::execution(format!("Failed to initialize logging: {e}")))?;

    Ok(())
}

/// Creates a tracing span for command execution.
///
/// Spans help organize log messages by context. This is particularly useful
/// for tracing execution flow and debugging complex operations.
///
/// # Arguments
///
/// * `name` - The name of the span (typically the command name)
///
/// # Returns
///
/// A tracing span that will log entry/exit when trace level is enabled.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::logger::{init_logging, command_span};
/// use sublime_cli_tools::cli::LogLevel;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// init_logging(LogLevel::Trace, false)?;
///
/// let _span = command_span("bump");
/// tracing::info!("Executing bump command");
/// // Span automatically closes when dropped
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn command_span(name: &str) -> tracing::Span {
    tracing::span!(tracing::Level::INFO, "command", name = name)
}

/// Creates a tracing span for a specific operation.
///
/// Use this for fine-grained tracking of operations within commands.
///
/// # Arguments
///
/// * `operation` - The name of the operation
///
/// # Returns
///
/// A tracing span that will log entry/exit when trace level is enabled.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::output::logger::{init_logging, operation_span};
/// use sublime_cli_tools::cli::LogLevel;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// init_logging(LogLevel::Trace, false)?;
///
/// {
///     let _span = operation_span("load_config");
///     tracing::debug!("Loading configuration file");
/// }
/// // Span automatically closes here
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn operation_span(operation: &str) -> tracing::Span {
    tracing::span!(tracing::Level::DEBUG, "operation", name = operation)
}

/// Re-exports for convenience.
///
/// These re-exports allow command implementations to use logging macros
/// without importing tracing directly.
///
/// # Examples
///
/// ```rust
/// // Instead of: use tracing::{info, debug, warn, error, trace};
/// // Just use the macros directly after this module is in scope
/// ```
pub use tracing::{debug, error, info, trace, warn};
