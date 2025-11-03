//! Upgrade rollback command implementation.
//!
//! This module implements backup management commands for upgrade rollback functionality.
//!
//! # What
//!
//! Provides:
//! - Listing available upgrade backups with metadata
//! - Restoring package.json files from specific backups
//! - Cleaning up old backups to manage disk space
//! - Validation of backup integrity before restore
//! - Formatted output (table or JSON)
//!
//! # How
//!
//! The backup commands:
//! 1. Use `sublime-package-tools` BackupManager for backup operations
//! 2. Load backup configuration from workspace settings
//! 3. List backups with timestamps and files
//! 4. Restore backups with confirmation and validation
//! 5. Clean old backups respecting retention policy
//! 6. Format output as tables or JSON based on user preference
//!
//! # Why
//!
//! Backup management is critical for safe dependency upgrades:
//! - Provides rollback capability when upgrades fail
//! - Maintains backup history with metadata
//! - Prevents disk space issues with cleanup
//! - Validates backup integrity before restore
//! - Gives visibility into available restore points
//!
//! # Examples
//!
//! ```bash
//! # List all available backups
//! wnt upgrade backups list
//!
//! # Restore a specific backup
//! wnt upgrade backups restore backup_20240115_103045
//!
//! # Restore with force (skip confirmation)
//! wnt upgrade backups restore backup_20240115_103045 --force
//!
//! # Clean old backups, keeping last 10
//! wnt upgrade backups clean --keep 10
//!
//! # JSON output for programmatic use
//! wnt upgrade backups list --format json
//! ```

use crate::cli::commands::{
    UpgradeBackupCleanArgs, UpgradeBackupListArgs, UpgradeBackupRestoreArgs,
};
use crate::error::{CliError, Result};
use crate::interactive::prompts::prompt_confirm;
use crate::output::{JsonResponse, Output, table::TableBuilder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::upgrade::BackupManager;
use sublime_standard_tools::filesystem::FileSystemManager;
use tracing::{debug, info, instrument};

/// Metadata for a backup entry.
///
/// Contains information about a backup including its ID, creation time,
/// and backed up files.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::rollback::BackupInfo;
/// use chrono::Utc;
/// use std::path::PathBuf;
///
/// let backup = BackupInfo {
///     id: "backup_20240115_103045".to_string(),
///     created_at: Utc::now(),
///     files: vec![PathBuf::from("packages/core/package.json")],
///     file_count: 1,
/// };
///
/// assert_eq!(backup.id, "backup_20240115_103045");
/// assert_eq!(backup.file_count, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Unique backup identifier
    pub id: String,

    /// When the backup was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// List of files included in the backup
    pub files: Vec<PathBuf>,

    /// Number of files in the backup
    #[serde(rename = "fileCount")]
    pub file_count: usize,
}

/// JSON response structure for backup list command.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::rollback::BackupListResponse;
///
/// let response = BackupListResponse {
///     success: true,
///     backups: vec![],
///     total: 0,
/// };
///
/// assert!(response.success);
/// assert_eq!(response.total, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupListResponse {
    /// Whether the command succeeded
    pub success: bool,

    /// List of available backups
    pub backups: Vec<BackupInfo>,

    /// Total number of backups
    pub total: usize,
}

/// JSON response structure for backup restore command.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::rollback::BackupRestoreResponse;
///
/// let response = BackupRestoreResponse {
///     success: true,
///     backup_id: "backup_20240115_103045".to_string(),
///     files_restored: 2,
/// };
///
/// assert!(response.success);
/// assert_eq!(response.files_restored, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRestoreResponse {
    /// Whether the command succeeded
    pub success: bool,

    /// ID of the restored backup
    #[serde(rename = "backupId")]
    pub backup_id: String,

    /// Number of files that were restored
    #[serde(rename = "filesRestored")]
    pub files_restored: usize,
}

/// JSON response structure for backup clean command.
///
/// # Examples
///
/// ```rust
/// use sublime_cli_tools::commands::upgrade::rollback::BackupCleanResponse;
///
/// let response = BackupCleanResponse {
///     success: true,
///     message: "Cleaned 5 backups".to_string(),
/// };
///
/// assert!(response.success);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCleanResponse {
    /// Whether the command succeeded
    pub success: bool,

    /// Message describing what was cleaned
    pub message: String,
}

