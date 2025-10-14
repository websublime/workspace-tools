use std::{collections::HashMap, path::PathBuf};

use petgraph::Graph;
use serde::{Deserialize, Serialize};

use crate::{error::DependencyError, PackageResult, ResolvedVersion};

/// Type of dependency relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Regular runtime dependency
    Runtime,
    /// Development dependency
    Development,
    /// Optional dependency
    Optional,
    /// Peer dependency
    Peer,
}

/// Graph representation of package dependencies.
///
/// Maintains the relationship between packages and their dependencies
/// for analysis and propagation calculations.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// The underlying graph structure
    pub(crate) graph: Graph<DependencyNode, DependencyEdge>,
    /// Mapping from package names to graph node indices
    pub(crate) package_index: HashMap<String, petgraph::graph::NodeIndex>,
}

/// Individual package node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Package name
    pub name: String,
    /// Current version
    pub version: ResolvedVersion,
    /// Path to package directory
    pub path: PathBuf,
    /// Regular dependencies
    pub dependencies: HashMap<String, String>,
    /// Development dependencies
    pub dev_dependencies: HashMap<String, String>,
    /// Optional dependencies
    pub optional_dependencies: HashMap<String, String>,
    /// Peer dependencies
    pub peer_dependencies: HashMap<String, String>,
}

/// Edge in the dependency graph representing a dependency relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Version requirement
    pub version_requirement: String,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self { graph: Graph::new(), package_index: HashMap::new() }
    }

    /// Adds a package node to the graph.
    ///
    /// # Arguments
    ///
    /// * `node` - The dependency node to add
    ///
    /// # Returns
    ///
    /// The node index in the graph
    pub fn add_node(&mut self, node: DependencyNode) -> petgraph::graph::NodeIndex {
        let node_index = self.graph.add_node(node.clone());
        self.package_index.insert(node.name.clone(), node_index);
        node_index
    }

    /// Adds a dependency edge between two packages.
    ///
    /// # Arguments
    ///
    /// * `from_package` - Source package name
    /// * `to_package` - Target package name
    /// * `edge` - Dependency edge information
    pub fn add_edge(
        &mut self,
        from_package: &str,
        to_package: &str,
        edge: DependencyEdge,
    ) -> PackageResult<()> {
        let from_index = self.package_index.get(from_package).ok_or_else(|| {
            DependencyError::MissingDependency {
                package: from_package.to_string(),
                dependency: to_package.to_string(),
            }
        })?;

        let to_index = self.package_index.get(to_package).ok_or_else(|| {
            DependencyError::MissingDependency {
                package: from_package.to_string(),
                dependency: to_package.to_string(),
            }
        })?;

        self.graph.add_edge(*from_index, *to_index, edge);
        Ok(())
    }

    /// Gets a package node by name.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find
    #[must_use]
    pub fn get_package(&self, package_name: &str) -> Option<&DependencyNode> {
        self.package_index.get(package_name).and_then(|&index| self.graph.node_weight(index))
    }

    /// Detects circular dependencies in the graph.
    ///
    /// Uses Tarjan's strongly connected components algorithm via petgraph
    /// to find all cycles in the dependency graph.
    ///
    /// # Returns
    ///
    /// Vector of cycles found, each cycle is a vector of package names
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::dependency::{DependencyGraph, DependencyNode, DependencyEdge, DependencyType};
    /// use sublime_pkg_tools::version::Version;
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    ///
    /// let mut graph = DependencyGraph::new();
    /// let version = Version::from_str("1.0.0").unwrap();
    ///
    /// // Add nodes that form a cycle
    /// let node_a = DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
    /// let node_b = DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
    /// graph.add_node(node_a);
    /// graph.add_node(node_b);
    ///
    /// // Add edges that create a cycle: A -> B -> A
    /// let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
    /// graph.add_edge("pkg-a", "pkg-b", edge.clone()).unwrap();
    /// graph.add_edge("pkg-b", "pkg-a", edge).unwrap();
    ///
    /// let cycles = graph.detect_cycles();
    /// assert!(!cycles.is_empty());
    /// ```
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        use petgraph::algo::tarjan_scc;

        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            // A strongly connected component with more than one node,
            // or a single node with a self-loop, indicates a cycle
            if scc.len() > 1 || (scc.len() == 1 && self.graph.find_edge(scc[0], scc[0]).is_some()) {
                let cycle_names: Vec<String> = scc
                    .into_iter()
                    .filter_map(|node_idx| {
                        self.graph.node_weight(node_idx).map(|node| node.name.clone())
                    })
                    .collect();

                if !cycle_names.is_empty() {
                    cycles.push(cycle_names);
                }
            }
        }

        cycles
    }

    /// Gets all direct dependents of a package.
    ///
    /// Returns packages that directly depend on the specified package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// Vector of package names that depend on the specified package
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::dependency::{DependencyGraph, DependencyNode, DependencyEdge, DependencyType};
    /// use sublime_pkg_tools::version::Version;
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    ///
    /// let mut graph = DependencyGraph::new();
    /// let version = Version::from_str("1.0.0").unwrap();
    ///
    /// // Add packages
    /// let node_a = DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
    /// let node_b = DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
    /// graph.add_node(node_a);
    /// graph.add_node(node_b);
    ///
    /// // B depends on A
    /// let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
    /// graph.add_edge("pkg-b", "pkg-a", edge).unwrap();
    ///
    /// let dependents = graph.get_dependents("pkg-a");
    /// assert_eq!(dependents, vec!["pkg-b"]);
    /// ```
    #[must_use]
    pub fn get_dependents(&self, package_name: &str) -> Vec<String> {
        let Some(&target_index) = self.package_index.get(package_name) else {
            return Vec::new();
        };

        self.graph
            .neighbors_directed(target_index, petgraph::Direction::Incoming)
            .filter_map(|node_idx| self.graph.node_weight(node_idx).map(|node| node.name.clone()))
            .collect()
    }

    /// Gets all direct dependencies of a package.
    ///
    /// Returns packages that the specified package directly depends on.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependencies for
    ///
    /// # Returns
    ///
    /// Vector of package names that the specified package depends on
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_pkg_tools::dependency::{DependencyGraph, DependencyNode, DependencyEdge, DependencyType};
    /// use sublime_pkg_tools::version::Version;
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    ///
    /// let mut graph = DependencyGraph::new();
    /// let version = Version::from_str("1.0.0").unwrap();
    ///
    /// // Add packages
    /// let node_a = DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
    /// let node_b = DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
    /// graph.add_node(node_a);
    /// graph.add_node(node_b);
    ///
    /// // B depends on A
    /// let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
    /// graph.add_edge("pkg-b", "pkg-a", edge).unwrap();
    ///
    /// let dependencies = graph.get_dependencies("pkg-b");
    /// assert_eq!(dependencies, vec!["pkg-a"]);
    /// ```
    #[must_use]
    pub fn get_dependencies(&self, package_name: &str) -> Vec<String> {
        let Some(&source_index) = self.package_index.get(package_name) else {
            return Vec::new();
        };

        self.graph
            .neighbors_directed(source_index, petgraph::Direction::Outgoing)
            .filter_map(|node_idx| self.graph.node_weight(node_idx).map(|node| node.name.clone()))
            .collect()
    }
}

