//! Validation configuration types
//!
//! This module defines configuration structures for validation rules, quality gates,
//! and threshold values used throughout the monorepo analysis and validation processes.
//! 
//! ## What
//! Provides comprehensive validation configuration including:
//! - Task priority levels and scoring
//! - Change detection rule priorities and weights
//! - Version bump rule priorities and thresholds
//! - Dependency analysis limits and thresholds
//! - Pattern scoring algorithms and weights
//! - Validation patterns for commits, branches, files
//! - Quality gate thresholds and limits
//! 
//! ## How
//! Uses structured configuration with defaults based on best practices:
//! - Priority-based rule systems for change detection and version bumps
//! - Configurable thresholds for analysis depth and complexity limits
//! - Pattern-based validation for naming conventions and structures
//! - Quality gates with customizable pass/fail criteria
//! 
//! ## Why
//! Centralizes all validation rules and thresholds that were previously
//! hardcoded throughout the codebase, enabling users to tune validation
//! behavior based on their specific project requirements and quality standards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Task priority level configuration
    pub task_priorities: TaskPriorityConfig,

    /// Change detection rules and priorities
    pub change_detection_rules: ChangeDetectionRulesConfig,

    /// Version bump rules and priorities
    pub version_bump_rules: VersionBumpRulesConfig,

    /// Dependency analysis thresholds and limits
    pub dependency_analysis: DependencyAnalysisConfig,

    /// Pattern scoring algorithms and weights
    pub pattern_scoring: PatternScoringConfig,

    /// Validation patterns for various checks
    pub validation_patterns: ValidationPatternsConfig,

    /// Quality gates and thresholds
    pub quality_gates: QualityGatesConfig,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            task_priorities: TaskPriorityConfig::default(),
            change_detection_rules: ChangeDetectionRulesConfig::default(),
            version_bump_rules: VersionBumpRulesConfig::default(),
            dependency_analysis: DependencyAnalysisConfig::default(),
            pattern_scoring: PatternScoringConfig::default(),
            validation_patterns: ValidationPatternsConfig::default(),
            quality_gates: QualityGatesConfig::default(),
        }
    }
}

/// Task priority level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPriorityConfig {
    /// Low priority task value
    pub low: u32,

    /// Normal priority task value
    pub normal: u32,

    /// High priority task value
    pub high: u32,

    /// Critical priority task value
    pub critical: u32,
}

impl Default for TaskPriorityConfig {
    fn default() -> Self {
        Self {
            low: 0,
            normal: 50,
            high: 100,
            critical: 200,
        }
    }
}

/// Change detection rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionRulesConfig {
    /// Priority for dependency changes
    pub dependency_changes_priority: u32,

    /// Priority for source code changes
    pub source_code_changes_priority: u32,

    /// Priority for configuration changes
    pub configuration_changes_priority: u32,

    /// Priority for test changes
    pub test_changes_priority: u32,

    /// Priority for documentation changes
    pub documentation_changes_priority: u32,

    /// Significance priorities
    pub significance_priorities: ChangeSignificancePriorities,
}

impl Default for ChangeDetectionRulesConfig {
    fn default() -> Self {
        Self {
            dependency_changes_priority: 100,
            source_code_changes_priority: 80,
            configuration_changes_priority: 70,
            test_changes_priority: 60,
            documentation_changes_priority: 50,
            significance_priorities: ChangeSignificancePriorities::default(),
        }
    }
}

/// Change significance priorities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSignificancePriorities {
    /// Priority for public API changes
    pub public_api_changes: u32,

    /// Priority for internal changes
    pub internal_changes: u32,

    /// Priority for test changes
    pub test_changes: u32,

    /// Priority for documentation changes
    pub documentation_changes: u32,
}

impl Default for ChangeSignificancePriorities {
    fn default() -> Self {
        Self {
            public_api_changes: 100,
            internal_changes: 80,
            test_changes: 60,
            documentation_changes: 40,
        }
    }
}

/// Version bump rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBumpRulesConfig {
    /// Priority for breaking changes
    pub breaking_changes_priority: u32,

    /// Priority for feature changes
    pub feature_changes_priority: u32,

    /// Priority for dependency changes
    pub dependency_changes_priority: u32,

    /// Priority for patch changes
    pub patch_changes_priority: u32,

    /// Priority for test/documentation changes
    pub test_documentation_priority: u32,

    /// Priority for documentation only changes
    pub documentation_only_priority: u32,
}

