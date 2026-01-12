//! Command implementations module.
//!
//! This module contains all command execution logic for the CLI.
//!
//! # What
//!
//! Provides implementations for all CLI commands:
//! - Configuration commands (`init`, `config`)
//! - Changeset commands (`add`, `list`, `show`, `update`, `edit`, `remove`, `history`)
//! - Version management commands (`bump`, `changes`)
//! - Upgrade commands (`check`, `apply`, `rollback`)
//! - Audit commands (`audit` with various modes)
//!
//! # How
//!
//! Each command is implemented as an async function that:
//! 1. Validates arguments
//! 2. Creates necessary managers/services from internal crates
//! 3. Executes the operation
//! 4. Formats and outputs results
//! 5. Returns appropriate exit codes on errors
//!
//! Commands use the `Output` context for consistent formatting across
//! different output modes (human, JSON, compact JSON).
//!
//! # Why
//!
//! Separating command logic from CLI definition improves testability,
//! maintainability, and allows reuse of command logic in other contexts.
//!
//! ## Module Organization
//!
//! Commands will be organized by epic/feature area:
//! - `config.rs` - Configuration management commands
//! - `changeset.rs` - Changeset workflow commands
//! - `version.rs` - Version management commands
//! - `upgrade.rs` - Dependency upgrade commands
//! - `audit.rs` - Audit and health check commands
//! - `changes.rs` - Change analysis commands

// Module exports
pub mod audit;
pub mod bump;
pub mod changes;
pub mod changeset;
pub mod clone;
pub mod config;
pub mod execute;
pub mod init;
pub mod status;
pub mod upgrade;
pub mod version;

// Common utilities
use crate::error::{CliError, Result};
use std::fmt::Write;
use std::path::Path;
use sublime_pkg_tools::config::{ConfigFormat, PackageToolsConfig};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info};

/// Helper to find and load workspace configuration.
///
/// This function implements the standard config file resolution logic used across all commands:
/// 1. If `config_path` is provided (via `--config`), use that file exclusively
/// 2. Otherwise, search for default config files: `repo.config.{toml,json,yaml,yml}`
///
/// # Arguments
///
/// * `root` - Workspace root directory
/// * `config_path` - Optional path to config file (from global `--config` option)
///
/// # Returns
///
/// Returns `Ok(Some(config))` if a config file was found and loaded successfully,
/// `Ok(None)` if no config file exists (only when `config_path` is None),
/// or an error if a specified config file is missing or invalid.
///
/// # Errors
///
/// Returns an error if:
/// - `config_path` is provided but the file doesn't exist
/// - A config file exists but cannot be parsed
pub async fn find_and_load_config(
    root: &Path,
    config_path: Option<&Path>,
) -> Result<Option<PackageToolsConfig>> {
    let fs = FileSystemManager::new();

    // Use provided config path or search for default files
    let found_config = if let Some(config) = config_path {
        // Use the explicitly provided config file
        let config_file =
            if config.is_absolute() { config.to_path_buf() } else { root.join(config) };

        if fs.exists(&config_file).await {
            Some(config_file)
        } else {
            return Err(CliError::configuration(format!(
                "Config file not found: {}",
                config_file.display()
            )));
        }
    } else {
        // Search for default config files
        let config_files = vec![
            root.join("repo.config.toml"),
            root.join("repo.config.json"),
            root.join("repo.config.yaml"),
            root.join("repo.config.yml"),
        ];

        debug!(
            "Searching for config in: {}",
            config_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
        );

        let mut found = None;
        for config_file in &config_files {
            if fs.exists(config_file).await {
                found = Some(config_file.clone());
                break;
            }
        }
        found
    };

    // Load configuration if found
    if let Some(config_file) = found_config {
        // Read file content
        let content = fs.read_file_string(&config_file).await.map_err(|e| {
            CliError::configuration(format!(
                "Failed to read config from {}: {}",
                config_file.display(),
                e
            ))
        })?;

        // Determine format from extension
        let format = match config_file.extension().and_then(|e| e.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("json") => ConfigFormat::Json,
            Some("yaml" | "yml") => ConfigFormat::Yaml,
            _ => {
                return Err(CliError::configuration(format!(
                    "Unsupported config file extension: {}",
                    config_file.display()
                )));
            }
        };

        // Parse configuration
        match PackageToolsConfig::from_str(&content, format) {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_file.display());
                Ok(Some(config))
            }
            Err(e) => Err(CliError::configuration(format!(
                "Failed to parse config from {}: {}",
                config_file.display(),
                e
            ))),
        }
    } else {
        debug!("No configuration file found");
        Ok(None)
    }
}

