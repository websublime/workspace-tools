//! Graph construction utilities.

use crate::error::Result;
use crate::graph::node::{Node, Step};
use crate::types::package::{Package, PackageInfo};
use petgraph::{stable_graph::StableDiGraph, Direction};
use std::collections::HashMap;

/// Dependency types to include in upgrades
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DependencyFilter {
    /// Include only production dependencies
    ProductionOnly,
    /// Include production and development dependencies
    WithDevelopment,
    /// Include production, development, and optional dependencies
    AllDependencies,
}

impl Default for DependencyFilter {
    fn default() -> Self {
        Self::WithDevelopment
    }
}

/// The dependency graph structure
#[derive(Debug, Clone)]
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, ()>,
    pub node_indices: HashMap<N::Identifier, petgraph::stable_graph::NodeIndex>,
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

    pub fn get_node_index(&self, id: &N::Identifier) -> Option<petgraph::stable_graph::NodeIndex> {
        self.node_indices.get(id).copied()
    }

    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>> {
        self.get_node_index(id).and_then(|idx| self.graph.node_weight(idx))
    }

    pub fn get_dependents(&self, id: &N::Identifier) -> Result<&Vec<N::Identifier>> {
        self.dependents
            .get(id)
            .ok_or_else(|| crate::error::PkgError::PackageNotFound { name: id.to_string() })
    }
}

/// Build a dependency graph from packages
pub fn build_dependency_graph_from_packages(packages: &[Package]) -> DependencyGraph<'_, Package> {
    DependencyGraph::from(packages)
}

/// Build a dependency graph from package infos
pub fn build_dependency_graph_from_package_infos<'a>(
    package_infos: &[PackageInfo],
    packages: &'a mut Vec<Package>,
) -> DependencyGraph<'a, Package> {
    // Extract packages from package infos
    packages.clear();
    for pkg_info in package_infos {
        packages.push(pkg_info.package.borrow().clone());
    }

    // Build dependency graph
    build_dependency_graph_from_packages(packages)
}
