use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::{StandardError, StandardResult};

/// Configuration scope
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

/// Configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

/// Configuration value types
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
    /// Returns true if this value is a string
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns true if this value is an integer
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Returns true if this value is a float
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Returns true if this value is a boolean
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns true if this value is an array
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns true if this value is a map
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    /// Returns true if this value is null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Gets this value as a string
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Gets this value as an integer
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Gets this value as a float
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Gets this value as a boolean
    #[must_use]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Gets this value as an array
    #[must_use]
    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Gets this value as a map
    #[must_use]
    pub fn as_map(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// Configuration manager for storing and retrieving settings
#[derive(Debug, Clone)]
pub struct ConfigManager {
    settings: Arc<RwLock<HashMap<String, ConfigValue>>>,
    files: HashMap<ConfigScope, PathBuf>,
}

impl ConfigManager {
    /// Creates a new, empty configuration manager
    #[must_use]
    pub fn new() -> Self {
        Self { settings: Arc::new(RwLock::new(HashMap::new())), files: HashMap::new() }
    }

    /// Sets the path for a specific configuration scope
    pub fn set_path(&mut self, scope: ConfigScope, path: impl Into<PathBuf>) {
        self.files.insert(scope, path.into());
    }

    /// Gets the path for a specific configuration scope
    #[must_use]
    pub fn get_path(&self, scope: ConfigScope) -> Option<&PathBuf> {
        self.files.get(&scope)
    }

    /// Loads configurations from all registered file paths
    pub fn load_all(&self) -> StandardResult<()> {
        for (scope, path) in &self.files {
            match scope {
                ConfigScope::Runtime => continue, // Skip runtime scope
                _ => self.load_from_file(path)?,
            }
        }
        Ok(())
    }

    /// Loads configuration from a specific file
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
                let mut settings = self.settings.write().expect("Failed to acquire write lock");
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

    /// Saves all configurations to their respective files
    pub fn save_all(&self) -> StandardResult<()> {
        for (scope, path) in &self.files {
            match scope {
                ConfigScope::Runtime => continue, // Skip runtime scope
                _ => self.save_to_file(path)?,
            }
        }
        Ok(())
    }

    /// Saves configuration to a specific file
    pub fn save_to_file(&self, path: &Path) -> StandardResult<()> {
        let settings = self.settings.read().expect("Failed to acquire read lock");
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

    /// Gets a configuration value by key
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        let settings = self.settings.read().expect("Failed to acquire read lock");
        settings.get(key).cloned()
    }

    /// Sets a configuration value
    pub fn set(&self, key: &str, value: ConfigValue) {
        let mut settings = self.settings.write().expect("Failed to acquire write lock");
        settings.insert(key.to_string(), value);
    }

    /// Removes a configuration value
    pub fn remove(&self, key: &str) -> Option<ConfigValue> {
        let mut settings = self.settings.write().expect("Failed to acquire write lock");
        settings.remove(key)
    }

    #[allow(clippy::unused_self)]
    /// Detects the configuration format from a file path
    fn detect_format(&self, path: &Path) -> ConfigFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("yaml" | "yml") => ConfigFormat::Yaml,
            _ => ConfigFormat::Json, // Default to JSON
        }
    }

    #[allow(clippy::unused_self)]
    /// Parses a configuration string into a ConfigValue
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
    /// Serializes a ConfigValue into a string
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
