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

use super::branding;
use crate::cli::{Cli, Commands};
use crate::commands::{audit, bump, changeset, config, init, upgrade, version};
use crate::error::Result;
use crate::output::{Output, OutputFormat};
use std::path::{Path, PathBuf};

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
// Allow too_many_lines: This function serves as the central command dispatcher and needs to handle
// all CLI commands. Breaking it into smaller functions would reduce readability and make the
// command routing logic harder to follow. The length is justified by the need to dispatch to
// many different command implementations.
#[allow(clippy::too_many_lines)]
pub async fn dispatch_command(cli: &Cli) -> Result<()> {
    use crate::cli::commands::{ConfigCommands, UpgradeBackupCommands, UpgradeCommands};

    // Extract global options
    let root = cli.root.as_deref().unwrap_or_else(|| Path::new("."));
    let format = cli.output_format();
    let config_path = cli.config_path();

    // Display branded header for human-readable output (except for version command which handles its own header)
    if should_show_header(&cli.command, format) {
        branding::print_header(env!("CARGO_PKG_VERSION"));
    }

    match &cli.command {
        Commands::Init(args) => {
            init::execute_init(args, root, format).await?;
        }

        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show(args) => {
                config::execute_show(args, root, config_path.map(PathBuf::as_path), format).await?;
            }
            ConfigCommands::Validate(args) => {
                config::execute_validate(args, root, config_path.map(PathBuf::as_path), format)
                    .await?;
            }
        },

        Commands::Changeset(changeset_cmd) => {
            use crate::cli::commands::ChangesetCommands;
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());
            match changeset_cmd {
                ChangesetCommands::Create(args) => {
                    changeset::execute_add(
                        args,
                        &output,
                        Some(root.to_path_buf()),
                        config_path.map(PathBuf::from),
                    )
                    .await?;
                }
                ChangesetCommands::Update(args) => {
                    changeset::execute_update(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
                ChangesetCommands::List(args) => {
                    changeset::execute_list(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
                ChangesetCommands::Show(args) => {
                    changeset::execute_show(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
                ChangesetCommands::Edit(args) => {
                    changeset::execute_edit(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
                ChangesetCommands::Delete(args) => {
                    changeset::execute_remove(args, &output, Some(root), config_path.map(|v| &**v))
                        .await?;
                }
                ChangesetCommands::History(args) => {
                    changeset::execute_history(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
                ChangesetCommands::Check(args) => {
                    changeset::execute_check(
                        args,
                        &output,
                        Some(root),
                        config_path.as_ref().map(|p| p.as_path()),
                    )
                    .await?;
                }
            }
        }

        Commands::Bump(args) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());

            // Route to snapshot, execute, or preview mode based on flags
            if args.snapshot {
                // Snapshot mode - generate snapshot versions without consuming changesets
                bump::execute_bump_snapshot(
                    args,
                    &output,
                    root,
                    config_path.as_ref().map(|p| p.as_path()),
                )
                .await?;
            } else if args.execute {
                // Execute mode - apply version bumps
                bump::execute_bump_apply(
                    args,
                    &output,
                    root,
                    config_path.as_ref().map(|p| p.as_path()),
                )
                .await?;
            } else {
                // Preview mode (default) - dry-run
                bump::execute_bump_preview(
                    args,
                    &output,
                    root,
                    config_path.as_ref().map(|p| p.as_path()),
                )
                .await?;
            }
        }

        Commands::Upgrade(upgrade_cmd) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());
            match upgrade_cmd {
                UpgradeCommands::Check(args) => {
                    upgrade::execute_upgrade_check(args, &output, root).await?;
                }
                UpgradeCommands::Apply(args) => {
                    upgrade::execute_upgrade_apply(args, &output, root).await?;
                }
                UpgradeCommands::Backups(backup_cmd) => match backup_cmd {
                    UpgradeBackupCommands::List(args) => {
                        upgrade::execute_backup_list(args, &output, root).await?;
                    }
                    UpgradeBackupCommands::Restore(args) => {
                        upgrade::execute_backup_restore(args, &output, root).await?;
                    }
                    UpgradeBackupCommands::Clean(args) => {
                        upgrade::execute_backup_clean(args, &output, root).await?;
                    }
                },
            }
        }

        Commands::Audit(args) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());
            audit::execute_audit(args, &output, root, config_path.as_ref().map(|p| p.as_path()))
                .await?;
        }

        Commands::Changes(args) => {
            let output = Output::new(format, std::io::stdout(), cli.is_color_disabled());
            crate::commands::changes::execute_changes(
                args,
                &output,
                root,
                config_path.as_ref().map(|p| p.as_path()),
            )
            .await?;
        }

        Commands::Version(args) => {
            version::execute_version(args, root, format)?;
        }

        Commands::Clone(args) => {
            crate::commands::clone::execute_clone(args).await?;
        }
    }

    Ok(())
}

/// Determines if the branded header should be displayed.
///
/// The header is shown when:
/// - Output format is Human (not JSON, JSON-compact, or Quiet)
/// - Command is not `version` (version command shows header always, even in quiet modes)
///
/// # Arguments
///
/// * `command` - The command being executed
/// * `format` - The output format
///
/// # Returns
///
/// Returns `true` if the header should be displayed, `false` otherwise.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::cli::Commands;
/// use sublime_cli_tools::output::OutputFormat;
///
/// // Header shown for init command in human format
/// assert!(should_show_header(&Commands::Init(...), OutputFormat::Human));
///
/// // Header not shown for version command (handles its own header display)
/// assert!(!should_show_header(&Commands::Version(...), OutputFormat::Human));
///
/// // Header not shown for JSON output (prevents contamination)
/// assert!(!should_show_header(&Commands::Init(...), OutputFormat::Json));
/// ```
fn should_show_header(command: &Commands, format: OutputFormat) -> bool {
    // Only show header for human-readable output
    if !matches!(format, OutputFormat::Human) {
        return false;
    }

    // Version command shows header unconditionally (in its own implementation)
    !matches!(command, Commands::Version(_))
}
