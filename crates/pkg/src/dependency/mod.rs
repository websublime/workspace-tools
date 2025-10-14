//! Dependency management module for sublime_pkg_tools.
//!
//! This module handles dependency analysis, graph construction, and
//! propagation of version updates across package dependencies. It provides
//! tools for detecting circular dependencies and calculating dependency
//! update impacts.
//!
//! # What
//!
//! Provides dependency management functionality:
//! - `DependencyGraph`: Graph representation of package dependencies
//! - `DependencyNode`: Individual package node in the graph
//! - `DependencyAnalyzer`: Service for dependency analysis
//! - `PropagatedUpdate`: Information about propagated dependency updates
//!
//! # How
//!
//! Uses `petgraph` for efficient graph operations and cycle detection.
//! Analyzes package.json files to build dependency relationships and
//! calculates transitive impacts of version changes.
//!
//! # Why
//!
//! Ensures that version updates are properly propagated to all dependent
//! packages while detecting and preventing circular dependency issues
//! that could cause build failures.
mod analyzer;
mod graph;
mod propagator;

#[cfg(test)]
mod tests;

pub use analyzer::{DependencyAnalyzer, DependencyGraphBuilder};
pub use graph::{DependencyEdge, DependencyGraph, DependencyNode, DependencyType};
pub use propagator::{PropagatedUpdate, PropagationReason};
