//! Configurable change detection engine

use super::types::{
    ChangeDetectionRules, ChangeTypeRule, FilePattern, PatternType, RuleConditions,
    SignificanceRule, ChangeDetectionEngine,
};
use super::types::engine::CompiledPattern;
use crate::core::MonorepoPackageInfo;
use glob::Pattern;
use log::warn;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use sublime_git_tools::GitChangedFile;
use sublime_standard_tools::filesystem::FileSystem;

impl ChangeDetectionEngine {
    /// Create a new engine with default rules
    #[must_use]
    pub fn new() -> Self {
        Self::with_rules(ChangeDetectionRules::default())
    }

    /// Create engine with custom rules
    #[must_use]
    pub fn with_rules(rules: ChangeDetectionRules) -> Self {
        Self { rules, regex_cache: HashMap::new(), glob_cache: HashMap::new() }
    }

    /// Validate all patterns in the rules and return any errors found
    #[must_use]
    pub fn validate_rules(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate change type rules
        for rule in &self.rules.change_type_rules {
            for pattern in &rule.patterns {
                if let Err(e) = Self::validate_pattern(pattern) {
                    errors.push(format!("Rule '{rule_name}': {e}", rule_name = rule.name));
                }
            }
        }

        // Validate significance rules
        for rule in &self.rules.significance_rules {
            for pattern in &rule.patterns {
                if let Err(e) = Self::validate_pattern(pattern) {
                    errors.push(format!("Rule '{rule_name}': {e}", rule_name = rule.name));
                }
            }
        }

        errors
    }

    /// Validate a single pattern
    fn validate_pattern(pattern: &FilePattern) -> Result<(), String> {
        match &pattern.pattern_type {
            PatternType::Glob => Pattern::new(&pattern.pattern)
                .map(|_| ())
                .map_err(|e| format!("Invalid glob pattern '{pattern}': {e}", pattern = pattern.pattern)),
            PatternType::Regex => Regex::new(&pattern.pattern)
                .map(|_| ())
                .map_err(|e| format!("Invalid regex pattern '{pattern}': {e}", pattern = pattern.pattern)),
            _ => Ok(()), // Other pattern types don't need validation
        }
    }

