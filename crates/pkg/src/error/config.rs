//! # Configuration error types and implementations
//!
//! ## What
//! This module provides error types specific to configuration operations,
//! including validation, parsing, and environment-specific configuration issues.
//!
//! ## How
//! Provides detailed error types for configuration-related failures with specific
//! context for different configuration domains and validation rules.
//!
//! ## Why
//! Configuration management is fundamental to package tools operation and
//! requires precise error handling to provide clear feedback about invalid
//! settings, missing configurations, and environment setup issues.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for configuration operations.
///
/// This is a convenience type alias for Results with `ConfigError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{ConfigResult, ConfigError};
///
/// fn validate_registry_config(url: &str) -> ConfigResult<()> {
///     if url.is_empty() {
///         return Err(ConfigError::InvalidRegistryConfig {
///             registry: "default".to_string(),
///             reason: "Empty registry URL".to_string(),
///         });
///     }
///     Ok(())
/// }
/// ```
pub type ConfigResult<T> = StdResult<T, ConfigError>;

/// Configuration-related error types.
///
/// Handles errors in configuration validation and processing including
/// package configuration, environment setup, and registry configuration.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::ConfigError;
///
/// let error = ConfigError::InvalidPackageConfig {
///     field: "version_strategy".to_string(),
///     reason: "Unknown strategy 'custom'".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Invalid package tools configuration for field 'version_strategy': Unknown strategy 'custom'
/// ```
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    /// Invalid package tools configuration
    #[error("Invalid package tools configuration for field '{field}': {reason}")]
    InvalidPackageConfig {
        /// Configuration field name
        field: String,
        /// Reason why configuration is invalid
        reason: String,
    },

    /// Invalid environment configuration
    #[error("Invalid environment configuration for '{environment}': {reason}")]
    InvalidEnvironmentConfig {
        /// Environment name
        environment: String,
        /// Reason why environment configuration is invalid
        reason: String,
    },

    /// Invalid registry configuration
    #[error("Invalid registry configuration for '{registry}': {reason}")]
    InvalidRegistryConfig {
        /// Registry name or URL
        registry: String,
        /// Reason why registry configuration is invalid
        reason: String,
    },

    /// Invalid version strategy configuration
    #[error("Invalid version strategy '{strategy}': {reason}")]
    InvalidVersionStrategy {
        /// Version strategy name
        strategy: String,
        /// Reason why strategy is invalid
        reason: String,
    },
}

impl ConfigError {
    /// Creates an invalid package config error.
    ///
    /// # Arguments
    ///
    /// * `field` - Configuration field name
    /// * `reason` - Why configuration is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_package_config(
    ///     "changeset.path",
    ///     "Path must be relative"
    /// );
    /// assert!(error.to_string().contains("Invalid package tools configuration"));
    /// ```
    #[must_use]
    pub fn invalid_package_config(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPackageConfig { field: field.into(), reason: reason.into() }
    }

    /// Creates an invalid environment config error.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    /// * `reason` - Why environment configuration is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_environment_config(
    ///     "production",
    ///     "Missing registry URL"
    /// );
    /// assert!(error.to_string().contains("Invalid environment configuration"));
    /// ```
    #[must_use]
    pub fn invalid_environment_config(
        environment: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidEnvironmentConfig { environment: environment.into(), reason: reason.into() }
    }

    /// Creates an invalid registry config error.
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry name or URL
    /// * `reason` - Why registry configuration is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_registry_config(
    ///     "https://registry.npmjs.org",
    ///     "Invalid authentication method"
    /// );
    /// assert!(error.to_string().contains("Invalid registry configuration"));
    /// ```
    #[must_use]
    pub fn invalid_registry_config(registry: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidRegistryConfig { registry: registry.into(), reason: reason.into() }
    }

    /// Creates an invalid version strategy error.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Version strategy name
    /// * `reason` - Why strategy is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_version_strategy(
    ///     "custom-strategy",
    ///     "Strategy not implemented"
    /// );
    /// assert!(error.to_string().contains("Invalid version strategy"));
    /// ```
    #[must_use]
    pub fn invalid_version_strategy(
        strategy: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidVersionStrategy { strategy: strategy.into(), reason: reason.into() }
    }

    /// Checks if this is a package configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_package_config("field", "reason");
    /// assert!(error.is_package_config_error());
    ///
    /// let error = ConfigError::invalid_environment_config("env", "reason");
    /// assert!(!error.is_package_config_error());
    /// ```
    #[must_use]
    pub fn is_package_config_error(&self) -> bool {
        matches!(self, Self::InvalidPackageConfig { .. })
    }

