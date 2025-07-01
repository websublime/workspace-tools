//! Configuration manager for monorepo tools
//!
//! This is a facade that provides a unified interface over the focused configuration components.
//! It maintains backward compatibility while internally using the new component architecture.

use crate::config::{
    ChangelogConfig, ChangesetsConfig, HooksConfig, PackageManagerType, PluginsConfig, TasksConfig,
    VersioningConfig, WorkspaceConfig, WorkspacePattern, ConfigManager, PatternMatcher,
    ConfigPersistence, ConfigReader,
};
use crate::error::{Error, Result};
use crate::{Environment, MonorepoConfig};
use glob::Pattern;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl ConfigManager {
    /// Create a new configuration manager with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MonorepoConfig::default(),
            config_path: None,
            auto_save: false,
        }
    }

    /// Create a configuration manager with a specific config
    #[must_use]
    pub fn with_config(config: MonorepoConfig) -> Self {
        Self { config, config_path: None, auto_save: false }
    }

    /// Load configuration from a file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let persistence = ConfigPersistence::new();
        let config = persistence.load_from_file(path)?;

        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
            auto_save: false,
        })
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let persistence = ConfigPersistence::new();
        persistence.save_to_file(&self.config, path)
    }

    /// Save configuration to the loaded path
    pub fn save(&self) -> Result<()> {
        match &self.config_path {
            Some(path) => self.save_to_file(path),
            None => Err(Error::config("No config file path set")),
        }
    }

    /// Get the current configuration (clone)
    #[must_use]
    pub fn get_clone(&self) -> MonorepoConfig {
        let reader = ConfigReader::new(&self.config, self.config_path.as_deref());
        reader.get_clone()
    }

    /// Update the configuration and return a new ConfigManager with the updated config
    /// 
    /// This follows Rust ownership principles by returning a new instance instead of mutating.
    pub fn with_update<F>(mut self, updater: F) -> Result<Self>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        log::debug!("Updating configuration with provided updater function");
        
        updater(&mut self.config);
        log::debug!("Configuration updated successfully");

        if self.auto_save {
            log::debug!("Auto-save enabled, saving configuration");
            self.save()?;
        }

        Ok(self)
    }
    
    /// Update the configuration in place (for compatibility)
    /// 
    /// Note: This is a transitional method. New code should prefer `with_update`.
    pub fn update<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut MonorepoConfig),
    {
        log::debug!("Updating configuration with provided updater function");
        
        updater(&mut self.config);
        log::debug!("Configuration updated successfully");

        if self.auto_save {
            log::debug!("Auto-save enabled, saving configuration");
            self.save()?;
        }

        Ok(())
    }

    /// Get a specific configuration section
    #[must_use]
    pub fn get_versioning(&self) -> &VersioningConfig {
        &self.config.versioning
    }

    /// Get tasks configuration
    #[must_use]
    pub fn get_tasks(&self) -> &TasksConfig {
        &self.config.tasks
    }

    /// Get changelog configuration
    #[must_use]
    pub fn get_changelog(&self) -> &ChangelogConfig {
        &self.config.changelog
    }

    /// Get hooks configuration
    #[must_use]
    pub fn get_hooks(&self) -> &HooksConfig {
        &self.config.hooks
    }

    /// Get changesets configuration
    #[must_use]
    pub fn get_changesets(&self) -> &ChangesetsConfig {
        &self.config.changesets
    }

    /// Get plugins configuration
    #[must_use]
    pub fn get_plugins(&self) -> &PluginsConfig {
        &self.config.plugins
    }

    /// Get environments
    #[must_use]
    pub fn get_environments(&self) -> &[Environment] {
        &self.config.environments
    }

    /// Get workspace configuration
    #[must_use]
    pub fn get_workspace(&self) -> &WorkspaceConfig {
        &self.config.workspace
    }

    /// Set auto-save behavior
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Get the config file path
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Create default configuration files in a directory
    pub fn create_default_config_files(dir: impl AsRef<Path>) -> Result<()> {
        let persistence = ConfigPersistence::new();
        persistence.create_default_config_files(dir)
    }

    /// Look for configuration file in standard locations
    pub fn find_config_file(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let persistence = ConfigPersistence::new();
        persistence.find_config_file(start_dir)
    }

    /// Get workspace patterns filtered by package manager type and environment
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_workspace_patterns(
        &self,
        package_manager: Option<PackageManagerType>,
        environment: Option<&Environment>,
    ) -> Vec<WorkspacePattern> {
        let workspace = self.get_workspace();

        let patterns: Vec<WorkspacePattern> = workspace
            .patterns
            .iter()
            .cloned()
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

        patterns
    }

    /// Get effective workspace patterns combining config patterns with auto-detected ones
    pub fn get_effective_workspace_patterns(
        &self,
        auto_detected: Vec<String>,
        package_manager: Option<PackageManagerType>,
        environment: Option<&Environment>,
    ) -> Vec<String> {
        let workspace = self.get_workspace();
        let config_patterns = self.get_workspace_patterns(package_manager.clone(), environment);

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
        let workspace_patterns = self.get_workspace_patterns(package_manager, environment);
        if !workspace_patterns.is_empty() {
            let mut pattern_priorities: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for wp in workspace_patterns {
                pattern_priorities.insert(wp.pattern, wp.priority);
            }

            patterns.sort_by(|a, b| {
                let priority_a = pattern_priorities.get(a).unwrap_or(&100);
                let priority_b = pattern_priorities.get(b).unwrap_or(&100);
                priority_b.cmp(priority_a) // Higher priority first
            });
        }

        patterns
    }

    /// Add a workspace pattern to the configuration and return new ConfigManager
    #[must_use]
    pub fn with_workspace_pattern(mut self, pattern: WorkspacePattern) -> Self {
        self.config.workspace.patterns.push(pattern);
        self
    }
    
    /// Add a workspace pattern to the configuration in place
    pub fn add_workspace_pattern(&mut self, pattern: WorkspacePattern) -> Result<()> {
        self.update(|config| {
            config.workspace.patterns.push(pattern);
        })
    }

    /// Remove a workspace pattern and return new ConfigManager and success flag
    pub fn without_workspace_pattern(mut self, pattern: &str) -> (Self, bool) {
        let initial_len = self.config.workspace.patterns.len();
        self.config.workspace.patterns.retain(|p| p.pattern != pattern);
        let removed = self.config.workspace.patterns.len() < initial_len;
        (self, removed)
    }
    
    /// Remove a workspace pattern by pattern string
    pub fn remove_workspace_pattern(&mut self, pattern: &str) -> Result<bool> {
        let mut removed = false;
        self.update(|config| {
            let initial_len = config.workspace.patterns.len();
            config.workspace.patterns.retain(|p| p.pattern != pattern);
            removed = config.workspace.patterns.len() < initial_len;
        })?;
        Ok(removed)
    }

    /// Update a workspace pattern and return new ConfigManager and success flag
    pub fn with_updated_workspace_pattern<F>(mut self, pattern: &str, updater: F) -> (Self, bool)
    where
        F: FnOnce(&mut WorkspacePattern),
    {
        let mut found = false;
        if let Some(wp) = self.config.workspace.patterns.iter_mut().find(|p| p.pattern == pattern) {
            updater(wp);
            found = true;
        }
        (self, found)
    }
    
    /// Update a workspace pattern
    pub fn update_workspace_pattern<F>(&mut self, pattern: &str, updater: F) -> Result<bool>
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
    ) -> Vec<String> {
        let workspace = self.get_workspace();

        // Check for package manager specific overrides
        let override_patterns = match package_manager {
            PackageManagerType::Npm => {
                workspace.package_manager_configs.npm.as_ref().and_then(|config| config.workspaces_override.clone())
            }
            PackageManagerType::Yarn | PackageManagerType::YarnBerry => {
                workspace.package_manager_configs.yarn.as_ref().and_then(|config| config.workspaces_override.clone())
            }
            PackageManagerType::Pnpm => {
                workspace.package_manager_configs.pnpm.as_ref().and_then(|config| config.packages_override.clone())
            }
            PackageManagerType::Bun => {
                workspace.package_manager_configs.bun.as_ref().and_then(|config| config.workspaces_override.clone())
            }
            PackageManagerType::Custom(_) => None,
        };

        if let Some(patterns) = override_patterns {
            patterns
        } else {
            // Fall back to general workspace patterns
            let patterns = self.get_workspace_patterns(Some(package_manager), None);
            patterns.into_iter().map(|p| p.pattern).collect()
        }
    }

    /// Validate workspace configuration
    pub fn validate_workspace_config(&self, existing_packages: &[String]) -> Vec<String> {
        let workspace = self.get_workspace();
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

        validation_errors
    }

    /// Check if a pattern matches a package path
    ///
    /// DRY: Simplified pattern matching using standard glob functionality
    /// Supports glob patterns: `*`, `**`, `?`, `[seq]`, `[!seq]`
    /// Maintains backward compatibility for single `*` patterns.
    ///
    /// # Examples
    /// ```ignore
    /// pattern_matches_package("packages/*", "packages/core") // true
    /// pattern_matches_package("packages/**", "packages/apps/web") // true
    /// pattern_matches_package("@scope/*", "@scope/package") // true
    /// ```
    #[must_use]
    pub fn pattern_matches_package(&self, pattern: &str, package_path: &str) -> bool {
        // Early return for exact matches (optimization)
        if !pattern.contains(['*', '?', '[']) {
            return package_path == pattern;
        }

        // Normalize paths for consistent matching
        let normalized_pattern = pattern.replace('\\', "/");
        let normalized_path = package_path.replace('\\', "/");

        // DRY: Use standard Pattern matching with backward compatibility
        match Pattern::new(&normalized_pattern) {
            Ok(glob_pattern) => {
                if !glob_pattern.matches(&normalized_path) {
                    return false;
                }
                
                // Maintain backward compatibility: single * should not match across path segments
                if normalized_pattern.contains('*') && !normalized_pattern.contains("**") {
                    let pattern_segments = normalized_pattern.split('/').count();
                    let path_segments = normalized_path.split('/').count();
                    
                    if pattern_segments != path_segments {
                        return false;
                    }
                }
                
                true
            }
            Err(_) => {
                // DRY: Simplified error handling - fall back to exact match
                package_path == pattern
            }
        }
    }

    /// Check multiple patterns against multiple package paths efficiently
    ///
    /// DRY: Simplified batch pattern matching using standard functionality
    ///
    /// # Returns
    /// A vector of tuples containing (pattern_index, package_index) for all matches
    ///
    /// # Examples
    /// ```ignore
    /// let patterns = vec!["packages/*", "@scope/*", "apps/**"];
    /// let packages = vec!["packages/core", "@scope/lib", "apps/web/src"];
    /// let matches = manager.batch_pattern_matches(&patterns, &packages);
    /// ```
    pub fn batch_pattern_matches(
        &self,
        patterns: &[String],
        packages: &[String],
    ) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        for (pattern_idx, pattern) in patterns.iter().enumerate() {
            for (package_idx, package) in packages.iter().enumerate() {
                if self.pattern_matches_package(pattern, package) {
                    matches.push((pattern_idx, package_idx));
                }
            }
        }

        matches
    }

    /// Create a pattern matcher that can be reused for multiple checks
    ///
    /// DRY: Simplified pattern matcher using standard glob functionality
    ///
    /// # Returns
    /// A closure that can be used to check if a package matches the pattern
    ///
    /// # Examples
    /// ```ignore
    /// let matcher = manager.create_pattern_matcher("packages/*")?;
    /// assert!(matcher("packages/core"));
    /// assert!(!matcher("apps/core"));
    /// ```
    pub fn create_pattern_matcher(&self, pattern: &str) -> Result<PatternMatcher> {
        let normalized_pattern = pattern.replace('\\', "/");

        // DRY: Simplified pattern compilation using standard functionality
        let glob_pattern = Pattern::new(&normalized_pattern)
            .map_err(|e| Error::config(format!("Invalid glob pattern '{pattern}': {e}")))?;
        
        let glob_pattern = Arc::new(glob_pattern);
        
        Ok(Box::new(move |package_path: &str| {
            let normalized_path = package_path.replace('\\', "/");
            glob_pattern.matches(&normalized_path)
        }))
    }

    /// Load configuration from a root path
    ///
    /// Loads configuration from the standard monorepo.toml file in the given path.
    /// If no configuration file exists, returns default configuration.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root path of the monorepo
    ///
    /// # Returns
    ///
    /// Loaded configuration or default if file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file exists but is malformed.
    pub fn load_config(&self, root_path: &Path) -> Result<MonorepoConfig> {
        let config_path = root_path.join("monorepo.toml");
        
        if config_path.exists() {
            let persistence = ConfigPersistence::new();
            persistence.load_from_file(&config_path)
        } else {
            // Return default configuration if no file exists
            Ok(MonorepoConfig::default())
        }
    }

    /// Validate a configuration
    ///
    /// Performs comprehensive validation of a configuration object to ensure
    /// all settings are valid and consistent.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration to validate
    ///
    /// # Returns
    ///
    /// Success if configuration is valid.
    ///
    /// # Errors
    ///
    /// Returns an error with validation details if configuration is invalid.
    pub fn validate_config(&self, config: &MonorepoConfig) -> Result<()> {
        // Validate workspace patterns
        for pattern in &config.workspace.patterns {
            if pattern.pattern.is_empty() {
                return Err(Error::config_validation("Workspace pattern cannot be empty"));
            }
            
            // Validate that pattern is a valid glob
            if let Err(e) = Pattern::new(&pattern.pattern) {
                return Err(Error::config_validation(format!(
                    "Invalid workspace pattern '{}': {}", 
                    pattern.pattern, e
                )));
            }
        }

        // Validate version constraints if any are specified
        if let Some(ref constraint) = config.versioning.version_constraint {
            if constraint.is_empty() {
                return Err(Error::config_validation("Version constraint cannot be empty"));
            }
        }

        // Validate hook configuration
        if config.hooks.enabled {
            // Validate pre-commit hook
            if config.hooks.pre_commit.enabled && config.hooks.pre_commit.run_tasks.is_empty() && config.hooks.pre_commit.custom_script.is_none() {
                return Err(Error::config_validation("Pre-commit hook must have either tasks or custom script"));
            }
            
            // Validate pre-push hook
            if config.hooks.pre_push.enabled && config.hooks.pre_push.run_tasks.is_empty() && config.hooks.pre_push.custom_script.is_none() {
                return Err(Error::config_validation("Pre-push hook must have either tasks or custom script"));
            }
            
            // Validate post-merge hook if present
            if let Some(ref post_merge) = config.hooks.post_merge {
                if post_merge.enabled && post_merge.run_tasks.is_empty() && post_merge.custom_script.is_none() {
                    return Err(Error::config_validation("Post-merge hook must have either tasks or custom script"));
                }
            }
        }

        // Validate changeset directory exists or can be created
        if config.changesets.changeset_dir.as_os_str().is_empty() {
            return Err(Error::config_validation("Changeset directory cannot be empty"));
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