/// Validates that provided package names exist in the workspace.
///
/// This function checks that all package names provided by the user (via `--packages` flag)
/// actually exist in the workspace. This prevents creating changesets or running commands
/// with invalid package names that would fail later in the process.
///
/// # Arguments
///
/// * `provided_packages` - Package names provided by the user
/// * `available_packages` - All package names available in the workspace
///
/// # Returns
///
/// Returns `Ok(())` if all provided packages exist in the workspace.
///
/// # Errors
///
/// Returns a `CliError::validation` error if any package name is not found,
/// listing all invalid packages and suggesting similar names if available.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::validate_package_names;
///
/// let provided = vec!["@myorg/core".to_string(), "@myorg/utils".to_string()];
/// let available = vec!["@myorg/core".to_string(), "@myorg/utils".to_string(), "@myorg/cli".to_string()];
///
/// // This will succeed
/// validate_package_names(&provided, &available)?;
///
/// // This will fail with a helpful error message
/// let invalid = vec!["nonexistent-package".to_string()];
/// validate_package_names(&invalid, &available)?; // Error!
/// ```
pub fn validate_package_names(
    provided_packages: &[String],
    available_packages: &[String],
) -> Result<()> {
    let mut invalid_packages = Vec::new();

    for package in provided_packages {
        if !available_packages.contains(package) {
            invalid_packages.push(package.clone());
        }
    }

    if invalid_packages.is_empty() {
        return Ok(());
    }

    // Build helpful error message with suggestions
    let mut error_message = format!(
        "The following package(s) were not found in the workspace: {}",
        invalid_packages.join(", ")
    );

    // Try to find similar package names for suggestions
    let suggestions: Vec<String> = invalid_packages
        .iter()
        .filter_map(|invalid| find_similar_package(invalid, available_packages))
        .collect();

    if !suggestions.is_empty() {
        error_message.push_str("\n\nDid you mean one of these?");
        for suggestion in &suggestions {
            let _ = write!(error_message, "\n  - {suggestion}");
        }
    }

    // Always show available packages for reference
    if !available_packages.is_empty() {
        error_message.push_str("\n\nAvailable packages:");
        for pkg in available_packages {
            let _ = write!(error_message, "\n  - {pkg}");
        }
    }

    Err(CliError::validation(error_message))
}

/// Finds a similar package name for suggestions.
///
/// Uses simple heuristics to find packages that might be what the user meant:
/// - Checks if the invalid name is a substring of an available package
/// - Checks if an available package ends with the invalid name (missing scope)
/// - Checks for packages with similar endings
fn find_similar_package(invalid: &str, available: &[String]) -> Option<String> {
    // Check if any available package contains the invalid name
    for pkg in available {
        // Case 1: Invalid name is a suffix (user forgot the scope)
        // e.g., "vite-plugin-open-api-server" matches "@websublime/vite-plugin-open-api-server"
        if pkg.ends_with(invalid) {
            return Some(pkg.clone());
        }

        // Case 2: Invalid name is contained in the package name
        if pkg.contains(invalid) {
            return Some(pkg.clone());
        }

        // Case 3: Package name is contained in invalid (user added extra)
        if invalid.contains(pkg.as_str()) {
            return Some(pkg.clone());
        }
    }

    // Case 4: Check for packages with similar last segment
    let invalid_last_segment = invalid.split('/').next_back().unwrap_or(invalid);
    for pkg in available {
        let pkg_last_segment = pkg.split('/').next_back().unwrap_or(pkg);
        if pkg_last_segment == invalid_last_segment {
            return Some(pkg.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests;