/// Executes the backup list command.
///
/// Lists all available backups with their metadata including creation time
/// and files.
///
/// # Arguments
///
/// * `_args` - Command arguments from CLI (currently unused)
/// * `output` - Output context for formatting
/// * `workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns an error if:
/// - Workspace root is invalid
/// - Configuration cannot be loaded
/// - Backup manager initialization fails
/// - Listing backups fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::upgrade::rollback::execute_backup_list;
/// use sublime_cli_tools::cli::commands::UpgradeBackupListArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpgradeBackupListArgs {};
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_backup_list(&args, &output, workspace_root).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(output), level = "debug")]
pub async fn execute_backup_list(
    #[allow(clippy::used_underscore_binding)] _args: &UpgradeBackupListArgs,
    output: &Output,
    workspace_root: &Path,
) -> Result<()> {
    info!("Listing upgrade backups");
    debug!("Workspace root: {}", workspace_root.display());

    // Load configuration
    let config = load_config().await?;

    // Create filesystem manager
    let fs = FileSystemManager::new();

    // Create backup manager
    let backup_manager =
        BackupManager::new(workspace_root.to_path_buf(), config.upgrade.backup.clone(), fs);

    // List backups
    debug!("Fetching backup list");
    let backups = backup_manager
        .list_backups()
        .await
        .map_err(|e| CliError::execution(format!("Failed to list backups: {e}")))?;

    debug!("Found {} backups", backups.len());

    // Convert to CLI types
    let backup_infos: Vec<BackupInfo> = backups
        .into_iter()
        .map(|b| BackupInfo {
            id: b.id,
            created_at: b.created_at,
            file_count: b.files.len(),
            files: b.files,
        })
        .collect();

    // Output results
    output_backup_list(output, backup_infos)?;

    info!("Backup list completed successfully");
    Ok(())
}

/// Executes the backup restore command.
///
/// Restores package.json files from a specific backup. Validates the backup
/// exists and prompts for confirmation unless --force is specified.
///
/// # Arguments
///
/// * `args` - Command arguments from CLI
/// * `output` - Output context for formatting
/// * `workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns an error if:
/// - Workspace root is invalid
/// - Configuration cannot be loaded
/// - Backup manager initialization fails
/// - Backup ID doesn't exist
/// - User cancels confirmation
/// - Restore operation fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::upgrade::rollback::execute_backup_restore;
/// use sublime_cli_tools::cli::commands::UpgradeBackupRestoreArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpgradeBackupRestoreArgs {
///     id: "backup_20240115_103045".to_string(),
///     force: false,
/// };
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_backup_restore(&args, &output, workspace_root).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(output), level = "debug")]
pub async fn execute_backup_restore(
    args: &UpgradeBackupRestoreArgs,
    output: &Output,
    workspace_root: &Path,
) -> Result<()> {
    info!("Restoring upgrade backup: {}", args.id);
    debug!("Workspace root: {}", workspace_root.display());
    debug!("Force: {}", args.force);

    // Load configuration
    let config = load_config().await?;

    // Create filesystem manager
    let fs = FileSystemManager::new();

    // Create backup manager
    let backup_manager =
        BackupManager::new(workspace_root.to_path_buf(), config.upgrade.backup.clone(), fs);

    // Verify backup exists
    debug!("Verifying backup exists: {}", args.id);
    let backups = backup_manager
        .list_backups()
        .await
        .map_err(|e| CliError::execution(format!("Failed to list backups: {e}")))?;

    let backup_metadata = backups
        .iter()
        .find(|b| b.id == args.id)
        .ok_or_else(|| CliError::validation(format!("Backup not found: {}", args.id)))?;

    info!("Found backup: {} files", backup_metadata.files.len());

    // Confirm with user (unless --force)
    if !args.force && output.format().is_human() {
        let should_proceed = confirm_restore(output, backup_metadata, output.no_color())?;
        if !should_proceed {
            info!("User cancelled backup restore");
            output.info("Restore cancelled")?;
            return Ok(());
        }
    }

    // Perform restore
    info!("Restoring backup: {}", args.id);
    backup_manager
        .restore_backup(&args.id)
        .await
        .map_err(|e| CliError::execution(format!("Failed to restore backup: {e}")))?;

    // Output results
    let files_restored = backup_metadata.files.len();

    output_restore_result(output, &args.id, files_restored)?;

    info!("Backup restore completed successfully");
    Ok(())
}

