//! Core monorepo project types and implementations
//! 
//! This module provides the central `MonorepoProject` type that aggregates
//! functionality from all base crates, as well as the enhanced `MonorepoPackageInfo`
//! type that provides monorepo-specific package information.

pub mod types;
mod project;
mod package;
mod version;
mod tools;

#[cfg(test)]
mod tests;

pub use types::{
    // Core package types
    Changeset,
    ChangesetStatus,
    MonorepoPackageInfo,
    VersionStatus,
    // Version management types
    VersioningResult,
    PackageVersionUpdate,
    PropagationResult,
    VersionConflict,
    ConflictType,
    VersionImpactAnalysis,
    PackageImpactAnalysis,
    BreakingChangeAnalysis,
    DependencyChainImpact,
    VersioningPlan,
    VersioningPlanStep,
    // Versioning strategy implementations
    VersioningStrategy,
    DefaultVersioningStrategy,
    ConservativeVersioningStrategy,
    AggressiveVersioningStrategy,
    // Core implementation types
    MonorepoProject,
    VersionManager,
    MonorepoTools,
};