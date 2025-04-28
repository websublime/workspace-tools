use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::{StandardError, StandardResult};

/// Configuration scope defining where settings apply.
///
/// Represents the different levels at which configuration can be stored and applied.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::config::ConfigScope;
///
/// // User-specific settings
/// let user_scope = ConfigScope::User;
///
/// // Project-specific settings
/// let project_scope = ConfigScope::Project;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// Global configuration (system-wide)
    Global,
    /// User configuration (user-specific)
    User,
    /// Project configuration (project-specific)
    Project,
    /// Runtime configuration (in-memory only)
    Runtime,
}

/// Configuration file format.
///
/// Supported serialization formats for configuration files.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::config::ConfigFormat;
///
/// // JSON format
/// let json_format = ConfigFormat::Json;
///
/// // TOML format
/// let toml_format = ConfigFormat::Toml;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

/// Configuration value types for storing different kinds of settings.
///
/// A variant enum that can represent any valid configuration value type.
///
/// # Examples
///
/// ```
/// use sublime_standard_tools::config::ConfigValue;
/// use std::collections::HashMap;
///
/// // String value
/// let name = ConfigValue::String("my-app".to_string());
///
/// // Integer value
/// let timeout = ConfigValue::Integer(30);
///
/// // Boolean value
/// let enabled = ConfigValue::Boolean(true);
///
/// // Nested map value
/// let mut server = HashMap::new();
/// server.insert("host".to_string(), ConfigValue::String("localhost".to_string()));
/// server.insert("port".to_string(), ConfigValue::Integer(8080));
/// let server_config = ConfigValue::Map(server);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Map of values
    Map(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl ConfigValue {
    /// Returns true if this value is a string.
    ///
    /// # Returns
    ///
    /// `true` if the value is a string, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(value.is_string());
    ///
    /// let value = ConfigValue::Integer(42);
    /// assert!(!value.is_string());
    /// ```
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns true if this value is an integer.
    ///
    /// # Returns
    ///
    /// `true` if the value is an integer, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Integer(42);
    /// assert!(value.is_integer());
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(!value.is_integer());
    /// ```
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Returns true if this value is a float.
    ///
    /// # Returns
    ///
    /// `true` if the value is a float, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Float(3.14);
    /// assert!(value.is_float());
    ///
    /// let value = ConfigValue::Integer(42);
    /// assert!(!value.is_float());
    /// ```
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Returns true if this value is a boolean.
    ///
    /// # Returns
    ///
    /// `true` if the value is a boolean, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Boolean(true);
    /// assert!(value.is_boolean());
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(!value.is_boolean());
    /// ```
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns true if this value is an array.
    ///
    /// # Returns
    ///
    /// `true` if the value is an array, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Array(vec![
    ///     ConfigValue::Integer(1),
    ///     ConfigValue::Integer(2)
    /// ]);
    /// assert!(value.is_array());
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(!value.is_array());
    /// ```
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this value is a map.
    ///
    /// # Returns
    ///
    /// `true` if the value is a map, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), ConfigValue::String("value".to_string()));
    /// let value = ConfigValue::Map(map);
    /// assert!(value.is_map());
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(!value.is_map());
    /// ```
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Returns true if this value is null.
    ///
    /// # Returns
    ///
    /// `true` if the value is null, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Null;
    /// assert!(value.is_null());
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert!(!value.is_null());
    /// ```
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Gets this value as a string.
    ///
    /// # Returns
    ///
    /// The string value if this is a string, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_string(), Some("test"));
    ///
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_string(), None);
    /// ```
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Gets this value as an integer.
    ///
    /// # Returns
    ///
    /// The integer value if this is an integer, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_integer(), Some(42));
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_integer(), None);
    /// ```
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Gets this value as a float.
    ///
    /// Integers are automatically converted to floats.
    ///
    /// # Returns
    ///
    /// The float value if this is a float or integer, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Float(3.14);
    /// assert_eq!(value.as_float(), Some(3.14));
    ///
    /// // Integers can be converted to floats
    /// let value = ConfigValue::Integer(42);
    /// assert_eq!(value.as_float(), Some(42.0));
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_float(), None);
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

    /// Gets this value as a boolean.
    ///
    /// # Returns
    ///
    /// The boolean value if this is a boolean, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Boolean(true);
    /// assert_eq!(value.as_boolean(), Some(true));
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_boolean(), None);
    /// ```
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Gets this value as an array.
    ///
    /// # Returns
    ///
    /// A slice of the array values if this is an array, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    ///
    /// let value = ConfigValue::Array(vec![
    ///     ConfigValue::Integer(1),
    ///     ConfigValue::Integer(2)
    /// ]);
    /// assert_eq!(value.as_array().unwrap().len(), 2);
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_array(), None);
    /// ```
    #[must_use]
    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Gets this value as a map.
    ///
    /// # Returns
    ///
    /// A reference to the map if this is a map, or `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigValue;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), ConfigValue::String("value".to_string()));
    /// let value = ConfigValue::Map(map);
    ///
    /// let map_ref = value.as_map().unwrap();
    /// assert_eq!(map_ref.get("key").unwrap().as_string(), Some("value"));
    ///
    /// let value = ConfigValue::String("test".to_string());
    /// assert_eq!(value.as_map(), None);
    /// ```
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// Configuration manager for storing and retrieving settings.
///
/// Provides functionality for loading, saving, and accessing configuration values
/// from various scopes and file formats.
///
/// # Examples
///
/// ```no_run
/// use sublime_standard_tools::config::{ConfigManager, ConfigScope, ConfigValue};
///
/// let mut config = ConfigManager::new();
///
/// // Set configuration paths
/// config.set_path(ConfigScope::User, "~/.config/app.json");
/// config.set_path(ConfigScope::Project, "./app.json");
///
/// // Load configurations
/// config.load_all().expect("Failed to load configurations");
///
/// // Get a configuration value
/// if let Some(timeout) = config.get("timeout") {
///     if let Some(secs) = timeout.as_integer() {
///         println!("Timeout: {} seconds", secs);
///     }
/// }
///
/// // Set a configuration value
/// config.set("timeout", ConfigValue::Integer(30));
///
/// // Save configurations
/// config.save_all().expect("Failed to save configurations");
/// ```
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Configuration settings
    settings: Arc<RwLock<HashMap<String, ConfigValue>>>,
    /// Paths for different configuration scopes
    files: HashMap<ConfigScope, PathBuf>,
}

