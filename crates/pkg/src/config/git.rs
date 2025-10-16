//! Git integration configuration for commit message templates.
//!
//! **What**: Defines configuration for git integration, including merge commit templates
//! and breaking change warning templates.
//!
//! **How**: This module provides the `GitConfig` structure that controls how commit messages
//! are formatted during releases and how breaking changes are communicated.
//!
//! **Why**: To enable consistent, informative commit messages that clearly communicate
//! releases and breaking changes across different project types.

use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for git integration and commit message templates.
///
/// This structure controls how commit messages are formatted for releases and
/// how breaking changes are communicated in merge commits.
///
/// # Fields
///
/// - `merge_commit_template`: Template for single-package release commits
/// - `monorepo_merge_commit_template`: Template for monorepo release commits
/// - `include_breaking_warning`: Whether to include breaking change warnings
/// - `breaking_warning_template`: Template for breaking change warnings
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::GitConfig;
///
/// let config = GitConfig::default();
/// assert!(config.include_breaking_warning);
/// ```
///
/// # TOML Representation
///
/// ```toml
/// [package_tools.git]
/// merge_commit_template = "chore(release): release version {version}"
/// monorepo_merge_commit_template = "chore(release): release packages\n\n{packages}"
/// include_breaking_warning = true
/// breaking_warning_template = "⚠️ BREAKING CHANGES\n\n{changes}"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitConfig {
    /// Template for single-package release merge commits.
    ///
    /// This template is used when creating a release commit for a single package.
    /// Available placeholders:
    /// - `{version}`: The version being released
    /// - `{name}`: The package name
    ///
    /// # Default
    ///
    /// `"chore(release): release version {version}"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    ///
    /// let config = GitConfig {
    ///     merge_commit_template: "release: v{version}".to_string(),
    ///     ..Default::default()
    /// };
    /// assert_eq!(config.merge_commit_template, "release: v{version}");
    /// ```
    pub merge_commit_template: String,

    /// Template for monorepo release merge commits.
    ///
    /// This template is used when creating a release commit for multiple packages
    /// in a monorepo. Available placeholders:
    /// - `{packages}`: List of packages being released with their versions
    /// - `{count}`: Number of packages being released
    ///
    /// # Default
    ///
    /// `"chore(release): release packages\n\n{packages}"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    ///
    /// let config = GitConfig {
    ///     monorepo_merge_commit_template: "release: {count} packages\n\n{packages}".to_string(),
    ///     ..Default::default()
    /// };
    /// assert!(config.monorepo_merge_commit_template.contains("{count}"));
    /// ```
    pub monorepo_merge_commit_template: String,

    /// Whether to include breaking change warnings in merge commits.
    ///
    /// When enabled, merge commits will include a prominent warning section
    /// if any of the changes include breaking changes.
    ///
    /// # Default
    ///
    /// `true`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    ///
    /// let config = GitConfig {
    ///     include_breaking_warning: false,
    ///     ..Default::default()
    /// };
    /// assert!(!config.include_breaking_warning);
    /// ```
    pub include_breaking_warning: bool,

    /// Template for breaking change warnings in merge commits.
    ///
    /// This template is used when `include_breaking_warning` is true and
    /// breaking changes are detected. Available placeholders:
    /// - `{changes}`: Description of breaking changes
    /// - `{count}`: Number of breaking changes
    ///
    /// # Default
    ///
    /// `"⚠️ BREAKING CHANGES\n\n{changes}"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    ///
    /// let config = GitConfig {
    ///     breaking_warning_template: "BREAKING: {count} changes\n{changes}".to_string(),
    ///     ..Default::default()
    /// };
    /// assert!(config.breaking_warning_template.contains("{changes}"));
    /// ```
    pub breaking_warning_template: String,
}

impl Default for GitConfig {
    /// Creates a new `GitConfig` with default values.
    ///
    /// The default configuration uses conventional commit style templates and
    /// includes breaking change warnings with a prominent emoji indicator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    ///
    /// let config = GitConfig::default();
    /// assert_eq!(config.merge_commit_template, "chore(release): release version {version}");
    /// assert_eq!(config.monorepo_merge_commit_template, "chore(release): release packages\n\n{packages}");
    /// assert!(config.include_breaking_warning);
    /// assert_eq!(config.breaking_warning_template, "⚠️ BREAKING CHANGES\n\n{changes}");
    /// ```
    fn default() -> Self {
        Self {
            merge_commit_template: "chore(release): release version {version}".to_string(),
            monorepo_merge_commit_template: "chore(release): release packages\n\n{packages}"
                .to_string(),
            include_breaking_warning: true,
            breaking_warning_template: "⚠️ BREAKING CHANGES\n\n{changes}".to_string(),
        }
    }
}

impl Configurable for GitConfig {
    /// Validates the git configuration.
    ///
    /// This method ensures that:
    /// - Merge commit template is not empty
    /// - Monorepo merge commit template is not empty
    /// - Breaking warning template is not empty (if warnings are enabled)
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sublime_pkg_tools::config::GitConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let config = GitConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    fn validate(&self) -> ConfigResult<()> {
        if self.merge_commit_template.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "git.merge_commit_template: Merge commit template cannot be empty"
                    .to_string(),
            });
        }

        if self.monorepo_merge_commit_template.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "git.monorepo_merge_commit_template: Monorepo merge commit template cannot be empty".to_string(),
            });
        }

        if self.include_breaking_warning && self.breaking_warning_template.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "git.breaking_warning_template: Breaking warning template cannot be empty when warnings are enabled"
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
    /// use sublime_pkg_tools::config::GitConfig;
    /// use sublime_standard_tools::config::Configurable;
    ///
    /// let mut base = GitConfig::default();
    /// let override_config = GitConfig {
    ///     include_breaking_warning: false,
    ///     ..Default::default()
    /// };
    ///
    /// base.merge_with(override_config).expect("Merge should succeed");
    /// assert!(!base.include_breaking_warning);
    /// ```
    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.merge_commit_template = other.merge_commit_template;
        self.monorepo_merge_commit_template = other.monorepo_merge_commit_template;
        self.include_breaking_warning = other.include_breaking_warning;
        self.breaking_warning_template = other.breaking_warning_template;
        Ok(())
    }
}

