//! Configuration validation utilities for package tools.
//!
//! **What**: Provides comprehensive validation for configuration structures with detailed
//! error messages and validation rules.
//!
//! **How**: This module extends the basic `Configurable::validate()` implementations with
//! additional validation logic, path checking, and cross-field validation. It provides
//! detailed error messages that help users fix configuration issues.
//!
//! **Why**: To ensure configuration is valid before use, preventing runtime errors and
//! providing clear, actionable error messages when configuration is invalid.

use std::collections::HashSet;
use std::path::Path;

use sublime_standard_tools::config::{ConfigError, ConfigResult, Configurable};

use super::PackageToolsConfig;

/// Validates a configuration structure with enhanced error messages.
///
/// This function performs comprehensive validation beyond the basic `validate()`
/// method, including:
/// - Path validation (checking if paths are valid format)
/// - Cross-field validation (checking relationships between fields)
/// - Value range validation
/// - Format validation
///
/// # Arguments
///
/// * `config` - The configuration to validate
///
/// # Returns
///
/// `Ok(())` if the configuration is valid.
///
/// # Errors
///
/// Returns a detailed error if validation fails, including:
/// - Which field is invalid
/// - Why it's invalid
/// - Suggested fixes
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::{PackageToolsConfig, validate_config};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = PackageToolsConfig::default();
/// validate_config(&config)?;
/// println!("Configuration is valid");
/// # Ok(())
/// # }
/// ```
pub fn validate_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    // First run basic validation
    config.validate()?;

    // Additional validation
    validate_changeset_config(config)?;
    validate_version_config(config)?;
    validate_dependency_config(config)?;
    validate_upgrade_config(config)?;
    validate_changelog_config(config)?;
    validate_git_config(config)?;
    validate_audit_config(config)?;

    Ok(())
}

/// Validates changeset configuration.
fn validate_changeset_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let changeset = &config.changeset;

    // Validate path format
    if changeset.path.contains("..") {
        return Err(ConfigError::validation(
            "changeset.path: Path should not contain '..' (parent directory references). Use absolute or simple relative paths.",
        ));
    }

    if changeset.history_path.contains("..") {
        return Err(ConfigError::validation(
            "changeset.history_path: Path should not contain '..' (parent directory references). Use absolute or simple relative paths.",
        ));
    }

    // Validate that history path is different from changeset path
    if changeset.path == changeset.history_path {
        return Err(ConfigError::validation(
            "changeset.history_path: History path must be different from changeset path to avoid conflicts.",
        ));
    }

    // Validate environment names
    for env in &changeset.available_environments {
        if env.is_empty() {
            return Err(ConfigError::validation(
                "changeset.available_environments: Environment names cannot be empty.",
            ));
        }

        if env.contains(char::is_whitespace) {
            return Err(ConfigError::validation(format!(
                "changeset.available_environments: Environment name '{}' contains whitespace. Use kebab-case or underscores instead.",
                env
            )));
        }
    }

    // Check for duplicate environments
    let mut seen = HashSet::new();
    for env in &changeset.available_environments {
        if !seen.insert(env) {
            return Err(ConfigError::validation(format!(
                "changeset.available_environments: Duplicate environment name '{}'. Each environment must be unique.",
                env
            )));
        }
    }

    Ok(())
}

/// Validates version configuration.
fn validate_version_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let version = &config.version;

    // Validate default_bump
    let valid_bumps = ["major", "minor", "patch", "none"];
    if !valid_bumps.contains(&version.default_bump.as_str()) {
        return Err(ConfigError::validation(format!(
            "version.default_bump: Invalid bump type '{}'. Must be one of: {}",
            version.default_bump,
            valid_bumps.join(", ")
        )));
    }

    // Validate strategy - VersioningStrategy is an enum, so we check it directly
    // The validation is already done by the enum type, no additional check needed here

    // Validate snapshot format if present
    if !version.snapshot_format.contains("{version}") {
        return Err(ConfigError::validation(
            "version.snapshot_format: Format must contain '{version}' placeholder.",
        ));
    }

    Ok(())
}

