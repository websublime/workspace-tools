//! Core traits for the configuration system.
//!
//! This module defines the fundamental traits that enable a generic and extensible
//! configuration framework. Types implementing these traits can be used with the
//! generic ConfigManager to provide configuration capabilities.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::ConfigResult;
use super::value::ConfigValue;

/// Trait for types that can be configured.
///
/// This trait must be implemented by any type that wants to be managed by the
/// configuration system. It provides methods for validation and merging.
///
/// # Example
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use sublime_standard_tools::config::{Configurable, ConfigResult};
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
/// ```
pub trait Configurable: Serialize + DeserializeOwned + Send + Sync {
    /// Validates the configuration.
    ///
    /// This method should check that all values are within acceptable ranges
    /// and that the configuration is internally consistent.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    fn validate(&self) -> ConfigResult<()>;

    /// Merges another configuration into this one.
    ///
    /// This method should implement the merging logic for the configuration type.
    /// Typically, values from `other` should override values in `self`, but the
    /// exact behavior depends on the specific configuration type.
    ///
    /// # Arguments
    ///
    /// * `other` - The configuration to merge into this one
    ///
    /// # Errors
    ///
    /// Returns an error if the merge operation fails.
    fn merge_with(&mut self, other: Self) -> ConfigResult<()>;

    /// Provides default values for the configuration.
    ///
    /// This method has a default implementation that uses the `Default` trait if
    /// available, but can be overridden for custom default behavior.
    fn default_values() -> Option<Self>
    where
        Self: Default,
    {
        Some(Self::default())
    }
}

/// Trait for configuration providers.
///
/// Configuration providers are sources of configuration data. They can load
/// configuration from files, environment variables, or any other source.
///
/// # Example
///
/// ```
/// use async_trait::async_trait;
/// use sublime_standard_tools::config::{ConfigProvider, ConfigResult, ConfigValue};
/// use std::collections::HashMap;
///
/// struct MemoryProvider {
///     data: HashMap<String, ConfigValue>,
/// }
///
/// #[async_trait]
/// impl ConfigProvider for MemoryProvider {
///     async fn load(&self) -> ConfigResult<ConfigValue> {
///         Ok(ConfigValue::Map(self.data.clone()))
///     }
///
///     async fn save(&self, _value: &ConfigValue) -> ConfigResult<()> {
///         // Memory provider doesn't persist
///         Ok(())
///     }
///
///     fn name(&self) -> &str {
///         "memory"
///     }
/// }
/// ```
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Loads configuration data from this provider.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded.
    async fn load(&self) -> ConfigResult<ConfigValue>;

    /// Saves configuration data to this provider.
    ///
    /// Not all providers support saving (e.g., environment variables).
    /// Those that don't should return Ok(()) or an appropriate error.
    ///
    /// # Arguments
    ///
    /// * `value` - The configuration value to save
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    async fn save(&self, value: &ConfigValue) -> ConfigResult<()>;

    /// Returns a human-readable name for this provider.
    ///
    /// This is used for debugging and error messages.
    fn name(&self) -> &str;

    /// Returns whether this provider supports saving.
    ///
    /// Default implementation returns true, but providers that are read-only
    /// (like environment variables) should override this to return false.
    fn supports_save(&self) -> bool {
        true
    }

    /// Returns the priority of this provider.
    ///
    /// Used to determine the order in which providers are consulted.
    /// Higher priority providers override lower priority ones.
    fn priority(&self) -> i32 {
        0
    }
}
