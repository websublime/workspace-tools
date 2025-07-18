//! # Configuration Management Implementation
//!
//! ## What
//! This file implements advanced functionality for the `ConfigManager` struct,
//! providing methods to load, save, and manipulate configuration settings from files.
//! It supports multiple configuration formats including JSON, TOML, and YAML.
//!
//! ## How
//! The implementation provides methods for loading and saving configurations from files,
//! converting between different configuration file formats, and managing configuration
//! file I/O operations.
//!
//! ## Why
//! Applications need to persist configuration across different scopes and formats.
//! This implementation provides file I/O operations that complement the basic
//! in-memory configuration management provided by the base types.

use super::types::{ConfigFormat, ConfigManager, ConfigScope, ConfigValue};
use crate::error::{Error, FileSystemError, Result};
use std::{
    collections::HashMap,
    fs,
    path::Path,
};

impl ConfigManager {
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
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::ConfigManager;
    /// # use std::path::PathBuf;
    /// let mut config_manager = ConfigManager::new();
    /// match config_manager.load_all() {
    ///     Ok(()) => println!("All configurations loaded successfully"),
    ///     Err(e) => println!("Failed to load configurations: {}", e),
    /// }
    /// ```
    pub fn load_all(&mut self) -> Result<()> {
        for (scope, path) in &self.files.clone() {
            if *scope != ConfigScope::Runtime {
                self.load_from_file(path)?;
            }
        }
        Ok(())
    }

