//! Changeset add command implementation.
//!
//! This module implements the `changeset add` command for creating new changesets.
//!
//! # What
//!
//! Provides the `execute_add` function that:
//! - Creates changesets in both interactive and non-interactive modes
//! - Detects affected packages from git changes
//! - Validates workspace configuration
//! - Prompts for user input in interactive mode
//! - Generates and saves changeset files
//!
//! # How
//!
//! The command flow:
//! 1. Loads workspace configuration and validates initialization
//! 2. Opens git repository and detects current branch
//! 3. Detects affected packages from git changes (if not specified)
//! 4. In interactive mode: prompts for packages, bump type, environments, and summary
//! 5. In non-interactive mode: uses provided flags or defaults
//! 6. Creates changeset using ChangesetManager
//! 7. Outputs success message with changeset details
//!
//! Uses:
//! - `ChangesetManager` from pkg tools for changeset operations
//! - `PackageDetector` from pkg tools for git-based package detection
//! - `Repo` from git tools for git operations
//! - Interactive prompts from the prompts module
//!
//! # Why
//!
//! Centralizing changeset creation logic provides:
//! - Consistent changeset workflow
//! - Support for both interactive and automated usage
//! - Proper validation and error handling
//! - User-friendly prompts and feedback
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

use crate::cli::commands::ChangesetCreateArgs;
use crate::error::{CliError, Result};
use crate::interactive::prompts::{
    prompt_bump_type, prompt_environments, prompt_packages, prompt_summary,
};
use crate::output::styling::{Section, StatusSymbol, TextStyle, print_item};
use crate::output::{JsonResponse, Output};
use serde::Serialize;
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_pkg_tools::changeset::ChangesetManager;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_pkg_tools::types::{Changeset, VersionBump};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Response data for changeset add command (JSON output).
///
/// Contains all information about the created changeset.
#[derive(Debug, Serialize)]
struct ChangesetAddResponse {
    /// Whether the operation succeeded.
    success: bool,
    /// The created changeset details.
    changeset: ChangesetInfo,
}

/// Changeset information for JSON output.
#[derive(Debug, Serialize)]
struct ChangesetInfo {
    /// Changeset unique identifier.
    id: String,
    /// Branch name.
    branch: String,
    /// Version bump type.
    bump: String,
    /// List of affected packages.
    packages: Vec<String>,
    /// Target environments.
    environments: Vec<String>,
    /// Optional summary message.
    message: Option<String>,
}

