//! Version management module for sublime_pkg_tools.
//!
//! This module provides comprehensive version handling for package management,
//! including semantic versioning (SemVer), snapshot versions for development
//! branches, and version resolution strategies.
//!
//! # What
//!
//! Defines core version types and operations:
//! - `Version`: Standard semantic version representation
//! - `SnapshotVersion`: Development snapshot versions with commit identifiers
//! - `VersionBump`: Version increment types (major, minor, patch)
//! - `ResolvedVersion`: Union type for release and snapshot versions
//! - `VersionResolver`: Service for resolving current package versions
//!
//! # How
//!
//! Integrates with the `semver` crate for standard semantic versioning while
//! extending it with snapshot versions for development workflows. Snapshot
//! versions are calculated dynamically and never persisted to package.json.
//!
//! # Why
//!
//! Enables continuous deployment to development environments while maintaining
//! clean release versioning. Each commit on feature branches gets a unique
//! version identifier without polluting the Git history.
mod bump;
mod resolver;
mod snapshot;
mod versioning;

#[cfg(test)]
mod tests;

pub use bump::VersionBump;
pub use resolver::ResolvedVersion;
pub use snapshot::SnapshotVersion;
pub use versioning::{Version, VersionComparison};
