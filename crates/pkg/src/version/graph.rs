//! Dependency graph construction and analysis for package dependencies.
//!
//! **What**: Provides the `DependencyGraph` structure that represents the dependency relationships
//! between packages in a workspace. The graph is used for detecting circular dependencies, finding
//! dependents, and analyzing the impact of version changes.
//!
//! **How**: Uses the `petgraph` crate to build a directed graph where nodes represent packages
//! and edges represent dependencies. The graph construction filters out external dependencies
//! and local/workspace protocol dependencies that should not be tracked. It provides efficient
//! lookup of packages and their relationships through a hash map index.
//!
//! **Why**: To enable dependency propagation, circular dependency detection, and impact analysis
//! when resolving versions. Understanding the dependency graph is crucial for determining which
//! packages need version updates when their dependencies change.

use crate::error::{VersionError, VersionResult};
use crate::types::{CircularDependency, PackageInfo};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::path::PathBuf;

/// Dependency graph representing package relationships.
///
/// The `DependencyGraph` uses a directed graph where:
/// - **Nodes** represent packages (identified by package name)
/// - **Edges** represent dependencies (directed from dependent to dependency)
///
/// For example, if package A depends on package B, there's an edge from A to B.
///
/// # Type Parameters
///
/// The graph uses `String` for node weights (package names) and `()` for edge weights
/// since we only care about the existence of dependencies, not their properties.
///
/// # Fields
///
/// * `graph` - The underlying directed graph structure
/// * `node_map` - Maps package names to their node indices for efficient lookup
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::version::DependencyGraph;
/// use sublime_pkg_tools::types::PackageInfo;
///
/// // Build graph from workspace packages
/// let packages: Vec<PackageInfo> = vec![/* ... */];
/// let graph = DependencyGraph::from_packages(&packages)?;
///
/// // Find all packages that depend on a specific package
/// let dependents = graph.dependents("my-package");
/// for dependent in dependents {
///     println!("{} depends on my-package", dependent);
/// }
///
/// // Detect circular dependencies
/// let cycles = graph.detect_cycles();
/// if !cycles.is_empty() {
///     println!("Warning: Circular dependencies detected!");
///     for cycle in cycles {
///         println!("  Cycle: {}", cycle.display_cycle());
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// The directed graph structure.
    /// Edges point from dependent to dependency (A -> B means A depends on B).
    graph: DiGraph<String, ()>,

    /// Maps package names to their node indices for O(1) lookup.
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Builds a dependency graph from a collection of packages.
    ///
    /// This method constructs the graph in two phases:
    /// 1. **Node creation**: Add all packages as nodes
    /// 2. **Edge creation**: Add dependency relationships between packages
    ///
    /// Only internal dependencies (dependencies between packages in the workspace)
    /// are added to the graph. External dependencies are filtered out. Additionally,
    /// workspace protocol and local protocol dependencies are excluded as they are
    /// handled separately.
    ///
    /// # Arguments
    ///
    /// * `packages` - A slice of `PackageInfo` representing all packages in the workspace
    ///
    /// # Returns
    ///
    /// Returns a new `DependencyGraph` or an error if the graph cannot be constructed.
    ///
    /// # Errors
    ///
    /// Currently does not return errors, but the error type is preserved for future
    /// validation requirements.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    /// use sublime_pkg_tools::types::PackageInfo;
    ///
    /// let packages = vec![
    ///     // Package A with no dependencies
    ///     PackageInfo::new(/* ... */),
    ///     // Package B that depends on A
    ///     PackageInfo::new(/* ... */),
    /// ];
    ///
    /// let graph = DependencyGraph::from_packages(&packages)?;
    /// assert_eq!(graph.package_count(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_packages(packages: &[PackageInfo]) -> VersionResult<Self> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Phase 1: Add all packages as nodes
        for pkg in packages {
            let name = pkg.name().to_string();
            let idx = graph.add_node(name.clone());
            node_map.insert(name, idx);
        }

        // Phase 2: Add dependency edges
        // Only add edges for internal dependencies (dependencies between workspace packages)
        for pkg in packages {
            let from_name = pkg.name();
            let from_idx =
                node_map.get(from_name).copied().ok_or_else(|| VersionError::PackageNotFound {
                    name: from_name.to_string(),
                    workspace_root: PathBuf::new(),
                })?;

            // Get all dependencies for this package
            let dependencies = pkg.all_dependencies();

            for (dep_name, _version_spec, _dep_type) in dependencies {
                // Only add edge if the dependency is another package in the workspace
                if let Some(&to_idx) = node_map.get(&dep_name) {
                    // Add edge from dependent to dependency (A -> B means A depends on B)
                    graph.add_edge(from_idx, to_idx, ());
                }
                // External dependencies are not added to the graph
            }
        }

        Ok(Self { graph, node_map })
    }

    /// Returns all packages that depend on the given package.
    ///
    /// This method finds all incoming edges to the specified package node,
    /// which represent packages that declare this package as a dependency.
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// A vector of package names that depend on the specified package.
    /// Returns an empty vector if the package is not found or has no dependents.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// let dependents = graph.dependents("core-package");
    /// for dependent in dependents {
    ///     println!("{} depends on core-package", dependent);
    /// }
    /// ```
    #[must_use]
    pub fn dependents(&self, package: &str) -> Vec<String> {
        if let Some(&idx) = self.node_map.get(package) {
            // Get all nodes that have edges pointing to this package
            // (incoming edges represent packages that depend on this one)
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .map(|neighbor_idx| self.graph[neighbor_idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns all packages that the given package depends on.
    ///
    /// This method finds all outgoing edges from the specified package node,
    /// which represent packages that this package declares as dependencies.
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to find dependencies for
    ///
    /// # Returns
    ///
    /// A vector of package names that the specified package depends on.
    /// Returns an empty vector if the package is not found or has no dependencies.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// let dependencies = graph.dependencies("my-app");
    /// for dependency in dependencies {
    ///     println!("my-app depends on {}", dependency);
    /// }
    /// ```
    #[must_use]
    pub fn dependencies(&self, package: &str) -> Vec<String> {
        if let Some(&idx) = self.node_map.get(package) {
            // Get all nodes that this package has edges pointing to
            // (outgoing edges represent dependencies)
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Outgoing)
                .map(|neighbor_idx| self.graph[neighbor_idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Checks if a package exists in the graph.
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to check
    ///
    /// # Returns
    ///
    /// `true` if the package exists in the graph, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// if graph.contains("my-package") {
    ///     println!("Package exists in workspace");
    /// } else {
    ///     println!("Package not found in workspace");
    /// }
    /// ```
    #[must_use]
    pub fn contains(&self, package: &str) -> bool {
        self.node_map.contains_key(package)
    }

    /// Returns the total number of packages in the graph.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// println!("Workspace contains {} packages", graph.package_count());
    /// ```
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.node_map.len()
    }

    /// Returns the total number of dependency relationships in the graph.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// println!("Total internal dependencies: {}", graph.edge_count());
    /// ```
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns all package names in the graph.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// for package in graph.all_packages() {
    ///     println!("Package: {}", package);
    /// }
    /// ```
    #[must_use]
    pub fn all_packages(&self) -> Vec<String> {
        self.node_map.keys().cloned().collect()
    }

    /// Detects circular dependencies in the graph.
    ///
    /// This method uses Tarjan's strongly connected components algorithm to find
    /// cycles in the dependency graph. A strongly connected component (SCC) with
    /// more than one node represents a circular dependency.
    ///
    /// # Returns
    ///
    /// A vector of `CircularDependency` structures, each representing a cycle.
    /// Returns an empty vector if no circular dependencies are found.
    ///
    /// # Algorithm
    ///
    /// Uses `petgraph::algo::tarjan_scc` to find all strongly connected components.
    /// Any SCC with more than one package indicates a circular dependency cycle.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// let cycles = graph.detect_cycles();
    ///
    /// if cycles.is_empty() {
    ///     println!("No circular dependencies found");
    /// } else {
    ///     println!("Found {} circular dependency cycle(s)", cycles.len());
    ///     for cycle in cycles {
    ///         println!("  Cycle: {}", cycle.display_cycle());
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn detect_cycles(&self) -> Vec<CircularDependency> {
        use petgraph::algo::tarjan_scc;

        // Find all strongly connected components
        let sccs = tarjan_scc(&self.graph);

        // Filter to only SCCs with more than one node (cycles)
        // and convert to CircularDependency structures
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                let cycle = scc.iter().map(|&idx| self.graph[idx].clone()).collect();
                CircularDependency::new(cycle)
            })
            .collect()
    }

    /// Returns the node index for a given package name.
    ///
    /// This is an internal method used for advanced graph operations.
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package
    ///
    /// # Returns
    ///
    /// The node index if the package exists, `None` otherwise.
    ///
    /// # Note
    ///
    /// TODO: will be implemented on story 5.5 (Dependency Propagation)
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn get_node_index(&self, package: &str) -> Option<NodeIndex> {
        self.node_map.get(package).copied()
    }

    /// Returns a reference to the underlying graph.
    ///
    /// This is an internal method that provides access to the raw petgraph structure
    /// for advanced operations.
    ///
    /// # Note
    ///
    /// TODO: will be implemented on story 5.5 (Dependency Propagation)
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn inner_graph(&self) -> &DiGraph<String, ()> {
        &self.graph
    }

    /// Finds all transitive dependents of a package.
    ///
    /// This method performs a breadth-first traversal to find all packages that
    /// transitively depend on the given package (directly or indirectly).
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to find transitive dependents for
    ///
    /// # Returns
    ///
    /// A vector of package names that transitively depend on the specified package.
    /// Returns an empty vector if the package is not found.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// // If A depends on B, and B depends on C, then transitive_dependents("C")
    /// // returns ["B", "A"]
    /// let transitive = graph.transitive_dependents("core-lib");
    /// println!("{} packages transitively depend on core-lib", transitive.len());
    /// ```
    #[must_use]
    pub fn transitive_dependents(&self, package: &str) -> Vec<String> {
        use petgraph::visit::Bfs;
        use std::collections::HashSet;

        let Some(&start_idx) = self.node_map.get(package) else {
            return Vec::new();
        };

        let mut visited = HashSet::new();
        let mut bfs = Bfs::new(&self.graph, start_idx);

        // Skip the starting node itself
        let _ = bfs.next(&self.graph);

        // Collect all reachable nodes following incoming edges (dependents)
        // We need to use a custom traversal because Bfs follows outgoing edges
        // but we want incoming edges for dependents
        let mut result = Vec::new();
        let mut to_visit = vec![start_idx];
        visited.insert(start_idx);

        while let Some(current) = to_visit.pop() {
            // Get all packages that depend on the current package
            for neighbor in self.graph.neighbors_directed(current, petgraph::Direction::Incoming) {
                if visited.insert(neighbor) {
                    result.push(self.graph[neighbor].clone());
                    to_visit.push(neighbor);
                }
            }
        }

        result
    }

    /// Finds all transitive dependencies of a package.
    ///
    /// This method performs a breadth-first traversal to find all packages that
    /// the given package transitively depends on (directly or indirectly).
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to find transitive dependencies for
    ///
    /// # Returns
    ///
    /// A vector of package names that the specified package transitively depends on.
    /// Returns an empty vector if the package is not found.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::version::DependencyGraph;
    ///
    /// # let graph: DependencyGraph = todo!();
    /// // If A depends on B, and B depends on C, then transitive_dependencies("A")
    /// // returns ["B", "C"]
    /// let transitive = graph.transitive_dependencies("my-app");
    /// println!("my-app transitively depends on {} packages", transitive.len());
    /// ```
    #[must_use]
    pub fn transitive_dependencies(&self, package: &str) -> Vec<String> {
        use petgraph::visit::Bfs;

        let Some(&start_idx) = self.node_map.get(package) else {
            return Vec::new();
        };

        let mut result = Vec::new();
        let mut bfs = Bfs::new(&self.graph, start_idx);

        // Skip the starting node itself
        let _ = bfs.next(&self.graph);

        // Collect all reachable nodes following outgoing edges (dependencies)
        while let Some(node_idx) = bfs.next(&self.graph) {
            result.push(self.graph[node_idx].clone());
        }

        result
    }
}
