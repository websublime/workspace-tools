//! # Dependency Graph Module
//!
//! This module provides a graph-based representation of package dependencies.
//!
//! ## Overview
//!
//! The dependency graph is a core data structure that models the relationships between
//! packages in a project. It uses a directed graph where:
//!
//! - Nodes represent packages
//! - Edges represent dependencies between packages
//!
//! The graph allows for:
//! - Detecting circular dependencies
//! - Finding external dependencies (not resolved within the workspace)
//! - Identifying version conflicts
//! - Validating the dependency tree
//!
//! ## Implementation
//!
//! The graph is implemented using the `petgraph` crate's `StableDiGraph` type, which
//! provides a stable node indexing system and directed edges.
//!
//! ## Examples
//!
//! ```
//! use sublime_package_tools::{DependencyGraph, Package, ValidationOptions};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let packages = vec![];
//! // Create a dependency graph from packages
//! let graph = DependencyGraph::from(packages.as_slice());
//!
//! // Detect circular dependencies
//! let graph = graph.detect_circular_dependencies();
//! if graph.has_cycles() {
//!     println!("Found circular dependencies:");
//!     for cycle in graph.get_cycle_strings() {
//!         println!("  Cycle: {}", cycle.join(" -> "));
//!     }
//! }
//!
//! // Validate the graph with custom options
//! let options = ValidationOptions::new()
//!     .treat_unresolved_as_external(true)
//!     .with_internal_packages(vec!["@mycompany/ui", "@mycompany/core"]);
//!
//! let report = graph.validate_with_options(&options)?;
//!
//! if report.has_critical_issues() {
//!     println!("Found critical issues:");
//!     for issue in report.critical_issues() {
//!         println!("  {}", issue.message());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::{HashMap, HashSet};

use crate::{
    errors::{DependencyResolutionError, PackageError},
    Dependency, Node, Step, ValidationIssue, ValidationOptions, ValidationReport,
};
use petgraph::algo::tarjan_scc;
use petgraph::stable_graph::NodeIndex;
use petgraph::{stable_graph::StableDiGraph, Direction};

/// A graph representation of dependencies between packages.
///
/// The dependency graph models the relationships between packages, allowing for:
/// - Detecting circular dependencies
/// - Finding external dependencies
/// - Identifying version conflicts
/// - Validating the dependency tree
///
/// The graph is generic over the node type `N`, which must implement the `Node` trait.
/// This allows the graph to work with different package representations.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the nodes in the graph
/// * `N` - Type of nodes in the graph, must implement the `Node` trait
///
/// # Examples
///
/// ```
/// use sublime_package_tools::{DependencyGraph, Package};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let packages = vec![];
/// // Create a dependency graph from packages
/// let graph = DependencyGraph::from(packages.as_slice());
///
/// // Check if all dependencies are resolved within the graph
/// if graph.is_internally_resolvable() {
///     println!("All dependencies are internally resolvable");
/// } else {
///     println!("Graph has external dependencies");
/// }
/// # Ok(())
/// # }
/// ```
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct Graph<'a, N: Node> {
    /// The underlying graph structure
    pub graph: StableDiGraph<Step<'a, N>, ()>,
    /// Mapping from node identifiers to graph indices
    pub node_indices: HashMap<N::Identifier, NodeIndex>,
    /// Mapping from node identifiers to their dependents' identifiers
    pub dependents: HashMap<N::Identifier, Vec<N::Identifier>>,
    /// Information about cycles in the graph
    pub cycles: Vec<Vec<N::Identifier>>,
}

