//! Audit command type definitions.
//!
//! # What
//!
//! This module contains type definitions for the audit command, including
//! parameter structures and response data types. The audit command provides
//! comprehensive health checks and analysis of the workspace.
//!
//! # How
//!
//! Types are defined with `#[napi(object)]` attribute to be exposed as
//! JavaScript objects. The module provides:
//!
//! - `AuditParams`: Input parameters for the audit command
//! - `AuditData`: Response data containing audit results and health score
//!
//! # Why
//!
//! The audit command helps maintain workspace health by identifying issues
//! such as outdated dependencies, version inconsistencies, circular dependencies,
//! and potential breaking changes.
//!
//! # Examples
//!
//! ```typescript
//! import { audit, AuditParams, AuditData } from '@websublime/workspace-tools';
//!
//! const params: AuditParams = {
//!   root: '.',
//!   sections: ['upgrades', 'dependencies', 'versions'],
//!   minSeverity: 'medium',
//!   verbosity: 'normal'
//! };
//! const result = await audit(params);
//!
//! if (result.success) {
//!   const data: AuditData = result.data;
//!   console.log(`Health Score: ${data.summary.healthScore}%`);
//!   console.log(`Total Issues: ${data.summary.totalIssues}`);
//!   console.log(`Critical: ${data.summary.criticalIssues}`);
//!   console.log(`High: ${data.summary.highIssues}`);
//!
//!   // Check specific sections
//!   if (data.sections.upgrades) {
//!     console.log(`Available upgrades: ${data.sections.upgrades.totalUpgrades}`);
//!   }
//! }
//! ```

// TODO: will be implemented on story 9.1 - Audit Types
// This module will contain:
//
// Re-exports from sublime_pkg_tools:
// - pub use sublime_pkg_tools::audit::{
//     AuditReport, AuditSummary, AuditSections, AuditIssue,
//     IssueCategory, IssueSeverity, HealthScoreBreakdown,
//     UpgradeAuditSection, DependencyAuditSection,
//     VersionConsistencyAuditSection, BreakingChangesAuditSection,
//     DeprecatedPackage, VersionInconsistency, VersionConflict,
//     PackageBreakingChanges, BreakingChange, DependencyCategorization
// };
//
// NAPI-specific types:
// - AuditParams: { root, sections?, minSeverity?, verbosity?, outputFile? }
// - AuditData: { summary: AuditSummaryInfo, sections: AuditSectionsInfo, report?: string }
//
// Shared types:
// - AuditSummaryInfo: {
//     totalPackages, totalIssues, criticalIssues, highIssues,
//     mediumIssues, lowIssues, infoIssues, healthScore
//   }
// - AuditSectionsInfo: {
//     upgrades?: UpgradeAuditInfo,
//     dependencies?: DependencyAuditInfo,
//     versionConsistency?: VersionConsistencyAuditInfo,
//     breakingChanges?: BreakingChangesAuditInfo
//   }
// - AuditIssueInfo: { category, severity, title, description, affectedPackages }
