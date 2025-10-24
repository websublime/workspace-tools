//! Audit and health check module for package dependencies and project health.
//!
//! **What**: Provides comprehensive auditing capabilities for Node.js projects, including
//! dependency analysis, upgrade detection, breaking change detection, and health scoring.
//!
//! **How**: This module analyzes the project's dependency tree, checks for available upgrades,
//! detects circular dependencies, categorizes dependencies, and generates detailed health reports.
//!
//! **Why**: To enable proactive identification of dependency issues, security vulnerabilities,
//! and maintainability concerns before they impact production systems.
//!
//! # Features
//!
//! - **Upgrade Audits**: Detect available package upgrades and their severity
//! - **Dependency Analysis**: Check for circular dependencies, missing dependencies, and version conflicts
//! - **Breaking Changes Detection**: Identify potential breaking changes in upgrades
//! - **Dependency Categorization**: Classify dependencies as internal, external, workspace, or local
//! - **Version Consistency Checks**: Detect version inconsistencies across packages
//! - **Health Scoring**: Calculate overall project health metrics
//! - **Report Generation**: Export audit results in multiple formats (Markdown, JSON)
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::audit::AuditManager;
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let config = PackageToolsConfig::default();
//!
//! // Initialize audit manager
//! let audit_manager = AuditManager::new(workspace_root, config).await?;
//!
//! // Full audit functionality will be available in subsequent stories
//! // let audit_result = audit_manager.run_audit().await?;
//! //
//! // println!("Total issues found: {}", audit_result.total_issues());
//! // for issue in audit_result.critical_issues() {
//! //     println!("Critical: {}", issue.title);
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! This module contains:
//! - `manager`: The main `AuditManager` for orchestrating audit operations
//!
//! Additional components will be added in subsequent stories:
//! - `sections`: Individual audit section implementations (upgrades, dependencies, etc.)
//! - `issue`: Issue types and severity levels
//! - `report`: Report formatting and export capabilities

#![allow(clippy::todo)]

mod issue;
mod manager;
mod sections;

#[cfg(test)]
mod tests;

// Public exports
pub use manager::AuditManager;

// Issue types
pub use issue::{AuditIssue, IssueCategory, IssueSeverity};

// Section types and functions
pub use sections::{
    audit_breaking_changes, audit_dependencies, audit_upgrades, audit_version_consistency,
    categorize_dependencies, generate_categorization_issues, BreakingChange, BreakingChangeSource,
    BreakingChangesAuditSection, CategorizationStats, DependencyAuditSection,
    DependencyCategorization, DeprecatedPackage, ExternalPackage, InternalPackage, LocalLink,
    LocalLinkType, PackageBreakingChanges, UpgradeAuditSection, VersionConflict,
    VersionConsistencyAuditSection, VersionInconsistency, VersionUsage, WorkspaceLink,
};
