//! Core monorepo project types and implementations
//!
//! This module provides the central `MonorepoProject` type that aggregates
//! functionality from all base crates, as well as the enhanced `MonorepoPackageInfo`
//! type that provides monorepo-specific package information.

pub mod components;
mod package;
mod project;
pub mod services;
#[cfg(test)]
mod tests;
mod tools;
pub mod types;
mod version;

pub use types::{
    AggressiveVersioningStrategy,
    BreakingChangeAnalysis,
    // Core package types
    Changeset,
    ChangesetStatus,
    ConflictType,
    ConservativeVersioningStrategy,
    DefaultVersioningStrategy,
    DependencyChainImpact,
    MonorepoPackageInfo,
    // Core implementation types
    MonorepoProject,
    MonorepoTools,
    PackageImpactAnalysis,
    PackageVersionUpdate,
    PropagationResult,
    VersionConflict,
    VersionImpactAnalysis,
    VersionManager,
    VersionStatus,
    VersioningPlan,
    VersioningPlanStep,
    // Version management types
    VersioningResult,
    // Versioning strategy implementations
    VersioningStrategy,
};

// Interfaces removed - using direct access patterns instead
