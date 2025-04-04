use glob::glob;
use globset::{Glob, GlobSetBuilder};
use pathdiff::diff_paths;
use petgraph::algo::toposort;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_package_tools::{DependencyGraph, Node, Package, PackageInfo, ValidationReport};
use sublime_standard_tools::CorePackageManager;

use crate::{DiscoveryOptions, ValidationOptions, WorkspaceConfig, WorkspaceError, WorkspaceGraph};

#[derive(Debug, Clone)]
pub struct SortedPackages {
    /// Packages that can be topologically sorted (no cycles)
    pub sorted: Vec<Rc<RefCell<PackageInfo>>>,
    /// Groups of packages involved in circular dependencies
    pub circular: Vec<Vec<Rc<RefCell<PackageInfo>>>>,
}

/// Complete workspace representation.
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
    /// # Errors
    /// Returns an error if workspace creation fails.
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

    /// Discovers packages with custom options.
    ///
    /// # Errors
    /// Returns an error if package discovery fails.
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
            &self.config.packages
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
    /// # Errors
    /// Returns an error if analysis fails.
    pub fn analyze_dependencies(&self) -> Result<WorkspaceGraph, WorkspaceError> {
        // Extract owned packages
        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(packages.as_slice());

        // Check for cycles
        let cycles_detected = graph.detect_circular_dependencies().is_err();

        // Get missing dependencies
        let missing_dependencies = graph.find_missing_dependencies();

        // Get version conflicts
        let version_conflicts = graph.find_version_conflicts_for_package();

        // Run validation if no cycles
        let validation = if cycles_detected {
            None
        } else {
            match graph.validate_package_dependencies() {
                Ok(report) => Some(report),
                Err(_) => None,
            }
        };

        Ok(WorkspaceGraph { cycles_detected, missing_dependencies, version_conflicts, validation })
    }

    /// Gets a package by name.
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
    #[must_use]
    pub fn sorted_packages(&self) -> Vec<Rc<RefCell<PackageInfo>>> {
        self.get_sorted_packages_with_circulars().sorted
    }

    // New method that provides more detailed information
    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::items_after_statements)]
    pub fn get_sorted_packages_with_circulars(&self) -> SortedPackages {
        // Extract non-root packages
        let non_root_packages: Vec<Rc<RefCell<PackageInfo>>> = self
            .package_infos
            .iter()
            .filter(|info| {
                // Get the package's path
                let package_path = PathBuf::from(&info.borrow().package_path);
                // Check if the package path is different from the workspace root path
                package_path != self.root_path
            })
            .map(Rc::clone)
            .collect();

        // Nothing to sort if we have 0 or 1 packages
        if non_root_packages.is_empty() {
            return SortedPackages { sorted: Vec::new(), circular: Vec::new() };
        } else if non_root_packages.len() == 1 {
            return SortedPackages { sorted: non_root_packages, circular: Vec::new() };
        }

        // Extract the Package objects
        let packages: Vec<Package> =
            non_root_packages.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(packages.as_slice());

        // Create a map of package names to package info references
        let package_map: HashMap<String, Rc<RefCell<PackageInfo>>> = non_root_packages
            .iter()
            .map(|p| (p.borrow().package.borrow().name().to_string(), Rc::clone(p)))
            .collect();

        // Check for cycles
        let has_cycles = graph.detect_circular_dependencies().is_err();

        if has_cycles {
            // With cycles, we need to identify the components involved

            // For a proper implementation, we would use Tarjan's algorithm to find
            // strongly connected components (SCCs) which represent cycles.

            // For now, we'll use a simplified approach:
            // 1. Identify packages involved in dependencies
            // 2. Put those in the circular group
            // 3. Put the rest in sorted

            // This will need to be replaced with proper SCC detection
            let mut circular_packages = HashSet::new();

            // Basic cycle detection - this should be replaced with proper SCC
            let mut dependency_map: HashMap<String, HashSet<String>> = HashMap::new();

            // Build dependency map
            for pkg in &packages {
                let name = pkg.name().to_string();
                let deps: HashSet<String> = pkg
                    .dependencies()
                    .iter()
                    .map(|d| d.borrow().name().to_string())
                    .filter(|d| package_map.contains_key(d)) // Only include workspace packages
                    .collect();

                dependency_map.insert(name, deps);
            }

            // Simple cycle detection - mark packages in cycles
            for pkg_name in dependency_map.keys() {
                let mut visited = HashSet::new();
                let mut path = HashSet::new();

                fn has_cycle(
                    node: &str,
                    deps_map: &HashMap<String, HashSet<String>>,
                    visited: &mut HashSet<String>,
                    path: &mut HashSet<String>,
                    cycles: &mut HashSet<String>,
                ) -> bool {
                    if !visited.contains(node) {
                        visited.insert(node.to_string());
                        path.insert(node.to_string());

                        if let Some(deps) = deps_map.get(node) {
                            for dep in deps {
                                if !visited.contains(dep) {
                                    if has_cycle(dep, deps_map, visited, path, cycles) {
                                        cycles.insert(node.to_string());
                                        cycles.insert(dep.to_string());
                                        return true;
                                    }
                                } else if path.contains(dep) {
                                    // Found a cycle
                                    cycles.insert(node.to_string());
                                    cycles.insert(dep.to_string());
                                    return true;
                                }
                            }
                        }
                    }

                    path.remove(node);
                    false
                }

                has_cycle(
                    pkg_name,
                    &dependency_map,
                    &mut visited,
                    &mut path,
                    &mut circular_packages,
                );
            }

            // Separate circular and non-circular packages
            let mut circular_group = Vec::new();
            let mut sorted = Vec::new();

            for pkg in non_root_packages {
                let name = pkg.borrow().package.borrow().name().to_string();
                if circular_packages.contains(&name) {
                    circular_group.push(pkg);
                } else {
                    sorted.push(pkg);
                }
            }

            // We would need multiple circular groups based on SCCs
            let circular =
                if circular_group.is_empty() { Vec::new() } else { vec![circular_group] };

            return SortedPackages { sorted, circular };
        }

        // If no cycles, we can do a simple topological sort
        match toposort(&graph.graph, None) {
            Ok(sorted_indices) => {
                // Map sorted indices to package info references
                let sorted: Vec<_> = sorted_indices
                    .into_iter()
                    .filter_map(|idx| {
                        let node = graph.graph.node_weight(idx)?;
                        let resolved = node.as_resolved()?;
                        let name = resolved.identifier().to_string();
                        package_map.get(&name).map(Rc::clone)
                    })
                    .collect();

                // The natural output of toposort needs to be reversed
                // to get dependencies before dependents
                let mut reversed = Vec::new();
                for i in (0..sorted.len()).rev() {
                    reversed.push(Rc::clone(&sorted[i]));
                }

                SortedPackages { sorted: reversed, circular: Vec::new() }
            }
            Err(_) => {
                // This should never happen since we checked for cycles
                SortedPackages { sorted: non_root_packages, circular: Vec::new() }
            }
        }
    }

    /// Gets packages affected by changes in the specified packages.
    #[must_use]
    pub fn affected_packages(
        &self,
        changed_packages: &[&str],
        check_circular: Option<bool>,
    ) -> Vec<Rc<RefCell<PackageInfo>>> {
        let check = check_circular.unwrap_or(true); // Default to true for backward compatibility

        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let graph = DependencyGraph::from(packages.as_slice());

        if check && graph.detect_circular_dependencies().is_err() {
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
    #[must_use]
    pub fn dependents_of(
        &self,
        package_name: &str,
        check_circular: Option<bool>,
    ) -> Vec<Rc<RefCell<PackageInfo>>> {
        let check = check_circular.unwrap_or(true); // Default to true for backward compatibility

        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let graph = DependencyGraph::from(packages.as_slice());

        // Only check for cycles if the caller wants it
        if check && graph.detect_circular_dependencies().is_err() {
            return Vec::new();
        }

        match graph.get_dependents(&package_name.to_string()) {
            Ok(dependents) => dependents.iter().filter_map(|name| self.get_package(name)).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Gets direct dependencies of a package.
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
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Gets Git repository reference.
    #[must_use]
    pub fn git_repo(&self) -> Option<&Repo> {
        self.git_repo.as_deref()
    }

    /// Gets package manager.
    #[must_use]
    pub fn package_manager(&self) -> &Option<CorePackageManager> {
        &self.package_manager
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.package_infos.is_empty()
    }

    /// Writes package changes to disk.
    ///
    /// # Errors
    /// Returns an error if writing changes fails.
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
    /// # Errors
    /// Returns an error if validation fails.
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
    /// # Errors
    /// Returns an error if validation fails.
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

    // Private helper methods

    /// Processes a package.json file.
    ///
    /// # Errors
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
