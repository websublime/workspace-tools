//! Changeset edit command implementation.
//!
//! This module implements the `changeset edit` command for manually editing changesets.
//!
//! # What
//!
//! Provides the `execute_edit` function that:
//! - Opens a changeset file in the user's preferred text editor
//! - Allows manual editing of changeset content
//! - Validates the edited changeset before saving
//! - Detects and prevents invalid modifications
//! - Handles concurrent modifications and file system errors
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Determines which changeset to edit (from parameter or current git branch)
//! 3. Locates the changeset file on disk
//! 4. Opens the file in the user's preferred editor (using editor utility)
//! 5. Waits for the editor to close
//! 6. Reloads and validates the edited changeset
//! 7. Saves the validated changeset or reports validation errors
//! 8. Outputs success message or detailed error information
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset operations
//! - `Repo` from git tools for branch detection
//! - Editor utility for opening files in user's editor
//! - Styled output sections for human-readable display
//! - JSON serialization for machine-readable output
//!
//! # Why
//!
//! Manual changeset editing is essential for:
//! - Making complex changes that are difficult to express via CLI flags
//! - Bulk editing of changeset properties
//! - Advanced users who prefer direct file manipulation
//! - Fixing malformed or problematic changesets
//! - Providing maximum flexibility and control
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_edit;
//! use sublime_cli_tools::cli::commands::ChangesetEditArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetEditArgs {
//!     branch: Some("feature/new-api".to_string()),
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_edit(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetEditArgs;
use crate::error::{CliError, Result};
use crate::output::styling::{Section, print_item};
use crate::output::{JsonResponse, Output};
use crate::utils::editor::open_in_editor;
use serde::Serialize;
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_pkg_tools::types::Changeset;
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Response data for changeset edit command (JSON output).
///
/// Contains information about the edit operation result.
#[derive(Debug, Serialize)]
struct ChangesetEditResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// The branch name edited.
    branch: String,
    /// The changeset details after editing.
    changeset: ChangesetInfo,
}

/// Changeset information for output.
#[derive(Debug, Serialize)]
struct ChangesetInfo {
    /// Branch name (also serves as unique identifier).
    branch: String,
    /// Version bump type.
    bump: String,
    /// List of affected packages.
    packages: Vec<String>,
    /// Target environments.
    environments: Vec<String>,
    /// List of commit IDs.
    commits: Vec<String>,
    /// Creation timestamp (RFC3339 format).
    created_at: String,
    /// Last update timestamp (RFC3339 format).
    updated_at: String,
}

impl From<Changeset> for ChangesetInfo {
    fn from(changeset: Changeset) -> Self {
        Self {
            branch: changeset.branch,
            bump: changeset.bump.to_string().to_lowercase(),
            packages: changeset.packages,
            environments: changeset.environments,
            commits: changeset.changes,
            created_at: changeset.created_at.to_rfc3339(),
            updated_at: changeset.updated_at.to_rfc3339(),
        }
    }
}