impl DependencyNode {
    /// Creates a new dependency node.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Current version
    /// * `path` - Path to package directory
    #[must_use]
    pub fn new(name: String, version: ResolvedVersion, path: PathBuf) -> Self {
        Self {
            name,
            version,
            path,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
        }
    }

    /// Adds a runtime dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - Dependency name
    /// * `version_req` - Version requirement
    pub fn add_dependency(&mut self, name: String, version_req: String) {
        self.dependencies.insert(name, version_req);
    }

    /// Adds a development dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - Dependency name
    /// * `version_req` - Version requirement
    pub fn add_dev_dependency(&mut self, name: String, version_req: String) {
        self.dev_dependencies.insert(name, version_req);
    }

    /// Adds an optional dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - Dependency name
    /// * `version_req` - Version requirement
    pub fn add_optional_dependency(&mut self, name: String, version_req: String) {
        self.optional_dependencies.insert(name, version_req);
    }

    /// Adds a peer dependency.
    ///
    /// # Arguments
    ///
    /// * `name` - Dependency name
    /// * `version_req` - Version requirement
    pub fn add_peer_dependency(&mut self, name: String, version_req: String) {
        self.peer_dependencies.insert(name, version_req);
    }

    /// Gets all dependencies of a specific type.
    ///
    /// # Arguments
    ///
    /// * `dependency_type` - Type of dependencies to retrieve
    #[must_use]
    pub fn get_dependencies(&self, dependency_type: DependencyType) -> &HashMap<String, String> {
        match dependency_type {
            DependencyType::Runtime => &self.dependencies,
            DependencyType::Development => &self.dev_dependencies,
            DependencyType::Optional => &self.optional_dependencies,
            DependencyType::Peer => &self.peer_dependencies,
        }
    }
}

impl DependencyEdge {
    /// Creates a new dependency edge.
    ///
    /// # Arguments
    ///
    /// * `dependency_type` - Type of dependency
    /// * `version_requirement` - Version requirement string
    #[must_use]
    pub fn new(dependency_type: DependencyType, version_requirement: String) -> Self {
        Self { dependency_type, version_requirement }
    }
}