impl ConfigManager {
    /// Creates a new, empty configuration manager.
    ///
    /// # Returns
    ///
    /// A new configuration manager with no settings or file paths
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigManager;
    ///
    /// let config = ConfigManager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { settings: Arc::new(RwLock::new(HashMap::new())), files: HashMap::new() }
    }

    /// Sets the path for a specific configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The configuration scope to set the path for
    /// * `path` - The file path for the specified scope
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigScope};
    ///
    /// let mut config = ConfigManager::new();
    /// config.set_path(ConfigScope::User, "~/.config/app.json");
    /// config.set_path(ConfigScope::Project, "./app.json");
    /// ```
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>) {
        self.files.insert(scope, path.into());
    }

    /// Gets the path for a specific configuration scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - The configuration scope to get the path for
    ///
    /// # Returns
    ///
    /// The file path for the specified scope, or `None` if not set
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigScope};
    /// use std::path::PathBuf;
    ///
    /// let mut config = ConfigManager::new();
    /// config.set_path(ConfigScope::User, "~/.config/app.json");
    ///
    /// assert_eq!(config.get_path(ConfigScope::User), Some(&PathBuf::from("~/.config/app.json")));
    /// assert_eq!(config.get_path(ConfigScope::Project), None);
    /// ```
    #[must_use]
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf> {
        self.files.get(&scope)
    }

    /// Loads configurations from all registered file paths.
    ///
    /// Skips the Runtime scope since it's in-memory only.
    ///
    /// # Returns
    ///
    /// Success if all configurations loaded successfully, or an error if any failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::config::{ConfigManager, ConfigScope};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut config = ConfigManager::new();
    /// config.set_path(ConfigScope::User, "~/.config/app.json");
    /// config.set_path(ConfigScope::Project, "./app.json");
    ///
    /// config.load_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_all(&self) -> StandardResult<()> {
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
    /// # Arguments
    ///
    /// * `path` - The file path to load configuration from
    ///
    /// # Returns
    ///
    /// Success if the configuration loaded successfully, or an error if it failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::config::ConfigManager;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigManager::new();
    /// config.load_from_file(Path::new("./config.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_from_file(&self, path: &Path) -> StandardResult<()> {
        if !path.exists() {
            return Err(StandardError::operation(format!(
                "Configuration file does not exist: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            StandardError::operation(format!(
                "Failed to read configuration file {}: {}",
                path.display(),
                e
            ))
        })?;

        let format = self.detect_format(path);
        let config = self.parse_config(&content, format)?;

        match config {
            ConfigValue::Map(map) => {
                let mut settings = self.settings.write().map_err(|_| {
                    StandardError::operation(
                        "Failed to acquire write lock for configuration settings",
                    )
                })?;

                for (key, value) in map {
                    settings.insert(key, value);
                }
                Ok(())
            }
            _ => Err(StandardError::operation(format!(
                "Configuration must be a map: {}",
                path.display()
            ))),
        }
    }

    /// Saves all configurations to their respective files.
    ///
    /// Skips the Runtime scope since it's in-memory only.
    ///
    /// # Returns
    ///
    /// Success if all configurations saved successfully, or an error if any failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::config::{ConfigManager, ConfigScope};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut config = ConfigManager::new();
    /// config.set_path(ConfigScope::User, "~/.config/app.json");
    /// config.set_path(ConfigScope::Project, "./app.json");
    ///
    /// // After making changes
    /// config.save_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_all(&self) -> StandardResult<()> {
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
    /// # Arguments
    ///
    /// * `path` - The file path to save configuration to
    ///
    /// # Returns
    ///
    /// Success if the configuration saved successfully, or an error if it failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sublime_standard_tools::config::ConfigManager;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConfigManager::new();
    /// config.save_to_file(Path::new("./config.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_to_file(&self, path: &Path) -> StandardResult<()> {
        let settings = self.settings.read().map_err(|_| {
            StandardError::operation("Failed to acquire read lock for configuration settings")
        })?;

        let config = ConfigValue::Map(settings.clone());

        let format = self.detect_format(path);
        let content = self.serialize_config(&config, format)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                StandardError::operation(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            StandardError::operation(format!(
                "Failed to write configuration file {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Gets a configuration value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get the value for
    ///
    /// # Returns
    ///
    /// The configuration value for the specified key, or `None` if not found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigValue};
    ///
    /// let config = ConfigManager::new();
    /// config.set("timeout", ConfigValue::Integer(30));
    ///
    /// if let Some(timeout) = config.get("timeout") {
    ///     if let Some(secs) = timeout.as_integer() {
    ///         println!("Timeout: {} seconds", secs);
    ///     }
    /// }
    /// ```
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

    /// Sets a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set the value for
    /// * `value` - The value to set
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigValue};
    ///
    /// let config = ConfigManager::new();
    /// config.set("timeout", ConfigValue::Integer(30));
    /// config.set("server", ConfigValue::String("localhost".to_string()));
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

    /// Removes a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Returns
    ///
    /// The previous value for the key if it existed, or `None` if not found
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigValue};
    ///
    /// let config = ConfigManager::new();
    /// config.set("temporary", ConfigValue::String("value".to_string()));
    ///
    /// // Remove the temporary value
    /// let previous = config.remove("temporary");
    /// assert!(previous.is_some());
    /// assert!(config.get("temporary").is_none());
    /// ```
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

    #[allow(clippy::unused_self)]
    /// Detects the configuration format from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to detect the format from
    ///
    /// # Returns
    ///
    /// The detected configuration format based on file extension
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigFormat};
    /// use std::path::Path;
    ///
    /// let config = ConfigManager::new();
    ///
    /// // Method is not public, but would work like this:
    /// // assert_eq!(config.detect_format(Path::new("config.json")), ConfigFormat::Json);
    /// // assert_eq!(config.detect_format(Path::new("config.toml")), ConfigFormat::Toml);
    /// // assert_eq!(config.detect_format(Path::new("config.yaml")), ConfigFormat::Yaml);
    /// ```
    fn detect_format(&self, path: &Path) -> ConfigFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("yaml" | "yml") => ConfigFormat::Yaml,
            _ => ConfigFormat::Json, // Default to JSON
        }
    }

    #[allow(clippy::unused_self)]
    /// Parses a configuration string into a ConfigValue.
    ///
    /// # Arguments
    ///
    /// * `content` - The configuration content to parse
    /// * `format` - The format of the configuration content
    ///
    /// # Returns
    ///
    /// The parsed configuration value, or an error if parsing failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigFormat};
    ///
    /// let config = ConfigManager::new();
    ///
    /// // Method is not public, but would work like this:
    /// // let json = r#"{"name": "test", "value": 42}"#;
    /// // let parsed = config.parse_config(json, ConfigFormat::Json).unwrap();
    /// ```
    fn parse_config(&self, content: &str, format: ConfigFormat) -> StandardResult<ConfigValue> {
        match format {
            ConfigFormat::Json => serde_json::from_str(content).map_err(|e| {
                StandardError::operation(format!("Failed to parse JSON configuration: {e}"))
            }),
            ConfigFormat::Toml => Err(StandardError::operation("TOML format not yet supported")),
            ConfigFormat::Yaml => Err(StandardError::operation("YAML format not yet supported")),
        }
    }

    #[allow(clippy::unused_self)]
    /// Serializes a ConfigValue into a string.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration value to serialize
    /// * `format` - The format to serialize to
    ///
    /// # Returns
    ///
    /// The serialized configuration string, or an error if serialization failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigFormat, ConfigValue};
    ///
    /// let config = ConfigManager::new();
    ///
    /// // Method is not public, but would work like this:
    /// // let value = ConfigValue::String("test".to_string());
    /// // let json = config.serialize_config(&value, ConfigFormat::Json).unwrap();
    /// ```
    fn serialize_config(
        &self,
        config: &ConfigValue,
        format: ConfigFormat,
    ) -> StandardResult<String> {
        match format {
            ConfigFormat::Json => serde_json::to_string_pretty(config).map_err(|e| {
                StandardError::operation(format!("Failed to serialize JSON configuration: {e}"))
            }),
            ConfigFormat::Toml => Err(StandardError::operation("TOML format not yet supported")),
            ConfigFormat::Yaml => Err(StandardError::operation("YAML format not yet supported")),
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_config_value_types() {
        // Create different types of values
        let string_val = ConfigValue::String("test".to_string());
        let int_val = ConfigValue::Integer(42);
        let float_val = ConfigValue::Float(std::f64::consts::PI);
        let bool_val = ConfigValue::Boolean(true);
        let array_val = ConfigValue::Array(vec![ConfigValue::Integer(1), ConfigValue::Integer(2)]);
        let mut map = HashMap::new();
        map.insert("key".to_string(), ConfigValue::String("value".to_string()));
        let map_val = ConfigValue::Map(map);
        let null_val = ConfigValue::Null;

        // Test type checking
        assert!(string_val.is_string());
        assert!(int_val.is_integer());
        assert!(float_val.is_float());
        assert!(bool_val.is_boolean());
        assert!(array_val.is_array());
        assert!(map_val.is_map());
        assert!(null_val.is_null());

        // Test value extraction
        assert_eq!(string_val.as_string(), Some("test"));
        assert_eq!(int_val.as_integer(), Some(42));
        assert_eq!(float_val.as_float(), Some(std::f64::consts::PI));
        assert_eq!(int_val.as_float(), Some(42.0)); // Integer can be coerced to float
        assert_eq!(bool_val.as_boolean(), Some(true));

        // Test array and map access
        assert_eq!(array_val.as_array().unwrap().len(), 2);
        assert_eq!(map_val.as_map().unwrap().get("key").unwrap().as_string(), Some("value"));
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_config_manager_basic() {
        let config_manager = ConfigManager::new();

        // Set values
        config_manager.set("string", ConfigValue::String("value".to_string()));
        config_manager.set("integer", ConfigValue::Integer(42));

        // Get values
        assert_eq!(config_manager.get("string").unwrap().as_string(), Some("value"));
        assert_eq!(config_manager.get("integer").unwrap().as_integer(), Some(42));

        // Non-existent key
        assert_eq!(config_manager.get("nonexistent"), None);

        // Remove
        assert_eq!(config_manager.remove("string").unwrap().as_string(), Some("value"));
        assert_eq!(config_manager.get("string"), None);
    }

    #[allow(clippy::unwrap_used)]
    #[test]
    fn test_config_file_operations() -> StandardResult<()> {
        // Create a temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            r#"{{
                "string": "value",
                "integer": 42,
                "boolean": true
            }}"#
        )
        .unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Create config manager and load from file
        let mut config_manager = ConfigManager::new();
        config_manager.set_path(ConfigScope::Project, &temp_path);
        config_manager.load_from_file(&temp_path)?;

        // Check values were loaded
        assert_eq!(config_manager.get("string").unwrap().as_string(), Some("value"));
        assert_eq!(config_manager.get("integer").unwrap().as_integer(), Some(42));
        assert_eq!(config_manager.get("boolean").unwrap().as_boolean(), Some(true));

        // Modify and save
        config_manager.set("new_value", ConfigValue::String("new".to_string()));
        config_manager.save_to_file(&temp_path)?;

        // Create a new config manager and load the saved file
        let mut new_config_manager = ConfigManager::new();
        new_config_manager.set_path(ConfigScope::Project, &temp_path);
        new_config_manager.load_from_file(&temp_path)?;

        // Check all values, including the new one
        assert_eq!(new_config_manager.get("string").unwrap().as_string(), Some("value"));
        assert_eq!(new_config_manager.get("new_value").unwrap().as_string(), Some("new"));

        Ok(())
    }

    #[test]
    fn test_format_detection() {
        let config_manager = ConfigManager::new();

        assert_eq!(config_manager.detect_format(Path::new("config.json")), ConfigFormat::Json);
        assert_eq!(config_manager.detect_format(Path::new("config.toml")), ConfigFormat::Toml);
        assert_eq!(config_manager.detect_format(Path::new("config.yaml")), ConfigFormat::Yaml);
        assert_eq!(config_manager.detect_format(Path::new("config.yml")), ConfigFormat::Yaml);
        assert_eq!(
            config_manager.detect_format(Path::new("config")), // No extension
            ConfigFormat::Json                                 // Default
        );
    }
}
