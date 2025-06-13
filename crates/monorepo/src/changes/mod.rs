//! Change detection and analysis module
//!
//! This module provides comprehensive change detection capabilities including
//! configurable rules, detection engines, and change detectors for monorepos.

pub mod types;
mod detector;
mod rules;
mod engine;

#[cfg(test)]
mod tests;

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