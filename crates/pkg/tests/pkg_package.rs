#[cfg(test)]
#[allow(clippy::print_stdout)]
#[allow(clippy::uninlined_format_args)]
mod package_tests {
    use petgraph::dot::Dot;
    use semver::Version;
    use ws_pkg::dependency::DependencyGraph;
    use ws_pkg::package::{Dependency, Package};

    fn build_packages() -> Vec<Package> {
        vec![
            Package {
                name: "@scope/bar".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                dependencies: vec![Dependency {
                    name: "@scope/foo".to_string(),
                    version: ">=2.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "@scope/foo".to_string(),
                version: Version::parse("2.0.0").unwrap(),
                dependencies: vec![],
            },
            Package {
                name: "@scope/baz".to_string(),
                version: Version::parse("3.0.0").unwrap(),
                dependencies: vec![
                    Dependency {
                        name: "@scope/bar".to_string(),
                        version: ">=1.0.0".parse().unwrap(),
                    },
                    Dependency {
                        name: "@scope/foo".to_string(),
                        version: ">=2.0.0".parse().unwrap(),
                    },
                ],
            },
        ]
    }

    #[test]
    fn test_display() {
        let pkgs = build_packages();
        let dependency_graph = DependencyGraph::from(&pkgs[..]);
        let dot = Dot::new(&dependency_graph.graph);
        println!("{:?}", dot);
    }
}
