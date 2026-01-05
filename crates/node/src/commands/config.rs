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
    ConfigValidateApiResponse, ConfigValidateData, ConfigValidateParams, ConfigValidationIssue,
    DependencyConfigInfo, ExecuteConfigInfo, GitConfigInfo, HealthScoreWeightsInfo,
    RegistryConfigInfo, ScopedRegistryEntry, UpgradeConfigInfo, VersionConfigInfo,
};
use crate::validation::validators;

use sublime_standard_tools::config::Configurable;

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

/// Validates config validate command parameters.
///
/// Ensures the root path is valid before loading and validating the configuration.
///
/// # Arguments
///
/// * `params` - The config validate parameters to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated root path
/// * `Err(ErrorInfo)` - Validation failed
pub(crate) fn validate_validate_params(
    params: &ConfigValidateParams,
) -> Result<PathBuf, ErrorInfo> {
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
// configValidate - Story 7.3
// ============================================================================

/// Parses a validation error message to extract field and message components.
///
/// The pkg crate's validation errors follow the format: "field.path: Error message"
/// This function extracts both components for structured error reporting.
///
/// # Arguments
///
/// * `error_message` - The full error message from validation
///
/// # Returns
///
/// A tuple of (field, message) extracted from the error message.
pub(crate) fn parse_validation_error(error_message: &str) -> (String, String) {
    // Look for the pattern "field.path: message"
    if let Some(colon_pos) = error_message.find(": ") {
        let field = error_message[..colon_pos].trim().to_string();
        let message = error_message[colon_pos + 2..].trim().to_string();
        (field, message)
    } else {
        // If no colon found, use "config" as the field and the full message
        ("config".to_string(), error_message.to_string())
    }
}

/// Generates a suggestion for a validation error based on the field and message.
///
/// Provides helpful suggestions for common validation errors to guide users
/// toward fixing their configuration.
///
/// # Arguments
///
/// * `field` - The configuration field with the issue
/// * `message` - The error message
///
/// # Returns
///
/// An optional suggestion string for fixing the issue.
pub(crate) fn generate_suggestion(field: &str, message: &str) -> Option<String> {
    // Provide suggestions based on common error patterns
    let message_lower = message.to_lowercase();

    if message_lower.contains("cannot be empty") || message_lower.contains("is required") {
        return Some(format!("Provide a valid value for '{field}'"));
    }

    if message_lower.contains("invalid") && message_lower.contains("must be one of") {
        // Extract the valid options from the message if present
        if let Some(start) = message.find("Must be one of:") {
            let options = &message[start + 15..];
            return Some(format!("Use one of the valid options: {options}"));
        }
    }

    if field.contains("path") && message_lower.contains("empty") {
        return Some("Specify a valid file system path".to_string());
    }

    if field.contains("timeout") || field.contains("max_parallel") {
        return Some("Use a positive integer value".to_string());
    }

    if field.contains("weight") || field.contains("multiplier") {
        return Some("Use a positive numeric value".to_string());
    }

    if field.contains("registry") && message_lower.contains("url") {
        return Some("Use a valid URL starting with http:// or https://".to_string());
    }

    if field.contains("environment") && message_lower.contains("default") {
        return Some(
            "Ensure default environments are included in available_environments".to_string(),
        );
    }

    None
}

/// Performs additional semantic validation checks on the configuration.
///
/// These checks go beyond the basic field validation performed by the pkg crate
/// and identify potential issues that could cause problems at runtime.
///
/// # Arguments
///
/// * `config` - The parsed configuration to validate
///
/// # Returns
///
/// A vector of validation warnings found during semantic analysis.
pub(crate) fn perform_semantic_checks(config: &PackageToolsConfig) -> Vec<ConfigValidationIssue> {
    let mut warnings = Vec::new();

    // Check for potential issues that might cause runtime problems

    // Warning: Empty available_environments with default_environments set
    if config.changeset.available_environments.is_empty()
        && !config.changeset.default_environments.is_empty()
    {
        warnings.push(ConfigValidationIssue::warning(
            "changeset.available_environments".to_string(),
            "No available environments defined but default environments are set".to_string(),
        ));
    }

    // Warning: Very high max_parallel could cause resource issues
    if config.execute.max_parallel > 16 {
        warnings.push(ConfigValidationIssue::warning_with_suggestion(
            "execute.max_parallel".to_string(),
            format!(
                "High parallelism value ({}) may cause resource contention",
                config.execute.max_parallel
            ),
            "Consider using a value between 4-16 for optimal performance".to_string(),
        ));
    }

    // Warning: Changelog enabled but no repository URL
    if config.changelog.enabled
        && config.changelog.include_commit_links
        && config.changelog.repository_url.is_none()
    {
        warnings.push(ConfigValidationIssue::warning_with_suggestion(
            "changelog.repository_url".to_string(),
            "Commit links enabled but no repository URL configured".to_string(),
            "Set changelog.repository_url to enable commit links in changelogs".to_string(),
        ));
    }

    // Warning: Backup disabled but upgrade operations may need rollback
    if !config.upgrade.backup.enabled {
        warnings.push(ConfigValidationIssue::info(
            "upgrade.backup.enabled".to_string(),
            "Backup is disabled; upgrade operations cannot be rolled back".to_string(),
        ));
    }

    // Warning: Very short timeout values
    if config.execute.timeout_secs > 0 && config.execute.timeout_secs < 10 {
        warnings.push(ConfigValidationIssue::warning_with_suggestion(
            "execute.timeout_secs".to_string(),
            format!(
                "Very short timeout ({} seconds) may cause premature command failures",
                config.execute.timeout_secs
            ),
            "Consider using at least 30 seconds for most operations".to_string(),
        ));
    }

    if config.execute.per_package_timeout_secs > 0 && config.execute.per_package_timeout_secs < 5 {
        warnings.push(ConfigValidationIssue::warning_with_suggestion(
            "execute.per_package_timeout_secs".to_string(),
            format!(
                "Very short per-package timeout ({} seconds) may cause premature failures",
                config.execute.per_package_timeout_secs
            ),
            "Consider using at least 10 seconds for per-package operations".to_string(),
        ));
    }

    // Warning: Dependency propagation depth is very high
    if config.dependency.max_depth > 10 {
        warnings.push(ConfigValidationIssue::info(
            "dependency.max_depth".to_string(),
            format!(
                "High propagation depth ({}) may cause long processing times in large monorepos",
                config.dependency.max_depth
            ),
        ));
    }

    warnings
}

/// Validate the workspace configuration.
///
/// Loads and validates the workspace configuration from the `repo.config` file
/// (in JSON, TOML, or YAML format). This command performs both structural
/// validation (required fields, valid values) and semantic validation
/// (cross-field consistency, potential issues).
///
/// The validation returns:
/// - `valid: true` if no errors were found (warnings are allowed)
/// - `valid: false` if there are validation errors that must be fixed
/// - A list of errors (issues that must be fixed)
/// - A list of warnings (potential issues that should be reviewed)
///
/// @param params - Config validate parameters containing:
///   - `root`: Workspace root directory path (required)
///   - `configPath`: Optional custom config file path
///
/// @returns `Promise<ConfigValidateApiResponse>` containing:
///   - On success: `{ success: true, data: ConfigValidateData }`
///   - On failure: `{ success: false, error: ErrorInfo }`
///
/// @example Basic usage
/// ```typescript
/// const result = await configValidate({ root: '/path/to/project' });
/// if (result.success) {
///   if (result.data.valid) {
///     console.log('Configuration is valid!');
///   } else {
///     console.error(`Found ${result.data.errors.length} errors`);
///     for (const error of result.data.errors) {
///       console.error(`  [${error.field}]: ${error.message}`);
///       if (error.suggestion) {
///         console.log(`    Suggestion: ${error.suggestion}`);
///       }
///     }
///   }
///
///   if (result.data.warnings.length > 0) {
///     console.warn(`Found ${result.data.warnings.length} warnings`);
///     for (const warning of result.data.warnings) {
///       console.warn(`  [${warning.field}]: ${warning.message}`);
///     }
///   }
/// } else {
///   console.error(`Error: ${result.error.code} - ${result.error.message}`);
/// }
/// ```
///
/// @example With custom config path
/// ```typescript
/// const result = await configValidate({
///   root: '/path/to/project',
///   configPath: 'custom/repo.config.json'
/// });
/// ```
///
/// @example CI/CD pipeline validation
/// ```typescript
/// const result = await configValidate({ root: '.' });
/// if (!result.success) {
///   console.error('Failed to load configuration');
///   process.exit(1);
/// }
///
/// if (!result.data.valid) {
///   console.error('Configuration validation failed:');
///   for (const error of result.data.errors) {
///     console.error(`  - ${error.field}: ${error.message}`);
///   }
///   process.exit(1);
/// }
///
/// // Optionally fail on warnings in strict mode
/// if (process.env.STRICT_CONFIG && result.data.warnings.length > 0) {
///   console.error('Configuration has warnings (strict mode):');
///   for (const warning of result.data.warnings) {
///     console.error(`  - ${warning.field}: ${warning.message}`);
///   }
///   process.exit(1);
/// }
///
/// console.log('Configuration is valid');
/// ```
///
/// @example Error handling
/// ```typescript
/// const result = await configValidate({ root: '/nonexistent' });
/// if (!result.success) {
///   if (result.error.code === 'ENOENT') {
///     console.error('Path not found');
///   } else if (result.error.code === 'ECONFIG') {
///     console.error('Configuration error:', result.error.message);
///   }
/// }
/// ```
#[napi]
pub async fn config_validate(params: ConfigValidateParams) -> ConfigValidateApiResponse {
    // 1. Validate parameters (synchronous validation before async operations)
    let root_path = match validate_validate_params(&params) {
        Ok(path) => path,
        Err(error) => return ConfigValidateApiResponse::failure(error),
    };

    // 2. Prepare config path - clone the Option<String> to own the data for the blocking task
    let config_path_owned = params.config_path.clone();

    // 3. Execute the config loading and validation operation
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

            let config_path_str = config_info.path.to_string_lossy().to_string();

            // Parse the configuration
            let config =
                match PackageToolsConfig::from_str(&config_info.content, config_info.format) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        // Parse error is different from validation error
                        return Err(ErrorInfo::configuration(format!(
                            "Failed to parse configuration file '{}': {e}",
                            config_info.path.display()
                        )));
                    }
                };

            // Perform structural validation using the pkg crate's validate method
            let mut errors: Vec<ConfigValidationIssue> = Vec::new();

            if let Err(validation_error) = config.validate() {
                // Convert the validation error to our structured format
                let error_message = validation_error.to_string();

                // The pkg crate returns errors one at a time (fails fast on first error)
                // Parse the error message to extract field and message
                let (field, message) = parse_validation_error(&error_message);
                let suggestion = generate_suggestion(&field, &message);

                if let Some(suggestion_text) = suggestion {
                    errors.push(ConfigValidationIssue::error_with_suggestion(
                        field,
                        message,
                        suggestion_text,
                    ));
                } else {
                    errors.push(ConfigValidationIssue::error(field, message));
                }
            }

            // Perform semantic validation checks (warnings and info)
            let warnings = perform_semantic_checks(&config);

            // Build the response
            let valid = errors.is_empty();
            let validate_data = ConfigValidateData::new(valid, config_path_str, errors, warnings);

            Ok(validate_data)
        })
    })
    .await;

    // 4. Handle spawn_blocking result
    match result {
        Ok(Ok(data)) => ConfigValidateApiResponse::success(data),
        Ok(Err(error)) => ConfigValidateApiResponse::failure(error),
        Err(join_error) => ConfigValidateApiResponse::failure(ErrorInfo::execution(format!(
            "Task execution failed: {join_error}"
        ))),
    }
}
