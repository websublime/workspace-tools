//! Analysis type definitions module
//!
//! This module contains all analysis-related type definitions organized
//! in separate files for better maintainability and consistency.
//!
//! The module is organized as follows:
//! - `core`: Core analysis result types
//! - `package_manager`: Package manager analysis types
//! - `packages`: Package classification and information types
//! - `dependency_graph`: Dependency graph analysis types
//! - `registries`: Registry analysis types
//! - `workspace`: Workspace configuration analysis types
//! - `upgrades`: Package upgrade analysis types
//! - `diff`: Diff analysis and change detection types

mod analyzer;
mod core;
mod package_manager;
mod packages;
mod dependency_graph;
mod registries;
mod workspace;
mod upgrades;
pub mod diff;

pub use analyzer::MonorepoAnalyzer;
pub use core::*;
pub use package_manager::*;
pub use packages::*;
pub use dependency_graph::*;
pub use registries::*;
pub use workspace::*;
pub use upgrades::*;
pub use diff::*;