//! Version configuration for versioning strategy and options.
//!
//! **What**: Defines configuration for version resolution strategies, default bump types,
//! and snapshot version formatting.
//!
//! **How**: This module provides the `VersionConfig` structure that controls how versions
//! are calculated and applied, using the `VersioningStrategy` type from the types module.
//!
//! **Why**: To enable flexible versioning that supports both monorepo and single-package
//! projects, with clear control over version resolution behavior.

use crate::types::VersioningStrategy;
use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for version management.
///
/// This structure controls the versioning strategy (independent vs unified), default
/// version bump type, and snapshot version formatting.
///
/// # Fields
///
/// - `strategy`: The versioning strategy to use (independent or unified)
/// - `default_bump`: Default version bump when none is specified in changeset
/// - `snapshot_format`: Format template for snapshot versions
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::{VersionConfig, VersioningStrategy};
///
/// let config = VersionConfig::default();
/// assert_eq!(config.strategy, VersioningStrategy::Independent);
/// assert_eq!(config.default_bump, "patch");
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.version]
/// strategy = "independent"
/// default_bump = "patch"
/// snapshot_format = "{version}-{branch}.{timestamp}"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionConfig {
    /// The versioning strategy to use.
    ///
    /// Determines whether packages are versioned independently or with a unified version.
    ///
    /// # Default
    ///
    /// `VersioningStrategy::Independent`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::{VersionConfig, VersioningStrategy};
    ///
    /// let config = VersionConfig {
    ///     strategy: VersioningStrategy::Unified,
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.strategy, VersioningStrategy::Unified);
    /// ```
    pub strategy: VersioningStrategy,

    /// Default version bump type when not specified in changeset.
    ///
    /// Valid values are: "major", "minor", "patch", "none"
    ///
    /// # Default
    ///
    /// `"patch"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::VersionConfig;
    ///
    /// let config = VersionConfig {
    ///     default_bump: "minor".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.default_bump, "minor");
    /// ```
    pub default_bump: String,

    /// Format template for snapshot versions.
    ///
    /// Supports the following placeholders:
    /// - `{version}`: The base version number
    /// - `{branch}`: The current git branch name
    /// - `{timestamp}`: Unix timestamp
    /// - `{short_hash}`: Short git commit hash
    ///
    /// # Default
    ///
    /// `"{version}-{branch}.{timestamp}"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::VersionConfig;
    ///
    /// let config = VersionConfig {
    ///     snapshot_format: "{version}-snapshot.{short_hash}".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.snapshot_format, "{version}-snapshot.{short_hash}");
    /// ```
    pub snapshot_format: String,
}

/// Versioning strategy for packages.
///
/// Defines whether packages in a workspace are versioned independently or with
/// a unified version number.
///
/// # Variants
///
/// - `Independent`: Each package maintains its own version
/// - `Unified`: All packages share the same version number
///
/// # Example
///
/// ```rust
impl Default for VersionConfig {
    /// Creates a new `VersionConfig` with default values.
    ///
    /// The default configuration uses independent versioning with patch bumps
    /// and a standard snapshot format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::{VersionConfig, VersioningStrategy};
    ///
    /// let config = VersionConfig::default();
    /// assert_eq!(config.strategy, VersioningStrategy::Independent);
    /// assert_eq!(config.default_bump, "patch");
    /// assert_eq!(config.snapshot_format, "{version}-{branch}.{timestamp}");
    /// ```
    fn default() -> Self {
        Self {
            strategy: VersioningStrategy::Independent,
            default_bump: "patch".to_string(),
            snapshot_format: "{version}-{branch}.{timestamp}".to_string(),
        }
    }
}

impl Configurable for VersionConfig {
    /// Validates the version configuration.
    ///
    /// This method ensures that:
    /// - Default bump is one of: "major", "minor", "patch", "none"
    /// - Snapshot format is not empty
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::VersionConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = VersionConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        // Validate default_bump
        match self.default_bump.as_str() {
            "major" | "minor" | "patch" | "none" => {}
            _ => {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "version.default_bump: Invalid bump type '{}'. Must be one of: major, minor, patch, none",
                        self.default_bump
                    ),
                });
            }
        }

        // Validate snapshot_format is not empty
        if self.snapshot_format.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "version.snapshot_format: Snapshot format cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Merges this configuration with another configuration.
    ///
    /// Values from `other` take precedence over values in `self`.
    ///
    /// # Arguments
    ///
    /// * `other` - The configuration to merge into this one
    ///
    /// # Errors
    ///
    /// Returns an error if the merged configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::{VersionConfig, VersioningStrategy};
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = VersionConfig::default();
    /// let override_config = VersionConfig {
    ///     strategy: VersioningStrategy::Unified,
    ///     ..Default::default()
    /// };
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// assert_eq!(base.strategy, VersioningStrategy::Unified);
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.strategy = other.strategy;
        self.default_bump = other.default_bump;
        self.snapshot_format = other.snapshot_format;
        Ok(())
    }
}
