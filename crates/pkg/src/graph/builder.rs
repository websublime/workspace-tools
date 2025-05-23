use crate::{DependencyGraph, Package, PackageInfo};

/// Build a dependency graph from packages
pub fn build_dependency_graph_from_packages(packages: &[Package]) -> DependencyGraph<'_, Package> {
    DependencyGraph::from(packages)
}

/// Build a dependency graph from package infos
pub fn build_dependency_graph_from_package_infos<'a>(
    package_infos: &[PackageInfo],
    packages: &'a mut Vec<Package>,
) -> DependencyGraph<'a, Package> {
    // Extract packages from package infos
    packages.clear();
    for pkg_info in package_infos {
        packages.push(pkg_info.package.borrow().clone());
    }

    // Build dependency graph
    build_dependency_graph_from_packages(packages)
}
