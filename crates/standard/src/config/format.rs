//! Configuration format handling.
//!
//! This module provides support for various configuration file formats,
//! including JSON, TOML, and YAML.

use std::path::Path;

use super::error::{ConfigError, ConfigResult};
use super::value::ConfigValue;

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
}

impl ConfigFormat {
    /// Detects the format from a file path based on its extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to detect format from
    ///
    /// # Returns
    ///
    /// * `Some(ConfigFormat)` - If the extension is recognized
    /// * `None` - If the extension is not recognized
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use sublime_standard_tools::config::ConfigFormat;
    ///
    /// assert_eq!(ConfigFormat::from_path(Path::new("config.json")), Some(ConfigFormat::Json));
    /// assert_eq!(ConfigFormat::from_path(Path::new("config.toml")), Some(ConfigFormat::Toml));
    /// assert_eq!(ConfigFormat::from_path(Path::new("config.yaml")), Some(ConfigFormat::Yaml));
    /// assert_eq!(ConfigFormat::from_path(Path::new("config.yml")), Some(ConfigFormat::Yaml));
    /// assert_eq!(ConfigFormat::from_path(Path::new("config.txt")), None);
    /// ```
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension().and_then(|ext| ext.to_str()).and_then(Self::from_extension)
    }

    /// Detects the format from a file extension.
    ///
    /// # Arguments
    ///
    /// * `extension` - The file extension (without the dot)
    ///
    /// # Returns
    ///
    /// * `Some(ConfigFormat)` - If the extension is recognized
    /// * `None` - If the extension is not recognized
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigFormat;
    ///
    /// assert_eq!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json));
    /// assert_eq!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml));
    /// assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
    /// assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
    /// assert_eq!(ConfigFormat::from_extension("txt"), None);
    /// ```
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    /// Gets the typical file extension for this format.
    ///
    /// # Returns
    ///
    /// The file extension without the dot.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigFormat;
    ///
    /// assert_eq!(ConfigFormat::Json.extension(), "json");
    /// assert_eq!(ConfigFormat::Toml.extension(), "toml");
    /// assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
    /// ```
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
        }
    }

    /// Gets the format name as a string.
    ///
    /// # Returns
    ///
    /// The format name.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigFormat;
    ///
    /// assert_eq!(ConfigFormat::Json.as_str(), "JSON");
    /// assert_eq!(ConfigFormat::Toml.as_str(), "TOML");
    /// assert_eq!(ConfigFormat::Yaml.as_str(), "YAML");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
        }
    }

    /// Parses a string into a ConfigValue using this format.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to parse
    ///
    /// # Returns
    ///
    /// * `Ok(ConfigValue)` - The parsed configuration value
    /// * `Err(ConfigError)` - If parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigFormat, ConfigValue};
    ///
    /// let json = r#"{"name": "test", "value": 42}"#;
    /// let value = ConfigFormat::Json.parse(json).unwrap();
    /// assert_eq!(value.get("name").and_then(|v| v.as_string()), Some("test"));
    /// assert_eq!(value.get("value").and_then(|v| v.as_integer()), Some(42));
    /// ```
    pub fn parse(&self, content: &str) -> ConfigResult<ConfigValue> {
        match self {
            Self::Json => Self::parse_json(content),
            Self::Toml => Self::parse_toml(content),
            Self::Yaml => Self::parse_yaml(content),
        }
    }

    /// Serializes a ConfigValue to a string using this format.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to serialize
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The serialized string
    /// * `Err(ConfigError)` - If serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigFormat, ConfigValue};
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("name".to_string(), ConfigValue::String("test".to_string()));
    /// map.insert("value".to_string(), ConfigValue::Integer(42));
    /// let value = ConfigValue::Map(map);
    ///
    /// let json = ConfigFormat::Json.serialize(&value).unwrap();
    /// assert!(json.contains("\"name\":\"test\""));
    /// assert!(json.contains("\"value\":42"));
    /// ```
    pub fn serialize(&self, value: &ConfigValue) -> ConfigResult<String> {
        match self {
            Self::Json => Self::serialize_json(value),
            Self::Toml => Self::serialize_toml(value),
            Self::Yaml => Self::serialize_yaml(value),
        }
    }

    fn parse_json(content: &str) -> ConfigResult<ConfigValue> {
        serde_json::from_str(content).map_err(|e| ConfigError::parse("JSON", e.to_string()))
    }

    fn parse_toml(content: &str) -> ConfigResult<ConfigValue> {
        toml::from_str(content).map_err(|e| ConfigError::parse("TOML", e.to_string()))
    }

    fn parse_yaml(content: &str) -> ConfigResult<ConfigValue> {
        serde_yaml::from_str(content).map_err(|e| ConfigError::parse("YAML", e.to_string()))
    }

    fn serialize_json(value: &ConfigValue) -> ConfigResult<String> {
        serde_json::to_string_pretty(value)
            .map_err(|e| ConfigError::serialize("JSON", e.to_string()))
    }

    fn serialize_toml(value: &ConfigValue) -> ConfigResult<String> {
        toml::to_string_pretty(value).map_err(|e| ConfigError::serialize("TOML", e.to_string()))
    }

    fn serialize_yaml(value: &ConfigValue) -> ConfigResult<String> {
        serde_yaml::to_string(value).map_err(|e| ConfigError::serialize("YAML", e.to_string()))
    }
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
