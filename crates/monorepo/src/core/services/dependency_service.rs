//! Dependency analysis service implementation
//!
//! Handles dependency graph analysis, conflict detection, and dependency
//! relationship mapping within the monorepo. Provides centralized dependency management.

use crate::config::MonorepoConfig;
use crate::core::types::MonorepoPackageInfo;
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use sublime_package_tools::{DependencyRegistry, RegistryManager};
use super::PackageDiscoveryService;

/// Dependency analysis service
///
/// Provides comprehensive dependency analysis for the monorepo including
/// dependency graph construction, conflict detection, circular dependency
/// detection, and dependency relationship mapping.
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::core::services::{DependencyAnalysisService, PackageDiscoveryService, FileSystemService};
/// use sublime_monorepo_tools::config::MonorepoConfig;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs_service = FileSystemService::new("/path/to/monorepo")?;
/// let config = MonorepoConfig::default();
/// let package_service = PackageDiscoveryService::new("/path/to/monorepo", &fs_service, &config)?;
/// let dependency_service = DependencyAnalysisService::new(&package_service, &config)?;
/// 
/// // Analyze dependencies for all packages
/// let graph = dependency_service.build_dependency_graph()?;
/// 
/// // Check for circular dependencies
/// let cycles = dependency_service.detect_circular_dependencies()?;
/// if !cycles.is_empty() {
///     println!("Found {} circular dependencies", cycles.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub(crate) struct DependencyAnalysisService {
    /// Dependency registry for package management
    dependency_registry: DependencyRegistry,
    
    /// Registry manager for package lookups
    registry_manager: RegistryManager,
    
    /// Cached dependency graph
    dependency_graph: Option<DependencyGraph>,
    
    /// Reference to package discovery service
    packages: Vec<MonorepoPackageInfo>,
}

/// Represents a dependency graph for the monorepo
///
/// Contains the relationships between all packages and their dependencies,
/// including both internal monorepo dependencies and external dependencies.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Map of package name to its dependencies
    pub dependencies: HashMap<String, Vec<String>>,
    
    /// Map of package name to packages that depend on it
    pub dependents: HashMap<String, Vec<String>>,
    
    /// List of external dependencies not part of the monorepo
    pub external_dependencies: HashSet<String>,
    
    /// List of circular dependency chains detected
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Represents a dependency conflict
///
/// Contains information about conflicting dependency versions between packages.
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    /// Name of the conflicting dependency
    pub dependency_name: String,
    
    /// Packages that have conflicting versions
    pub conflicting_packages: Vec<ConflictingPackage>,
}

/// Represents a package with a conflicting dependency version
#[derive(Debug, Clone)]
pub struct ConflictingPackage {
    /// Name of the package
    pub package_name: String,
    
    /// Version requirement for the dependency
    pub version_requirement: String,
}

#[allow(dead_code)]
impl DependencyAnalysisService {
    /// Create a new dependency analysis service
    ///
    /// Initializes the dependency analysis service with the provided package
    /// discovery service and configuration.
    ///
    /// # Arguments
    ///
    /// * `package_service` - Package discovery service for package information
    /// * `config` - Configuration for dependency analysis rules
    ///
    /// # Returns
    ///
    /// A new dependency analysis service.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package discovery service cannot provide package information
    /// - Dependency registry initialization fails
    /// - Configuration is invalid for dependency analysis
    pub fn new(
        package_service: &PackageDiscoveryService,
        _config: &MonorepoConfig,
    ) -> Result<Self> {
        // Initialize dependency registry and registry manager
        let dependency_registry = DependencyRegistry::new();
        
        let registry_manager = RegistryManager::new();
        
        // Get packages from the package service
        let packages = package_service.discover_packages()?;
        
        Ok(Self {
            dependency_registry,
            registry_manager,
            dependency_graph: None,
            packages,
        })
    }
    
