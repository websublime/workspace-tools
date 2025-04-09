use std::collections::{HashMap, HashSet};

use crate::{
    Dependency, DependencyResolutionError, Node, PackageError, Step, ValidationIssue,
    ValidationOptions, ValidationReport,
};
use petgraph::algo::tarjan_scc;
use petgraph::stable_graph::NodeIndex;
use petgraph::{stable_graph::StableDiGraph, Direction};

#[derive(Debug, Clone)]
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
    // New field to store cycle information
    pub cycles: Vec<Vec<N::Identifier>>,
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

        // Detect cycles immediately using Tarjan's algorithm
        let mut pg_graph = petgraph::Graph::<(), (), petgraph::Directed>::new();
        let mut pg_indices = HashMap::new();

        // Create a petgraph representation
        for (id, &idx) in &node_indices {
            if let Some(Step::Resolved(_)) = graph.node_weight(idx) {
                let pg_idx = pg_graph.add_node(());
                pg_indices.insert(id.clone(), pg_idx);
            }
        }

        // Add edges
        for (id, &idx) in &node_indices {
            if let Some(&from_idx) = pg_indices.get(id) {
                for neighbor in graph.neighbors_directed(idx, Direction::Outgoing) {
                    if let Some(Step::Resolved(node)) = graph.node_weight(neighbor) {
                        let dep_id = node.identifier();
                        if let Some(&to_idx) = pg_indices.get(&dep_id) {
                            pg_graph.add_edge(from_idx, to_idx, ());
                        }
                    }
                }
            }
        }

        // Find strongly connected components (cycles)
        let sccs = tarjan_scc(&pg_graph);

        // Filter for actual cycles (SCCs with more than one node)
        let mut cycles = Vec::new();
        for scc in sccs {
            if scc.len() > 1 {
                let mut cycle = Vec::new();
                for &idx in &scc {
                    // Find the corresponding identifier
                    for (id, &pg_idx) in &pg_indices {
                        if pg_idx == idx {
                            cycle.push(id.clone());
                            break;
                        }
                    }
                }
                if !cycle.is_empty() {
                    cycles.push(cycle);
                }
            }
        }

        Self { graph, node_indices, dependents, cycles }
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
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    /// Check for circular dependencies in the graph
    ///
    /// This method no longer returns an error but instead provides information
    /// about any cycles detected in the dependency graph.
    ///
    /// Returns a reference to the graph for method chaining.
    pub fn detect_circular_dependencies(&self) -> &Self {
        // Cycles were already detected during construction
        self
    }

    /// Returns whether the graph has any circular dependencies
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Returns information about the cycles in the graph
    pub fn get_cycles(&self) -> &Vec<Vec<N::Identifier>> {
        &self.cycles
    }

    /// Get the cycle groups as strings for easier reporting
    pub fn get_cycle_strings(&self) -> Vec<Vec<String>> {
        self.cycles
            .iter()
            .map(|cycle| cycle.iter().map(std::string::ToString::to_string).collect())
            .collect()
    }

    // Add this new method with the correct name
    /// Find all external dependencies in the workspace (dependencies not found within the workspace)
    pub fn find_external_dependencies(&self) -> Vec<String>
    where
        N: Node<DependencyType = Dependency>,
    {
        let mut external = Vec::new();

        // Get all package names in the graph
        let resolved_node_names: HashSet<String> =
            self.resolved_dependencies().map(|node| node.identifier().to_string()).collect();

        // Check each unresolved dependency
        for dep in self.unresolved_dependencies() {
            let name = dep.name().to_string();
            if !resolved_node_names.contains(&name) {
                external.push(name);
            }
        }

        external
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

        // Check for circular dependencies - now just adds information to report
        if !self.cycles.is_empty() {
            // Add each cycle as a separate issue
            for cycle in self.get_cycle_strings() {
                report.add_issue(ValidationIssue::CircularDependency { path: cycle });
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

    /// Get dependents of a node, even if cycles exist
    pub fn get_dependents(&self, id: &N::Identifier) -> Result<&Vec<N::Identifier>, PackageError> {
        self.dependents.get(id).ok_or_else(|| PackageError::PackageNotFound(id.to_string()))
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

        // Check for circular dependencies - now just adds information to report
        if !self.cycles.is_empty() {
            // Add each cycle as a separate issue
            for cycle in self.get_cycle_strings() {
                report.add_issue(ValidationIssue::CircularDependency { path: cycle });
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
