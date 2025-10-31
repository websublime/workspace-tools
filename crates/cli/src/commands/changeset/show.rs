//! Changeset show command implementation.
//!
//! This module implements the `changeset show` command for displaying detailed information
//! about a specific changeset.
//!
//! # What
//!
//! Provides the `execute_show` function that:
//! - Displays complete details of a specific changeset
//! - Loads changeset by branch name or ID
//! - Shows all metadata including packages, environments, commits, and timestamps
//! - Formats output in human-readable or JSON format
//! - Handles not-found errors gracefully
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Creates ChangesetManager to access changeset storage
//! 3. Loads the specific changeset using the branch name
//! 4. Formats and displays all changeset details with sections and styling
//! 5. Outputs in table/formatted view or JSON based on user preference
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset retrieval
//! - Styled output sections for human-readable display
//! - JSON serialization for machine-readable output
//!
//! # Why
//!
//! Viewing detailed changeset information is essential for:
//! - Understanding what changes are included in a release
//! - Reviewing affected packages and target environments
//! - Verifying commit associations before version bumps
//! - Auditing changeset metadata and timestamps
//! - CI/CD integration for automated changeset inspection
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_show;
//! use sublime_cli_tools::cli::commands::ChangesetShowArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetShowArgs {
//!     branch: "feature/new-api".to_string(),
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_show(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetShowArgs;
use crate::error::{CliError, Result};
use crate::output::styling::{Section, print_item};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_pkg_tools::types::{Changeset, VersionBump};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info};

/// Response data for changeset show command (JSON output).
///
/// Contains the complete changeset information.
#[derive(Debug, Serialize)]
struct ChangesetShowResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// The changeset details.
    changeset: ChangesetShowItem,
}

/// Detailed changeset information for show output.
#[derive(Debug, Serialize)]
struct ChangesetShowItem {
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

impl From<Changeset> for ChangesetShowItem {
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

/// Execute the changeset show command.
///
/// Displays detailed information for a specific changeset identified by branch name.
/// Outputs results in either human-readable formatted view or JSON format.
///
/// # Arguments
///
/// * `args` - Command arguments including the branch name to display
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
/// - The changeset file cannot be read or parsed
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_show;
/// use sublime_cli_tools::cli::commands::ChangesetShowArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ChangesetShowArgs {
///     branch: "feature/my-feature".to_string(),
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_show(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_show(
    args: &ChangesetShowArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));
    debug!("Showing changeset '{}' in workspace: {}", args.branch, workspace_root.display());

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;

    info!("Configuration loaded successfully");

    // Create changeset manager
    let manager =
        ChangesetManager::new(workspace_root.to_path_buf(), FileSystemManager::new(), config)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to create changeset manager: {e}")))?;

    // Load the specific changeset
    let changeset = manager
        .load(&args.branch)
        .await
        .map_err(|e| CliError::Execution(format!("Changeset '{}' not found: {e}", args.branch)))?;

    debug!("Loaded changeset: {}", changeset.branch);
    info!("Displaying changeset details");

    // Output results
    if output.format().is_json() {
        output_json(output, changeset)?;
    } else {
        output_formatted(output, &changeset)?;
    }

    Ok(())
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

/// Output changeset in JSON format.
fn output_json(output: &Output, changeset: Changeset) -> Result<()> {
    let item: ChangesetShowItem = changeset.into();
    let response = JsonResponse::success(ChangesetShowResponse { success: true, changeset: item });

    output
        .json(&response)
        .map_err(|e| CliError::Execution(format!("Failed to output JSON: {e}")))?;

    Ok(())
}

/// Output changeset in human-readable formatted view.
fn output_formatted(output: &Output, changeset: &Changeset) -> Result<()> {
    // Header section
    let section = Section::new(format!("Changeset: {}", changeset.branch));
    section.print();

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Basic information
    let section = Section::new("Basic Information");
    section.print();

    print_item("Branch", &changeset.branch, true);
    print_item("Bump Type", &format_bump_type(changeset.bump), true);
    print_item("Created", &changeset.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(), true);
    print_item("Updated", &changeset.updated_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(), true);

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Affected packages
    let section = Section::new("Affected Packages");
    section.print();

    if changeset.packages.is_empty() {
        output
            .info("  No packages specified")
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
    } else {
        for (idx, package) in changeset.packages.iter().enumerate() {
            let is_last = idx == changeset.packages.len() - 1;
            print_item("", package, is_last);
        }
    }

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Target environments
    let section = Section::new("Target Environments");
    section.print();

    if changeset.environments.is_empty() {
        output
            .info("  No environments specified")
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
    } else {
        for (idx, env) in changeset.environments.iter().enumerate() {
            let is_last = idx == changeset.environments.len() - 1;
            print_item("", env, is_last);
        }
    }

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Commits
    let section = Section::new("Commits");
    section.print();

    if changeset.changes.is_empty() {
        output
            .info("  No commits recorded")
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
    } else {
        print_item("Total Commits", &changeset.changes.len().to_string(), false);
        output
            .blank_line()
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
        for (idx, commit) in changeset.changes.iter().enumerate() {
            let is_last = idx == changeset.changes.len() - 1;
            print_item("", commit, is_last);
        }
    }

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    Ok(())
}

/// Format bump type as a lowercase string.
fn format_bump_type(bump: VersionBump) -> String {
    match bump {
        VersionBump::Major => String::from("major"),
        VersionBump::Minor => String::from("minor"),
        VersionBump::Patch => String::from("patch"),
        VersionBump::None => String::from("none"),
    }
}
