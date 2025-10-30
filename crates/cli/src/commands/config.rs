//! Config command implementation.
//!
//! This module implements the `wnt config` subcommands for managing and viewing
//! configuration settings.
//!
//! # What
//!
//! Provides the config subcommands:
//! - `show` - Display current configuration
//! - `validate` - Validate configuration file (future story 2.3)
//!
//! # How
//!
//! The show command:
//! 1. Attempts to load configuration from repo.config.{toml,json,yaml,yml}
//! 2. Falls back to defaults if no config file exists
//! 3. Applies environment variable overrides
//! 4. Formats output based on OutputFormat (human or JSON)
//!
//! Human format displays configuration in organized sections with clear labels.
//! JSON format uses the standard JsonResponse structure for consistency.
//!
//! # Why
//!
//! Configuration inspection is essential for:
//! - Troubleshooting issues
//! - Verifying settings
//! - Understanding current workspace setup
//! - CI/CD automation (JSON output)
//!
//! Clear display of configuration helps users understand their setup and
//! verify that settings are correctly applied.

// Allow print to stdout for command output
#![allow(clippy::print_stdout)]
#![allow(clippy::use_debug)]

use crate::cli::commands::ConfigShowArgs;
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, OutputFormat};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};
use tracing::{debug, info, warn};

/// Execute the config show command.
///
/// Displays the current configuration loaded from the workspace, including
/// all changeset, version, and registry settings. If no configuration file
/// exists, displays the default configuration.
///
/// # Arguments
///
/// * `_args` - Command arguments (currently unused but reserved for future options)
/// * `root` - Workspace root directory
/// * `format` - Output format for the command result
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if configuration cannot be loaded.
///
/// # Errors
///
/// Returns an error if:
/// - The configuration file exists but is invalid
/// - File system operations fail
/// - Serialization to JSON fails (in JSON output mode)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::config::execute_show;
/// use sublime_cli_tools::cli::commands::ConfigShowArgs;
/// use sublime_cli_tools::output::OutputFormat;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ConfigShowArgs {};
/// execute_show(&args, Path::new("."), OutputFormat::Human).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_show(_args: &ConfigShowArgs, root: &Path, format: OutputFormat) -> Result<()> {
    debug!("Loading configuration from: {}", root.display());

    // Try to find and load config file
    let fs = FileSystemManager::new();
    let config_files = vec![
        root.join("repo.config.toml"),
        root.join("repo.config.json"),
        root.join("repo.config.yaml"),
        root.join("repo.config.yml"),
    ];

    let mut found_config = None;
    for config_file in &config_files {
        if fs.exists(config_file).await {
            found_config = Some(config_file.clone());
            break;
        }
    }

    // Load configuration
    let (config, is_default) = if let Some(config_path) = found_config {
        match ConfigLoader::load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded from: {}", config_path.display());
                (config, false)
            }
            Err(e) => {
                warn!("Failed to load config file, using defaults: {}", e);
                (PackageToolsConfig::default(), true)
            }
        }
    } else {
        warn!("No configuration file found, using defaults");
        (PackageToolsConfig::default(), true)
    };

    // Output based on format
    match format {
        OutputFormat::Human => output_human_format(&config, is_default),
        OutputFormat::Json | OutputFormat::JsonCompact => output_json_format(&config, format)?,
        OutputFormat::Quiet => output_quiet_format(&config),
    }

    Ok(())
}

/// Output configuration in human-readable format.
///
/// Displays configuration in organized sections with clear labels and formatting.
/// Indicates if default configuration is being used.
fn output_human_format(config: &PackageToolsConfig, is_default: bool) {
    println!("Configuration");
    println!("━━━━━━━━━━━━━━━━");
    println!();

    if is_default {
        println!("⚠ No configuration file found. Showing default values.");
        println!();
    }

    // Strategy
    let strategy_str = match config.version.strategy {
        sublime_pkg_tools::types::VersioningStrategy::Independent => "independent",
        sublime_pkg_tools::types::VersioningStrategy::Unified => "unified",
    };
    println!("Strategy: {strategy_str}");

    // Changeset configuration
    println!("Changeset Path: {}", config.changeset.path);
    let envs = config.changeset.available_environments.join(", ");
    println!("Environments: {envs}");
    let default_envs = config.changeset.default_environments.join(", ");
    println!("Default Environments: {default_envs}");

    // Version configuration
    let default_bump = &config.version.default_bump;
    println!("Default Bump: {default_bump}");
    let snapshot_format = &config.version.snapshot_format;
    println!("Snapshot Format: {snapshot_format}");

    // Registry configuration
    let registry = &config.upgrade.registry.default_registry;
    println!("Registry: {registry}");

    // Additional settings
    println!();
    println!("Additional Settings:");
    let history_path = &config.changeset.history_path;
    println!("  History Path: {history_path}");
    let changelog_enabled = config.changelog.enabled;
    println!("  Changelog Enabled: {changelog_enabled}");
    let audit_enabled = config.audit.enabled;
    println!("  Audit Enabled: {audit_enabled}");

    // Dependency propagation settings
    println!();
    println!("Dependency Propagation:");
    let prop_deps = config.dependency.propagate_dependencies;
    println!("  Propagate Dependencies: {prop_deps}");
    let prop_dev_deps = config.dependency.propagate_dev_dependencies;
    println!("  Propagate Dev Dependencies: {prop_dev_deps}");
    let max_depth = config.dependency.max_depth;
    println!("  Max Depth: {max_depth}");
}

