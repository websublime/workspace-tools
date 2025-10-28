//! Configuration sources.
//!
//! This module defines different sources from which configuration can be loaded,
//! such as files, environment variables, and defaults.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{ConfigError, ConfigResult};
use crate::filesystem::AsyncFileSystem;

use super::format::ConfigFormat;
use super::traits::ConfigProvider;
use super::value::ConfigValue;

/// Priority levels for configuration sources.
///
/// Higher priority sources override lower priority ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigSourcePriority {
    /// Built-in default values (lowest priority)
    Default = 0,
    /// System-wide configuration
    Global = 1,
    /// Project-specific configuration
    Project = 2,
    /// Environment variables
    Environment = 3,
    /// Runtime programmatic overrides (highest priority)
    Runtime = 4,
}

/// Different sources of configuration data.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Configuration from a file
    File {
        /// Path to the configuration file
        path: PathBuf,
        /// Format of the file (auto-detected if None)
        format: Option<ConfigFormat>,
        /// Priority of this source
        priority: ConfigSourcePriority,
    },
    /// Configuration from environment variables
    Environment {
        /// Prefix for environment variables (e.g., "SUBLIME_")
        prefix: String,
        /// Priority of this source
        priority: ConfigSourcePriority,
    },
    /// Default configuration values
    Default {
        /// The default values
        values: ConfigValue,
        /// Priority of this source
        priority: ConfigSourcePriority,
    },
    /// In-memory configuration
    Memory {
        /// The configuration values
        values: HashMap<String, ConfigValue>,
        /// Priority of this source
        priority: ConfigSourcePriority,
    },
}

impl ConfigSource {
    /// Creates a new file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `priority` - Priority of this source
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigSource, ConfigSourcePriority};
    ///
    /// let source = ConfigSource::file("config.toml", ConfigSourcePriority::Project);
    /// ```
    pub fn file(path: impl AsRef<Path>, priority: ConfigSourcePriority) -> Self {
        Self::File { path: path.as_ref().to_path_buf(), format: None, priority }
    }

    /// Creates a new file configuration source with a specific format.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `format` - The format of the file
    /// * `priority` - Priority of this source
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigSource, ConfigFormat, ConfigSourcePriority};
    ///
    /// let source = ConfigSource::file_with_format(
    ///     "config.custom",
    ///     ConfigFormat::Toml,
    ///     ConfigSourcePriority::Project
    /// );
    /// ```
    pub fn file_with_format(
        path: impl AsRef<Path>,
        format: ConfigFormat,
        priority: ConfigSourcePriority,
    ) -> Self {
        Self::File { path: path.as_ref().to_path_buf(), format: Some(format), priority }
    }

    /// Creates a new environment variable configuration source.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for environment variables
    /// * `priority` - Priority of this source
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigSource, ConfigSourcePriority};
    ///
    /// let source = ConfigSource::environment("SUBLIME", ConfigSourcePriority::Environment);
    /// ```
    pub fn environment(prefix: impl Into<String>, priority: ConfigSourcePriority) -> Self {
        Self::Environment { prefix: prefix.into(), priority }
    }

    /// Creates a new default configuration source.
    ///
    /// # Arguments
    ///
    /// * `values` - The default configuration values
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigSource, ConfigValue};
    /// use std::collections::HashMap;
    ///
    /// let mut defaults = HashMap::new();
    /// defaults.insert("timeout".to_string(), ConfigValue::Integer(30));
    /// let source = ConfigSource::defaults(ConfigValue::Map(defaults));
    /// ```
    pub fn defaults(values: ConfigValue) -> Self {
        Self::Default { values, priority: ConfigSourcePriority::Default }
    }

    /// Creates a new in-memory configuration source.
    ///
    /// # Arguments
    ///
    /// * `values` - The configuration values
    /// * `priority` - Priority of this source
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigSource, ConfigValue, ConfigSourcePriority};
    /// use std::collections::HashMap;
    ///
    /// let mut values = HashMap::new();
    /// values.insert("key".to_string(), ConfigValue::String("value".to_string()));
    /// let source = ConfigSource::memory(values, ConfigSourcePriority::Runtime);
    /// ```
    pub fn memory(values: HashMap<String, ConfigValue>, priority: ConfigSourcePriority) -> Self {
        Self::Memory { values, priority }
    }

    /// Gets the priority of this source.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Each variant has its own priority field
    pub fn priority(&self) -> ConfigSourcePriority {
        match self {
            Self::File { priority, .. } => *priority,
            Self::Environment { priority, .. } => *priority,
            Self::Default { priority, .. } => *priority,
            Self::Memory { priority, .. } => *priority,
        }
    }
}

/// File-based configuration provider.
pub struct FileProvider<FS: AsyncFileSystem> {
    path: PathBuf,
    format: ConfigFormat,
    priority: ConfigSourcePriority,
    fs: FS,
}

impl<FS: AsyncFileSystem> FileProvider<FS> {
    /// Creates a new file provider.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `format` - Format of the file
    /// * `priority` - Priority of this provider
    /// * `fs` - Filesystem to use
    pub fn new(
        path: PathBuf,
        format: ConfigFormat,
        priority: ConfigSourcePriority,
        fs: FS,
    ) -> Self {
        Self { path, format, priority, fs }
    }
}

