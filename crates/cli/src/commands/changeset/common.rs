//! Common utilities for changeset commands.
//!
//! This module provides shared functionality used across multiple changeset commands
//! to eliminate code duplication and ensure consistent behavior.
//!
//! # What
//!
//! Provides shared functions for:
//! - Configuration loading with auto-detection
//! - Git branch detection
//! - Bump type validation and parsing
//! - Environment validation
//! - Changeset file path management
//! - Branch name sanitization
//!
//! # How
//!
//! Each function encapsulates a specific piece of functionality that was previously
//! duplicated across multiple command implementations. These functions follow the
//! same error handling patterns and use the same base crates (standard, git, pkg).
//!
//! # Why
//!
//! Centralizing common functionality:
//! - Eliminates code duplication (DRY principle)
//! - Ensures consistency across commands
//! - Simplifies maintenance and updates
//! - Makes testing easier
//! - Reduces the risk of inconsistent behavior
//!
//! # Examples
//!
//! ```rust,ignore
//! use super::common::{load_config, detect_current_branch};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = load_config(Path::new("."), None).await?;
//! let branch = detect_current_branch(Path::new("."))?;
//! # Ok(())
//! # }
//! ```

use crate::error::{CliError, Result};
use std::path::{Path, PathBuf};
use sublime_git_tools::Repo;
use sublime_pkg_tools::config::PackageToolsConfig;
use sublime_pkg_tools::types::VersionBump;
use tracing::debug;

/// Loads workspace configuration from file or defaults.
///
/// Attempts to load configuration from the provided path or auto-detects it
/// by searching for standard config file names in the workspace root.
///
/// # Arguments
///
/// * `workspace_root` - Root directory of the workspace
/// * `config_path` - Optional explicit config file path
///
/// # Returns
///
/// Returns the loaded `PackageToolsConfig` or defaults if no config file is found.
///
/// # Errors
///
/// Returns an error if:
/// - The explicitly provided config file does not exist
/// - A config file exists but cannot be parsed
/// - File system operations fail
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::load_config;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Auto-detect config file
/// let config = load_config(Path::new("."), None).await?;
///
/// // Use specific config file
/// let config = load_config(Path::new("."), Some(Path::new("custom.config.toml"))).await?;
/// # Ok(())
/// # }
/// ```
pub(crate) async fn load_config(
    workspace_root: &Path,
    config_path: Option<&Path>,
) -> Result<PackageToolsConfig> {
    debug!("Loading workspace configuration from: {}", workspace_root.display());

    // Use the centralized config loading function
    let config = crate::commands::find_and_load_config(workspace_root, config_path).await?;

    match config {
        Some(config) => {
            debug!("Configuration loaded successfully");
            Ok(config)
        }
        None => Err(CliError::configuration(
            "Workspace not initialized. Run 'workspace init' first.".to_string(),
        )),
    }
}