    /// Get the dependency registry
    ///
    /// Provides access to the underlying dependency registry for operations
    /// that require direct registry manipulation.
    ///
    /// # Returns
    ///
    /// Reference to the dependency registry.
    pub fn registry(&self) -> &DependencyRegistry {
        &self.dependency_registry
    }
    
    /// Get the registry manager
    ///
    /// Provides access to the underlying registry manager for package
    /// lookup and resolution operations.
    ///
    /// # Returns
    ///
    /// Reference to the registry manager.
    pub fn registry_manager(&self) -> &RegistryManager {
        &self.registry_manager
    }
    
    /// Build dependency graph for all packages
    ///
    /// Constructs a complete dependency graph showing relationships between
    /// all packages in the monorepo and their external dependencies.
    ///
    /// # Returns
    ///
    /// Complete dependency graph for the monorepo.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package dependencies cannot be resolved
    /// - Dependency information is malformed
    /// - Circular dependencies prevent graph construction
    pub fn build_dependency_graph(&mut self) -> Result<&DependencyGraph> {
        let mut dependencies = HashMap::new();
        let mut dependents = HashMap::new();
        let mut external_dependencies = HashSet::new();
        
        // Get all package names for internal dependency detection
        let package_names: HashSet<String> = self.packages.iter()
            .map(|pkg| pkg.name().to_string())
            .collect();
        
        // Build dependency relationships
        for package in &self.packages {
            let mut pkg_dependencies = Vec::new();
            
            for dependency in &package.dependencies {
                pkg_dependencies.push(dependency.name.clone());
                
                // Check if this is an internal or external dependency
                if package_names.contains(&dependency.name) {
                    // Internal dependency - update dependents map
                    dependents.entry(dependency.name.clone())
                        .or_insert_with(Vec::new)
                        .push(package.name().to_string());
                } else {
                    // External dependency
                    external_dependencies.insert(dependency.name.clone());
                }
            }
            
            dependencies.insert(package.name().to_string(), pkg_dependencies);
        }
        
        // Detect circular dependencies
        let circular_dependencies = Self::detect_cycles_in_graph(&dependencies)?;
        
        let graph = DependencyGraph {
            dependencies,
            dependents,
            external_dependencies,
            circular_dependencies,
        };
        
        self.dependency_graph = Some(graph);
        
        // Return reference to the stored graph
        if let Some(ref graph) = self.dependency_graph {
            Ok(graph)
        } else {
            // This should never happen as we just set it
            Err(crate::error::Error::Analysis("Failed to store dependency graph".to_string()))
        }
    }
    
    /// Get cached dependency graph
    ///
    /// Returns the cached dependency graph if available, otherwise builds
    /// a new one. Use build_dependency_graph() to force a rebuild.
    ///
    /// # Returns
    ///
    /// Reference to the dependency graph.
    ///
    /// # Errors
    ///
    /// Returns an error if graph cannot be built.
    pub fn get_dependency_graph(&mut self) -> Result<&DependencyGraph> {
        if self.dependency_graph.is_none() {
            self.build_dependency_graph()?;
        }
        
        self.dependency_graph.as_ref()
            .ok_or_else(|| crate::error::Error::Analysis("Failed to build dependency graph".to_string()))
    }
    
    /// Detect circular dependencies
    ///
    /// Analyzes the dependency graph to find circular dependency chains
    /// that could cause build or runtime issues.
    ///
    /// # Returns
    ///
    /// Vector of circular dependency chains, where each chain is a vector
    /// of package names forming a cycle.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph analysis fails.
    pub fn detect_circular_dependencies(&mut self) -> Result<Vec<Vec<String>>> {
        let graph = self.get_dependency_graph()?;
        Ok(graph.circular_dependencies.clone())
    }
    
    /// Get packages that depend on a specific package
    ///
    /// Returns all packages that have the specified package as a dependency.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find dependents for
    ///
    /// # Returns
    ///
    /// Vector of package names that depend on the specified package.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph cannot be accessed.
    pub fn get_dependents(&mut self, package_name: &str) -> Result<Vec<String>> {
        let graph = self.get_dependency_graph()?;
        Ok(graph.dependents.get(package_name)
            .cloned()
            .unwrap_or_default())
    }
    
