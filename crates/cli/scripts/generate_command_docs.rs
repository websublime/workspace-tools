#!/usr/bin/env rust-script
//! Generates command documentation from CLI help text.
//!
//! This script extracts help text from the compiled CLI binary and formats
//! it into markdown documentation. It ensures that the command reference
//! stays in sync with the actual CLI implementation.
//!
//! # What
//!
//! Provides:
//! - Help text extraction from CLI binary
//! - Markdown formatting of help output
//! - Documentation synchronization checks
//! - Command discovery and enumeration
//!
//! # How
//!
//! Executes the CLI binary with various --help flags to extract:
//! - Global options
//! - Command descriptions
//! - Subcommand hierarchies
//! - Argument specifications
//! - Usage examples
//!
//! # Why
//!
//! Automated documentation generation ensures:
//! - Documentation stays in sync with code
//! - No manual duplication of help text
//! - Consistent formatting across all commands
//! - Easy updates when commands change
//!
//! # Usage
//!
//! ```bash
//! # Run after building CLI
//! cargo build --release
//! cargo run --bin generate_command_docs
//!
//! # Or directly with rust-script
//! rust-script scripts/generate_command_docs.rs
//! ```
//!
//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! clap = { version = "4.4", features = ["derive"] }
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Main entry point for documentation generation.
fn main() -> Result<()> {
    println!("Generating command documentation from CLI help text...\n");

    // Find the CLI binary
    let binary_path = find_cli_binary()?;
    println!("Found CLI binary: {}", binary_path.display());

    // Extract help text for all commands
    println!("\nExtracting help text...");
    let global_help = extract_help(&binary_path, &[])?;
    let commands = discover_commands(&global_help)?;

    println!("Discovered commands: {}", commands.join(", "));

    // Extract help for each command
    let mut command_help = Vec::new();
    for cmd in &commands {
        println!("  Extracting: {}", cmd);
        let help = extract_help(&binary_path, &[cmd, "--help"])?;
        command_help.push((cmd.clone(), help));

        // Check for subcommands
        let subcommands = discover_subcommands(&help)?;
        for subcmd in subcommands {
            println!("    Extracting: {} {}", cmd, subcmd);
            let sub_help = extract_help(&binary_path, &[cmd, &subcmd, "--help"])?;
            command_help.push((format!("{} {}", cmd, subcmd), sub_help));
        }
    }

    // Generate markdown documentation
    println!("\nGenerating markdown...");
    let markdown = generate_markdown(&global_help, &command_help)?;

    // Write to file
    let output_path = Path::new("docs/COMMANDS_GENERATED.md");
    fs::write(output_path, markdown).context("Failed to write generated documentation")?;

    println!("\n✓ Documentation generated: {}", output_path.display());
    println!("\nCompare with existing COMMANDS.md:");
    println!("  diff docs/COMMANDS.md docs/COMMANDS_GENERATED.md");

    Ok(())
}

/// Finds the CLI binary in target directory.
fn find_cli_binary() -> Result<PathBuf> {
    // Check release build first
    let release_path = PathBuf::from("target/release/workspace");
    if release_path.exists() {
        return Ok(release_path);
    }

    // Check debug build
    let debug_path = PathBuf::from("target/debug/workspace");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    // Check platform-specific extensions
    if cfg!(windows) {
        let release_path = PathBuf::from("target/release/workspace.exe");
        if release_path.exists() {
            return Ok(release_path);
        }

        let debug_path = PathBuf::from("target/debug/workspace.exe");
        if debug_path.exists() {
            return Ok(debug_path);
        }
    }

    anyhow::bail!("CLI binary not found. Build it first with: cargo build --release");
}

/// Extracts help text for a command.
fn extract_help(binary: &Path, args: &[&str]) -> Result<String> {
    let mut cmd_args = vec!["--help"];
    cmd_args.extend_from_slice(args);

    let output =
        Command::new(binary).args(&cmd_args).output().context("Failed to execute CLI binary")?;

    if !output.status.success() {
        anyhow::bail!("CLI command failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Discovers available commands from global help text.
fn discover_commands(help_text: &str) -> Result<Vec<String>> {
    let mut commands = Vec::new();
    let mut in_commands_section = false;

    for line in help_text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Commands:") {
            in_commands_section = true;
            continue;
        }

        if in_commands_section {
            // Stop at next section or empty line
            if trimmed.is_empty() || !line.starts_with(' ') {
                break;
            }

            // Extract command name (first word after whitespace)
            if let Some(cmd) = trimmed.split_whitespace().next() {
                // Skip help command
                if cmd != "help" {
                    commands.push(cmd.to_string());
                }
            }
        }
    }

    Ok(commands)
}

/// Discovers subcommands from command help text.
fn discover_subcommands(help_text: &str) -> Result<Vec<String>> {
    let mut subcommands = Vec::new();
    let mut in_subcommands_section = false;

    for line in help_text.lines() {
        let trimmed = line.trim();

        // Look for "Commands:" or "Subcommands:" section
        if trimmed.starts_with("Commands:") || trimmed.starts_with("Subcommands:") {
            in_subcommands_section = true;
            continue;
        }

        if in_subcommands_section {
            // Stop at next section or empty line
            if trimmed.is_empty() || !line.starts_with(' ') {
                break;
            }

            // Extract subcommand name
            if let Some(subcmd) = trimmed.split_whitespace().next() {
                if subcmd != "help" {
                    subcommands.push(subcmd.to_string());
                }
            }
        }
    }

    Ok(subcommands)
}

/// Generates markdown documentation from help text.
fn generate_markdown(global_help: &str, commands: &[(String, String)]) -> Result<String> {
    let mut md = String::new();

    // Header
    md.push_str("# Workspace Tools - Command Reference (Generated)\n\n");
    md.push_str("**Generated from CLI help text**\n\n");
    md.push_str("⚠️  **Note**: This file is auto-generated. ");
    md.push_str("For human-friendly documentation, see [COMMANDS.md](./COMMANDS.md)\n\n");
    md.push_str("---\n\n");

    // Global help
    md.push_str("## Global Options\n\n");
    md.push_str("```\n");
    md.push_str(global_help);
    md.push_str("\n```\n\n");
    md.push_str("---\n\n");

    // Commands
    md.push_str("## Commands\n\n");

    for (name, help) in commands {
        // Determine heading level (2 for commands, 3 for subcommands)
        let heading = if name.contains(' ') { "###" } else { "##" };

        md.push_str(&format!("{} `{}`\n\n", heading, name));
        md.push_str("```\n");
        md.push_str(help);
        md.push_str("\n```\n\n");
        md.push_str("---\n\n");
    }

    // Footer
    md.push_str("## Generation Info\n\n");
    md.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    md.push_str("To regenerate:\n");
    md.push_str("```bash\n");
    md.push_str("cargo run --bin generate_command_docs\n");
    md.push_str("```\n");

    Ok(md)
}
