//! Core workspace representation and functionality.
//!
//! This module provides the `Workspace` struct, which is the central component
//! for managing packages in a monorepo. It supports package discovery,
//! dependency analysis, and workspace-wide operations.
//!
//! The workspace handles package dependencies, circular dependency detection,
//! and provides utilities for traversing the package graph.

use crate::{DiscoveryOptions, ValidationOptions, WorkspaceConfig, WorkspaceError, WorkspaceGraph};
use glob::glob;
use globset::{Glob, GlobSetBuilder};
use pathdiff::diff_paths;
use petgraph::graph::DiGraph;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyGraph, Package, PackageInfo, ValidationReport};
use sublime_standard_tools::CorePackageManager;

/// Result of topologically sorting workspace packages.
///
/// This structure separates packages that can be topologically sorted
/// (those without circular dependencies) from those involved in cycles.
///
/// # Examples
///
/// ```no_run
/// # use sublime_monorepo_tools::{Workspace, DiscoveryOptions};
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let workspace = Workspace::new(std::path::PathBuf::from("."), Default::default(), None)?;
/// // Get sorted packages with cycle information
/// let sorted = workspace.get_sorted_packages_with_circulars();
///
/// // Work with packages that have no circular dependencies
/// for pkg in sorted.sorted {
///     println!("Regular package: {}", pkg.borrow().package.borrow().name());
/// }
///
/// // Work with cycle groups
/// for (i, cycle) in sorted.circular.iter().enumerate() {
///     println!("Cycle group {}:", i+1);
///     for pkg in cycle {
///         println!("  - {}", pkg.borrow().package.borrow().name());
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SortedPackages {
    /// Packages that can be topologically sorted (no cycles)
    pub sorted: Vec<Rc<RefCell<PackageInfo>>>,
    /// Groups of packages involved in circular dependencies
    pub circular: Vec<Vec<Rc<RefCell<PackageInfo>>>>,
}

/// Complete workspace representation.
///
/// The `Workspace` struct represents a monorepo workspace, containing packages
/// and their relationships. It provides methods for discovering packages,
/// analyzing dependencies, and performing operations on the workspace.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use sublime_monorepo_tools::{Workspace, WorkspaceConfig, DiscoveryOptions};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a workspace
/// let config = WorkspaceConfig::new(PathBuf::from("."));
/// let mut workspace = Workspace::new(PathBuf::from("."), config, None)?;
///
/// // Discover packages
/// workspace.discover_packages_with_options(&DiscoveryOptions::default())?;
///
/// // Work with packages
/// for pkg_info in workspace.sorted_packages() {
///     let name = pkg_info.borrow().package.borrow().name();
///     println!("Package: {}", name);
///
///     // Get dependencies of this package
///     let deps = workspace.dependencies_of(name);
///     println!("Dependencies: {}", deps.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Workspace {
    /// Root path of the workspace
    root_path: PathBuf,
    /// Information about all packages in the workspace
    package_infos: Vec<Rc<RefCell<PackageInfo>>>,
    /// Package manager used in the workspace
    package_manager: Option<CorePackageManager>,
    /// Git repository
    git_repo: Option<Rc<Repo>>,
    /// Configuration for the workspace
    config: WorkspaceConfig,
}

// Can we implement std::fmt::Debug to Workspace struct?
impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workspace")
            .field("root_path", &self.root_path)
            .field("package_infos", &self.package_infos)
            .field("package_manager", &self.package_manager)
            .field("git_repo", &self.git_repo)
            .field("config", &self.config)
            .finish()
    }
}