    /// Loads configuration from a specific file.
    ///
    /// This method reads and parses a configuration file, determining the format
    /// automatically from the file extension. The loaded configuration is merged
    /// with existing configuration values.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file to load
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The file cannot be read
    /// - The file format cannot be determined
    /// - The file contains invalid syntax
    /// - An I/O error occurs
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::ConfigManager;
    /// # use std::path::Path;
    /// let mut config_manager = ConfigManager::new();
    /// match config_manager.load_from_file(Path::new("config.json")) {
    ///     Ok(()) => println!("Configuration loaded successfully"),
    ///     Err(e) => println!("Failed to load configuration: {}", e),
    /// }
    /// ```
    pub fn load_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| FileSystemError::from_io(e, path.to_path_buf()))?;
        
        let format = self.detect_format(path)?;
        let config_map = self.parse_config(&content, format)?;
        
        // Merge loaded configuration with existing settings
        if let Ok(mut settings) = self.settings.write() {
            for (key, value) in config_map {
                settings.insert(key, value);
            }
        }
        
        Ok(())
    }

    /// Saves all configuration changes to their respective files.
    ///
    /// This method writes all configuration values to their configured file paths,
    /// except for Runtime scope which is memory-only. Each scope is saved to its
    /// configured file path using the appropriate format.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - A configuration file cannot be written
    /// - A directory cannot be created
    /// - An I/O error occurs while writing
    /// - The configuration format cannot be determined
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::ConfigManager;
    /// let mut config_manager = ConfigManager::new();
    /// match config_manager.save_all() {
    ///     Ok(()) => println!("All configurations saved successfully"),
    ///     Err(e) => println!("Failed to save configurations: {}", e),
    /// }
    /// ```
    pub fn save_all(&mut self) -> Result<()> {
        for (scope, path) in &self.files.clone() {
            if *scope != ConfigScope::Runtime {
                self.save_to_file(path)?;
            }
        }
        Ok(())
    }

    /// Saves configuration to a specific file.
    ///
    /// This method writes all current configuration values to the specified file,
    /// using the format determined by the file extension. The directory is created
    /// if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the configuration file should be saved
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The file cannot be written
    /// - The directory cannot be created
    /// - An I/O error occurs
    /// - The configuration format cannot be determined
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::ConfigManager;
    /// # use std::path::Path;
    /// let mut config_manager = ConfigManager::new();
    /// match config_manager.save_to_file(Path::new("config.json")) {
    ///     Ok(()) => println!("Configuration saved successfully"),
    ///     Err(e) => println!("Failed to save configuration: {}", e),
    /// }
    /// ```
    pub fn save_to_file(&mut self, path: &Path) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| FileSystemError::from_io(e, parent.to_path_buf()))?;
        }
        
        let format = self.detect_format(path)?;
        let content = self.serialize_config(format)?;
        
        fs::write(path, content)
            .map_err(|e| FileSystemError::from_io(e, path.to_path_buf()))?;
        
        Ok(())
    }

    /// Detects the configuration format from a file path.
    ///
    /// This method examines the file extension to determine the appropriate
    /// configuration format (JSON, TOML, or YAML).
    ///
    /// # Arguments
    ///
    /// * `path` - The path to examine for format detection
    ///
    /// # Returns
    ///
    /// Returns the detected `ConfigFormat` based on the file extension.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file extension is not recognized or missing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_standard_tools::project::ConfigManager;
    /// # use std::path::Path;
    /// let config_manager = ConfigManager::new();
    /// match config_manager.detect_format(Path::new("config.json")) {
    ///     Ok(format) => println!("Detected format: {:?}", format),
    ///     Err(e) => println!("Could not detect format: {}", e),
    /// }
    /// ```
    pub fn detect_format(&self, path: &Path) -> Result<ConfigFormat> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| FileSystemError::Validation {
                path: path.to_path_buf(),
                reason: "No file extension found".to_string(),
            })?;
        
        match extension.to_lowercase().as_str() {
            "json" => Ok(ConfigFormat::Json),
            "toml" => Ok(ConfigFormat::Toml),
            "yaml" | "yml" => Ok(ConfigFormat::Yaml),
            _ => Err(FileSystemError::Validation {
                path: path.to_path_buf(),
                reason: format!("Unsupported configuration format: {extension}"),
            }.into()),
        }
    }

    /// Parses configuration content based on the specified format.
    ///
    /// This method takes configuration content as a string and parses it according
    /// to the specified format, returning a HashMap of configuration values.
    ///
    /// # Arguments
    ///
    /// * `content` - The configuration content to parse
    /// * `format` - The format to use for parsing
    ///
    /// # Returns
    ///
    /// Returns a HashMap containing the parsed configuration values.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the content cannot be parsed in the specified format.
    pub fn parse_config(&self, content: &str, format: ConfigFormat) -> Result<HashMap<String, ConfigValue>> {
        match format {
            ConfigFormat::Json => {
                let json_value: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| Error::Operation(format!("Invalid JSON: {e}")))?;
                self.convert_json_to_config_map(json_value)
            }
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(content)
                    .map_err(|e| Error::Operation(format!("Invalid TOML: {e}")))?;
                self.convert_toml_to_config_map(toml_value)
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
                    .map_err(|e| Error::Operation(format!("Invalid YAML: {e}")))?;
                self.convert_yaml_to_config_map(yaml_value)
            }
        }
    }

    /// Serializes configuration to a string in the specified format.
    ///
    /// This method converts all current configuration values to a string
    /// representation using the specified format.
    ///
    /// # Arguments
    ///
    /// * `format` - The format to use for serialization
    ///
    /// # Returns
    ///
    /// Returns a string containing the serialized configuration.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the configuration cannot be serialized.
    pub fn serialize_config(&self, format: ConfigFormat) -> Result<String> {
        let settings = self.settings.read()
            .map_err(|_| Error::Operation("Failed to read settings".to_string()))?;
        
        match format {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(&*settings)
                    .map_err(|e| Error::Operation(format!("Failed to serialize JSON: {e}")))
            }
            ConfigFormat::Toml => {
                toml::to_string_pretty(&*settings)
                    .map_err(|e| Error::Operation(format!("Failed to serialize TOML: {e}")))
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(&*settings)
                    .map_err(|e| Error::Operation(format!("Failed to serialize YAML: {e}")))
            }
        }
    }

    /// Converts a JSON value to a ConfigValue map.
    fn convert_json_to_config_map(&self, value: serde_json::Value) -> Result<HashMap<String, ConfigValue>> {
        match value {
            serde_json::Value::Object(map) => {
                let mut config_map = HashMap::new();
                for (key, value) in map {
                    config_map.insert(key, self.convert_json_to_config_value(value)?);
                }
                Ok(config_map)
            }
            _ => Err(Error::Operation("Root JSON value must be an object".to_string())),
        }
    }

    /// Converts a JSON value to a ConfigValue.
    #[allow(clippy::only_used_in_recursion)]
    fn convert_json_to_config_value(&self, value: serde_json::Value) -> Result<ConfigValue> {
        match value {
            serde_json::Value::String(s) => Ok(ConfigValue::String(s)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ConfigValue::Float(f))
                } else {
                    Err(Error::Operation("Invalid number format".to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(ConfigValue::Boolean(b)),
            serde_json::Value::Array(arr) => {
                let mut config_arr = Vec::new();
                for item in arr {
                    config_arr.push(self.convert_json_to_config_value(item)?);
                }
                Ok(ConfigValue::Array(config_arr))
            }
            serde_json::Value::Object(obj) => {
                let mut config_map = HashMap::new();
                for (key, value) in obj {
                    config_map.insert(key, self.convert_json_to_config_value(value)?);
                }
                Ok(ConfigValue::Map(config_map))
            }
            serde_json::Value::Null => Ok(ConfigValue::Null),
        }
    }

    /// Converts a TOML value to a ConfigValue map.
    fn convert_toml_to_config_map(&self, value: toml::Value) -> Result<HashMap<String, ConfigValue>> {
        match value {
            toml::Value::Table(table) => {
                let mut config_map = HashMap::new();
                for (key, value) in table {
                    config_map.insert(key, self.convert_toml_to_config_value(value)?);
                }
                Ok(config_map)
            }
            _ => Err(Error::Operation("Root TOML value must be a table".to_string())),
        }
    }

    /// Converts a TOML value to a ConfigValue.
    #[allow(clippy::only_used_in_recursion)]
    fn convert_toml_to_config_value(&self, value: toml::Value) -> Result<ConfigValue> {
        match value {
            toml::Value::String(s) => Ok(ConfigValue::String(s)),
            toml::Value::Integer(i) => Ok(ConfigValue::Integer(i)),
            toml::Value::Float(f) => Ok(ConfigValue::Float(f)),
            toml::Value::Boolean(b) => Ok(ConfigValue::Boolean(b)),
            toml::Value::Array(arr) => {
                let mut config_arr = Vec::new();
                for item in arr {
                    config_arr.push(self.convert_toml_to_config_value(item)?);
                }
                Ok(ConfigValue::Array(config_arr))
            }
            toml::Value::Table(table) => {
                let mut config_map = HashMap::new();
                for (key, value) in table {
                    config_map.insert(key, self.convert_toml_to_config_value(value)?);
                }
                Ok(ConfigValue::Map(config_map))
            }
            toml::Value::Datetime(_) => Err(Error::Operation("TOML datetime not supported".to_string())),
        }
    }

    /// Converts a YAML value to a ConfigValue map.
    fn convert_yaml_to_config_map(&self, value: serde_yaml::Value) -> Result<HashMap<String, ConfigValue>> {
        match value {
            serde_yaml::Value::Mapping(mapping) => {
                let mut config_map = HashMap::new();
                for (key, value) in mapping {
                    if let serde_yaml::Value::String(key_str) = key {
                        config_map.insert(key_str, self.convert_yaml_to_config_value(value)?);
                    }
                }
                Ok(config_map)
            }
            _ => Err(Error::Operation("Root YAML value must be a mapping".to_string())),
        }
    }

    /// Converts a YAML value to a ConfigValue.
    #[allow(clippy::only_used_in_recursion)]
    fn convert_yaml_to_config_value(&self, value: serde_yaml::Value) -> Result<ConfigValue> {
        match value {
            serde_yaml::Value::String(s) => Ok(ConfigValue::String(s)),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ConfigValue::Float(f))
                } else {
                    Err(Error::Operation("Invalid number format".to_string()))
                }
            }
            serde_yaml::Value::Bool(b) => Ok(ConfigValue::Boolean(b)),
            serde_yaml::Value::Sequence(seq) => {
                let mut config_arr = Vec::new();
                for item in seq {
                    config_arr.push(self.convert_yaml_to_config_value(item)?);
                }
                Ok(ConfigValue::Array(config_arr))
            }
            serde_yaml::Value::Mapping(mapping) => {
                let mut config_map = HashMap::new();
                for (key, value) in mapping {
                    if let serde_yaml::Value::String(key_str) = key {
                        config_map.insert(key_str, self.convert_yaml_to_config_value(value)?);
                    }
                }
                Ok(ConfigValue::Map(config_map))
            }
            serde_yaml::Value::Null => Ok(ConfigValue::Null),
            serde_yaml::Value::Tagged(_) => Err(Error::Operation("YAML tagged values not supported".to_string())),
        }
    }
}