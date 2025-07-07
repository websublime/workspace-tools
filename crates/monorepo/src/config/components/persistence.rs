//! Configuration persistence component
//!
//! Handles loading and saving configuration files in various formats (JSON, TOML, YAML).
//! This component is responsible for all file I/O operations related to configuration.

use crate::config::MonorepoConfig;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};

/// Component responsible for configuration file persistence
pub struct ConfigPersistence {
    fs: FileSystemManager,
}

impl ConfigPersistence {
    /// Create a new config persistence component
    #[must_use]
    pub fn new() -> Self {
        Self { fs: FileSystemManager::new() }
    }

    /// Load configuration from a file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// Loaded configuration
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn load_from_file(&self, path: impl AsRef<Path>) -> Result<MonorepoConfig> {
        let path = path.as_ref();
        log::info!("Loading configuration from file: {}", path.display());

        let content = self.fs.read_file_string(path).map_err(|e| {
            log::error!("Failed to read config file '{}': {}", path.display(), e);
            Error::config(format!("Failed to read config file: {e}"))
        })?;

        log::debug!("Read {} bytes from config file", content.len());

        let format = path.extension().and_then(|s| s.to_str()).unwrap_or("unknown");
        log::debug!("Parsing configuration file as format: {}", format);

        let config: MonorepoConfig = match format {
            "json" => serde_json::from_str(&content).map_err(|e| {
                log::error!("Failed to parse JSON config: {}", e);
                Error::config(format!("Failed to parse JSON: {e}"))
            })?,
            "toml" => toml::from_str(&content).map_err(|e| {
                log::error!("Failed to parse TOML config: {}", e);
                Error::config(format!("Failed to parse TOML: {e}"))
            })?,
            "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| {
                log::error!("Failed to parse YAML config: {}", e);
                Error::config(format!("Failed to parse YAML: {e}"))
            })?,
            _ => {
                log::error!("Unsupported config file format: {}", format);
                return Err(Error::config("Unsupported config file format"));
            }
        };

        log::info!("Successfully loaded configuration from {}", path.display());
        Ok(config)
    }

    /// Save configuration to a file
    ///
    /// # Arguments
    /// * `config` - Configuration to save
    /// * `path` - Path where to save the configuration file
    ///
    /// # Errors
    /// Returns an error if the file cannot be written or serialized
    pub fn save_to_file(&self, config: &MonorepoConfig, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        log::info!("Saving configuration to file: {}", path.display());

        let format = path.extension().and_then(|s| s.to_str()).unwrap_or("unknown");
        log::debug!("Serializing configuration as format: {}", format);

        let content = match format {
            "json" => serde_json::to_string_pretty(config).map_err(|e| {
                log::error!("Failed to serialize config to JSON: {}", e);
                Error::config(format!("Failed to serialize to JSON: {e}"))
            })?,
            "toml" => toml::to_string_pretty(config).map_err(|e| {
                log::error!("Failed to serialize config to TOML: {}", e);
                Error::config(format!("Failed to serialize to TOML: {e}"))
            })?,
            "yaml" | "yml" => serde_yaml::to_string(config).map_err(|e| {
                log::error!("Failed to serialize config to YAML: {}", e);
                Error::config(format!("Failed to serialize to YAML: {e}"))
            })?,
            _ => {
                log::error!("Unsupported config file format: {}", format);
                return Err(Error::config("Unsupported config file format"));
            }
        };

        log::debug!("Serialized configuration to {} bytes", content.len());

        self.fs.write_file_string(path, &content).map_err(|e| {
            log::error!("Failed to write config file '{}': {}", path.display(), e);
            Error::config(format!("Failed to write config file: {e}"))
        })?;

        log::info!("Successfully saved configuration to {}", path.display());
        Ok(())
    }

    /// Create default configuration files in a directory
    ///
    /// # Arguments
    /// * `dir` - Directory where to create the configuration files
    ///
    /// # Errors
    /// Returns an error if the files cannot be created
    pub fn create_default_config_files(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        log::info!("Creating default configuration files in: {}", dir.display());

        let default_config = MonorepoConfig::default();

        // Create main config file
        let config_path = dir.join("monorepo.toml");
        self.save_to_file(&default_config, &config_path)?;

        log::info!("Created default configuration files");
        Ok(())
    }

    /// Find configuration file by searching up the directory tree
    ///
    /// # Arguments
    /// * `start_dir` - Directory to start searching from
    ///
    /// # Returns
    /// Path to the configuration file if found
    pub fn find_config_file(&self, start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let start_dir = start_dir.as_ref();
        log::debug!("Searching for config file starting from: {}", start_dir.display());

        let config_names = [
            "monorepo.toml",
            "monorepo.json",
            "monorepo.yaml",
            "monorepo.yml",
            ".monorepo.toml",
            ".monorepo.json",
            ".monorepo.yaml",
            ".monorepo.yml",
        ];

        let mut current_dir = start_dir;

        loop {
            for config_name in &config_names {
                let config_path = current_dir.join(config_name);
                if self.fs.exists(&config_path) {
                    log::debug!("Found config file: {}", config_path.display());
                    return Some(config_path);
                }
            }

            // Move up one directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent;
            } else {
                break;
            }
        }

        log::debug!("No config file found");
        None
    }
}

impl Default for ConfigPersistence {
    fn default() -> Self {
        Self::new()
    }
}
