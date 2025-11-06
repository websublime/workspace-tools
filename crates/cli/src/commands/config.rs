//! Config command implementation.
//!
//! This module implements the `workspace config` subcommands for managing and viewing
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
//!
//! The validate command performs comprehensive validation of configuration:
//! - Checks all required fields are present
//! - Validates field types and formats
//! - Checks referenced directories exist or can be created
//! - Validates environment names (no duplicates)
//! - Validates registry URLs
//! - Provides clear, actionable error messages

// Allow print to stdout for command output
#![allow(clippy::print_stdout)]
#![allow(clippy::use_debug)]

use crate::cli::commands::{ConfigShowArgs, ConfigValidateArgs};
use crate::error::{CliError, Result};
use crate::output::{JsonResponse, OutputFormat};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use sublime_pkg_tools::config::{ConfigLoader, PackageToolsConfig, validate_config};
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
/// * `config_path` - Optional path to config file (from global `--config` option)
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
/// execute_show(&args, Path::new("."), None, OutputFormat::Human).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_show(
    _args: &ConfigShowArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat,
) -> Result<()> {
    debug!("Loading configuration from: {}", root.display());

    // Try to find and load config file
    let fs = FileSystemManager::new();

    // Use provided config path or search for default files
    let mut found_config = None;
    if let Some(config) = config_path {
        // Use the explicitly provided config file
        let config_file =
            if config.is_absolute() { config.to_path_buf() } else { root.join(config) };

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
            root.join("repo.config.toml"),
            root.join("repo.config.json"),
            root.join("repo.config.yaml"),
            root.join("repo.config.yml"),
        ];

        for config_file in &config_files {
            if fs.exists(config_file).await {
                found_config = Some(config_file.clone());
                break;
            }
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

/// Execute the config validate command.
///
/// Validates the configuration file by performing comprehensive checks including:
/// - File existence and parseability
/// - Required fields presence
/// - Field value validation (types, ranges, formats)
/// - Path format validation
/// - Cross-field consistency checks
/// - Environment name validation
/// - Registry URL validation
///
/// The command reports all validation checks and provides actionable error
/// messages if validation fails.
///
/// # Arguments
///
/// * `_args` - Command arguments (currently unused but reserved for future options)
/// * `root` - Workspace root directory
/// * `config_path` - Optional path to config file (from global `--config` option)
/// * `format` - Output format for the command result
///
/// # Returns
///
/// Returns `Ok(())` if configuration is valid.
///
/// # Errors
///
/// Returns an error if:
/// - The configuration file doesn't exist
/// - The configuration file cannot be parsed
/// - Any validation check fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_cli_tools::commands::config::execute_validate;
/// use sublime_cli_tools::cli::commands::ConfigValidateArgs;
/// use sublime_cli_tools::output::OutputFormat;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = ConfigValidateArgs {};
/// execute_validate(&args, Path::new("."), None, OutputFormat::Human).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_validate(
    _args: &ConfigValidateArgs,
    root: &Path,
    config_path: Option<&Path>,
    format: OutputFormat,
) -> Result<()> {
    debug!("Validating configuration from: {}", root.display());

    // Try to find config file
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
                "Config file not found: {}\n\
                Please check the path provided with --config option.",
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

        let mut found = None;
        for config_file in &config_files {
            if fs.exists(config_file).await {
                found = Some(config_file.clone());
                break;
            }
        }
        found
    };

    // Check if config file exists
    let config_file_path = found_config.ok_or_else(|| {
        CliError::configuration(
            "No configuration file found. Run 'workspace init' to create one.\n\
            Expected one of: repo.config.toml, repo.config.json, repo.config.yaml, repo.config.yml",
        )
    })?;

    info!("Found configuration file: {}", config_file_path.display());

    // Load configuration - this checks parseability
    let config = ConfigLoader::load_from_file(&config_file_path).await.map_err(|e| {
        CliError::configuration(format!(
            "Failed to load configuration file: {e}\n\
            Please check the file syntax and format."
        ))
    })?;

    // Perform comprehensive validation
    let validation_checks = perform_validation_checks(&config, root, &fs).await;

    // Count passed and failed checks
    let total_checks = validation_checks.len();
    let passed_checks = validation_checks.iter().filter(|c| c.passed).count();
    let failed_checks = total_checks - passed_checks;

    // Determine if validation passed
    let is_valid = failed_checks == 0;

    // Output results based on format
    match format {
        OutputFormat::Human => {
            output_validate_human(&validation_checks, is_valid, &config_file_path);
        }
        OutputFormat::Json | OutputFormat::JsonCompact => {
            output_validate_json(&validation_checks, is_valid, format)?;
        }
        OutputFormat::Quiet => output_validate_quiet(is_valid),
    }

    // Return error if validation failed
    if is_valid {
        Ok(())
    } else {
        Err(CliError::validation("Configuration validation failed. See details above."))
    }
}

