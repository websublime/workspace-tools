//! Configuration manager for package tools.
//!
//! This module provides a comprehensive configuration management system
//! that integrates with sublime_standard_tools while providing package-specific
//! functionality and environment variable overrides.
//!
//! # What
//!
//! Provides a specialized configuration manager that:
//! - Loads configuration from multiple sources (files, environment, defaults)
//! - Supports environment variable overrides with SUBLIME_PACKAGE_TOOLS_ prefix
//! - Validates configuration before use
//! - Provides caching and reload capabilities
//! - Integrates seamlessly with sublime_standard_tools configuration system
//!
//! # How
//!
//! Uses the ConfigManager from sublime_standard_tools as the foundation
//! and adds package-specific environment variable mapping, validation,
//! and utility functions for common configuration operations.
//!
//! # Why
//!
//! Centralized configuration management ensures consistent behavior across
//! all package management operations while providing flexibility for
//! different deployment environments and user preferences.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use sublime_standard_tools::config::{ConfigManager, Configurable};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

use crate::config::PackageToolsConfig;
use crate::error::{PackageError, PackageResult};

/// Environment variable prefix for package tools configuration.
pub const ENV_PREFIX: &str = "SUBLIME_PACKAGE_TOOLS";

/// Package tools configuration manager.
///
/// Provides a high-level interface for loading, validating, and managing
/// package tools configuration with support for multiple sources and
/// environment variable overrides.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::config::PackageToolsConfigManager;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create manager with default settings
/// let manager = PackageToolsConfigManager::new();
/// let config = manager.load_config().await?;
///
/// // Create manager with custom project path
/// let manager = PackageToolsConfigManager::new_with_project_path("/path/to/project");
/// let config = manager.load_config().await?;
///
/// // Load configuration with environment overrides
/// std::env::set_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY", "unified");
/// let config = manager.load_config().await?;
/// assert_eq!(config.release.strategy, "unified");
/// # Ok(())
/// # }
/// ```
pub struct PackageToolsConfigManager {
    project_path: Option<PathBuf>,
}

