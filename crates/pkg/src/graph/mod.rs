//! Dependency graph functionality.

mod builder;
mod node;
mod validation;

pub use builder::build_dependency_graph_from_package_infos;
pub use builder::build_dependency_graph_from_packages;
pub use node::Node;
pub use validation::{ValidationIssue, ValidationReport};

// Re-export DependencyGraph
pub use self::builder::{DependencyFilter, DependencyGraph};
