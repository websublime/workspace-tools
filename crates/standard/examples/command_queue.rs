//! # Command Queue Example
//!
//! This example demonstrates how to use the command queue functionality to
//! manage multiple commands with different priorities, control execution
//! concurrency, and handle command results.

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
    command::{CommandBuilder, CommandPriority, CommandQueue, CommandQueueConfig, CommandStatus},
    diagnostic::{DiagnosticCollector, DiagnosticLevel},
    error::StandardResult,
};

/// Demonstrates basic command queue functionality
async fn basic_queue_example() -> StandardResult<()> {
    println!("=== Basic Command Queue Example ===");

    // Create and start a queue with default configuration
    let mut queue = CommandQueue::new().start()?;
    println!("Command queue started with default configuration");

    // Create a simple command
    let command = CommandBuilder::new("echo").arg("Hello from command queue!").build();

    // Enqueue the command with normal priority
    println!("Enqueuing command with Normal priority");
    let id = queue.enqueue(command, CommandPriority::Normal).await?;
    println!("Command enqueued with ID: {}", id);

    // Wait for command to complete
    println!("Waiting for command to complete...");
    let result = queue.wait_for_command(&id, Duration::from_secs(5)).await?;

    // Display the result
    if result.is_successful() {
        if let Some(output) = &result.output {
            println!("Command completed successfully!");
            println!("Output: {}", output.stdout().trim());
            println!("Execution time: {:?}", output.duration());
        }
    } else if let Some(error) = &result.error {
        println!("Command failed: {}", error);
    }

    // Shutdown the queue
    println!("Shutting down queue...");
    queue.shutdown().await?;
    println!("Queue shutdown complete\n");

    Ok(())
}

/// Demonstrates command priorities in the queue
async fn priority_queue_example() -> StandardResult<()> {
    println!("=== Command Priority Example ===");

    // Create and start a queue with custom configuration
    // Use just 1 concurrent command to make priority behavior clear
    let config = CommandQueueConfig {
        max_concurrent_commands: 1,
        rate_limit: Some(Duration::from_millis(100)),
        default_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(5),
    };

    let mut queue = CommandQueue::with_config(config).start()?;
    println!("Command queue started with 1 concurrent command");

    // Create a sleep command (slower)
    #[cfg(target_family = "unix")]
    let slow_command = CommandBuilder::new("sleep").arg("1").build();

    #[cfg(target_family = "windows")]
    let slow_command = CommandBuilder::new("timeout").arg("1").build();

    // Create some echo commands (faster)
    let commands = [
        (
            CommandBuilder::new("echo").arg("Low priority command").build(),
            CommandPriority::Low,
            "low",
        ),
        (
            CommandBuilder::new("echo").arg("Normal priority command").build(),
            CommandPriority::Normal,
            "normal",
        ),
        (
            CommandBuilder::new("echo").arg("High priority command").build(),
            CommandPriority::High,
            "high",
        ),
        (
            CommandBuilder::new("echo").arg("Critical priority command").build(),
            CommandPriority::Critical,
            "critical",
        ),
    ];

    // First, enqueue the slow command with normal priority
    println!("Enqueuing slow command with Normal priority");
    let slow_id = queue.enqueue(slow_command, CommandPriority::Normal).await?;

    // Then quickly enqueue the other commands with different priorities
    let mut ids = Vec::new();
    for (command, priority, name) in &commands {
        println!("Enqueuing {} priority command", name);
        let id = queue.enqueue(command.clone(), *priority).await?;
        ids.push((id, (*name).to_string()));
    }

    // Wait for all commands to complete
    println!("Waiting for all commands to complete...");
    queue.wait_for_completion().await?;

    // The commands should have executed in priority order
    println!("All commands completed. Results:");

    // First check the slow command
    let slow_result = queue.get_result(&slow_id);
    if let Some(result) = slow_result {
        println!("Slow command status: {:?}", result.status);
    }

    // Then check the priority commands
    for (id, name) in ids {
        let result = queue.get_result(&id);
        if let Some(result) = result {
            println!("{} priority command - Status: {:?}", name, result.status);
            if let Some(output) = &result.output {
                println!("  Output: {}", output.stdout().trim());
            }
        }
    }

    // Shutdown the queue
    println!("Shutting down queue...");
    queue.shutdown().await?;
    println!("Queue shutdown complete\n");

    Ok(())
}

