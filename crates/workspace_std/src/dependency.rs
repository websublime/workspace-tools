use petgraph::{stable_graph::StableDiGraph, Direction};
use std::fmt::Display;

/// Must be implemented by the type you wish
pub trait Node {
    /// Encodes a dependency relationship. In a Package Manager dependency graph for instance, this might be a (package name, version) tuple.
    /// It might also just be the exact same type as the one that implements the Node trait, in which case `Node::matches` can be implemented through simple equality.
    type DependencyType;

    /// Returns a slice of dependencies for this Node
    fn dependencies(&self) -> &[Self::DependencyType];

    /// Returns true if the `dependency` can be met by us.
    fn matches(&self, dependency: &Self::DependencyType) -> bool;
}

/// Wrapper around dependency graph nodes.
/// Since a graph might have dependencies that cannot be resolved internally,
/// this wrapper is necessary to differentiate between internally resolved and
/// externally (unresolved) dependencies.
/// An Unresolved dependency does not necessarily mean that it *cannot* be resolved,
/// only that no Node within the graph fulfills it.
#[derive(Debug, Clone)]
pub enum Step<'a, N: Node> {
    Resolved(&'a N),
    Unresolved(&'a N::DependencyType),
}

impl<'a, N: Node> Step<'a, N> {
    pub fn is_resolved(&self) -> bool {
        match self {
            Step::Resolved(_) => true,
            Step::Unresolved(_) => false,
        }
    }

    pub fn as_resolved(&self) -> Option<&N> {
        match self {
            Step::Resolved(node) => Some(node),
            Step::Unresolved(_) => None,
        }
    }

    pub fn as_unresolved(&self) -> Option<&N::DependencyType> {
        match self {
            Step::Resolved(_) => None,
            Step::Unresolved(dependency) => Some(dependency),
        }
    }
}

impl<'a, N: Node> Display for Step<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Resolved(_node) => write!(f, "Resolved"),
            Step::Unresolved(_dependency) => write!(f, "Unresolved"),
        }
    }
}

/// The [`DependencyGraph`] structure builds an internal [Directed Graph](`petgraph::stable_graph::StableDiGraph`), which can then be traversed
/// in an order which ensures that dependent Nodes are visited before their parents.
#[derive(Debug, Clone)]
pub struct DependencyGraph<'a, N: Node> {
    pub graph: StableDiGraph<Step<'a, N>, &'a N::DependencyType>,
}

/// The only way to build a [`DependencyGraph`] is from a slice of objects implementing [`Node`].
/// The graph references the original items, meaning the objects cannot be modified while
/// the [`DependencyGraph`] holds a reference to them.
impl<'a, N> From<&'a [N]> for DependencyGraph<'a, N>
where
    N: Node,
{
    fn from(nodes: &'a [N]) -> Self {
        let mut graph = StableDiGraph::<Step<'a, N>, &'a N::DependencyType>::new();

        // Insert the input nodes into the graph, and record their positions.
        // We'll be adding the edges next, and filling in any unresolved
        // steps we find along the way.
        let nodes: Vec<(_, _)> =
            nodes.iter().map(|node| (node, graph.add_node(Step::Resolved(node)))).collect();

        for (node, index) in &nodes {
            for dependency in node.dependencies() {
                // Check to see if we can resolve this dependency internally.
                if let Some((_, dependent)) = nodes.iter().find(|(dep, _)| dep.matches(dependency))
                {
                    // If we can, just add an edge between the two nodes.
                    graph.add_edge(*index, *dependent, dependency);
                } else {
                    // If not, create a new "Unresolved" node, and create an edge to that.
                    let unresolved = graph.add_node(Step::Unresolved(dependency));
                    graph.add_edge(*index, unresolved, dependency);
                }
            }
        }

        Self { graph }
    }
}

impl<'a, N> DependencyGraph<'a, N>
where
    N: Node,
{
    /// True if all graph [`Node`]s have only references to other internal [`Node`]s.
    /// That is, there are no unresolved dependencies between nodes.
    pub fn is_internally_resolvable(&self) -> bool {
        self.graph.node_weights().all(Step::is_resolved)
    }

    /// Get an iterator over unresolved dependencies, without traversing the whole graph.
    /// Useful for doing pre-validation or pre-fetching of external dependencies before
    /// starting to resolve internal dependencies.
    pub fn unresolved_dependencies(&self) -> impl Iterator<Item = &N::DependencyType> {
        self.graph.node_weights().filter_map(Step::as_unresolved)
    }

    pub fn resolved_dependencies(&self) -> impl Iterator<Item = &N> {
        self.graph.node_weights().filter_map(Step::as_resolved)
    }
}

/// Iterate over the DependencyGraph in an order which ensures dependencies are resolved before each Node is visited.
/// Note: If a `Step::Unresolved` node is returned, it is the caller's responsibility to ensure the dependency is resolved
/// before continuing.
impl<'a, N> Iterator for DependencyGraph<'a, N>
where
    N: Node,
{
    type Item = Step<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        // Returns the first node, which does not have any Outgoing
        // edges, which means it is terminal.
        for index in self.graph.node_indices().rev() {
            if self.graph.neighbors_directed(index, Direction::Outgoing).count() == 0 {
                return self.graph.remove_node(index);
            }
        }

        None
    }
}

#[cfg(test)]
#[allow(clippy::print_stdout)]
mod tests {

    use std::fmt::Display;

    use super::{DependencyGraph, Node, Step};
    use petgraph::dot::Dot;
    use semver::{BuildMetadata, Prerelease, Version, VersionReq};

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

        fn dependencies(&self) -> &[Self::DependencyType] {
            &self.dependencies[..]
        }

        fn matches(&self, dependency: &Self::DependencyType) -> bool {
            self.name == dependency.name && dependency.version.matches(&self.version)
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
}
