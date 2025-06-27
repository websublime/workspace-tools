//! Configuration writer component
//!
//! Handles updating configuration values with validation and change tracking.
//! This component is responsible for all configuration modification operations.

use crate::config::{MonorepoConfig, ConfigPersistence};
use crate::error::Result;
use std::path::{Path, PathBuf};

/// Component responsible for writing and updating configuration
pub struct ConfigWriter {
    config: MonorepoConfig,
    config_path: Option<PathBuf>,
    auto_save: bool,
    persistence: ConfigPersistence,
}

impl ConfigWriter {
    /// Create a new config writer with the given configuration
    #[must_use]
    pub fn new(config: MonorepoConfig) -> Self {
        Self {
            config,
            config_path: None,
            auto_save: false,
            persistence: ConfigPersistence::new(),
        }
    }

    /// Create a config writer from file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// New config writer loaded from file
    ///
    /// # Errors
    /// Returns an error if the file cannot be loaded
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let persistence = ConfigPersistence::new();
        let config = persistence.load_from_file(path)?;
        
        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
            auto_save: false,
            persistence,
        })
    }

    /// Get immutable reference to the configuration
    #[must_use]
    pub fn config(&self) -> &MonorepoConfig {
        &self.config
    }

    /// Get the configuration file path
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Check if auto-save is enabled
    #[must_use]
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save
    }

    /// Update configuration using a function (functional style)
    ///
    /// # Arguments
    /// * `updater` - Function to update the configuration
    ///
    /// # Returns
    /// New config writer with updated configuration
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn with_update<F>(mut self, updater: F) -> Result<Self>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        updater(&mut self.config);
        if self.auto_save {
            self.save()?;
        }
        Ok(self)
    }

    /// Update configuration in place using a function
    ///
    /// # Arguments
    /// * `updater` - Function to update the configuration
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn update<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        updater(&mut self.config);
        if self.auto_save {
            self.save()?;
        }
        Ok(())
    }

    /// Set auto-save behavior
    ///
    /// # Arguments
    /// * `auto_save` - Whether to automatically save after updates
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Set auto-save behavior (functional style)
    ///
    /// # Arguments
    /// * `auto_save` - Whether to automatically save after updates
    ///
    /// # Returns
    /// Self with updated auto-save setting
    #[must_use]
    pub fn with_auto_save(mut self, auto_save: bool) -> Self {
        self.auto_save = auto_save;
        self
    }

    /// Set the configuration file path
    ///
    /// # Arguments
    /// * `path` - Path to set as the configuration file location
    pub fn set_config_path(&mut self, path: impl AsRef<Path>) {
        self.config_path = Some(path.as_ref().to_path_buf());
    }

    /// Set the configuration file path (functional style)
    ///
    /// # Arguments
    /// * `path` - Path to set as the configuration file location
    ///
    /// # Returns
    /// Self with updated config path
    #[must_use]
    pub fn with_config_path(mut self, path: impl AsRef<Path>) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Save configuration to the configured path
    ///
    /// # Errors
    /// Returns an error if no path is configured or saving fails
    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            self.persistence.save_to_file(&self.config, path)
        } else {
            Err(crate::error::Error::config("No config path set for saving"))
        }
    }

    /// Save configuration to a specific file
    ///
    /// # Arguments
    /// * `path` - Path where to save the configuration
    ///
    /// # Errors
    /// Returns an error if saving fails
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.persistence.save_to_file(&self.config, path)
    }

    /// Update specific configuration sections with validation
    
    /// Enable or disable hooks globally
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable hooks
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn set_hooks_enabled(&mut self, enabled: bool) -> Result<()> {
        self.update(|config| {
            config.hooks.enabled = enabled;
        })
    }

    /// Set changeset requirement
    ///
    /// # Arguments
    /// * `required` - Whether changesets are required
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn set_changesets_required(&mut self, required: bool) -> Result<()> {
        self.update(|config| {
            config.changesets.required = required;
        })
    }

    /// Enable or disable automatic tagging
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable automatic tagging
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn set_auto_tagging_enabled(&mut self, enabled: bool) -> Result<()> {
        self.update(|config| {
            config.versioning.auto_tag = enabled;
        })
    }

    /// Enable or disable breaking changes inclusion in changelogs
    ///
    /// # Arguments
    /// * `enabled` - Whether to include breaking changes
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn set_breaking_changes_included(&mut self, enabled: bool) -> Result<()> {
        self.update(|config| {
            config.changelog.include_breaking_changes = enabled;
        })
    }

    /// Add an environment to the configuration
    ///
    /// # Arguments
    /// * `environment` - Environment to add
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn add_environment(&mut self, environment: crate::Environment) -> Result<()> {
        self.update(|config| {
            if !config.environments.contains(&environment) {
                config.environments.push(environment);
            }
        })
    }

    /// Remove an environment from the configuration
    ///
    /// # Arguments
    /// * `environment` - Environment to remove
    ///
    /// # Returns
    /// True if the environment was removed, false if it wasn't found
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn remove_environment(&mut self, environment: &crate::Environment) -> Result<bool> {
        let mut removed = false;
        self.update(|config| {
            if let Some(pos) = config.environments.iter().position(|e| e == environment) {
                config.environments.remove(pos);
                removed = true;
            }
        })?;
        Ok(removed)
    }

    /// Reset configuration to defaults
    ///
    /// # Errors
    /// Returns an error if auto-save fails
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.update(|config| {
            *config = MonorepoConfig::default();
        })
    }

    /// Consume the writer and return the final configuration
    #[must_use]
    pub fn into_config(self) -> MonorepoConfig {
        self.config
    }

    /// Validate the current configuration
    ///
    /// # Returns
    /// List of validation errors, if any
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate workspace patterns
        for pattern in &self.config.workspace.patterns {
            if pattern.pattern.is_empty() {
                errors.push("Workspace pattern cannot be empty".to_string());
            }
        }

        // Validate environments
        if self.config.environments.is_empty() {
            errors.push("At least one environment must be configured".to_string());
        }

        errors
    }
}