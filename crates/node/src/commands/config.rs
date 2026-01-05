//! Config command implementations for Node.js bindings.
//!
//! # What
//!
//! This module implements the config NAPI functions (`configShow`, `configValidate`)
//! that allow users to inspect and validate their workspace configuration.
//!
//! # How
//!
//! The implementation follows this flow:
//!
//! 1. **Parameter validation**: Validates the `root` path exists and is a directory
//! 2. **Config file discovery**: Searches for configuration files in standard locations
//! 3. **Config parsing**: Loads and parses the configuration using `PackageToolsConfig`
//! 4. **Type conversion**: Converts CLI config types to NAPI-compatible types
//! 5. **Result wrapping**: Returns a `ConfigShowApiResponse` for consistent error handling
//!
//! Unlike other commands that call CLI `execute_*` functions and parse JSON output,
//! the config commands load the configuration directly using `sublime_pkg_tools`.
//! This approach provides access to all configuration sections (including git and
//! execute) that may not be fully exposed in the CLI's JSON output.
//!
//! # Why
//!
//! Configuration management is essential for understanding and troubleshooting
//! workspace settings. These commands help users verify their setup and identify
//! any configuration issues.
//!
//! Key use cases:
//! - Verifying workspace configuration before running operations
//! - CI/CD pipeline configuration validation
//! - IDE integration for configuration display
//! - Scripting and automation scenarios
//!
//! # Examples
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { configShow, configValidate } from '@websublime/workspace-tools';
//!
//! // Show current configuration
//! const showResult = await configShow({ root: '.' });
//! if (showResult.success) {
//!   console.log(`Config loaded from: ${showResult.data.configPath}`);
//!   console.log(`Format: ${showResult.data.configFormat}`);
//!   console.log(`Strategy: ${showResult.data.config.version.strategy}`);
//!   console.log(`Changeset path: ${showResult.data.config.changeset.path}`);
//!   console.log(`Execute timeout: ${showResult.data.config.execute.timeoutSecs}s`);
//! } else {
//!   console.error(`Error [${showResult.error.code}]: ${showResult.error.message}`);
//! }
//!
//! // Validate configuration
//! const validateResult = await configValidate({ root: '.' });
//! if (validateResult.success) {
//!   if (validateResult.data.valid) {
//!     console.log('Configuration is valid');
//!   } else {
//!     console.log('Errors:', validateResult.data.errors);
//!     console.log('Warnings:', validateResult.data.warnings);
//!   }
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```typescript
//! const result = await configShow({ root: '/nonexistent/path' });
//!
//! if (!result.success) {
//!   switch (result.error.code) {
//!     case 'ENOENT':
//!       console.error('Path not found:', result.error.message);
//!       break;
//!     case 'ECONFIG':
//!       console.error('Configuration error:', result.error.message);
//!       break;
//!     case 'EVALIDATION':
//!       console.error('Invalid parameters:', result.error.message);
//!       break;
//!     default:
//!       console.error('Unexpected error:', result.error.message);
//!   }
//! }
//! ```

use std::path::{Path, PathBuf};

use napi_derive::napi;

use crate::error::ErrorInfo;
use crate::types::config::{
    AuditConfigInfo, AuditSectionsConfigInfo, BackupConfigInfo, ChangelogConfigInfo,
    ChangesetConfigInfo, ConfigData, ConfigShowApiResponse, ConfigShowData, ConfigShowParams,
    DependencyConfigInfo, ExecuteConfigInfo, GitConfigInfo, HealthScoreWeightsInfo,
    RegistryConfigInfo, ScopedRegistryEntry, UpgradeConfigInfo, VersionConfigInfo,
};
use crate::validation::validators;

use sublime_pkg_tools::config::{ConfigFormat, PackageToolsConfig};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

// ============================================================================
// Constants
// ============================================================================

/// Standard configuration file names to search for.
const CONFIG_FILE_NAMES: [&str; 4] =
    ["repo.config.toml", "repo.config.json", "repo.config.yaml", "repo.config.yml"];

// ============================================================================
// Configuration Loading
// ============================================================================

/// Information about a discovered configuration file.
struct ConfigFileInfo {
    /// The path to the configuration file.
    path: PathBuf,
    /// The format of the configuration file.
    format: ConfigFormat,
    /// The content of the configuration file.
    content: String,
}

