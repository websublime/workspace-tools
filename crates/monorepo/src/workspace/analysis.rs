use std::collections::HashMap;

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
