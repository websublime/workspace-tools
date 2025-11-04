//! Audit command implementations.
//!
//! This module provides functionality for auditing project health across multiple dimensions.
//!
//! # What
//!
//! Provides implementations for:
//! - Comprehensive audit - Full project health check across all dimensions
//! - Upgrade audit - Analysis of available dependency upgrades
//! - Dependency audit - Health check for dependency graph and issues
//! - Version consistency audit - Version alignment across monorepo packages
//! - Breaking changes audit - Detection of potential breaking changes
//!
//! # How
//!
//! The audit commands:
//! 1. Use `sublime-package-tools` audit module for core audit logic
//! 2. Load project configuration and workspace information
//! 3. Execute selected audit sections (or all by default)
//! 4. Calculate health scores with configurable weights
//! 5. Generate comprehensive reports in multiple formats
//! 6. Provide actionable recommendations for improvement
//! 7. Support filtering by severity level and verbosity control
//!
//! # Why
//!
//! Regular health audits are essential for:
//! - Maintaining code quality and consistency
//! - Identifying security and stability risks
//! - Preventing technical debt accumulation
//! - Ensuring dependency health and upgrade readiness
//! - Detecting breaking changes before they cause issues
//!
//! This module provides comprehensive visibility into project health with
//! actionable insights for maintaining a healthy codebase.

pub mod comprehensive;
pub mod dependencies;
pub mod report;
pub mod types;
pub mod upgrades;

#[cfg(test)]
mod tests;

// Re-export command implementations
pub use comprehensive::execute_audit;
pub use dependencies::execute_dependency_audit;
pub use upgrades::execute_upgrade_audit;
