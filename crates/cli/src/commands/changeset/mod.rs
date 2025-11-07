//! Changeset command implementations module.
//!
//! This module contains all changeset-related command execution logic.
//!
//! # What
//!
//! Provides implementations for changeset commands:
//! - `add` - Create new changesets (interactive and non-interactive)
//! - `update` - Update existing changesets with new commits/packages
//! - `list` - List all active changesets
//! - `show` - Display detailed information about a changeset
//! - `delete` - Remove a changeset
//! - `history` - Query archived changesets
//! - `check` - Verify if a changeset exists for a branch
//!
//! # How
//!
//! Each command is implemented as an async function that:
//! 1. Validates arguments and loads workspace configuration
//! 2. Creates necessary managers (ChangesetManager, PackageDetector)
//! 3. Executes the changeset operation using the pkg tools library
//! 4. Formats and outputs results using the Output context
//! 5. Returns appropriate error codes on failures
//!
//! Commands use the interactive prompts module for user input in interactive mode,
//! and support non-interactive mode with command-line flags.
//!
//! # Why
//!
//! Centralizing changeset command logic provides:
//! - Consistent changeset workflow across all operations
//! - Reusable command implementations
//! - Comprehensive error handling with user-friendly messages
//! - Support for both interactive and automated (CI/CD) usage
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_add;
//! use sublime_cli_tools::cli::commands::ChangesetCreateArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetCreateArgs {
//!     bump: Some("minor".to_string()),
//!     env: Some(vec!["production".to_string()]),
//!     branch: None,
//!     message: Some("Add new feature".to_string()),
//!     packages: None,
//!     non_interactive: true,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_add(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

pub mod add;
pub mod check;
pub mod edit;
pub mod history;
pub mod list;
pub mod remove;
pub mod show;
pub mod update;

// Internal modules for shared functionality
pub(crate) mod common;
pub(crate) mod types;

#[cfg(test)]
mod tests;

// Re-export command functions for convenience
pub use add::execute_add;
pub use check::execute_check;
pub use edit::execute_edit;
pub use history::execute_history;
pub use list::execute_list;
pub use remove::execute_remove;
pub use show::execute_show;
pub use update::execute_update;
