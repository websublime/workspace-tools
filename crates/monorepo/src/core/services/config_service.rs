//! Configuration service implementation
//!
//! Handles all configuration-related operations for the monorepo including
//! loading, validating, and providing access to configuration settings.

use super::FileSystemService;
use crate::config::{ConfigManager, MonorepoConfig};
use crate::error::Result;
use std::path::Path;

/// Configuration management service
///
/// Provides centralized configuration management for the monorepo including
/// loading configuration from files, environment variables, and defaults.
/// Handles validation and ensures configuration consistency.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::{ConfigurationService, FileSystemService};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs_service = FileSystemService::new("/path/to/monorepo")?;
/// let config_service = ConfigurationService::new("/path/to/monorepo", &fs_service)?;
///
/// let config = config_service.get_configuration();
/// println!("Workspace type: {:?}", config.workspace.workspace_type);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct ConfigurationService {
    /// Configuration manager for loading and validation
    #[allow(dead_code)]
    config_manager: ConfigManager,

    /// Current loaded configuration
    config: MonorepoConfig,

    /// Root path of the monorepo
    root_path: std::path::PathBuf,
}

#[allow(dead_code)]
impl ConfigurationService {
    /// Create a new configuration service
    ///
    /// Loads and validates the monorepo configuration from the specified
    /// root path using the provided file system service.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the monorepo
    /// * `file_system_service` - File system service for configuration loading
    ///
    /// # Returns
    ///
    /// A new configuration service with loaded configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration files cannot be read
    /// - Configuration is invalid or malformed
    /// - Required configuration sections are missing
    pub fn new<P: AsRef<Path>>(
        root_path: P,
        _file_system_service: &FileSystemService,
    ) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Create configuration manager and load configuration
        let config_manager = ConfigManager::new();
        let config = config_manager.load_config(&root_path)?;

        // Validate configuration
        config_manager.validate_config(&config)?;

        Ok(Self { config_manager, config, root_path })
    }

    /// Get the current configuration
    ///
    /// Returns a reference to the loaded and validated configuration.
    /// The configuration is guaranteed to be valid and complete.
    ///
    /// # Returns
    ///
    /// Reference to the current monorepo configuration.
    pub fn get_configuration(&self) -> &MonorepoConfig {
        &self.config
    }

    /// Reload configuration from disk
    ///
    /// Reloads the configuration from the file system, useful when
    /// configuration files have been modified externally.
    ///
    /// # Returns
    ///
    /// Success if configuration was reloaded and validated successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the new configuration is invalid or cannot be loaded.
    pub fn reload_configuration(&mut self) -> Result<()> {
        let new_config = self.config_manager.load_config(&self.root_path)?;
        self.config_manager.validate_config(&new_config)?;
        self.config = new_config;
        Ok(())
    }

    /// Validate a configuration
    ///
    /// Validates a configuration object without loading it as the active
    /// configuration. Useful for testing configuration changes.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to validate
    ///
    /// # Returns
    ///
    /// Success if the configuration is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid with details
    /// about what validation rules failed.
    pub fn validate_configuration(&self, config: &MonorepoConfig) -> Result<()> {
        self.config_manager.validate_config(config)
    }

    /// Get the root path
    ///
    /// Returns the root path of the monorepo that this configuration
    /// service is managing.
    ///
    /// # Returns
    ///
    /// Reference to the root path.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Check if configuration file exists
    ///
    /// Checks whether a monorepo configuration file exists at the expected
    /// location within the root path.
    ///
    /// # Returns
    ///
    /// True if configuration file exists, false otherwise.
    pub fn has_configuration_file(&self) -> bool {
        let config_path = self.root_path.join("monorepo.toml");
        config_path.exists()
    }

    /// Get configuration file path
    ///
    /// Returns the expected path to the monorepo configuration file.
    ///
    /// # Returns
    ///
    /// Path to the configuration file.
    pub fn config_file_path(&self) -> std::path::PathBuf {
        self.root_path.join("monorepo.toml")
    }
}