    /// Get dependencies of a specific package
    ///
    /// Returns all dependencies of the specified package, including both
    /// internal monorepo dependencies and external dependencies.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to get dependencies for
    ///
    /// # Returns
    ///
    /// Vector of dependency names for the specified package.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph cannot be accessed.
    pub fn get_dependencies(&mut self, package_name: &str) -> Result<Vec<String>> {
        let graph = self.get_dependency_graph()?;
        Ok(graph.dependencies.get(package_name)
            .cloned()
            .unwrap_or_default())
    }
    
    /// Get affected packages for changes
    ///
    /// Determines which packages might be affected by changes to the
    /// specified packages, based on the dependency graph.
    ///
    /// # Arguments
    ///
    /// * `changed_packages` - Names of packages that have changed
    ///
    /// # Returns
    ///
    /// Set of package names that might be affected by the changes.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph cannot be accessed.
    pub fn get_affected_packages(&mut self, changed_packages: &[String]) -> Result<HashSet<String>> {
        let graph = self.get_dependency_graph()?;
        let mut affected = HashSet::new();
        let mut to_process: Vec<String> = changed_packages.to_vec();
        
        // Add initially changed packages
        for package in changed_packages {
            affected.insert(package.clone());
        }
        
        // Traverse dependents recursively
        while let Some(current_package) = to_process.pop() {
            if let Some(dependents) = graph.dependents.get(&current_package) {
                for dependent in dependents {
                    if !affected.contains(dependent) {
                        affected.insert(dependent.clone());
                        to_process.push(dependent.clone());
                    }
                }
            }
        }
        
        Ok(affected)
    }
    
    /// Detect dependency conflicts
    ///
    /// Analyzes dependencies across all packages to find version conflicts
    /// where different packages require incompatible versions of the same dependency.
    ///
    /// # Returns
    ///
    /// Vector of dependency conflicts found in the monorepo.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency analysis fails.
    pub fn detect_dependency_conflicts(&self) -> Vec<DependencyConflict> {
        let mut dependency_versions: HashMap<String, Vec<ConflictingPackage>> = HashMap::new();
        
        // Collect all dependency versions across packages
        for package in &self.packages {
            for dependency in &package.dependencies {
                dependency_versions
                    .entry(dependency.name.clone())
                    .or_default()
                    .push(ConflictingPackage {
                        package_name: package.name().to_string(),
                        version_requirement: dependency.version_requirement.clone(),
                    });
            }
        }
        
        // Find conflicts (same dependency with different version requirements)
        let mut conflicts = Vec::new();
        
        for (dep_name, packages) in dependency_versions {
            if packages.len() > 1 {
                // Check if all version requirements are the same
                let first_version = &packages[0].version_requirement;
                let has_conflict = packages.iter()
                    .any(|pkg| pkg.version_requirement != *first_version);
                
                if has_conflict {
                    conflicts.push(DependencyConflict {
                        dependency_name: dep_name,
                        conflicting_packages: packages,
                    });
                }
            }
        }
        
        conflicts
    }
    
    /// Get external dependencies
    ///
    /// Returns all external dependencies (not part of the monorepo) used
    /// by packages in the monorepo.
    ///
    /// # Returns
    ///
    /// Set of external dependency names.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency graph cannot be accessed.
    pub fn get_external_dependencies(&mut self) -> Result<HashSet<String>> {
        let graph = self.get_dependency_graph()?;
        Ok(graph.external_dependencies.clone())
    }
    
