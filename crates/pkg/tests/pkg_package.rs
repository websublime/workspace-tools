#[cfg(test)]
#[allow(clippy::print_stdout)]
#[allow(clippy::uninlined_format_args)]
mod package_tests {
    use ws_pkg::dependency::DependencyGraph;
    use ws_pkg::package::{Dependency, Package, PackageInfo};

    #[test]
    fn test_dependency() {
        let dep_foo = Dependency::new("@scope/foo", "1.0.0");

        assert_eq!(dep_foo.name(), "@scope/foo");
        assert_eq!(dep_foo.version().to_string(), "^1.0.0");

        let dep_bar = Dependency::new("@scope/bar", ">=1.0.0");
        dep_bar.update_version("2.0.0");

        assert_eq!(dep_bar.name(), "@scope/bar");
        assert_eq!(dep_bar.version().to_string(), "^2.0.0");
    }

    #[test]
    fn test_package() {
        let dep_foo = Dependency::new("@scope/foo", "1.0.0");
        let dep_bar = Dependency::new("@scope/bar", "1.1.0");

        let pkg_charlie =
            Package::new("@scope/charlie", "0.1.1", Some(vec![dep_bar.clone(), dep_foo.clone()]));

        assert_eq!(pkg_charlie.name(), "@scope/charlie");
        assert_eq!(pkg_charlie.version().to_string(), "0.1.1");
        assert_eq!(pkg_charlie.dependencies().len(), 2);
        assert_eq!(pkg_charlie.dependencies()[0].name(), "@scope/bar");
        assert_eq!(pkg_charlie.dependencies()[0].version().to_string(), "^1.1.0");
        assert_eq!(pkg_charlie.dependencies()[1].name(), "@scope/foo");
        assert_eq!(pkg_charlie.dependencies()[1].version().to_string(), "^1.0.0");

        pkg_charlie.update_version("0.2.0");
        pkg_charlie.update_dependency_version("@scope/foo", "2.0.0");

        assert_eq!(pkg_charlie.version().to_string(), "0.2.0");
        assert_eq!(pkg_charlie.dependencies()[1].version().to_string(), "^2.0.0");
        assert_eq!(dep_foo.version().to_string(), "^2.0.0");
    }