/// Execute the changeset edit command.
///
/// Opens the specified changeset in the user's preferred text editor,
/// waits for editing to complete, then validates and saves the changes.
///
/// # Arguments
///
/// * `args` - Command arguments including the branch name to edit
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
/// - No editor can be detected
/// - The editor process fails to start or exits with an error
/// - The edited changeset is invalid
/// - The changeset file cannot be read, written, or parsed
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_edit;
/// use sublime_cli_tools::cli::commands::ChangesetEditArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ChangesetEditArgs {
///     branch: Some("feature/my-feature".to_string()),
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_edit(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_edit(
    args: &ChangesetEditArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    info!("Executing changeset edit command");
    debug!("Args: {:?}", args);

    // Load configuration
    let (config, workspace_root) = load_config(root, config_path).await?;
    debug!("Configuration loaded from: {}", workspace_root.display());

    // Determine branch
    let branch = if let Some(ref b) = args.branch {
        b.clone()
    } else {
        detect_current_branch(&workspace_root)?
    };
    debug!("Editing changeset for branch: {}", branch);

    // Create filesystem manager and changeset manager
    let manager =
        ChangesetManager::new(workspace_root.clone(), FileSystemManager::new(), config.clone())
            .await
            .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    // Load changeset before editing (to capture original state and verify it exists)
    let original_changeset = manager.load(&branch).await.map_err(|e| {
        CliError::validation(format!(
            "Changeset not found for branch '{branch}'. Use 'wnt changeset create' to create one. Error: {e}"
        ))
    })?;
    debug!("Original changeset loaded");

    // Get the changeset file path
    let changeset_path = get_changeset_file_path(&workspace_root, &config, &branch);
    debug!("Changeset file path: {}", changeset_path.display());

    if !changeset_path.exists() {
        return Err(CliError::io(format!(
            "Changeset file not found: {}",
            changeset_path.display()
        )));
    }

    // Open in editor
    info!("Opening changeset file in editor");
    let _ = output.info("Opening changeset in editor...");

    open_in_editor(&changeset_path)?;

    info!("Editor closed, validating changes");
    let _ = output.info("Editor closed. Validating changes...");

    // Reload and validate the edited changeset
    match validate_edited_changeset(&manager, &branch).await {
        Ok(edited_changeset) => {
            debug!("Changeset validation successful");
            output_results(output, &branch, &edited_changeset)?;
            Ok(())
        }
        Err(e) => {
            warn!("Changeset validation failed: {}", e);
            // Try to restore original changeset
            match manager.update(&original_changeset).await {
                Ok(()) => {
                    debug!("Original changeset restored after validation failure");
                    Err(CliError::validation(format!(
                        "Edited changeset is invalid and has been reverted: {e}"
                    )))
                }
                Err(restore_err) => {
                    warn!("Failed to restore original changeset: {}", restore_err);
                    Err(CliError::execution(format!(
                        "Edited changeset is invalid: {e}. \
                         Additionally, failed to restore original: {restore_err}"
                    )))
                }
            }
        }
    }
}

/// Loads the workspace configuration.
///
/// # Arguments
///
/// * `root` - Optional workspace root directory
/// * `config_path` - Optional path to configuration file
///
/// # Returns
///
/// Returns a tuple of (config, workspace_root) on success.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file cannot be found or loaded
/// - Configuration is invalid
/// - File system operations fail
async fn load_config(
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<(PackageToolsConfig, PathBuf)> {
    let workspace_root = root.map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        std::path::Path::to_path_buf,
    );

    debug!("Loading configuration from workspace root: {}", workspace_root.display());

    let config = if let Some(config_file) = config_path {
        debug!("Using specified config file: {}", config_file.display());
        ConfigLoader::load_from_file(config_file)
            .await
            .map_err(|e| CliError::configuration(format!("Failed to load config file: {e}")))?
    } else {
        debug!("Auto-detecting configuration file");
        let fs = FileSystemManager::new();

        // Search for standard config file names
        let candidates = vec![
            workspace_root.join("repo.config.toml"),
            workspace_root.join("repo.config.json"),
            workspace_root.join("repo.config.yaml"),
            workspace_root.join("repo.config.yml"),
        ];

        let mut found_config = None;
        for candidate in candidates {
            if fs.exists(&candidate).await {
                found_config = Some(candidate);
                break;
            }
        }

        if let Some(config_path) = found_config {
            ConfigLoader::load_from_file(&config_path).await.map_err(|e| {
                CliError::configuration(format!(
                    "Failed to load configuration from {}: {e}",
                    config_path.display()
                ))
            })?
        } else {
            return Err(CliError::configuration(
                "Workspace not initialized. Run 'wnt init' first.".to_string(),
            ));
        }
    };

    debug!("Configuration loaded successfully");
    Ok((config, workspace_root))
}

/// Detects the current Git branch.
///
/// # Arguments
///
/// * `workspace_root` - The workspace root directory
///
/// # Returns
///
/// Returns the current branch name.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a Git repository
/// - Cannot determine current branch
fn detect_current_branch(workspace_root: &Path) -> Result<String> {
    debug!("Detecting current Git branch");

    let workspace_str = workspace_root
        .to_str()
        .ok_or_else(|| CliError::io("Workspace path contains invalid UTF-8 characters"))?;

    let repo = Repo::open(workspace_str).map_err(|e| {
        CliError::git(format!(
            "Failed to open Git repository at '{}': {e}",
            workspace_root.display()
        ))
    })?;

    let branch = repo.get_current_branch().map_err(|e| {
        CliError::git(format!(
            "Failed to detect current branch. Are you in a Git repository? Error: {e}"
        ))
    })?;

    debug!("Current branch: {}", branch);
    Ok(branch)
}

/// Gets the file path for a changeset.
///
/// # Arguments
///
/// * `workspace_root` - The workspace root directory
/// * `config` - The package tools configuration
/// * `branch` - The branch name
///
/// # Returns
///
/// Returns the path to the changeset file.
pub(crate) fn get_changeset_file_path(
    workspace_root: &Path,
    config: &PackageToolsConfig,
    branch: &str,
) -> PathBuf {
    let changeset_dir = workspace_root.join(&config.changeset.path);
    let sanitized_branch = sanitize_branch_name(branch);
    changeset_dir.join(format!("{sanitized_branch}.json"))
}

/// Sanitizes a branch name for use as a filename.
///
/// Replaces characters that are invalid in filenames with safe alternatives.
///
/// # Arguments
///
/// * `branch` - The branch name to sanitize
///
/// # Returns
///
/// Returns the sanitized branch name.
fn sanitize_branch_name(branch: &str) -> String {
    branch.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Validates an edited changeset by reloading it.
///
/// # Arguments
///
/// * `manager` - The changeset manager
/// * `branch` - The branch name
///
/// # Returns
///
/// Returns the validated changeset.
///
/// # Errors
///
/// Returns an error if:
/// - The changeset file cannot be read
/// - The changeset JSON is invalid
/// - The changeset fails validation
async fn validate_edited_changeset(
    manager: &ChangesetManager<
        sublime_pkg_tools::changeset::FileBasedChangesetStorage<FileSystemManager>,
    >,
    branch: &str,
) -> Result<Changeset> {
    debug!("Validating edited changeset for branch: {}", branch);

    // Attempt to reload the changeset
    let changeset = manager.load(branch).await.map_err(|e| {
        CliError::validation(format!(
            "Failed to parse edited changeset. The file may contain invalid JSON or \
             missing required fields: {e}"
        ))
    })?;

    // Additional validation checks
    if changeset.branch != branch {
        return Err(CliError::validation(format!(
            "Branch name mismatch: expected '{branch}', found '{}'. \
             The 'branch' field must not be changed.",
            changeset.branch
        )));
    }

    if changeset.packages.is_empty() {
        return Err(CliError::validation(
            "Changeset must contain at least one package. \
             The 'packages' array cannot be empty.",
        ));
    }

    if changeset.environments.is_empty() {
        return Err(CliError::validation(
            "Changeset must contain at least one environment. \
             The 'environments' array cannot be empty.",
        ));
    }

    debug!("Changeset validation successful");
    Ok(changeset)
}

/// Outputs the command results.
///
/// Formats output based on the output format (JSON or human-readable).
///
/// # Arguments
///
/// * `output` - Output handler
/// * `branch` - The branch name
/// * `changeset` - The edited changeset
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns an error if output formatting or writing fails.
fn output_results(output: &Output, branch: &str, changeset: &Changeset) -> Result<()> {
    if output.format().is_json() {
        output_json(output, branch, changeset)
    } else {
        output_formatted(output, branch, changeset)
    }
}

/// Outputs results in JSON format.
fn output_json(output: &Output, branch: &str, changeset: &Changeset) -> Result<()> {
    let response = ChangesetEditResponse {
        success: true,
        branch: branch.to_string(),
        changeset: changeset.clone().into(),
    };

    output.json(&JsonResponse::success(response))?;
    Ok(())
}

/// Outputs results in human-readable format.
fn output_formatted(output: &Output, branch: &str, changeset: &Changeset) -> Result<()> {
    let section = Section::new("Changeset Edited Successfully");
    section.print();

    output.blank_line().map_err(|e| CliError::execution(format!("Failed to write output: {e}")))?;

    print_item("Branch", &changeset.branch, false);
    print_item("Bump Type", &format_bump_type(changeset.bump), false);
    print_item("Packages", &format!("{} package(s)", changeset.packages.len()), false);
    print_item("Environments", &format!("{} environment(s)", changeset.environments.len()), false);
    print_item("Commits", &format!("{} commit(s)", changeset.changes.len()), true);

    output.blank_line().map_err(|e| CliError::execution(format!("Failed to write output: {e}")))?;

    output
        .success(&format!("✓ Changeset for '{branch}' has been updated successfully."))
        .map_err(|e| CliError::execution(format!("Failed to write output: {e}")))?;

    Ok(())
}

/// Formats a bump type for display.
pub(crate) fn format_bump_type(bump: sublime_pkg_tools::types::VersionBump) -> String {
    match bump {
        sublime_pkg_tools::types::VersionBump::Major => "major".to_string(),
        sublime_pkg_tools::types::VersionBump::Minor => "minor".to_string(),
        sublime_pkg_tools::types::VersionBump::Patch => "patch".to_string(),
        sublime_pkg_tools::types::VersionBump::None => "none".to_string(),
    }
}
