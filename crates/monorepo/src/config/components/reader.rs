//! Configuration reader component
//!
//! Provides read-only access to configuration sections. This component is responsible
//! for retrieving specific configuration sections and values without modification.

use crate::config::{
    ChangelogConfig, ChangesetsConfig, HooksConfig, PluginsConfig, TasksConfig,
    VersioningConfig, WorkspaceConfig, MonorepoConfig,
};
use crate::Environment;
use std::path::Path;

/// Component responsible for reading configuration sections
pub struct ConfigReader<'a> {
    config: &'a MonorepoConfig,
    config_path: Option<&'a Path>,
}

impl<'a> ConfigReader<'a> {
    /// Create a new config reader for the given configuration
    #[must_use]
    pub fn new(config: &'a MonorepoConfig, config_path: Option<&'a Path>) -> Self {
        Self { config, config_path }
    }

    /// Get a clone of the entire configuration
    ///
    /// # Returns
    /// Complete cloned configuration
    #[must_use]
    pub fn get_clone(&self) -> MonorepoConfig {
        self.config.clone()
    }

    /// Get the versioning configuration section
    ///
    /// # Returns
    /// Reference to versioning configuration
    #[must_use]
    pub fn get_versioning(&self) -> &VersioningConfig {
        &self.config.versioning
    }

    /// Get the tasks configuration section
    ///
    /// # Returns
    /// Reference to tasks configuration
    #[must_use]
    pub fn get_tasks(&self) -> &TasksConfig {
        &self.config.tasks
    }

    /// Get the changelog configuration section
    ///
    /// # Returns
    /// Reference to changelog configuration
    #[must_use]
    pub fn get_changelog(&self) -> &ChangelogConfig {
        &self.config.changelog
    }

    /// Get the hooks configuration section
    ///
    /// # Returns
    /// Reference to hooks configuration
    #[must_use]
    pub fn get_hooks(&self) -> &HooksConfig {
        &self.config.hooks
    }

    /// Get the changesets configuration section
    ///
    /// # Returns
    /// Reference to changesets configuration
    #[must_use]
    pub fn get_changesets(&self) -> &ChangesetsConfig {
        &self.config.changesets
    }

    /// Get the plugins configuration section
    ///
    /// # Returns
    /// Reference to plugins configuration
    #[must_use]
    pub fn get_plugins(&self) -> &PluginsConfig {
        &self.config.plugins
    }

    /// Get the list of configured environments
    ///
    /// # Returns
    /// Reference to environments list
    #[must_use]
    pub fn get_environments(&self) -> &[Environment] {
        &self.config.environments
    }

    /// Get the workspace configuration section
    ///
    /// # Returns
    /// Reference to workspace configuration
    #[must_use]
    pub fn get_workspace(&self) -> &WorkspaceConfig {
        &self.config.workspace
    }

    /// Get the configuration file path (if loaded from file)
    ///
    /// # Returns
    /// Optional path to the configuration file
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path
    }

    /// Check if a specific environment is configured
    ///
    /// # Arguments
    /// * `env` - Environment to check for
    ///
    /// # Returns
    /// True if the environment is configured
    #[must_use]
    pub fn has_environment(&self, env: &Environment) -> bool {
        self.config.environments.contains(env)
    }

    /// Get configuration for a specific environment
    ///
    /// # Arguments
    /// * `env` - Environment to get configuration for
    ///
    /// # Returns
    /// Environment-specific configuration if available
    #[must_use]
    pub fn get_environment_config(&self, env: &Environment) -> Option<&Environment> {
        self.config.environments.iter().find(|&e| e == env)
    }

    /// Check if hooks are enabled globally
    ///
    /// # Returns
    /// True if hooks are globally enabled
    #[must_use]
    pub fn are_hooks_enabled(&self) -> bool {
        self.config.hooks.enabled
    }

    /// Check if changesets are required
    ///
    /// # Returns
    /// True if changesets are required for changes
    #[must_use]
    pub fn are_changesets_required(&self) -> bool {
        self.config.changesets.required
    }

    /// Check if automatic tagging is enabled
    ///
    /// # Returns
    /// True if auto-tagging is enabled
    #[must_use]
    pub fn is_auto_tagging_enabled(&self) -> bool {
        self.config.versioning.auto_tag
    }

    /// Check if breaking changes are included in changelogs
    ///
    /// # Returns
    /// True if breaking changes are included
    #[must_use]
    pub fn are_breaking_changes_included(&self) -> bool {
        self.config.changelog.include_breaking_changes
    }

    /// Get the configured package manager patterns
    ///
    /// # Returns
    /// List of package manager specific patterns
    #[must_use]
    pub fn get_package_manager_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Add patterns based on workspace configuration
        for pattern in &self.config.workspace.patterns {
            patterns.push(pattern.pattern.clone());
        }
        
        patterns
    }

    /// Get summary statistics about the configuration
    ///
    /// # Returns
    /// Tuple of (environments_count, workspace_patterns_count, plugins_count)
    #[must_use]
    pub fn get_config_stats(&self) -> (usize, usize, usize) {
        (
            self.config.environments.len(),
            self.config.workspace.patterns.len(),
            self.config.plugins.enabled.len(),
        )
    }
}