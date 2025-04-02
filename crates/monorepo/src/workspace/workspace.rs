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
    ) -> Result<(), WorkspaceError> {
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

                // If it's a package.json file, process it
                if path.file_name().map_or(false, |n| n == "package.json") {
                    if let Some(package_info) = self.process_package_json(&path)? {
                        package_infos.push(package_info);
                    }
                }
            }
        }

        self.package_infos = package_infos;
        Ok(())
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
        // Extract non-root packages
        let non_root_packages: Vec<Rc<RefCell<PackageInfo>>> = self
            .package_infos
            .iter()
            .filter(|info| {
                // Get the package's path
                let package_path = PathBuf::from(&info.borrow().package_path);

                // Check if the package path is different from the workspace root path
                // A non-root package will be in a subdirectory of the workspace
                package_path != self.root_path
            })
            .map(Rc::clone)
            .collect();

        // Nothing to sort if we have 0 or 1 packages
        if non_root_packages.is_empty() {
            return Vec::new();
        } else if non_root_packages.len() == 1 {
            return non_root_packages;
        }

        // Extract the Package objects for the dependency graph
        let packages: Vec<Package> =
            non_root_packages.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        // Create the dependency graph
        let graph = DependencyGraph::from(packages.as_slice());

        // Create a map of package names to package info references
        let package_map: HashMap<String, Rc<RefCell<PackageInfo>>> = non_root_packages
            .iter()
            .map(|p| (p.borrow().package.borrow().name().to_string(), Rc::clone(p)))
            .collect();

        // If there are cycles, just return unsorted non-root packages
        if graph.detect_circular_dependencies().is_err() {
            return non_root_packages;
        }

        // Perform topological sort
        match toposort(&graph.graph, None) {
            Ok(sorted_indices) => {
                // Map sorted indices to package info references
                // The result of toposort has dependents before dependencies,
                // so we need to reverse it to get the correct order
                let mut result: Vec<_> = sorted_indices
                    .into_iter()
                    .filter_map(|idx| {
                        let node = graph.graph.node_weight(idx)?;
                        let resolved = node.as_resolved()?;
                        let name = resolved.identifier().to_string();
                        package_map.get(&name).map(Rc::clone)
                    })
                    .collect();

                // Reverse to get dependencies before dependents
                result.reverse();
                result
            }
            Err(_) => {
                // This shouldn't happen since we already checked for cycles,
                // but just in case, return unsorted packages
                non_root_packages
            }
        }
    }

    /// Gets packages affected by changes in the specified packages.
    #[must_use]
    pub fn affected_packages(&self, changed_packages: &[&str]) -> Vec<Rc<RefCell<PackageInfo>>> {
        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let graph = DependencyGraph::from(packages.as_slice());

        if graph.detect_circular_dependencies().is_err() {
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
    pub fn dependents_of(&self, package_name: &str) -> Vec<Rc<RefCell<PackageInfo>>> {
        let packages: Vec<Package> =
            self.package_infos.iter().map(|info| info.borrow().package.borrow().clone()).collect();

        let graph = DependencyGraph::from(packages.as_slice());

        if graph.detect_circular_dependencies().is_err() {
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

        // Extract name and version using let...else pattern
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

        // Get package path using let...else pattern
        let Some(package_path) = path.parent() else {
            return Err(WorkspaceError::InvalidConfiguration(format!(
                "Invalid package path: {}",
                path.display()
            )));
        };
        let package_path = package_path.to_path_buf();

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
