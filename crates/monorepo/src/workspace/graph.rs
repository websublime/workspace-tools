//! Workspace dependency graph abstraction.
//!
//! This module provides a representation of the package dependency graph,
//! including information about cycles, external dependencies, and version conflicts.

use std::collections::HashMap;
use sublime_package_tools::ValidationReport;

/// Abstraction of a workspace dependency graph.
///
/// Contains information about the package dependency structure,
/// including cycles, conflicts, and external dependencies.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::WorkspaceGraph;
///
/// # fn example(graph: WorkspaceGraph) {
/// // Check for cycles
/// if graph.cycles_detected {
///     println!("Cycles detected in dependency graph:");
///     for cycle in &graph.cycles {
///         println!("  {}", cycle.join(" → "));
///     }
/// }
///
/// // Check for external dependencies
/// if !graph.external_dependencies.is_empty() {
///     println!("External dependencies: {}", graph.external_dependencies.join(", "));
/// }
///
/// // Check for version conflicts
/// for (package, conflicts) in &graph.version_conflicts {
///     println!("Package {} has version conflicts: {}", package, conflicts.join(", "));
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct WorkspaceGraph {
    /// Whether cycles were detected in the graph
    pub cycles_detected: bool,
    /// The actual cycle groups identified in the graph
    pub cycles: Vec<Vec<String>>,
    /// External dependencies
    pub external_dependencies: Vec<String>,
    /// Version conflicts
    pub version_conflicts: HashMap<String, Vec<String>>,
    /// Validation report
    pub validation: Option<ValidationReport>,
}