impl<'a, N> From<&'a [N]> for Graph<'a, N>
where
    N: Node,
{
    /// Creates a dependency graph from a slice of nodes.
    ///
    /// This constructor builds a graph representation of the dependencies between
    /// the provided nodes, identifying both resolved and unresolved dependencies.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A slice of nodes implementing the `Node` trait
    ///
    /// # Returns
    ///
    /// A `Graph` representing the dependencies between the provided nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{Dependency, DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create some packages
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// // Create a dependency graph
    /// let graph = Graph::from(packages.as_slice());
    /// # Ok(())
    /// # }
    /// ```
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

impl<'a, N> Iterator for Graph<'a, N>
where
    N: Node,
{
    type Item = Step<'a, N>;

    /// Returns the next resolved node in topological order (leaf nodes first).
    ///
    /// This iterator removes and returns nodes from the graph in dependency order,
    /// ensuring that nodes are only returned after all their dependencies have been
    /// processed. This is useful for operations that need to be performed in
    /// dependency order, such as building or publishing packages.
    ///
    /// # Returns
    ///
    /// The next node in topological order, or None if all nodes have been processed.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, Dependency, DependencyRegistry, Package, Step};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![("lib", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// // Create a dependency graph
    /// let mut graph = Graph::from(packages.as_slice());
    ///
    /// // Process nodes in dependency order (leaves first)
    /// while let Some(step) = graph.next() {
    ///     if let Step::Resolved(node) = step {
    ///         println!("Processing: {}", node.identifier());
    ///         // Do something with the node...
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        for index in self.graph.node_indices().rev() {
            if self.graph.neighbors_directed(index, Direction::Outgoing).count() == 0 {
                return self.graph.remove_node(index);
            }
        }
        None
    }
}

impl<'a, N> Graph<'a, N>
where
    N: Node,
{
    /// Checks if all dependencies in the graph can be resolved internally.
    ///
    /// A graph is internally resolvable if all dependency relationships point to
    /// nodes that are present in the graph, with no unresolved dependencies.
    ///
    /// # Returns
    ///
    /// `true` if all dependencies are resolved within the graph, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![("lib", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// if graph.is_internally_resolvable() {
    ///     println!("All dependencies are resolved within the workspace");
    /// } else {
    ///     println!("There are external dependencies not in the workspace");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_internally_resolvable(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }

    /// Returns an iterator over unresolved dependencies in the graph.
    ///
    /// Unresolved dependencies are those that are referenced by nodes in the graph
    /// but don't have a corresponding resolved node. These typically represent
    /// external dependencies not included in the local workspace.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to unresolved dependencies.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry(
    ///         "app",
    ///         "1.0.0",
    ///         Some(vec![("lib", "^1.0.0"), ("react", "^17.0.0")]),  // react is external
    ///         &mut registry
    ///     )?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// println!("External dependencies:");
    /// for dep in graph.unresolved_dependencies() {
    ///     println!("  {} {}", dep.name(), dep.version());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType> {
        self.graph.node_weights().filter_map(Step::as_unresolved)
    }

    /// Returns an iterator over resolved nodes in the graph.
    ///
    /// Resolved nodes are those that have a concrete implementation in the graph,
    /// not just references from other dependencies.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to resolved nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![("lib", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// println!("Resolved packages:");
    /// for pkg in graph.resolved_dependencies() {
    ///     println!("  {} v{}", pkg.name(), pkg.version_str());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N> {
        self.graph.node_weights().filter_map(Step::as_resolved)
    }

    /// Gets the graph index for a node with the given identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the node to find
    ///
    /// # Returns
    ///
    /// The graph index of the node if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// if let Some(idx) = graph.get_node_index(&"app".to_string()) {
    ///     println!("Found node index for 'app'");
    /// } else {
    ///     println!("'app' not found in graph");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_node_index(&self, id: &N::Identifier) -> Option<NodeIndex> {
        self.node_indices.get(id).copied()
    }

    /// Gets the node with the given identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the node to find
    ///
    /// # Returns
    ///
    /// A reference to the node if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package, Step};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// if let Some(node) = graph.get_node(&"app".to_string()) {
    ///     if let Step::Resolved(pkg) = node {
    ///         println!("Found package: {} v{}", pkg.name(), pkg.version_str());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_node(&self, id: &N::Identifier) -> Option<&Step<'a, N>> {
        self.get_node_index(id).and_then(|idx| self.graph.node_weight(idx))
    }
}

