//! Changeset update command implementation.
//!
//! This module implements the `changeset update` command for modifying existing changesets.
//!
//! # What
//!
//! Provides the `execute_update` function that:
//! - Updates existing changesets with new packages, commits, bump types, or environments
//! - Loads changeset by branch name or ID (auto-detects current branch if not provided)
//! - Validates updates before applying them
//! - Formats output in human-readable or JSON format
//! - Handles not-found errors gracefully with clear error messages
//! - Tracks modification history via updated_at timestamp
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Determines which changeset to update (from ID parameter or current git branch)
//! 3. Loads the existing changeset using ChangesetManager
//! 4. Applies requested updates (packages, commits, bump type, environments)
//! 5. Validates the updated changeset
//! 6. Saves changes back to storage
//! 7. Outputs success message with update summary
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset operations
//! - `Repo` from git tools for branch detection
//! - Styled output sections for human-readable display
//! - JSON serialization for machine-readable output
//!
//! # Why
//!
//! Updating existing changesets is essential for:
//! - Adding new packages to an existing changeset as development progresses
//! - Tracking additional commits associated with the changeset
//! - Adjusting version bump types when scope changes
//! - Modifying target environments before release
//! - Maintaining accurate changeset information throughout the development cycle
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::changeset::execute_update;
//! use sublime_cli_tools::cli::commands::ChangesetUpdateArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Update changeset for current branch
//! let args = ChangesetUpdateArgs {
//!     id: None, // Will auto-detect current branch
//!     commit: Some("abc123".to_string()),
//!     packages: Some(vec!["new-package".to_string()]),
//!     bump: Some("major".to_string()),
//!     env: Some(vec!["staging".to_string()]),
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! execute_update(&args, &output, None, None).await?;
//! # Ok(())
//! # }
//! ```

use crate::cli::commands::ChangesetUpdateArgs;
use crate::error::{CliError, Result};
use crate::output::styling::{Section, StatusSymbol, TextStyle, print_item};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::Path;
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_pkg_tools::types::{Changeset, VersionBump};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info};

/// Response data for changeset update command (JSON output).
///
/// Contains information about the update operation.
#[derive(Debug, Serialize)]
struct ChangesetUpdateResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// Summary of what was updated.
    updated: UpdateSummary,
    /// The updated changeset details.
    changeset: ChangesetInfo,
}

/// Summary of updates applied to a changeset.
#[derive(Debug, Serialize)]
struct UpdateSummary {
    /// Number of packages added.
    packages_added: usize,
    /// Number of commits added.
    commits_added: usize,
    /// Whether bump type was changed.
    bump_updated: bool,
    /// Number of environments added.
    environments_added: usize,
}

/// Changeset information for JSON output.
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

