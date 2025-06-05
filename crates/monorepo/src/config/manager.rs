//! Configuration manager for monorepo tools

use crate::config::{
    ChangelogConfig, ChangesetsConfig, HooksConfig, PackageManagerType, PluginsConfig, TasksConfig,
    VersioningConfig, WorkspaceConfig, WorkspacePattern,
};
use crate::error::{Error, Result};
use crate::{Environment, MonorepoConfig};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Configuration manager that handles loading, saving, and managing monorepo configurations
pub struct ConfigManager {
    /// The current configuration
    config: Arc<RwLock<MonorepoConfig>>,

    /// Path to the configuration file
    config_path: Option<PathBuf>,

    /// Whether to auto-save on changes
    auto_save: bool,
}

impl ConfigManager {
    /// Create a new configuration manager with default config
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(MonorepoConfig::default())),
            config_path: None,
            auto_save: false,
        }
    }

    /// Create a configuration manager with a specific config
    pub fn with_config(config: MonorepoConfig) -> Self {
        Self { config: Arc::new(RwLock::new(config)), config_path: None, auto_save: false }
    }

    /// Load configuration from a file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::config(format!("Failed to read config file: {e}")))?;

        let config: MonorepoConfig = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)
                .map_err(|e| Error::config(format!("Failed to parse TOML: {e}")))?,
            Some("yaml" | "yml") => serde_yaml::from_str(&content)
                .map_err(|e| Error::config(format!("Failed to parse YAML: {e}")))?,
            _ => return Err(Error::config("Unsupported config file format")),
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Some(path.to_path_buf()),
            auto_save: false,
        })
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let config =
            self.config.read().map_err(|_| Error::config("Failed to acquire read lock"))?;

        let content = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::to_string_pretty(&*config)?,
            Some("toml") => toml::to_string_pretty(&*config)
                .map_err(|e| Error::config(format!("Failed to serialize to TOML: {e}")))?,
            Some("yaml" | "yml") => serde_yaml::to_string(&*config)
                .map_err(|e| Error::config(format!("Failed to serialize to YAML: {e}")))?,
            _ => return Err(Error::config("Unsupported config file format")),
        };

        std::fs::write(path, content)
            .map_err(|e| Error::config(format!("Failed to write config file: {e}")))?;

        Ok(())
    }

    /// Save configuration to the loaded path
    pub fn save(&self) -> Result<()> {
        match &self.config_path {
            Some(path) => self.save_to_file(path),
            None => Err(Error::config("No config file path set")),
        }
    }

    /// Get the current configuration
    pub fn get(&self) -> Result<MonorepoConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.clone())
    }

    /// Update the configuration
    pub fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        let mut config =
            self.config.write().map_err(|_| Error::config("Failed to acquire write lock"))?;

        updater(&mut config);

        drop(config); // Explicitly drop the lock

        if self.auto_save {
            self.save()?;
        }

        Ok(())
    }

    /// Get a specific configuration section
    pub fn get_versioning(&self) -> Result<VersioningConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.versioning.clone())
    }

    /// Get tasks configuration
    pub fn get_tasks(&self) -> Result<TasksConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.tasks.clone())
    }

    /// Get changelog configuration
    pub fn get_changelog(&self) -> Result<ChangelogConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.changelog.clone())
    }

    /// Get hooks configuration
    pub fn get_hooks(&self) -> Result<HooksConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.hooks.clone())
    }

    /// Get changesets configuration
    pub fn get_changesets(&self) -> Result<ChangesetsConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.changesets.clone())
    }

    /// Get plugins configuration
    pub fn get_plugins(&self) -> Result<PluginsConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.plugins.clone())
    }

    /// Get environments
    pub fn get_environments(&self) -> Result<Vec<Environment>> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.environments.clone())
    }

    /// Get workspace configuration
    pub fn get_workspace(&self) -> Result<WorkspaceConfig> {
        self.config
            .read()
            .map_err(|_| Error::config("Failed to acquire read lock"))
            .map(|config| config.workspace.clone())
    }

    /// Set auto-save behavior
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Get the config file path
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Create default configuration files in a directory
    pub fn create_default_config_files(dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        let config = MonorepoConfig::default();

        // Create .monorepo directory
        let config_dir = dir.join(".monorepo");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| Error::config(format!("Failed to create config directory: {e}")))?;

        // Save as JSON
        let json_path = config_dir.join("config.json");
        let json_content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&json_path, json_content)
            .map_err(|e| Error::config(format!("Failed to write JSON config: {e}")))?;

        // Save as TOML (alternative)
        let toml_path = config_dir.join("config.toml");
        let toml_content = toml::to_string_pretty(&config)
            .map_err(|e| Error::config(format!("Failed to serialize to TOML: {e}")))?;
        std::fs::write(&toml_path, toml_content)
            .map_err(|e| Error::config(format!("Failed to write TOML config: {e}")))?;

        Ok(())
    }

    /// Look for configuration file in standard locations
    pub fn find_config_file(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let start_dir = start_dir.as_ref();

        // Check for config files in order of preference
        let config_names = [
            ".monorepo/config.json",
            ".monorepo/config.toml",
            ".monorepo/config.yaml",
            ".monorepo/config.yml",
            "monorepo.config.json",
            "monorepo.config.toml",
            "monorepo.config.yaml",
            "monorepo.config.yml",
        ];

        // Check current directory and parent directories
        let mut current = start_dir.to_path_buf();
        loop {
            for config_name in &config_names {
                let config_path = current.join(config_name);
                if config_path.exists() {
                    return Some(config_path);
                }
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get workspace patterns filtered by package manager type and environment
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_workspace_patterns(
        &self,
        package_manager: Option<PackageManagerType>,
        environment: Option<&Environment>,
    ) -> Result<Vec<WorkspacePattern>> {
        let workspace = self.get_workspace()?;

        let patterns: Vec<WorkspacePattern> = workspace
            .patterns
            .into_iter()
            .filter(|pattern| {
                // Filter by enabled status
                if !pattern.enabled {
                    return false;
                }

                // Filter by package manager
                if let Some(pm) = &package_manager {
                    if let Some(pattern_pms) = &pattern.package_managers {
                        if !pattern_pms.contains(pm) {
                            return false;
                        }
                    }
                }

                // Filter by environment
                if let Some(env) = environment {
                    if let Some(pattern_envs) = &pattern.environments {
                        if !pattern_envs.contains(env) {
                            return false;
                        }
                    }
                }

                true
            })
            .collect();

        Ok(patterns)
    }

    /// Get effective workspace patterns combining config patterns with auto-detected ones
    pub fn get_effective_workspace_patterns(
        &self,
        auto_detected: Vec<String>,
        package_manager: Option<PackageManagerType>,
        environment: Option<&Environment>,
    ) -> Result<Vec<String>> {
        let workspace = self.get_workspace()?;
        let config_patterns = self.get_workspace_patterns(package_manager.clone(), environment)?;

        let mut patterns = Vec::new();

        // Add patterns from configuration
        for pattern in config_patterns {
            if pattern.options.override_detection {
                // If this pattern overrides detection, clear auto-detected patterns
                patterns.clear();
            }
            patterns.push(pattern.pattern);
        }

        // Add auto-detected patterns if merge is enabled and no override patterns exist
        if workspace.merge_with_detected
            && !workspace.patterns.iter().any(|p| p.options.override_detection)
        {
            for auto_pattern in auto_detected {
                if !patterns.contains(&auto_pattern) {
                    patterns.push(auto_pattern);
                }
            }
        }

        // Sort by priority if we have config patterns
        let workspace_patterns = self.get_workspace_patterns(package_manager, environment)?;
        if !workspace_patterns.is_empty() {
            let mut pattern_priorities: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for wp in workspace_patterns {
                pattern_priorities.insert(wp.pattern.clone(), wp.priority);
            }

            patterns.sort_by(|a, b| {
                let priority_a = pattern_priorities.get(a).unwrap_or(&100);
                let priority_b = pattern_priorities.get(b).unwrap_or(&100);
                priority_b.cmp(priority_a) // Higher priority first
            });
        }

        Ok(patterns)
    }

    /// Add a workspace pattern to the configuration
    pub fn add_workspace_pattern(&self, pattern: WorkspacePattern) -> Result<()> {
        self.update(|config| {
            config.workspace.patterns.push(pattern);
        })
    }

    /// Remove a workspace pattern by pattern string
    pub fn remove_workspace_pattern(&self, pattern: &str) -> Result<bool> {
        let mut removed = false;
        self.update(|config| {
            let initial_len = config.workspace.patterns.len();
            config.workspace.patterns.retain(|p| p.pattern != pattern);
            removed = config.workspace.patterns.len() < initial_len;
        })?;
        Ok(removed)
    }

    /// Update a workspace pattern
    pub fn update_workspace_pattern<F>(&self, pattern: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut WorkspacePattern),
    {
        let mut found = false;
        self.update(|config| {
            if let Some(wp) = config.workspace.patterns.iter_mut().find(|p| p.pattern == pattern) {
                updater(wp);
                found = true;
            }
        })?;
        Ok(found)
    }

    /// Get workspace patterns for a specific package manager
    pub fn get_package_manager_patterns(
        &self,
        package_manager: PackageManagerType,
    ) -> Result<Vec<String>> {
        let workspace = self.get_workspace()?;

        // Check for package manager specific overrides
        let override_patterns = match package_manager {
            PackageManagerType::Npm => {
                workspace.package_manager_configs.npm.and_then(|config| config.workspaces_override)
            }
            PackageManagerType::Yarn | PackageManagerType::YarnBerry => {
                workspace.package_manager_configs.yarn.and_then(|config| config.workspaces_override)
            }
            PackageManagerType::Pnpm => {
                workspace.package_manager_configs.pnpm.and_then(|config| config.packages_override)
            }
            PackageManagerType::Bun => {
                workspace.package_manager_configs.bun.and_then(|config| config.workspaces_override)
            }
            PackageManagerType::Custom(_) => None,
        };

        if let Some(patterns) = override_patterns {
            Ok(patterns)
        } else {
            // Fall back to general workspace patterns
            let patterns = self.get_workspace_patterns(Some(package_manager), None)?;
            Ok(patterns.into_iter().map(|p| p.pattern).collect())
        }
    }

    /// Validate workspace configuration
    pub fn validate_workspace_config(&self, existing_packages: &[String]) -> Result<Vec<String>> {
        let workspace = self.get_workspace()?;
        let mut validation_errors = Vec::new();

        // Validate that patterns match existing packages if required
        if workspace.validation.require_pattern_matches {
            for pattern in &workspace.patterns {
                if pattern.enabled {
                    let pattern_matches = existing_packages
                        .iter()
                        .any(|pkg| self.pattern_matches_package(&pattern.pattern, pkg));

                    if !pattern_matches {
                        validation_errors.push(format!(
                            "Workspace pattern '{}' does not match any existing packages",
                            pattern.pattern
                        ));
                    }
                }
            }
        }

        // Validate naming conventions
        if workspace.validation.validate_naming && !workspace.validation.naming_patterns.is_empty()
        {
            for package in existing_packages {
                let matches_naming = workspace
                    .validation
                    .naming_patterns
                    .iter()
                    .any(|pattern| self.pattern_matches_package(pattern, package));

                if !matches_naming {
                    validation_errors.push(format!(
                        "Package '{package}' does not match any naming convention patterns"
                    ));
                }
            }
        }

        Ok(validation_errors)
    }

    /// Check if a pattern matches a package path
    pub fn pattern_matches_package(&self, pattern: &str, package_path: &str) -> bool {
        // Simple glob-style matching - could be enhanced with proper glob library
        if pattern.contains('*') {
            if pattern.ends_with("/*") {
                let base = &pattern[..pattern.len() - 2];
                package_path.starts_with(base)
            } else if pattern.starts_with('*') {
                let suffix = &pattern[1..];
                package_path.ends_with(suffix)
            } else if let Some(star_pos) = pattern.find('*') {
                let prefix = &pattern[..star_pos];
                let suffix = &pattern[star_pos + 1..];
                package_path.starts_with(prefix) && package_path.ends_with(suffix)
            } else {
                false
            }
        } else {
            package_path == pattern
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::VersionBumpType;

    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_default_config() {
        let manager = ConfigManager::new();
        let config = manager.get().unwrap();

        assert_eq!(config.versioning.default_bump, VersionBumpType::Patch);
        assert!(config.versioning.propagate_changes);
        assert!(config.hooks.enabled);
        assert_eq!(config.environments.len(), 3);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_update_config() {
        let manager = ConfigManager::new();

        manager
            .update(|config| {
                config.versioning.default_bump = VersionBumpType::Minor;
                config.versioning.auto_tag = false;
            })
            .unwrap();

        let config = manager.get().unwrap();
        assert_eq!(config.versioning.default_bump, VersionBumpType::Minor);
        assert!(!config.versioning.auto_tag);
    }
}
