//! Dependency configuration for propagation and resolution settings.
//!
//! **What**: Defines configuration for dependency propagation behavior, including which
//! dependency types to propagate, version spec skipping, and circular dependency handling.
//!
//! **How**: This module provides the `DependencyConfig` structure that controls how version
//! changes propagate through the dependency graph and what protocols to skip.
//!
//! **Why**: To enable fine-grained control over dependency propagation while preventing
//! issues with workspace protocols and circular dependencies.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for dependency propagation and resolution.
///
/// This structure controls how version changes propagate through the dependency graph,
/// what types of dependencies are propagated, and how to handle special cases like
/// workspace protocols and circular dependencies.
///
/// # Fields
///
/// - `propagation_bump`: Version bump type to use for propagated updates
/// - `propagate_dependencies`: Whether to propagate to regular dependencies
/// - `propagate_dev_dependencies`: Whether to propagate to devDependencies
/// - `propagate_peer_dependencies`: Whether to propagate to peerDependencies
/// - `max_depth`: Maximum depth for dependency propagation
/// - `fail_on_circular`: Whether to fail when circular dependencies are detected
/// - `skip_workspace_protocol`: Skip dependencies using workspace protocol
/// - `skip_file_protocol`: Skip dependencies using file protocol
/// - `skip_link_protocol`: Skip dependencies using link protocol
/// - `skip_portal_protocol`: Skip dependencies using portal protocol
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::DependencyConfig;
///
/// let config = DependencyConfig::default();
/// assert_eq!(config.propagation_bump.as_ref(), "patch");
/// assert!(config.propagate_dependencies);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.dependency]
/// propagation_bump = "patch"
/// propagate_dependencies = true
/// propagate_dev_dependencies = false
/// propagate_peer_dependencies = true
/// max_depth = 10
/// fail_on_circular = true
/// skip_workspace_protocol = true
/// skip_file_protocol = true
/// skip_link_protocol = true
/// skip_portal_protocol = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyConfig {
    /// Version bump type to use when propagating updates.
    ///
    /// When a package version changes and triggers updates to its dependents,
    /// this setting controls what version bump those dependents receive.
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
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     propagation_bump: "minor".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.propagation_bump, "minor");
    /// ```
    pub propagation_bump: String,

    /// Whether to propagate version updates to regular dependencies.
    ///
    /// When enabled, if package A depends on package B and B's version changes,
    /// A will also receive a version update.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     propagate_dependencies: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.propagate_dependencies);
    /// ```
    pub propagate_dependencies: bool,

    /// Whether to propagate version updates to devDependencies.
    ///
    /// When enabled, changes to packages listed in devDependencies will also
    /// trigger version updates to the dependent package.
    ///
    /// # Default
    ///
    /// `false`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     propagate_dev_dependencies: true,
    ///     ..Default::default()
    /// };
    /// assert!(config.propagate_dev_dependencies);
    /// ```
    pub propagate_dev_dependencies: bool,

    /// Whether to propagate version updates to peerDependencies.
    ///
    /// When enabled, changes to packages listed in peerDependencies will also
    /// trigger version updates to the dependent package.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     propagate_peer_dependencies: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.propagate_peer_dependencies);
    /// ```
    pub propagate_peer_dependencies: bool,

    /// Maximum depth for dependency propagation.
    ///
    /// Limits how far changes propagate through the dependency graph. A depth of 1
    /// means only direct dependents are updated, 2 means dependents of dependents, etc.
    /// A value of 0 or very large number effectively means unlimited.
    ///
    /// # Default
    ///
    /// `10`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     max_depth: 5,
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.max_depth, 5);
    /// ```
    pub max_depth: usize,

    /// Whether to fail when circular dependencies are detected.
    ///
    /// When enabled, circular dependencies cause an error. When disabled,
    /// they are reported but don't prevent version resolution.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     fail_on_circular: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.fail_on_circular);
    /// ```
    pub fail_on_circular: bool,

    /// Skip dependencies using the workspace protocol.
    ///
    /// When enabled, dependencies with version specs like "workspace:*" or "workspace:^1.0.0"
    /// are not updated during version propagation.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     skip_workspace_protocol: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.skip_workspace_protocol);
    /// ```
    pub skip_workspace_protocol: bool,

    /// Skip dependencies using the file protocol.
    ///
    /// When enabled, dependencies with version specs like "file:../other-package"
    /// are not updated during version propagation.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     skip_file_protocol: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.skip_file_protocol);
    /// ```
    pub skip_file_protocol: bool,

    /// Skip dependencies using the link protocol.
    ///
    /// When enabled, dependencies with version specs like "link:../other-package"
    /// are not updated during version propagation.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     skip_link_protocol: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.skip_link_protocol);
    /// ```
    pub skip_link_protocol: bool,

    /// Skip dependencies using the portal protocol.
    ///
    /// When enabled, dependencies with version specs like "portal:../other-package"
    /// are not updated during version propagation.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig {
    ///     skip_portal_protocol: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.skip_portal_protocol);
    /// ```
    pub skip_portal_protocol: bool,
}

