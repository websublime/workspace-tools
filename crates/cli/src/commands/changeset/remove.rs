//! Changeset remove command implementation.
//!
//! This module implements the `changeset remove` command for deleting changesets.
//!
//! # What
//!
//! Provides the `execute_remove` function that:
//! - Deletes a specified changeset from the workspace
//! - Archives the changeset before deletion for recovery purposes
//! - Requires user confirmation by default (unless --force flag is used)
//! - Validates that the changeset exists before attempting deletion
//! - Provides clear feedback on what was deleted
//! - Handles errors gracefully with user-friendly messages
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Creates ChangesetManager to access changeset storage
//! 3. Verifies the changeset exists
//! 4. Loads the changeset to display what will be deleted
//! 5. Prompts for confirmation (unless --force is used)
//! 6. Archives the changeset with a special marker indicating manual deletion
//! 7. Deletes the changeset from active changesets
//! 8. Outputs success message with details of what was removed
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset operations
//! - `prompt_confirm` from interactive module for user confirmation
//! - Styled output sections for human-readable display
//! - JSON serialization for machine-readable output
//!
//! The archiving step creates a backup in the workspace history directory with a special
//! ReleaseInfo indicating this was a manual deletion, not a release. This allows
//! recovery of accidentally deleted changesets if needed.
//!
//! # Why
//!
//! Safe changeset deletion is essential for:
//! - Removing obsolete or incorrect changesets
//! - Cleaning up after mistakes or experimental work
//! - Maintaining a clean changeset directory
//! - Preventing accidental data loss through confirmation prompts
//! - Preserving deleted changesets for audit/recovery purposes
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_remove;
//! use sublime_cli_tools::cli::commands::ChangesetDeleteArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetDeleteArgs {
//!     branch: "feature/old-feature".to_string(),
//!     force: false,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_remove(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetDeleteArgs;
use crate::error::{CliError, Result};
use crate::interactive::prompts::prompt_confirm;
use crate::output::styling::{Section, print_item};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::types::{Changeset, ReleaseInfo};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Response data for changeset remove command (JSON output).
///
/// Contains information about the removed changeset.
#[derive(Debug, Serialize)]
struct ChangesetRemoveResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// The branch name that was removed.
    branch: String,
    /// Whether the changeset was archived before removal.
    archived: bool,
    /// Details of the removed changeset.
    changeset: RemovedChangesetInfo,
}

/// Information about the removed changeset for output.
#[derive(Debug, Serialize)]
pub(crate) struct RemovedChangesetInfo {
    /// Branch name (also serves as unique identifier).
    pub(crate) branch: String,
    /// Version bump type.
    pub(crate) bump: String,
    /// List of affected packages.
    pub(crate) packages: Vec<String>,
    /// Target environments.
    pub(crate) environments: Vec<String>,
    /// Number of commits.
    pub(crate) commit_count: usize,
}

impl From<&Changeset> for RemovedChangesetInfo {
    fn from(changeset: &Changeset) -> Self {
        Self {
            branch: changeset.branch.clone(),
            bump: changeset.bump.to_string().to_lowercase(),
            packages: changeset.packages.clone(),
            environments: changeset.environments.clone(),
            commit_count: changeset.changes.len(),
        }
    }
}

