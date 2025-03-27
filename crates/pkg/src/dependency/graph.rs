use std::collections::{HashMap, HashSet};

use crate::{
    Dependency, DependencyResolutionError, Node, PackageError, Step, ValidationIssue,
    ValidationOptions, ValidationReport,
};
use petgraph::algo::is_cyclic_directed;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::{stable_graph::StableDiGraph, Direction};

#[derive(Debug, Clone)]
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
}

impl<'a, N> From<&'a [N]> for DependencyGraph<'a, N>
where
    N: Node,
{
    fn from(nodes: &'a [N]) -> Self {
        let mut graph = StableDiGraph::<Step<'a, N>, ()>::new();
        let mut node_indices = HashMap::new();
        let mut dependents: HashMap<N::Identifier, Vec<N::Identifier>> = HashMap::new();

        // Insert nodes first
        for node in nodes {
            let idx = graph.add_node(Step::Resolved(node));
            node_indices.insert(node.identifier(), idx);
        }

        // Now process dependencies
        for node in nodes {
            let node_idx = node_indices[&node.identifier()];

            // Get dependencies for this node
            for dep_info in node.dependencies_vec() {
                // Try to find a matching node for this dependency
                let mut found_match = false;

                for dep_node in nodes {
                    if dep_node.matches(&dep_info) {
                        let dep_idx = node_indices[&dep_node.identifier()];

                        // Add edge without storing dependency reference
                        graph.add_edge(node_idx, dep_idx, ());

                        // Record dependent relationship
                        dependents
                            .entry(dep_node.identifier())
                            .or_default()
                            .push(node.identifier());

                        found_match = true;
                        break;
                    }
                }

                // If no matching node was found, create an unresolved node
                if !found_match {
                    // Create a new unresolved node with owned data
                    let unresolved = graph.add_node(Step::Unresolved(dep_info));

                    // Add edge
                    graph.add_edge(node_idx, unresolved, ());
                }
            }
        }

        Self { graph, node_indices, dependents }
    }
}

impl<'a, N> Iterator for DependencyGraph<'a, N>
where
    N: Node,
{
    type Item = Step<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        for index in self.graph.node_indices().rev() {
            if self.graph.neighbors_directed(index, Direction::Outgoing).count() == 0 {
                return self.graph.remove_node(index);
            }
        }
        None
    }
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    pub fn is_internally_resolvable(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }

    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType> {
        self.graph.node_weights().filter_map(Step::as_unresolved)
    }

    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N> {
        self.graph.node_weights().filter_map(Step::as_resolved)
    }

    pub fn get_node_index(&self, id: &N::Identifier) -> Option<NodeIndex> {
        self.node_indices.get(id).copied()
    }

    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>> {
        self.get_node_index(id).and_then(|idx| self.graph.node_weight(idx))
    }

    pub fn get_dependents(&self, id: &N::Identifier) -> Result<&Vec<N::Identifier>, PackageError> {
        self.dependents.get(id).ok_or_else(|| PackageError::PackageNotFound(id.to_string()))
    }
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    /// Check for circular dependencies in the graph
    pub fn detect_circular_dependencies(&self) -> Result<&Self, DependencyResolutionError> {
        if is_cyclic_directed(&self.graph) {
            // Find one of the cycles for more detailed error reporting
            let cycle = self.find_cycle();

            // Convert identifiers to strings for error reporting
            let cycle_strings: Vec<String> = cycle.into_iter().map(|id| id.to_string()).collect();

            return Err(DependencyResolutionError::CircularDependency { path: cycle_strings });
        }

        Ok(self)
    }

    /// Find a cycle in the graph, if one exists
    fn find_cycle(&self) -> Vec<N::Identifier> {
        let mut cycle = Vec::new();

        // Use Petgraph's cycle detection algorithm
        // A simpler approach that works for our needs
        for node_idx in self.graph.node_indices() {
            let node_weight = self.graph.node_weight(node_idx).expect("Node should exist");
            if let Step::Resolved(node) = node_weight {
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
                            if let Some(Step::Resolved(next_node)) =
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
        let resolved_node_names: HashSet<String> =
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
            if let Some(Step::Resolved(node)) = self.graph.node_weight(node_idx) {
                for dep in node.dependencies_vec() {
                    requirements_by_name.entry(dep.name().to_string()).or_default().push(
                        dep.fixed_version().map_or("no-version".to_string(), |v| v.to_string()),
                    );
                }
            }
        }

        // Find conflicting requirements
        for (name, reqs) in requirements_by_name {
            // Count unique version requirements
            let unique_reqs: HashSet<_> = reqs.iter().collect();

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
    pub fn validate_package_dependencies(
        &self,
    ) -> Result<ValidationReport, DependencyResolutionError>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut report = ValidationReport::new();

        // Check for circular dependencies
        if let Err(e) = self.detect_circular_dependencies() {
            if let DependencyResolutionError::CircularDependency { path } = e {
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
                version_req: dep.version().to_string(),
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

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    /// Validates the dependency graph for Package nodes with custom options
    pub fn validate_with_options(
        &self,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, DependencyResolutionError>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut report = ValidationReport::new();

        // Check for circular dependencies
        if let Err(e) = self.detect_circular_dependencies() {
            if let DependencyResolutionError::CircularDependency { path } = e {
                report.add_issue(ValidationIssue::CircularDependency { path });
            } else {
                // Unexpected error type
                return Err(e);
            }
        }

        // Check for unresolved dependencies
        for dep in self.unresolved_dependencies() {
            // Only add unresolved dependency issues if:
            // 1. We're not treating unresolved as external, OR
            // 2. This specific dependency is explicitly marked as internal
            let name = dep.name().to_string();

            if !options.treat_unresolved_as_external || options.is_internal_dependency(&name) {
                report.add_issue(ValidationIssue::UnresolvedDependency {
                    name,
                    version_req: dep.version().to_string(),
                });
            }
        }

        // Find version conflicts
        if let Some(conflicts) = self.find_version_conflicts() {
            for (name, versions) in conflicts {
                report.add_issue(ValidationIssue::VersionConflict { name, versions });
            }
        }

        Ok(report)
    }
}
