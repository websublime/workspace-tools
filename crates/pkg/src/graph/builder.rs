use crate::{Graph, Package, Info};

/// Build a dependency graph from packages
pub fn build_dependency_graph_from_packages(packages: &[Package]) -> Graph<'_, Package> {
    Graph::from(packages)
}

/// Build a dependency graph from package infos
pub fn build_dependency_graph_from_package_infos<'a>(
    package_infos: &[Info],
    packages: &'a mut Vec<Package>,
) -> Graph<'a, Package> {
    // Extract packages from package infos
    packages.clear();
    for pkg_info in package_infos {
        packages.push(pkg_info.package.clone());
    }

    // Build dependency graph
    build_dependency_graph_from_packages(packages)
}
