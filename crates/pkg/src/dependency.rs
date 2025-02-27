use petgraph::{stable_graph::StableDiGraph, Direction};
use std::{collections::HashMap, fmt::Display};

/// Must be implemented by the type you wish
pub trait Node {
    /// Encodes a dependency relationship. In a Package Manager dependency graph for instance, this might be a (package name, version) tuple.
    /// It might also just be the exact same type as the one that implements the Node trait, in which case `Node::matches` can be implemented through simple equality.
    type DependencyType: std::fmt::Debug + Clone;

    type Identifier: std::hash::Hash + Eq + Clone;

    /// Returns a slice of dependencies for this Node
    fn dependencies(&self) -> &[Self::DependencyType];
    fn dependencies_vec(&self) -> Vec<Self::DependencyType>;
    /// Returns true if the `dependency` can be met by us.
    fn matches(&self, dependency: &Self::DependencyType) -> bool;

    fn identifier(&self) -> Self::Identifier;
}

/// Wrapper around dependency graph nodes.
/// Since a graph might have dependencies that cannot be resolved internally,
/// this wrapper is necessary to differentiate between internally resolved and
/// externally (unresolved) dependencies.
/// An Unresolved dependency does not necessarily mean that it *cannot* be resolved,
/// only that no Node within the graph fulfills it.
#[derive(Debug, Clone)]
pub enum Step<'a, N: Node> {
    Resolved(&'a N),
    Unresolved(N::DependencyType), // Own the dependency info
}

impl<'a, N: Node> Step<'a, N> {
    pub fn is_resolved(&self) -> bool {
        match self {
            Step::Resolved(_) => true,
            Step::Unresolved(_) => false,
        }
    }

    pub fn as_resolved(&self) -> Option<&N> {
        match self {
            Step::Resolved(node) => Some(node),
            Step::Unresolved(_) => None,
        }
    }

    pub fn as_unresolved(&self) -> Option<&N::DependencyType> {
        match self {
            Step::Resolved(_) => None,
            Step::Unresolved(dependency) => Some(dependency),
        }
    }
}

impl<'a, N: Node> Display for Step<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Resolved(_node) => write!(f, "Resolved"),
            Step::Unresolved(_dependency) => write!(f, "Unresolved"),
        }
    }
}

/// The [`DependencyGraph`] structure builds an internal [Directed Graph](`petgraph::stable_graph::StableDiGraph`), which can then be traversed
/// in an order which ensures that dependent Nodes are visited before their parents.
#[derive(Debug, Clone)]
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, ()>, // Changed to unit type
    pub node_indices: HashMap<N::Identifier, petgraph::stable_graph::NodeIndex>,
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
}

/// The only way to build a [`DependencyGraph`] is from a slice of objects implementing [`Node`].
/// The graph references the original items, meaning the objects cannot be modified while
/// the [`DependencyGraph`] holds a reference to them.
#[allow(clippy::ref_as_ptr)]
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

    pub fn get_dependents(&self, id: &N::Identifier) -> Option<&Vec<N::Identifier>> {
        self.dependents.get(id)
    }
}

/// Iterate over the DependencyGraph in an order which ensures dependencies are resolved before each Node is visited.
/// Note: If a `Step::Unresolved` node is returned, it is the caller's responsibility to ensure the dependency is resolved
/// before continuing.
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
