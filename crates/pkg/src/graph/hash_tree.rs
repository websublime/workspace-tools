//! # Dependency Hash Tree - Structured Queryable Model
//!
//! This module provides a structured, queryable model for dependency graphs.
//! Unlike traditional graph visualizations (ASCII/DOT), this hash tree serves as a
//! queryable data structure that enables efficient analysis of package dependencies,
//! dependents, and dependency paths.
//!
//! ## What
//! 
//! The `DependencyHashTree` is a structured model that represents package dependency
//! relationships as queryable data. It maintains bidirectional mappings between packages
//! and their dependencies, enabling efficient queries about dependency relationships.
//!
//! ## How
//! 
//! The implementation uses HashMap-based data structures for O(1) lookups and maintains
//! both forward (depends_on) and reverse (dependency_of) mappings. The tree structure
//! allows for efficient traversal and analysis of dependency relationships.
//!
//! ## Why
//! 
//! Traditional dependency graphs are optimized for visualization but lack efficient
//! querying capabilities. This hash tree provides enterprise-grade dependency analysis
//! capabilities including affected package detection, circular dependency analysis,
//! and dependency path finding.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::context::dependency_source::DependencySource;

/// A structured, queryable model for dependency graphs
///
/// This hash tree serves as a queryable data structure for dependency analysis,
/// providing efficient access to dependency relationships, dependent detection,
/// and impact analysis.
///
/// # Examples
///
/// ```
/// use sublime_package_tools::graph::DependencyHashTree;
/// use sublime_package_tools::core::dependency::DependencySource;
/// 
/// let mut tree = DependencyHashTree::new();
/// 
/// // Add packages to the tree
/// tree.add_package("app", "1.0.0", PackageLocation::Internal, vec![
///     DependencyReference::new("utils", DependencySource::Workspace { 
///         name: "utils".to_string(), 
///         constraint: WorkspaceConstraint::Any 
///     })
/// ]);
/// 
/// // Query dependents
/// let dependents = tree.find_dependents("utils");
/// assert_eq!(dependents.len(), 1);
/// assert_eq!(dependents[0].name, "app");
/// ```
#[derive(Debug, Clone)]
pub struct DependencyHashTree {
    /// All packages in the dependency tree
    pub packages: HashMap<String, PackageNode>,
    /// Forward dependency graph: package name -> list of dependencies
    pub dependency_graph: HashMap<String, Vec<String>>,
    /// Reverse dependency graph: package name -> list of dependents
    pub dependent_graph: HashMap<String, Vec<String>>,
}

/// Represents a package node in the dependency hash tree
///
/// Contains comprehensive metadata about a package including its dependencies,
/// dependents, and location information.
#[derive(Debug, Clone)]
pub struct PackageNode {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Dependencies of this package
    pub depends_on: Vec<DependencyReference>,
    /// Packages that depend on this package
    pub dependency_of: Vec<String>,
    /// Package location (internal vs external)
    pub location: PackageLocation,
}

/// Reference to a dependency with source information
#[derive(Debug, Clone)]
pub struct DependencyReference {
    /// Name of the dependency
    pub name: String,
    /// Source type and metadata
    pub source: DependencySource,
}

/// Location of a package in the workspace
#[derive(Debug, Clone, PartialEq)]
pub enum PackageLocation {
    /// Package is internal to the workspace
    Internal,
    /// Package is external (from registry, git, etc.)
    External,
}

/// Represents a circular dependency in the dependency graph
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// The dependency path that forms the cycle
    pub path: Vec<String>,
    /// Type of circular dependency
    pub cycle_type: CircularDependencyType,
    /// Severity level of the cycle
    pub severity: CycleSeverity,
}

/// Type of circular dependency based on dependency categories
#[derive(Debug, Clone, PartialEq)]
pub enum CircularDependencyType {
    /// Cycle involves only development dependencies
    DevDependencies,
    /// Cycle involves optional dependencies
    OptionalDependencies,
    /// Cycle involves production dependencies
    ProductionDependencies,
}

/// Severity level of circular dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum CycleSeverity {
    /// Warning level - eligible cycle that doesn't block operations
    Warning,
    /// Error level - problematic cycle that may cause issues
    Error,
}