impl Default for VersionBumpRulesConfig {
    fn default() -> Self {
        Self {
            breaking_changes_priority: 100,
            feature_changes_priority: 80,
            dependency_changes_priority: 70,
            patch_changes_priority: 60,
            test_documentation_priority: 50,
            documentation_only_priority: 40,
        }
    }
}

/// Dependency analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisConfig {
    /// Maximum dependency chain depth
    pub max_chain_depth: usize,

    /// Maximum propagation depth for changes
    pub max_propagation_depth: usize,

    /// Maximum analysis depth for complex chains
    pub max_analysis_depth: usize,

    /// Threshold for considering a dependency complex
    pub complex_dependency_threshold: usize,

    /// Maximum number of dependents to analyze
    pub max_dependents_analysis: usize,
}

impl Default for DependencyAnalysisConfig {
    fn default() -> Self {
        Self {
            max_chain_depth: 5,
            max_propagation_depth: 5,
            max_analysis_depth: 10,
            complex_dependency_threshold: 20,
            max_dependents_analysis: 100,
        }
    }
}

/// Pattern scoring algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternScoringConfig {
    /// Multiplier for path component specificity
    pub path_component_multiplier: u32,

    /// Bonus for exact matches
    pub exact_match_bonus: u32,

    /// Base penalty for wildcards
    pub wildcard_base_penalty: u32,

    /// Penalty multiplier per wildcard
    pub wildcard_penalty_multiplier: u32,

    /// Bonus for non-wildcard components
    pub non_wildcard_bonus: u32,

    /// Multiplier for specific parts
    pub specific_parts_multiplier: u32,
}

impl Default for PatternScoringConfig {
    fn default() -> Self {
        Self {
            path_component_multiplier: 10,
            exact_match_bonus: 100,
            wildcard_base_penalty: 50,
            wildcard_penalty_multiplier: 10,
            non_wildcard_bonus: 5,
            specific_parts_multiplier: 3,
        }
    }
}

/// Validation patterns configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPatternsConfig {
    /// Required dependency files
    pub required_dependency_files: Vec<String>,

    /// Valid branch naming patterns
    pub branch_naming_patterns: Vec<String>,

    /// Valid conventional commit types
    pub conventional_commit_types: Vec<String>,

    /// Required package files for validation
    pub required_package_files: Vec<String>,

    /// File patterns that require special validation
    pub special_validation_patterns: Vec<String>,

    /// Patterns for security-sensitive files
    pub security_sensitive_patterns: Vec<String>,
}

impl Default for ValidationPatternsConfig {
    fn default() -> Self {
        Self {
            required_dependency_files: vec![
                "package.json".to_string(),
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
            ],
            branch_naming_patterns: vec![
                "feature/*".to_string(),
                "fix/*".to_string(),
                "hotfix/*".to_string(),
                "release/*".to_string(),
                "feat/*".to_string(),
                "bugfix/*".to_string(),
                "chore/*".to_string(),
            ],
            conventional_commit_types: vec![
                "feat".to_string(),
                "fix".to_string(),
                "docs".to_string(),
                "style".to_string(),
                "refactor".to_string(),
                "test".to_string(),
                "chore".to_string(),
                "perf".to_string(),
                "ci".to_string(),
                "build".to_string(),
                "revert".to_string(),
            ],
            required_package_files: vec![
                "package.json".to_string(),
                "README.md".to_string(),
            ],
            special_validation_patterns: vec![
                "**/*.config.{js,ts,json}".to_string(),
                "**/Dockerfile*".to_string(),
                "**/.env*".to_string(),
                "**/docker-compose*.{yml,yaml}".to_string(),
            ],
            security_sensitive_patterns: vec![
                "**/*.key".to_string(),
                "**/*.pem".to_string(),
                "**/*.p12".to_string(),
                "**/*.pfx".to_string(),
                "**/*secret*".to_string(),
                "**/*password*".to_string(),
                "**/*token*".to_string(),
            ],
        }
    }
}

/// Quality gates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesConfig {
    /// Minimum test coverage percentage required
    pub min_test_coverage: f32,

    /// Maximum cyclomatic complexity allowed
    pub max_cyclomatic_complexity: u32,

    /// Maximum file size in bytes
    pub max_file_size_bytes: u64,

    /// Maximum lines per file
    pub max_lines_per_file: u32,

    /// Maximum dependencies per package
    pub max_dependencies_per_package: u32,

    /// Minimum documentation coverage percentage
    pub min_documentation_coverage: f32,

    /// Maximum build time in seconds
    pub max_build_time_seconds: u64,

    /// Maximum technical debt ratio
    pub max_technical_debt_ratio: f32,

    /// Security scan thresholds
    pub security_thresholds: SecurityThresholds,
}