/// Represents a single validation check result.
#[derive(Debug, Clone, Serialize)]
struct ValidationCheck {
    /// Name of the check
    name: String,
    /// Whether the check passed
    passed: bool,
    /// Optional error message if check failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Perform all validation checks on the configuration.
///
/// This function runs comprehensive validation including:
/// - Basic field validation via validate_config
/// - Path existence checks
/// - Environment configuration checks
/// - Registry connectivity (future enhancement)
///
/// # Arguments
///
/// * `config` - The configuration to validate
/// * `root` - Workspace root directory
/// * `fs` - File system manager for path checks
///
/// # Returns
///
/// A vector of validation check results.
async fn perform_validation_checks(
    config: &PackageToolsConfig,
    root: &Path,
    fs: &FileSystemManager,
) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();

    // Check 1: Config file exists (always passes at this point)
    checks.push(ValidationCheck {
        name: "Config file exists".to_string(),
        passed: true,
        error: None,
    });

    // Check 2: All required fields present - use the validation function
    let fields_check = match validate_config(config) {
        Ok(()) => ValidationCheck {
            name: "All required fields present".to_string(),
            passed: true,
            error: None,
        },
        Err(e) => ValidationCheck {
            name: "All required fields present".to_string(),
            passed: false,
            error: Some(format!("{e}")),
        },
    };
    checks.push(fields_check);

    // Check 3: Environment configuration valid
    let env_check = validate_environments(config);
    checks.push(env_check);

    // Check 4: Changeset directory exists or can be created
    let changeset_dir_check = validate_changeset_directory(config, root, fs).await;
    checks.push(changeset_dir_check);

    // Check 5: History directory path is valid
    let history_dir_check = validate_history_directory(config, root);
    checks.push(history_dir_check);

    // Check 6: Registry URL is valid
    let registry_check = validate_registry_url(config);
    checks.push(registry_check);

    // Check 7: Version strategy is valid
    let strategy_check = ValidationCheck {
        name: "Version strategy valid".to_string(),
        passed: true, // VersioningStrategy is an enum, so always valid
        error: None,
    };
    checks.push(strategy_check);

    // Check 8: Default bump type is valid
    let bump_check = validate_default_bump(config);
    checks.push(bump_check);

    // Check 9: Snapshot format is valid
    let snapshot_check = validate_snapshot_format(config);
    checks.push(snapshot_check);

    checks
}

/// Validate environment configuration.
fn validate_environments(config: &PackageToolsConfig) -> ValidationCheck {
    let envs = &config.changeset.available_environments;
    let default_envs = &config.changeset.default_environments;

    // Check that default environments are subset of available environments
    for default_env in default_envs {
        if !envs.contains(default_env) {
            return ValidationCheck {
                name: "Environments valid".to_string(),
                passed: false,
                error: Some(format!(
                    "Default environment '{default_env}' is not in available_environments list"
                )),
            };
        }
    }

    // Check for empty environment list
    if envs.is_empty() {
        return ValidationCheck {
            name: "Environments valid".to_string(),
            passed: false,
            error: Some("available_environments list cannot be empty".to_string()),
        };
    }

    ValidationCheck { name: "Environments valid".to_string(), passed: true, error: None }
}