    /// Load rules from configuration file
    pub fn from_config_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = sublime_standard_tools::filesystem::FileSystemManager::new();
        let content = fs.read_file_string(path)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let rules: ChangeDetectionRules =
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                serde_yaml::from_str(&content)?
            } else {
                serde_json::from_str(&content)?
            };

        let engine = Self::with_rules(rules);

        // Validate rules after loading
        let validation_errors = engine.validate_rules();
        if !validation_errors.is_empty() {
            warn!("Configuration loaded with {} invalid patterns:", validation_errors.len());
            for error in &validation_errors {
                warn!("  - {}", error);
            }
        }

        Ok(engine)
    }

    /// Determine change type for a set of files
    pub fn determine_change_type(
        &mut self,
        files: &[GitChangedFile],
        package: &MonorepoPackageInfo,
    ) -> PackageChangeType {
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
    pub fn analyze_significance(
        &mut self,
        files: &[GitChangedFile],
        package: &MonorepoPackageInfo,
    ) -> ChangeSignificance {
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
        ChangeSignificance::Low
    }

    /// Suggest version bump
    #[must_use]
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
            ChangeSignificance::High => VersionBumpType::Major,
            ChangeSignificance::Medium => VersionBumpType::Minor,
            ChangeSignificance::Low => VersionBumpType::Patch,
        }
    }

    /// Evaluate a change type rule
    fn evaluate_change_type_rule(
        &mut self,
        rule: &ChangeTypeRule,
        files: &[GitChangedFile],
        package: &MonorepoPackageInfo,
    ) -> bool {
        let matching_files: Vec<_> = files
            .iter()
            .filter(|file| self.file_matches_patterns(&rule.patterns, &file.path, package))
            .collect();

        if matching_files.is_empty() {
            return false;
        }

        // Check additional conditions
        if let Some(conditions) = &rule.conditions {
            if !Self::evaluate_conditions(conditions, &matching_files) {
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
        let matching_files: Vec<_> = files
            .iter()
            .filter(|file| {
                // Check pattern match
                let pattern_match = self.file_matches_patterns(&rule.patterns, &file.path, package);

                // Check git status if specified
                let status_match = rule
                    .git_status
                    .as_ref()
                    .map_or(true, |statuses| statuses.contains(&file.status));

                pattern_match && status_match
            })
            .collect();

        if matching_files.is_empty() {
            return false;
        }

        // Check additional conditions
        if let Some(conditions) = &rule.conditions {
            if !Self::evaluate_conditions(conditions, &matching_files) {
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
        let relative_path = if let Ok(stripped) =
            std::path::Path::new(file_path).strip_prefix(package.relative_path())
        {
            stripped.to_string_lossy().to_string()
        } else {
            file_path.to_string()
        };

        let mut matches = false;

        for pattern in patterns {
            let pattern_matches = match &pattern.pattern_type {
                PatternType::Glob => {
                    let compiled =
                        self.glob_cache.entry(pattern.pattern.clone()).or_insert_with(|| {
                            match Pattern::new(&pattern.pattern) {
                                Ok(glob) => CompiledPattern::Valid(glob),
                                Err(e) => {
                                    warn!(
                                        "Invalid glob pattern '{}': {}. Pattern will never match.",
                                        pattern.pattern, e
                                    );
                                    CompiledPattern::Invalid(())
                                }
                            }
                        });

                    match compiled {
                        CompiledPattern::Valid(glob) => glob.matches(&relative_path),
                        CompiledPattern::Invalid(()) => false,
                    }
                }
                PatternType::Regex => {
                    let compiled =
                        self.regex_cache.entry(pattern.pattern.clone()).or_insert_with(|| {
                            match Regex::new(&pattern.pattern) {
                                Ok(regex) => CompiledPattern::Valid(regex),
                                Err(e) => {
                                    warn!(
                                        "Invalid regex pattern '{}': {}. Pattern will never match.",
                                        pattern.pattern, e
                                    );
                                    CompiledPattern::Invalid(())
                                }
                            }
                        });

                    match compiled {
                        CompiledPattern::Valid(regex) => regex.is_match(&relative_path),
                        CompiledPattern::Invalid(()) => false,
                    }
                }
                PatternType::Contains => relative_path.contains(&pattern.pattern),
                PatternType::Extension => std::path::Path::new(&relative_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext.eq_ignore_ascii_case(&pattern.pattern)),
                PatternType::Exact => relative_path == pattern.pattern,
            };

            if pattern.exclude {
                if pattern_matches {
                    return false; // Excluded pattern matched
                }
            } else if pattern_matches {
                matches = true;
            }
        }

        matches
    }

    /// Evaluate additional rule conditions
    fn evaluate_conditions(conditions: &RuleConditions, files: &[&GitChangedFile]) -> bool {
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
        if let Some(file_size) = &conditions.file_size {
            log::debug!("Evaluating file size conditions for {} files", files.len());
            
            let mut total_size = 0u64;
            let mut largest_file_size = 0u64;
            
            // Calculate file sizes for all changed files
            for file in files {
                let file_path = Path::new(&file.path);
                
                // Only check size for existing files (not deleted ones)
                if file_path.exists() {
                    match std::fs::metadata(file_path) {
                        Ok(metadata) => {
                            let size = metadata.len();
                            total_size += size;
                            largest_file_size = largest_file_size.max(size);
                            log::trace!("File {} has size {} bytes", file.path, size);
                        }
                        Err(e) => {
                            log::warn!("Failed to get size for file {}: {}", file.path, e);
                            // Continue with other files rather than failing
                        }
                    }
                }
            }
            
            log::debug!("Total size: {} bytes, largest file: {} bytes", total_size, largest_file_size);
            
            // Check minimum total size constraint
            if let Some(min_total_size) = file_size.min_total_size {
                if total_size < min_total_size {
                    log::debug!("Total size {} < min required {}, condition failed", total_size, min_total_size);
                    return false;
                }
            }
            
            // Check maximum total size constraint
            if let Some(max_total_size) = file_size.max_total_size {
                if total_size > max_total_size {
                    log::debug!("Total size {} > max allowed {}, condition failed", total_size, max_total_size);
                    return false;
                }
            }
            
            // Check minimum largest file size constraint
            if let Some(min_largest_file) = file_size.min_largest_file {
                if largest_file_size < min_largest_file {
                    log::debug!("Largest file size {} < min required {}, condition failed", largest_file_size, min_largest_file);
                    return false;
                }
            }
            
            log::debug!("All file size conditions passed");
        }

        // Custom script execution (if specified)
        if let Some(script) = &conditions.custom_script {
            log::debug!("Executing custom validation script: {}", script);
            
            // Prepare environment variables for the script
            let changed_files = files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>().join(",");
            let file_count = files.len().to_string();
            
            // Create and execute the command
            let mut command = if script.contains(' ') || script.contains('|') || script.contains(';') {
                // Complex script - run through shell
                let shell = if cfg!(windows) { "cmd" } else { "sh" };
                let shell_flag = if cfg!(windows) { "/C" } else { "-c" };
                let mut cmd = std::process::Command::new(shell);
                cmd.arg(shell_flag).arg(script);
                cmd
            } else {
                // Simple command
                std::process::Command::new(script)
            };
            
            // Add environment variables
            command
                .env("CHANGED_FILES", &changed_files)
                .env("FILE_COUNT", &file_count);
            
            // Execute the script
            match command.output() {
                Ok(output) => {
                    if output.status.success() {
                        log::debug!("Custom script succeeded: {}", String::from_utf8_lossy(&output.stdout));
                    } else {
                        log::debug!("Custom script failed with exit code {}: {}", 
                            output.status.code().unwrap_or(-1),
                            String::from_utf8_lossy(&output.stderr));
                        return false;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to execute custom script '{}': {}", script, e);
                    return false;
                }
            }
            
            log::debug!("Custom script validation passed");
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
pub use super::types::{ChangeSignificance, PackageChangeType, VersionBumpType};