/// Execute the changeset remove command.
///
/// Removes a specified changeset after confirmation, archiving it for recovery.
/// Outputs results in either human-readable formatted view or JSON format.
///
/// # Arguments
///
/// * `args` - Command arguments including the branch name to remove and force flag
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Optional workspace root directory (defaults to current directory)
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no configuration found)
/// - The specified changeset does not exist
/// - User cancels the confirmation prompt
/// - Archiving fails
/// - Deletion fails
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_remove;
/// use sublime_cli_tools::cli::commands::ChangesetDeleteArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Remove with confirmation
/// let args = ChangesetDeleteArgs {
///     branch: "feature/my-feature".to_string(),
///     force: false,
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_remove(&args, &output, None, None).await?;
///
/// // Force remove without confirmation
/// let args_force = ChangesetDeleteArgs {
///     branch: "feature/another-feature".to_string(),
///     force: true,
/// };
/// execute_remove(&args_force, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_remove(
    args: &ChangesetDeleteArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));
    debug!("Removing changeset '{}' from workspace: {}", args.branch, workspace_root.display());

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;

    info!("Configuration loaded successfully");

    // Create changeset manager
    let manager =
        ChangesetManager::new(workspace_root.to_path_buf(), FileSystemManager::new(), config)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to create changeset manager: {e}")))?;

    // Check if changeset exists by trying to load it
    // The ChangesetManager doesn't have an exists() method, so we try to load

    // Load changeset details before deletion (for archiving and display)
    let changeset = manager.load(&args.branch).await.map_err(|e| {
        CliError::Execution(format!("Failed to load changeset '{}': {e}", args.branch))
    })?;

    debug!("Loaded changeset: {}", changeset.branch);

    // Prompt for confirmation unless force flag is set
    #[allow(clippy::if_not_else)]
    if !args.force {
        // Display changeset details
        if !output.format().is_json() {
            display_changeset_for_confirmation(output, &changeset)?;
        }

        // Prompt for confirmation
        let confirmation_message = format!(
            "Are you sure you want to delete changeset '{}'?\nThis will remove:\n  - {} package(s): {}\n  - {} commit(s)\n  - {} environment(s): {}",
            args.branch,
            changeset.packages.len(),
            changeset.packages.join(", "),
            changeset.changes.len(),
            changeset.environments.len(),
            changeset.environments.join(", ")
        );

        let confirmed = prompt_confirm(
            &confirmation_message,
            false, // default to No for safety
            output.no_color(),
        )?;

        if !confirmed {
            warn!("Changeset removal cancelled by user");
            if output.format().is_json() {
                output
                    .json(&JsonResponse::<()>::error("Operation cancelled by user".to_string()))?;
            } else {
                output.info("Operation cancelled")?;
            }
            return Ok(());
        }
    } else {
        debug!("Skipping confirmation due to --force flag");
    }

    info!("Archiving changeset before deletion");

    // Archive the changeset before deletion
    // Use a special ReleaseInfo to indicate manual deletion (not a release)
    let release_info = create_deletion_release_info();

    manager.archive(&args.branch, release_info).await.map_err(|e| {
        CliError::Execution(format!(
            "Failed to archive changeset '{}' before deletion: {e}",
            args.branch
        ))
    })?;

    debug!("Changeset archived successfully");

    // Delete the changeset
    manager.delete(&args.branch).await.map_err(|e| {
        CliError::Execution(format!("Failed to delete changeset '{}': {e}", args.branch))
    })?;

    info!("Changeset '{}' removed successfully", args.branch);

    // Output results
    output_results(output, &args.branch, &changeset, true)?;

    Ok(())
}

/// Load workspace configuration.
///
/// Attempts to load configuration from the workspace, trying multiple file names
/// and falling back to defaults if no config file is found.
///
/// # Arguments
///
/// * `workspace_root` - The workspace root directory
/// * `config_path` - Optional explicit config file path
///
/// # Returns
///
/// Returns the loaded `PackageToolsConfig`.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no config found)
/// - The config file exists but cannot be parsed
/// - File system operations fail
async fn load_config(
    workspace_root: &Path,
    config_path: Option<&Path>,
) -> Result<PackageToolsConfig> {
    debug!("Loading workspace configuration");

    let fs = FileSystemManager::new();

    // Try to find and load config file
    let mut found_config = None;
    if let Some(config) = config_path {
        found_config = Some(config.to_path_buf());
    } else {
        // Search for standard config file names
        let candidates = vec![
            workspace_root.join("repo.config.toml"),
            workspace_root.join("repo.config.json"),
            workspace_root.join("repo.config.yaml"),
            workspace_root.join("repo.config.yml"),
        ];

        for candidate in candidates {
            if fs.exists(&candidate).await {
                found_config = Some(candidate);
                break;
            }
        }
    }

    let config = if let Some(config_path) = found_config {
        // TODO: ConfigLoader will be implemented in future stories
        // For now, using a placeholder that matches the expected API
        match sublime_pkg_tools::config::ConfigLoader::load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_path.display());
                config
            }
            Err(e) => {
                return Err(CliError::Configuration(format!(
                    "Workspace not initialized. Run 'wnt init' first.\nDetails: {e}"
                )));
            }
        }
    } else {
        return Err(CliError::Configuration(
            "Workspace not initialized. Run 'wnt init' first.".to_string(),
        ));
    };

    Ok(config)
}

