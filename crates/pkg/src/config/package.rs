use serde::{Deserialize, Serialize};
use sublime_standard_tools::config::Configurable;

use crate::config::{
    ChangelogConfig, ChangesetConfig, ConventionalConfig, CustomRegistryConfig, DependencyConfig,
    RegistryConfig, ReleaseConfig, VersionConfig,
};

/// Main configuration structure for package tools.
///
/// This structure contains all configuration sections for package management
/// operations. It follows the standard configuration pattern and integrates
/// with the sublime_standard_tools configuration system.
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use sublime_standard_tools::config::ConfigManager;
///
/// async fn load_config() -> Result<PackageToolsConfig, Box<dyn std::error::Error>> {
///     let config_manager = ConfigManager::<PackageToolsConfig>::builder()
///         .with_defaults(PackageToolsConfig::default())
///         .with_env_prefix("SUBLIME_PACKAGE_TOOLS")
///         .build();
///
///     Ok(config_manager.load().await?)
/// }
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PackageToolsConfig {
    /// Changeset management configuration
    pub changeset: ChangesetConfig,

    /// Version management configuration
    pub version: VersionConfig,

    /// Registry configuration
    pub registry: RegistryConfig,

    /// Release management configuration
    pub release: ReleaseConfig,

    /// Dependency management configuration
    pub dependency: DependencyConfig,

    /// Conventional commit configuration
    pub conventional: ConventionalConfig,

    /// Changelog generation configuration
    pub changelog: ChangelogConfig,
}

impl Configurable for PackageToolsConfig {
    fn validate(&self) -> Result<(), sublime_standard_tools::config::ConfigError> {
        // Validate changeset configuration
        if self.changeset.available_environments.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "changeset.available_environments: Must have at least one available environment",
            ));
        }

        for env in &self.changeset.default_environments {
            if !self.changeset.available_environments.contains(env) {
                return Err(sublime_standard_tools::config::ConfigError::validation(
                    format!("changeset.default_environments[{}]: Default environment not in available environments", env)
                ));
            }
        }

        // Validate version configuration
        if self.version.commit_hash_length == 0 || self.version.commit_hash_length > 40 {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "version.commit_hash_length: Must be between 1 and 40",
            ));
        }

        // Validate release strategy
        if !["independent", "unified"].contains(&self.release.strategy.as_str()) {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "release.strategy: Must be 'independent' or 'unified'",
            ));
        }

        // Validate dependency configuration
        if self.dependency.max_propagation_depth == 0 {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "dependency.max_propagation_depth: Must be greater than 0",
            ));
        }

        if !["patch", "minor", "major"].contains(&self.dependency.dependency_update_bump.as_str()) {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "dependency.dependency_update_bump: Must be 'patch', 'minor', or 'major'",
            ));
        }

        // Validate conventional commit types
        for (commit_type, config) in &self.conventional.types {
            if !["none", "patch", "minor", "major"].contains(&config.bump.as_str()) {
                return Err(sublime_standard_tools::config::ConfigError::validation(format!(
                    "conventional.types.{}.bump: Must be 'none', 'patch', 'minor', or 'major'",
                    commit_type
                )));
            }
        }

        // Validate registry configuration
        if self.registry.timeout == 0 {
            return Err(sublime_standard_tools::config::ConfigError::validation(
                "registry.timeout: Timeout must be greater than 0",
            ));
        }

        Ok(())
    }

    fn merge_with(
        &mut self,
        other: Self,
    ) -> Result<(), sublime_standard_tools::config::ConfigError> {
        // Simple field-by-field merge for now
        // In a full implementation, this would be more sophisticated
        *self = other;
        Ok(())
    }
}

/// Utility functions for configuration management.
impl PackageToolsConfig {
    /// Gets the bump type for a given conventional commit type.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The conventional commit type (e.g., "feat", "fix")
    ///
    /// # Returns
    ///
    /// The version bump type (patch/minor/major/none) or the default if unknown.
    #[must_use]
    pub fn get_bump_type(&self, commit_type: &str) -> &str {
        self.conventional
            .types
            .get(commit_type)
            .map(|config| config.bump.as_str())
            .unwrap_or(&self.conventional.default_bump_type)
    }

    /// Checks if a commit type should appear in the changelog.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The conventional commit type
    ///
    /// # Returns
    ///
    /// True if the commit type should be included in changelog
    #[must_use]
    pub fn should_include_in_changelog(&self, commit_type: &str) -> bool {
        self.conventional.types.get(commit_type).map(|config| config.changelog).unwrap_or(false)
    }

    /// Gets the changelog title for a commit type.
    ///
    /// # Arguments
    ///
    /// * `commit_type` - The conventional commit type
    ///
    /// # Returns
    ///
    /// The changelog section title for this commit type
    #[must_use]
    pub fn get_changelog_title(&self, commit_type: &str) -> Option<&str> {
        self.conventional
            .types
            .get(commit_type)
            .and_then(|config| config.changelog_title.as_deref())
    }

    /// Checks if an environment is available for releases.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment name to check
    ///
    /// # Returns
    ///
    /// True if the environment is configured as available
    #[must_use]
    pub fn is_environment_available(&self, environment: &str) -> bool {
        self.changeset.available_environments.contains(&environment.to_string())
    }

    /// Gets the registry configuration for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The registry name (or "default" for main registry)
    ///
    /// # Returns
    ///
    /// Registry configuration if found
    #[must_use]
    pub fn get_registry_config(&self, name: &str) -> Option<&CustomRegistryConfig> {
        if name == "default" {
            None // Use default registry config
        } else {
            self.registry.registries.get(name)
        }
    }
}