impl<'a, N> Graph<'a, N>
where
    N: Node,
{
    /// Detects circular dependencies in the graph.
    ///
    /// Circular dependencies occur when packages depend on each other in a cycle,
    /// creating a situation where none of the packages can be built first.
    ///
    /// Note: This method doesn't return an error anymore but provides information
    /// about any cycles detected in the dependency graph through the `cycles` field.
    ///
    /// # Returns
    ///
    /// A reference to the graph for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    /// let graph = graph.detect_circular_dependencies();
    ///
    /// if graph.has_cycles() {
    ///     println!("Circular dependencies detected:");
    ///     for cycle in graph.get_cycle_strings() {
    ///         println!("  Cycle: {}", cycle.join(" -> "));
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn detect_circular_dependencies(&self) -> &Self {
        // Cycles were already detected during construction
        self
    }

    /// Checks if the graph has any circular dependencies.
    ///
    /// # Returns
    ///
    /// `true` if the graph contains any cycles, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// if graph.has_cycles() {
    ///     println!("Warning: Circular dependencies detected!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Returns information about the cycles in the graph.
    ///
    /// # Returns
    ///
    /// A reference to the vector of cycles, where each cycle is represented as
    /// a vector of node identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-c", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-c", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// for cycle in graph.get_cycles() {
    ///     println!("Cycle detected: {:?}", cycle);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_cycles(&self) -> &Vec<Vec<N::Identifier>> {
        &self.cycles
    }

    /// Get the cycle information as strings for easier reporting.
    ///
    /// # Returns
    ///
    /// A vector of cycles, where each cycle is represented as a vector of strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// for cycle in graph.get_cycle_strings() {
    ///     println!("Cycle: {}", cycle.join(" -> "));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn get_cycle_strings(&self) -> Vec<Vec<String>> {
        self.cycles
            .iter()
            .map(|cycle| cycle.iter().map(std::string::ToString::to_string).collect())
            .collect()
    }

    /// Find all external dependencies in the workspace (dependencies not found within the workspace).
    ///
    /// External dependencies are those that are referenced by packages in the workspace
    /// but are not themselves part of the workspace.
    ///
    /// # Returns
    ///
    /// A vector of external dependency names.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry(
    ///         "app",
    ///         "1.0.0",
    ///         Some(vec![("lib", "^1.0.0"), ("react", "^17.0.0")]),  // react is external
    ///         &mut registry
    ///     )?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// let externals = graph.find_external_dependencies();
    /// println!("External dependencies: {:?}", externals);  // Should contain "react"
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
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

    /// Find all version conflicts in the graph for Package nodes.
    ///
    /// Version conflicts occur when multiple packages depend on different versions of the same package.
    ///
    /// # Returns
    ///
    /// A `HashMap` where keys are package names with conflicts and values are lists of conflicting versions.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("shared", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("shared", "^2.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// if let Some(conflicts) = graph.find_version_conflicts() {
    ///     println!("Version conflicts detected:");
    ///     for (name, versions) in conflicts {
    ///         println!("  Package '{}' has conflicting versions: {}", name, versions.join(", "));
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
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

    /// Find all version conflicts in the dependency graph.
    ///
    /// This is a convenience wrapper around `find_version_conflicts_for_package` that
    /// returns `None` if there are no conflicts.
    ///
    /// # Returns
    ///
    /// `Some(conflicts)` if conflicts are found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("react", "^16.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("react", "^17.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// match graph.find_version_conflicts() {
    ///     Some(conflicts) => {
    ///         println!("Found version conflicts:");
    ///         for (name, versions) in conflicts {
    ///             println!("  {} has conflicting versions: {}", name, versions.join(", "));
    ///         }
    ///     }
    ///     None => println!("No version conflicts detected"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
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

    /// Validates the dependency graph for Package nodes, checking for various issues.
    ///
    /// This performs a comprehensive validation of the dependency graph, checking for:
    /// - Circular dependencies
    /// - Unresolved dependencies
    /// - Version conflicts
    ///
    /// # Returns
    ///
    /// A validation report containing any issues found.
    ///
    /// # Errors
    ///
    /// Returns a `DependencyResolutionError` if the validation process itself fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("external", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    /// let report = graph.validate_package_dependencies()?;
    ///
    /// if report.has_issues() {
    ///     println!("Validation found issues:");
    ///     for issue in report.issues() {
    ///         println!("  {}", issue.message());
    ///     }
    /// } else {
    ///     println!("No issues found!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unnecessary_wraps)]
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

    /// Get dependents of a node, even if cycles exist.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the node to find dependents for
    ///
    /// # Returns
    ///
    /// A vector of identifiers for nodes that depend on the specified node.
    ///
    /// # Errors
    ///
    /// Returns a `PackageError::PackageNotFound` if the node is not found in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![("lib", "^1.0.0")]), &mut registry)?,
    ///     Package::new_with_registry("lib", "1.0.0", Some(vec![]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// // Find packages that depend on 'lib'
    /// let dependents = graph.get_dependents(&"lib".to_string())?;
    /// println!("Packages depending on 'lib': {:?}", dependents);  // Should include 'app'
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_dependents(
        &mut self,
        id: &N::Identifier,
    ) -> Result<&Vec<N::Identifier>, PackageError> {
        // First check if the package exists in the graph
        if !self.node_indices.contains_key(id) {
            return Err(PackageError::PackageNotFound(format!("{id:?}")));
        }

        // Use entry API to insert an empty vector if the key doesn't exist
        Ok(self.dependents.entry(id.clone()).or_default())
    }

    /// Check if dependencies can be upgraded to newer compatible versions.
    ///
    /// This method would normally query a package registry to find newer versions,
    /// but in this implementation it returns an empty HashMap since it's not
    /// connected to an actual registry.
    ///
    /// # Returns
    ///
    /// A HashMap mapping package names to lists of possible upgrades, where each
    /// upgrade is a tuple of (current version, new version).
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry("app", "1.0.0", Some(vec![("react", "^16.0.0")]), &mut registry)?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    /// let upgrades = graph.check_upgradable_dependencies();
    ///
    /// // In a real implementation, this might contain upgrade information
    /// assert!(upgrades.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn check_upgradable_dependencies(&self) -> HashMap<String, Vec<(String, String)>>
    where
        N: Node<DependencyType = Dependency>,
    {
        // This would check a package registry for newer versions
        HashMap::new()
    }
}

