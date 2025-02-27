/*#[cfg(test)]
#[allow(clippy::print_stdout)]
mod dependency_tests {

    use std::fmt::Display;

    use petgraph::dot::Dot;
    use semver::{BuildMetadata, Prerelease, Version, VersionReq};
    use ws_pkg::dependency::{DependencyGraph, Node, Step};

    #[derive(Debug)]
    struct Package {
        name: &'static str,
        version: Version,
        dependencies: Vec<Dependency>,
    }

    #[derive(Debug)]
    struct Dependency {
        name: &'static str,
        version: VersionReq,
    }

    impl Node for Package {
        type DependencyType = Dependency;

        type Identifier = String;

        fn dependencies(&self) -> &[Self::DependencyType] {
            &self.dependencies[..]
        }

        fn matches(&self, dependency: &Self::DependencyType) -> bool {
            self.name == dependency.name && dependency.version.matches(&self.version)
        }

        fn identifier(&self) -> Self::Identifier {
            self.name.to_string()
        }
    }

    impl Display for Package {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}@{}", self.name, self.version)
        }
    }

    impl Display for Dependency {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}@{}", self.name, self.version)
        }
    }

    fn build_test_graph() -> Vec<Package> {
        vec![
            Package {
                name: "@scope/bar",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            Package {
                name: "@scope/foo",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "@scope/bar",
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "@scope/baz",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "@scope/foo",
                    version: ">=1.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "@scope/charlie",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    Dependency { name: "@scope/bar", version: ">=1.0.0".parse().unwrap() },
                    Dependency { name: "@scope/foo", version: ">=1.0.0".parse().unwrap() },
                ],
            },
            Package {
                name: "@scope/major",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            Package {
                name: "@scope/tom",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    Dependency { name: "@scope/unknown", version: ">=1.0.0".parse().unwrap() },
                    Dependency { name: "@scope/remote", version: "=3.0.0".parse().unwrap() },
                ],
            },
        ]
    }

    #[test]
    fn test_graph() {
        let build = build_test_graph();
        let dependency_graph = DependencyGraph::from(&build[..]);

        println!("{}", Dot::new(&dependency_graph.graph));
    }

    #[test]
    fn test_dependencies_synchronous() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build[..]);

        assert!(!graph.is_internally_resolvable());

        for node in graph {
            match node {
                Step::Resolved(build) => println!("build: {:?}", build.name),
                Step::Unresolved(lookup) => println!("lookup: {:?}", lookup.name),
            }
        }
    }

    #[test]
    fn test_unresolved_dependencies() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build[..]);

        assert!(!graph.is_internally_resolvable());

        let unresolved_dependencies: Vec<_> =
            graph.unresolved_dependencies().map(|dep| dep.name).collect();

        assert_eq!(unresolved_dependencies, vec!["@scope/unknown", "@scope/remote"]);
    }

    #[test]
    fn test_resolved_dependencies() {
        let build = build_test_graph();
        let graph = DependencyGraph::from(&build[..]);

        assert!(!graph.is_internally_resolvable());

        let resolved_dependencies: Vec<_> =
            graph.resolved_dependencies().map(|package| package.name).collect();

        assert_eq!(
            resolved_dependencies,
            vec![
                "@scope/bar",
                "@scope/foo",
                "@scope/baz",
                "@scope/charlie",
                "@scope/major",
                "@scope/tom"
            ]
        );
    }

    #[test]
    fn test_generate_dependency_graph() {
        let _ = DependencyGraph::from(&build_test_graph()[..]);
    }

    #[test]
    fn test_internally_resolved() {
        let packages = vec![
            Package {
                name: "@scope/bar",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![],
            },
            Package {
                name: "@scope/foo",
                version: semver::Version {
                    major: 3,
                    minor: 2,
                    patch: 0,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "@scope/bar",
                    version: "=1.2.3".parse().unwrap(),
                }],
            },
            Package {
                name: "@scope/baz",
                version: semver::Version {
                    major: 11,
                    minor: 2,
                    patch: 4,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![Dependency {
                    name: "@scope/foo",
                    version: ">=3.0.0".parse().unwrap(),
                }],
            },
            Package {
                name: "@scope/tom",
                version: semver::Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::EMPTY,
                },
                dependencies: vec![
                    Dependency { name: "@scope/unknown", version: ">=1.0.0".parse().unwrap() },
                    Dependency { name: "@scope/remote", version: "=3.0.0".parse().unwrap() },
                ],
            },
        ];

        let graph = DependencyGraph::from(&packages[..]);

        for package in graph {
            match package {
                // Print out the package name so we can verify the order ourselves
                Step::Resolved(package) => println!("Building {}!", package.name),

                // Since we know that all our Packages only have internal references to each other,
                // we can safely ignore any Unresolved steps in the graph.
                //
                // If for example `second_order` required some unknown package `external_package`,
                // iterating over our graph would yield that as a Step::Unresolved *before*
                // the `second_order` package.
                Step::Unresolved(dependency) => {
                    println!("External {}@{}!", dependency.name, dependency.version);
                }
            }
        }
    }
}*/