/// Validates dependency configuration.
fn validate_dependency_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let dependency = &config.dependency;

    // Validate propagation_bump
    let valid_bumps = ["major", "minor", "patch", "none"];
    if !valid_bumps.contains(&dependency.propagation_bump.as_str()) {
        return Err(ConfigError::validation(format!(
            "dependency.propagation_bump: Invalid bump type '{}'. Must be one of: {}",
            dependency.propagation_bump,
            valid_bumps.join(", ")
        )));
    }

    // Validate max_depth
    if dependency.max_depth == 0 {
        return Err(ConfigError::validation(
            "dependency.max_depth: Must be greater than 0. Use a reasonable value like 10.",
        ));
    }

    if dependency.max_depth > 100 {
        return Err(ConfigError::validation(
            "dependency.max_depth: Value is very high (>100), which may cause performance issues. Consider a lower value like 10.",
        ));
    }

    // Validate that at least one propagation is enabled
    if !dependency.propagate_dependencies
        && !dependency.propagate_dev_dependencies
        && !dependency.propagate_peer_dependencies
    {
        return Err(ConfigError::validation(
            "dependency: At least one of propagate_dependencies, propagate_dev_dependencies, or propagate_peer_dependencies must be true.",
        ));
    }

    Ok(())
}

/// Validates upgrade configuration.
fn validate_upgrade_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let upgrade = &config.upgrade;

    // Validate changeset_bump if auto_changeset is enabled
    if upgrade.auto_changeset {
        let valid_bumps = ["major", "minor", "patch", "none"];
        if !valid_bumps.contains(&upgrade.changeset_bump.as_str()) {
            return Err(ConfigError::validation(format!(
                "upgrade.changeset_bump: Invalid bump type '{}'. Must be one of: {}",
                upgrade.changeset_bump,
                valid_bumps.join(", ")
            )));
        }
    }

    // Validate registry configuration
    let registry = &upgrade.registry;

    if registry.default_registry.is_empty() {
        return Err(ConfigError::validation(
            "upgrade.registry.default_registry: Registry URL cannot be empty.",
        ));
    }

    if !registry.default_registry.starts_with("http://")
        && !registry.default_registry.starts_with("https://")
    {
        return Err(ConfigError::validation(
            "upgrade.registry.default_registry: Registry URL must start with http:// or https://",
        ));
    }

    if registry.timeout_secs == 0 {
        return Err(ConfigError::validation(
            "upgrade.registry.timeout_secs: Timeout must be greater than 0.",
        ));
    }

    if registry.timeout_secs > 300 {
        return Err(ConfigError::validation(
            "upgrade.registry.timeout_secs: Timeout is very high (>300s). Consider a lower value like 30.",
        ));
    }

    if registry.retry_attempts > 10 {
        return Err(ConfigError::validation(
            "upgrade.registry.retry_attempts: Too many retry attempts (>10). Consider a lower value like 3.",
        ));
    }

    if registry.retry_delay_ms == 0 {
        return Err(ConfigError::validation(
            "upgrade.registry.retry_delay_ms: Retry delay must be greater than 0.",
        ));
    }

    // Validate backup configuration
    let backup = &upgrade.backup;

    if backup.enabled {
        if backup.backup_dir.is_empty() {
            return Err(ConfigError::validation(
                "upgrade.backup.backup_dir: Backup directory cannot be empty when backup is enabled.",
            ));
        }

        if backup.max_backups == 0 {
            return Err(ConfigError::validation(
                "upgrade.backup.max_backups: Must be greater than 0 when backup is enabled.",
            ));
        }

        if backup.max_backups > 100 {
            return Err(ConfigError::validation(
                "upgrade.backup.max_backups: Too many backups (>100). Consider a lower value like 10.",
            ));
        }
    }

    Ok(())
}

