//! Change detection types organized by functionality

// Core change detection types
pub mod core;
pub use core::{PackageChangeType, ChangeSignificance, VersionBumpType, PackageChange};

// Rule definitions
pub mod rules;
pub use rules::{ChangeDetectionRules, ChangeTypeRule, SignificanceRule, VersionBumpRule};

// Pattern matching types
pub mod patterns;
pub use patterns::{FilePattern, PatternType, RuleConditions, FileSizeCondition};

// Project overrides and thresholds
pub mod overrides;
pub use overrides::{ProjectRuleOverrides, SignificanceThresholds};