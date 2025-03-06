//! Graph validation utilities.

use crate::error::{PkgError, Result};
use crate::graph::builder::DependencyGraph;
use crate::graph::node::Node;
use crate::types::dependency::Dependency;
use petgraph::visit::Dfs;
use std::collections::HashMap;

/// Types of validation issues that can occur
#[derive(Debug)]
pub enum ValidationIssue {
    /// Circular dependency detected
    CircularDependency { path: Vec<String> },

    /// Unresolved dependency
    UnresolvedDependency { name: String, version_req: String },

    /// Version conflict
    VersionConflict { name: String, versions: Vec<String> },
}

impl ValidationIssue {
    /// Returns true if this is a critical issue that should be fixed
    pub fn is_critical(&self) -> bool {
        match self {
            Self::UnresolvedDependency { .. } | Self::CircularDependency { .. } => true,
            Self::VersionConflict { .. } => false, // Consider version conflicts as warnings
        }
    }

    /// Returns a descriptive message for this issue
    pub fn message(&self) -> String {
        match self {
            Self::CircularDependency { path } => {
                format!("Circular dependency detected: {}", path.join(" -> "))
            }
            Self::UnresolvedDependency { name, version_req } => {
                format!("Unresolved dependency: {name} {version_req}")
            }
            Self::VersionConflict { name, versions } => {
                format!("Version conflict for {}: {}", name, versions.join(", "))
            }
        }
    }
}

/// Report containing validation issues
#[derive(Debug, Default)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(ValidationIssue::is_critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|issue| !issue.is_critical())
    }

    pub fn critical_issues(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_critical()).collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter().filter(|issue| !issue.is_critical()).collect()
    }
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    /// Check for circular dependencies in the graph
    pub fn detect_circular_dependencies(&self) -> Result<()> {
        use petgraph::algo::is_cyclic_directed;

        if is_cyclic_directed(&self.graph) {
            // Find one of the cycles for more detailed error reporting
            let cycle = self.find_cycle();

            // Convert identifiers to strings for error reporting
            let cycle_strings: Vec<String> = cycle.into_iter().map(|id| id.to_string()).collect();

            return Err(PkgError::CircularDependency { path: cycle_strings });
        }

        Ok(())
    }

    /// Find a cycle in the graph, if one exists
    fn find_cycle(&self) -> Vec<N::Identifier> {
        let mut cycle = Vec::new();

        // Use Petgraph's cycle detection algorithm
        // A simpler approach that works for our needs
        for node_idx in self.graph.node_indices() {
            let node_weight = self.graph.node_weight(node_idx).expect("Node should exist");
            if let crate::graph::node::Step::Resolved(node) = node_weight {
                // Check if this node is part of a cycle
                let mut dfs = Dfs::new(&self.graph, node_idx);

                // Start a DFS from this node
                while let Some(next_idx) = dfs.next(&self.graph) {
                    // If we can follow edges and get back to our source, we have a cycle
                    for neighbor in self.graph.neighbors(next_idx) {
                        if neighbor == node_idx && next_idx != node_idx {
                            // Found a cycle
                            cycle.push(node.identifier());

                            // Add the immediate successor to the cycle as well
                            if let Some(crate::graph::node::Step::Resolved(next_node)) =
                                self.graph.node_weight(next_idx)
                            {
                                cycle.push(next_node.identifier());
                            }

                            // Return early - we found a cycle
                            return cycle;
                        }
                    }
                }
            }
        }

        cycle
    }

    /// Find all missing dependencies in the workspace
    pub fn find_missing_dependencies(&self) -> Vec<String>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut missing = Vec::new();

        // Get all package names in the graph
        let resolved_node_names: std::collections::HashSet<String> =
            self.resolved_dependencies().map(|node| node.identifier().to_string()).collect();

        // Check each unresolved dependency
        for dep in self.unresolved_dependencies() {
            let name = dep.name().to_string();
            if !resolved_node_names.contains(&name) {
                missing.push(name);
            }
        }

        missing
    }

    /// Find all version conflicts in the graph
    ///
    /// This implementation requires that N::DependencyType implements a way to get
    /// name and version strings. For the Package implementation, we know this is Dependency,
    /// but we can't access those methods directly with the generic type.
    pub fn find_version_conflicts_for_package(&self) -> HashMap<String, Vec<String>>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut conflicts = HashMap::new();

        // Group all dependencies by name
        let mut requirements_by_name: HashMap<String, Vec<String>> = HashMap::new();

        // Collect dependencies from all nodes in the graph
        for node_idx in self.graph.node_indices() {
            if let Some(crate::graph::node::Step::Resolved(node)) = self.graph.node_weight(node_idx)
            {
                for dep in node.dependencies_vec() {
                    requirements_by_name
                        .entry(dep.name().to_string())
                        .or_default()
                        .push(dep.version_str());
                }
            }
        }

        // Find conflicting requirements
        for (name, reqs) in requirements_by_name {
            // Count unique version requirements
            let unique_reqs: std::collections::HashSet<_> = reqs.iter().collect();

            // If there's more than one unique requirement, it's a conflict
            if unique_reqs.len() > 1 {
                conflicts.insert(name, reqs);
            }
        }

        conflicts
    }

    /// Find all version conflicts in the dependency graph
    pub fn find_version_conflicts(&self) -> Option<HashMap<String, Vec<String>>>
    where
        N: Node<DependencyType = Dependency>,
    {
        let conflicts = self.find_version_conflicts_for_package();
        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    /// Validates the dependency graph for Package nodes, checking for various issues
    pub fn validate_package_dependencies(&self) -> Result<ValidationReport>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut report = ValidationReport::new();

        // Check for circular dependencies
        if let Err(e) = self.detect_circular_dependencies() {
            if let PkgError::CircularDependency { path } = e {
                report.add_issue(ValidationIssue::CircularDependency { path });
            } else {
                // Unexpected error type
                return Err(e);
            }
        }

        // Check for unresolved dependencies
        for dep in self.unresolved_dependencies() {
            report.add_issue(ValidationIssue::UnresolvedDependency {
                name: dep.name().to_string(),
                version_req: dep.version_str(),
            });
        }

        // Find version conflicts
        if let Some(conflicts) = self.find_version_conflicts() {
            for (name, versions) in conflicts {
                report.add_issue(ValidationIssue::VersionConflict { name, versions });
            }
        }

        Ok(report)
    }

    /// Check if dependencies can be upgraded to newer compatible versions
    pub fn check_upgradable_dependencies(&self) -> HashMap<String, Vec<(String, String)>>
    where
        N: Node<DependencyType = Dependency>,
    {
        // This would check a package registry for newer versions
        HashMap::new()
    }
}
