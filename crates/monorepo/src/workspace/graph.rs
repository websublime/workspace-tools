//! Workspace dependency graph abstraction.

use std::collections::HashMap;
use sublime_package_tools::ValidationReport;

/// Abstraction of a workspace dependency graph.
#[derive(Debug)]
pub struct WorkspaceGraph {
    /// Whether cycles were detected in the graph
    pub cycles_detected: bool,
    /// Missing dependencies
    pub missing_dependencies: Vec<String>,
    /// Version conflicts
    pub version_conflicts: HashMap<String, Vec<String>>,
    /// Validation report
    pub validation: Option<ValidationReport>,
}
