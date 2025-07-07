//! Project-specific rule overrides and significance thresholds

use super::{ChangeDetectionRules, FilePattern};
use serde::{Deserialize, Serialize};

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