/// Detects the current Git branch.
///
/// Opens the Git repository at the workspace root and retrieves the current branch name.
///
/// # Arguments
///
/// * `workspace_root` - The workspace root directory
///
/// # Returns
///
/// Returns the current branch name as a String.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a Git repository
/// - Cannot determine current branch (e.g., detached HEAD state)
/// - Workspace path contains invalid UTF-8 characters
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::detect_current_branch;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let branch = detect_current_branch(Path::new("."))?;
/// println!("Current branch: {}", branch);
/// # Ok(())
/// # }
/// ```
pub(crate) fn detect_current_branch(workspace_root: &Path) -> Result<String> {
    debug!("Detecting current Git branch at: {}", workspace_root.display());

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
/// Constructs the full path to a changeset file based on the workspace root,
/// configuration, and branch name. The branch name is sanitized for filesystem safety.
///
/// # Arguments
///
/// * `workspace_root` - The workspace root directory
/// * `config` - The package tools configuration
/// * `branch` - The branch name
///
/// # Returns
///
/// Returns the path to the changeset file as a `PathBuf`.
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::get_changeset_file_path;
/// use std::path::Path;
///
/// let path = get_changeset_file_path(
///     Path::new("."),
///     &config,
///     "feature/new-api"
/// );
/// // Returns: ./.changesets/feature-new-api.json
/// ```
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
/// Replaces characters that are invalid or problematic in filenames with safe
/// alternatives (hyphens). This ensures branch names can be safely used as
/// part of file paths on all platforms.
///
/// # Arguments
///
/// * `branch` - The branch name to sanitize
///
/// # Returns
///
/// Returns the sanitized branch name as a String.
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::sanitize_branch_name;
///
/// assert_eq!(sanitize_branch_name("feature/new-api"), "feature-new-api");
/// assert_eq!(sanitize_branch_name("fix:bug"), "fix-bug");
/// assert_eq!(sanitize_branch_name("test<file>"), "test-file-");
/// ```
pub(crate) fn sanitize_branch_name(branch: &str) -> String {
    branch.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Validates a bump type string.
///
/// Ensures the bump type is one of the valid values: patch, minor, major, or none.
/// Case-insensitive comparison is used.
///
/// # Arguments
///
/// * `bump` - The bump type string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the bump type is valid.
///
/// # Errors
///
/// Returns a validation error if the bump type is not recognized.
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::validate_bump_type;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// validate_bump_type("minor")?; // OK
/// validate_bump_type("MAJOR")?; // OK (case-insensitive)
/// validate_bump_type("invalid")?; // Error
/// # Ok(())
/// # }
/// ```
pub(crate) fn validate_bump_type(bump: &str) -> Result<()> {
    match bump.to_lowercase().as_str() {
        "patch" | "minor" | "major" | "none" => Ok(()),
        _ => Err(CliError::validation(format!(
            "Invalid bump type '{bump}'. Must be one of: patch, minor, major, none"
        ))),
    }
}

/// Parses a bump type string into a VersionBump enum.
///
/// Converts the string representation to the appropriate enum variant.
/// Case-insensitive comparison is used.
///
/// # Arguments
///
/// * `bump` - The bump type string to parse
///
/// # Returns
///
/// Returns the corresponding `VersionBump` enum variant.
///
/// # Errors
///
/// Returns a validation error if the bump type is not recognized.
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::parse_bump_type;
/// use sublime_pkg_tools::types::VersionBump;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let bump = parse_bump_type("minor")?;
/// assert_eq!(bump, VersionBump::Minor);
///
/// let bump = parse_bump_type("MAJOR")?; // Case-insensitive
/// assert_eq!(bump, VersionBump::Major);
/// # Ok(())
/// # }
/// ```
pub(crate) fn parse_bump_type(bump: &str) -> Result<VersionBump> {
    match bump.to_lowercase().as_str() {
        "patch" => Ok(VersionBump::Patch),
        "minor" => Ok(VersionBump::Minor),
        "major" => Ok(VersionBump::Major),
        "none" => Ok(VersionBump::None),
        _ => Err(CliError::validation(format!(
            "Invalid bump type '{bump}'. Must be one of: patch, minor, major, none"
        ))),
    }
}

/// Validates that all provided environments are in the available list.
///
/// If the available list is empty, all environments are considered valid
/// (no validation is performed). This allows flexibility in configuration.
///
/// # Arguments
///
/// * `provided` - The list of environment names to validate
/// * `available` - The list of configured available environments
///
/// # Returns
///
/// Returns `Ok(())` if all provided environments are valid.
///
/// # Errors
///
/// Returns a validation error if any provided environment is not in the available list.
///
/// # Examples
///
/// ```rust,ignore
/// use super::common::validate_environments;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let available = vec!["dev".to_string(), "staging".to_string(), "prod".to_string()];
/// let provided = vec!["dev".to_string(), "prod".to_string()];
///
/// validate_environments(&provided, &available)?; // OK
///
/// let invalid = vec!["invalid".to_string()];
/// validate_environments(&invalid, &available)?; // Error
/// # Ok(())
/// # }
/// ```
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