/// Execute the changeset update command.
///
/// Updates an existing changeset with new packages, commits, bump type, or environments.
/// If no ID is provided, automatically detects the current git branch and uses it to
/// locate the changeset. Provides clear error messages when the changeset is not found.
///
/// # Arguments
///
/// * `args` - Command arguments including optional ID and update fields
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
/// - No ID provided and not in a git repository
/// - No ID provided and in detached HEAD state
/// - The specified changeset does not exist for the branch or ID
/// - Invalid bump type provided
/// - Invalid environment names provided
/// - The changeset file cannot be read, parsed, or saved
/// - File system operations fail
/// - Validation fails after updates
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::changeset::execute_update;
/// use sublime_cli_tools::cli::commands::ChangesetUpdateArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Update with explicit branch ID
/// let args = ChangesetUpdateArgs {
///     id: Some("feature/my-feature".to_string()),
///     commit: Some("abc123def456".to_string()),
///     packages: Some(vec!["my-package".to_string()]),
///     bump: Some("minor".to_string()),
///     env: Some(vec!["production".to_string()]),
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_update(&args, &output, None, None).await?;
///
/// // Update using current branch (auto-detect)
/// let args = ChangesetUpdateArgs {
///     id: None, // Will detect current branch
///     commit: None,
///     packages: Some(vec!["another-package".to_string()]),
///     bump: None,
///     env: None,
/// };
/// execute_update(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_lines)]
pub async fn execute_update(
    args: &ChangesetUpdateArgs,
    output: &Output,
    root: Option<&Path>,
    config_path: Option<&Path>,
) -> Result<()> {
    let workspace_root = root.unwrap_or_else(|| Path::new("."));

    info!("Starting changeset update command");
    debug!("Workspace root: {}", workspace_root.display());

    // Load configuration
    let config = load_config(workspace_root, config_path).await?;
    info!("Configuration loaded successfully");

    // Determine which changeset to update
    let branch = if let Some(id) = &args.id {
        debug!("Using provided changeset ID: {}", id);
        id.clone()
    } else {
        debug!("No ID provided, detecting current branch");
        detect_current_branch(workspace_root)?
    };

    debug!("Target changeset branch: {}", branch);

    // Create changeset manager
    let manager = ChangesetManager::new(
        workspace_root.to_path_buf(),
        FileSystemManager::new(),
        config.clone(),
    )
    .await
    .map_err(|e| CliError::Execution(format!("Failed to create changeset manager: {e}")))?;

    // Load the existing changeset
    let mut changeset = manager.load(&branch).await.map_err(|e| {
        CliError::Execution(format!(
            "No changeset found for branch '{branch}'. Create one with 'wnt changeset create' first.\nDetails: {e}"
        ))
    })?;

    debug!("Loaded changeset for branch: {}", changeset.branch);

    // Track what was updated
    let mut packages_added = 0;
    let mut commits_added = 0;
    let mut bump_updated = false;
    let mut environments_added = 0;

    // Update packages
    if let Some(packages) = &args.packages {
        debug!("Adding packages: {:?}", packages);
        for package in packages {
            if changeset.has_package(package) {
                debug!("Package '{}' already in changeset, skipping", package);
            } else {
                changeset.add_package(package);
                packages_added += 1;
            }
        }
    }

    // Update commits
    if let Some(commit) = &args.commit {
        debug!("Adding commit: {}", commit);
        if changeset.changes.contains(commit) {
            debug!("Commit '{}' already in changeset, skipping", commit);
        } else {
            changeset.add_commit(commit);
            commits_added += 1;
        }
    }

    // Update bump type
    if let Some(bump_str) = &args.bump {
        debug!("Updating bump type to: {}", bump_str);
        let new_bump = parse_bump_type(bump_str)?;
        if changeset.bump == new_bump {
            debug!("Bump type unchanged, skipping");
        } else {
            changeset.set_bump(new_bump);
            bump_updated = true;
        }
    }

    // Update environments
    if let Some(environments) = &args.env {
        debug!("Adding environments: {:?}", environments);

        // Validate environments against configured ones
        if !config.changeset.available_environments.is_empty() {
            validate_environments(environments, &config.changeset.available_environments)?;
        }

        // Collect new environments to add
        let mut new_envs = changeset.environments.clone();
        for env in environments {
            if new_envs.contains(env) {
                debug!("Environment '{}' already in changeset, skipping", env);
            } else {
                new_envs.push(env.clone());
                environments_added += 1;
            }
        }
        if environments_added > 0 {
            changeset.set_environments(new_envs);
        }
    }

    // Check if anything was updated
    if packages_added == 0 && commits_added == 0 && !bump_updated && environments_added == 0 {
        return Err(CliError::Validation(
            "No updates were applied. The changeset already contains the specified values."
                .to_string(),
        ));
    }

    // Validate the updated changeset
    let available_envs: Vec<&str> =
        config.changeset.available_environments.iter().map(std::string::String::as_str).collect();
    changeset
        .validate(&available_envs)
        .map_err(|e| CliError::Validation(format!("Updated changeset validation failed: {e}")))?;

    debug!("Changeset validated successfully");

    // Save the updated changeset
    manager
        .update(&changeset)
        .await
        .map_err(|e| CliError::Execution(format!("Failed to save updated changeset: {e}")))?;

    info!("Changeset updated successfully for branch: {}", changeset.branch);

    // Output results
    let summary = UpdateSummary { packages_added, commits_added, bump_updated, environments_added };

    output_results(output, &changeset, &summary)?;

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

/// Detect the current git branch.
///
/// Opens the git repository and retrieves the current branch name.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - In detached HEAD state
/// - Unable to determine current branch
fn detect_current_branch(workspace_root: &Path) -> Result<String> {
    debug!("Detecting current git branch");

    let repo =
        Repo::open(workspace_root.to_str().ok_or_else(|| {
            CliError::execution("Failed to convert workspace root path to string")
        })?)
        .map_err(|e| {
            CliError::git(format!(
                "Failed to open git repository. Are you in a git repository?\nDetails: {e}"
            ))
        })?;

    let branch = repo.get_current_branch().map_err(|e| {
        CliError::git(format!(
            "Failed to get current branch. Are you in detached HEAD state?\nDetails: {e}"
        ))
    })?;

    debug!("Detected current branch: {}", branch);
    Ok(branch)
}

/// Parse bump type string into VersionBump enum.
///
/// # Arguments
///
/// * `bump_str` - String representation of bump type (case-insensitive)
///
/// # Errors
///
/// Returns validation error if bump type is invalid.
pub(crate) fn parse_bump_type(bump_str: &str) -> Result<VersionBump> {
    match bump_str.to_lowercase().as_str() {
        "major" => Ok(VersionBump::Major),
        "minor" => Ok(VersionBump::Minor),
        "patch" => Ok(VersionBump::Patch),
        _ => Err(CliError::Validation(format!(
            "Invalid bump type '{bump_str}'. Must be one of: patch, minor, major"
        ))),
    }
}

/// Validate environment names against configured environments.
///
/// # Arguments
///
/// * `provided` - List of environment names to validate
/// * `available` - List of valid environment names from configuration
///
/// # Errors
///
/// Returns validation error if any provided environment is not in the available list.
pub(crate) fn validate_environments(provided: &[String], available: &[String]) -> Result<()> {
    if available.is_empty() {
        // No restrictions - all environments are valid
        return Ok(());
    }

    for env in provided {
        if !available.contains(env) {
            return Err(CliError::Validation(format!(
                "Environment '{env}' is not configured. Available environments: {}",
                available.join(", ")
            )));
        }
    }

    Ok(())
}

/// Output update results in JSON format.
fn output_json(output: &Output, changeset: &Changeset, summary: &UpdateSummary) -> Result<()> {
    let info: ChangesetInfo = changeset.clone().into();
    let response = JsonResponse::success(ChangesetUpdateResponse {
        success: true,
        updated: UpdateSummary {
            packages_added: summary.packages_added,
            commits_added: summary.commits_added,
            bump_updated: summary.bump_updated,
            environments_added: summary.environments_added,
        },
        changeset: info,
    });

    output
        .json(&response)
        .map_err(|e| CliError::Execution(format!("Failed to output JSON: {e}")))?;

    Ok(())
}

/// Output update results in human-readable formatted view.
fn output_formatted(output: &Output, changeset: &Changeset, summary: &UpdateSummary) -> Result<()> {
    // Success header
    StatusSymbol::Success
        .print_line(&TextStyle::success(&format!("Changeset updated: {}", changeset.branch)));

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Update summary section
    let section = Section::new("Updates Applied");
    section.print();

    if summary.packages_added > 0 {
        print_item("Packages Added", &summary.packages_added.to_string(), false);
    }
    if summary.commits_added > 0 {
        print_item("Commits Added", &summary.commits_added.to_string(), false);
    }
    if summary.bump_updated {
        print_item("Bump Type Updated", "Yes", false);
    }
    if summary.environments_added > 0 {
        print_item("Environments Added", &summary.environments_added.to_string(), false);
    }

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    // Current changeset state
    let section = Section::new("Current Changeset State");
    section.print();

    print_item("Branch", &changeset.branch, false);
    print_item("Bump Type", &format_bump_type(changeset.bump), false);
    print_item("Packages", &format!("{} package(s)", changeset.packages.len()), false);
    print_item("Environments", &format!("{} environment(s)", changeset.environments.len()), false);
    print_item("Commits", &format!("{} commit(s)", changeset.changes.len()), false);
    print_item("Updated", &changeset.updated_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(), true);

    output.blank_line().map_err(|e| CliError::Execution(format!("Failed to write output: {e}")))?;

    Ok(())
}

/// Output results based on output format.
fn output_results(output: &Output, changeset: &Changeset, summary: &UpdateSummary) -> Result<()> {
    if output.format().is_json() {
        output_json(output, changeset, summary)
    } else {
        output_formatted(output, changeset, summary)
    }
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
