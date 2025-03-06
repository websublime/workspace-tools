//! Package management utilities for workspaces.
//!
//! This module provides tools for managing package dependencies, resolving version conflicts,
//! detecting dependency cycles, and upgrading dependencies.

pub mod bump;
pub mod error;
pub mod graph;
pub mod registry;
pub mod types;
pub mod upgrader;

// Re-export commonly used items for convenience
pub use error::{PkgError, Result};
pub use graph::{DependencyGraph, Node};
pub use registry::{DependencyRegistry, ResolutionResult};
pub use types::{
    dependency::Dependency,
    diff::{ChangeType, DependencyChange, PackageDiff},
    package::Package,
    version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy},
};
pub use upgrader::{
    AvailableUpgrade, DependencyUpgrader, ExecutionMode, PackageRegistry, UpgradeConfig,
    UpgradeStatus,
};
