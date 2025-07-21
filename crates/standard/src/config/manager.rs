//! Configuration manager implementation.
//!
//! This module provides a generic configuration manager that can handle any
//! type implementing the `Configurable` trait, with support for multiple
//! configuration sources and hierarchical loading.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::filesystem::AsyncFileSystem;

use super::error::{ConfigError, ConfigResult};
use super::format::ConfigFormat;
use super::source::{
    ConfigSource, ConfigSourcePriority, DefaultProvider, EnvironmentProvider, FileProvider,
    MemoryProvider,
};
use super::traits::{ConfigProvider, Configurable};
use super::value::ConfigValue;

/// Generic configuration manager for any Configurable type.
///
/// This manager handles loading configuration from multiple sources,
/// merging them according to priority, and caching the result.
///
/// # Type Parameters
///
/// * `T` - The configuration type, must implement `Configurable`
///
/// # Examples
///
/// ```no_run
/// use sublime_standard_tools::config::{ConfigManager, Configurable, ConfigResult};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyConfig {
///     name: String,
///     timeout: u64,
/// }
///
/// impl Configurable for MyConfig {
///     fn validate(&self) -> ConfigResult<()> {
///         if self.timeout == 0 {
///             return Err("Timeout must be greater than 0".into());
///         }
///         Ok(())
///     }
///
///     fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
///         if !other.name.is_empty() {
///             self.name = other.name;
///         }
///         if other.timeout > 0 {
///             self.timeout = other.timeout;
///         }
///         Ok(())
///     }
/// }
///
/// async fn example() -> ConfigResult<MyConfig> {
///     let manager = ConfigManager::<MyConfig>::builder()
///         .with_defaults()
///         .with_file("config.toml")
///         .with_env_prefix("MY_APP")
///         .build()?;
///     
///     manager.load().await
/// }
/// ```
pub struct ConfigManager<T: Configurable> {
    providers: Vec<Box<dyn ConfigProvider>>,
    cache: Arc<RwLock<Option<T>>>,
    _phantom: PhantomData<T>,
}

impl<T: Configurable + Clone> ConfigManager<T> {
    /// Creates a new configuration builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigManager;
    /// use sublime_standard_tools::config::standard::StandardConfig;
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder();
    /// ```
    pub fn builder() -> ConfigBuilder<T> {
        ConfigBuilder::new()
    }

    /// Creates a new configuration manager with the given providers.
    ///
    /// # Arguments
    ///
    /// * `providers` - Configuration providers sorted by priority
    fn new(providers: Vec<Box<dyn ConfigProvider>>) -> Self {
        Self { providers, cache: Arc::new(RwLock::new(None)), _phantom: PhantomData }
    }