impl<'a, N> Graph<'a, N>
where
    N: Node,
{
    /// Validates the dependency graph for Package nodes with custom options.
    ///
    /// This performs validation with customizable behavior, such as:
    /// - Treating unresolved dependencies as external (not errors)
    /// - Specifying which packages should be considered internal
    ///
    /// # Arguments
    ///
    /// * `options` - The validation options to use
    ///
    /// # Returns
    ///
    /// A validation report containing any issues found.
    ///
    /// # Errors
    ///
    /// Returns a `DependencyResolutionError` if the validation process itself fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::{DependencyGraph, DependencyRegistry, Package, ValidationOptions};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut registry = DependencyRegistry::new();
    /// let packages = vec![
    ///     Package::new_with_registry(
    ///         "app",
    ///         "1.0.0",
    ///         Some(vec![("internal", "^1.0.0"), ("react", "^17.0.0")]),
    ///         &mut registry
    ///     )?,
    /// ];
    ///
    /// let graph = Graph::from(packages.as_slice());
    ///
    /// // Create custom validation options
    /// let options = ValidationOptions::new()
    ///     .treat_unresolved_as_external(true)  // Don't flag external dependencies as errors
    ///     .with_internal_packages(vec!["internal"]);  // But "internal" should be an error if missing
    ///
    /// let report = graph.validate_with_options(&options)?;
    ///
    /// // This should only report "internal" as an issue, not "react"
    /// for issue in report.issues() {
    ///     println!("{}", issue.message());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::unnecessary_wraps)]
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

/// Type alias for backward compatibility
///
/// # Deprecation
///
/// This alias maintains compatibility with existing code.
/// Prefer using `Graph` directly in new code.
pub type DependencyGraph<'a, N> = Graph<'a, N>;