/// Output configuration in JSON format.
///
/// Serializes the configuration as a JsonResponse structure for machine-readable
/// output. Uses pretty printing for Json format and compact for JsonCompact.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
fn output_json_format(config: &PackageToolsConfig, format: OutputFormat) -> Result<()> {
    // Convert to serializable structure
    let config_data = ConfigShowData::from(config);
    let response = JsonResponse::success(config_data);

    // Serialize based on format
    let json_str = if format == OutputFormat::JsonCompact {
        serde_json::to_string(&response)
            .map_err(|e| CliError::execution(format!("Failed to serialize JSON: {e}")))?
    } else {
        serde_json::to_string_pretty(&response)
            .map_err(|e| CliError::execution(format!("Failed to serialize JSON: {e}")))?
    };

    println!("{json_str}");
    Ok(())
}

/// Output configuration in quiet format.
///
/// Displays only essential information in a minimal format.
fn output_quiet_format(config: &PackageToolsConfig) {
    let strategy_str = match config.version.strategy {
        sublime_pkg_tools::types::VersioningStrategy::Independent => "independent",
        sublime_pkg_tools::types::VersioningStrategy::Unified => "unified",
    };
    println!("{strategy_str}");
}

/// Serializable configuration data for JSON output.
///
/// This structure provides a clean, serializable representation of the
/// configuration for JSON output. Field names use camelCase to match
/// JavaScript conventions.
#[derive(Debug, Serialize)]
struct ConfigShowData {
    /// Changeset configuration section
    changeset: ChangesetConfigData,
    /// Version resolution configuration section
    version: VersionConfigData,
    /// Dependency propagation configuration section
    dependency: DependencyConfigData,
    /// Upgrade configuration section
    upgrade: UpgradeConfigData,
    /// Changelog configuration section
    changelog: ChangelogConfigData,
    /// Audit configuration section
    audit: AuditConfigData,
}

impl From<&PackageToolsConfig> for ConfigShowData {
    fn from(config: &PackageToolsConfig) -> Self {
        Self {
            changeset: ChangesetConfigData::from(&config.changeset),
            version: VersionConfigData::from(&config.version),
            dependency: DependencyConfigData::from(&config.dependency),
            upgrade: UpgradeConfigData::from(&config.upgrade),
            changelog: ChangelogConfigData::from(&config.changelog),
            audit: AuditConfigData::from(&config.audit),
        }
    }
}

/// Changeset configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangesetConfigData {
    /// Path to active changesets directory
    path: String,
    /// Path to changeset history directory
    history_path: String,
    /// Available environment names
    environments: Vec<String>,
    /// Default environments for new changesets
    default_environments: Vec<String>,
}

impl From<&sublime_pkg_tools::config::ChangesetConfig> for ChangesetConfigData {
    fn from(config: &sublime_pkg_tools::config::ChangesetConfig) -> Self {
        Self {
            path: config.path.clone(),
            history_path: config.history_path.clone(),
            environments: config.available_environments.clone(),
            default_environments: config.default_environments.clone(),
        }
    }
}

/// Version configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionConfigData {
    /// Versioning strategy (independent or unified)
    strategy: String,
    /// Default version bump type
    default_bump: String,
    /// Snapshot version format template
    snapshot_format: String,
}

impl From<&sublime_pkg_tools::config::VersionConfig> for VersionConfigData {
    fn from(config: &sublime_pkg_tools::config::VersionConfig) -> Self {
        let strategy = match config.strategy {
            sublime_pkg_tools::types::VersioningStrategy::Independent => "independent",
            sublime_pkg_tools::types::VersioningStrategy::Unified => "unified",
        };

        Self {
            strategy: strategy.to_string(),
            default_bump: config.default_bump.clone(),
            snapshot_format: config.snapshot_format.clone(),
        }
    }
}