#[async_trait]
impl<FS: AsyncFileSystem> ConfigProvider for FileProvider<FS> {
    async fn load(&self) -> ConfigResult<ConfigValue> {
        let content = self.fs.read_file_string(&self.path).await.map_err(|e| {
            ConfigError::FileReadError { path: self.path.clone(), message: e.to_string() }
        })?;

        self.format.parse(&content)
    }

    async fn save(&self, value: &ConfigValue) -> ConfigResult<()> {
        let content = self.format.serialize(value)?;
        self.fs.write_file(&self.path, content.as_bytes()).await.map_err(|e| {
            ConfigError::FileWriteError { path: self.path.clone(), message: e.to_string() }
        })
    }

    fn name(&self) -> &str {
        self.path.to_str().unwrap_or("file")
    }

    fn priority(&self) -> i32 {
        self.priority as i32
    }
}

/// Environment variable configuration provider.
pub struct EnvironmentProvider {
    prefix: String,
    priority: ConfigSourcePriority,
}

impl EnvironmentProvider {
    /// Creates a new environment provider.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for environment variables
    /// * `priority` - Priority of this provider
    pub fn new(prefix: String, priority: ConfigSourcePriority) -> Self {
        Self { prefix, priority }
    }

    /// Inserts a value into a nested structure.
    fn insert_nested_value(
        map: &mut HashMap<String, ConfigValue>,
        parts: &[&str],
        value: ConfigValue,
    ) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
            return;
        }

        let key = parts[0].to_string();
        let rest = &parts[1..];

        let entry = map.entry(key).or_insert_with(|| ConfigValue::Map(HashMap::new()));

        if let ConfigValue::Map(nested_map) = entry {
            Self::insert_nested_value(nested_map, rest, value);
        }
    }
}

#[async_trait]
impl ConfigProvider for EnvironmentProvider {
    async fn load(&self) -> ConfigResult<ConfigValue> {
        let mut map = HashMap::new();
        let prefix_with_underscore = format!("{}_", self.prefix);

        for (key, value) in std::env::vars() {
            if key.starts_with(&prefix_with_underscore) {
                let config_key =
                    key[prefix_with_underscore.len()..].to_lowercase().replace('_', ".");

                // Try to parse as different types
                let config_value = if let Ok(b) = value.parse::<bool>() {
                    ConfigValue::Boolean(b)
                } else if let Ok(i) = value.parse::<i64>() {
                    ConfigValue::Integer(i)
                } else if let Ok(f) = value.parse::<f64>() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::String(value)
                };

                // Handle nested keys (e.g., "database.host" -> { database: { host: ... } })
                let parts: Vec<&str> = config_key.split('.').collect();
                Self::insert_nested_value(&mut map, &parts, config_value);
            }
        }

        Ok(ConfigValue::Map(map))
    }

    async fn save(&self, _value: &ConfigValue) -> ConfigResult<()> {
        // Environment variables cannot be saved
        Ok(())
    }

    fn name(&self) -> &'static str {
        "environment"
    }

    fn supports_save(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        self.priority as i32
    }
}

/// Default values configuration provider.
pub struct DefaultProvider {
    values: ConfigValue,
    priority: ConfigSourcePriority,
}

impl DefaultProvider {
    /// Creates a new default provider.
    ///
    /// # Arguments
    ///
    /// * `values` - The default values
    /// * `priority` - Priority of this provider
    pub fn new(values: ConfigValue, priority: ConfigSourcePriority) -> Self {
        Self { values, priority }
    }
}

#[async_trait]
impl ConfigProvider for DefaultProvider {
    async fn load(&self) -> ConfigResult<ConfigValue> {
        Ok(self.values.clone())
    }

    async fn save(&self, _value: &ConfigValue) -> ConfigResult<()> {
        // Default values cannot be saved
        Ok(())
    }

    fn name(&self) -> &'static str {
        "defaults"
    }

    fn supports_save(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        self.priority as i32
    }
}

/// In-memory configuration provider.
pub struct MemoryProvider {
    values: HashMap<String, ConfigValue>,
    priority: ConfigSourcePriority,
}

impl MemoryProvider {
    /// Creates a new memory provider.
    ///
    /// # Arguments
    ///
    /// * `values` - The configuration values
    /// * `priority` - Priority of this provider
    pub fn new(values: HashMap<String, ConfigValue>, priority: ConfigSourcePriority) -> Self {
        Self { values, priority }
    }
}

#[async_trait]
impl ConfigProvider for MemoryProvider {
    async fn load(&self) -> ConfigResult<ConfigValue> {
        Ok(ConfigValue::Map(self.values.clone()))
    }

    async fn save(&self, _value: &ConfigValue) -> ConfigResult<()> {
        // Memory provider doesn't persist
        Ok(())
    }

    fn name(&self) -> &'static str {
        "memory"
    }

    fn priority(&self) -> i32 {
        self.priority as i32
    }
}
