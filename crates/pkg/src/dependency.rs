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
impl<'a, N> From<&'a [N]> for DependencyGraph<'a, N>
where
    N: Node,
{
    fn from(nodes: &'a [N]) -> Self {
        let mut graph = StableDiGraph::<Step<'a, N>, &'a N::DependencyType>::new();
        let mut node_indices = HashMap::new();
        let mut dependents: HashMap<N::Identifier, Vec<N::Identifier>> = HashMap::new();

        // Insert the input nodes into the graph, and record their positions.
        // We'll be adding the edges next, and filling in any unresolved
        // steps we find along the way.
        let nodes: Vec<(_, _)> = nodes
            .iter()
            .map(|node| {
                let index = graph.add_node(Step::Resolved(node));
                node_indices.insert(node.identifier(), index);
                (node, index)
            })
            .collect();

        for (node, index) in &nodes {
            for dependency in node.dependencies() {
                // Check to see if we can resolve this dependency internally.
                if let Some((dep_node, dependent)) =
                    nodes.iter().find(|(dep, _)| dep.matches(dependency))
                {
                    // If we can, just add an edge between the two nodes.
                    graph.add_edge(*index, *dependent, dependency);
                    dependents.entry(dep_node.identifier()).or_default().push(node.identifier());
                } else {
                    // If not, create a new "Unresolved" node, and create an edge to that.
                    let unresolved = graph.add_node(Step::Unresolved(dependency));
                    graph.add_edge(*index, unresolved, dependency);
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
    /// True if all graph [`Node`]s have only references to other internal [`Node`]s.
    /// That is, there are no unresolved dependencies between nodes.
    pub fn is_internally_resolvable(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }

    /// Get an iterator over unresolved dependencies, without traversing the whole graph.
    /// Useful for doing pre-validation or pre-fetching of external dependencies before
    /// starting to resolve internal dependencies.
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType> {
        self.graph.node_weights().filter_map(Step::as_unresolved)
    }

    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N> {
        self.graph.node_weights().filter_map(Step::as_resolved)
    }

    // Get node index by identifier
    pub fn get_node_index(&self, id: &N::Identifier) -> Option<petgraph::stable_graph::NodeIndex> {
        self.node_indices.get(id).copied()
    }

    // Get node by identifier
    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>> {
        self.get_node_index(id).and_then(|idx| self.graph.node_weight(idx))
    }

    // Get dependents by identifier
    pub fn get_dependents(&self, id: &N::Identifier) -> Option<&Vec<N::Identifier>> {
        self.dependents.get(id)
    }

    // Propagate update starting from a node identified by its identifier
    pub fn propagate_update<F>(&self, start_id: &N::Identifier, mut update_fn: F)
    where
        F: FnMut(&Step<'a, N>, &[N::Identifier]),
    {
        let mut visited = HashSet::new();
        let mut stack = vec![start_id.clone()];

        while let Some(current_id) = stack.pop() {
            if visited.insert(current_id.clone()) {
                if let Some(node_weight) = self.get_node(&current_id) {
                    let empty_deps = vec![];
                    // Get dependents for current node
                    let deps = self.get_dependents(&current_id).unwrap_or(&empty_deps);

                    // Apply update
                    update_fn(node_weight, deps);

                    // Add dependents to stack for processing
                    if let Some(dependents) = self.get_dependents(&current_id) {
                        stack.extend(dependents.iter().cloned());
                    }
                }
            }
        }
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
        // Returns the first node, which does not have any Outgoing
        // edges, which means it is terminal.
        for index in self.graph.node_indices().rev() {
            if self.graph.neighbors_directed(index, Direction::Outgoing).count() == 0 {
                return self.graph.remove_node(index);
            }
        }

        None
    }
}
