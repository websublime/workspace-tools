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
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]
#![allow(clippy::exit)] // Required for main entry point

use std::process;
use sublime_cli::error::Result;

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
/// This function will:
/// 1. Parse CLI arguments
/// 2. Initialize logging
/// 3. Dispatch to command handlers
/// 4. Return results
///
/// # Errors
///
/// Returns `CliError` for any operational failures.
#[allow(clippy::unused_async)] // TODO: will be implemented in story 1.4
async fn async_main() -> Result<()> {
    // TODO: will be implemented in story 1.4 (CLI Framework)
    // 1. Parse CLI arguments with clap
    // 2. Initialize logging based on --log-level
    // 3. Create global context from arguments
    // 4. Dispatch to command handler
    // 5. Handle output formatting based on --format

    // For now, return Ok to allow compilation
    Ok(())
}
