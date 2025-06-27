//! Focused components for monorepo package operations
//!
//! This module provides focused components that encapsulate specific
//! responsibilities, replacing the monolithic MonorepoPackageInfo approach.
//! Each component has a single responsibility and clear ownership boundaries.

/// Package information reader component
pub mod package_info;

/// Package version management component
pub mod version_manager;

/// Package changeset management component
pub mod changeset_manager;

/// Package dependency management component
pub mod dependency_manager;

/// Package persistence component
pub mod persistence;

// Re-export main components
pub use package_info::{PackageInfoReader, PackageStats};
pub use version_manager::PackageVersionManager;
pub use changeset_manager::{PackageChangesetManager, ChangesetDeploymentSummary};
pub use dependency_manager::PackageDependencyManager;
pub use persistence::PackagePersistence;
