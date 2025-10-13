//! # Registry error types and implementations
//!
//! ## What
//! This module provides error types specific to registry operations,
//! including authentication, publishing, fetching, and network-related failures.
//!
//! ## How
//! Provides detailed error types for registry-related failures with specific
//! context for different registry operations and network conditions.
//!
//! ## Why
//! Registry operations are critical for package publishing and management,
//! requiring precise error handling to provide clear feedback about network
//! issues, authentication failures, and publishing problems.

use std::result::Result as StdResult;
use thiserror::Error;

/// Result type for registry operations.
///
/// This is a convenience type alias for Results with `RegistryError`.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::{RegistryResult, RegistryError};
///
/// fn publish_package(name: &str) -> RegistryResult<()> {
///     if name.is_empty() {
///         return Err(RegistryError::InvalidConfig {
///             reason: "Package name cannot be empty".to_string(),
///         });
///     }
///     Ok(())
/// }
/// ```
pub type RegistryResult<T> = StdResult<T, RegistryError>;

/// Registry-related error types.
///
/// Handles errors in registry operations including authentication,
/// publishing, network failures, and configuration issues.
///
/// # Examples
///
/// ```
/// use sublime_pkg_tools::error::RegistryError;
///
/// let error = RegistryError::AuthenticationFailed {
///     registry: "https://registry.npmjs.org".to_string(),
///     reason: "Invalid API token".to_string(),
/// };
///
/// println!("Error: {}", error);
/// // Output: Registry authentication failed for 'https://registry.npmjs.org': Invalid API token
/// ```
#[derive(Error, Debug, Clone)]
pub enum RegistryError {
    /// Authentication failed for registry
    #[error("Registry authentication failed for '{registry}': {reason}")]
    AuthenticationFailed {
        /// Registry URL
        registry: String,
        /// Reason for authentication failure
        reason: String,
    },

    /// Package not found in registry
    #[error("Package '{package}' not found in registry '{registry}'")]
    PackageNotFound {
        /// Package name
        package: String,
        /// Registry URL
        registry: String,
    },

    /// Package publishing failed
    #[error("Failed to publish package '{package}' to registry '{registry}': {reason}")]
    PublishFailed {
        /// Package name
        package: String,
        /// Registry URL
        registry: String,
        /// Reason for publish failure
        reason: String,
    },

    /// Network operation failed
    #[error("Network operation failed for registry '{registry}': {reason}")]
    NetworkFailed {
        /// Registry URL
        registry: String,
        /// Reason for network failure
        reason: String,
    },

    /// Invalid registry configuration
    #[error("Invalid registry configuration: {reason}")]
    InvalidConfig {
        /// Reason for invalid configuration
        reason: String,
    },

    /// Version already exists in registry
    #[error("Version '{version}' of package '{package}' already exists in registry '{registry}'")]
    VersionAlreadyExists {
        /// Package name
        package: String,
        /// Version string
        version: String,
        /// Registry URL
        registry: String,
    },

    /// Registry operation timed out
    #[error("Registry operation timed out for '{registry}' after {timeout_ms}ms")]
    Timeout {
        /// Registry URL
        registry: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },
}

impl RegistryError {
    /// Creates an authentication failed error.
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry URL
    /// * `reason` - Why authentication failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::authentication_failed(
    ///     "https://registry.npmjs.org",
    ///     "Invalid token"
    /// );
    /// assert!(error.to_string().contains("authentication failed"));
    /// ```
    #[must_use]
    pub fn authentication_failed(registry: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed { registry: registry.into(), reason: reason.into() }
    }

