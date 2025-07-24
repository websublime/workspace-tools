//! # Generic Graph Utilities Module
//!
//! This module provides generic utilities for working with dependency graphs.
//!
//! ## Purpose and Separation of Responsibilities
//!
//! This module contains **generic utilities** that can be used with any graph structure,
//! while `dependency/graph.rs` contains the **core graph implementation** specifically
//! for package dependency graphs.
//!
//! ### This module (`graph/`) provides:
//! - **Builder utilities** (`builder.rs`) - Helper functions for constructing graphs
//! - **Validation utilities** (`validation.rs`) - Generic validation logic and reporting
//! - **Visualization utilities** (`visualization.rs`) - DOT format generation and ASCII art
//! - **Node utilities** (`node.rs`) - Generic node-related functionality
//! - **Hash Tree utilities** (`hash_tree.rs`) - Structured queryable dependency model
//!
//! ### The main graph module (`dependency/graph.rs`) provides:
//! - **Core Graph struct** - The main dependency graph data structure
//! - **Graph-specific methods** - Methods directly tied to the Graph implementation
//! - **Dependency-specific logic** - Business logic for package dependencies
//!
//! ## Design Rationale
//!
//! This separation allows for:
//! - **Modularity**: Generic utilities can be reused across different graph types
//! - **Clarity**: Core graph implementation is separated from utility functions
//! - **Maintainability**: Changes to utilities don't affect the core graph structure
//! - **Testability**: Utilities can be tested independently
//!
//! ## Usage
//!
//! These utilities are typically used in conjunction with the main `Graph` struct:
//!
//! ```
//! use sublime_package_tools::{Graph, Package, ValidationOptions};
//! use sublime_package_tools::graph::{build_dependency_graph_from_packages, DotOptions};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let packages = vec![];
//! // Use builder utilities to create graph
//! let graph = build_dependency_graph_from_packages(&packages);
//!
//! // Use validation utilities to check the graph
//! let options = ValidationOptions::new().treat_unresolved_as_external(true);
//! let report = graph.validate_with_options(&options)?;
//!
//! // Use visualization utilities to generate DOT output
//! let dot_options = DotOptions::default();
//! let dot_output = graph.to_dot_with_options(&dot_options)?;
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod hash_tree;
pub mod node;
#[cfg(test)]
pub mod tests;
pub mod validation;
pub mod visualization;
