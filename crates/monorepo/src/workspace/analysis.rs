use std::collections::HashMap;

#[derive(Debug)]
pub struct WorkspaceAnalysis {
    /// Whether a cycle was detected in the dependency graph
    pub cycle_detected: bool,
    /// Missing dependencies in the workspace
    pub missing_dependencies: Vec<String>,
    /// Version conflicts in the workspace
    pub version_conflicts: HashMap<String, Vec<String>>,
    /// Whether there are validation issues
    pub validation_issues: bool,
}