impl DependencyReference {
    /// Creates a new dependency reference
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the dependency
    /// * `source` - Source information for the dependency
    ///
    /// # Returns
    ///
    /// A new `DependencyReference` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyReference};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let dep_ref = DependencyReference::new(
    ///     "react".to_string(),
    ///     DependencySource::Registry { 
    ///         name: "react".to_string(), 
    ///         version_req: "^18.0.0".parse().unwrap() 
    ///     }
    /// );
    /// ```
    #[must_use]
    pub fn new(name: String, source: DependencySource) -> Self {
        Self { name, source }
    }
}

impl PackageNode {
    /// Creates a new package node
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Package version
    /// * `location` - Package location (internal vs external)
    /// * `depends_on` - List of dependencies
    ///
    /// # Returns
    ///
    /// A new `PackageNode` instance
    #[must_use]
    pub fn new(
        name: String,
        version: String,
        location: PackageLocation,
        depends_on: Vec<DependencyReference>,
    ) -> Self {
        Self {
            name,
            version,
            depends_on,
            dependency_of: Vec::new(),
            location,
        }
    }
}

impl DependencyHashTree {
    /// Creates a new empty dependency hash tree
    ///
    /// # Returns
    ///
    /// A new `DependencyHashTree` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::DependencyHashTree;
    /// 
    /// let tree = DependencyHashTree::new();
    /// assert!(tree.packages.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            dependency_graph: HashMap::new(),
            dependent_graph: HashMap::new(),
        }
    }

    /// Adds a package to the dependency hash tree
    ///
    /// This method adds a package and automatically updates the bidirectional
    /// dependency mappings.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Package version
    /// * `location` - Package location (internal vs external)
    /// * `dependencies` - List of dependencies
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation, DependencyReference};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// tree.add_package(
    ///     "my-app".to_string(),
    ///     "1.0.0".to_string(),
    ///     PackageLocation::Internal,
    ///     vec![]
    /// );
    /// 
    /// assert!(tree.packages.contains_key("my-app"));
    /// ```
    pub fn add_package(
        &mut self,
        name: String,
        version: String,
        location: PackageLocation,
        dependencies: Vec<DependencyReference>,
    ) {
        // Extract dependency names for graph building
        let dep_names: Vec<String> = dependencies.iter().map(|dep| dep.name.clone()).collect();

        // Create package node
        let package_node = PackageNode::new(name.clone(), version, location, dependencies);

        // Add to packages map
        self.packages.insert(name.clone(), package_node);

        // Update forward dependency graph
        self.dependency_graph.insert(name.clone(), dep_names.clone());

        // Update reverse dependency graph
        for dep_name in dep_names {
            self.dependent_graph
                .entry(dep_name.clone())
                .or_default()
                .push(name.clone());

            // Update the dependency_of field in the dependent package
            if let Some(dep_package) = self.packages.get_mut(&dep_name) {
                if !dep_package.dependency_of.contains(&name) {
                    dep_package.dependency_of.push(name.clone());
                }
            }
        }
    }

    /// Finds all packages that depend on the specified package
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// A vector of package nodes that depend on the specified package
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation, DependencyReference};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// 
    /// // Add a dependency relationship: app -> utils
    /// tree.add_package("utils".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![]);
    /// tree.add_package("app".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("utils".to_string(), DependencySource::Registry { 
    ///         name: "utils".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// 
    /// let dependents = tree.find_dependents("utils");
    /// assert_eq!(dependents.len(), 1);
    /// assert_eq!(dependents[0].name, "app");
    /// ```
    #[must_use]
    pub fn find_dependents(&self, package_name: &str) -> Vec<&PackageNode> {
        self.dependent_graph
            .get(package_name)
            .map_or_else(Vec::new, |dependents| {
                dependents
                    .iter()
                    .filter_map(|dep_name| self.packages.get(dep_name))
                    .collect()
            })
    }

    /// Finds a dependency path between two packages
    ///
    /// Uses breadth-first search to find the shortest dependency path from
    /// the source package to the target package.
    ///
    /// # Arguments
    ///
    /// * `from` - Source package name
    /// * `to` - Target package name
    ///
    /// # Returns
    ///
    /// `Some(path)` if a path exists, `None` otherwise. The path includes
    /// both the source and target packages.
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation, DependencyReference};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// 
    /// // Add dependency chain: app -> middleware -> utils
    /// tree.add_package("utils".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![]);
    /// tree.add_package("middleware".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("utils".to_string(), DependencySource::Registry { 
    ///         name: "utils".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// tree.add_package("app".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("middleware".to_string(), DependencySource::Registry { 
    ///         name: "middleware".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// 
    /// let path = tree.find_dependency_path("app", "utils");
    /// assert_eq!(path, Some(vec!["app".to_string(), "middleware".to_string(), "utils".to_string()]));
    /// ```
    #[must_use]
    pub fn find_dependency_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(dependencies) = self.dependency_graph.get(&current) {
                for dep in dependencies {
                    if dep == to {
                        // Found the target, reconstruct path
                        let mut path = Vec::new();
                        let mut node = current.clone();
                        path.push(to.to_string());

                        while let Some(p) = parent.get(&node) {
                            path.push(node.clone());
                            node = p.clone();
                        }
                        path.push(from.to_string());
                        path.reverse();
                        return Some(path);
                    }

                    if !visited.contains(dep) {
                        visited.insert(dep.clone());
                        parent.insert(dep.clone(), current.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        None
    }

    /// Finds all packages affected by changes to the specified packages
    ///
    /// This method performs impact analysis to determine which packages would
    /// be affected if the specified packages were modified. It uses dependency
    /// graph traversal to find all transitive dependents.
    ///
    /// # Arguments
    ///
    /// * `changed_packages` - Names of packages that have changed
    ///
    /// # Returns
    ///
    /// A vector of package names that would be affected by the changes
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation, DependencyReference};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// 
    /// // Create dependency chain: app -> middleware -> utils
    /// tree.add_package("utils".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![]);
    /// tree.add_package("middleware".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("utils".to_string(), DependencySource::Registry { 
    ///         name: "utils".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// tree.add_package("app".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("middleware".to_string(), DependencySource::Registry { 
    ///         name: "middleware".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// 
    /// let affected = tree.affected_by_change(&["utils".to_string()]);
    /// assert!(affected.contains(&"middleware".to_string()));
    /// assert!(affected.contains(&"app".to_string()));
    /// ```
    #[must_use]
    pub fn affected_by_change(&self, changed_packages: &[String]) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        // Initialize queue with changed packages
        for package in changed_packages {
            queue.push_back(package.clone());
        }

        // Traverse dependents
        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = self.dependent_graph.get(&current) {
                for dependent in dependents {
                    if !affected.contains(dependent) {
                        affected.insert(dependent.clone());
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        affected.into_iter().collect()
    }

    /// Detects circular dependencies in the dependency graph
    ///
    /// This method analyzes the dependency graph to identify circular dependencies
    /// and categorizes them by type and severity. Different types of cycles
    /// (development, optional, production) are handled with appropriate severity levels.
    ///
    /// # Returns
    ///
    /// A vector of `CircularDependency` instances representing detected cycles
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation, DependencyReference, CycleSeverity};
    /// use sublime_package_tools::core::dependency::DependencySource;
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// 
    /// // Create a circular dependency: pkg-a -> pkg-b -> pkg-a
    /// tree.add_package("pkg-a".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("pkg-b".to_string(), DependencySource::Registry { 
    ///         name: "pkg-b".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// tree.add_package("pkg-b".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![
    ///     DependencyReference::new("pkg-a".to_string(), DependencySource::Registry { 
    ///         name: "pkg-a".to_string(), 
    ///         version_req: "^1.0.0".parse().unwrap() 
    ///     })
    /// ]);
    /// 
    /// let cycles = tree.detect_circular_deps();
    /// assert!(!cycles.is_empty());
    /// ```
    #[must_use]
    pub fn detect_circular_deps(&self) -> Vec<CircularDependency> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for package_name in self.packages.keys() {
            if !visited.contains(package_name) {
                self.detect_cycles_dfs(
                    package_name,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// Depth-first search for cycle detection
    ///
    /// This is a helper method for `detect_circular_deps` that performs
    /// depth-first traversal to identify cycles.
    fn detect_cycles_dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<CircularDependency>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(dependencies) = self.dependency_graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.detect_cycles_dfs(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle - extract the cycle path
                    if let Some(cycle_start) = path.iter().position(|p| p == dep) {
                        let cycle_path = path[cycle_start..].to_vec();
                        let mut cycle_path_with_end = cycle_path;
                        cycle_path_with_end.push(dep.clone());

                        // Determine cycle type and severity
                        let (cycle_type, severity) = self.analyze_cycle_type(&cycle_path_with_end);

                        cycles.push(CircularDependency {
                            path: cycle_path_with_end,
                            cycle_type,
                            severity,
                        });
                    }
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Analyzes the type and severity of a circular dependency
    ///
    /// This method examines the dependency types in a cycle to determine
    /// if it involves development dependencies, optional dependencies, or
    /// production dependencies, and assigns appropriate severity.
    fn analyze_cycle_type(&self, _cycle_path: &[String]) -> (CircularDependencyType, CycleSeverity) {
        // For now, assume all cycles are production dependencies with error severity
        // In a real implementation, this would analyze the actual dependency types
        // from the package.json files or dependency metadata
        (CircularDependencyType::ProductionDependencies, CycleSeverity::Error)
    }

    /// Renders the dependency tree as ASCII art
    ///
    /// This method generates an ASCII representation of the dependency tree
    /// for visualization purposes. The ASCII output is generated from the
    /// structured hash tree model.
    ///
    /// # Returns
    ///
    /// A string containing the ASCII representation of the dependency tree
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation};
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// tree.add_package("app".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![]);
    /// 
    /// let ascii = tree.render_ascii_tree();
    /// assert!(ascii.contains("app"));
    /// ```
    #[must_use]
    pub fn render_ascii_tree(&self) -> String {
        let mut output = String::new();
        output.push_str("Dependency Tree:\n");

        for (name, package) in &self.packages {
            output.push_str(&format!("├── {} v{}\n", name, package.version));

            for dep in &package.depends_on {
                output.push_str(&format!("│   └── {} ({})\n", dep.name, self.format_dependency_source(&dep.source)));
            }
        }

        output
    }

    /// Renders the dependency tree as DOT graph format
    ///
    /// This method generates a DOT format representation of the dependency tree
    /// suitable for rendering with Graphviz tools. The DOT output is generated
    /// from the structured hash tree model.
    ///
    /// # Returns
    ///
    /// A string containing the DOT representation of the dependency tree
    ///
    /// # Examples
    ///
    /// ```
    /// use sublime_package_tools::graph::{DependencyHashTree, PackageLocation};
    /// 
    /// let mut tree = DependencyHashTree::new();
    /// tree.add_package("app".to_string(), "1.0.0".to_string(), PackageLocation::Internal, vec![]);
    /// 
    /// let dot = tree.render_dot_graph();
    /// assert!(dot.starts_with("digraph"));
    /// ```
    #[must_use]
    pub fn render_dot_graph(&self) -> String {
        let mut output = String::new();
        output.push_str("digraph DependencyGraph {\n");
        output.push_str("  rankdir=TB;\n");
        output.push_str("  node [shape=box, style=rounded];\n");

        // Add nodes
        for (name, package) in &self.packages {
            let color = match package.location {
                PackageLocation::Internal => "lightblue",
                PackageLocation::External => "lightgray",
            };
            output.push_str(&format!(
                "  \"{}\" [label=\"{}\\nv{}\", fillcolor={}, style=\"rounded,filled\"];\n",
                name, name, package.version, color
            ));
        }

        // Add edges
        for (name, dependencies) in &self.dependency_graph {
            for dep in dependencies {
                output.push_str(&format!("  \"{}\" -> \"{}\";\n", name, dep));
            }
        }

        output.push_str("}\n");
        output
    }

    /// Formats a dependency source for display
    fn format_dependency_source(&self, source: &DependencySource) -> String {
        match source {
            DependencySource::Registry { version_req, .. } => format!("registry: {version_req}"),
            DependencySource::Workspace { constraint, .. } => format!("workspace: {constraint:?}"),
            DependencySource::File { path, .. } => format!("file: {}", path.display()),
            DependencySource::Git { repo, reference, .. } => {
                format!("git: {} @ {reference:?}", repo)
            }
            DependencySource::GitHub { user, repo, reference, .. } => {
                let ref_str = reference.as_deref().unwrap_or("HEAD");
                format!("github: {}/{} @ {}", user, repo, ref_str)
            }
            DependencySource::Url { url, .. } => format!("url: {url}"),
            _ => format!("{source:?}"),
        }
    }
}

impl Default for DependencyHashTree {
    fn default() -> Self {
        Self::new()
    }
}