    /// Checks if this is an environment configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_environment_config("production", "reason");
    /// assert!(error.is_environment_config_error());
    ///
    /// let error = ConfigError::invalid_registry_config("registry", "reason");
    /// assert!(!error.is_environment_config_error());
    /// ```
    #[must_use]
    pub fn is_environment_config_error(&self) -> bool {
        matches!(self, Self::InvalidEnvironmentConfig { .. })
    }

    /// Checks if this is a registry configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_registry_config("registry", "reason");
    /// assert!(error.is_registry_config_error());
    ///
    /// let error = ConfigError::invalid_version_strategy("strategy", "reason");
    /// assert!(!error.is_registry_config_error());
    /// ```
    #[must_use]
    pub fn is_registry_config_error(&self) -> bool {
        matches!(self, Self::InvalidRegistryConfig { .. })
    }

    /// Checks if this is a strategy configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_version_strategy("strategy", "reason");
    /// assert!(error.is_strategy_config_error());
    ///
    /// let error = ConfigError::invalid_package_config("field", "reason");
    /// assert!(!error.is_strategy_config_error());
    /// ```
    #[must_use]
    pub fn is_strategy_config_error(&self) -> bool {
        matches!(self, Self::InvalidVersionStrategy { .. })
    }

    /// Gets the field name from package config errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_package_config("changeset.path", "reason");
    /// assert_eq!(error.field_name(), Some("changeset.path"));
    ///
    /// let error = ConfigError::invalid_environment_config("env", "reason");
    /// assert_eq!(error.field_name(), None);
    /// ```
    #[must_use]
    pub fn field_name(&self) -> Option<&str> {
        match self {
            Self::InvalidPackageConfig { field, .. } => Some(field),
            _ => None,
        }
    }

    /// Gets the environment name from environment config errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_environment_config("production", "reason");
    /// assert_eq!(error.environment_name(), Some("production"));
    ///
    /// let error = ConfigError::invalid_registry_config("registry", "reason");
    /// assert_eq!(error.environment_name(), None);
    /// ```
    #[must_use]
    pub fn environment_name(&self) -> Option<&str> {
        match self {
            Self::InvalidEnvironmentConfig { environment, .. } => Some(environment),
            _ => None,
        }
    }

    /// Gets the registry name from registry config errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_registry_config("npm-registry", "reason");
    /// assert_eq!(error.registry_name(), Some("npm-registry"));
    ///
    /// let error = ConfigError::invalid_version_strategy("strategy", "reason");
    /// assert_eq!(error.registry_name(), None);
    /// ```
    #[must_use]
    pub fn registry_name(&self) -> Option<&str> {
        match self {
            Self::InvalidRegistryConfig { registry, .. } => Some(registry),
            _ => None,
        }
    }

    /// Gets the strategy name from strategy config errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_version_strategy("independent", "reason");
    /// assert_eq!(error.strategy_name(), Some("independent"));
    ///
    /// let error = ConfigError::invalid_package_config("field", "reason");
    /// assert_eq!(error.strategy_name(), None);
    /// ```
    #[must_use]
    pub fn strategy_name(&self) -> Option<&str> {
        match self {
            Self::InvalidVersionStrategy { strategy, .. } => Some(strategy),
            _ => None,
        }
    }

    /// Gets the reason from all error variants.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::ConfigError;
    ///
    /// let error = ConfigError::invalid_package_config("field", "Invalid value");
    /// assert_eq!(error.reason(), "Invalid value");
    ///
    /// let error = ConfigError::invalid_environment_config("env", "Missing URL");
    /// assert_eq!(error.reason(), "Missing URL");
    /// ```
    #[must_use]
    pub fn reason(&self) -> &str {
        match self {
            Self::InvalidPackageConfig { reason, .. }
            | Self::InvalidEnvironmentConfig { reason, .. }
            | Self::InvalidRegistryConfig { reason, .. }
            | Self::InvalidVersionStrategy { reason, .. } => reason,
        }
    }
}

impl AsRef<str> for ConfigError {
    fn as_ref(&self) -> &str {
        match self {
            ConfigError::InvalidPackageConfig { .. } => "ConfigError::InvalidPackageConfig",
            ConfigError::InvalidEnvironmentConfig { .. } => "ConfigError::InvalidEnvironmentConfig",
            ConfigError::InvalidRegistryConfig { .. } => "ConfigError::InvalidRegistryConfig",
            ConfigError::InvalidVersionStrategy { .. } => "ConfigError::InvalidVersionStrategy",
        }
    }
}
