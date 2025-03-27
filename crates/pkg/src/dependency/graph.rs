use std::collections::HashMap;

use crate::{Node, PackageError, Step};
use petgraph::stable_graph::NodeIndex;
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
