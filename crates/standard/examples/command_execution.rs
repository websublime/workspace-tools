//! # Command Execution Example
//!
//! This example demonstrates how to use the command execution functionality
//! to run and monitor external commands with proper error handling and
//! resource management.

#![allow(clippy::print_stdout)]
#![allow(clippy::uninlined_format_args)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

use std::time::Duration;
use sublime_standard_tools::{
    command::{
        CommandBuilder, CommandExecutor, DefaultCommandExecutor, StreamConfig, StreamOutput,
    },
    diagnostic::{DiagnosticCollector, DiagnosticLevel},
    error::StandardResult,
};

/// Executes a simple command and logs the result
async fn execute_simple_command(
    executor: &impl CommandExecutor,
    diagnostics: &DiagnosticCollector,
) -> StandardResult<()> {
    println!("=== Running Simple Command Example ===");

    // Create a command with timeout
    let command = CommandBuilder::new("echo")
        .arg("Command executed successfully!")
        .timeout(Duration::from_secs(5))
        .build();

    // Record that we're about to execute a command
    diagnostics.info("command_execution", "Executing echo command");

    // Execute the command
    let start = std::time::Instant::now();
    let result = executor.execute(command).await;
    let elapsed = start.elapsed();

    match result {
        Ok(output) => {
            println!("Command output: {}", output.stdout().trim());
            println!("Execution time: {:?}", elapsed);
            diagnostics.info(
                "command_execution",
                format!("Command executed successfully in {:?}", elapsed),
            );
            Ok(())
        }
        Err(error) => {
            diagnostics.error("command_execution", format!("Command failed: {}", error));
            Err(error.into())
        }
    }
}

/// Executes a command with streaming output
async fn execute_streaming_command(
    executor: &impl CommandExecutor,
    diagnostics: &DiagnosticCollector,
) -> StandardResult<()> {
    println!("\n=== Running Streaming Command Example ===");

    // Choose a command that generates multiple lines of output
    // On Unix-like systems, use `ls -la /`
    // On Windows, use `dir C:\`
    #[cfg(target_family = "unix")]
    let command = CommandBuilder::new("ls").arg("-la").arg("/").build();

    #[cfg(target_family = "windows")]
    let command = CommandBuilder::new("dir").arg("C:\\").build();

    // Configure streaming with buffer size and read timeout
    let stream_config = StreamConfig::new(100, Duration::from_millis(500));

    diagnostics.info("command_execution", "Executing streaming command");

    // Execute with streaming
    let (mut stream, mut child) = executor.execute_stream(command, stream_config).await?;

    // Process stream output with timeout
    let mut line_count = 0;
    while let Ok(Some(output)) = stream.next_timeout(Duration::from_secs(1)).await {
        match output {
            StreamOutput::Stdout(line) => {
                println!("stdout: {}", line);
                line_count += 1;
            }
            StreamOutput::Stderr(line) => {
                eprintln!("stderr: {}", line);
                diagnostics.warning("command_execution", format!("Stderr output: {}", line));
            }
            StreamOutput::End => {
                println!("Stream ended");
                break;
            }
        }
    }

    // Ensure the child process is properly terminated
    if let Err(e) = child.kill().await {
        diagnostics.warning("command_execution", format!("Failed to kill child process: {}", e));
    }

    diagnostics.info(
        "command_execution",
        format!("Streaming command completed with {} lines of output", line_count),
    );

    println!("Processed {} lines of output", line_count);
    Ok(())
}

/// Long-running command with timeout handling
async fn execute_command_with_timeout(
    executor: &impl CommandExecutor,
    diagnostics: &DiagnosticCollector,
) -> StandardResult<()> {
    println!("\n=== Running Command with Timeout Example ===");

    // Create a command that will take longer than our timeout
    // Using sleep command on Unix-like systems
    // Using timeout command on Windows
    #[cfg(target_family = "unix")]
    let command = CommandBuilder::new("sleep").arg("5").timeout(Duration::from_secs(2)).build();

    #[cfg(target_family = "windows")]
    let command = CommandBuilder::new("timeout").arg("5").timeout(Duration::from_secs(2)).build();

    diagnostics.info("command_execution", "Executing command with short timeout");

    println!("Running command with 2 second timeout...");
    let start = std::time::Instant::now();
    let result = executor.execute(command).await;
    let elapsed = start.elapsed();

    match result {
        Ok(output) => {
            println!("Command completed in {:?}", elapsed);
            println!("Output: {}", output.stdout().trim());
            diagnostics.info(
                "command_execution",
                format!("Command completed successfully in {:?}", elapsed),
            );
            Ok(())
        }
        Err(error) => {
            println!("Command failed as expected: {}", error);
            if error.to_string().contains("timed out") {
                diagnostics.info(
                    "command_execution",
                    format!("Command timed out after {:?} as expected", elapsed),
                );
                Ok(()) // This is expected behavior for this example
            } else {
                diagnostics
                    .error("command_execution", format!("Command failed unexpectedly: {}", error));
                Err(error.into())
            }
        }
    }
}

/// Demonstrates using the diagnostic collector to track command operations
fn display_diagnostics(diagnostics: &DiagnosticCollector) {
    println!("\n=== Diagnostic Summary ===");

    let all_entries = diagnostics.entries();
    println!("Total diagnostic entries: {}", all_entries.len());

    let warnings_and_errors = diagnostics.entries_with_level_at_or_above(DiagnosticLevel::Warning);
    println!("Warnings and errors: {}", warnings_and_errors.len());

    for entry in warnings_and_errors {
        let level = match entry.level {
            DiagnosticLevel::Info => "INFO",
            DiagnosticLevel::Warning => "WARNING",
            DiagnosticLevel::Error => "ERROR",
            DiagnosticLevel::Critical => "CRITICAL",
        };

        println!("[{}] {}: {}", level, entry.context, entry.message);
    }
}

#[tokio::main]
async fn main() -> StandardResult<()> {
    // Create a command executor and diagnostic collector
    let executor = DefaultCommandExecutor::new();
    let diagnostics = DiagnosticCollector::new();

    // Run the examples
    execute_simple_command(&executor, &diagnostics).await?;
    execute_streaming_command(&executor, &diagnostics).await?;
    execute_command_with_timeout(&executor, &diagnostics).await?;

    // Show diagnostic summary
    display_diagnostics(&diagnostics);

    Ok(())
}