#[allow(unused_assignments)]
/// Demonstrates concurrent command execution
async fn concurrent_queue_example() -> StandardResult<()> {
    println!("=== Concurrent Command Execution Example ===");

    // Create a queue with high concurrency
    let config = CommandQueueConfig {
        max_concurrent_commands: 4,
        rate_limit: None,
        default_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(5),
    };

    let mut queue = CommandQueue::with_config(config).start()?;
    println!("Command queue started with 4 concurrent commands");

    // Create a set of commands that will run concurrently
    let command_count = 10;
    let mut ids = Vec::new();

    println!("Enqueuing {} commands...", command_count);
    for i in 0..command_count {
        let command = CommandBuilder::new("echo").arg(format!("Command {i} output")).build();

        let id = queue.enqueue(command, CommandPriority::Normal).await?;
        ids.push(id);
    }

    // Monitor command execution status
    println!("Monitoring command status...");
    let start_time = std::time::Instant::now();

    let mut completed = 0;
    let mut running = 0;
    let mut queued = 0;

    while completed < command_count {
        // Reset counters
        completed = 0;
        running = 0;
        queued = 0;

        // Check status of all commands
        for id in &ids {
            match queue.get_status(id) {
                Some(CommandStatus::Completed) => completed += 1,
                Some(CommandStatus::Running) => running += 1,
                Some(CommandStatus::Queued) => queued += 1,
                _ => {}
            }
        }

        println!(
            "Status after {:?}: Completed: {}, Running: {}, Queued: {}",
            start_time.elapsed(),
            completed,
            running,
            queued
        );

        // Don't busy-wait
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Break if all commands are complete
        if completed == command_count {
            break;
        }
    }

    println!("All commands completed in {:?}", start_time.elapsed());

    // Shutdown the queue
    println!("Shutting down queue...");
    queue.shutdown().await?;
    println!("Queue shutdown complete\n");

    Ok(())
}

/// Demonstrates error handling in command queue
async fn error_handling_example(diagnostics: &DiagnosticCollector) -> StandardResult<()> {
    println!("=== Error Handling Example ===");

    let mut queue = CommandQueue::new().start()?;
    println!("Command queue started");

    // Enqueue a command that will fail
    let invalid_command = CommandBuilder::new("non_existent_command").arg("test").build();

    println!("Enqueuing invalid command that will fail");
    let id = queue.enqueue(invalid_command, CommandPriority::Normal).await?;

    // Try to wait for the command
    println!("Waiting for command result...");
    match queue.wait_for_command(&id, Duration::from_secs(5)).await {
        Ok(result) => {
            println!("Command completed with status: {:?}", result.status);
            if let Some(error) = result.error {
                println!("Error: {}", error);
                diagnostics.error("command_queue", format!("Command failed: {}", error));
            }
        }
        Err(error) => {
            println!("Error waiting for command: {}", error);
            diagnostics.error("command_queue", format!("Wait error: {}", error));
            return Err(error);
        }
    }

    // Shutdown the queue
    println!("Shutting down queue...");
    queue.shutdown().await?;
    println!("Queue shutdown complete\n");

    Ok(())
}