impl Workspace {
    /// Creates a new workspace.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root path of the workspace
    /// * `config` - Configuration for the workspace
    /// * `git_repo` - Optional Git repository for this workspace
    ///
    /// # Returns
    ///
    /// A new workspace instance.
    ///
    /// # Errors
    ///
    /// Returns an error if workspace creation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::{Workspace, WorkspaceConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a basic workspace
    /// let config = WorkspaceConfig::new(PathBuf::from("."));
    /// let workspace = Workspace::new(PathBuf::from("."), config, None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        root_path: PathBuf,
        config: WorkspaceConfig,
        git_repo: Option<Repo>,
    ) -> Result<Self, WorkspaceError> {
        let package_manager = config.package_manager.as_deref().and_then(|pm| pm.try_into().ok());

        // Convert git_repo to Rc if present
        let git_repo = git_repo.map(Rc::new);

        Ok(Self { root_path, package_infos: Vec::new(), package_manager, git_repo, config })
    }

    /// Returns information about circular dependencies in the workspace.
    ///
    /// This method provides explicit information about dependency cycles
    /// without treating them as errors. It returns groups of packages that
    /// form circular dependencies.
    ///
    /// # Returns
    ///
    /// A vector of cycle groups, where each group is a vector of package names.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{Workspace, DiscoveryOptions};
    /// # fn example(workspace: &Workspace) {
    /// // Get circular dependencies
    /// let cycles = workspace.get_circular_dependencies();
    ///
    /// // Print cycles
    /// if cycles.is_empty() {
    ///     println!("No circular dependencies found");
    /// } else {
    ///     println!("Found {} cycle groups:", cycles.len());
    ///     for (i, group) in cycles.iter().enumerate() {
    ///         println!("Cycle {}: {}", i+1, group.join(" → "));
    ///     }
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_circular_dependencies(&self) -> Vec<Vec<String>> {
        self.get_sorted_packages_with_circulars()
            .circular
            .iter()
            .map(|group| {
                group.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect()
            })
            .collect()
    }

    /// Checks if a package is part of a circular dependency.
    ///
    /// Returns `true` if the package is involved in a circular dependency.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to check
    ///
    /// # Returns
    ///
    /// `true` if the package is in a dependency cycle, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{Workspace, DiscoveryOptions};
    /// # fn example(workspace: &Workspace) {
    /// // Check if a package is in a cycle
    /// if workspace.is_in_cycle("my-package") {
    ///     println!("Package 'my-package' is part of a circular dependency");
    ///
    ///     // Get the cycle group
    ///     if let Some(cycle) = workspace.get_cycle_for_package("my-package") {
    ///         println!("Cycle: {}", cycle.join(" → "));
    ///     }
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_in_cycle(&self, package_name: &str) -> bool {
        let circles = self.get_sorted_packages_with_circulars();

        circles.circular.iter().any(|group| {
            group.iter().any(|pkg| pkg.borrow().package.borrow().name() == package_name)
        })
    }

    /// Gets the cycle group containing a package, if any.
    ///
    /// Returns the group of packages forming a cycle that includes the specified
    /// package, or `None` if the package is not part of a cycle.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find the cycle for
    ///
    /// # Returns
    ///
    /// An optional vector of package names forming a cycle.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// if let Some(cycle) = workspace.get_cycle_for_package("ui-components") {
    ///     println!("Package is in a cycle with: {}", cycle.join(", "));
    /// } else {
    ///     println!("Package is not in a dependency cycle");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_cycle_for_package(&self, package_name: &str) -> Option<Vec<String>> {
        let circles = self.get_sorted_packages_with_circulars();

        for group in &circles.circular {
            if group.iter().any(|pkg| pkg.borrow().package.borrow().name() == package_name) {
                return Some(
                    group.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect(),
                );
            }
        }

        None
    }

    /// Returns a mapping of packages to their cycle groups.
    ///
    /// The returned HashMap maps package names to cycle group indices.
    /// Packages not in cycles are not included in the map.
    ///
    /// # Returns
    ///
    /// A HashMap mapping package names to their cycle group index.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Get cycle membership
    /// let membership = workspace.get_cycle_membership();
    ///
    /// // Check if a package is in a cycle
    /// if let Some(group_index) = membership.get("ui-components") {
    ///     println!("Package is in cycle group {}", group_index + 1);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_cycle_membership(&self) -> HashMap<String, usize> {
        let circles = self.get_sorted_packages_with_circulars();
        let mut membership = HashMap::new();

        for (i, group) in circles.circular.iter().enumerate() {
            for pkg in group {
                let name = pkg.borrow().package.borrow().name().to_string();
                membership.insert(name, i);
            }
        }

        membership
    }

    /// Discovers packages with custom options.
    ///
    /// Scans the workspace for packages according to the provided options.
    ///
    /// # Arguments
    ///
    /// * `options` - Options controlling package discovery
    ///
    /// # Returns
    ///
    /// A reference to self for method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if package discovery fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use sublime_monorepo_tools::{DiscoveryOptions, Workspace, WorkspaceConfig};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create workspace
    /// let config = WorkspaceConfig::new(PathBuf::from("."));
    /// let mut workspace = Workspace::new(PathBuf::from("."), config, None)?;
    ///
    /// // Configure discovery options
    /// let options = DiscoveryOptions::new()
    ///     .include_patterns(vec!["packages/*/package.json"])
    ///     .exclude_patterns(vec!["**/test/**"])
    ///     .include_private(false);
    ///
    /// // Discover packages
    /// workspace.discover_packages_with_options(&options)?;
    ///
    /// println!("Found {} packages", workspace.sorted_packages().len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover_packages_with_options(
        &mut self,
        options: &DiscoveryOptions,
    ) -> Result<&Self, WorkspaceError> {
        let mut package_infos = Vec::new();

        // Compile exclude patterns into a globset
        let mut builder = GlobSetBuilder::new();
        for pattern in &options.exclude_patterns {
            let glob = Glob::new(pattern).map_err(|e| {
                WorkspaceError::InvalidConfiguration(format!(
                    "Invalid exclude pattern '{pattern}': {e}"
                ))
            })?;
            builder.add(glob);
        }
        let exclude_set = builder.build().map_err(|e| {
            WorkspaceError::InvalidConfiguration(format!("Failed to build exclude set: {e}"))
        })?;

        // Use package patterns from config or default to include patterns from options
        let patterns = if self.config.packages.is_empty() {
            &options.include_patterns
        } else {
            // Check if we're using the default options and have workspaces in package.json
            // If so, prefer options.include_patterns which will match more patterns
            if options.include_patterns.contains(&"**/package.json".to_string())
                && !options.include_patterns.is_empty()
            {
                &options.include_patterns
            } else {
                &self.config.packages
            }
        };

        for pattern in patterns {
            let glob_pattern = format!("{}/{}", self.root_path.display(), pattern);

            for entry in glob(&glob_pattern).map_err(|e| {
                WorkspaceError::InvalidConfiguration(format!("Invalid glob pattern: {e}"))
            })? {
                let path = entry.map_err(|e| {
                    WorkspaceError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Glob error: {e}"),
                    ))
                })?;

                // Skip node_modules (hardcoded exclusion)
                if path.to_string_lossy().contains("node_modules") {
                    continue;
                }

                // Get the path relative to workspace root for matching against exclude patterns
                let relative_path =
                    diff_paths(&path, &self.root_path).unwrap_or_else(|| path.clone());
                let relative_str = relative_path.to_string_lossy();

                // Check if the path matches any exclude pattern
                if exclude_set.is_match(relative_str.as_ref()) {
                    continue;
                }

                // Calculate the directory depth from the root path and apply max_depth filter
                let depth = relative_path.components().count();
                if depth > options.max_depth {
                    // Skip if beyond max_depth
                    continue;
                }

                // If it's a package.json file, process it
                if path.file_name().map_or(false, |n| n == "package.json") {
                    if let Some(package_info) = self.process_package_json(&path)? {
                        // Check if this is a private package that should be excluded
                        if !options.include_private {
                            // Parse the package.json to check for private flag
                            let pkg_json: serde_json::Value = serde_json::from_str(
                                &std::fs::read_to_string(&path)?,
                            )
                            .map_err(|e| WorkspaceError::ManifestParseError {
                                path: path.clone(),
                                error: e,
                            })?;

                            // Skip private packages if include_private is false
                            if pkg_json
                                .get("private")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false)
                            {
                                continue;
                            }
                        }

                        package_infos.push(package_info);
                    }
                }
            }
        }

        self.package_infos = package_infos;
        Ok(self)
    }

    /// Analyzes workspace dependencies.
    ///
    /// Analyzes the dependency graph of the workspace, identifying cycles,
    /// external dependencies, and version conflicts.
    ///
    /// # Returns
    ///
    /// A `WorkspaceGraph` containing analysis results.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// // Analyze dependencies
    /// let analysis = workspace.analyze_dependencies()?;
    ///
    /// // Check results
    /// if analysis.cycles_detected {
    ///     println!("Circular dependencies detected!");
    ///     for cycle in &analysis.cycles {
    ///         println!("Cycle: {}", cycle.join(" → "));
    ///     }
    /// }
    ///
    /// // Check for external dependencies
    /// if !analysis.external_dependencies.is_empty() {
    ///     println!("External dependencies: {}", analysis.external_dependencies.join(", "));
    /// }
    ///
    /// // Check for version conflicts
    /// if !analysis.version_conflicts.is_empty() {
    ///     println!("Version conflicts found:");
    ///     for (pkg, versions) in &analysis.version_conflicts {
    ///         println!("  {}: {}", pkg, versions.join(", "));
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn analyze_dependencies(&self) -> Result<WorkspaceGraph, WorkspaceError> {
        // Extract owned packages
        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(packages.as_slice());

        // Get cycles directly from the graph instead of checking for errors
        let cycles_detected = graph.has_cycles();

        // Get any cycle information
        let cycles = graph.get_cycle_strings();

        // Get external dependencies (previously called missing dependencies)
        let external_dependencies = graph.find_external_dependencies();

        // Get version conflicts
        let version_conflicts = graph.find_version_conflicts_for_package();

        // Run validation
        let validation = match graph.validate_package_dependencies() {
            Ok(report) => Some(report),
            Err(_) => None,
        };

        Ok(WorkspaceGraph {
            cycles_detected,
            cycles,
            external_dependencies,
            version_conflicts,
            validation,
        })
    }

    /// Gets a package by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the package to retrieve
    ///
    /// # Returns
    ///
    /// An optional reference to the package information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Get a package
    /// if let Some(pkg) = workspace.get_package("ui-components") {
    ///     let info = pkg.borrow();
    ///     let package = info.package.borrow();
    ///     println!("Found package: {} v{}", package.name(), package.version_str());
    /// } else {
    ///     println!("Package not found");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<Rc<RefCell<PackageInfo>>> {
        self.package_infos
            .iter()
            .find(|p| p.borrow().package.borrow().name() == name)
            .map(Rc::clone) // Fixed: Using Rc::clone instead of .clone()
    }

    /// Gets packages in topological order, excluding the root workspace package.
    ///
    /// Returns packages sorted so that dependencies come before the packages that depend on them.
    /// For example, if package A depends on package B, then B will appear before A in the result.
    ///
    /// The root package of the workspace (located directly at the workspace root path)
    /// is automatically excluded from the result.
    ///
    /// # Returns
    ///
    /// A vector of package references in dependency order.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Get sorted packages
    /// let packages = workspace.sorted_packages();
    ///
    /// // Process in dependency order (dependencies before dependents)
    /// for pkg in packages {
    ///     let info = pkg.borrow();
    ///     let package = info.package.borrow();
    ///     println!("Package: {}", package.name());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn sorted_packages(&self) -> Vec<Rc<RefCell<PackageInfo>>> {
        let result = self.get_sorted_packages_with_circulars();

        // Combine sorted packages and packages in circular dependencies
        let mut all_packages = result.sorted;

        // Add all packages from circular dependency groups
        for group in result.circular {
            all_packages.extend(group);
        }

        all_packages
    }

    /// Gets packages sorted by dependencies with cycle information.
    ///
    /// Returns detailed information about the package sorting, separating
    /// packages that can be topologically sorted from those involved in
    /// circular dependencies.
    ///
    /// # Returns
    ///
    /// A `SortedPackages` struct containing sorted packages and cycle groups.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Get sorted packages with cycle info
    /// let result = workspace.get_sorted_packages_with_circulars();
    ///
    /// // Work with sorted packages (no circular dependencies)
    /// println!("Packages with no circular dependencies: {}", result.sorted.len());
    ///
    /// // Work with cycle groups
    /// println!("Found {} cycle groups", result.circular.len());
    /// for group in result.circular {
    ///     let names: Vec<String> = group
    ///         .iter()
    ///         .map(|p| p.borrow().package.borrow().name().to_string())
    ///         .collect();
    ///     println!("Cycle: {}", names.join(" → "));
    /// }
    /// # }
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::items_after_statements)]
    pub fn get_sorted_packages_with_circulars(&self) -> SortedPackages {
        // Extract non-root packages
        let non_root_packages: Vec<Rc<RefCell<PackageInfo>>> = self
            .package_infos
            .iter()
            .filter_map(|info| {
                let pkg_info = info.borrow();
                let pkg_path_str = &pkg_info.package_path;

                // Convert to absolute path
                let pkg_path = if Path::new(pkg_path_str).is_absolute() {
                    PathBuf::from(pkg_path_str)
                } else {
                    self.root_path.join(pkg_path_str)
                };

                // Include if within a subdirectory of the root
                if let Some(rel_path) = pathdiff::diff_paths(&pkg_path, &self.root_path) {
                    let components = rel_path.components().count();
                    if components > 0 {
                        return Some(Rc::clone(info));
                    }
                }
                None
            })
            .collect();

        // Nothing to sort if we have 0 or 1 packages
        if non_root_packages.is_empty() {
            return SortedPackages { sorted: Vec::new(), circular: Vec::new() };
        } else if non_root_packages.len() == 1 {
            return SortedPackages { sorted: non_root_packages, circular: Vec::new() };
        }

        // Create a package name to package info map for quick lookups
        let package_map: HashMap<String, Rc<RefCell<PackageInfo>>> = non_root_packages
            .iter()
            .map(|p| (p.borrow().package.borrow().name().to_string(), Rc::clone(p)))
            .collect();

        // Build a directed graph for internal dependencies only
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();

        // Add all packages as nodes
        for pkg in &non_root_packages {
            let name = pkg.borrow().package.borrow().name().to_string();
            let idx = graph.add_node(name.clone());
            node_indices.insert(name, idx);
        }

        // Add internal dependencies as edges
        for pkg in &non_root_packages {
            let from_name = pkg.borrow().package.borrow().name().to_string();
            let from_idx = node_indices[&from_name];

            // Get all dependencies of this package
            let pkg_borrow = pkg.borrow();
            let package_borrow = pkg_borrow.package.borrow();
            let deps = package_borrow.dependencies();

            for dep in deps {
                let dep_name = dep.borrow().name().to_string();

                // Only add edge if dependency is an internal package
                if let Some(to_idx) = node_indices.get(&dep_name) {
                    graph.add_edge(*to_idx, from_idx, ()); // Dependency points to dependent
                }
                // External dependencies are ignored for cycle detection
            }
        }

        // Use Kosaraju's algorithm to find strongly connected components (cycles)
        let mut sccs = Vec::new();
        let scc_result = petgraph::algo::kosaraju_scc(&graph);

        for component in scc_result {
            // Only components with more than one node represent cycles
            if component.len() > 1 {
                let mut cycle_group = Vec::new();
                for &node_idx in &component {
                    let pkg_name = &graph[node_idx];
                    if let Some(pkg) = package_map.get(pkg_name) {
                        cycle_group.push(Rc::clone(pkg));
                    }
                }
                if !cycle_group.is_empty() {
                    sccs.push(cycle_group);
                }
            }
        }

        // Try to perform a topological sort on the graph
        let mut sorted = Vec::new();

        // If we found cycles, we need to exclude those nodes from the sort
        let mut cycle_nodes = HashSet::new();
        for component in &sccs {
            for pkg in component {
                cycle_nodes.insert(pkg.borrow().package.borrow().name().to_string());
            }
        }

        // Create a subgraph without cycle nodes
        let mut acyclic_graph = graph.clone();
        let nodes_to_remove: Vec<_> =
            graph.node_indices().filter(|&idx| cycle_nodes.contains(&graph[idx])).collect();

        // Remove nodes in reverse order to maintain indices
        for idx in nodes_to_remove.into_iter().rev() {
            acyclic_graph.remove_node(idx);
        }

        // Perform toposort on acyclic portion
        if let Ok(ordered) = petgraph::algo::toposort(&acyclic_graph, None) {
            for idx in ordered {
                let pkg_name = &acyclic_graph[idx];
                if let Some(pkg) = package_map.get(pkg_name) {
                    sorted.push(Rc::clone(pkg));
                }
            }
        }

        // Add remaining non-cycle packages
        for pkg in &non_root_packages {
            let name = pkg.borrow().package.borrow().name().to_string();
            if !cycle_nodes.contains(&name)
                && !sorted.iter().any(|p| p.borrow().package.borrow().name() == name)
            {
                sorted.push(Rc::clone(pkg));
            }
        }

        // Return both sorted packages and cycle groups
        SortedPackages { sorted, circular: sccs }
    }

    /// Gets packages affected by changes in the specified packages.
    ///
    /// Determines which packages are affected by changes to the specified packages,
    /// by traversing the dependency graph to find all dependents.
    ///
    /// # Arguments
    ///
    /// * `changed_packages` - Names of packages that have changed
    /// * `check_circular` - Whether to check for circular dependencies (returns empty vec if found)
    ///
    /// # Returns
    ///
    /// A vector of package references that are affected by the changes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Find packages affected by changes to "core" and "utils"
    /// let affected = workspace.affected_packages(&["core", "utils"], None);
    ///
    /// println!("Changes to core and utils affect {} packages", affected.len());
    /// for pkg in affected {
    ///     let name = pkg.borrow().package.borrow().name();
    ///     println!("Affected: {}", name);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn affected_packages(
        &self,
        changed_packages: &[&str],
        check_circular: Option<bool>,
    ) -> Vec<Rc<RefCell<PackageInfo>>> {
        let check = check_circular.unwrap_or(true); // Default to true for backward compatibility

        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let mut graph = DependencyGraph::from(packages.as_slice());

        // Updated to use has_cycles() instead of checking for errors
        if check && graph.has_cycles() {
            return Vec::new();
        }

        // Start with the directly changed packages
        let mut affected =
            changed_packages.iter().filter_map(|&name| self.get_package(name)).collect::<Vec<_>>();

        // Track packages we've already processed to avoid duplicates
        let mut processed_packages = HashSet::new();
        for pkg in &affected {
            processed_packages.insert(pkg.borrow().package.borrow().name().to_string());
        }

        // Add dependents of changed packages (recursively)
        let mut pending = changed_packages.iter().map(|&s| s.to_string()).collect::<Vec<_>>();

        while let Some(package_name) = pending.pop() {
            if let Ok(dependents) = graph.get_dependents(&package_name) {
                for dep_name in dependents {
                    let dep_name_str = dep_name.to_string();
                    if !processed_packages.contains(&dep_name_str) {
                        if let Some(pkg) = self.get_package(&dep_name_str) {
                            affected.push(Rc::clone(&pkg));
                            processed_packages.insert(dep_name_str.clone());
                            pending.push(dep_name_str);
                        }
                    }
                }
            }
        }

        affected
    }

    /// Gets packages that depend on a specific package.
    ///
    /// Finds all packages that directly depend on the specified package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// A vector of package references that depend on the specified package.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Find packages that depend on "core"
    /// let dependents = workspace.dependents_of("core");
    ///
    /// println!("Packages depending on core: {}", dependents.len());
    /// for pkg in dependents {
    ///     let name = pkg.borrow().package.borrow().name();
    ///     println!("  - {}", name);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn dependents_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>> {
        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let mut graph = DependencyGraph::from(packages.as_slice());

        // Always get dependents, even if cycles exist
        match graph.get_dependents(&package_name.to_string()) {
            Ok(dependents) => dependents.iter().filter_map(|name| self.get_package(name)).collect(),
            Err(_) => Vec::new(), // Only for package not found errors
        }
    }

    /// Gets direct dependencies of a package.
    ///
    /// Finds all packages that the specified package directly depends on.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependencies for
    ///
    /// # Returns
    ///
    /// A vector of package references that the specified package depends on.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// // Find dependencies of "ui-components"
    /// let dependencies = workspace.dependencies_of("ui-components");
    ///
    /// println!("Dependencies of ui-components: {}", dependencies.len());
    /// for pkg in dependencies {
    ///     let name = pkg.borrow().package.borrow().name();
    ///     println!("  - {}", name);
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn dependencies_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>> {
        // Find the package - using let...else pattern
        let Some(package) = self.get_package(package_name) else {
            // Fixed: Using let...else
            return Vec::new();
        };

        // Get its dependencies
        let package_borrowed = package.borrow();
        let pkg_borrowed = package_borrowed.package.borrow();
        let dependencies = pkg_borrowed.dependencies();

        // Map dependencies to package info references
        dependencies
            .iter()
            .filter_map(|dep| {
                let dep_borrowed = dep.borrow();
                self.get_package(dep_borrowed.name())
            })
            .collect()
    }

    /// Gets workspace root path.
    ///
    /// # Returns
    ///
    /// The root path of the workspace.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// let root = workspace.root_path();
    /// println!("Workspace root: {}", root.display());
    /// # }
    /// ```
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Gets Git repository reference.
    ///
    /// # Returns
    ///
    /// An optional reference to the Git repository.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// if let Some(repo) = workspace.git_repo() {
    ///     // Use the Git repository
    ///     if let Ok(sha) = repo.get_current_sha() {
    ///         println!("Current Git SHA: {}", sha);
    ///     }
    /// } else {
    ///     println!("No Git repository available");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn git_repo(&self) -> Option<&Repo> {
        self.git_repo.as_deref()
    }

    /// Gets package manager.
    ///
    /// # Returns
    ///
    /// An optional reference to the package manager.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// if let Some(pm) = workspace.package_manager() {
    ///     println!("Package manager: {:?}", pm);
    /// } else {
    ///     println!("No package manager detected");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn package_manager(&self) -> &Option<CorePackageManager> {
        &self.package_manager
    }

    /// Checks if the workspace is empty.
    ///
    /// # Returns
    ///
    /// `true` if the workspace has no packages, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) {
    /// if workspace.is_empty() {
    ///     println!("Workspace has no packages");
    /// } else {
    ///     println!("Workspace has {} packages", workspace.sorted_packages().len());
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.package_infos.is_empty()
    }

    /// Writes package changes to disk.
    ///
    /// Saves any changes made to packages back to their package.json files.
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if writing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if writing changes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// // Make changes to packages
    /// // ...
    ///
    /// // Write changes to disk
    /// workspace.write_changes()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_changes(&self) -> Result<(), WorkspaceError> {
        for package_info in &self.package_infos {
            package_info.borrow().write_package_json().map_err(WorkspaceError::PackageError)?;
        }
        Ok(())
    }

    /// Validates workspace consistency.
    ///
    /// By default, unresolved dependencies are treated as issues.
    ///
    /// # Returns
    ///
    /// A validation report.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::Workspace;
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// // Validate workspace
    /// let report = workspace.validate()?;
    ///
    /// // Check for issues
    /// if report.has_issues() {
    ///     println!("Workspace has validation issues:");
    ///     for issue in report.issues() {
    ///         println!("- {}", issue);
    ///     }
    /// } else {
    ///     println!("Workspace is valid");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate(&self) -> Result<ValidationReport, WorkspaceError> {
        // Extract non-root packages
        let non_root_packages: Vec<Package> = self
            .package_infos
            .iter()
            .filter(|info| {
                // Get the package's path
                let package_path = PathBuf::from(&info.borrow().package_path);

                // Check if the package path is different from the workspace root path
                package_path != self.root_path
            })
            .map(|info| info.borrow().package.borrow().clone())
            .collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(non_root_packages.as_slice());

        // Validate with default options (unresolved deps are issues)
        let report = graph
            .validate_package_dependencies()
            .map_err(WorkspaceError::DependencyResolutionError)?;

        Ok(report)
    }

    /// Validates workspace consistency with custom options.
    ///
    /// # Arguments
    ///
    /// * `options` - Validation options
    ///
    /// # Returns
    ///
    /// A validation report.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sublime_monorepo_tools::{Workspace, ValidationOptions};
    /// # fn example(workspace: &Workspace) -> Result<(), Box<dyn std::error::Error>> {
    /// // Configure validation options
    /// let options = ValidationOptions::new()
    ///     .treat_unresolved_as_external(true)
    ///     .with_internal_dependencies(vec!["core", "shared"]);
    ///
    /// // Validate with custom options
    /// let report = workspace.validate_with_options(&options)?;
    ///
    /// println!("Validation complete: {} issues found", report.issues().len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn validate_with_options(
        &self,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, WorkspaceError> {
        // Extract non-root packages
        let non_root_packages: Vec<Package> = self
            .package_infos
            .iter()
            .filter(|info| {
                // Get the package's path
                let package_path = PathBuf::from(&info.borrow().package_path);

                // Check if the package path is different from the workspace root path
                package_path != self.root_path
            })
            .map(|info| info.borrow().package.borrow().clone())
            .collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(non_root_packages.as_slice());

        // Create validation options for the package tools
        let pkg_options = sublime_package_tools::ValidationOptions::new()
            .treat_unresolved_as_external(options.treat_unresolved_as_external)
            .with_internal_packages(options.internal_dependencies.clone());

        // Use the options during validation
        let report = graph
            .validate_with_options(&pkg_options)
            .map_err(WorkspaceError::DependencyResolutionError)?;

        Ok(report)
    }

    /// Processes a package.json file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the package.json file
    ///
    /// # Returns
    ///
    /// An optional package info reference.
    ///
    /// # Errors
    ///
    /// Returns an error if processing fails.
    fn process_package_json(
        &self,
        path: &Path,
    ) -> Result<Option<Rc<RefCell<PackageInfo>>>, WorkspaceError> {
        // Read package.json
        let content = std::fs::read_to_string(path).map_err(|e| {
            WorkspaceError::ManifestReadError { path: path.to_path_buf(), error: e }
        })?;

        // Parse package.json
        let pkg_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            WorkspaceError::ManifestParseError { path: path.to_path_buf(), error: e }
        })?;

        // Extract name and version
        let Some(name) = pkg_json["name"].as_str() else {
            return Err(WorkspaceError::InvalidConfiguration(format!(
                "Missing name in {}",
                path.display()
            )));
        };

        let Some(version) = pkg_json["version"].as_str() else {
            return Err(WorkspaceError::InvalidConfiguration(format!(
                "Missing version in {}",
                path.display()
            )));
        };

        // Get package path
        let Some(package_path) = path.parent() else {
            return Err(WorkspaceError::InvalidConfiguration(format!(
                "Invalid package path: {}",
                path.display()
            )));
        };
        let package_path = package_path.to_path_buf();

        // Skip the root package.json - don't count it as a package
        if package_path == self.root_path {
            return Ok(None);
        }

        // Get relative path from workspace root
        let Some(relative_path) = diff_paths(&package_path, &self.root_path) else {
            return Err(WorkspaceError::InvalidConfiguration(format!(
                "Cannot determine relative path: {} from {}",
                package_path.display(),
                self.root_path.display()
            )));
        };

        // Extract dependencies from package.json
        let mut dependencies = Vec::new();

        // Process regular dependencies
        if let Some(deps_obj) = pkg_json["dependencies"].as_object() {
            for (dep_name, dep_version) in deps_obj {
                if let Some(dep_version_str) = dep_version.as_str() {
                    // Create a dependency object - propagate any errors
                    let dep = sublime_package_tools::Dependency::new(dep_name, dep_version_str)
                        .map_err(WorkspaceError::VersionError)?;
                    dependencies.push(Rc::new(RefCell::new(dep)));
                } else {
                    // Non-string version is invalid
                    return Err(WorkspaceError::InvalidConfiguration(format!(
                        "Invalid dependency version format for {} in {}",
                        dep_name,
                        path.display()
                    )));
                }
            }
        }

        // Create Package object with dependencies
        let package = Package::new(name, version, Some(dependencies))
            .map_err(WorkspaceError::VersionError)?;

        // Create PackageInfo
        let package_info = PackageInfo::new(
            package,
            path.to_string_lossy().to_string(),
            package_path.to_string_lossy().to_string(),
            relative_path.to_string_lossy().to_string(),
            pkg_json,
        );

        Ok(Some(Rc::new(RefCell::new(package_info))))
    }
}