/// Validates changelog configuration.
fn validate_changelog_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let changelog = &config.changelog;

    if changelog.enabled {
        if changelog.filename.is_empty() {
            return Err(ConfigError::validation(
                "changelog.filename: Filename cannot be empty when changelog is enabled.",
            ));
        }

        // Validate repository URL if commit links are enabled
        if changelog.include_commit_links
            && let Some(ref url) = changelog.repository_url {
                if url.is_empty() {
                    return Err(ConfigError::validation(
                        "changelog.repository_url: URL cannot be empty when include_commit_links is true.",
                    ));
                }

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(ConfigError::validation(
                        "changelog.repository_url: URL must start with http:// or https://",
                    ));
                }
            }
            // Note: repository_url being None is allowed - it will be detected at runtime when actually generating changelogs

        // Validate tag formats
        if changelog.version_tag_format.is_empty() {
            return Err(ConfigError::validation(
                "changelog.version_tag_format: Tag format cannot be empty.",
            ));
        }

        if !changelog.version_tag_format.contains("{version}") {
            return Err(ConfigError::validation(
                "changelog.version_tag_format: Format must contain '{version}' placeholder.",
            ));
        }

        if changelog.root_tag_format.is_empty() {
            return Err(ConfigError::validation(
                "changelog.root_tag_format: Tag format cannot be empty.",
            ));
        }

        if !changelog.root_tag_format.contains("{version}") {
            return Err(ConfigError::validation(
                "changelog.root_tag_format: Format must contain '{version}' placeholder.",
            ));
        }
    }

    Ok(())
}

/// Validates git configuration.
fn validate_git_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let git = &config.git;

    // Validate merge commit template
    if git.merge_commit_template.is_empty() {
        return Err(ConfigError::validation(
            "git.merge_commit_template: Template cannot be empty.",
        ));
    }

    // Validate monorepo merge commit template
    if git.monorepo_merge_commit_template.is_empty() {
        return Err(ConfigError::validation(
            "git.monorepo_merge_commit_template: Template cannot be empty.",
        ));
    }

    // Validate breaking warning template if enabled
    if git.include_breaking_warning && git.breaking_warning_template.is_empty() {
        return Err(ConfigError::validation(
            "git.breaking_warning_template: Template cannot be empty when include_breaking_warning is true.",
        ));
    }

    Ok(())
}

/// Validates audit configuration.
fn validate_audit_config(config: &PackageToolsConfig) -> ConfigResult<()> {
    let audit = &config.audit;

    if audit.enabled {
        // Validate severity
        let severity_str = audit.min_severity.to_lowercase();
        let valid_severities = ["critical", "warning", "info"];
        if !valid_severities.contains(&severity_str.as_str()) {
            return Err(ConfigError::validation(format!(
                "audit.min_severity: Invalid severity '{}'. Must be one of: {}",
                severity_str,
                valid_severities.join(", ")
            )));
        }

        // Validate that at least one section is enabled
        let sections = &audit.sections;
        if !sections.upgrades
            && !sections.dependencies
            && !sections.breaking_changes
            && !sections.categorization
            && !sections.version_consistency
        {
            return Err(ConfigError::validation(
                "audit.sections: At least one audit section must be enabled when audit is enabled.",
            ));
        }
    }

    Ok(())
}

/// Validates that a path is in a valid format.
///
/// # Arguments
///
/// * `path` - The path to validate
/// * `field_name` - Name of the configuration field for error messages
///
/// # Returns
///
/// `Ok(())` if the path is valid.
///
/// # Errors
///
/// Returns an error if the path format is invalid.
pub fn validate_path_format(path: &str, field_name: &str) -> ConfigResult<()> {
    if path.is_empty() {
        return Err(ConfigError::validation(format!("{}: Path cannot be empty", field_name)));
    }

    // Check for invalid characters (platform-specific)
    if cfg!(windows) {
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        for ch in invalid_chars {
            if path.contains(ch) {
                return Err(ConfigError::validation(format!(
                    "{}: Path contains invalid character '{}' on Windows",
                    field_name, ch
                )));
            }
        }
    }

    Ok(())
}

/// Validates that a URL is in a valid format.
///
/// # Arguments
///
/// * `url` - The URL to validate
/// * `field_name` - Name of the configuration field for error messages
///
/// # Returns
///
/// `Ok(())` if the URL is valid.
///
/// # Errors
///
/// Returns an error if the URL format is invalid.
pub fn validate_url_format(url: &str, field_name: &str) -> ConfigResult<()> {
    if url.is_empty() {
        return Err(ConfigError::validation(format!("{}: URL cannot be empty", field_name)));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ConfigError::validation(format!(
            "{}: URL must start with http:// or https://",
            field_name
        )));
    }

    Ok(())
}

/// Checks if a path exists on the filesystem.
///
/// This is an optional validation that can be performed when filesystem
/// access is available.
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path exists, `false` otherwise.
pub fn path_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