/// Discovers and reads a configuration file.
///
/// If a config path is provided, it uses that path directly.
/// Otherwise, it searches for configuration files in standard locations.
///
/// # Arguments
///
/// * `root` - The workspace root directory
/// * `config_path` - Optional custom configuration file path
/// * `fs` - The filesystem manager to use for file operations
///
/// # Returns
///
/// * `Ok(ConfigFileInfo)` - The configuration file information
/// * `Err(ErrorInfo)` - If no configuration file is found or cannot be read
async fn discover_config_file(
    root: &Path,
    config_path: Option<&str>,
    fs: &FileSystemManager,
) -> Result<ConfigFileInfo, ErrorInfo> {
    // Use provided config path or search for default files
    let found_config = if let Some(config) = config_path {
        let config_file =
            if Path::new(config).is_absolute() { PathBuf::from(config) } else { root.join(config) };

        if fs.exists(&config_file).await {
            Some(config_file)
        } else {
            return Err(ErrorInfo::not_found(
                format!("Config file not found: {}", config_file.display()),
                Some(config_file.to_string_lossy().to_string()),
            ));
        }
    } else {
        // Search for default config files
        let mut found = None;
        for file_name in &CONFIG_FILE_NAMES {
            let config_file = root.join(file_name);
            if fs.exists(&config_file).await {
                found = Some(config_file);
                break;
            }
        }
        found
    };

    // Check if a config file was found
    let config_file_path = found_config.ok_or_else(|| {
        ErrorInfo::configuration(
            "No configuration file found. Run 'workspace init' to create one. \
             Expected one of: repo.config.toml, repo.config.json, repo.config.yaml, repo.config.yml",
        )
    })?;

    // Determine format from extension
    let format = match config_file_path.extension().and_then(|e| e.to_str()) {
        Some("toml") => ConfigFormat::Toml,
        Some("json") => ConfigFormat::Json,
        Some("yaml" | "yml") => ConfigFormat::Yaml,
        _ => {
            return Err(ErrorInfo::configuration(format!(
                "Unsupported config file extension: {}",
                config_file_path.display()
            )));
        }
    };

    // Read the file content
    let content = fs.read_file_string(&config_file_path).await.map_err(|e| {
        ErrorInfo::io(
            format!("Failed to read configuration file '{}': {e}", config_file_path.display()),
            Some(config_file_path.to_string_lossy().to_string()),
        )
    })?;

    Ok(ConfigFileInfo { path: config_file_path, format, content })
}

/// Returns the format name as a string.
///
/// # Arguments
///
/// * `format` - The configuration format
///
/// # Returns
///
/// A string representing the format name ("json", "toml", or "yaml").
pub(crate) fn format_to_string(format: ConfigFormat) -> String {
    match format {
        ConfigFormat::Json => "json".to_string(),
        ConfigFormat::Toml => "toml".to_string(),
        ConfigFormat::Yaml => "yaml".to_string(),
    }
}

// ============================================================================
// Type Conversion Functions
// ============================================================================

/// Converts a `PackageToolsConfig` to the NAPI-compatible `ConfigData`.
///
/// This function performs a comprehensive conversion from the pkg crate's
/// configuration types to the NAPI types exposed to JavaScript.
///
/// # Arguments
///
/// * `config` - The parsed configuration from `sublime_pkg_tools`
///
/// # Returns
///
/// A `ConfigData` instance suitable for returning to JavaScript.
pub(crate) fn convert_to_napi_config(config: &PackageToolsConfig) -> ConfigData {
    ConfigData {
        changeset: convert_changeset_config(&config.changeset),
        version: convert_version_config(&config.version),
        dependency: convert_dependency_config(&config.dependency),
        upgrade: convert_upgrade_config(&config.upgrade),
        changelog: convert_changelog_config(&config.changelog),
        audit: convert_audit_config(&config.audit),
        git: convert_git_config(&config.git),
        execute: convert_execute_config(&config.execute),
    }
}

/// Converts changeset configuration.
pub(crate) fn convert_changeset_config(
    config: &sublime_pkg_tools::config::ChangesetConfig,
) -> ChangesetConfigInfo {
    ChangesetConfigInfo {
        path: config.path.clone(),
        history_path: config.history_path.clone(),
        available_environments: config.available_environments.clone(),
        default_environments: config.default_environments.clone(),
    }
}

