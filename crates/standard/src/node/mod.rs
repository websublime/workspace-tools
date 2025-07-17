//! # Node.js Project Module
//!
//! ## What
//! This module provides generic Node.js repository concepts and abstractions
//! that are used across different project types (simple repositories and monorepos).
//! It defines fundamental types for repository identification, package management,
//! and cross-platform Node.js utilities.
//!
//! ## How
//! The module is organized into several sub-modules:
//! - `types`: Core repository types and enums (`RepoKind`, etc.)
//! - `package_manager`: Package manager abstractions and detection
//! - `repository`: Repository trait definitions and implementations
//! 
//! Each sub-module focuses on a specific aspect of Node.js project management
//! while maintaining clean separation of concerns and proper module boundaries.
//!
//! ## Why
//! Previously, generic Node.js concepts like `PackageManager` and `PackageManagerKind`
//! were incorrectly placed in the monorepo module, creating conceptual dependencies
//! and violating separation of concerns. This module provides the correct home
//! for these fundamental abstractions, enabling all project types to access
//! them without importing monorepo-specific logic.

mod package_manager;
mod repository;
mod types;

#[cfg(test)]
mod tests;

pub use package_manager::{PackageManager, PackageManagerKind};
pub use repository::RepositoryInfo;
pub use types::RepoKind;