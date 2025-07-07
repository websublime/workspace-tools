//! Change detection and analysis module
//!
//! This module provides comprehensive change detection capabilities including
//! configurable rules, detection engines, and change detectors for monorepos.

pub mod conventional;
mod detector;
mod engine;
mod rules;
#[cfg(test)]
mod tests;
pub mod types;

// Explicit re-exports from types module
pub use types::{
    ChangeDetectionEngine,
    // Rule definitions
    ChangeDetectionRules,
    // Implementation structs
    ChangeDetector,
    ChangeSignificance,
    ChangeTypeRule,
    // Pattern matching
    FilePattern,
    FileSizeCondition,
    PackageChange,
    // Core types
    PackageChangeType,
    PatternType,
    // Project overrides
    ProjectRuleOverrides,
    RuleConditions,
    SignificanceRule,
    SignificanceThresholds,
    VersionBumpRule,
    VersionBumpType,
};

// Re-exports from conventional commits module
pub use conventional::{ChangeDecisionSource, ConventionalCommitParser};