    /// Creates a package not found error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `registry` - Registry URL
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::package_not_found(
    ///     "@myorg/missing-package",
    ///     "https://registry.npmjs.org"
    /// );
    /// assert!(error.to_string().contains("not found"));
    /// ```
    #[must_use]
    pub fn package_not_found(package: impl Into<String>, registry: impl Into<String>) -> Self {
        Self::PackageNotFound { package: package.into(), registry: registry.into() }
    }

    /// Creates a publish failed error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `registry` - Registry URL
    /// * `reason` - Why publishing failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::publish_failed(
    ///     "@myorg/my-package",
    ///     "https://registry.npmjs.org",
    ///     "Insufficient permissions"
    /// );
    /// assert!(error.to_string().contains("Failed to publish"));
    /// ```
    #[must_use]
    pub fn publish_failed(
        package: impl Into<String>,
        registry: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::PublishFailed {
            package: package.into(),
            registry: registry.into(),
            reason: reason.into(),
        }
    }

    /// Creates a network failed error.
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry URL
    /// * `reason` - Why network operation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::network_failed(
    ///     "https://registry.npmjs.org",
    ///     "Connection timeout"
    /// );
    /// assert!(error.to_string().contains("Network operation failed"));
    /// ```
    #[must_use]
    pub fn network_failed(registry: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::NetworkFailed { registry: registry.into(), reason: reason.into() }
    }

    /// Creates an invalid config error.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why the configuration is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::invalid_config("Missing registry URL");
    /// assert!(error.to_string().contains("Invalid registry configuration"));
    /// ```
    #[must_use]
    pub fn invalid_config(reason: impl Into<String>) -> Self {
        Self::InvalidConfig { reason: reason.into() }
    }

    /// Creates a version already exists error.
    ///
    /// # Arguments
    ///
    /// * `package` - Package name
    /// * `version` - Version string
    /// * `registry` - Registry URL
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::version_already_exists(
    ///     "@myorg/my-package",
    ///     "1.0.0",
    ///     "https://registry.npmjs.org"
    /// );
    /// assert!(error.to_string().contains("already exists"));
    /// ```
    #[must_use]
    pub fn version_already_exists(
        package: impl Into<String>,
        version: impl Into<String>,
        registry: impl Into<String>,
    ) -> Self {
        Self::VersionAlreadyExists {
            package: package.into(),
            version: version.into(),
            registry: registry.into(),
        }
    }

    /// Creates a timeout error.
    ///
    /// # Arguments
    ///
    /// * `registry` - Registry URL
    /// * `timeout_ms` - Timeout in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::timeout("https://registry.npmjs.org", 5000);
    /// assert!(error.to_string().contains("timed out"));
    /// ```
    #[must_use]
    pub fn timeout(registry: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout { registry: registry.into(), timeout_ms }
    }

    /// Checks if this is an authentication error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::authentication_failed("registry", "reason");
    /// assert!(error.is_authentication_error());
    ///
    /// let error = RegistryError::network_failed("registry", "reason");
    /// assert!(!error.is_authentication_error());
    /// ```
    #[must_use]
    pub fn is_authentication_error(&self) -> bool {
        matches!(self, Self::AuthenticationFailed { .. })
    }

    /// Checks if this is a network error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::network_failed("registry", "reason");
    /// assert!(error.is_network_error());
    ///
    /// let error = RegistryError::timeout("registry", 5000);
    /// assert!(error.is_network_error());
    /// ```
    #[must_use]
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::NetworkFailed { .. } | Self::Timeout { .. })
    }

    /// Checks if this is a publishing error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::publish_failed("pkg", "registry", "reason");
    /// assert!(error.is_publishing_error());
    ///
    /// let error = RegistryError::version_already_exists("pkg", "1.0.0", "registry");
    /// assert!(error.is_publishing_error());
    /// ```
    #[must_use]
    pub fn is_publishing_error(&self) -> bool {
        matches!(self, Self::PublishFailed { .. } | Self::VersionAlreadyExists { .. })
    }

    /// Gets the registry URL from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::network_failed("https://registry.npmjs.org", "reason");
    /// assert_eq!(error.registry_url(), Some("https://registry.npmjs.org"));
    ///
    /// let error = RegistryError::invalid_config("reason");
    /// assert_eq!(error.registry_url(), None);
    /// ```
    #[must_use]
    pub fn registry_url(&self) -> Option<&str> {
        match self {
            Self::AuthenticationFailed { registry, .. }
            | Self::PackageNotFound { registry, .. }
            | Self::PublishFailed { registry, .. }
            | Self::NetworkFailed { registry, .. }
            | Self::VersionAlreadyExists { registry, .. }
            | Self::Timeout { registry, .. } => Some(registry),
            Self::InvalidConfig { .. } => None,
        }
    }

    /// Gets the package name from errors that include it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::error::RegistryError;
    ///
    /// let error = RegistryError::package_not_found("@myorg/pkg", "registry");
    /// assert_eq!(error.package_name(), Some("@myorg/pkg"));
    ///
    /// let error = RegistryError::network_failed("registry", "reason");
    /// assert_eq!(error.package_name(), None);
    /// ```
    #[must_use]
    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::PackageNotFound { package, .. }
            | Self::PublishFailed { package, .. }
            | Self::VersionAlreadyExists { package, .. } => Some(package),
            Self::AuthenticationFailed { .. }
            | Self::NetworkFailed { .. }
            | Self::InvalidConfig { .. }
            | Self::Timeout { .. } => None,
        }
    }
}
