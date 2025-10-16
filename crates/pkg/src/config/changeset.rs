//! Changeset configuration for storage and management settings.
//!
//! **What**: Defines configuration for changeset storage paths, history location,
//! and available deployment environments.
//!
//! **How**: This module provides the `ChangesetConfig` structure that controls where
//! changesets are stored, archived, and what environments are available for targeting.
//!
//! **Why**: To enable flexible changeset management that supports different project
//! structures and deployment workflows while maintaining sensible defaults.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for changeset management.
///
/// This structure controls where changesets are stored, where their history is archived,
/// and what environments are available for deployment targeting.
///
/// # Fields
///
/// - `path`: Directory where active changesets are stored
/// - `history_path`: Directory where archived changesets are stored
/// - `available_environments`: List of valid environment names
/// - `default_environments`: Environments to use when none are specified
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::ChangesetConfig;
///
/// let config = ChangesetConfig::default();
/// assert_eq!(config.path, ".changesets");
/// assert_eq!(config.history_path, ".changesets/history");
/// assert_eq!(config.available_environments, vec!["production"]);
/// assert_eq!(config.default_environments, vec!["production"]);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.changeset]
/// path = ".changesets"
/// history_path = ".changesets/history"
/// available_environments = ["development", "staging", "production"]
/// default_environments = ["production"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangesetConfig {
    /// Path to the directory where active changesets are stored.
    ///
    /// This directory contains changesets that have not yet been released.
    /// Each changeset is typically stored as a separate file.
    ///
    /// # Default
    ///
    /// `.changesets`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    ///
    /// let config = ChangesetConfig {
    ///     path: ".custom-changesets".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.path, ".custom-changesets");
    /// ```
    pub path: String,

    /// Path to the directory where archived changesets are stored.
    ///
    /// When a changeset is released and archived, it is moved to this directory
    /// along with release metadata. This provides a historical record of all releases.
    ///
    /// # Default
    ///
    /// `.changesets/history`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    ///
    /// let config = ChangesetConfig {
    ///     history_path: ".releases".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.history_path, ".releases");
    /// ```
    pub history_path: String,

    /// List of available environment names for deployment targeting.
    ///
    /// These are the valid environment names that can be used when creating or
    /// updating changesets. This helps prevent typos and ensures consistency.
    ///
    /// # Default
    ///
    /// `["production"]`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    ///
    /// let config = ChangesetConfig {
    ///     available_environments: vec![
    ///         "development".to_string(),
    ///         "staging".to_string(),
    ///         "production".to_string(),
    ///     ],
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.available_environments.len(), 3);
    /// ```
    pub available_environments: Vec<String>,

    /// Default environments to use when none are specified.
    ///
    /// When creating a changeset without explicitly specifying environments,
    /// these environments will be used automatically.
    ///
    /// # Default
    ///
    /// `["production"]`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    ///
    /// let config = ChangesetConfig {
    ///     default_environments: vec!["staging".to_string()],
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.default_environments, vec!["staging"]);
    /// ```
    pub default_environments: Vec<String>,
}

impl Default for ChangesetConfig {
    /// Creates a new `ChangesetConfig` with default values.
    ///
    /// The default configuration stores changesets in `.changesets` and archives
    /// them in `.changesets/history`, with only `production` as an available environment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    ///
    /// let config = ChangesetConfig::default();
    /// assert_eq!(config.path, ".changesets");
    /// assert_eq!(config.history_path, ".changesets/history");
    /// assert_eq!(config.available_environments, vec!["production"]);
    /// assert_eq!(config.default_environments, vec!["production"]);
    /// ```
    fn default() -> Self {
        Self {
            path: ".changesets".to_string(),
            history_path: ".changesets/history".to_string(),
            available_environments: vec!["production".to_string()],
            default_environments: vec!["production".to_string()],
        }
    }
}

impl Configurable for ChangesetConfig {
    /// Validates the changeset configuration.
    ///
    /// This method ensures that:
    /// - Path is not empty
    /// - History path is not empty
    /// - At least one environment is available
    /// - Default environments are all in available environments
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = ChangesetConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        if self.path.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changeset.path: Path cannot be empty".to_string(),
            });
        }

        if self.history_path.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changeset.history_path: History path cannot be empty".to_string(),
            });
        }

        if self.available_environments.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message:
                    "changeset.available_environments: At least one environment must be available"
                        .to_string(),
            });
        }

        // Validate that default environments are in available environments
        for env in &self.default_environments {
            if !self.available_environments.contains(env) {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "changeset.default_environments: Default environment '{}' is not in available environments",
                        env
                    ),
                });
            }
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
    /// use sublime_pkg_tools::config::ChangesetConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = ChangesetConfig::default();
    /// let override_config = ChangesetConfig {
    ///     path: ".custom-changesets".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// assert_eq!(base.path, ".custom-changesets");
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.path = other.path;
        self.history_path = other.history_path;
        self.available_environments = other.available_environments;
        self.default_environments = other.default_environments;
        Ok(())
    }
}