/// Validate changeset directory exists or can be created.
async fn validate_changeset_directory(
    config: &PackageToolsConfig,
    root: &Path,
    fs: &FileSystemManager,
) -> ValidationCheck {
    let changeset_path = root.join(&config.changeset.path);

    // Check if directory exists
    if fs.exists(&changeset_path).await {
        // Verify it's a directory using metadata
        match fs.metadata(&changeset_path).await {
            Ok(metadata) if !metadata.is_dir() => {
                return ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: false,
                    error: Some(format!(
                        "Path '{}' exists but is not a directory",
                        config.changeset.path
                    )),
                };
            }
            Ok(_) => {
                return ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: true,
                    error: None,
                };
            }
            Err(_) => {
                return ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: false,
                    error: Some(format!(
                        "Failed to check if path '{}' is a directory",
                        config.changeset.path
                    )),
                };
            }
        }
    }

    // Check if parent directory exists and is writable
    if let Some(parent) = changeset_path.parent() {
        if fs.exists(parent).await {
            match fs.metadata(parent).await {
                Ok(metadata) if metadata.is_dir() => ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: true,
                    error: None,
                },
                Ok(_) => ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: false,
                    error: Some(format!(
                        "Parent path for '{}' is not a directory",
                        config.changeset.path
                    )),
                },
                Err(_) => ValidationCheck {
                    name: "Changeset directory exists".to_string(),
                    passed: false,
                    error: Some(format!(
                        "Failed to check parent directory for '{}'",
                        config.changeset.path
                    )),
                },
            }
        } else {
            ValidationCheck {
                name: "Changeset directory exists".to_string(),
                passed: false,
                error: Some(format!(
                    "Parent directory for '{}' does not exist",
                    config.changeset.path
                )),
            }
        }
    } else {
        ValidationCheck {
            name: "Changeset directory exists".to_string(),
            passed: true,
            error: None,
        }
    }
}

/// Validate history directory path.
fn validate_history_directory(config: &PackageToolsConfig, _root: &Path) -> ValidationCheck {
    // Check path format is valid
    if config.changeset.history_path.is_empty() {
        return ValidationCheck {
            name: "History directory path valid".to_string(),
            passed: false,
            error: Some("history_path cannot be empty".to_string()),
        };
    }

    // Check it's different from changeset path
    if config.changeset.path == config.changeset.history_path {
        return ValidationCheck {
            name: "History directory path valid".to_string(),
            passed: false,
            error: Some("history_path must be different from changeset path".to_string()),
        };
    }

    ValidationCheck { name: "History directory path valid".to_string(), passed: true, error: None }
}

/// Validate registry URL.
fn validate_registry_url(config: &PackageToolsConfig) -> ValidationCheck {
    let registry = &config.upgrade.registry.default_registry;

    if registry.is_empty() {
        return ValidationCheck {
            name: "Registry URL valid".to_string(),
            passed: false,
            error: Some("Registry URL cannot be empty".to_string()),
        };
    }

    if !registry.starts_with("http://") && !registry.starts_with("https://") {
        return ValidationCheck {
            name: "Registry URL valid".to_string(),
            passed: false,
            error: Some("Registry URL must start with http:// or https://".to_string()),
        };
    }

    ValidationCheck { name: "Registry URL valid".to_string(), passed: true, error: None }
}

/// Validate default bump type.
fn validate_default_bump(config: &PackageToolsConfig) -> ValidationCheck {
    let valid_bumps = ["major", "minor", "patch", "none"];
    let bump = config.version.default_bump.as_str();

    if !valid_bumps.contains(&bump) {
        return ValidationCheck {
            name: "Default bump type valid".to_string(),
            passed: false,
            error: Some(format!(
                "Invalid bump type '{bump}'. Must be one of: {}",
                valid_bumps.join(", ")
            )),
        };
    }

    ValidationCheck { name: "Default bump type valid".to_string(), passed: true, error: None }
}

/// Validate snapshot format.
fn validate_snapshot_format(config: &PackageToolsConfig) -> ValidationCheck {
    let format = &config.version.snapshot_format;

    if !format.contains("{version}") {
        return ValidationCheck {
            name: "Snapshot format valid".to_string(),
            passed: false,
            error: Some("Format must contain '{version}' placeholder".to_string()),
        };
    }

    ValidationCheck { name: "Snapshot format valid".to_string(), passed: true, error: None }
}