    /// Loads configuration from all sources and merges them.
    ///
    /// Sources are loaded in priority order, with higher priority sources
    /// overriding values from lower priority ones.
    ///
    /// # Returns
    ///
    /// The loaded and merged configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Loading from any required source fails
    /// - Deserialization fails
    /// - Validation fails
    /// - Merging fails
    pub async fn load(&self) -> ConfigResult<T> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(config) = cache.as_ref() {
                return Ok(config.clone());
            }
        }

        // Load from all providers
        let mut merged_value = ConfigValue::Map(HashMap::default());

        for provider in &self.providers {
            match provider.load().await {
                Ok(value) => {
                    merged_value.merge(value);
                }
                Err(e) => {
                    // Log error but continue with other providers
                    log::warn!("Failed to load from provider '{}': {}", provider.name(), e);
                }
            }
        }

        // Deserialize the merged value
        let config: T = serde_json::from_value(serde_json::to_value(&merged_value)?)
            .map_err(|e| ConfigError::parse("configuration", e.to_string()))?;

        // Validate the configuration
        config.validate()?;

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            *cache = Some(config.clone());
        }

        Ok(config)
    }

    /// Loads configuration from all sources without using cache.
    ///
    /// This forces a fresh load from all sources.
    ///
    /// # Returns
    ///
    /// The loaded and merged configuration.
    ///
    /// # Errors
    ///
    /// Same as `load()`.
    pub async fn reload(&self) -> ConfigResult<T> {
        // Clear cache
        {
            let mut cache = self.cache.write().await;
            *cache = None;
        }

        self.load().await
    }

    /// Saves the configuration to all providers that support saving.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to save
    ///
    /// # Errors
    ///
    /// Returns an error if saving to any provider fails.
    pub async fn save(&self, config: &T) -> ConfigResult<()> {
        // Validate before saving
        config.validate()?;

        // Convert to ConfigValue
        let value = serde_json::to_value(config)?;
        let config_value: ConfigValue = serde_json::from_value(value)?;

        // Save to all providers that support it
        for provider in &self.providers {
            if provider.supports_save() {
                provider
                    .save(&config_value)
                    .await
                    .map_err(|e| ConfigError::provider(provider.name(), e.to_string()))?;
            }
        }

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(config.clone());
        }

        Ok(())
    }

    /// Gets a value by key path from the cached configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - Dot-separated key path (e.g., "database.host")
    ///
    /// # Returns
    ///
    /// The value at the given key path, if it exists and can be deserialized.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No configuration is loaded
    /// - The key doesn't exist
    /// - Deserialization fails
    pub async fn get<V: DeserializeOwned>(&self, key: &str) -> ConfigResult<Option<V>> {
        let cache = self.cache.read().await;
        let config = cache.as_ref().ok_or_else(|| ConfigError::other("No configuration loaded"))?;

        // Convert to ConfigValue to access by key
        let value = serde_json::to_value(config)?;
        let config_value: ConfigValue = serde_json::from_value(value)?;

        // Navigate to the key
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &config_value;

        for part in parts {
            match current.get(part) {
                Some(v) => current = v,
                None => return Ok(None),
            }
        }

        // Deserialize the value
        let result: V = serde_json::from_value(serde_json::to_value(current)?)
            .map_err(|e| ConfigError::parse("value", e.to_string()))?;

        Ok(Some(result))
    }

    /// Sets a value by key path and saves the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - Dot-separated key path (e.g., "database.host")
    /// * `value` - The value to set
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No configuration is loaded
    /// - Serialization fails
    /// - Validation fails
    /// - Saving fails
    pub async fn set<V: Serialize>(&self, key: &str, value: V) -> ConfigResult<()> {
        let mut cache = self.cache.write().await;
        let config = cache.as_mut().ok_or_else(|| ConfigError::other("No configuration loaded"))?;

        // Convert to ConfigValue for manipulation
        let mut config_value: ConfigValue =
            serde_json::from_value(serde_json::to_value(&*config)?)?;

        // Navigate to the key and set the value
        let parts: Vec<&str> = key.split('.').collect();
        let new_value: ConfigValue = serde_json::from_value(serde_json::to_value(&value)?)?;

        // Handle nested keys
        let mut current = &mut config_value;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let ConfigValue::Map(map) = current {
                    map.insert((*part).to_string(), new_value.clone());
                } else {
                    return Err(ConfigError::type_error("map", current.type_name()));
                }
            } else {
                // Navigate deeper, creating maps as needed
                if let ConfigValue::Map(map) = current {
                    let entry = map
                        .entry((*part).to_string())
                        .or_insert(ConfigValue::Map(HashMap::default()));
                    current = entry;
                } else {
                    return Err(ConfigError::type_error("map", current.type_name()));
                }
            }
        }

        // Convert back to T
        *config = serde_json::from_value(serde_json::to_value(&config_value)?)
            .map_err(|e| ConfigError::parse("configuration", e.to_string()))?;

        // Validate the new configuration
        config.validate()?;

        // Save is handled separately by the caller if needed
        Ok(())
    }

    /// Merges another configuration into the current one.
    ///
    /// # Arguments
    ///
    /// * `other` - The configuration to merge
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No configuration is loaded
    /// - Merging fails
    /// - Validation fails
    pub async fn merge(&self, other: T) -> ConfigResult<()> {
        let mut cache = self.cache.write().await;
        let config = cache.as_mut().ok_or_else(|| ConfigError::other("No configuration loaded"))?;

        config.merge_with(other)?;
        config.validate()?;

        Ok(())
    }

    /// Validates the current configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No configuration is loaded
    /// - Validation fails
    pub async fn validate(&self) -> ConfigResult<()> {
        let cache = self.cache.read().await;
        let config = cache.as_ref().ok_or_else(|| ConfigError::other("No configuration loaded"))?;

        config.validate()
    }

    /// Clears the configuration cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}

/// Builder for creating a ConfigManager.
///
/// This builder provides a fluent API for configuring the sources
/// from which configuration should be loaded.
pub struct ConfigBuilder<T: Configurable + Clone> {
    sources: Vec<ConfigSource>,
    _phantom: PhantomData<T>,
}

impl<T: Configurable + Clone> ConfigBuilder<T> {
    /// Creates a new configuration builder.
    fn new() -> Self {
        Self { sources: Vec::new(), _phantom: PhantomData }
    }

