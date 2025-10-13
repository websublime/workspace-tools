//! Release management module for sublime_pkg_tools.
//!
//! This module handles release planning, execution, and coordination across
//! multiple packages and environments. It orchestrates the entire release
//! process from changeset application to package publishing and tagging.
//!
//! # What
//!
//! Provides release management functionality:
//! - `ReleaseManager`: Orchestrates the complete release process
//! - `ReleasePlan`: Plan for executing a release
//! - `PackageRelease`: Individual package release information
//! - `ReleaseStrategy`: Strategy for coordinating package releases
//!
//! # How
//!
//! Coordinates with changeset, registry, and git modules to execute releases.
//! Supports both independent and unified versioning strategies with dry-run
//! capabilities for validation before execution.
//!
//! # Why
//!
//! Provides safe, coordinated release execution with proper rollback
//! capabilities and comprehensive validation to prevent failed releases
//! and ensure consistent package publishing across environments.
mod manager;
mod plan;

#[cfg(test)]
mod tests;

pub use manager::ReleaseManager;
pub use plan::{DryRunResult, PackageRelease, ReleasePlan, ReleaseResult, ReleaseStrategy};
