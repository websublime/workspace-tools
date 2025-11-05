//! Core configuration types for package tools.
//!
//! **What**: Defines the main `PackageToolsConfig` structure that aggregates all configuration
//! settings for package management operations.
//!
//! **How**: This module provides a hierarchical configuration structure that integrates with
//! `sublime_standard_tools` configuration system, supporting defaults, validation, and merging.
//!
//! **Why**: To provide a single entry point for all package tools configuration, enabling
//! consistent access to settings across all modules while maintaining clear organization.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable, StandardConfig};

use super::{
    audit::AuditConfig, changelog::ChangelogConfig, changeset::ChangesetConfig,
    dependency::DependencyConfig, git::GitConfig, upgrade::UpgradeConfig, version::VersionConfig,
};

/// Main configuration structure for package tools.
///
/// This structure aggregates all configuration settings for package management operations,
/// including changesets, versioning, dependencies, upgrades, changelogs, git integration,
/// and audits.
///
/// # Configuration Hierarchy
///
/// The configuration is organized into logical sections:
/// - [`changeset`](ChangesetConfig): Changeset storage and management settings
/// - [`version`](VersionConfig): Versioning strategy and options
/// - [`dependency`](DependencyConfig): Dependency propagation and resolution settings
/// - [`upgrade`](UpgradeConfig): Dependency upgrade detection and application
/// - [`changelog`](ChangelogConfig): Changelog generation settings
/// - [`git`](GitConfig): Git integration and commit message templates
/// - [`audit`](AuditConfig): Audit and health check configuration
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_standard_tools::config::Configurable;
///
/// // Create default configuration
/// let config = PackageToolsConfig::default();
///
/// // Validate configuration
/// config.validate().expect("Configuration should be valid");
///
/// // Access nested configuration
/// println!("Changeset path: {}", config.changeset.path);
/// println!("Version strategy: {:?}", config.version.strategy);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.changeset]
/// path = ".changesets"
/// history_path = ".changesets/history"
///
/// [package_tools.version]
/// strategy = "independent"
/// default_bump = "patch"
///
/// [package_tools.dependency]
/// propagation_bump = "patch"
/// propagate_dependencies = true
///
/// [package_tools.upgrade]
/// auto_changeset = true
///
/// [package_tools.changelog]
/// enabled = true
/// format = "keep-a-changelog"
///
/// [package_tools.git]
/// include_breaking_warning = true
///
/// [package_tools.audit]
/// enabled = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "package_tools")]
pub struct PackageToolsConfig {
    /// Changeset management configuration.
    ///
    /// Controls where changesets are stored, history location, and available environments.
    pub changeset: ChangesetConfig,

    /// Versioning strategy and configuration.
    ///
    /// Defines how versions are calculated and applied (independent vs unified).
    pub version: VersionConfig,

    /// Dependency propagation configuration.
    ///
    /// Controls how version changes propagate through the dependency graph.
    pub dependency: DependencyConfig,

    /// Dependency upgrade configuration.
    ///
    /// Settings for detecting and applying external dependency upgrades.
    pub upgrade: UpgradeConfig,

    /// Changelog generation configuration.
    ///
    /// Controls how changelogs are generated and formatted.
    pub changelog: ChangelogConfig,

    /// Git integration configuration.
    ///
    /// Templates for merge commits and breaking change warnings.
    pub git: GitConfig,

    /// Audit and health check configuration.
    ///
    /// Settings for dependency audits and health score calculation.
    pub audit: AuditConfig,

    /// Workspace configuration for monorepo projects.
    ///
    /// Contains project-specific workspace patterns extracted from package.json.
    /// This represents the actual workspace configuration declared in THIS project,
    /// as opposed to StandardConfig's generic search patterns used for package discovery.
    ///
    /// - `Some(WorkspaceConfig)` - Monorepo with workspace patterns
    /// - `None` - Single-package project (no workspaces field in package.json)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<super::WorkspaceConfig>,

    /// Standard tools configuration.
    ///
    /// Configuration from sublime_standard_tools for filesystem, package managers, etc.
    /// This field is not serialized in the package_tools section but is loaded separately.
    #[serde(skip)]
    pub standard_config: StandardConfig,
}

