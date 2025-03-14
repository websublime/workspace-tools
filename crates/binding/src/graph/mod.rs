//! Graph functionality for dependency management.

pub mod builder;
pub mod node;
pub mod validation;
pub mod visualization;

// Re-export main types for convenience
pub use builder::{
    build_dependency_graph_from_package_infos, build_dependency_graph_from_packages,
    DependencyFilter, DependencyGraph,
};
pub use node::Node;
pub use validation::{ValidationIssueInfo, ValidationIssueType, ValidationReport};
pub use visualization::{generate_ascii, generate_dot, save_dot_to_file, DotOptions};
