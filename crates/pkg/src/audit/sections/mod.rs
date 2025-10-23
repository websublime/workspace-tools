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
//! Future sections to be implemented:
//! - **Version Consistency** (Story 10.6): Checks version consistency across packages

#![allow(clippy::todo)]

// Upgrades section (Story 10.2 - IMPLEMENTED)
pub(crate) mod upgrades;

// Dependencies section (Story 10.3 - IMPLEMENTED)
pub(crate) mod dependencies;

// Categorization section (Story 10.4 - IMPLEMENTED)
pub(crate) mod categorization;

// Breaking changes section (Story 10.5 - IMPLEMENTED)
pub(crate) mod breaking_changes;

// Public exports
pub use breaking_changes::{
    audit_breaking_changes, BreakingChange, BreakingChangeSource, BreakingChangesAuditSection,
    PackageBreakingChanges,
};
pub use categorization::{
    categorize_dependencies, generate_categorization_issues, CategorizationStats,
    DependencyCategorization, ExternalPackage, InternalPackage, LocalLink, LocalLinkType,
    WorkspaceLink,
};
pub use dependencies::{audit_dependencies, DependencyAuditSection, VersionConflict, VersionUsage};
pub use upgrades::{audit_upgrades, DeprecatedPackage, UpgradeAuditSection};

// Future sections will be added here:
// - Story 10.6: Version consistency section
