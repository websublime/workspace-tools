//! Configurable change detection engine

use super::change_rules::*;
use crate::core::MonorepoPackageInfo;
use sublime_git_tools::GitChangedFile;
use std::collections::HashMap;
use regex::Regex;
use glob::Pattern;

/// Configurable change detection engine
pub struct ChangeDetectionEngine {
    /// Rules configuration
    rules: ChangeDetectionRules,
    
    /// Compiled regex patterns cache
    regex_cache: HashMap<String, Regex>,
    
    /// Compiled glob patterns cache
    glob_cache: HashMap<String, Pattern>,
}

impl ChangeDetectionEngine {
    /// Create a new engine with default rules
    pub fn new() -> Self {
        Self::with_rules(ChangeDetectionRules::default())
    }
    
    /// Create engine with custom rules
    pub fn with_rules(rules: ChangeDetectionRules) -> Self {
        Self {
            rules,
            regex_cache: HashMap::new(),
            glob_cache: HashMap::new(),
        }
    }
    
    /// Load rules from configuration file
    pub fn from_config_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let rules: ChangeDetectionRules = if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };
        
        Ok(Self::with_rules(rules))
    }
    
    /// Determine change type for a set of files
    pub fn determine_change_type(&mut self, files: &[GitChangedFile], package: &MonorepoPackageInfo) -> PackageChangeType {
        let mut applicable_rules = self.rules.change_type_rules.clone();
        
        // Apply project-specific overrides
        if let Some(overrides) = self.rules.project_overrides.get(package.name()) {
            // Remove disabled rules
            applicable_rules.retain(|rule| !overrides.disabled_rules.contains(&rule.name));
            
            // Add additional rules
            if let Some(additional) = &overrides.additional_rules {
                applicable_rules.extend(additional.change_type_rules.clone());
            }
        }
        
        // Sort by priority (highest first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Evaluate rules
        for rule in applicable_rules {
            if self.evaluate_change_type_rule(&rule, files, package) {
                return rule.change_type;
            }
        }
        
        // Default fallback
        PackageChangeType::SourceCode
    }
    
    /// Analyze change significance
    pub fn analyze_significance(&mut self, files: &[GitChangedFile], package: &MonorepoPackageInfo) -> ChangeSignificance {
        let mut applicable_rules = self.rules.significance_rules.clone();
        
        // Apply project-specific overrides
        if let Some(overrides) = self.rules.project_overrides.get(package.name()) {
            applicable_rules.retain(|rule| !overrides.disabled_rules.contains(&rule.name));
            
            if let Some(additional) = &overrides.additional_rules {
                applicable_rules.extend(additional.significance_rules.clone());
            }
        }
        
        // Sort by priority
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Evaluate rules
        for rule in applicable_rules {
            if self.evaluate_significance_rule(&rule, files, package) {
                return rule.significance;
            }
        }
        
        // Default fallback
        ChangeSignificance::Patch
    }
    
    /// Suggest version bump
    pub fn suggest_version_bump(
        &self,
        change_type: &PackageChangeType,
        significance: &ChangeSignificance,
        package: &MonorepoPackageInfo,
    ) -> VersionBumpType {
        let mut applicable_rules = self.rules.version_bump_rules.clone();
        
        // Apply project-specific overrides
        if let Some(overrides) = self.rules.project_overrides.get(package.name()) {
            if let Some(additional) = &overrides.additional_rules {
                applicable_rules.extend(additional.version_bump_rules.clone());
            }
        }
        
        // Sort by priority
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Find matching rule
        for rule in applicable_rules {
            let type_matches = rule.change_type.as_ref().map_or(true, |t| t == change_type);
            let sig_matches = rule.significance.as_ref().map_or(true, |s| s == significance);
            
            if type_matches && sig_matches {
                return rule.version_bump;
            }
        }
        
        // Default fallback based on significance
        match significance {
            ChangeSignificance::Breaking => VersionBumpType::Major,
            ChangeSignificance::Feature => VersionBumpType::Minor,
            ChangeSignificance::Patch => VersionBumpType::Patch,
        }
    }
    
    /// Evaluate a change type rule
    fn evaluate_change_type_rule(
        &mut self,
        rule: &ChangeTypeRule,
        files: &[GitChangedFile],
        package: &MonorepoPackageInfo,
    ) -> bool {
        let matching_files: Vec<_> = files.iter()
            .filter(|file| self.file_matches_patterns(&rule.patterns, &file.path, package))
            .collect();
            
        if matching_files.is_empty() {
            return false;
        }
        
        // Check additional conditions
        if let Some(conditions) = &rule.conditions {
            if !self.evaluate_conditions(conditions, &matching_files) {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate a significance rule
    fn evaluate_significance_rule(
        &mut self,
        rule: &SignificanceRule,
        files: &[GitChangedFile],
        package: &MonorepoPackageInfo,
    ) -> bool {
        let matching_files: Vec<_> = files.iter()
            .filter(|file| {
                // Check pattern match
                let pattern_match = self.file_matches_patterns(&rule.patterns, &file.path, package);
                
                // Check git status if specified
                let status_match = rule.git_status.as_ref()
                    .map_or(true, |statuses| statuses.contains(&file.status));
                
                pattern_match && status_match
            })
            .collect();
            
        if matching_files.is_empty() {
            return false;
        }
        
        // Check additional conditions
        if let Some(conditions) = &rule.conditions {
            if !self.evaluate_conditions(conditions, &matching_files) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a file matches any of the given patterns
    fn file_matches_patterns(
        &mut self,
        patterns: &[FilePattern],
        file_path: &str,
        package: &MonorepoPackageInfo,
    ) -> bool {
        // Convert absolute path to relative to package
        let relative_path = if let Ok(stripped) = std::path::Path::new(file_path).strip_prefix(package.relative_path()) {
            stripped.to_string_lossy().to_string()
        } else {
            file_path.to_string()
        };
        
        let mut matches = false;
        
        for pattern in patterns {
            let pattern_matches = match &pattern.pattern_type {
                PatternType::Glob => {
                    let glob_pattern = self.glob_cache.entry(pattern.pattern.clone())
                        .or_insert_with(|| Pattern::new(&pattern.pattern).unwrap_or_else(|_| Pattern::new("**").unwrap()));
                    glob_pattern.matches(&relative_path)
                },
                PatternType::Regex => {
                    let regex = self.regex_cache.entry(pattern.pattern.clone())
                        .or_insert_with(|| Regex::new(&pattern.pattern).unwrap_or_else(|_| Regex::new(r".*").unwrap()));
                    regex.is_match(&relative_path)
                },
                PatternType::Contains => relative_path.contains(&pattern.pattern),
                PatternType::Extension => {
                    std::path::Path::new(&relative_path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map_or(false, |ext| ext.eq_ignore_ascii_case(&pattern.pattern))
                },
                PatternType::Exact => relative_path == pattern.pattern,
            };
            
            if pattern.exclude {
                if pattern_matches {
                    return false; // Excluded pattern matched
                }
            } else {
                if pattern_matches {
                    matches = true;
                }
            }
        }
        
        matches
    }
    
    /// Evaluate additional rule conditions
    fn evaluate_conditions(&self, conditions: &RuleConditions, files: &[&GitChangedFile]) -> bool {
        // Check file count conditions
        if let Some(min_files) = conditions.min_files {
            if files.len() < min_files {
                return false;
            }
        }
        
        if let Some(max_files) = conditions.max_files {
            if files.len() > max_files {
                return false;
            }
        }
        
        // Check file size conditions (if available)
        if let Some(_file_size) = &conditions.file_size {
            // File size checking would require additional git info
            // This is a placeholder for more sophisticated analysis
        }
        
        // Custom script execution (if specified)
        if let Some(_script) = &conditions.custom_script {
            // Execute custom validation script
            // This is a placeholder for extensibility
        }
        
        true
    }
}

impl Default for ChangeDetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export types for convenience
pub use super::change_rules::{PackageChangeType, ChangeSignificance, VersionBumpType};