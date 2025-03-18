//! JavaScript bindings for graph construction utilities.

use crate::graph::validation::ValidationReport; // Add this import
use crate::types::package::Package;
use napi::bindgen_prelude::*;
use napi::Result as NapiResult;
use napi_derive::napi;
use ws_pkg::graph::{
    build_dependency_graph_from_packages as ws_build_from_packages,
    DependencyFilter as WsDependencyFilter, DependencyGraph as WsDependencyGraph,
};

/// JavaScript binding for ws_pkg::graph::DependencyFilter
#[napi]
#[derive(Clone)]
pub enum DependencyFilter {
    /// Include only production dependencies
    ProductionOnly,
    /// Include production and development dependencies
    WithDevelopment,
    /// Include production, development, and optional dependencies
    AllDependencies,
}

impl From<DependencyFilter> for WsDependencyFilter {
    fn from(filter: DependencyFilter) -> Self {
        match filter {
            DependencyFilter::ProductionOnly => WsDependencyFilter::ProductionOnly,
            DependencyFilter::WithDevelopment => WsDependencyFilter::WithDevelopment,
            DependencyFilter::AllDependencies => WsDependencyFilter::AllDependencies,
        }
    }
}

impl From<WsDependencyFilter> for DependencyFilter {
    fn from(filter: WsDependencyFilter) -> Self {
        match filter {
            WsDependencyFilter::ProductionOnly => DependencyFilter::ProductionOnly,
            WsDependencyFilter::WithDevelopment => DependencyFilter::WithDevelopment,
            WsDependencyFilter::AllDependencies => DependencyFilter::AllDependencies,
        }
    }
}

/// JavaScript binding for ws_pkg::graph::DependencyGraph
#[napi]
pub struct DependencyGraph {
    // Store owned packages to ensure they live as long as the graph
    packages: Vec<ws_pkg::Package>,
    // Store a reference to the inner graph
    pub(crate) inner: WsDependencyGraph<'static, ws_pkg::Package>,
}

#[napi]
impl DependencyGraph {
    /// Check if all dependencies in the graph can be resolved internally
    ///
    /// @returns {boolean} True if all dependencies can be resolved within the workspace
    #[napi]
    pub fn is_internally_resolvable(&self) -> bool {
        self.inner.is_internally_resolvable()
    }

    /// Find missing (unresolved) dependencies in the workspace
    ///
    /// @returns {string[]} Array of missing dependency names
    #[napi]
    pub fn find_missing_dependencies(&self) -> Vec<String> {
        self.inner.find_missing_dependencies()
    }

    /// Find version conflicts in the dependency graph
    ///
    /// @returns {Object | null} Map of dependency names to arrays of conflicting versions,
    ///                         or null if no conflicts found
    #[napi(ts_return_type = "Record<string,string>|null")]
    pub fn find_version_conflicts(&self, env: Env) -> NapiResult<Option<Object>> {
        let conflicts_opt = self.inner.find_version_conflicts();

        if let Some(conflicts) = conflicts_opt {
            let mut result_obj = env.create_object()?;

            for (name, versions) in conflicts {
                let mut versions_array = env.create_array_with_length(versions.len())?;

                for (i, version) in versions.iter().enumerate() {
                    let js_version = env.create_string(version)?;
                    versions_array.set_element(i as u32, js_version)?;
                }

                result_obj.set_named_property(&name, versions_array)?;
            }

            Ok(Some(result_obj))
        } else {
            Ok(None)
        }
    }

    /// Detect circular dependencies in the graph
    ///
    /// @returns {string[] | null} Path of the cycle if found, null otherwise
    #[napi]
    pub fn detect_circular_dependencies(&self) -> Option<Vec<String>> {
        match self.inner.detect_circular_dependencies() {
            Ok(()) => None,
            Err(ws_pkg::PkgError::CircularDependency { path }) => Some(path),
            Err(_) => None, // Other errors are ignored here, as they're unexpected
        }
    }

    /// Get a node by its identifier
    ///
    /// @param {string} id - The node identifier
    /// @returns {Package | null} The package if found, null otherwise
    #[napi]
    pub fn get_node(&self, id: String) -> Option<Package> {
        // For safety, directly search in owned packages instead
        for pkg in &self.packages {
            if pkg.name() == id {
                return Some(Package { inner: pkg.clone() });
            }
        }

        None
    }

    /// Get dependents of a package
    ///
    /// @param {string} id - The package identifier
    /// @returns {string[]} Array of package names that depend on this package
    #[napi(ts_return_type = "string[]")]
    pub fn get_dependents(&self, id: String) -> NapiResult<Vec<String>> {
        // Build dependents map manually for safety
        let mut dependents = Vec::new();

        for pkg in &self.packages {
            let pkg_name = pkg.name().to_string(); // Get and own the package name

            // Check if this package depends on the target
            let depends_on_target = pkg.dependencies().iter().any(|dep_rc| {
                // Use a separate scope to limit the borrow
                let dep_name = {
                    let dep = dep_rc.borrow();
                    dep.name().to_string() // Clone the name to own it
                };

                dep_name == id
            });

            if depends_on_target {
                dependents.push(pkg_name);
            }
        }

        Ok(dependents)
    }

