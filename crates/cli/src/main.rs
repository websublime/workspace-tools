//! Main entry point for the Workspace Node Tools CLI.
//!
//! This binary provides the `wnt` command-line interface for managing Node.js
//! workspaces and monorepos with changeset-based version management.
//!
//! # What
//!
//! The main function:
//! - Initializes the tokio async runtime
//! - Parses command-line arguments using clap
//! - Sets up logging based on global options
//! - Dispatches to appropriate command handlers
//! - Handles errors and sets appropriate exit codes
//!
//! # How
//!
//! Uses tokio for async runtime, clap for argument parsing, and tracing for
//! logging. The main function is kept minimal - most logic is in command
//! implementations and modules.
//!
//! # Why
//!
//! Separating the entry point from command logic improves testability and
//! allows the library (lib.rs) to be used in other contexts without the
//! binary overhead.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// TODO: Re-enable after all commands are implemented
// #![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
#![allow(clippy::exit)] // Required for main entry point

use clap::Parser;
use std::process;
use sublime_cli_tools::cli::{Cli, dispatch_command};
use sublime_cli_tools::error::Result;

/// Main entry point for the CLI.
///
/// This function:
/// 1. Initializes the tokio async runtime
/// 2. Calls the async main function
/// 3. Handles the result and sets the exit code
///
/// # Examples
///
/// ```bash
/// wnt --help
/// wnt init
/// wnt changeset add
/// ```
fn main() {
    // Create tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build();

    let runtime = match runtime {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {e}");
            process::exit(1);
        }
    };

    // Run the async main function
    let result = runtime.block_on(async_main());

    // Handle the result and exit with appropriate code
    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {e}");
            let exit_code = e.exit_code();
            process::exit(exit_code);
        }
    }
}

/// Async main function that performs the actual CLI logic.
///
/// This function:
/// 1. Parses CLI arguments using clap
/// 2. Initializes logging based on `--log-level` (stderr only)
/// 3. Changes working directory if `--root` is specified
/// 4. Dispatches to the appropriate command handler
/// 5. Returns results for proper exit code handling
///
/// # Errors
///
/// Returns `CliError` for any operational failures including:
/// - Invalid CLI arguments
/// - Failed directory changes
/// - Command execution failures
/// - Configuration errors
/// - Git operation errors
///
/// # Examples
///
/// The function is called from `main()` within the tokio runtime:
///
/// ```rust,ignore
/// let result = runtime.block_on(async_main());
/// ```
async fn async_main() -> Result<()> {
    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // 2. Initialize logging based on --log-level (affects stderr only)
    sublime_cli_tools::output::logger::init_logging(cli.log_level(), cli.is_color_disabled())?;

    // 3. Change to root directory if specified
    if let Some(root) = cli.root() {
        std::env::set_current_dir(root).map_err(|e| {
            sublime_cli_tools::CliError::io(format!(
                "Failed to change directory to '{}': {}",
                root.display(),
                e
            ))
        })?;
    }

    // 4. Dispatch to command handler
    // Each command handler will:
    // - Receive the parsed arguments
    // - Execute the command logic
    // - Return results
    // - Handle output formatting based on global --format option
    dispatch_command(&cli).await?;

    Ok(())
}
