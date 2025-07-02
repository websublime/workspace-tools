//! Core type definitions module
//!
//! This module contains all core-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `changeset`: Changeset-related types and status definitions
//! - `package`: Package information and version status types
//! - `versioning`: Version management types and result structures
//! - `impact`: Version impact analysis types (submodule structure)
//! - `versioning`: Versioning plan and step types (submodule structure)
//! - `strategies`: Versioning strategy implementations
//! - `project`: Monorepo project implementation struct
//! - `version_manager`: Version manager implementation struct
//! - `tools`: Monorepo tools implementation struct

mod changeset;
pub mod impact;
mod package;
mod strategies;
pub mod versioning;
mod versioning_old;

// Implementation structs (moved from main modules)
mod project;
mod tools;
mod version_manager;

// Explicit exports to avoid wildcard re-exports

// Changeset types
pub use changeset::{Changeset, ChangesetStatus};

// Package types
pub use package::{
    DependencyType, MonorepoPackageInfo, PackageDependency, PackageType, VersionStatus,
};

// Versioning types
pub use versioning_old::{
    ConflictType, PackageVersionUpdate, PropagationResult, VersionConflict, VersioningResult,
};

// Impact analysis types
pub use impact::{
    BreakingChangeAnalysis, DependencyChainImpact, PackageImpactAnalysis, VersionImpactAnalysis,
};

// Versioning plan types
pub use versioning::{VersioningPlan, VersioningPlanStep};

// Strategy types
pub use strategies::{
    AggressiveVersioningStrategy, ConservativeVersioningStrategy, DefaultVersioningStrategy,
    VersioningStrategy,
};

// Implementation structs
pub use project::MonorepoProject;
pub use tools::MonorepoTools;
pub use version_manager::VersionManager;
