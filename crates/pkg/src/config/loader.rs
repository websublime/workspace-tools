//! Configuration loading utilities for package tools.
//!
//! **What**: Provides utilities for loading configuration from files and environment variables
//! using the `sublime_standard_tools` ConfigManager.
//!
//! **How**: This module provides a `ConfigLoader` that integrates with ConfigManager to load
//! configuration from multiple sources (TOML, YAML, JSON files and environment variables),
//! merge them according to priority, and validate the result.
//!
//! **Why**: To provide a simple, consistent API for loading package tools configuration
//! across different environments and use cases, with proper error handling and validation.

use std::path::Path;

use sublime_standard_tools::config::{ConfigManager, ConfigResult, Configurable};
use sublime_standard_tools::filesystem::{AsyncFileSystem, FileSystemManager};

use super::PackageToolsConfig;

/// Configuration loader for package tools.
///
/// This structure provides utilities for loading configuration from various sources
/// and integrating with the `sublime_standard_tools` ConfigManager.
///
/// # Features
///
/// - Load from TOML, YAML, or JSON files (auto-detected by extension)
/// - Environment variable overrides with `SUBLIME_PKG_` prefix
/// - Multiple configuration file support with priority ordering
/// - Validation of loaded configuration
/// - Sensible defaults
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::load_config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load with defaults and optional files
/// let config = load_config().await?;
///
/// println!("Loaded configuration: {:?}", config);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Loads configuration with default values only.
    ///
    /// This initializes configuration with only the default values,
    /// without loading from files or environment variables.
    ///
    /// # Returns
    ///
    /// The default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails (which should not happen for defaults).
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ConfigLoader;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::load_defaults().await?;
    ///
    /// // Config has all default values
    /// assert_eq!(config.changeset.path, ".changesets");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_defaults() -> ConfigResult<PackageToolsConfig> {
        let fs = FileSystemManager::new();
        let manager = ConfigManager::<PackageToolsConfig>::builder().with_defaults().build(fs)?;

        let config = manager.load().await?;
        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from a specific file with defaults and environment variables.
    ///
    /// This function loads configuration with default values, the specified file,
    /// and environment variable overrides with "SUBLIME_PKG" prefix.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// The loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be parsed
    /// - The configuration is invalid
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sublime_pkg_tools::config::ConfigLoader;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::load_from_file("custom-config.toml").await?;
    /// println!("Config loaded: {:?}", config);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_from_file(path: impl AsRef<Path>) -> ConfigResult<PackageToolsConfig> {
        let fs = FileSystemManager::new();
        let manager = ConfigManager::<PackageToolsConfig>::builder()
            .with_defaults()
            .with_file(path)
            .with_env_prefix("SUBLIME_PKG")
            .build(fs)?;

        let config = manager.load().await?;
        config.validate()?;
        Ok(config)
    }

    /// Loads configuration from multiple files with defaults and environment variables.
    ///
    /// Files are loaded in order, with later files taking precedence.
    /// Files that don't exist are silently skipped.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths to configuration files
    ///
    /// # Returns
    ///
    /// The loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A file exists but cannot be parsed
    /// - The merged configuration is invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ConfigLoader;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigLoader::load_from_files(vec![
    ///     "package-tools.toml",
    ///     ".sublime/package-tools.toml",
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_from_files<P: AsRef<Path>>(
        paths: Vec<P>,
    ) -> ConfigResult<PackageToolsConfig> {
        let fs = FileSystemManager::new();

        // Start with defaults
        let mut builder = ConfigManager::<PackageToolsConfig>::builder().with_defaults();

        // Add files that exist
        for path in paths {
            if fs.exists(path.as_ref()).await {
                builder = builder.with_file(path);
            }
        }

        // Add environment variables
        builder = builder.with_env_prefix("SUBLIME_PKG");

        let manager = builder.build(fs)?;
        let config = manager.load().await?;
        config.validate()?;
        Ok(config)
    }
}

/// Convenience function to load configuration with defaults.
///
/// This function loads configuration with default values, optional files from
/// current directory, and environment variable overrides with "SUBLIME_PKG" prefix.
///
/// Looks for configuration in the following locations (in order):
/// 1. `package-tools.toml` in current directory
/// 2. `.sublime/package-tools.toml`
/// 3. Environment variables with `SUBLIME_PKG_` prefix
///
/// Files that don't exist are silently skipped.
///
/// # Returns
///
/// The loaded configuration.
///
/// # Errors
///
/// Returns an error if a configuration file exists but cannot be parsed,
/// or if the configuration is invalid.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::load_config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = load_config().await?;
/// println!("Config loaded: {:?}", config);
/// # Ok(())
/// # }
/// ```
pub async fn load_config() -> ConfigResult<PackageToolsConfig> {
    ConfigLoader::load_from_files(vec!["package-tools.toml", ".sublime/package-tools.toml"]).await
}

/// Convenience function to load configuration from a specific file.
///
/// This function loads configuration with default values, the specified file,
/// and environment variable overrides with "SUBLIME_PKG" prefix.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// The loaded configuration.
///
/// # Errors
///
/// Returns an error if the file does not exist, cannot be parsed, or the
/// configuration is invalid.
///
/// # Example
///
/// ```rust,no_run
/// use sublime_pkg_tools::config::load_config_from_file;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = load_config_from_file("custom-config.toml").await?;
/// println!("Config loaded: {:?}", config);
/// # Ok(())
/// # }
/// ```
pub async fn load_config_from_file(path: impl AsRef<Path>) -> ConfigResult<PackageToolsConfig> {
    ConfigLoader::load_from_file(path).await
}