/// Executes the backup clean command.
///
/// Removes old backups while keeping a specified number of recent backups.
/// Respects the retention policy and prompts for confirmation unless --force
/// is specified.
///
/// # Arguments
///
/// * `args` - Command arguments from CLI
/// * `output` - Output context for formatting
/// * `workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns an error if:
/// - Workspace root is invalid
/// - Configuration cannot be loaded
/// - Backup manager initialization fails
/// - User cancels confirmation
/// - Cleanup operation fails
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::upgrade::rollback::execute_backup_clean;
/// use sublime_cli_tools::cli::commands::UpgradeBackupCleanArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpgradeBackupCleanArgs {
///     keep: 5,
///     force: false,
/// };
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// let workspace_root = Path::new(".");
///
/// execute_backup_clean(&args, &output, workspace_root).await?;
/// # Ok(())
/// # }
/// ```
#[instrument(skip(output), level = "debug")]
pub async fn execute_backup_clean(
    args: &UpgradeBackupCleanArgs,
    output: &Output,
    workspace_root: &Path,
) -> Result<()> {
    info!("Cleaning old upgrade backups (keep: {})", args.keep);
    debug!("Workspace root: {}", workspace_root.display());
    debug!("Force: {}", args.force);

    // Load configuration
    let config = load_config().await?;

    // Create filesystem manager
    let fs = FileSystemManager::new();

    // Create backup manager with updated max_backups
    let mut backup_config = config.upgrade.backup.clone();
    backup_config.max_backups = args.keep;

    let backup_manager = BackupManager::new(workspace_root.to_path_buf(), backup_config, fs);

    // Get current backup list
    debug!("Fetching backup list");
    let backups = backup_manager
        .list_backups()
        .await
        .map_err(|e| CliError::execution(format!("Failed to list backups: {e}")))?;

    let total_backups = backups.len();
    let backups_to_remove = total_backups.saturating_sub(args.keep);

    if backups_to_remove == 0 {
        info!("No backups to clean");
        output.info(&format!(
            "No backups to clean. Found {total_backups} backup(s), keeping {}",
            args.keep
        ))?;
        return Ok(());
    }

    // Confirm with user (unless --force)
    if !args.force && output.format().is_human() {
        let should_proceed = confirm_clean(output, backups_to_remove, output.no_color())?;
        if !should_proceed {
            info!("User cancelled backup clean");
            output.info("Clean cancelled")?;
            return Ok(());
        }
    }

    // Perform cleanup
    info!("Cleaning up old backups");
    backup_manager
        .cleanup_old_backups()
        .await
        .map_err(|e| CliError::execution(format!("Failed to clean backups: {e}")))?;

    // Output results
    let message = format!("Cleaned {backups_to_remove} old backup(s), kept {}", args.keep);
    output_clean_result(output, &message)?;

    info!("Backup clean completed successfully");
    Ok(())
}

/// Loads package tools configuration.
///
/// # Returns
///
/// * `Result<PackageToolsConfig>` - Loaded configuration
///
/// # Errors
///
/// Returns an error if configuration cannot be loaded or validated.
async fn load_config() -> Result<PackageToolsConfig> {
    sublime_pkg_tools::config::load_config()
        .await
        .map_err(|e| CliError::configuration(format!("Failed to load configuration: {e}")))
}

/// Prompts the user to confirm backup restore.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backup` - Backup metadata
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<bool>` - True if user confirms, false otherwise
///
/// # Errors
///
/// Returns an error if user input fails.
fn confirm_restore(
    output: &Output,
    backup: &sublime_pkg_tools::upgrade::BackupMetadata,
    no_color: bool,
) -> Result<bool> {
    use console::style;

    output.blank_line()?;
    let message = style(format!(
        "This will restore {} file(s) from backup: {}",
        backup.files.len(),
        backup.id
    ))
    .yellow();
    output.plain(&message.to_string())?;

    output.plain(&format!("  Created: {}", backup.created_at.format("%Y-%m-%d %H:%M:%S UTC")))?;
    output.plain("  Files:")?;
    for file in &backup.files {
        if let Some(filename) = file.file_name() {
            output.plain(&format!("    - {}", filename.to_string_lossy()))?;
        }
    }

    output.blank_line()?;
    output.plain(
        &style("WARNING: This will overwrite current package.json files!").red().bold().to_string(),
    )?;
    output.blank_line()?;

    // Use interactive prompt to confirm
    prompt_confirm("Do you want to proceed with the restore?", false, no_color)
}

/// Prompts the user to confirm backup cleanup.
///
/// # Arguments
///
/// * `output` - Output context
/// * `count` - Number of backups to remove
/// * `no_color` - Whether to disable colored output
///
/// # Returns
///
/// * `Result<bool>` - True if user confirms, false otherwise
///
/// # Errors
///
/// Returns an error if user input fails.
fn confirm_clean(output: &Output, count: usize, no_color: bool) -> Result<bool> {
    use console::style;

    output.blank_line()?;
    let message = style(format!(
        "This will remove {} old backup{}.",
        count,
        if count == 1 { "" } else { "s" },
    ))
    .yellow();
    output.plain(&message.to_string())?;
    output.blank_line()?;

    // Use interactive prompt to confirm
    prompt_confirm("Do you want to proceed?", true, no_color)
}

/// Outputs the backup list in the appropriate format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backups` - List of backups
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_backup_list(output: &Output, backups: Vec<BackupInfo>) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            output_backup_list_json(output, backups)
        }
        crate::output::OutputFormat::Human => output_backup_list_human(output, &backups),
        crate::output::OutputFormat::Quiet => output_backup_list_quiet(output, &backups),
    }
}

