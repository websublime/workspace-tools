//! Upgrade configuration for dependency upgrade detection and application.
//!
//! **What**: Defines configuration for external dependency upgrades, including registry
//! settings, authentication, backup behavior, and automatic changeset creation.
//!
//! **How**: This module provides the `UpgradeConfig` structure that controls how dependency
//! upgrades are detected from registries and applied to package.json files.
//!
//! **Why**: To enable controlled, safe dependency upgrades with proper authentication,
//! retry logic, and automatic backup/rollback capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for dependency upgrade operations.
///
/// This structure controls how external dependency upgrades are detected and applied,
/// including registry communication, automatic changeset creation, and backup behavior.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::UpgradeConfig;
///
/// let config = UpgradeConfig::default();
/// assert!(config.auto_changeset);
/// assert_eq!(config.changeset_bump, "patch");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpgradeConfig {
    /// Registry configuration for package lookups.
    pub registry: RegistryConfig,

    /// Whether to automatically create a changeset for upgrades.
    ///
    /// When enabled, applying upgrades will automatically create or update
    /// a changeset with the upgrade information.
    ///
    /// # Default: `true`
    pub auto_changeset: bool,

    /// Version bump type to use for automatic changeset creation.
    ///
    /// Valid values: "major", "minor", "patch", "none"
    ///
    /// # Default: `"patch"`
    pub changeset_bump: String,

    /// Backup configuration for upgrade operations.
    pub backup: BackupConfig,
}

/// Configuration for NPM registry communication.
///
/// Controls how the system communicates with NPM registries to fetch
/// package metadata and detect available upgrades.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::RegistryConfig;
///
/// let config = RegistryConfig::default();
/// assert_eq!(config.default_registry, "https://registry.npmjs.org");
/// assert_eq!(config.timeout_secs, 30);
/// assert_eq!(config.retry_attempts, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryConfig {
    /// Default registry URL for package lookups.
    ///
    /// # Default: `"https://registry.npmjs.org"`
    pub default_registry: String,

    /// Scoped registry mappings.
    ///
    /// Maps scope names (without @) to registry URLs.
    /// Example: "myorg" -> "<https://npm.myorg.com>"
    ///
    /// # Default: empty
    pub scoped_registries: HashMap<String, String>,

    /// Authentication tokens for registries.
    ///
    /// Maps registry URLs to authentication tokens.
    ///
    /// # Default: empty
    pub auth_tokens: HashMap<String, String>,

    /// HTTP request timeout in seconds.
    ///
    /// # Default: `30`
    pub timeout_secs: u64,

    /// Number of retry attempts for failed requests.
    ///
    /// # Default: `3`
    pub retry_attempts: usize,

    /// Delay in milliseconds between retry attempts.
    ///
    /// # Default: `1000` (1 second)
    pub retry_delay_ms: u64,

    /// Whether to read configuration from .npmrc files.
    ///
    /// When enabled, will merge settings from .npmrc files in the workspace.
    ///
    /// # Default: `true`
    pub read_npmrc: bool,
}

/// Configuration for backup and rollback operations.
///
/// Controls how package.json files are backed up before applying upgrades
/// and how those backups are managed.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::BackupConfig;
///
/// let config = BackupConfig::default();
/// assert!(config.enabled);
/// assert_eq!(config.backup_dir, ".wnt-backups");
/// assert_eq!(config.max_backups, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupConfig {
    /// Whether automatic backups are enabled.
    ///
    /// When enabled, package.json files are backed up before applying upgrades.
    ///
    /// # Default: `true`
    pub enabled: bool,

    /// Directory where backups are stored.
    ///
    /// # Default: `".wnt-backups"`
    pub backup_dir: String,

    /// Whether to keep backups after successful operations.
    ///
    /// When false, backups are deleted after successful upgrade application.
    ///
    /// # Default: `false`
    pub keep_after_success: bool,

    /// Maximum number of backups to retain.
    ///
    /// Older backups are deleted when this limit is exceeded.
    ///
    /// # Default: `5`
    pub max_backups: usize,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            auto_changeset: true,
            changeset_bump: "patch".to_string(),
            backup: BackupConfig::default(),
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_registry: "https://registry.npmjs.org".to_string(),
            scoped_registries: HashMap::new(),
            auth_tokens: HashMap::new(),
            timeout_secs: 30,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            read_npmrc: true,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: ".wnt-backups".to_string(),
            keep_after_success: false,
            max_backups: 5,
        }
    }
}

impl Configurable for UpgradeConfig {
    fn validate(&self) -> ConfigResult<()> {
        // Validate changeset_bump
        match self.changeset_bump.as_str() {
            "major" | "minor" | "patch" | "none" => {}
            _ => {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "upgrade.changeset_bump: Invalid bump type '{}'. Must be one of: major, minor, patch, none",
                        self.changeset_bump
                    ),
                });
            }
        }

        self.registry.validate()?;
        self.backup.validate()?;
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.registry.merge_with(other.registry)?;
        self.auto_changeset = other.auto_changeset;
        self.changeset_bump = other.changeset_bump;
        self.backup.merge_with(other.backup)?;
        Ok(())
    }
}

impl Configurable for RegistryConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.default_registry.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "upgrade.registry.default_registry: Default registry URL cannot be empty"
                    .to_string(),
            });
        }

        if self.timeout_secs == 0 {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "upgrade.registry.timeout_secs: Timeout must be greater than 0"
                    .to_string(),
            });
        }

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.default_registry = other.default_registry;
        self.scoped_registries = other.scoped_registries;
        self.auth_tokens = other.auth_tokens;
        self.timeout_secs = other.timeout_secs;
        self.retry_attempts = other.retry_attempts;
        self.retry_delay_ms = other.retry_delay_ms;
        self.read_npmrc = other.read_npmrc;
        Ok(())
    }
}

impl Configurable for BackupConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.backup_dir.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "upgrade.backup.backup_dir: Backup directory cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enabled = other.enabled;
        self.backup_dir = other.backup_dir;
        self.keep_after_success = other.keep_after_success;
        self.max_backups = other.max_backups;
        Ok(())
    }
}
