//! Configuration manager for monorepo tools

use crate::config::{
    ChangelogConfig, ChangesetsConfig, HooksConfig, PackageManagerType, PluginsConfig, TasksConfig,
    VersioningConfig, WorkspaceConfig, WorkspacePattern, ConfigManager, PatternMatcher,
};
use crate::error::{Error, Result};
use crate::{Environment, MonorepoConfig};
use glob::Pattern;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use sublime_standard_tools::filesystem::{FileSystem, FileSystemManager};

impl ConfigManager {
    /// Create a new configuration manager with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(MonorepoConfig::default())),
            config_path: None,
            auto_save: false,
        }
    }

    /// Create a configuration manager with a specific config
    #[must_use]
    pub fn with_config(config: MonorepoConfig) -> Self {
        Self { config: Arc::new(RwLock::new(config)), config_path: None, auto_save: false }
    }

    /// Load configuration from a file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // DRY: Use FileSystemManager instead of manual std::fs operations
        let fs = FileSystemManager::new();
        let content = fs.read_file_string(path)
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

        // DRY: Use FileSystemManager instead of manual std::fs operations
        let fs = FileSystemManager::new();
        fs.write_file_string(path, &content)
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
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Create default configuration files in a directory
    pub fn create_default_config_files(dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        let config = MonorepoConfig::default();

        // DRY: Use FileSystemManager instead of manual std::fs operations
        let fs = FileSystemManager::new();
        
        // Create .monorepo directory
        let config_dir = dir.join(".monorepo");
        fs.create_dir_all(&config_dir)
            .map_err(|e| Error::config(format!("Failed to create config directory: {e}")))?;

        // Save as JSON
        let json_path = config_dir.join("config.json");
        let json_content = serde_json::to_string_pretty(&config)?;
        fs.write_file_string(&json_path, &json_content)
            .map_err(|e| Error::config(format!("Failed to write JSON config: {e}")))?;

        // Save as TOML (alternative)
        let toml_path = config_dir.join("config.toml");
        let toml_content = toml::to_string_pretty(&config)
            .map_err(|e| Error::config(format!("Failed to serialize to TOML: {e}")))?;
        fs.write_file_string(&toml_path, &toml_content)
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

        // DRY: Use FileSystemManager for file existence checks
        let fs = FileSystemManager::new();
        
        // Check current directory and parent directories
        let mut current = start_dir.to_path_buf();
        loop {
            for config_name in &config_names {
                let config_path = current.join(config_name);
                if fs.exists(&config_path) {
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
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