    /// Analyze dependency update impact
    ///
    /// Determines the impact of updating a specific dependency across
    /// all packages that use it.
    ///
    /// # Arguments
    ///
    /// * `dependency_name` - Name of the dependency to analyze
    /// * `new_version` - New version to update to
    ///
    /// # Returns
    ///
    /// Vector of package names that would be affected by the update.
    ///
    /// # Errors
    ///
    /// Returns an error if dependency analysis fails.
    pub fn analyze_dependency_update_impact(
        &self,
        dependency_name: &str,
        _new_version: &str,
    ) -> Vec<String> {
        let mut affected_packages = Vec::new();
        
        // Find all packages that use this dependency
        for package in &self.packages {
            for dependency in &package.dependencies {
                if dependency.name == dependency_name {
                    affected_packages.push(package.name().to_string());
                    break;
                }
            }
        }
        
        affected_packages
    }
    
    /// Validate dependency constraints
    ///
    /// Validates that all dependency constraints are satisfied and
    /// there are no impossible dependency requirements.
    ///
    /// # Returns
    ///
    /// Success if all dependency constraints are valid.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Circular dependencies are detected
    /// - Incompatible version constraints exist
    /// - Required dependencies are missing
    pub fn validate_dependency_constraints(&mut self) -> Result<()> {
        // Check for circular dependencies
        let circular_deps = self.detect_circular_dependencies()?;
        if !circular_deps.is_empty() {
            return Err(crate::error::Error::dependency(format!(
                "Circular dependencies detected: {} cycles found",
                circular_deps.len()
            )));
        }
        
        // Check for dependency conflicts
        let conflicts = self.detect_dependency_conflicts();
        if !conflicts.is_empty() {
            return Err(crate::error::Error::dependency(format!(
                "Dependency conflicts detected: {} conflicts found",
                conflicts.len()
            )));
        }
        
        Ok(())
    }
    
    /// Detect cycles in dependency graph
    ///
    /// Internal method to detect circular dependencies using depth-first search.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Map of package dependencies
    ///
    /// # Returns
    ///
    /// Vector of circular dependency chains found.
    ///
    /// # Errors
    ///
    /// Returns an error if graph traversal fails.
    fn detect_cycles_in_graph(
        dependencies: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Vec<String>>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();
        
        for package in dependencies.keys() {
            if !visited.contains(package) {
                let mut path = Vec::new();
                Self::dfs_detect_cycle(
                    package,
                    dependencies,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                )?;
            }
        }
        
        Ok(cycles)
    }
    
    /// Depth-first search for cycle detection
    ///
    /// Internal recursive method for detecting cycles using DFS.
    ///
    /// # Arguments
    ///
    /// * `package` - Current package being processed
    /// * `dependencies` - Map of package dependencies
    /// * `visited` - Set of visited packages
    /// * `rec_stack` - Recursion stack for cycle detection
    /// * `path` - Current path in DFS
    /// * `cycles` - Found cycles collector
    ///
    /// # Returns
    ///
    /// Success if DFS completes without error.
    ///
    /// # Errors
    ///
    /// Returns an error if DFS encounters an issue.
    fn dfs_detect_cycle(
        package: &str,
        dependencies: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) -> Result<()> {
        visited.insert(package.to_string());
        rec_stack.insert(package.to_string());
        path.push(package.to_string());
        
        if let Some(deps) = dependencies.get(package) {
            for dep in deps {
                if !visited.contains(dep) {
                    Self::dfs_detect_cycle(dep, dependencies, visited, rec_stack, path, cycles)?;
                } else if rec_stack.contains(dep) {
                    // Found a cycle - extract the cycle from the path
                    if let Some(cycle_start) = path.iter().position(|p| p == dep) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(package);
        
        Ok(())
    }
    
    /// Refresh packages from package service
    ///
    /// Updates the internal package list with fresh data from the package
    /// discovery service and invalidates cached dependency graphs.
    ///
    /// # Arguments
    ///
    /// * `package_service` - Package discovery service for fresh data
    ///
    /// # Returns
    ///
    /// Success if packages were refreshed successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if package discovery fails.
    pub fn refresh_packages(&mut self, package_service: &PackageDiscoveryService) -> Result<()> {
        self.packages = package_service.discover_packages()?;
        self.dependency_graph = None; // Invalidate cached graph
        Ok(())
    }
}