    #[test]
    fn test_package_info() {
        // Create a dependency
        let dep_foo = Dependency::new("@scope/foo", ">=2.0.0");

        // Create a package with the dependency
        let pkg = Package::new("@scope/bar", "1.0.0", Some(vec![dep_foo.clone()]));

        // Create package JSON
        let mut pkg_json = serde_json::Map::new();
        pkg_json.insert("name".to_string(), serde_json::Value::String("@scope/bar".to_string()));
        pkg_json.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));

        let mut deps_map = serde_json::Map::new();
        deps_map.insert("@scope/foo".to_string(), serde_json::Value::String(">=2.0.0".to_string()));
        pkg_json.insert("dependencies".to_string(), serde_json::Value::Object(deps_map));

        // Create package info
        let mut pkg_info = PackageInfo::new(
            pkg.clone(),
            String::from("/path/to/package.json"),
            String::from("/path/to"),
            String::from("path/to"),
            serde_json::Value::Object(pkg_json),
        );

        // Update version through package info
        pkg_info.update_version("2.0.0");

        // Update dependency version through package info
        pkg_info.update_dependency_version("@scope/foo", "3.0.0");

        // Verify package version was updated
        assert_eq!(pkg.version_str(), "2.0.0");

        // Verify original dependency reference was updated
        assert_eq!(dep_foo.version_str(), "^3.0.0");

        // Verify package info JSON was updated
        assert_eq!(pkg_info.pkg_json["version"].as_str().unwrap(), "2.0.0");
        assert_eq!(pkg_info.pkg_json["dependencies"]["@scope/foo"].as_str().unwrap(), "3.0.0");
    }

    #[test]
    fn test_multiple_packages_share_dependency() {
        // Create a shared dependency
        let dep_foo = Dependency::new("@scope/foo", ">=2.0.0");

        // Create two packages with the same dependency
        let pkg1 = Package::new("@scope/bar", "1.0.0", Some(vec![dep_foo.clone()]));

        let pkg2 = Package::new("@scope/baz", "1.0.0", Some(vec![dep_foo.clone()]));

        // Update dependency through first package
        pkg1.update_dependency_version("@scope/foo", "3.0.0");

        // Verify all instances see the update
        assert_eq!(dep_foo.version_str(), "^3.0.0");
        assert_eq!(pkg1.dependencies()[0].version_str(), "^3.0.0");
        assert_eq!(pkg2.dependencies()[0].version_str(), "^3.0.0");
    }

    #[test]
    fn test_dependency_graph() {
        // Create actual package nodes for the dependencies
        let pkg_foo = Package::new("@scope/foo", "1.0.0", None);
        let pkg_bar = Package::new("@scope/bar", "1.1.0", None);
        let pkg_baz = Package::new("@scope/baz", "1.2.0", None);

        // Then create dependencies that reference these packages
        let dep_foo = Dependency::new("@scope/foo", "1.0.0");
        let dep_bar = Dependency::new("@scope/bar", "1.1.0");
        let dep_baz = Dependency::new("@scope/baz", "1.2.0");

        let pkg_charlie =
            Package::new("@scope/charlie", "0.1.1", Some(vec![dep_bar.clone(), dep_foo.clone()]));
        let pkg_delta =
            Package::new("@scope/delta", "0.2.1", Some(vec![dep_baz.clone(), dep_foo.clone()]));
        let pkg_echo =
            Package::new("@scope/echo", "0.3.1", Some(vec![dep_baz.clone(), dep_bar.clone()]));

        // Include all packages in the graph
        let pkgs = [pkg_foo, pkg_bar, pkg_baz, pkg_charlie, pkg_delta, pkg_echo];

        let dependency_graph = DependencyGraph::from(&pkgs[..]);

        // Now we can get the dependents of foo, which should be charlie and delta
        let dependents = dependency_graph
            .get_dependents(&"@scope/foo".to_string())
            .expect("Error getting dependents");

        assert_eq!(dependents.len(), 2);
        assert_eq!(dependents[0], "@scope/charlie");
        assert_eq!(dependents[1], "@scope/delta");

        // Now we can get the dependents of bar, which should be charlie and echo
        let dependents = dependency_graph
            .get_dependents(&"@scope/bar".to_string())
            .expect("Error getting dependents");

        assert_eq!(dependents.len(), 2);
        assert_eq!(dependents[0], "@scope/charlie");
        assert_eq!(dependents[1], "@scope/echo");
    }
}
/*#[cfg(test)]
#[allow(clippy::print_stdout)]
#[allow(clippy::uninlined_format_args)]
mod package_tests {
    use petgraph::dot::Dot;
    use semver::Version;
    use ws_pkg::dependency::{DependencyGraph, Node};
    use ws_pkg::package::{Dependency, Package, PackageInfo};

    fn build_packages() -> Vec<Package> {
        vec![
            Package::new(
                "@scope/bar",
                Version::parse("1.0.0").unwrap().to_string().as_str(),
                Some(vec![Dependency {
                    name: "@scope/foo".to_string(),
                    version: ">=2.0.0".parse().unwrap(),
                }]),
            ),
            Package::new("@scope/foo", Version::parse("2.0.0").unwrap().to_string().as_str(), None),
            Package::new(
                "@scope/baz",
                Version::parse("3.0.0").unwrap().to_string().as_str(),
                Some(vec![
                    Dependency {
                        name: "@scope/bar".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                    Dependency {
                        name: "@scope/foo".to_string(),
                        version: ">=2.0.0".parse().unwrap(),
                    },
                ]),
            ),
        ]
    }

    #[test]
    fn test_display() {
        let pkgs = build_packages();
        let dependency_graph = DependencyGraph::from(&pkgs[..]);
        let dot = Dot::new(&dependency_graph.graph);
        println!("{:?}", dot);
    }

    #[test]
    fn test_packages_dependencies_and_dependents() {
        let pkgs = build_packages();

        let dependency_graph = DependencyGraph::from(&pkgs[..]);
        let dep: Vec<&Package> = dependency_graph.resolved_dependencies().collect();
        let dependents = dependency_graph
            .get_dependents(&"@scope/foo".to_string())
            .expect("Error getting dependents");

        assert_eq!(dep.len(), 3);

        assert_eq!(dep[0].name, "@scope/bar");
        assert_eq!(dep[0].version.to_string(), "1.0.0");
        assert_eq!(dep[1].name, "@scope/foo");
        assert_eq!(dep[1].version.to_string(), "2.0.0");
        assert_eq!(dep[2].name, "@scope/baz");
        assert_eq!(dep[2].version.to_string(), "3.0.0");

        assert_eq!(dependents.len(), 2);
        assert_eq!(dependents[0], "@scope/bar");
        assert_eq!(dependents[1], "@scope/baz");
    }

    #[test]
    fn test_build_dependency_graph_from_package_infos() {
        let pkgs = build_packages();
        let package_infos: Vec<PackageInfo> = pkgs
            .iter()
            .map(|pkg| PackageInfo {
                package: pkg.clone(),
                package_json_path: String::from("/root/package/package.json"),
                package_path: String::from("/root/package"),
                package_relative_path: String::from("package"),
                pkg_json: serde_json::Value::String("{}".to_string()),
            })
            .collect();
        let mut packages = Vec::new();

        let dependency_graph = ws_pkg::package::build_dependency_graph_from_package_infos(
            &package_infos,
            &mut packages,
        );
        let dep: Vec<&Package> = dependency_graph.resolved_dependencies().collect();
        let dependents = dependency_graph
            .get_dependents(&"@scope/foo".to_string())
            .expect("Error getting dependents");

        assert_eq!(dep.len(), 3);

        assert_eq!(dep[0].name, "@scope/bar");
        assert_eq!(dep[0].version.to_string(), "1.0.0");
        assert_eq!(dep[1].name, "@scope/foo");
        assert_eq!(dep[1].version.to_string(), "2.0.0");
        assert_eq!(dep[2].name, "@scope/baz");
        assert_eq!(dep[2].version.to_string(), "3.0.0");

        assert_eq!(dependents.len(), 2);
        assert_eq!(dependents[0], "@scope/bar");
        assert_eq!(dependents[1], "@scope/baz");
    }

    #[test]
    fn test_package_updates() {
        let dep_foo = Dependency {
            name: "@scope/foo".to_string(),
            version: ">=2.0.0".parse().unwrap(),
        };

        let mut pkg = Package::new(
            "@scope/bar",
            Version::parse("1.0.0").unwrap().to_string().as_str(),
            Some(vec![dep_foo.clone()]),
        );

        pkg.update_version("2.0.0");
        pkg.update_dependency_version("@scope/foo", "3.0.0");

        assert_eq!(pkg.version.to_string(), "2.0.0");
        assert_eq!(pkg.dependencies().len(), 1);
        assert_eq!(dep_foo.version.to_string(), "3.0.0");
    }

    #[test]
    fn test_packages_info_updates() {
        let pkgs = build_packages();
        let mut package_infos: Vec<PackageInfo> = pkgs
            .iter()
            .map(|pkg| {
                let mut pkg_json = serde_json::Map::new();
                pkg_json
                    .insert("name".to_string(), serde_json::Value::String(pkg.name.to_string()));
                pkg_json.insert(
                    "version".to_string(),
                    serde_json::Value::String(pkg.version.to_string()),
                );
                pkg_json.insert("dependencies".to_string(), {
                    let mut deps = serde_json::Map::new();
                    for dep in pkg.dependencies() {
                        deps.insert(
                            dep.name.clone(),
                            serde_json::Value::String(dep.version.to_string()),
                        );
                    }
                    serde_json::Value::Object(deps)
                });

                PackageInfo {
                    package: pkg.clone(),
                    package_json_path: format!("/root/packages/{}/package.json", pkg.name),
                    package_path: format!("/root/packages/{}", pkg.name),
                    package_relative_path: format!("packages/{}", pkg.name),
                    pkg_json: serde_json::Value::Object(pkg_json),
                }
            })
            .collect();

        for pkg_info in &mut package_infos {
            if pkg_info.package.name == "@scope/foo" {
                pkg_info.update_version("3.0.0");
            }
        }

        for pkg_info in &mut package_infos {
            pkg_info.update_dependency_version("@scope/foo", "3.0.0");
        }

        // find all changed packages
        let changed_packages: Vec<&Package> = package_infos
            .iter()
            .filter(|pkg_info| {
                let pkg = &pkg_info.package;
                let pkg_json = &pkg_info.pkg_json;
                let version = pkg_json["version"].as_str().unwrap();
                let dependencies = pkg_json["dependencies"].as_object().unwrap();

                pkg.version.to_string() != version
                    || pkg.dependencies().iter().any(|dep| {
                        let dep_version = dependencies[&dep.name].as_str().unwrap();
                        dep.version.to_string() != dep_version
                    })
            })
            .map(|pkg_info| &pkg_info.package)
            .collect();

        assert_eq!(changed_packages.len(), 2);
    }
}*/
