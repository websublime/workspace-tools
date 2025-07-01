//! Workspace pattern management component
//!
//! Handles workspace pattern operations including pattern matching, validation,
//! and workspace configuration management.

use crate::config::{WorkspacePattern, WorkspaceConfig, PackageManagerType};
use crate::error::{Error, Result};
use glob::Pattern;
use std::collections::HashMap;

/// Component responsible for workspace pattern management
pub struct WorkspacePatternManager {
    workspace_config: WorkspaceConfig,
}

impl WorkspacePatternManager {
    /// Create a new workspace pattern manager
    #[must_use]
    pub fn new(workspace_config: WorkspaceConfig) -> Self {
        Self { workspace_config }
    }

    /// Get all workspace patterns
    ///
    /// # Returns
    /// Vector of all configured workspace patterns
    #[must_use]
    pub fn get_workspace_patterns(&self) -> Vec<&WorkspacePattern> {
        self.workspace_config.patterns.iter().collect()
    }

    /// Get effective workspace patterns (enabled patterns only)
    ///
    /// # Returns
    /// Vector of enabled workspace patterns
    #[must_use]
    pub fn get_effective_workspace_patterns(&self) -> Vec<&WorkspacePattern> {
        self.workspace_config
            .patterns
            .iter()
            .filter(|pattern| pattern.enabled)
            .collect()
    }

    /// Add a workspace pattern
    ///
    /// # Arguments
    /// * `pattern` - Workspace pattern to add
    ///
    /// # Returns
    /// Updated workspace pattern manager
    #[must_use]
    pub fn with_workspace_pattern(mut self, pattern: WorkspacePattern) -> Self {
        self.workspace_config.patterns.push(pattern);
        self
    }

    /// Add a workspace pattern in place
    ///
    /// # Arguments
    /// * `pattern` - Workspace pattern to add
    ///
    /// # Errors
    /// Returns an error if the pattern is invalid
    pub fn add_workspace_pattern(&mut self, pattern: WorkspacePattern) -> Result<()> {
        // Validate pattern
        if pattern.pattern.is_empty() {
            return Err(Error::config("Workspace pattern cannot be empty"));
        }

        // Test if pattern is valid glob
        Pattern::new(&pattern.pattern)
            .map_err(|e| Error::config(format!("Invalid glob pattern '{pattern}': {e}", pattern = pattern.pattern)))?;

        self.workspace_config.patterns.push(pattern);
        Ok(())
    }

    /// Remove a workspace pattern by pattern string
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to remove
    ///
    /// # Returns
    /// Tuple of (updated manager, whether pattern was found and removed)
    #[must_use]
    pub fn without_workspace_pattern(mut self, pattern: &str) -> (Self, bool) {
        let initial_len = self.workspace_config.patterns.len();
        self.workspace_config.patterns.retain(|p| p.pattern != pattern);
        let removed = self.workspace_config.patterns.len() < initial_len;
        (self, removed)
    }

    /// Remove a workspace pattern by pattern string in place
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to remove
    ///
    /// # Returns
    /// True if a pattern was removed
    ///
    /// # Errors
    /// Returns an error if the pattern is not found
    pub fn remove_workspace_pattern(&mut self, pattern: &str) -> Result<bool> {
        let initial_len = self.workspace_config.patterns.len();
        self.workspace_config.patterns.retain(|p| p.pattern != pattern);
        let removed = self.workspace_config.patterns.len() < initial_len;
        
        if removed {
            log::debug!("Removed workspace pattern: {}", pattern);
        }
        
        Ok(removed)
    }

    /// Update a workspace pattern
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to update
    /// * `updater` - Function to update the pattern
    ///
    /// # Returns
    /// Tuple of (updated manager, whether pattern was found and updated)
    #[must_use]
    pub fn with_updated_workspace_pattern<F>(mut self, pattern: &str, updater: F) -> (Self, bool)
    where
        F: FnOnce(&mut WorkspacePattern),
    {
        let mut found = false;
        for workspace_pattern in &mut self.workspace_config.patterns {
            if workspace_pattern.pattern == pattern {
                updater(workspace_pattern);
                found = true;
                break;
            }
        }
        (self, found)
    }