/// Executes the changeset add command.
///
/// Creates a new changeset for the current branch with the specified bump type
/// and environments. Supports both interactive and non-interactive modes.
///
/// # Arguments
///
/// * `args` - Command arguments from CLI
/// * `output` - Output handler for formatting results
/// * `root` - Optional workspace root path (defaults to current directory)
/// * `config_path` - Optional config file path (auto-detected if None)
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Errors
///
/// Returns errors for:
/// - Missing or invalid workspace configuration
/// - Git repository errors (not a git repo, detached HEAD, etc.)
/// - Changeset already exists for the branch
/// - Invalid input (empty packages, invalid bump type, etc.)
/// - File system errors when saving changeset
/// - User cancellation in interactive mode
///
/// # Examples
///
/// ```rust,no_run
/// use sublime_cli_tools::commands::changeset::execute_add;
/// use sublime_cli_tools::cli::commands::ChangesetCreateArgs;
/// use sublime_cli_tools::output::{Output, OutputFormat};
/// use std::io;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Non-interactive mode
/// let args = ChangesetCreateArgs {
///     bump: Some("minor".to_string()),
///     env: Some(vec!["production".to_string()]),
///     branch: None,
///     message: Some("Add new feature".to_string()),
///     packages: Some(vec!["my-package".to_string()]),
///     non_interactive: true,
/// };
///
/// let output = Output::new(OutputFormat::Human, io::stdout(), false);
/// execute_add(&args, &output, None, None).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_lines)]
pub async fn execute_add(
    args: &ChangesetCreateArgs,
    output: &Output,
    root: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    info!("Starting changeset add command");

    // Determine workspace root
    let workspace_root = root.unwrap_or_else(|| PathBuf::from("."));
    debug!("Using workspace root: {}", workspace_root.display());

    // Load configuration
    debug!("Loading workspace configuration");
    let config = load_config(&workspace_root, config_path).await?;

    // Open git repository
    debug!("Opening git repository");
    let repo =
        Repo::open(workspace_root.to_str().ok_or_else(|| {
            CliError::execution("Failed to convert workspace root path to string")
        })?)
        .map_err(|e| CliError::git(format!("Failed to open git repository: {e}")))?;

    // Get current branch or use provided branch
    let branch = if let Some(branch_name) = &args.branch {
        debug!("Using provided branch: {}", branch_name);
        branch_name.clone()
    } else {
        let current_branch = repo.get_current_branch().map_err(|e| {
            CliError::git(format!(
                "Failed to get current branch: {e}. Are you in detached HEAD state?"
            ))
        })?;
        debug!("Using current git branch: {}", current_branch);
        current_branch
    };

    // Create filesystem manager
    let fs = FileSystemManager::new();

    // Create changeset manager
    debug!("Creating changeset manager");
    let changeset_manager =
        ChangesetManager::new(workspace_root.clone(), fs.clone(), config.clone())
            .await
            .map_err(|e| CliError::execution(format!("Failed to create changeset manager: {e}")))?;

    // Check if changeset already exists for this branch
    debug!("Checking if changeset exists for branch: {}", branch);
    // Try to load the changeset - if it exists, this will succeed
    let exists = changeset_manager.load(&branch).await.is_ok();

    if exists {
        return Err(CliError::validation(format!(
            "Changeset already exists for branch '{branch}'. Use 'wnt changeset update' to modify it."
        )));
    }

    // Load packages from workspace
    // TODO: will be implemented on story 5.x - workspace package loading
    // For now, we'll use a placeholder that returns empty list
    let all_packages = load_workspace_packages(&workspace_root, &config);

    // Detect affected packages from git if not provided
    let detected_packages = if args.packages.is_none() && !all_packages.is_empty() {
        debug!("Detecting affected packages from git changes");
        detect_affected_packages(&workspace_root, &repo, &fs, &all_packages)
    } else {
        vec![]
    };

    // Determine packages
    let packages = if let Some(pkg_list) = &args.packages {
        debug!("Using provided packages: {:?}", pkg_list);
        pkg_list.clone()
    } else if args.non_interactive {
        if detected_packages.is_empty() {
            warn!("No packages detected from git changes in non-interactive mode");
            return Err(CliError::validation(
                "No packages specified and none detected from git changes. \
                 Please specify packages with --packages flag in non-interactive mode.",
            ));
        }
        debug!("Using detected packages in non-interactive mode: {:?}", detected_packages);
        detected_packages.clone()
    } else {
        // Interactive mode
        if all_packages.is_empty() {
            return Err(CliError::validation(
                "No packages found in workspace. Cannot create changeset.",
            ));
        }
        debug!("Prompting for package selection (detected: {:?})", detected_packages);
        prompt_packages(&all_packages, &detected_packages, output.no_color())?
    };

    if packages.is_empty() {
        return Err(CliError::validation("At least one package must be specified"));
    }

    // Determine bump type
    let bump_str = if let Some(bump) = &args.bump {
        debug!("Using provided bump type: {}", bump);
        validate_bump_type(bump)?;
        bump.clone()
    } else if args.non_interactive {
        return Err(CliError::validation(
            "Bump type must be specified with --bump flag in non-interactive mode",
        ));
    } else {
        // Interactive mode
        debug!("Prompting for bump type selection");
        prompt_bump_type(output.no_color())?
    };

    let bump = parse_bump_type(&bump_str)?;

    // Determine environments
    let available_envs = config.changeset.available_environments.clone();
    let default_envs = config.changeset.default_environments.clone();

    let environments = if let Some(env_list) = &args.env {
        debug!("Using provided environments: {:?}", env_list);
        validate_environments(env_list, &available_envs)?;
        env_list.clone()
    } else if args.non_interactive {
        if default_envs.is_empty() {
            warn!("No default environments configured");
            vec![]
        } else {
            debug!("Using default environments in non-interactive mode: {:?}", default_envs);
            default_envs.clone()
        }
    } else {
        // Interactive mode
        if available_envs.is_empty() {
            debug!("No environments configured, using empty list");
            vec![]
        } else {
            debug!("Prompting for environment selection");
            prompt_environments(&available_envs, &default_envs, output.no_color())?
        }
    };

    // Get summary message
    let message = if let Some(msg) = &args.message {
        debug!("Using provided message");
        Some(msg.clone())
    } else if args.non_interactive {
        None
    } else {
        // Interactive mode
        debug!("Prompting for summary message");
        match prompt_summary(None, output.no_color()) {
            Ok(summary) => Some(summary),
            Err(_) => {
                debug!("User skipped summary prompt");
                None
            }
        }
    };

    // Create the changeset
    info!("Creating changeset for branch: {}", branch);
    let mut changeset = changeset_manager
        .create(&branch, bump, environments.clone())
        .await
        .map_err(|e| CliError::execution(format!("Failed to create changeset: {e}")))?;

    // Add packages to changeset
    for package in &packages {
        changeset.add_package(package);
    }

    // Note: Message/summary is not part of the Changeset struct in the current SPEC
    // It may be added in a future story or stored separately
    // For now, we skip setting the message on the changeset itself

    // Save the changeset
    debug!("Saving changeset");
    changeset_manager
        .update(&changeset)
        .await
        .map_err(|e| CliError::execution(format!("Failed to save changeset: {e}")))?;

    info!("Changeset created successfully for branch: {}", changeset.branch);

    // Output results
    output_results(output, &changeset, message.as_ref())?;

    Ok(())
}

