use petgraph::{stable_graph::StableDiGraph, Direction};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

/// Must be implemented by the type you wish
pub trait Node {
    /// Encodes a dependency relationship. In a Package Manager dependency graph for instance, this might be a (package name, version) tuple.
    /// It might also just be the exact same type as the one that implements the Node trait, in which case `Node::matches` can be implemented through simple equality.
    type DependencyType;

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
    Unresolved(&'a N::DependencyType),
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
    pub graph: StableDiGraph<Step<'a, N>, &'a N::DependencyType>,
    // Map node identifiers to their indices
    pub node_indices: HashMap<N::Identifier, petgraph::stable_graph::NodeIndex>,
    // Map node identifiers to their dependents' identifiers
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
        let mut graph = StableDiGraph::<Step<'a, N>, &'a N::DependencyType>::new();
        let mut node_indices = HashMap::new();
        let mut dependents: HashMap<N::Identifier, Vec<N::Identifier>> = HashMap::new();

        // Store owned copies of dependencies to create safe references
        let mut all_dependencies: Vec<Vec<N::DependencyType>> = Vec::new();

        // Insert nodes
        let nodes_with_indices: Vec<(_, _)> = nodes
            .iter()
            .map(|node| {
                let index = graph.add_node(Step::Resolved(node));
                node_indices.insert(node.identifier(), index);
                (node, index)
            })
            .collect();

        // Process dependencies
        for (node, node_idx) in &nodes_with_indices {
            // Get dependencies using the new method
            let deps = node.dependencies_vec();
            all_dependencies.push(deps);
            let deps = all_dependencies.last().unwrap();

            for dependency in deps {
                // Try to find a matching node
                if let Some((dep_node, dependent)) =
                    nodes_with_indices.iter().find(|(dep, _)| dep.matches(dependency))
                {
                    // Create edge with a reference to the dependency
                    let dep_ref: &N::DependencyType =
                        unsafe { &*(dependency as *const N::DependencyType) };

                    graph.add_edge(*node_idx, *dependent, dep_ref);
                    dependents.entry(dep_node.identifier()).or_default().push(node.identifier());
                } else {
                    // Create unresolved node
                    let dep_ref: &N::DependencyType =
                        unsafe { &*(dependency as *const N::DependencyType) };

                    let unresolved = graph.add_node(Step::Unresolved(dep_ref));
                    graph.add_edge(*node_idx, unresolved, dep_ref);
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