impl Default for PackageToolsConfig {
    /// Creates a new `PackageToolsConfig` with default values.
    ///
    /// All nested configuration structures are initialized with their respective defaults,
    /// providing sensible values that work out of the box for most projects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    ///
    /// let config = PackageToolsConfig::default();
    /// assert_eq!(config.changeset.path, ".changesets");
    /// ```
    fn default() -> Self {
        Self {
            changeset: ChangesetConfig::default(),
            version: VersionConfig::default(),
            dependency: DependencyConfig::default(),
            upgrade: UpgradeConfig::default(),
            changelog: ChangelogConfig::default(),
            git: GitConfig::default(),
            audit: AuditConfig::default(),
            workspace: None,
            standard_config: StandardConfig::default(),
        }
    }
}

impl Configurable for PackageToolsConfig {
    /// Validates the configuration structure.
    ///
    /// This method validates all nested configuration structures and ensures that
    /// the overall configuration is consistent and valid.
    ///
    /// # Errors
    ///
    /// Returns an error if any nested configuration is invalid or if there are
    /// inconsistencies between configuration sections.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = PackageToolsConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        // Validate all nested configurations
        self.changeset.validate()?;
        self.version.validate()?;
        self.dependency.validate()?;
        self.upgrade.validate()?;
        self.changelog.validate()?;
        self.git.validate()?;
        self.audit.validate()?;

        Ok(())
    }

    /// Merges this configuration with another configuration.
    ///
    /// Values from `other` take precedence over values in `self`. This enables
    /// layered configuration where base settings can be overridden by more specific
    /// configurations.
    ///
    /// # Arguments
    ///
    /// * `other` - The configuration to merge into this one
    ///
    /// # Errors
    ///
    /// Returns an error if merging any nested configuration fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = PackageToolsConfig::default();
    /// let override_config = PackageToolsConfig::default();
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        // Merge all nested configurations
        self.changeset.merge_with(other.changeset)?;
        self.version.merge_with(other.version)?;
        self.dependency.merge_with(other.dependency)?;
        self.upgrade.merge_with(other.upgrade)?;
        self.changelog.merge_with(other.changelog)?;
        self.git.merge_with(other.git)?;
        self.audit.merge_with(other.audit)?;

        // Merge workspace configuration
        if let Some(other_workspace) = other.workspace {
            if let Some(ref mut self_workspace) = self.workspace {
                self_workspace.merge_with(other_workspace)?;
            } else {
                self.workspace = Some(other_workspace);
            }
        }

        self.standard_config.merge_with(other.standard_config)?;

        Ok(())
    }
}

impl PackageToolsConfig {
    /// Returns a reference to the standard configuration.
    ///
    /// This provides access to the `StandardConfig` from `sublime_standard_tools`,
    /// which includes settings for filesystem, package managers, monorepo detection, etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    ///
    /// let config = PackageToolsConfig::default();
    /// let standard = config.get_standard_config();
    /// println!("Monorepo patterns: {:?}", standard.monorepo.workspace_patterns);
    /// ```
    #[must_use]
    pub fn get_standard_config(&self) -> &StandardConfig {
        &self.standard_config
    }

    /// Sets the standard configuration.
    ///
    /// This allows updating the standard configuration after initialization,
    /// which is useful when loading configuration from files.
    ///
    /// # Arguments
    ///
    /// * `standard_config` - The standard configuration to set
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::config::StandardConfig;
    ///
    /// let mut config = PackageToolsConfig::default();
    /// let standard = StandardConfig::default();
    /// config.set_standard_config(standard);
    /// ```
    pub fn set_standard_config(&mut self, standard_config: StandardConfig) {
        self.standard_config = standard_config;
    }

    /// Creates a new `PackageToolsConfig` with a specific standard configuration.
    ///
    /// This is useful when you want to create a configuration with custom
    /// standard settings while using defaults for package tools settings.
    ///
    /// # Arguments
    ///
    /// * `standard_config` - The standard configuration to use
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_standard_tools::config::StandardConfig;
    ///
    /// let standard = StandardConfig::default();
    /// let config = PackageToolsConfig::with_standard_config(standard);
    /// ```
    #[must_use]
    pub fn with_standard_config(standard_config: StandardConfig) -> Self {
        Self { standard_config, ..Self::default() }
    }
}