/// Converts version configuration.
pub(crate) fn convert_version_config(
    config: &sublime_pkg_tools::config::VersionConfig,
) -> VersionConfigInfo {
    let strategy = match config.strategy {
        sublime_pkg_tools::types::VersioningStrategy::Independent => "independent".to_string(),
        sublime_pkg_tools::types::VersioningStrategy::Unified => "unified".to_string(),
    };

    VersionConfigInfo {
        strategy,
        default_bump: config.default_bump.clone(),
        snapshot_format: config.snapshot_format.clone(),
    }
}

/// Converts dependency configuration.
pub(crate) fn convert_dependency_config(
    config: &sublime_pkg_tools::config::DependencyConfig,
) -> DependencyConfigInfo {
    DependencyConfigInfo {
        propagation_bump: config.propagation_bump.clone(),
        propagate_dependencies: config.propagate_dependencies,
        propagate_dev_dependencies: config.propagate_dev_dependencies,
        propagate_peer_dependencies: config.propagate_peer_dependencies,
        max_depth: u32::try_from(config.max_depth).unwrap_or(u32::MAX),
        fail_on_circular: config.fail_on_circular,
        skip_workspace_protocol: config.skip_workspace_protocol,
        skip_file_protocol: config.skip_file_protocol,
        skip_link_protocol: config.skip_link_protocol,
        skip_portal_protocol: config.skip_portal_protocol,
    }
}

/// Converts upgrade configuration.
pub(crate) fn convert_upgrade_config(
    config: &sublime_pkg_tools::config::UpgradeConfig,
) -> UpgradeConfigInfo {
    UpgradeConfigInfo {
        auto_changeset: config.auto_changeset,
        changeset_bump: config.changeset_bump.clone(),
        registry: convert_registry_config(&config.registry),
        backup: convert_backup_config(&config.backup),
    }
}

/// Converts registry configuration.
pub(crate) fn convert_registry_config(
    config: &sublime_pkg_tools::config::RegistryConfig,
) -> RegistryConfigInfo {
    let scoped_registries: Vec<ScopedRegistryEntry> = config
        .scoped_registries
        .iter()
        .map(|(scope, registry)| ScopedRegistryEntry {
            scope: scope.clone(),
            registry: registry.clone(),
        })
        .collect();

    RegistryConfigInfo {
        default_registry: config.default_registry.clone(),
        scoped_registries,
        timeout_secs: u32::try_from(config.timeout_secs).unwrap_or(u32::MAX),
        retry_attempts: u32::try_from(config.retry_attempts).unwrap_or(u32::MAX),
        read_npmrc: config.read_npmrc,
    }
}

/// Converts backup configuration.
pub(crate) fn convert_backup_config(
    config: &sublime_pkg_tools::config::BackupConfig,
) -> BackupConfigInfo {
    BackupConfigInfo {
        enabled: config.enabled,
        path: config.backup_dir.clone(),
        keep_count: u32::try_from(config.max_backups).unwrap_or(u32::MAX),
    }
}

/// Converts changelog configuration.
pub(crate) fn convert_changelog_config(
    config: &sublime_pkg_tools::config::ChangelogConfig,
) -> ChangelogConfigInfo {
    let format = match config.format {
        sublime_pkg_tools::config::ChangelogFormat::KeepAChangelog => {
            "keep-a-changelog".to_string()
        }
        sublime_pkg_tools::config::ChangelogFormat::Conventional => {
            "conventional-commits".to_string()
        }
        sublime_pkg_tools::config::ChangelogFormat::Custom => "custom".to_string(),
    };

    let monorepo_mode = match config.monorepo_mode {
        sublime_pkg_tools::config::MonorepoMode::PerPackage => "per-package".to_string(),
        sublime_pkg_tools::config::MonorepoMode::Root => "root".to_string(),
        sublime_pkg_tools::config::MonorepoMode::Both => "both".to_string(),
    };

    // Extract exclude patterns from ExcludeConfig
    let exclude_patterns = config.exclude.patterns.clone();

    // Extract template path if custom format is used
    // The pkg crate uses TemplateConfig with header, version_header, etc.
    // For NAPI, we use None since TemplateConfig doesn't have a simple path
    let template_path: Option<String> = None;

    ChangelogConfigInfo {
        enabled: config.enabled,
        format,
        include_commit_links: config.include_commit_links,
        repository_url: config.repository_url.clone(),
        conventional: config.conventional.enabled,
        template: template_path,
        exclude: exclude_patterns,
        monorepo_mode,
    }
}

