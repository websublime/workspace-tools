#[cfg(test)]
#[allow(clippy::print_stdout)]
#[allow(clippy::uninlined_format_args)]
mod package_tests {
    use petgraph::dot::Dot;
    use semver::Version;
    use ws_pkg::dependency::{DependencyGraph, Node, Step};
    use ws_pkg::package::{Dependency, Package};

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
    fn test_packages_dependents() {
        let pkgs = &build_packages();

        let dependency_graph = DependencyGraph::from(&pkgs[..]);
        let dep = dependency_graph.resolved_dependencies().map(|f| f).collect::<Vec<_>>();
        let dependents = dependency_graph.get_dependents(&"@scope/foo".to_string());

        dependency_graph.propagate_update(&"@scope/foo".to_string(), |node, dependents| {
            match node {
                Step::Resolved(package) => {
                    // Update the package and its dependents
                    println!("Updating {} and its dependents: {:?}", package.name, dependents);
                }
                Step::Unresolved(_) => {
                    println!("Cannot update unresolved dependency");
                }
            }
        });

        for pkg in pkgs {
            pkg.dependencies().iter().for_each(|dependency| {
                println!("{:?}", dependency.name);
            });
        }
    }
}
