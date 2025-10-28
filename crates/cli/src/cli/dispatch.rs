//! Command dispatcher for routing parsed commands to their handlers.
//!
//! This module provides the central dispatch logic that routes parsed CLI
//! commands to their appropriate handler functions in the commands module.
//!
//! # What
//!
//! Provides:
//! - `dispatch_command` function to route commands to handlers
//! - Centralized command execution logic
//! - Error handling and context passing
//! - Async command execution support
//!
//! # How
//!
//! Takes the parsed `Commands` enum and matches it to the appropriate
//! handler function. Each handler receives the global context and command
//! arguments, executes the command logic, and returns a result.
//!
//! # Why
//!
//! Centralizing dispatch logic:
//! - Separates parsing from execution
//! - Provides a single point for cross-cutting concerns
//! - Makes testing command routing easier
//! - Keeps main.rs clean and focused
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::cli::{Cli, dispatch_command};
//! use sublime_cli_tools::error::Result;
//!
//! async fn run_cli() -> Result<()> {
//!     let cli = Cli::parse();
//!     dispatch_command(&cli).await
//! }
//! ```

use crate::cli::{Cli, Commands};
use crate::commands::init;
use crate::error::Result;
use std::path::Path;

/// Dispatches a parsed command to its handler.
///
/// This function takes the parsed CLI arguments and routes the command
/// to the appropriate handler function. All handlers are async and return
/// a `Result` for consistent error handling.
///
/// # Arguments
///
/// * `cli` - The parsed CLI arguments including global options and command
///
/// # Returns
///
/// Returns `Ok(())` if the command executed successfully, or an error
/// if execution failed.
///
/// # Errors
///
/// Returns an error if:
/// - The command handler encounters an error
/// - Required resources are not available
/// - Configuration is invalid
/// - Git operations fail
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::cli::{Cli, dispatch_command};
/// use clap::Parser;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let cli = Cli::parse();
/// dispatch_command(&cli).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::todo)]
pub async fn dispatch_command(cli: &Cli) -> Result<()> {
    use crate::cli::commands::{ConfigCommands, UpgradeBackupCommands, UpgradeCommands};

    // Extract global options
    let root = cli.root.as_deref().unwrap_or_else(|| Path::new("."));
    let format = cli.output_format();

    match &cli.command {
        Commands::Init(args) => {
            init::execute_init(args, root, format).await?;
        }

        Commands::Config(config_cmd) => {
            let _ = (root, format); // Will be used when implemented
            match config_cmd {
                ConfigCommands::Show(_args) => {
                    // TODO: will be implemented on story 2.2
                    todo!("Config show command will be implemented in story 2.2")
                }
                ConfigCommands::Validate(_args) => {
                    // TODO: will be implemented on story 2.3
                    todo!("Config validate command will be implemented in story 2.3")
                }
            }
        }

        Commands::Changeset(changeset_cmd) => {
            use crate::cli::commands::ChangesetCommands;
            match changeset_cmd {
                ChangesetCommands::Create(_args) => {
                    // TODO: will be implemented on story 4.1
                    todo!("Changeset create command will be implemented in story 4.1")
                }
                ChangesetCommands::Update(_args) => {
                    // TODO: will be implemented on story 4.5
                    todo!("Changeset update command will be implemented in story 4.5")
                }
                ChangesetCommands::List(_args) => {
                    // TODO: will be implemented on story 4.3
                    todo!("Changeset list command will be implemented in story 4.3")
                }
                ChangesetCommands::Show(_args) => {
                    // TODO: will be implemented on story 4.4
                    todo!("Changeset show command will be implemented in story 4.4")
                }
                ChangesetCommands::Delete(_args) => {
                    // TODO: will be implemented on story 4.7
                    todo!("Changeset delete command will be implemented in story 4.7")
                }
                ChangesetCommands::History(_args) => {
                    // TODO: will be implemented on story 4.8
                    todo!("Changeset history command will be implemented in story 4.8")
                }
                ChangesetCommands::Check(_args) => {
                    // TODO: will be implemented on story 4.3
                    todo!("Changeset check command will be implemented in story 4.3")
                }
            }
        }

        Commands::Bump(_args) => {
            // TODO: will be implemented on story 5.1 and 5.2
            todo!("Bump command will be implemented in story 5.1 and 5.2")
        }

        Commands::Upgrade(upgrade_cmd) => {
            match upgrade_cmd {
                UpgradeCommands::Check(_args) => {
                    // TODO: will be implemented on story 6.1
                    todo!("Upgrade check command will be implemented in story 6.1")
                }
                UpgradeCommands::Apply(_args) => {
                    // TODO: will be implemented on story 6.2
                    todo!("Upgrade apply command will be implemented in story 6.2")
                }
                UpgradeCommands::Backups(backup_cmd) => match backup_cmd {
                    UpgradeBackupCommands::List(_args) => {
                        // TODO: will be implemented on story 6.2
                        todo!("Upgrade backups list command will be implemented in story 6.2")
                    }
                    UpgradeBackupCommands::Restore(_args) => {
                        // TODO: will be implemented on story 6.3
                        todo!("Upgrade backups restore command will be implemented in story 6.3")
                    }
                    UpgradeBackupCommands::Clean(_args) => {
                        // TODO: will be implemented on story 6.2
                        todo!("Upgrade backups clean command will be implemented in story 6.2")
                    }
                },
            }
        }

        Commands::Audit(_args) => {
            // TODO: will be implemented on story 7.1
            todo!("Audit command will be implemented in story 7.1")
        }

        Commands::Changes(_args) => {
            // TODO: will be implemented on story 5.4
            todo!("Changes command will be implemented in story 5.4")
        }

        Commands::Version(_args) => {
            let _ = (root, format); // Will be used when implemented
            // TODO: will be implemented on story 1.4 (basic version display)
            todo!("Version command will be implemented in story 1.4")
        }
    }

    Ok(())
}