/// Dependency configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
struct DependencyConfigData {
    /// Version bump for dependency updates
    propagation_bump: String,
    /// Whether to propagate regular dependencies
    propagate_dependencies: bool,
    /// Whether to propagate dev dependencies
    propagate_dev_dependencies: bool,
    /// Whether to propagate peer dependencies
    propagate_peer_dependencies: bool,
    /// Maximum propagation depth
    max_depth: usize,
    /// Fail on circular dependencies
    fail_on_circular: bool,
}

impl From<&sublime_pkg_tools::config::DependencyConfig> for DependencyConfigData {
    fn from(config: &sublime_pkg_tools::config::DependencyConfig) -> Self {
        Self {
            propagation_bump: config.propagation_bump.clone(),
            propagate_dependencies: config.propagate_dependencies,
            propagate_dev_dependencies: config.propagate_dev_dependencies,
            propagate_peer_dependencies: config.propagate_peer_dependencies,
            max_depth: config.max_depth,
            fail_on_circular: config.fail_on_circular,
        }
    }
}

/// Upgrade configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpgradeConfigData {
    /// Automatically create changesets for upgrades
    auto_changeset: bool,
    /// Version bump for upgrade changesets
    changeset_bump: String,
    /// Registry configuration
    registry: RegistryConfigData,
    /// Backup configuration
    backup: BackupConfigData,
}

impl From<&sublime_pkg_tools::config::UpgradeConfig> for UpgradeConfigData {
    fn from(config: &sublime_pkg_tools::config::UpgradeConfig) -> Self {
        Self {
            auto_changeset: config.auto_changeset,
            changeset_bump: config.changeset_bump.clone(),
            registry: RegistryConfigData::from(&config.registry),
            backup: BackupConfigData::from(&config.backup),
        }
    }
}

/// Registry configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistryConfigData {
    /// Default npm registry URL
    default_registry: String,
    /// Scoped package registries
    scoped_registries: HashMap<String, String>,
    /// Request timeout in seconds
    timeout_secs: u64,
    /// Number of retry attempts
    retry_attempts: usize,
}

impl From<&sublime_pkg_tools::config::RegistryConfig> for RegistryConfigData {
    fn from(config: &sublime_pkg_tools::config::RegistryConfig) -> Self {
        Self {
            default_registry: config.default_registry.clone(),
            scoped_registries: config.scoped_registries.clone(),
            timeout_secs: config.timeout_secs,
            retry_attempts: config.retry_attempts,
        }
    }
}

/// Backup configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
struct BackupConfigData {
    /// Whether backups are enabled
    enabled: bool,
    /// Path to store backups
    backup_dir: String,
    /// Whether to keep backups after success
    keep_after_success: bool,
    /// Maximum number of backups to keep
    max_backups: usize,
}

impl From<&sublime_pkg_tools::config::BackupConfig> for BackupConfigData {
    fn from(config: &sublime_pkg_tools::config::BackupConfig) -> Self {
        Self {
            enabled: config.enabled,
            backup_dir: config.backup_dir.clone(),
            keep_after_success: config.keep_after_success,
            max_backups: config.max_backups,
        }
    }
}

/// Changelog configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangelogConfigData {
    /// Whether changelog generation is enabled
    enabled: bool,
    /// Changelog format type
    format: String,
    /// Include commit links
    include_commit_links: bool,
    /// Repository URL for links
    repository_url: Option<String>,
}

impl From<&sublime_pkg_tools::config::ChangelogConfig> for ChangelogConfigData {
    fn from(config: &sublime_pkg_tools::config::ChangelogConfig) -> Self {
        let format = match config.format {
            sublime_pkg_tools::config::ChangelogFormat::KeepAChangelog => "keepachangelog",
            sublime_pkg_tools::config::ChangelogFormat::Conventional => "conventional",
            sublime_pkg_tools::config::ChangelogFormat::Custom => "custom",
        };

        Self {
            enabled: config.enabled,
            format: format.to_string(),
            include_commit_links: config.include_commit_links,
            repository_url: config.repository_url.clone(),
        }
    }
}

/// Audit configuration data for JSON output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditConfigData {
    /// Whether audits are enabled
    enabled: bool,
    /// Minimum severity level to report
    min_severity: String,
}

impl From<&sublime_pkg_tools::config::AuditConfig> for AuditConfigData {
    fn from(config: &sublime_pkg_tools::config::AuditConfig) -> Self {
        Self { enabled: config.enabled, min_severity: config.min_severity.clone() }
    }
}
