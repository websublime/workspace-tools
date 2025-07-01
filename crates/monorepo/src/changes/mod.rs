//! Change detection and analysis module
//!
//! This module provides comprehensive change detection capabilities including
//! configurable rules, detection engines, and change detectors for monorepos.

pub mod types;
pub mod conventional;
mod detector;
mod rules;
mod engine;


// Explicit re-exports from types module
pub use types::{
    // Core types
    PackageChangeType, ChangeSignificance, VersionBumpType, PackageChange,
    // Rule definitions
    ChangeDetectionRules, ChangeTypeRule, SignificanceRule, VersionBumpRule,
    // Pattern matching
    FilePattern, PatternType, RuleConditions, FileSizeCondition,
    // Project overrides
    ProjectRuleOverrides, SignificanceThresholds,
    // Implementation structs
    ChangeDetector, ChangeDetectionEngine,
};

// Re-exports from conventional commits module
pub use conventional::{
    ChangeDecisionSource, ConventionalCommitParser,
};