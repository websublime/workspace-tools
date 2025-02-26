#[cfg(test)]
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
    fn test_packages_updates() {
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
}