impl Default for DependencyConfig {
    /// Creates a new `DependencyConfig` with default values.
    ///
    /// The default configuration propagates regular and peer dependencies using patch bumps,
    /// skips all special protocols, and fails on circular dependencies.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    ///
    /// let config = DependencyConfig::default();
    /// assert_eq!(config.propagation_bump, "patch");
    /// assert!(config.propagate_dependencies);
    /// assert!(!config.propagate_dev_dependencies);
    /// assert!(config.propagate_peer_dependencies);
    /// assert_eq!(config.max_depth, 10);
    /// assert!(config.fail_on_circular);
    /// assert!(config.skip_workspace_protocol);
    /// assert!(config.skip_file_protocol);
    /// assert!(config.skip_link_protocol);
    /// assert!(config.skip_portal_protocol);
    /// ```
    fn default() -> Self {
        Self {
            propagation_bump: "patch".to_string(),
            propagate_dependencies: true,
            propagate_dev_dependencies: false,
            propagate_peer_dependencies: true,
            max_depth: 10,
            fail_on_circular: true,
            skip_workspace_protocol: true,
            skip_file_protocol: true,
            skip_link_protocol: true,
            skip_portal_protocol: true,
        }
    }
}

impl Configurable for DependencyConfig {
    /// Validates the dependency configuration.
    ///
    /// This method ensures that:
    /// - Propagation bump is one of: "major", "minor", "patch", "none"
    /// - At least one dependency type is enabled for propagation
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::DependencyConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = DependencyConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        // Validate propagation_bump
        match self.propagation_bump.as_str() {
            "major" | "minor" | "patch" | "none" => {}
            _ => {
                return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                    message: format!(
                        "dependency.propagation_bump: Invalid bump type '{}'. Must be one of: major, minor, patch, none",
                        self.propagation_bump
                    ),
                });
            }
        }

        // Validate that at least one propagation type is enabled
        if !self.propagate_dependencies
            && !self.propagate_dev_dependencies
            && !self.propagate_peer_dependencies
        {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "dependency: At least one dependency type must be enabled for propagation"
                    .to_string(),
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
    /// use sublime_pkg_tools::config::DependencyConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = DependencyConfig::default();
    /// let override_config = DependencyConfig {
    ///     propagation_bump: "minor".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// assert_eq!(base.propagation_bump, "minor");
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.propagation_bump = other.propagation_bump;
        self.propagate_dependencies = other.propagate_dependencies;
        self.propagate_dev_dependencies = other.propagate_dev_dependencies;
        self.propagate_peer_dependencies = other.propagate_peer_dependencies;
        self.max_depth = other.max_depth;
        self.fail_on_circular = other.fail_on_circular;
        self.skip_workspace_protocol = other.skip_workspace_protocol;
        self.skip_file_protocol = other.skip_file_protocol;
        self.skip_link_protocol = other.skip_link_protocol;
        self.skip_portal_protocol = other.skip_portal_protocol;
        Ok(())
    }
}