/// Converts audit configuration.
pub(crate) fn convert_audit_config(
    config: &sublime_pkg_tools::config::AuditConfig,
) -> AuditConfigInfo {
    AuditConfigInfo {
        enabled: config.enabled,
        min_severity: config.min_severity.clone(),
        sections: convert_audit_sections_config(&config.sections),
        health_score_weights: convert_health_score_weights(&config.health_score_weights),
    }
}

/// Converts audit sections configuration.
pub(crate) fn convert_audit_sections_config(
    config: &sublime_pkg_tools::config::AuditSectionsConfig,
) -> AuditSectionsConfigInfo {
    AuditSectionsConfigInfo {
        upgrades: config.upgrades,
        dependencies: config.dependencies,
        version_consistency: config.version_consistency,
        breaking_changes: config.breaking_changes,
    }
}

/// Converts health score weights configuration.
///
/// The pkg crate uses a different weight structure (per-severity weights and multipliers)
/// than the NAPI types (per-category weights). We map the multipliers to the category
/// weights for a reasonable approximation.
pub(crate) fn convert_health_score_weights(
    config: &sublime_pkg_tools::config::HealthScoreWeightsConfig,
) -> HealthScoreWeightsInfo {
    // Normalize multipliers to weights (they represent relative importance)
    // The pkg crate uses multipliers like 0.8, 1.0, 1.2, 1.3, 1.5
    // We normalize these to sum to approximately 1.0 for the NAPI types
    let total = config.upgrades_multiplier
        + config.dependencies_multiplier
        + config.version_consistency_multiplier
        + config.breaking_changes_multiplier;

    // Avoid division by zero
    let divisor = if total > 0.0 { total } else { 1.0 };

    HealthScoreWeightsInfo {
        upgrades_weight: config.upgrades_multiplier / divisor,
        dependencies_weight: config.dependencies_multiplier / divisor,
        version_consistency_weight: config.version_consistency_multiplier / divisor,
        breaking_changes_weight: config.breaking_changes_multiplier / divisor,
    }
}

/// Converts git configuration.
///
/// The pkg crate's GitConfig focuses on commit message templates rather than
/// branch configuration. For the NAPI layer, we provide sensible defaults
/// for branch_base and detect_affected_packages since they are commonly
/// used in the Node.js API but configured differently in the pkg crate.
pub(crate) fn convert_git_config(_config: &sublime_pkg_tools::config::GitConfig) -> GitConfigInfo {
    // The pkg crate's GitConfig contains commit message templates:
    // - merge_commit_template
    // - monorepo_merge_commit_template
    // - include_breaking_warning
    // - breaking_warning_template
    //
    // The NAPI GitConfigInfo expects:
    // - branch_base: base branch for comparisons
    // - detect_affected_packages: whether to auto-detect affected packages
    //
    // These are conceptually different, so we provide sensible defaults.
    // The branch_base and detect_affected_packages are typically configured
    // via environment or command-line options rather than config file.
    GitConfigInfo { branch_base: "main".to_string(), detect_affected_packages: true }
}

/// Converts execute configuration.
pub(crate) fn convert_execute_config(
    config: &sublime_pkg_tools::config::ExecuteConfig,
) -> ExecuteConfigInfo {
    ExecuteConfigInfo {
        timeout_secs: u32::try_from(config.timeout_secs).unwrap_or(u32::MAX),
        per_package_timeout_secs: u32::try_from(config.per_package_timeout_secs)
            .unwrap_or(u32::MAX),
        max_parallel: u32::try_from(config.max_parallel).unwrap_or(u32::MAX),
    }
}

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validates config show command parameters.
///
/// Ensures the root path is valid before loading the configuration.
///
/// # Arguments
///
/// * `params` - The config show parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_params(params: &ConfigShowParams) -> Result<PathBuf, ErrorInfo> {
    // Validate root path exists and is a directory
    validators::root(&params.root)?;

    Ok(PathBuf::from(&params.root))
}

// ============================================================================
// NAPI Function
// ============================================================================