    /// Adds default configuration values.
    ///
    /// This uses the `Configurable::default_values()` method if available.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigManager;
    /// use sublime_standard_tools::config::standard::StandardConfig;
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder()
    ///     .with_defaults();
    /// ```
    #[must_use]
    pub fn with_defaults(mut self) -> Self
    where
        T: Default,
    {
        if let Some(defaults) = T::default_values() {
            let value = serde_json::to_value(defaults)
                .ok()
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or(ConfigValue::Map(HashMap::default()));

            self.sources.push(ConfigSource::defaults(value));
        }
        self
    }

    /// Adds a configuration file source.
    ///
    /// The format is auto-detected from the file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigManager;
    /// use sublime_standard_tools::config::standard::StandardConfig;
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder()
    ///     .with_file("config.toml");
    /// ```
    #[must_use]
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let priority = if path.to_string_lossy().contains(".sublime") {
            ConfigSourcePriority::Project
        } else {
            ConfigSourcePriority::Global
        };

        self.sources.push(ConfigSource::file(path, priority));
        self
    }

    /// Adds a configuration file source with a specific priority.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `priority` - Priority of this source
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigSourcePriority};
    /// use sublime_standard_tools::config::standard::StandardConfig;
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder()
    ///     .with_file_priority("~/.config/sublime/config.toml", ConfigSourcePriority::Global);
    /// ```
    #[must_use]
    pub fn with_file_priority(
        mut self,
        path: impl AsRef<Path>,
        priority: ConfigSourcePriority,
    ) -> Self {
        self.sources.push(ConfigSource::file(path, priority));
        self
    }

    /// Adds an environment variable source.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for environment variables (e.g., "SUBLIME")
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::ConfigManager;
    /// use sublime_standard_tools::config::standard::StandardConfig;
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder()
    ///     .with_env_prefix("SUBLIME");
    /// ```
    #[must_use]
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.sources.push(ConfigSource::environment(prefix, ConfigSourcePriority::Environment));
        self
    }

    /// Adds a custom configuration source.
    ///
    /// # Arguments
    ///
    /// * `source` - The configuration source to add
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_standard_tools::config::{ConfigManager, ConfigSource, ConfigValue, ConfigSourcePriority};
    /// use sublime_standard_tools::config::standard::StandardConfig;
    /// use std::collections::HashMap;
    ///
    /// let mut runtime_config = HashMap::new();
    /// runtime_config.insert("debug".to_string(), ConfigValue::Boolean(true));
    ///
    /// let builder = ConfigManager::<StandardConfig>::builder()
    ///     .with_source(ConfigSource::memory(runtime_config, ConfigSourcePriority::Runtime));
    /// ```
    #[must_use]
    pub fn with_source(mut self, source: ConfigSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Builds the configuration manager.
    ///
    /// This method requires an async filesystem implementation to be available.
    ///
    /// # Arguments
    ///
    /// * `fs` - The async filesystem implementation to use
    ///
    /// # Returns
    ///
    /// A configured `ConfigManager` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if provider creation fails.
    #[allow(clippy::needless_pass_by_value)] // fs needs to be cloned for multiple providers
    pub fn build<FS: AsyncFileSystem + Clone + 'static>(
        self,
        fs: FS,
    ) -> ConfigResult<ConfigManager<T>> {
        let mut providers: Vec<Box<dyn ConfigProvider>> = Vec::new();

        // Sort sources by priority
        let mut sources = self.sources;
        sources.sort_by_key(ConfigSource::priority);

        // Create providers for each source
        for source in sources {
            match source {
                ConfigSource::File { path, format, priority } => {
                    let fmt =
                        format.or_else(|| ConfigFormat::from_path(&path)).ok_or_else(|| {
                            ConfigError::other(format!(
                                "Cannot determine format for file: {}",
                                path.display()
                            ))
                        })?;

                    providers.push(Box::new(FileProvider::new(path, fmt, priority, fs.clone())));
                }
                ConfigSource::Environment { prefix, priority } => {
                    providers.push(Box::new(EnvironmentProvider::new(prefix, priority)));
                }
                ConfigSource::Default { values, priority } => {
                    providers.push(Box::new(DefaultProvider::new(values, priority)));
                }
                ConfigSource::Memory { values, priority } => {
                    providers.push(Box::new(MemoryProvider::new(values, priority)));
                }
            }
        }

        Ok(ConfigManager::new(providers))
    }
}

impl<T: Configurable + Clone> Default for ConfigBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