/// Loads the workspace configuration.
///
/// Attempts to load configuration from the provided path or auto-detect it.
/// Uses ConfigLoader to search for config files in standard locations.
async fn load_config(
    workspace_root: &Path,
    config_path: Option<PathBuf>,
) -> Result<PackageToolsConfig> {
    debug!("Loading workspace configuration");

    let fs = FileSystemManager::new();

    // Try to find and load config file
    let mut found_config = None;
    if let Some(config) = config_path {
        // Use the explicitly provided config file
        let config_file = if config.is_absolute() { config } else { workspace_root.join(config) };

        if fs.exists(&config_file).await {
            found_config = Some(config_file);
        } else {
            return Err(CliError::configuration(format!(
                "Config file not found: {}",
                config_file.display()
            )));
        }
    } else {
        // Search for default config files
        let config_files = vec![
            workspace_root.join("repo.config.toml"),
            workspace_root.join("repo.config.json"),
            workspace_root.join("repo.config.yaml"),
            workspace_root.join("repo.config.yml"),
        ];

        for config_file in &config_files {
            if fs.exists(config_file).await {
                found_config = Some(config_file.clone());
                break;
            }
        }
    }

    // Load configuration
    let config = if let Some(config_path) = found_config {
        match ConfigLoader::load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_path.display());
                config
            }
            Err(e) => {
                warn!("Failed to load config file, using defaults: {}", e);
                PackageToolsConfig::default()
            }
        }
    } else {
        warn!("No configuration file found, using defaults");
        PackageToolsConfig::default()
    };

    Ok(config)
}

/// Loads all packages from the workspace.
///
/// Returns a list of all available package names in the workspace.
fn load_workspace_packages(_workspace_root: &PathBuf, _config: &PackageToolsConfig) -> Vec<String> {
    // TODO: will be implemented on story 5.x - workspace package detection
    // For now, return an empty list as a placeholder
    debug!("Loading workspace packages (placeholder - will be implemented in story 5.x)");

    // In a real implementation, this would:
    // 1. Detect if it's a monorepo or single package
    // 2. Parse package.json files
    // 3. Return list of package names

    vec![]
}

/// Detects affected packages from git changes.
///
/// Uses the PackageDetector to analyze git changes and determine which
/// packages are affected.
fn detect_affected_packages(
    _workspace_root: &PathBuf,
    _repo: &Repo,
    _fs: &FileSystemManager,
    _all_packages: &[String],
) -> Vec<String> {
    // TODO: will be implemented with proper PackageDetector integration
    // For now, return empty list as we don't have package detection yet
    debug!("Detecting affected packages from git (placeholder - will be implemented in story 5.x)");

    // In a real implementation, this would:
    // 1. Get changed files from git
    // 2. Use PackageDetector to map files to packages
    // 3. Return list of affected package names

    vec![]
}

/// Validates a bump type string.
///
/// Ensures the bump type is one of: patch, minor, major.
pub(crate) fn validate_bump_type(bump: &str) -> Result<()> {
    match bump.to_lowercase().as_str() {
        "patch" | "minor" | "major" => Ok(()),
        _ => Err(CliError::validation(format!(
            "Invalid bump type '{bump}'. Must be one of: patch, minor, major"
        ))),
    }
}

