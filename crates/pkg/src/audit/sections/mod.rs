//! Audit section implementations for different aspects of project health.
//!
//! **What**: Contains individual audit section modules that analyze specific
//! aspects of a project, such as upgrades, dependencies, breaking changes, etc.
//!
//! **How**: Each section module provides specialized analysis functionality
//! that can be called independently or as part of a complete audit. Sections
//! produce standardized results with audit issues.
//!
//! **Why**: To organize audit functionality into focused, testable modules
//! that can be developed and maintained independently while providing a
//! consistent interface for the audit manager.
//!
//! # Available Sections
//!
//! - **Upgrades** (Story 10.2): Detects available package upgrades and deprecated packages
//! - **Dependencies** (Story 10.3): Analyzes dependency graph for issues
//! - **Categorization** (Story 10.4): Categorizes dependencies into internal, external, workspace, and local
//! - **Breaking Changes** (Story 10.5): Detects potential breaking changes in commits and changesets
//!
#![allow(clippy::todo)]

// Upgrades section (Story 10.2 - IMPLEMENTED)
pub(crate) mod upgrades;

// Dependencies section (Story 10.3 - IMPLEMENTED)
pub(crate) mod dependencies;

// Categorization section (Story 10.4 - IMPLEMENTED)
pub(crate) mod categorization;

// Breaking changes section (Story 10.5 - IMPLEMENTED)
pub(crate) mod breaking_changes;

// Version consistency section (Story 10.6 - IMPLEMENTED)
pub(crate) mod version_consistency;

// Public exports
pub use breaking_changes::{
    BreakingChange, BreakingChangeSource, BreakingChangesAuditSection, PackageBreakingChanges,
    audit_breaking_changes,
};
pub use categorization::{
    CategorizationStats, DependencyCategorization, ExternalPackage, InternalPackage, LocalLink,
    LocalLinkType, WorkspaceLink, categorize_dependencies, generate_categorization_issues,
};
pub use dependencies::{DependencyAuditSection, VersionConflict, VersionUsage, audit_dependencies};
pub use upgrades::{DeprecatedPackage, UpgradeAuditSection, audit_upgrades};
pub use version_consistency::{
    VersionConsistencyAuditSection, VersionInconsistency, audit_version_consistency,
};
