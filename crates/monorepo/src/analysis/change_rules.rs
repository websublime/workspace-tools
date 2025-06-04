//! Configurable change detection rules system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sublime_git_tools::GitFileStatus;

/// Type of changes in a package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageChangeType {
    /// Source code changes
    SourceCode,
    /// Dependency changes
    Dependencies,
    /// Configuration changes
    Configuration,
    /// Documentation changes
    Documentation,
    /// Test changes
    Tests,
}

/// Significance level of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSignificance {
    /// Breaking changes requiring major version bump
    Breaking,
    /// New features requiring minor version bump
    Feature,
    /// Bug fixes or small changes requiring patch bump
    Patch,
}

/// Version bump types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionBumpType {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
}

/// Configurable rules for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionRules {
    /// Rules for determining change types
    pub change_type_rules: Vec<ChangeTypeRule>,
    
    /// Rules for determining significance
    pub significance_rules: Vec<SignificanceRule>,
    
    /// Rules for version bump suggestions
    pub version_bump_rules: Vec<VersionBumpRule>,
    
    /// Project-specific overrides
    pub project_overrides: HashMap<String, ProjectRuleOverrides>,
}

/// Rule for determining change type based on file patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTypeRule {
    /// Name of the rule for debugging
    pub name: String,
    
    /// Priority (higher = evaluated first)
    pub priority: u32,
    
    /// File patterns that trigger this rule
    pub patterns: Vec<FilePattern>,
    
    /// Resulting change type
    pub change_type: PackageChangeType,
    
    /// Conditions that must be met
    pub conditions: Option<RuleConditions>,
}

/// Rule for determining change significance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceRule {
    /// Name of the rule
    pub name: String,
    
    /// Priority (higher = evaluated first)
    pub priority: u32,
    
    /// File patterns and conditions
    pub patterns: Vec<FilePattern>,
    
    /// Git status requirements
    pub git_status: Option<Vec<GitFileStatus>>,
    
    /// Resulting significance
    pub significance: ChangeSignificance,
    
    /// Additional conditions
    pub conditions: Option<RuleConditions>,
}

/// Rule for version bump suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBumpRule {
    /// Name of the rule
    pub name: String,
    
    /// Change type this rule applies to
    pub change_type: Option<PackageChangeType>,
    
    /// Change significance this rule applies to
    pub significance: Option<ChangeSignificance>,
    
    /// Resulting version bump
    pub version_bump: VersionBumpType,
    
    /// Priority for conflicting rules
    pub priority: u32,
}

/// File pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePattern {
    /// Pattern type
    pub pattern_type: PatternType,
    
    /// The pattern itself
    pub pattern: String,
    
    /// Whether the pattern should match or exclude
    pub exclude: bool,
}

/// Types of file patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Glob pattern (e.g., "src/**/*.ts")
    Glob,
    
    /// Regular expression
    Regex,
    
    /// Simple path contains
    Contains,
    
    /// File extension
    Extension,
    
    /// Exact path match
    Exact,
}

/// Additional rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    /// Minimum number of files that must match
    pub min_files: Option<usize>,
    
    /// Maximum number of files that must match
    pub max_files: Option<usize>,
    
    /// File size thresholds
    pub file_size: Option<FileSizeCondition>,
    
    /// Custom script to run for validation
    pub custom_script: Option<String>,
}

/// File size conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeCondition {
    /// Minimum total size of changed files
    pub min_total_size: Option<u64>,
    
    /// Maximum total size of changed files
    pub max_total_size: Option<u64>,
    
    /// Minimum size of largest changed file
    pub min_largest_file: Option<u64>,
}

/// Project-specific rule overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRuleOverrides {
    /// Override specific rules by name
    pub disabled_rules: Vec<String>,
    
    /// Additional rules specific to this project
    pub additional_rules: Option<ChangeDetectionRules>,
    
    /// Custom significance thresholds
    pub significance_thresholds: Option<SignificanceThresholds>,
}

/// Configurable significance thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceThresholds {
    /// Patterns that always indicate breaking changes
    pub breaking_patterns: Vec<FilePattern>,
    
    /// Patterns that indicate new features
    pub feature_patterns: Vec<FilePattern>,
    
    /// Patterns that indicate patches only
    pub patch_patterns: Vec<FilePattern>,
}

impl Default for ChangeDetectionRules {
    fn default() -> Self {
        Self {
            change_type_rules: default_change_type_rules(),
            significance_rules: default_significance_rules(),
            version_bump_rules: default_version_bump_rules(),
            project_overrides: HashMap::new(),
        }
    }
}

fn default_change_type_rules() -> Vec<ChangeTypeRule> {
    vec![
        ChangeTypeRule {
            name: "dependency_changes".to_string(),
            priority: 100,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "package-lock.json".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Exact,
                    pattern: "yarn.lock".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::Dependencies,
            conditions: None,
        },
        ChangeTypeRule {
            name: "source_code_changes".to_string(),
            priority: 80,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/**/*.{ts,js,tsx,jsx}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "lib/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            change_type: PackageChangeType::SourceCode,
            conditions: None,
        },
        // More rules...
    ]
}

fn default_significance_rules() -> Vec<SignificanceRule> {
    vec![
        SignificanceRule {
            name: "public_api_changes".to_string(),
            priority: 100,
            patterns: vec![
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/index.{ts,js}".to_string(),
                    exclude: false,
                },
                FilePattern {
                    pattern_type: PatternType::Glob,
                    pattern: "src/api/**/*.{ts,js}".to_string(),
                    exclude: false,
                },
            ],
            git_status: None,
            significance: ChangeSignificance::Breaking,
            conditions: None,
        },
        // More rules...
    ]
}

fn default_version_bump_rules() -> Vec<VersionBumpRule> {
    vec![
        VersionBumpRule {
            name: "breaking_changes".to_string(),
            change_type: None,
            significance: Some(ChangeSignificance::Breaking),
            version_bump: VersionBumpType::Major,
            priority: 100,
        },
        // More rules...
    ]
}