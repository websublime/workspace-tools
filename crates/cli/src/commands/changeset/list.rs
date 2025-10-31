//! Changeset list command implementation.
//!
//! This module implements the `changeset list` command for displaying all pending changesets.
//!
//! # What
//!
//! Provides the `execute_list` function that:
//! - Lists all pending changesets in the workspace
//! - Supports filtering by package name, bump type, and environment
//! - Supports sorting by date, branch, or bump type
//! - Outputs results in table or JSON format
//! - Handles empty changeset directories gracefully
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Creates ChangesetManager to access changeset storage
//! 3. Retrieves all pending changesets via `list_pending()`
//! 4. Applies filters based on command arguments (package, bump, environment)
//! 5. Sorts changesets by specified field (date, branch, bump)
//! 6. Formats output as either a human-readable table or JSON
//! 7. Displays summary statistics (total count)
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset retrieval
//! - Table rendering for human-readable output
//! - JSON serialization for machine-readable output
//!
//! # Why
//!
//! Listing changesets is essential for:
//! - Understanding pending version bumps in the workspace
//! - Reviewing changesets before executing a release
//! - Filtering and finding specific changesets
//! - CI/CD integration for automated changeset inspection
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_list;
//! use sublime_cli_tools::cli::commands::ChangesetListArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = ChangesetListArgs {
//!     filter_package: Some("my-package".to_string()),
//!     filter_bump: Some("minor".to_string()),
//!     filter_env: None,
//!     sort: "date".to_string(),
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_list(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetListArgs;
use crate::error::{CliError, Result};
use crate::output::styling::{Section, print_item};
use crate::output::table::{ColumnAlignment, TableBuilder, TableTheme};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_pkg_tools::types::{Changeset, VersionBump};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info};

/// Response data for changeset list command (JSON output).
///
/// Contains the list of changesets and summary information.
#[derive(Debug, Serialize)]
struct ChangesetListResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// List of changesets with their details.
    changesets: Vec<ChangesetListItem>,
    /// Total number of changesets returned.
    total: usize,
}

/// Individual changeset information for list output.
#[derive(Debug, Serialize)]
struct ChangesetListItem {
    /// Branch name (also serves as unique identifier).
    branch: String,
    /// Version bump type.
    bump: String,
    /// List of affected packages.
    packages: Vec<String>,
    /// Target environments.
    environments: Vec<String>,
    /// Number of commits in the changeset.
    commit_count: usize,
    /// Creation timestamp.
    created_at: String,
    /// Last update timestamp.
    updated_at: String,
}

impl From<Changeset> for ChangesetListItem {
    fn from(changeset: Changeset) -> Self {
        Self {
            branch: changeset.branch,
            bump: changeset.bump.to_string().to_lowercase(),
            packages: changeset.packages,
            environments: changeset.environments,
            commit_count: changeset.changes.len(),
            created_at: changeset.created_at.to_rfc3339(),
            updated_at: changeset.updated_at.to_rfc3339(),
        }
    }
}

