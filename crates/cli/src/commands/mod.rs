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
pub mod init;
pub mod upgrade;
pub mod version;

// Common utilities
use crate::error::{CliError, Result};
use std::path::Path;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
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
        match ConfigLoader::load_from_file(&config_file).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_file.display());
                Ok(Some(config))
            }
            Err(e) => Err(CliError::configuration(format!(
                "Failed to load config from {}: {}",
                config_file.display(),
                e
            ))),
        }
    } else {
        debug!("No configuration file found");
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