/// Outputs backup list in JSON format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backups` - List of backups
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_backup_list_json(output: &Output, backups: Vec<BackupInfo>) -> Result<()> {
    let total = backups.len();
    let response = BackupListResponse { success: true, backups, total };

    let json_response = JsonResponse::success(response);
    output.json(&json_response)
}

/// Outputs backup list in human-readable format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backups` - List of backups
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_backup_list_human(output: &Output, backups: &[BackupInfo]) -> Result<()> {
    use console::style;

    if backups.is_empty() {
        output.info("No backups found")?;
        output.plain("Run upgrades with `wnt upgrade apply` to create backups automatically.")?;
        return Ok(());
    }

    // Header
    output.plain(&style("Available Backups").bold().cyan().to_string())?;
    output.plain("━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
    output.blank_line()?;

    // Table
    let mut table = TableBuilder::new().columns(&["Backup ID", "Created", "Files"]).build();

    for backup in backups {
        let created = backup.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let file_count = backup.file_count.to_string();

        table.add_row(&[&backup.id, &created, &file_count]);
    }

    output.table(&mut table)?;
    output.blank_line()?;

    // Summary
    output.plain(&format!("Total backups: {}", backups.len()))?;
    output.plain(
        &style("  Use `wnt upgrade backups restore <ID>` to restore a backup").dim().to_string(),
    )?;

    Ok(())
}

/// Outputs backup list in quiet format.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backups` - List of backups
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_backup_list_quiet(output: &Output, backups: &[BackupInfo]) -> Result<()> {
    for backup in backups {
        output.plain(&backup.id)?;
    }
    Ok(())
}

/// Outputs backup restore result.
///
/// # Arguments
///
/// * `output` - Output context
/// * `backup_id` - Restored backup ID
/// * `files` - Number of files restored
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_restore_result(output: &Output, backup_id: &str, files: usize) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            let response = BackupRestoreResponse {
                success: true,
                backup_id: backup_id.to_string(),
                files_restored: files,
            };
            let json_response = JsonResponse::success(response);
            output.json(&json_response)
        }
        crate::output::OutputFormat::Human => {
            output.blank_line()?;
            output.success(&format!("Successfully restored backup: {backup_id}"))?;
            output.blank_line()?;
            output.plain(&format!("Total files restored: {files}"))?;
            output.blank_line()?;
            output.info("Remember to run `npm install` to sync dependencies")?;

            Ok(())
        }
        crate::output::OutputFormat::Quiet => {
            output.plain(&format!("restored {files}"))?;
            Ok(())
        }
    }
}

/// Outputs backup clean result.
///
/// # Arguments
///
/// * `output` - Output context
/// * `message` - Message describing the cleanup result
///
/// # Returns
///
/// * `Result<()>` - Success or error
fn output_clean_result(output: &Output, message: &str) -> Result<()> {
    match output.format() {
        crate::output::OutputFormat::Json | crate::output::OutputFormat::JsonCompact => {
            let response = BackupCleanResponse { success: true, message: message.to_string() };
            let json_response = JsonResponse::success(response);
            output.json(&json_response)
        }
        crate::output::OutputFormat::Human => {
            output.blank_line()?;
            output.success(message)?;
            Ok(())
        }
        crate::output::OutputFormat::Quiet => {
            output.plain(message)?;
            Ok(())
        }
    }
}
