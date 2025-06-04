//! Configuration manager for monorepo tools

use crate::config::{
    ChangelogConfig, ChangesetsConfig, HooksConfig, PluginsConfig, TasksConfig, VersioningConfig,
};
use crate::error::{Error, Result};
use crate::{Environment, MonorepoConfig};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Configuration manager that handles loading, saving, and managing monorepo configurations
pub struct ConfigManager {
    /// The current configuration
    config: Arc<RwLock<MonorepoConfig>>,

    /// Path to the configuration file
    config_path: Option<PathBuf>,

    /// Whether to auto-save on changes
    auto_save: bool,
}

impl ConfigManager {
    /// Create a new configuration manager with default config
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(MonorepoConfig::default())),
            config_path: None,
            auto_save: false,
        }
    }

    /// Create a configuration manager with a specific config
    pub fn with_config(config: MonorepoConfig) -> Self {
        Self { config: Arc::new(RwLock::new(config)), config_path: None, auto_save: false }
    }

    /// Load configuration from a file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::config(format!("Failed to read config file: {e}")))?;

        let config: MonorepoConfig = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)
                .map_err(|e| Error::config(format!("Failed to parse TOML: {e}")))?,
            Some("yaml" | "yml") => serde_yaml::from_str(&content)
                .map_err(|e| Error::config(format!("Failed to parse YAML: {e}")))?,
            _ => return Err(Error::config("Unsupported config file format")),
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Some(path.to_path_buf()),
            auto_save: false,
        })
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let config =
            self.config.read().map_err(|_| Error::config("Failed to acquire read lock"))?;

        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::to_string_pretty(&*config)?,
            Some("toml") => toml::to_string_pretty(&*config)
                .map_err(|e| Error::config(format!("Failed to serialize to TOML: {e}")))?,
            Some("yaml" | "yml") => serde_yaml::to_string(&*config)
                .map_err(|e| Error::config(format!("Failed to serialize to YAML: {e}")))?,
            _ => return Err(Error::config("Unsupported config file format")),
        };

        std::fs::write(path, content)
            .map_err(|e| Error::config(format!("Failed to write config file: {e}")))?;

        Ok(())
    }

    /// Save configuration to the loaded path
    pub fn save(&self) -> Result<()> {
        match &self.config_path {
            Some(path) => self.save_to_file(path),
            None => Err(Error::config("No config file path set")),
        }
    }

    /// Get the current configuration
    pub fn get(&self) -> Result<MonorepoConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.clone())
    }

    /// Update the configuration
    pub fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        let mut config =
            self.config.write().map_err(|_| Error::config("Failed to acquire write lock"))?;

        updater(&mut config);

        drop(config); // Explicitly drop the lock

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Get a specific configuration section
    pub fn get_versioning(&self) -> Result<VersioningConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.versioning.clone())
    }

    /// Get tasks configuration
    pub fn get_tasks(&self) -> Result<TasksConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.tasks.clone())
    }

    /// Get changelog configuration
    pub fn get_changelog(&self) -> Result<ChangelogConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.changelog.clone())
    }

    /// Get hooks configuration
    pub fn get_hooks(&self) -> Result<HooksConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.hooks.clone())
    }

    /// Get changesets configuration
    pub fn get_changesets(&self) -> Result<ChangesetsConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.changesets.clone())
    }

    /// Get plugins configuration
    pub fn get_plugins(&self) -> Result<PluginsConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.plugins.clone())
    }

    /// Get environments
    pub fn get_environments(&self) -> Result<Vec<Environment>> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.environments.clone())
    }

    /// Set auto-save behavior
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Get the config file path
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Create default configuration files in a directory
    pub fn create_default_config_files(dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        let config = MonorepoConfig::default();

        // Create .monorepo directory
        let config_dir = dir.join(".monorepo");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| Error::config(format!("Failed to create config directory: {e}")))?;

        // Save as JSON
        let json_path = config_dir.join("config.json");
        let json_content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&json_path, json_content)
            .map_err(|e| Error::config(format!("Failed to write JSON config: {e}")))?;

        // Save as TOML (alternative)
        let toml_path = config_dir.join("config.toml");
        let toml_content = toml::to_string_pretty(&config)
            .map_err(|e| Error::config(format!("Failed to serialize to TOML: {e}")))?;
        std::fs::write(&toml_path, toml_content)
            .map_err(|e| Error::config(format!("Failed to write TOML config: {e}")))?;

        Ok(())
    }

    /// Look for configuration file in standard locations
    pub fn find_config_file(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let start_dir = start_dir.as_ref();

        // Check for config files in order of preference
        let config_names = [
            ".monorepo/config.json",
            ".monorepo/config.toml",
            ".monorepo/config.yaml",
            ".monorepo/config.yml",
            "monorepo.config.json",
            "monorepo.config.toml",
            "monorepo.config.yaml",
            "monorepo.config.yml",
        ];

        // Check current directory and parent directories
        let mut current = start_dir.to_path_buf();
        loop {
            for config_name in &config_names {
                let config_path = current.join(config_name);
                if config_path.exists() {
                    return Some(config_path);
                }
            }

            if !current.pop() {
                break;
            }
        }

        None
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::VersionBumpType;

    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_default_config() {
        let manager = ConfigManager::new();
        let config = manager.get().unwrap();

        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        assert!(config.versioning.propagate_changes);
        assert!(config.hooks.enabled);
        assert_eq!(config.environments.len(), 3);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_update_config() {
        let manager = ConfigManager::new();

        manager
            .update(|config| {
                config.versioning.default_bump = VersionBumpType::Minor;
                config.versioning.auto_tag = false;
            })
            .unwrap();

        let config = manager.get().unwrap();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Minor);
        assert!(!config.versioning.auto_tag);
    }
}
