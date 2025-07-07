//! Core monorepo project types and implementations
//!
//! This module provides the central `MonorepoProject` type that uses base crates directly
//! for optimal CLI/daemon performance. The module focuses on direct ownership patterns
//! and eliminates service abstractions for sub-second analysis performance.
//!
//! # Main Types
//!
//! - [`MonorepoProject`] - Main entry point for monorepo operations
//! - [`MonorepoPackageInfo`] - Enhanced package information with dependency analysis
//! - [`VersionManager`] - Version management with dependency propagation
//! - [`VersioningResult`] - Results from version operations
//!
//! # Examples
//!
//! ```rust
//! use sublime_monorepo_tools::core::MonorepoProject;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize project with direct base crate integration
//! let project = MonorepoProject::new(".")?;
//!
//! // Access project information
//! println!("Root: {}", project.root_path().display());
//! println!("Packages: {}", project.packages.len());
//!
//! // Get package information
//! if let Some(pkg) = project.get_package("my-package") {
//!     println!("Package path: {}", pkg.path().display());
//! }
//! # Ok(())
//! # }
//! ```

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
