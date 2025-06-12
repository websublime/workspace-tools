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
mod package;
mod versioning_old;
pub mod impact;
pub mod versioning;
mod strategies;

// Implementation structs (moved from main modules)
mod project;
mod version_manager;
mod tools;

// Explicit exports to avoid wildcard re-exports

// Changeset types
pub use changeset::{Changeset, ChangesetStatus};

// Package types
pub use package::{MonorepoPackageInfo, VersionStatus};

// Versioning types
pub use versioning_old::{
    VersioningResult, PackageVersionUpdate, PropagationResult, 
    VersionConflict, ConflictType
};

// Impact analysis types
pub use impact::{
    VersionImpactAnalysis, PackageImpactAnalysis, BreakingChangeAnalysis, DependencyChainImpact
};

// Versioning plan types
pub use versioning::{VersioningPlan, VersioningPlanStep};

// Strategy types
pub use strategies::{
    VersioningStrategy,
    DefaultVersioningStrategy,
    ConservativeVersioningStrategy,
    AggressiveVersioningStrategy,
};

// Implementation structs
pub use project::MonorepoProject;
pub use version_manager::VersionManager;
pub use tools::MonorepoTools;