//! # Configuration Management Implementation
//!
//! ## What
//! This file implements functionality for the `ConfigManager` struct and `ConfigValue` enum,
//! providing methods to load, save, and manipulate configuration settings across different
//! scopes (global, user, project, runtime). It supports multiple configuration formats
//! including JSON, TOML, and YAML.
//!
//! ## How
//! The implementation provides methods for loading and saving configurations from files,
//! accessing and modifying configuration values in a thread-safe manner using `RwLock`,
//! and converting between different configuration file formats. Configuration values
//! are represented as a flexible enum that can store various data types.
//!
//! ## Why
//! Applications need consistent and flexible configuration management that works across
//! multiple scopes (from system-wide to in-memory runtime settings). This implementation
//! provides a unified approach to configuration handling with strong typing and thread safety.

use super::types::{ConfigFormat, ConfigManager, ConfigScope, ConfigValue};
use crate::error::{Error, FileSystemError, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

impl ConfigValue {
    /// Checks if the configuration value is a string.
    ///
    /// # Returns
    ///
    /// `true` if the value is a string, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(val.is_string());
    ///
    /// let val = ConfigValue::Integer(42);
    /// assert!(!val.is_string());
    /// ```
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Checks if the configuration value is an integer.
    ///
    /// # Returns
    ///
    /// `true` if the value is an integer, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Integer(42);
    /// assert!(val.is_integer());
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(!val.is_integer());
    /// ```
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Checks if the configuration value is a floating-point number.
    ///
    /// # Returns
    ///
    /// `true` if the value is a float, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Float(3.14);
    /// assert!(val.is_float());
    ///
    /// let val = ConfigValue::Integer(42);
    /// assert!(!val.is_float());
    /// ```
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Checks if the configuration value is a boolean.
    ///
    /// # Returns
    ///
    /// `true` if the value is a boolean, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Boolean(true);
    /// assert!(val.is_boolean());
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(!val.is_boolean());
    /// ```
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Checks if the configuration value is an array.
    ///
    /// # Returns
    ///
    /// `true` if the value is an array, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Array(vec![]);
    /// assert!(val.is_array());
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(!val.is_array());
    /// ```
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Checks if the configuration value is a map.
    ///
    /// # Returns
    ///
    /// `true` if the value is a map, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Map(HashMap::new());
    /// assert!(val.is_map());
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(!val.is_map());
    /// ```
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Checks if the configuration value is null.
    ///
    /// # Returns
    ///
    /// `true` if the value is null, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Null;
    /// assert!(val.is_null());
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert!(!val.is_null());
    /// ```
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the string value if this configuration value is a string.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - If the value is a string
    /// * `None` - If the value is not a string
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_string(), Some("hello"));
    ///
    /// let val = ConfigValue::Integer(42);
    /// assert_eq!(val.as_string(), None);
    /// ```
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the integer value if this configuration value is an integer.
    ///
    /// # Returns
    ///
    /// * `Some(i64)` - If the value is an integer
    /// * `None` - If the value is not an integer
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Integer(42);
    /// assert_eq!(val.as_integer(), Some(42));
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_integer(), None);
    /// ```
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the floating-point value if this configuration value is a float.
    ///
    /// This method also returns integer values converted to floats.
    ///
    /// # Returns
    ///
    /// * `Some(f64)` - If the value is a float or integer
    /// * `None` - If the value is neither a float nor an integer
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Float(3.14);
    /// assert_eq!(val.as_float(), Some(3.14));
    ///
    /// // Integer values are converted to floats
    /// let val = ConfigValue::Integer(42);
    /// assert_eq!(val.as_float(), Some(42.0));
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_float(), None);
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Returns the boolean value if this configuration value is a boolean.
    ///
    /// # Returns
    ///
    /// * `Some(bool)` - If the value is a boolean
    /// * `None` - If the value is not a boolean
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Boolean(true);
    /// assert_eq!(val.as_boolean(), Some(true));
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_boolean(), None);
    /// ```
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the array value if this configuration value is an array.
    ///
    /// # Returns
    ///
    /// * `Some(&[ConfigValue])` - If the value is an array
    /// * `None` - If the value is not an array
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let val = ConfigValue::Array(vec![ConfigValue::Integer(1), ConfigValue::Integer(2)]);
    /// assert!(val.as_array().is_some());
    /// assert_eq!(val.as_array().unwrap().len(), 2);
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_array(), None);
    /// ```
    #[must_use]
    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the map value if this configuration value is a map.
    ///
    /// # Returns
    ///
    /// * `Some(&HashMap<String, ConfigValue>)` - If the value is a map
    /// * `None` - If the value is not a map
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use sublime_standard_tools::project::ConfigValue;
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), ConfigValue::Integer(42));
    /// let val = ConfigValue::Map(map);
    ///
    /// assert!(val.as_map().is_some());
    /// assert_eq!(val.as_map().unwrap().len(), 1);
    ///
    /// let val = ConfigValue::String("hello".to_string());
    /// assert_eq!(val.as_map(), None);
    /// ```
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    /// Creates a new `ConfigManager` with empty settings.
    ///
    /// # Returns
    ///
    /// A new `ConfigManager` instance with no configuration loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::project::types::ConfigManager;
    ///
    /// let config_manager = ConfigManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { settings: Arc::new(RwLock::new(HashMap::new())), files: HashMap::new() }
    }

    /// Sets the path for a specific configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope to set the path for (Global, User, Project, Runtime)
    /// * `path` - The path where the configuration file for this scope is located
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigScope};
    /// let mut config_manager = ConfigManager::new();
    ///
    /// // Set the path for the project configuration
    /// config_manager.set_path(ConfigScope::Project, "/path/to/project/config.json");
    /// ```
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>) {
        self.files.insert(scope, path.into());
    }

    /// Gets the path for a specific configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope to get the path for
    ///
    /// # Returns
    ///
    /// * `Some(&PathBuf)` - The path where the configuration file for this scope is located
    /// * `None` - If no path has been set for this scope
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigScope};
    /// # let mut config_manager = ConfigManager::new();
    /// # config_manager.set_path(ConfigScope::Project, "/path/to/project/config.json");
    /// if let Some(path) = config_manager.get_path(ConfigScope::Project) {
    ///     println!("Project configuration path: {}", path.display());
    /// }
    /// ```
    #[must_use]
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf> {
        self.files.get(&scope)
    }

    /// Loads all configuration files from all scopes.
    ///
    /// This method loads configuration from all configured scopes except Runtime,
    /// which is memory-only. If a configuration file cannot be loaded, an error
    /// is returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A configuration file cannot be read
    /// - A configuration file contains invalid format or syntax
    /// - An I/O error occurs while accessing a configuration file
    /// - The configuration format cannot be determined
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all configurations were loaded successfully
    /// * `Err(Error)` - If an error occurred while loading any configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigScope};
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let config_manager = ConfigManager::new();
    /// // After setting paths for various scopes...
    /// config_manager.load_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_all(&self) -> Result<()> {
        for (scope, path) in &self.files {
            match scope {
                ConfigScope::Runtime => continue, // Skip runtime scope
                _ => self.load_from_file(path)?,
            }
        }
        Ok(())
    }

    /// Loads configuration from a specific file.
    ///
    /// This method reads the configuration file at the specified path,
    /// parses it according to its format, and merges it into the current settings.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file to load
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The configuration file cannot be read
    /// - The file contains invalid format or syntax
    /// - An I/O error occurs while accessing the file
    /// - The configuration format cannot be determined from the file extension
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration was loaded successfully
    /// * `Err(Error)` - If an error occurred while loading the configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::project::types::ConfigManager;
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let config_manager = ConfigManager::new();
    /// config_manager.load_from_file(Path::new("/path/to/config.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_from_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::FileSystem(FileSystemError::NotFound { path: path.to_path_buf() }))?;
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path.to_path_buf())))?;

        let format = ConfigManager::detect_format(path);
        let config = ConfigManager::parse_config(&content, format)?;

        match config {
            ConfigValue::Map(map) => {
                let mut settings = self.settings.write().map_err(|_| {
                    Error::operation("Failed to acquire write lock for configuration settings")
                })?;

                for (key, value) in map {
                    settings.insert(key, value);
                }
                Ok(())
            }
            _ => Err(Error::operation(format!("Configuration must be a map: {}", path.display()))),
        }
    }

    /// Saves all configurations to their respective files.
    ///
    /// This method saves the current settings to all configured scopes except Runtime,
    /// which is memory-only. If a configuration file cannot be saved, an error
    /// is returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A configuration file cannot be written
    /// - An I/O error occurs while accessing a configuration file
    /// - The configuration format cannot be determined
    /// - The configuration data cannot be serialized
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all configurations were saved successfully
    /// * `Err(Error)` - If an error occurred while saving any configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigScope};
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let config_manager = ConfigManager::new();
    /// // After setting values and paths...
    /// config_manager.save_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_all(&self) -> Result<()> {
        for (scope, path) in &self.files {
            match scope {
                ConfigScope::Runtime => continue, // Skip runtime scope
                _ => self.save_to_file(path)?,
            }
        }
        Ok(())
    }

    /// Saves configuration to a specific file.
    ///
    /// This method serializes the current settings to the specified file
    /// in the format determined by the file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file to save
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The configuration file cannot be written
    /// - An I/O error occurs while accessing the file
    /// - The configuration format cannot be determined from the file extension
    /// - The configuration data cannot be serialized to the target format
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration was saved successfully
    /// * `Err(Error)` - If an error occurred while saving the configuration
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::project::types::ConfigManager;
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let config_manager = ConfigManager::new();
    /// // After setting values...
    /// config_manager.save_to_file(Path::new("/path/to/config.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let settings = self.settings.read().map_err(|_| {
            Error::operation("Failed to acquire read lock for configuration settings")
        })?;

        let config = ConfigValue::Map(settings.clone());

        let format = ConfigManager::detect_format(path);
        let content = ConfigManager::serialize_config(&config, format)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path.to_path_buf())))?;
        }

        fs::write(path, content)
            .map_err(|e| Error::FileSystem(FileSystemError::from_io(e, path.to_path_buf())))?;

        Ok(())
    }

    /// Gets a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up in the configuration
    ///
    /// # Returns
    ///
    /// * `Some(ConfigValue)` - The value associated with the key, if it exists
    /// * `None` - If the key does not exist or the lock could not be acquired
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigValue};
    /// let config_manager = ConfigManager::new();
    /// config_manager.set("version", ConfigValue::String("1.0.0".to_string()));
    ///
    /// if let Some(value) = config_manager.get("version") {
    ///     if let Some(version) = value.as_string() {
    ///         println!("Version: {}", version);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        match self.settings.read() {
            Ok(settings) => settings.get(key).cloned(),
            Err(e) => {
                // Log the error and return None
                log::error!("Failed to acquire read lock for configuration: {}", e);
                None
            }
        }
    }

    /// Sets a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set in the configuration
    /// * `value` - The value to associate with the key
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigValue};
    /// let config_manager = ConfigManager::new();
    /// config_manager.set("version", ConfigValue::String("1.0.0".to_string()));
    /// config_manager.set("debug", ConfigValue::Boolean(true));
    /// ```
    pub fn set(&self, key: &str, value: ConfigValue) {
        match self.settings.write() {
            Ok(mut settings) => {
                settings.insert(key.to_string(), value);
            }
            Err(e) => {
                // Log the error
                log::error!("Failed to acquire write lock for configuration: {}", e);
            }
        }
    }

    /// Removes a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove from the configuration
    ///
    /// # Returns
    ///
    /// * `Some(ConfigValue)` - The value that was removed, if the key existed
    /// * `None` - If the key did not exist or the lock could not be acquired
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigValue};
    /// let config_manager = ConfigManager::new();
    /// config_manager.set("temp", ConfigValue::String("temporary value".to_string()));
    ///
    /// // Later, remove the temporary value
    /// if let Some(removed) = config_manager.remove("temp") {
    ///     println!("Removed temporary value");
    /// }
    /// ```
    #[must_use]
    pub fn remove(&self, key: &str) -> Option<ConfigValue> {
        match self.settings.write() {
            Ok(mut settings) => settings.remove(key),
            Err(e) => {
                // Log the error and return None
                log::error!("Failed to acquire write lock for configuration: {}", e);
                None
            }
        }
    }

    /// Detects the configuration format based on file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to determine the format for
    ///
    /// # Returns
    ///
    /// The detected `ConfigFormat` (Json, Toml, or Yaml)
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigFormat};
    /// let format = ConfigManager::detect_format(Path::new("config.json"));
    /// assert!(matches!(format, ConfigFormat::Json));
    ///
    /// let format = ConfigManager::detect_format(Path::new("config.toml"));
    /// assert!(matches!(format, ConfigFormat::Toml));
    ///
    /// let format = ConfigManager::detect_format(Path::new("config.yaml"));
    /// assert!(matches!(format, ConfigFormat::Yaml));
    /// ```
    pub(crate) fn detect_format(path: &Path) -> ConfigFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("yaml" | "yml") => ConfigFormat::Yaml,
            _ => ConfigFormat::Json,
        }
    }

    /// Parses configuration content based on the specified format.
    ///
    /// # Arguments
    ///
    /// * `content` - The string content to parse
    /// * `format` - The format to parse the content as
    ///
    /// # Returns
    ///
    /// * `Ok(ConfigValue)` - The parsed configuration value
    /// * `Err(Error)` - If parsing failed
    ///
    /// # Examples
    ///
    /// ```
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigFormat};
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let json_content = r#"{"name": "test", "value": 42}"#;
    /// let config = ConfigManager::parse_config(json_content, ConfigFormat::Json)?;
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn parse_config(content: &str, format: ConfigFormat) -> Result<ConfigValue> {
        match format {
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| Error::operation(format!("Failed to parse JSON configuration: {e}"))),
            ConfigFormat::Toml => toml::from_str(content)
                .map_err(|e| Error::operation(format!("Failed to parse TOML configuration: {e}"))),
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| Error::operation(format!("Failed to parse YAML configuration: {e}"))),
        }
    }

    /// Serializes configuration value to string based on the specified format.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration value to serialize
    /// * `format` - The format to serialize the value as
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The serialized configuration string
    /// * `Err(Error)` - If serialization failed
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use sublime_standard_tools::project::types::{ConfigManager, ConfigFormat, ConfigValue};
    /// # use sublime_standard_tools::error::Result;
    /// # fn example() -> Result<()> {
    /// let mut map = HashMap::new();
    /// map.insert("name".to_string(), ConfigValue::String("test".to_string()));
    /// let config = ConfigValue::Map(map);
    ///
    /// let json_string = ConfigManager::serialize_config(&config, ConfigFormat::Json)?;
    /// # Ok(())
    /// # }
    /// ```
    pub(crate) fn serialize_config(config: &ConfigValue, format: ConfigFormat) -> Result<String> {
        match format {
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                Error::operation(format!("Failed to serialize JSON configuration: {e}"))
            }),
            ConfigFormat::Toml => toml::to_string_pretty(config).map_err(|e| {
                Error::operation(format!("Failed to serialize TOML configuration: {e}"))
            }),
            ConfigFormat::Yaml => serde_yaml::to_string(config).map_err(|e| {
                Error::operation(format!("Failed to serialize YAML configuration: {e}"))
            }),
        }
    }
}
