//! Workspace analysis structures for dependency and version information.
//!
//! This module provides data structures for storing the results of workspace analysis,
//! including information about dependency cycles, external dependencies, and version
//! conflicts.

use std::collections::HashMap;

/// Results from analyzing a workspace.
///
/// Contains information about the workspace's dependency structure,
/// including cycles, external dependencies, and version conflicts.
///
/// # Examples
///
/// ```no_run
/// use sublime_monorepo_tools::WorkspaceAnalysis;
///
/// # fn example(analysis: WorkspaceAnalysis) {
/// // Check for cycles
/// if !analysis.cycles.is_empty() {
///     println!("Detected {} cycle groups", analysis.cycles.len());
///     for cycle in &analysis.cycles {
///         println!("Cycle: {}", cycle.join(" → "));
///     }
/// }
///
/// // Check for external dependencies
/// if !analysis.external_dependencies.is_empty() {
///     println!("External dependencies: {}", analysis.external_dependencies.join(", "));
/// }
///
/// // Check for version conflicts
/// for (package, conflicts) in &analysis.version_conflicts {
///     println!("Package {} has version conflicts: {}", package, conflicts.join(", "));
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct WorkspaceAnalysis {
    /// Cycle groups detected in the dependency graph
    pub cycles: Vec<Vec<String>>,
    /// External dependencies
    pub external_dependencies: Vec<String>,
    /// Version conflicts in the workspace
    pub version_conflicts: HashMap<String, Vec<String>>,
    /// Whether there are validation issues
    pub validation_issues: bool,
}
