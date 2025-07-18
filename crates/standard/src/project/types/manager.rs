//! # Configuration Manager
//!
//! ## What
//! This module provides the `ConfigManager` struct for managing configuration
//! settings across different scopes and file formats.
//!
//! ## How
//! The `ConfigManager` handles loading, saving, and manipulating configuration
//! settings with support for multiple scopes and file formats.
//!
//! ## Why
//! Centralized configuration management enables consistent settings handling
//! across the entire application with proper scope isolation.

use crate::project::types::{ConfigScope, ConfigValue};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Manages configuration across different scopes and file formats.
///
/// This struct provides functionality to load, save, and manipulate configuration
/// settings. It supports multiple scopes (global, user, project, runtime) and
/// different file formats (JSON, TOML, YAML).
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::project::{ConfigManager, ConfigScope, ConfigValue};
/// use std::path::PathBuf;
///
/// // Create a new configuration manager
/// let mut config_manager = ConfigManager::new();
///
/// // Set paths for different configuration scopes
/// config_manager.set_path(ConfigScope::User, PathBuf::from("~/.config/myapp.json"));
/// config_manager.set_path(ConfigScope::Project, PathBuf::from("./project-config.json"));
///
/// // Set a configuration value
/// config_manager.set("theme", ConfigValue::String("dark".to_string()));
///
/// // Get a configuration value
/// if let Some(theme) = config_manager.get("theme") {
///     if let Some(theme_str) = theme.as_string() {
///         println!("Current theme: {}", theme_str);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Configuration settings
    pub(crate) settings: Arc<RwLock<HashMap<String, ConfigValue>>>,
    /// Paths for different configuration scopes
    pub(crate) files: HashMap<ConfigScope, PathBuf>,
}

impl ConfigManager {
    /// Creates a new ConfigManager with empty settings.
    ///
    /// # Returns
    ///
    /// A new `ConfigManager` instance with no configuration loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::ConfigManager;
    ///
    /// let config_manager = ConfigManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(HashMap::new())),
            files: HashMap::new(),
        }
    }

    /// Sets the path for a configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The configuration scope to set the path for
    /// * `path` - The path to the configuration file
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ConfigManager, ConfigScope};
    /// use std::path::PathBuf;
    ///
    /// let mut config_manager = ConfigManager::new();
    /// config_manager.set_path(ConfigScope::User, PathBuf::from("~/.config/myapp.json"));
    /// ```
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>) {
        self.files.insert(scope, path.into());
    }

    /// Gets the path for a configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The configuration scope to get the path for
    ///
    /// # Returns
    ///
    /// * `Some(&PathBuf)` - If a path is set for the scope
    /// * `None` - If no path is set for the scope
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ConfigManager, ConfigScope};
    /// use std::path::PathBuf;
    ///
    /// let mut config_manager = ConfigManager::new();
    /// config_manager.set_path(ConfigScope::User, PathBuf::from("~/.config/myapp.json"));
    /// 
    /// assert!(config_manager.get_path(ConfigScope::User).is_some());
    /// assert!(config_manager.get_path(ConfigScope::Global).is_none());
    /// ```
    #[must_use]
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf> {
        self.files.get(&scope)
    }

    /// Sets a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    /// * `value` - The configuration value
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ConfigManager, ConfigValue};
    ///
    /// let mut config_manager = ConfigManager::new();
    /// config_manager.set("theme", ConfigValue::String("dark".to_string()));
    /// ```
    pub fn set(&mut self, key: &str, value: ConfigValue) {
        if let Ok(mut settings) = self.settings.write() {
            settings.insert(key.to_string(), value);
        }
    }

    /// Gets a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    ///
    /// # Returns
    ///
    /// * `Some(ConfigValue)` - If the key exists
    /// * `None` - If the key doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ConfigManager, ConfigValue};
    ///
    /// let mut config_manager = ConfigManager::new();
    /// config_manager.set("theme", ConfigValue::String("dark".to_string()));
    /// 
    /// if let Some(theme) = config_manager.get("theme") {
    ///     println!("Theme: {:?}", theme);
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        if let Ok(settings) = self.settings.read() {
            settings.get(key).cloned()
        } else {
            None
        }
    }

    /// Removes a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to remove
    ///
    /// # Returns
    ///
    /// * `Some(ConfigValue)` - The removed value if it existed
    /// * `None` - If the key didn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::{ConfigManager, ConfigValue};
    ///
    /// let mut config_manager = ConfigManager::new();
    /// config_manager.set("theme", ConfigValue::String("dark".to_string()));
    /// 
    /// let removed = config_manager.remove("theme");
    /// assert!(removed.is_some());
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<ConfigValue> {
        if let Ok(mut settings) = self.settings.write() {
            settings.remove(key)
        } else {
            None
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}