/// A real-world example demonstrating command queue with diagnostics
async fn project_build_example(diagnostics: &DiagnosticCollector) -> StandardResult<()> {
    println!("=== Project Build Example ===");
    println!("Simulating building a multi-package project");

    // Create a queue with concurrency based on system cores
    //let cores = num_cpus::get();
    let config = CommandQueueConfig {
        max_concurrent_commands: 5,
        rate_limit: None,
        default_timeout: Duration::from_secs(60),
        shutdown_timeout: Duration::from_secs(10),
    };

    let mut queue = CommandQueue::with_config(config).start()?;
    println!("Command queue started with 5 concurrent commands");
    diagnostics.info("build_system", "Build started with 5 concurrent jobs");

    // Simulate a multi-package project build
    let packages = [
        ("core-lib", CommandPriority::Critical),
        ("utils", CommandPriority::High),
        ("api", CommandPriority::Normal),
        ("ui", CommandPriority::Normal),
        ("docs", CommandPriority::Low),
    ];

    println!("Enqueueing build tasks for {} packages", packages.len());
    let start_time = std::time::Instant::now();

    // Enqueue all package builds
    let mut package_ids = Vec::new();
    for (package, priority) in &packages {
        diagnostics.info("build_system", format!("Scheduling build for '{}'", package));

        // Create a simulated build command (just echo in this example)
        let command =
            CommandBuilder::new("echo").arg(format!("Building {} package", package)).build();

        // Enqueue with appropriate priority
        let id = queue.enqueue(command, *priority).await?;
        package_ids.push((id, (*package).to_string()));
    }

    // Wait for all tasks to complete
    println!("Waiting for all build tasks to complete...");
    match queue.wait_for_completion().await {
        Ok(()) => {
            let duration = start_time.elapsed();
            println!("All build tasks completed in {:?}", duration);
            diagnostics
                .info("build_system", format!("Build completed successfully in {:?}", duration));
        }
        Err(e) => {
            diagnostics.error("build_system", format!("Build failed: {}", e));
            return Err(e);
        }
    }

    // Check results for each package
    println!("\nBuild results:");
    for (id, package) in package_ids {
        if let Some(result) = queue.get_result(&id) {
            if result.is_successful() {
                println!("  ✅ {} built successfully", package);
                if let Some(output) = &result.output {
                    println!("     Output: {}", output.stdout().trim());
                }
                diagnostics
                    .info("build_system", format!("Package '{}' built successfully", package));
            } else {
                println!("  ❌ {} build failed", package);
                if let Some(error) = &result.error {
                    println!("     Error: {}", error);
                }
                diagnostics.warning(
                    "build_system",
                    format!("Package '{}' build failed: {:?}", package, result.error),
                );
            }
        }
    }

    // Shutdown the queue
    queue.shutdown().await?;
    diagnostics.info("build_system", "Build system shutdown");

    Ok(())
}

/// Display the diagnostic logs collected during execution
fn display_diagnostics(diagnostics: &DiagnosticCollector) {
    println!("\n=== Diagnostic Logs ===");

    let entries = diagnostics.entries();
    if entries.is_empty() {
        println!("No diagnostic entries recorded");
        return;
    }

    println!("Total entries: {}", entries.len());

    // Group by context
    let mut contexts = std::collections::HashMap::new();
    for entry in &entries {
        contexts.entry(entry.context.clone()).or_insert_with(Vec::new).push(entry);
    }

    // Display entries grouped by context
    for (context, entries) in contexts {
        println!("\nContext: {}", context);
        for entry in entries {
            let level = match entry.level {
                DiagnosticLevel::Info => "INFO",
                DiagnosticLevel::Warning => "WARN",
                DiagnosticLevel::Error => "ERROR",
                DiagnosticLevel::Critical => "CRIT",
            };
            println!("[{}] {}", level, entry.message);
        }
    }
}

#[tokio::main]
async fn main() -> StandardResult<()> {
    // Create a diagnostic collector to track events
    let diagnostics = DiagnosticCollector::new();

    // Run the examples
    basic_queue_example().await?;
    priority_queue_example().await?;
    concurrent_queue_example().await?;
    error_handling_example(&diagnostics).await?;
    project_build_example(&diagnostics).await?;

    // Display collected diagnostics
    display_diagnostics(&diagnostics);

    Ok(())
}