/// Execute the changeset list command.
///
/// Lists all pending changesets in the workspace, with optional filtering and sorting.
/// Outputs results in either human-readable table format or JSON format.
///
/// # Arguments
///
/// * `args` - Command arguments including filters and sort options
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
/// - The changeset directory cannot be read
/// - Invalid filter or sort values are provided
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_list;
/// use sublime_cli_tools::cli::commands::ChangesetListArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ChangesetListArgs {
///     filter_package: None,
///     filter_bump: None,
///     filter_env: None,
///     sort: "date".to_string(),
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_list(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_list(
    args: &ChangesetListArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));
    debug!("Listing changesets in workspace: {}", workspace_root.display());

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;

    info!("Configuration loaded successfully");

    // Create changeset manager
    let manager =
        ChangesetManager::new(workspace_root.to_path_buf(), FileSystemManager::new(), config)
            .await
            .map_err(|e| CliError::Execution(format!("Failed to create changeset manager: {e}")))?;

    // Load all pending changesets
    let mut changesets = manager
        .list_pending()
        .await
        .map_err(|e| CliError::Execution(format!("Failed to load changesets: {e}")))?;

    debug!("Loaded {} changeset(s)", changesets.len());

    // Apply filters
    if let Some(package) = &args.filter_package {
        debug!("Filtering by package: {}", package);
        changesets.retain(|cs| cs.has_package(package));
    }

    if let Some(bump_str) = &args.filter_bump {
        let bump = parse_bump_type(bump_str)?;
        debug!("Filtering by bump type: {:?}", bump);
        changesets.retain(|cs| cs.bump == bump);
    }

    if let Some(env) = &args.filter_env {
        debug!("Filtering by environment: {}", env);
        changesets.retain(|cs| cs.environments.contains(env));
    }

    // Sort changesets
    sort_changesets(&mut changesets, &args.sort)?;

    let total = changesets.len();
    info!("Displaying {} changeset(s) after filtering", total);

    // Output results
    if output.format().is_json() {
        output_json(output, changesets)?;
    } else {
        output_table(output, &changesets)?;
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
/// Returns an error if the config file exists but cannot be parsed.
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

/// Sort changesets by the specified field.
///
/// # Arguments
///
/// * `changesets` - Mutable reference to the list of changesets to sort
/// * `sort_by` - Field to sort by: "date", "branch", or "bump"
///
/// # Errors
///
/// Returns an error if the sort field is invalid.
fn sort_changesets(changesets: &mut [Changeset], sort_by: &str) -> Result<()> {
    match sort_by {
        "date" => {
            changesets.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            debug!("Sorted by date (most recent first)");
        }
        "branch" => {
            changesets.sort_by(|a, b| a.branch.cmp(&b.branch));
            debug!("Sorted by branch (alphabetical)");
        }
        "bump" => {
            changesets.sort_by(|a, b| {
                let a_order = bump_order(a.bump);
                let b_order = bump_order(b.bump);
                b_order.cmp(&a_order)
            });
            debug!("Sorted by bump type (major > minor > patch)");
        }
        _ => {
            return Err(CliError::Validation(format!(
                "Invalid sort field '{sort_by}'. Valid options: date, branch, bump"
            )));
        }
    }

    Ok(())
}

/// Get sort order for bump types.
///
/// Major bumps are highest priority (3), followed by minor (2), then patch (1).
fn bump_order(bump: VersionBump) -> u8 {
    match bump {
        VersionBump::Major => 3,
        VersionBump::Minor => 2,
        VersionBump::Patch => 1,
        VersionBump::None => 0,
    }
}

/// Parse a bump type string into a VersionBump enum.
///
/// # Arguments
///
/// * `bump_str` - String representation of bump type (case-insensitive)
///
/// # Errors
///
/// Returns an error if the bump type is invalid.
pub(crate) fn parse_bump_type(bump_str: &str) -> Result<VersionBump> {
    match bump_str.to_lowercase().as_str() {
        "major" => Ok(VersionBump::Major),
        "minor" => Ok(VersionBump::Minor),
        "patch" => Ok(VersionBump::Patch),
        _ => Err(CliError::Validation(format!(
            "Invalid bump type '{bump_str}'. Valid options: major, minor, patch"
        ))),
    }
}

/// Output changesets in JSON format.
fn output_json(output: &Output, changesets: Vec<Changeset>) -> Result<()> {
    let items: Vec<ChangesetListItem> =
        changesets.into_iter().map(std::convert::Into::into).collect();

    let total = items.len();
    let response =
        JsonResponse::success(ChangesetListResponse { success: true, changesets: items, total });

    output
        .json(&response)
        .map_err(|e| CliError::Execution(format!("Failed to output JSON: {e}")))?;

    Ok(())
}

/// Output changesets in human-readable table format.
fn output_table(output: &Output, changesets: &[Changeset]) -> Result<()> {
    let total = changesets.len();

    if total == 0 {
        output
            .info("No changesets found")
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
        output
            .blank_line()
            .map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;
        return Ok(());
    }

    // Create table
    let mut table = TableBuilder::new()
        .theme(TableTheme::Default)
        .columns(&["Branch", "Bump", "Packages", "Environments", "Commits", "Updated"])
        .alignment(0, ColumnAlignment::Left)
        .alignment(1, ColumnAlignment::Left)
        .alignment(2, ColumnAlignment::Left)
        .alignment(3, ColumnAlignment::Left)
        .alignment(4, ColumnAlignment::Right)
        .alignment(5, ColumnAlignment::Left)
        .max_width(120)
        .build();

    // Add rows
    for changeset in changesets {
        let packages_display = if changeset.packages.is_empty() {
            String::from("-")
        } else if changeset.packages.len() == 1 {
            changeset.packages[0].clone()
        } else {
            format!("{} packages", changeset.packages.len())
        };

        let environments_display = if changeset.environments.is_empty() {
            String::from("-")
        } else {
            changeset.environments.join(", ")
        };

        let bump_display = format_bump_type(changeset.bump);

        let updated_display = format_relative_time(changeset);
        let commit_count = changeset.changes.len().to_string();

        table.add_row(&[
            &changeset.branch,
            &bump_display,
            &packages_display,
            &environments_display,
            &commit_count,
            &updated_display,
        ]);
    }

    // Display table
    output
        .table(&mut table)
        .map_err(|e| CliError::Execution(format!("Failed to render table: {e}")))?;

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Display summary
    let section = Section::new("Summary");
    section.print();
    print_item("Total Changesets", &total.to_string(), true);

    Ok(())
}

/// Format bump type with color styling.
fn format_bump_type(bump: VersionBump) -> String {
    match bump {
        VersionBump::Major => String::from("major"),
        VersionBump::Minor => String::from("minor"),
        VersionBump::Patch => String::from("patch"),
        VersionBump::None => String::from("none"),
    }
}

/// Format a timestamp as a relative time string.
///
/// Shows the time difference from now in a human-readable format.
fn format_relative_time(timestamp: &sublime_pkg_tools::types::Changeset) -> String {
    // For now, just format as a date string
    // In a real implementation, we'd calculate the time difference
    timestamp.updated_at.format("%Y-%m-%d %H:%M").to_string()
}