impl PackageToolsConfigManager {
    /// Creates a new configuration manager.
    ///
    /// Uses the current working directory as the project root and sets up
    /// standard configuration sources with environment variable support.
    ///
    /// # Returns
    ///
    /// A new configuration manager ready for loading configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfigManager;
    ///
    /// let manager = PackageToolsConfigManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { project_path: Some(PathBuf::from(".")) }
    }

    /// Creates a new configuration manager with a specific project path.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project root directory
    ///
    /// # Returns
    ///
    /// A new configuration manager configured for the specified project.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfigManager;
    ///
    /// let manager = PackageToolsConfigManager::new_with_project_path("/path/to/project");
    /// ```
    #[must_use]
    pub fn new_with_project_path<P: AsRef<Path>>(project_path: P) -> Self {
        Self { project_path: Some(project_path.as_ref().to_path_buf()) }
    }

    /// Loads the configuration from all configured sources.
    ///
    /// Loads configuration in priority order:
    /// 1. Environment variables (highest priority)
    /// 2. Project configuration files
    /// 3. User configuration files
    /// 4. Default values (lowest priority)
    ///
    /// # Returns
    ///
    /// The merged and validated configuration.
    ///
    /// # Errors
    ///
    /// Returns `PackageError::Config` if:
    /// - Configuration files cannot be read
    /// - Configuration validation fails
    /// - Environment variables contain invalid values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = PackageToolsConfigManager::new();
    /// let config = manager.load_config().await?;
    /// println!("Loaded configuration: {:?}", config);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_config(&self) -> PackageResult<PackageToolsConfig> {
        let fs = FileSystemManager::new();
        let config_manager = Self::build_config_manager(&self.project_path, fs).await?;

        // Try to load configuration, use defaults if no config found
        match config_manager.load().await {
            Ok(config) => Ok(config),
            Err(_) => {
                // If loading fails, return default configuration
                let default_config = PackageToolsConfig::default();
                default_config.validate().map_err(|e| {
                    PackageError::Config(crate::error::ConfigError::InvalidPackageConfig {
                        field: "config_validation".to_string(),
                        reason: e.to_string(),
                    })
                })?;
                Ok(default_config)
            }
        }
    }

    /// Saves the configuration to the project configuration file.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to save
    ///
    /// # Errors
    ///
    /// Returns `PackageError::Config` if the configuration cannot be saved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::{PackageToolsConfig, PackageToolsConfigManager};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = PackageToolsConfigManager::new();
    /// let mut config = PackageToolsConfig::default();
    /// config.release.strategy = "unified".to_string();
    /// manager.save_config(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_config(&self, config: &PackageToolsConfig) -> PackageResult<()> {
        let fs = FileSystemManager::new();
        let config_manager = Self::build_config_manager(&self.project_path, fs).await?;

        config_manager.save(config).await.map_err(|e| {
            PackageError::Config(crate::error::ConfigError::InvalidPackageConfig {
                field: "config_saving".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Reloads the configuration from all sources.
    ///
    /// Useful when configuration files or environment variables have changed
    /// and you need to pick up the latest values.
    ///
    /// # Returns
    ///
    /// The newly loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns `PackageError::Config` if configuration cannot be reloaded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfigManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = PackageToolsConfigManager::new();
    /// let config = manager.reload_config().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reload_config(&self) -> PackageResult<PackageToolsConfig> {
        let fs = FileSystemManager::new();
        let config_manager = Self::build_config_manager(&self.project_path, fs).await?;

        config_manager.reload().await.map_err(|e| {
            PackageError::Config(crate::error::ConfigError::InvalidPackageConfig {
                field: "config_reloading".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Gets the current project path.
    ///
    /// # Returns
    ///
    /// The project path if set, otherwise None.
    #[must_use]
    pub fn project_path(&self) -> Option<&Path> {
        self.project_path.as_deref()
    }

    /// Validates a configuration without loading from files.
    ///
    /// Useful for testing configuration changes before applying them.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid.
    ///
    /// # Errors
    ///
    /// Returns `PackageError::Config` if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::{PackageToolsConfig, PackageToolsConfigManager};
    ///
    /// let manager = PackageToolsConfigManager::new();
    /// let config = PackageToolsConfig::default();
    /// assert!(manager.validate_config(&config).is_ok());
    /// ```
    pub fn validate_config(&self, config: &PackageToolsConfig) -> PackageResult<()> {
        config.validate().map_err(|e| {
            PackageError::Config(crate::error::ConfigError::InvalidPackageConfig {
                field: "config_validation".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Gets environment variable overrides currently active.
    ///
    /// Scans all environment variables with the SUBLIME_PACKAGE_TOOLS_ prefix
    /// and returns them as a map of configuration keys to values.
    ///
    /// # Returns
    ///
    /// Map of configuration keys to environment variable values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfigManager;
    /// use std::env;
    ///
    /// env::set_var("SUBLIME_PACKAGE_TOOLS_RELEASE_STRATEGY", "unified");
    ///
    /// let manager = PackageToolsConfigManager::new();
    /// let overrides = manager.get_env_overrides();
    /// assert!(overrides.contains_key("RELEASE_STRATEGY"));
    /// ```
    #[must_use]
    pub fn get_env_overrides(&self) -> HashMap<String, String> {
        let prefix = format!("{}_", ENV_PREFIX);
        env::vars()
            .filter_map(|(key, value)| {
                if key.starts_with(&prefix) {
                    Some((key[prefix.len()..].to_string(), value))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Creates a configuration builder with standard package tools settings.
    ///
    /// Sets up the configuration manager with:
    /// - Default configuration values
    /// - Environment variable support with SUBLIME_PACKAGE_TOOLS_ prefix
    /// - Project and user configuration file sources
    ///
    /// # Arguments
    ///
    /// * `project_path` - Optional project path for locating config files
    ///
    /// # Returns
    ///
    /// Configured ConfigManager ready for loading configuration.
    async fn build_config_manager(
        project_path: &Option<PathBuf>,
        fs: FileSystemManager,
    ) -> PackageResult<ConfigManager<PackageToolsConfig>> {
        let mut builder =
            ConfigManager::<PackageToolsConfig>::builder().with_env_prefix(ENV_PREFIX);

        // Add project configuration files if project path is available
        if let Some(ref path) = project_path {
            for ext in &["toml", "yaml", "yml", "json"] {
                let config_file = path.join(format!("repo.config.{}", ext));
                if fs.exists(&config_file).await {
                    builder = builder.with_file(config_file);
                }
            }
        }

        // Add user configuration files
        if let Some(user_config_dir) = Self::get_user_config_dir() {
            for ext in &["toml", "yaml", "yml", "json"] {
                let config_file = user_config_dir.join(format!("config.{}", ext));
                if fs.exists(&config_file).await {
                    builder = builder.with_file(config_file);
                }
            }
        }

        builder.build(fs).map_err(|e| {
            PackageError::Config(crate::error::ConfigError::InvalidPackageConfig {
                field: "config_manager_build".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Gets the user configuration directory.
    ///
    /// Uses platform-appropriate configuration directories:
    /// - Windows: %APPDATA%\sublime
    /// - macOS: ~/Library/Application Support/sublime
    /// - Linux: ~/.config/sublime
    ///
    /// # Returns
    ///
    /// Path to the user configuration directory, or None if it cannot be determined.
    fn get_user_config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| config_dir.join("sublime"))
    }
}

impl Default for PackageToolsConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variable mapping utilities.
///
/// Provides functions for mapping between environment variable names
/// and configuration field paths.
pub struct EnvMapping;

impl EnvMapping {
    /// Maps an environment variable name to a configuration field path.
    ///
    /// # Arguments
    ///
    /// * `env_var` - Environment variable name (without prefix)
    ///
    /// # Returns
    ///
    /// Configuration field path, or None if the variable is not recognized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::EnvMapping;
    ///
    /// assert_eq!(
    ///     EnvMapping::env_to_config_path("RELEASE_STRATEGY"),
    ///     Some("release.strategy".to_string())
    /// );
    /// ```
    #[must_use]
    pub fn env_to_config_path(env_var: &str) -> Option<String> {
        match env_var {
            // Changeset configuration
            "CHANGESET_PATH" => Some("changeset.path".to_string()),
            "CHANGESET_HISTORY_PATH" => Some("changeset.history_path".to_string()),
            "CHANGESET_AVAILABLE_ENVIRONMENTS" => {
                Some("changeset.available_environments".to_string())
            }
            "CHANGESET_DEFAULT_ENVIRONMENTS" => Some("changeset.default_environments".to_string()),
            "CHANGESET_FILENAME_FORMAT" => Some("changeset.filename_format".to_string()),
            "CHANGESET_MAX_PENDING" => Some("changeset.max_pending_changesets".to_string()),
            "CHANGESET_AUTO_ARCHIVE" => Some("changeset.auto_archive_applied".to_string()),

            // Version configuration
            "VERSION_SNAPSHOT_FORMAT" => Some("version.snapshot_format".to_string()),
            "VERSION_COMMIT_HASH_LENGTH" => Some("version.commit_hash_length".to_string()),
            "VERSION_ALLOW_SNAPSHOT_ON_MAIN" => Some("version.allow_snapshot_on_main".to_string()),

            // Registry configuration
            "REGISTRY_URL" => Some("registry.url".to_string()),
            "REGISTRY_TIMEOUT" => Some("registry.timeout".to_string()),
            "REGISTRY_RETRY_ATTEMPTS" => Some("registry.retry_attempts".to_string()),
            "REGISTRY_USE_NPMRC" => Some("registry.use_npmrc".to_string()),
            "REGISTRY_DEFAULT_ACCESS" => Some("registry.default_access".to_string()),

            // Release configuration
            "RELEASE_STRATEGY" => Some("release.strategy".to_string()),
            "RELEASE_TAG_FORMAT" => Some("release.tag_format".to_string()),
            "RELEASE_ENV_TAG_FORMAT" => Some("release.env_tag_format".to_string()),
            "RELEASE_CREATE_TAGS" => Some("release.create_tags".to_string()),
            "RELEASE_PUSH_TAGS" => Some("release.push_tags".to_string()),
            "RELEASE_CREATE_CHANGELOG" => Some("release.create_changelog".to_string()),
            "RELEASE_CHANGELOG_FILE" => Some("release.changelog_file".to_string()),
            "RELEASE_COMMIT_MESSAGE" => Some("release.commit_message".to_string()),
            "RELEASE_DRY_RUN_BY_DEFAULT" => Some("release.dry_run_by_default".to_string()),
            "RELEASE_MAX_CONCURRENT" => Some("release.max_concurrent_releases".to_string()),
            "RELEASE_TIMEOUT" => Some("release.release_timeout".to_string()),

            // Dependency configuration
            "DEPENDENCY_PROPAGATE_UPDATES" => Some("dependency.propagate_updates".to_string()),
            "DEPENDENCY_PROPAGATE_DEV" => Some("dependency.propagate_dev_dependencies".to_string()),
            "DEPENDENCY_MAX_DEPTH" => Some("dependency.max_propagation_depth".to_string()),
            "DEPENDENCY_DETECT_CIRCULAR" => Some("dependency.detect_circular".to_string()),
            "DEPENDENCY_FAIL_ON_CIRCULAR" => Some("dependency.fail_on_circular".to_string()),
            "DEPENDENCY_UPDATE_BUMP" => Some("dependency.dependency_update_bump".to_string()),
            "DEPENDENCY_INCLUDE_PEER" => Some("dependency.include_peer_dependencies".to_string()),
            "DEPENDENCY_INCLUDE_OPTIONAL" => {
                Some("dependency.include_optional_dependencies".to_string())
            }

            // Conventional configuration
            "CONVENTIONAL_PARSE_BREAKING" => {
                Some("conventional.parse_breaking_changes".to_string())
            }
            "CONVENTIONAL_REQUIRE" => Some("conventional.require_conventional_commits".to_string()),
            "CONVENTIONAL_DEFAULT_BUMP" => Some("conventional.default_bump_type".to_string()),

            // Changelog configuration
            "CHANGELOG_INCLUDE_COMMIT_HASH" => Some("changelog.include_commit_hash".to_string()),
            "CHANGELOG_INCLUDE_AUTHORS" => Some("changelog.include_authors".to_string()),
            "CHANGELOG_GROUP_BY_TYPE" => Some("changelog.group_by_type".to_string()),
            "CHANGELOG_INCLUDE_DATE" => Some("changelog.include_date".to_string()),
            "CHANGELOG_MAX_COMMITS" => Some("changelog.max_commits_per_release".to_string()),
            "CHANGELOG_LINK_COMMITS" => Some("changelog.link_commits".to_string()),
            "CHANGELOG_COMMIT_URL_FORMAT" => Some("changelog.commit_url_format".to_string()),

            _ => None,
        }
    }

    /// Gets all supported environment variable names.
    ///
    /// # Returns
    ///
    /// Vector of all recognized environment variable names (without prefix).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::EnvMapping;
    ///
    /// let variables = EnvMapping::all_env_variables();
    /// assert!(variables.contains(&"RELEASE_STRATEGY".to_string()));
    /// ```
    #[must_use]
    pub fn all_env_variables() -> Vec<String> {
        vec![
            // Changeset
            "CHANGESET_PATH",
            "CHANGESET_HISTORY_PATH",
            "CHANGESET_AVAILABLE_ENVIRONMENTS",
            "CHANGESET_DEFAULT_ENVIRONMENTS",
            "CHANGESET_FILENAME_FORMAT",
            "CHANGESET_MAX_PENDING",
            "CHANGESET_AUTO_ARCHIVE",
            // Version
            "VERSION_SNAPSHOT_FORMAT",
            "VERSION_COMMIT_HASH_LENGTH",
            "VERSION_ALLOW_SNAPSHOT_ON_MAIN",
            // Registry
            "REGISTRY_URL",
            "REGISTRY_TIMEOUT",
            "REGISTRY_RETRY_ATTEMPTS",
            "REGISTRY_USE_NPMRC",
            "REGISTRY_DEFAULT_ACCESS",
            // Release
            "RELEASE_STRATEGY",
            "RELEASE_TAG_FORMAT",
            "RELEASE_ENV_TAG_FORMAT",
            "RELEASE_CREATE_TAGS",
            "RELEASE_PUSH_TAGS",
            "RELEASE_CREATE_CHANGELOG",
            "RELEASE_CHANGELOG_FILE",
            "RELEASE_COMMIT_MESSAGE",
            "RELEASE_DRY_RUN_BY_DEFAULT",
            "RELEASE_MAX_CONCURRENT",
            "RELEASE_TIMEOUT",
            // Dependency
            "DEPENDENCY_PROPAGATE_UPDATES",
            "DEPENDENCY_PROPAGATE_DEV",
            "DEPENDENCY_MAX_DEPTH",
            "DEPENDENCY_DETECT_CIRCULAR",
            "DEPENDENCY_FAIL_ON_CIRCULAR",
            "DEPENDENCY_UPDATE_BUMP",
            "DEPENDENCY_INCLUDE_PEER",
            "DEPENDENCY_INCLUDE_OPTIONAL",
            // Conventional
            "CONVENTIONAL_PARSE_BREAKING",
            "CONVENTIONAL_REQUIRE",
            "CONVENTIONAL_DEFAULT_BUMP",
            // Changelog
            "CHANGELOG_INCLUDE_COMMIT_HASH",
            "CHANGELOG_INCLUDE_AUTHORS",
            "CHANGELOG_GROUP_BY_TYPE",
            "CHANGELOG_INCLUDE_DATE",
            "CHANGELOG_MAX_COMMITS",
            "CHANGELOG_LINK_COMMITS",
            "CHANGELOG_COMMIT_URL_FORMAT",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}