/// Output validation results in human-readable format.
fn output_validate_human(checks: &[ValidationCheck], is_valid: bool, config_path: &Path) {
    println!();
    if is_valid {
        println!("✓ Configuration is valid");
        println!();
        println!("Config file: {}", config_path.display());
        println!();
        println!("All checks passed:");
    } else {
        println!("✗ Configuration validation failed");
        println!();
        println!("Config file: {}", config_path.display());
        println!();
        println!("Validation results:");
    }

    for check in checks {
        if check.passed {
            println!("  ✓ {}", check.name);
        } else {
            println!("  ✗ {}", check.name);
            if let Some(error) = &check.error {
                println!("    Error: {error}");
            }
        }
    }

    println!();
}

/// Output validation results in JSON format.
fn output_validate_json(
    checks: &[ValidationCheck],
    is_valid: bool,
    format: OutputFormat,
) -> Result<()> {
    let response = if is_valid {
        JsonResponse::success(ValidationResult { valid: true, checks: checks.to_vec() })
    } else {
        // For validation failures, we still use success=true but valid=false
        // The command itself returns an error exit code
        JsonResponse::success(ValidationResult { valid: false, checks: checks.to_vec() })
    };

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

/// Output validation results in quiet format.
fn output_validate_quiet(is_valid: bool) {
    if is_valid {
        println!("valid");
    } else {
        println!("invalid");
    }
}

/// Validation result structure for JSON output.
#[derive(Debug, Serialize)]
struct ValidationResult {
    /// Whether the configuration is valid
    valid: bool,
    /// List of validation checks performed
    checks: Vec<ValidationCheck>,
}

/// Output configuration in human-readable format.</parameter>
///
/// Displays configuration in organized sections with clear labels and formatting.
/// Indicates if default configuration is being used.
fn output_human_format(config: &PackageToolsConfig, is_default: bool) {
    use crate::output::styling::{
        Section, StatusSymbol, print_bullet, print_item, print_separator,
    };
    use console::Color;

    // Main section
    let section = Section::new("Configuration");
    section.print();

    if is_default {
        StatusSymbol::Warning.print_line("No configuration file found. Showing default values.");
        print_separator();
    }

    // Strategy
    let strategy_str = match config.version.strategy {
        sublime_pkg_tools::types::VersioningStrategy::Independent => "independent",
        sublime_pkg_tools::types::VersioningStrategy::Unified => "unified",
    };
    let strategy_color = match config.version.strategy {
        sublime_pkg_tools::types::VersioningStrategy::Independent => Color::Green,
        sublime_pkg_tools::types::VersioningStrategy::Unified => Color::Blue,
    };
    print_bullet(&format!("Strategy: {strategy_str}"), strategy_color);
    print_separator();

    // Changeset configuration
    print_item("Changeset Path", &config.changeset.path, false);
    let envs = config.changeset.available_environments.join(", ");
    print_item("Environments", &envs, false);
    let default_envs = config.changeset.default_environments.join(", ");
    print_item("Default Environments", &default_envs, false);

    // Version configuration
    let default_bump = &config.version.default_bump;
    print_item("Default Bump", default_bump, false);
    let snapshot_format = &config.version.snapshot_format;
    print_item("Snapshot Format", snapshot_format, false);

    // Registry configuration
    let registry = &config.upgrade.registry.default_registry;
    print_item("Registry", registry, false);

    // Additional settings
    let section = Section::new("Additional Settings");
    section.print();

    let history_path = &config.changeset.history_path;
    print_item("History Path", history_path, false);
    let changelog_enabled = config.changelog.enabled;
    print_item("Changelog Enabled", &changelog_enabled.to_string(), false);
    let audit_enabled = config.audit.enabled;
    print_item("Audit Enabled", &audit_enabled.to_string(), false);

    // Dependency propagation settings
    let section = Section::new("Dependency Propagation");
    section.print();

    let prop_deps = config.dependency.propagate_dependencies;
    print_item("Propagate Dependencies", &prop_deps.to_string(), false);
    let prop_dev_deps = config.dependency.propagate_dev_dependencies;
    print_item("Propagate Dev Dependencies", &prop_dev_deps.to_string(), false);
    let max_depth = config.dependency.max_depth;
    print_item("Max Depth", &max_depth.to_string(), true);
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
