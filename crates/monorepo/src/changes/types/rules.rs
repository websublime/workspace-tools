//! Change detection rules types

use super::{
    ChangeSignificance, FilePattern, PackageChangeType, ProjectRuleOverrides, RuleConditions,
    VersionBumpType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub git_status: Option<Vec<sublime_git_tools::GitFileStatus>>,

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