    /// Validate dependencies in the graph
    ///
    /// @returns {ValidationReport} A report of validation issues
    #[napi(ts_return_type = "ValidationReport")]
    pub fn validate_package_dependencies(&self) -> NapiResult<ValidationReport> {
        // Use the pkg_try! macro to handle errors
        let ws_report = crate::pkg_try!(self.inner.validate_package_dependencies());

        // Create a ValidationReport with the result
        Ok(ValidationReport { inner: ws_report })
    }
}

/// Build a dependency graph from packages
///
/// @param {Package[]} packages - Array of packages to include in the graph
/// @returns {DependencyGraph} The constructed dependency graph
#[napi]
pub fn build_dependency_graph_from_packages(packages: Vec<&Package>) -> DependencyGraph {
    // Clone all the packages to have owned data
    let owned_packages: Vec<ws_pkg::Package> = packages.iter().map(|p| p.inner.clone()).collect();

    // Build the graph using a reference to the owned packages
    let graph = ws_build_from_packages(&owned_packages);

    // Use transmute to extend lifetime, but now it's safe because we own the packages
    let graph_static = unsafe {
        std::mem::transmute::<
            ws_pkg::DependencyGraph<'_, ws_pkg::Package>,
            ws_pkg::DependencyGraph<'_, ws_pkg::Package>,
        >(graph)
    };

    DependencyGraph { packages: owned_packages, inner: graph_static }
}

/// Build a dependency graph from package infos
///
/// @param {PackageInfo[]} packageInfos - Array of package infos
/// @returns {DependencyGraph} The constructed dependency graph
#[napi]
pub fn build_dependency_graph_from_package_infos(
    package_infos: Vec<&crate::types::package::PackageInfo>,
) -> DependencyGraph {
    // Extract and clone packages from package infos
    let mut owned_packages = Vec::new();
    for pkg_info in &package_infos {
        owned_packages.push(pkg_info.inner.borrow().package.borrow().clone());
    }

    // Build graph using the owned packages
    let graph = ws_build_from_packages(&owned_packages);

    // Extend lifetime, now safe because we own the data
    let graph_static = unsafe {
        std::mem::transmute::<
            ws_pkg::DependencyGraph<'_, ws_pkg::Package>,
            ws_pkg::DependencyGraph<'_, ws_pkg::Package>,
        >(graph)
    };

    DependencyGraph { packages: owned_packages, inner: graph_static }
}

#[cfg(test)]
mod builder_binding_tests {
    use super::*;
    use crate::types::dependency::Dependency;
    use crate::types::package::Package;

    #[test]
    fn test_dependency_filter_conversion() {
        assert!(matches!(
            WsDependencyFilter::from(DependencyFilter::ProductionOnly),
            WsDependencyFilter::ProductionOnly
        ));
        assert!(matches!(
            WsDependencyFilter::from(DependencyFilter::WithDevelopment),
            WsDependencyFilter::WithDevelopment
        ));
        assert!(matches!(
            WsDependencyFilter::from(DependencyFilter::AllDependencies),
            WsDependencyFilter::AllDependencies
        ));

        assert!(matches!(
            DependencyFilter::from(WsDependencyFilter::ProductionOnly),
            DependencyFilter::ProductionOnly
        ));
        assert!(matches!(
            DependencyFilter::from(WsDependencyFilter::WithDevelopment),
            DependencyFilter::WithDevelopment
        ));
        assert!(matches!(
            DependencyFilter::from(WsDependencyFilter::AllDependencies),
            DependencyFilter::AllDependencies
        ));
    }

    #[test]
    fn test_build_dependency_graph() {
        // Create packages
        let mut pkg1 = Package::new("pkg1".to_string(), "1.0.0".to_string());
        let pkg2 = Package::new("pkg2".to_string(), "1.0.0".to_string());

        // Add dependency
        let dep = Dependency::new("pkg2".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep);

        // Build graph
        let graph = build_dependency_graph_from_packages(vec![&pkg1, &pkg2]);

        // Test basic functionality
        assert!(graph.is_internally_resolvable());
        assert!(graph.find_missing_dependencies().is_empty());

        // Test node access - using explicit debug output to see what's happening
        if let Some(node) = graph.get_node("pkg1".to_string()) {
            let actual_name = node.name();
            println!("Expected: pkg1, Got: {}", actual_name);
            assert_eq!(actual_name, "pkg1");
        } else {
            // If the node isn't found, that's a different error
            panic!("Node pkg1 not found in graph");
        }

        // Test dependents - this also needs to be fixed
        match graph.get_dependents("pkg2".to_string()) {
            Ok(dependents) => {
                assert!(!dependents.is_empty(), "Expected pkg2 to have dependents");
                if !dependents.is_empty() {
                    println!("First dependent: {}", dependents[0]);
                    assert_eq!(dependents[0], "pkg1");
                }
            }
            Err(e) => {
                panic!("Failed to get dependents: {:?}", e);
            }
        }
    }
}
