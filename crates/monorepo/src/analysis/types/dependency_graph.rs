//! Dependency graph analysis types

use std::collections::HashMap;

/// Analysis of the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyGraphAnalysis {
    /// Total number of nodes
    pub node_count: usize,
    
    /// Total number of edges
    pub edge_count: usize,
    
    /// Whether the graph has cycles
    pub has_cycles: bool,
    
    /// Detected circular dependencies
    pub cycles: Vec<Vec<String>>,
    
    /// Packages with version conflicts
    pub version_conflicts: HashMap<String, Vec<String>>,
    
    /// Packages that can be upgraded
    pub upgradable: HashMap<String, Vec<(String, String)>>,
    
    /// Maximum depth of the dependency tree
    pub max_depth: usize,
    
    /// Packages with the most dependencies
    pub most_dependencies: Vec<(String, usize)>,
    
    /// Packages with the most dependents
    pub most_dependents: Vec<(String, usize)>,
}