impl Default for QualityGatesConfig {
    fn default() -> Self {
        Self {
            min_test_coverage: 80.0,
            max_cyclomatic_complexity: 10,
            max_file_size_bytes: 100_000, // 100KB
            max_lines_per_file: 1000,
            max_dependencies_per_package: 50,
            min_documentation_coverage: 70.0,
            max_build_time_seconds: 600, // 10 minutes
            max_technical_debt_ratio: 0.05, // 5%
            security_thresholds: SecurityThresholds::default(),
        }
    }
}

/// Security validation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityThresholds {
    /// Maximum allowed high severity vulnerabilities
    pub max_high_severity_vulnerabilities: u32,

    /// Maximum allowed medium severity vulnerabilities
    pub max_medium_severity_vulnerabilities: u32,

    /// Maximum allowed outdated dependencies percentage
    pub max_outdated_dependencies_percentage: f32,

    /// Days before a dependency is considered outdated
    pub outdated_dependency_days: u32,

    /// Maximum allowed license issues
    pub max_license_issues: u32,
}

impl Default for SecurityThresholds {
    fn default() -> Self {
        Self {
            max_high_severity_vulnerabilities: 0,
            max_medium_severity_vulnerabilities: 5,
            max_outdated_dependencies_percentage: 20.0,
            outdated_dependency_days: 365,
            max_license_issues: 0,
        }
    }
}

impl ValidationConfig {
    /// Get task priority value by name
    #[must_use]
    pub fn get_task_priority(&self, priority_name: &str) -> u32 {
        match priority_name.to_lowercase().as_str() {
            "low" => self.task_priorities.low,
            "normal" => self.task_priorities.normal,
            "high" => self.task_priorities.high,
            "critical" => self.task_priorities.critical,
            _ => self.task_priorities.normal,
        }
    }

    /// Check if a branch name follows naming conventions
    #[must_use]
    pub fn is_valid_branch_name(&self, branch_name: &str) -> bool {
        self.validation_patterns.branch_naming_patterns
            .iter()
            .any(|pattern| {
                // Simple pattern matching - would use proper glob library in production
                if pattern.ends_with("/*") {
                    let prefix = &pattern[..pattern.len() - 2];
                    branch_name.starts_with(prefix)
                } else {
                    branch_name == pattern
                }
            })
    }

    /// Check if a commit type is valid according to conventional commits
    #[must_use]
    pub fn is_valid_commit_type(&self, commit_type: &str) -> bool {
        self.validation_patterns.conventional_commit_types
            .contains(&commit_type.to_string())
    }

    /// Check if a file is security-sensitive
    #[must_use]
    pub fn is_security_sensitive_file(&self, file_path: &str) -> bool {
        self.validation_patterns.security_sensitive_patterns
            .iter()
            .any(|pattern| {
                // Simplified pattern matching
                let clean_pattern = pattern.replace("**/", "").replace("*", "");
                file_path.contains(&clean_pattern)
            })
    }

    /// Validate if quality gates are met
    #[must_use]
    pub fn validate_quality_gates(&self, metrics: &QualityMetrics) -> Vec<String> {
        let mut violations = Vec::new();

        if metrics.test_coverage < self.quality_gates.min_test_coverage {
            violations.push(format!(
                "Test coverage {:.1}% is below minimum {:.1}%",
                metrics.test_coverage, self.quality_gates.min_test_coverage
            ));
        }

        if metrics.cyclomatic_complexity > self.quality_gates.max_cyclomatic_complexity {
            violations.push(format!(
                "Cyclomatic complexity {} exceeds maximum {}",
                metrics.cyclomatic_complexity, self.quality_gates.max_cyclomatic_complexity
            ));
        }

        if metrics.max_file_size > self.quality_gates.max_file_size_bytes {
            violations.push(format!(
                "File size {} bytes exceeds maximum {} bytes",
                metrics.max_file_size, self.quality_gates.max_file_size_bytes
            ));
        }

        violations
    }
}

/// Quality metrics for validation
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    /// Test coverage percentage
    pub test_coverage: f32,
    /// Maximum cyclomatic complexity found
    pub cyclomatic_complexity: u32,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Maximum lines in a single file
    pub max_lines_per_file: u32,
    /// Number of dependencies
    pub dependency_count: u32,
    /// Documentation coverage percentage
    pub documentation_coverage: f32,
    /// Build time in seconds
    pub build_time_seconds: u64,
    /// Technical debt ratio
    pub technical_debt_ratio: f32,
}