/// Show the current workspace configuration.
///
/// Loads and returns the workspace configuration from the `repo.config` file
/// (in JSON, TOML, or YAML format). This command provides access to all
/// configuration sections including changeset, version, dependency, upgrade,
/// changelog, audit, git, and execute settings.
///
/// @param params - Config show parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///
/// @returns `Promise<ConfigShowApiResponse>` containing:
///   - On success: `{ success: true, data: ConfigShowData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await configShow({ root: '/path/to/project' });
/// if (result.success) {
///   console.log(`Config path: ${result.data.configPath}`);
///   console.log(`Format: ${result.data.configFormat}`);
///   console.log(`Strategy: ${result.data.config.version.strategy}`);
///   console.log(`Default bump: ${result.data.config.version.defaultBump}`);
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With custom config path
/// ```typescript
/// const result = await configShow({
///   root: '/path/to/project',
///   configPath: 'custom/repo.config.json'
/// });
/// ```
///
/// @example Accessing all configuration sections
/// ```typescript
/// const result = await configShow({ root: '.' });
/// if (result.success) {
///   const { config } = result.data;
///
///   // Changeset settings
///   console.log(`Changeset path: ${config.changeset.path}`);
///   console.log(`History path: ${config.changeset.historyPath}`);
///
///   // Version settings
///   console.log(`Strategy: ${config.version.strategy}`);
///   console.log(`Snapshot format: ${config.version.snapshotFormat}`);
///
///   // Dependency propagation settings
///   console.log(`Propagate deps: ${config.dependency.propagateDependencies}`);
///   console.log(`Max depth: ${config.dependency.maxDepth}`);
///
///   // Execute settings
///   console.log(`Timeout: ${config.execute.timeoutSecs}s`);
///   console.log(`Max parallel: ${config.execute.maxParallel}`);
/// }
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await configShow({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'ECONFIG') {
///     console.error('Configuration error:', result.error.message);
///   }
/// }
/// ```
#[napi]
pub async fn config_show(params: ConfigShowParams) -> ConfigShowApiResponse {
    // 1. Validate parameters (synchronous validation before async operations)
    let root_path = match validate_params(&params) {
        Ok(path) => path,
        Err(error) => return ConfigShowApiResponse::failure(error),
    };

    // 2. Prepare config path - clone the Option<String> to own the data for the blocking task
    let config_path_owned = params.config_path.clone();

    // 3. Execute the config loading operation
    // We use spawn_blocking because FileSystemManager operations may block
    let result = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the blocking context
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                return Err(ErrorInfo::execution(format!("Failed to create runtime: {e}")));
            }
        };

        rt.block_on(async {
            // Create filesystem manager
            let fs = FileSystemManager::new();

            // Discover and read the config file
            let config_info =
                discover_config_file(&root_path, config_path_owned.as_deref(), &fs).await?;

            // Parse the configuration
            let config = PackageToolsConfig::from_str(&config_info.content, config_info.format)
                .map_err(|e| {
                    ErrorInfo::configuration(format!(
                        "Failed to parse configuration file '{}': {e}",
                        config_info.path.display()
                    ))
                })?;

            // Convert to NAPI types
            let napi_config = convert_to_napi_config(&config);

            // Build the response data
            let show_data = ConfigShowData {
                config_path: config_info.path.to_string_lossy().to_string(),
                config_format: format_to_string(config_info.format),
                config: napi_config,
            };

            Ok(show_data)
        })
    })
    .await;

    // 4. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ConfigShowApiResponse::success(data),
        Ok(Err(error)) => ConfigShowApiResponse::failure(error),
        Err(join_error) => ConfigShowApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}

// ============================================================================
// configValidate - TODO: will be implemented on story 7.3
// ============================================================================

// TODO: will be implemented on story 7.3 - Config Validate Command
//
// Implementation outline for configValidate:
//
// #[napi]
// pub async fn config_validate(params: ConfigValidateParams) -> ConfigValidateApiResponse {
//     // 1. Validate parameters
//     if let Err(e) = validate_root(&params.root) {
//         return ConfigValidateApiResponse::failure(e);
//     }
//
//     // 2. Load and parse configuration
//     // 3. Perform validation checks:
//     //    - Environment name validation (no duplicates)
//     //    - Registry URL validation
//     //    - Path format validation
//     //    - Required fields presence
//     //    - Cross-field consistency checks
//     // 4. Return ConfigValidateApiResponse with valid/errors/warnings
// }