    /// Update a workspace pattern in place
    ///
    /// # Arguments
    /// * `pattern` - Pattern string to update
    /// * `updater` - Function to update the pattern
    ///
    /// # Returns
    /// True if a pattern was found and updated
    ///
    /// # Errors
    /// Returns an error if the pattern is not found
    pub fn update_workspace_pattern<F>(&mut self, pattern: &str, updater: F) -> Result<bool>
    where
        F: FnOnce(&mut WorkspacePattern),
    {
        for workspace_pattern in &mut self.workspace_config.patterns {
            if workspace_pattern.pattern == pattern {
                updater(workspace_pattern);
                log::debug!("Updated workspace pattern: {}", pattern);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get package manager specific patterns
    ///
    /// # Arguments
    /// * `package_manager` - Package manager type to get patterns for
    ///
    /// # Returns
    /// Vector of patterns specific to the package manager
    #[must_use]
    pub fn get_package_manager_patterns(&self, package_manager: &PackageManagerType) -> Vec<String> {
        self.workspace_config
            .patterns
            .iter()
            .filter(|pattern| {
                pattern.package_managers
                    .as_ref()
                    .map_or(false, |managers| managers.contains(package_manager))
            })
            .map(|pattern| pattern.pattern.clone())
            .collect()
    }

    /// Validate workspace configuration against existing packages
    ///
    /// # Arguments
    /// * `existing_packages` - List of existing package paths
    ///
    /// # Returns
    /// Vector of validation warnings
    #[must_use]
    pub fn validate_workspace_config(&self, existing_packages: &[String]) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check if patterns are too broad or too narrow
        let effective_patterns = self.get_effective_workspace_patterns();
        
        if effective_patterns.is_empty() {
            warnings.push("No enabled workspace patterns found".to_string());
            return warnings;
        }

        // Check pattern coverage
        let mut matched_packages = 0;
        for package_path in existing_packages {
            let mut package_matched = false;
            for pattern in &effective_patterns {
                if self.pattern_matches_package(&pattern.pattern, package_path) {
                    package_matched = true;
                    break;
                }
            }
            if package_matched {
                matched_packages += 1;
            }
        }

        if matched_packages == 0 && !existing_packages.is_empty() {
            warnings.push("No workspace patterns match existing packages".to_string());
        } else if matched_packages < existing_packages.len() {
            let unmatched = existing_packages.len() - matched_packages;
            warnings.push(format!("{unmatched} packages don't match any workspace pattern"));
        }

        // Check for duplicate patterns
        let mut pattern_counts = HashMap::new();
        for pattern in &effective_patterns {
            *pattern_counts.entry(&pattern.pattern).or_insert(0) += 1;
        }
        
        for (pattern, count) in pattern_counts {
            if count > 1 {
                warnings.push(format!("Duplicate workspace pattern: {pattern}"));
            }
        }

        warnings
    }

    /// Check if a pattern matches a package path
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern to test
    /// * `package_path` - Package path to test against
    ///
    /// # Returns
    /// True if the pattern matches the package path
    #[must_use]
    pub fn pattern_matches_package(&self, pattern: &str, package_path: &str) -> bool {
        match Pattern::new(pattern) {
            Ok(glob_pattern) => {
                // Try exact match first
                if glob_pattern.matches(package_path) {
                    return true;
                }
                
                // Try with trailing slash for directories
                let package_path_with_slash = format!("{package_path}/", package_path = package_path.trim_end_matches('/'));
                if glob_pattern.matches(&package_path_with_slash) {
                    return true;
                }
                
                // Try matching just the package name (last component)
                if let Some(package_name) = package_path.split('/').last() {
                    if glob_pattern.matches(package_name) {
                        return true;
                    }
                }
                
                false
            }
            Err(_) => {
                log::warn!("Invalid glob pattern: {}", pattern);
                false
            }
        }
    }

    /// Batch check if patterns match packages
    ///
    /// # Arguments
    /// * `patterns` - List of patterns to test
    /// * `package_paths` - List of package paths to test against
    ///
    /// # Returns
    /// Map of pattern -> list of matching package paths
    #[must_use]
    pub fn batch_pattern_matches(
        &self,
        patterns: &[String],
        package_paths: &[String],
    ) -> HashMap<String, Vec<String>> {
        let mut results = HashMap::new();
        
        for pattern in patterns {
            let mut matches = Vec::new();
            for package_path in package_paths {
                if self.pattern_matches_package(pattern, package_path) {
                    matches.push(package_path.clone());
                }
            }
            results.insert(pattern.clone(), matches);
        }
        
        results
    }

    /// Create a pattern matcher for a specific pattern
    ///
    /// # Arguments
    /// * `pattern` - Pattern to create matcher for
    ///
    /// # Returns
    /// Pattern matcher that can be used for efficient repeated matching
    ///
    /// # Errors
    /// Returns an error if the pattern is invalid
    pub fn create_pattern_matcher(&self, pattern: &str) -> Result<crate::config::components::PatternMatcher> {
        crate::config::components::PatternMatcher::from_str(pattern)
    }

    /// Get workspace configuration
    #[must_use]
    pub fn workspace_config(&self) -> &WorkspaceConfig {
        &self.workspace_config
    }

    /// Get mutable workspace configuration
    #[must_use]
    pub fn workspace_config_mut(&mut self) -> &mut WorkspaceConfig {
        &mut self.workspace_config
    }

    /// Consume and return the workspace configuration
    #[must_use]
    pub fn into_workspace_config(self) -> WorkspaceConfig {
        self.workspace_config
    }

    /// Get statistics about workspace patterns
    ///
    /// # Returns
    /// Tuple of (total_patterns, enabled_patterns, package_managers_count)
    #[must_use]
    pub fn get_pattern_stats(&self) -> (usize, usize, usize) {
        let total_patterns = self.workspace_config.patterns.len();
        let enabled_patterns = self.workspace_config.patterns.iter()
            .filter(|p| p.enabled)
            .count();
        let package_managers: std::collections::HashSet<_> = self.workspace_config.patterns.iter()
            .filter_map(|p| p.package_managers.as_ref())
            .flatten()
            .collect();
        
        (total_patterns, enabled_patterns, package_managers.len())
    }
}