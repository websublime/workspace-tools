//! Example demonstrating the logging system.
//!
//! This example shows how logging works at different levels and how it's
//! independent of output format.
//!
//! Run with different log levels:
//! ```bash
//! cargo run --example logging_example -- --log-level silent
//! cargo run --example logging_example -- --log-level error
//! cargo run --example logging_example -- --log-level warn
//! cargo run --example logging_example -- --log-level info
//! cargo run --example logging_example -- --log-level debug
//! cargo run --example logging_example -- --log-level trace
//! ```
//!
//! Test with different output formats:
//! ```bash
//! cargo run --example logging_example -- --format human --log-level info
//! cargo run --example logging_example -- --format json --log-level info
//! cargo run --example logging_example -- --format json --log-level silent
//! ```

// Examples are allowed to use println! for demonstration purposes
#![allow(clippy::print_stdout)]

use sublime_cli_tools::cli::LogLevel;
use sublime_cli_tools::output::logger::{command_span, init_logging, operation_span};
use sublime_cli_tools::output::{Output, OutputFormat};
use tracing::{debug, error, info, trace, warn};

#[derive(serde::Serialize)]
struct ExampleData {
    processed: usize,
    status: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for log level and format
    let args: Vec<String> = std::env::args().collect();

    let log_level = parse_log_level(&args);
    let format = parse_format(&args);

    println!("=== Logging System Example ===");
    println!("Log Level: {log_level}");
    println!("Output Format: {format}");
    println!("Note: Logs go to stderr, output goes to stdout");
    println!();

    // Initialize logging
    init_logging(log_level, false)?;

    // Create output handler
    let output = Output::new(format, std::io::stdout(), false);

    // Demonstrate logging at different levels
    demonstrate_logging_levels();

    // Demonstrate spans
    demonstrate_spans();

    // Demonstrate logging with output
    demonstrate_logging_with_output(&output)?;

    Ok(())
}

fn demonstrate_logging_levels() {
    println!("\n--- Demonstrating Log Levels (stderr) ---");

    // These all go to stderr
    error!("This is an ERROR message - always visible except in silent mode");
    warn!("This is a WARN message - visible from warn level up");
    info!("This is an INFO message - visible from info level up (default)");
    debug!("This is a DEBUG message - visible from debug level up");
    trace!("This is a TRACE message - only visible in trace mode");

    // With structured fields
    info!(package = "example-package", version = "1.2.3", "Package version updated");

    debug!(file = "package.json", size = 1234, "Reading configuration file");
}

fn demonstrate_spans() {
    println!("\n--- Demonstrating Spans (stderr) ---");

    // Command-level span
    let _command_span = command_span("example_command");
    info!("Inside command span");

    {
        // Operation-level span (nested)
        let _op_span = operation_span("load_config");
        debug!("Inside operation span");
        trace!("Loading configuration from disk");
    }

    info!("Back in command span");
}

fn demonstrate_logging_with_output(output: &Output) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Demonstrating Logging + Output (mixed streams) ---");

    // Logs go to stderr
    info!("Starting data processing...");

    // Output goes to stdout
    output.info("Processing item 1")?;

    debug!("Detailed processing information (stderr)");

    output.info("Processing item 2")?;

    // Success message goes to stdout
    output.success("All items processed")?;

    info!("Operation completed successfully");

    // Demonstrate JSON output is clean
    if matches!(output.format(), OutputFormat::Json | OutputFormat::JsonCompact) {
        info!("Generating JSON output (this log won't contaminate JSON)");

        let data = ExampleData { processed: 42, status: "complete".to_string() };

        output.json(&sublime_cli_tools::output::JsonResponse::success(data))?;

        info!("JSON output complete (clean on stdout, logs on stderr)");
    }

    Ok(())
}

fn parse_log_level(args: &[String]) -> LogLevel {
    for i in 0..args.len() {
        if args[i] == "--log-level" && i + 1 < args.len() {
            return args[i + 1].parse().unwrap_or(LogLevel::Info);
        }
    }
    LogLevel::Info
}

fn parse_format(args: &[String]) -> OutputFormat {
    for i in 0..args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            return args[i + 1].parse().unwrap_or(OutputFormat::Human);
        }
    }
    OutputFormat::Human
}
