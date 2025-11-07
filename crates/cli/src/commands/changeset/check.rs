//! Changeset check command implementation.
//!
//! This module implements the `changeset check` command for verifying if a changeset
//! exists for a specific branch. This is particularly useful for Git hooks and CI/CD pipelines.
//!
//! # What
//!
//! Provides the `execute_check` function that:
//! - Checks if a changeset exists for the current or specified branch
//! - Returns appropriate exit codes for scripting (0 = exists, 1 = not exists)
//! - Supports optional branch parameter (defaults to current Git branch)
//! - Outputs minimal information suitable for automation
//! - Handles errors gracefully for use in Git hooks
//!
//! # How
//!
//! The command flow:
//! 1. Determines the target branch (from argument or current Git branch)
//! 2. Loads workspace configuration and validates initialization
//! 3. Creates ChangesetManager to access changeset storage
//! 4. Attempts to load the changeset for the specified branch
//! 5. Returns success (exit 0) if changeset exists, validation error (exit 1) if not
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset retrieval
//! - Git integration to detect current branch when not specified
//! - Special exit code handling for Git hook integration
//!
//! # Why
//!
//! Checking changeset existence is essential for:
//! - Git hooks enforcing changeset creation on feature branches
//! - CI/CD pipelines validating release requirements
//! - Pre-merge checks ensuring all changes are documented
//! - Automated validation in pull request workflows
//! - Branch protection rules requiring changesets
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_check;
//! use sublime_cli_tools::cli::commands::ChangesetCheckArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Check current branch
//! let args = ChangesetCheckArgs { branch: None };
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_check(&args, &output, None, None).await?;
//!
//! // Check specific branch
//! let args = ChangesetCheckArgs {
//!     branch: Some("feature/new-api".to_string()),
//! };
//! execute_check(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetCheckArgs;
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Response data for changeset check command (JSON output).
///
/// Contains the result of the check operation.
#[derive(Debug, Serialize)]
struct ChangesetCheckResponse {
    /// Whether the changeset exists.
    exists: bool,
    /// The branch that was checked.
    branch: String,
    /// Optional message describing the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Execute the changeset check command.
///
/// Verifies if a changeset exists for the current or specified branch.
/// This command is designed for use in Git hooks and CI/CD pipelines.
///
/// # Exit Codes
///
/// - `0`: Changeset exists for the branch
/// - `1`: Changeset does not exist (returned as CliError::Validation)
///
/// # Arguments
///
/// * `args` - Command arguments including optional branch name
/// * `output` - Output handler for formatting and displaying results
/// * `root` - Optional workspace root directory (defaults to current directory)
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(())` if changeset exists, or `Err(CliError::Validation)` if it does not.
/// Other errors indicate configuration or operational failures.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace is not initialized (no configuration found)
/// - The changeset does not exist (as `CliError::Validation` for exit code 1)
/// - Git operations fail when detecting current branch
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_check;
/// use sublime_cli_tools::cli::commands::ChangesetCheckArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check for changeset on current branch
/// let args = ChangesetCheckArgs { branch: None };
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
///
/// match execute_check(&args, &output, None, None).await {
///     Ok(()) => println!("Changeset exists"),
///     Err(e) => eprintln!("No changeset found: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
pub async fn execute_check(
    args: &ChangesetCheckArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));

    // Determine the branch to check
    let branch = if let Some(ref branch_name) = args.branch {
        branch_name.clone()
    } else {
        // Get current branch from Git
        detect_current_branch(workspace_root)?
    };

    debug!(
        "Checking for changeset on branch '{branch}' in workspace: {}",
        workspace_root.display()
    );

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;

    info!("Configuration loaded successfully");

    // Create changeset manager
    let manager =
        ChangesetManager::new(workspace_root.to_path_buf(), FileSystemManager::new(), config)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to create changeset manager: {e}")))?;

    // Check if changeset exists
    let exists = match manager.load(&branch).await {
        Ok(_changeset) => {
            debug!("Changeset found for branch '{branch}'");
            true
        }
        Err(_) => {
            debug!("No changeset found for branch '{branch}'");
            false
        }
    };

    // Output results
    if output.format().is_json() {
        output_json(output, &branch, exists)?;
    } else if !output.format().is_quiet() {
        output_human(output, &branch, exists)?;
    }

    // Return appropriate result for exit code handling
    if exists {
        info!("Changeset check succeeded: exists on branch '{branch}'");
        Ok(())
    } else {
        warn!("Changeset check failed: not found on branch '{branch}'");
        Err(CliError::Validation(format!("No changeset found for branch '{branch}'")))
    }
}

/// Detect the current Git branch.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
///
/// # Errors
///
/// Returns an error if:
/// - Git is not initialized in the workspace
/// - Cannot determine the current branch
/// - Workspace path contains invalid UTF-8 characters
fn detect_current_branch(workspace_root: &Path) -> Result<String> {
    debug!("Detecting current Git branch at: {}", workspace_root.display());

    let workspace_str = workspace_root.to_str().ok_or_else(|| {
        CliError::Execution("Workspace path contains invalid UTF-8 characters".to_string())
    })?;

    let repo = Repo::open(workspace_str).map_err(|e| {
        CliError::Execution(format!(
            "Failed to open Git repository at '{}': {e}",
            workspace_root.display()
        ))
    })?;

    let branch = repo.get_current_branch().map_err(|e| {
        CliError::Execution(format!(
            "Failed to detect current branch. Are you in a Git repository? Error: {e}"
        ))
    })?;

    debug!("Detected current branch: {}", branch);
    Ok(branch)
}

/// Load workspace configuration.
///
/// Attempts to load configuration from the workspace, trying multiple file names
/// and falling back to defaults if no config file is found.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
/// * `config_path` - Optional explicit config file path
///
/// # Errors
///
/// Returns an error if the config file exists but cannot be parsed, or if
/// the workspace is not initialized.
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
        match ConfigLoader::load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_path.display());
                config
            }
            Err(e) => {
                return Err(CliError::Configuration(format!(
                    "Workspace not initialized. Run 'workspace init' first.\nDetails: {e}"
                )));
            }
        }
    } else {
        return Err(CliError::Configuration(
            "Workspace not initialized. Run 'workspace init' first.".to_string(),
        ));
    };

    Ok(config)
}

/// Output check result in JSON format.
fn output_json(output: &Output, branch: &str, exists: bool) -> Result<()> {
    let response = ChangesetCheckResponse {
        exists,
        branch: branch.to_string(),
        message: if exists {
            Some(format!("Changeset exists for branch '{branch}'"))
        } else {
            Some(format!("No changeset found for branch '{branch}'"))
        },
    };

    let json_response = JsonResponse::success(response);

    output
        .json(&json_response)
        .map_err(|e| CliError::Execution(format!("Failed to output JSON: {e}")))?;

    Ok(())
}

/// Output check result in human-readable format.
fn output_human(output: &Output, branch: &str, exists: bool) -> Result<()> {
    if exists {
        output
            .success(&format!("✓ Changeset exists for branch '{branch}'"))
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
    } else {
        output
            .error(&format!("✗ No changeset found for branch '{branch}'"))
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
    }

    Ok(())
}