/// Parses a bump type string into a VersionBump enum.
///
/// Converts the string representation to the appropriate enum variant.
pub(crate) fn parse_bump_type(bump: &str) -> Result<VersionBump> {
    match bump.to_lowercase().as_str() {
        "patch" => Ok(VersionBump::Patch),
        "minor" => Ok(VersionBump::Minor),
        "major" => Ok(VersionBump::Major),
        _ => Err(CliError::validation(format!(
            "Invalid bump type '{bump}'. Must be one of: patch, minor, major"
        ))),
    }
}

/// Validates that all provided environments are in the available list.
///
/// If available list is empty, all environments are considered valid.
pub(crate) fn validate_environments(provided: &[String], available: &[String]) -> Result<()> {
    if available.is_empty() {
        // No validation needed if no environments configured
        return Ok(());
    }

    for env in provided {
        if !available.contains(env) {
            return Err(CliError::validation(format!(
                "Environment '{}' is not configured. Available: {}",
                env,
                available.join(", ")
            )));
        }
    }

    Ok(())
}

/// Outputs the command results based on the output format.
///
/// Formats and displays the created changeset information using modern styling.
fn output_results(output: &Output, changeset: &Changeset, message: Option<&String>) -> Result<()> {
    use crate::output::styling::print_bullet;
    use console::Color;

    if output.format().is_json() {
        // JSON output
        let response = ChangesetAddResponse {
            success: true,
            changeset: ChangesetInfo {
                id: changeset.branch.clone(), // Use branch as ID since there's no separate ID field
                branch: changeset.branch.clone(),
                bump: format!("{:?}", changeset.bump).to_lowercase(),
                packages: changeset.packages.clone(),
                environments: changeset.environments.clone(),
                message: message.cloned(),
            },
        };

        let json_response = JsonResponse::success(response);
        output.json(&json_response)?;
    } else if !output.format().is_quiet() {
        // Human-readable output with modern styling
        if output.no_color() {
            // Simple output without styling
            output.success("Changeset created successfully")?;
            output.blank_line()?;
            output.info(&format!("Branch:       {}", changeset.branch))?;
            let bump_str = format!("{:?}", changeset.bump).to_lowercase();
            output.info(&format!("Bump:         {bump_str}"))?;
            output.info(&format!("Packages:     {}", changeset.packages.join(", ")))?;

            if !changeset.environments.is_empty() {
                output.info(&format!("Environments: {}", changeset.environments.join(", ")))?;
            }

            if let Some(msg) = message {
                output.info(&format!("Message:      {msg}"))?;
            }

            output.blank_line()?;
            output.info("Next steps:")?;
            output.info("  • Make your changes and commit them")?;
            output.info("  • The changeset will be included in the next version bump")?;
            output.info("  • Use 'wnt changeset update' to modify the changeset")?;
            output.info("  • Use 'wnt bump --dry-run' to preview version changes")?;
        } else {
            // Modern styled output
            StatusSymbol::Success.print_line(&TextStyle::success("Changeset created successfully"));

            // Changeset details section
            let section = Section::new("Changeset Details");
            section.print();

            print_item("Branch", &changeset.branch, false);
            print_item("Bump Type", &format!("{:?}", changeset.bump).to_lowercase(), false);
            let is_last_before_env = !changeset.environments.is_empty() || message.is_some();
            print_item(
                "Packages",
                &format!("{} ({})", changeset.packages.join(", "), changeset.packages.len()),
                is_last_before_env,
            );

            if !changeset.environments.is_empty() {
                print_item("Environments", &changeset.environments.join(", "), message.is_none());
            }

            if let Some(msg) = message {
                print_item("Message", msg, true);
            }

            // Next steps section
            let next_steps = Section::new("Next Steps");
            next_steps.print();

            print_bullet("Make your changes and commit them", Color::Cyan);
            print_bullet("The changeset will be included in the next version bump", Color::Cyan);
            print_bullet(
                &format!("Use {} to modify the changeset", TextStyle::dim("wnt changeset update")),
                Color::Cyan,
            );
            print_bullet(
                &format!("Use {} to preview version changes", TextStyle::dim("wnt bump --dry-run")),
                Color::Cyan,
            );

            output.blank_line()?;
        }
    }

    Ok(())
}