/// Create a ReleaseInfo structure for manual deletion archiving.
///
/// This creates a special ReleaseInfo that indicates the changeset was manually
/// deleted rather than consumed in a release. This allows the changeset to be
/// archived for recovery while clearly marking it as a manual deletion.
///
/// # Returns
///
/// Returns a `ReleaseInfo` with placeholder values indicating manual deletion.
pub(crate) fn create_deletion_release_info() -> ReleaseInfo {
    ReleaseInfo::new("manual-deletion", "not-released", HashMap::new())
}

/// Display changeset details for confirmation.
///
/// Shows the user what will be deleted so they can confirm it's the right changeset.
///
/// # Arguments
///
/// * `output` - Output handler for displaying information
/// * `changeset` - The changeset to display
///
/// # Errors
///
/// Returns an error if output operations fail.
fn display_changeset_for_confirmation(output: &Output, changeset: &Changeset) -> Result<()> {
    let section = Section::new("Changeset to be deleted");
    section.print();

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    print_item("Branch", &changeset.branch, false);
    print_item("Bump", &changeset.bump.to_string().to_lowercase(), false);
    print_item(
        "Packages",
        &format!("{} package(s): {}", changeset.packages.len(), changeset.packages.join(", ")),
        false,
    );
    print_item("Environments", &changeset.environments.join(", "), false);
    print_item("Commits", &format!("{} commit(s)", changeset.changes.len()), true);

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    Ok(())
}

/// Output the removal results.
///
/// Formats and displays the results of the changeset removal operation.
/// Outputs in JSON or human-readable format based on user preference.
///
/// # Arguments
///
/// * `output` - Output handler for formatting results
/// * `branch` - The branch name that was removed
/// * `changeset` - The changeset that was removed
/// * `archived` - Whether the changeset was successfully archived
///
/// # Errors
///
/// Returns an error if output operations fail.
fn output_results(
    output: &Output,
    branch: &str,
    changeset: &Changeset,
    archived: bool,
) -> Result<()> {
    if output.format().is_json() {
        output_json(output, branch, changeset, archived)
    } else {
        output_formatted(output, branch, changeset, archived)
    }
}

/// Output results in JSON format.
///
/// # Arguments
///
/// * `output` - Output handler for JSON serialization
/// * `branch` - The branch name that was removed
/// * `changeset` - The changeset that was removed
/// * `archived` - Whether the changeset was successfully archived
///
/// # Errors
///
/// Returns an error if JSON serialization or output fails.
fn output_json(output: &Output, branch: &str, changeset: &Changeset, archived: bool) -> Result<()> {
    let response = ChangesetRemoveResponse {
        success: true,
        branch: branch.to_string(),
        archived,
        changeset: RemovedChangesetInfo::from(changeset),
    };

    let json_response = JsonResponse::success(response);
    output.json(&json_response)?;
    Ok(())
}

/// Output results in human-readable format.
///
/// # Arguments
///
/// * `output` - Output handler for formatted display
/// * `branch` - The branch name that was removed
/// * `changeset` - The changeset that was removed
/// * `archived` - Whether the changeset was successfully archived
///
/// # Errors
///
/// Returns an error if output operations fail.
fn output_formatted(
    output: &Output,
    branch: &str,
    changeset: &Changeset,
    archived: bool,
) -> Result<()> {
    output.success(&format!("Changeset '{branch}' removed successfully"))?;
    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    let section = Section::new("Removed changeset");
    section.print();

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    print_item("Branch", branch, false);
    print_item("Bump", &changeset.bump.to_string().to_lowercase(), false);

    #[allow(clippy::if_not_else)]
    if !changeset.packages.is_empty() {
        print_item(
            "Packages",
            &format!("{} ({})", changeset.packages.len(), changeset.packages.join(", ")),
            false,
        );
    } else {
        print_item("Packages", "none", false);
    }

    print_item("Environments", &changeset.environments.join(", "), false);
    print_item("Commits", &format!("{} commit(s)", changeset.changes.len()), false);

    if archived {
        print_item("Archived", "yes (can be recovered from history)", true);
    } else {
        print_item("Archived", "no (archiving failed, but changeset removed)", true);
    }

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    Ok(())
}
