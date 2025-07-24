//! # Tests for Hash Tree Module
//!
//! This module contains comprehensive tests for the DependencyHashTree implementation,
//! ensuring enterprise-grade reliability and correctness.

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod hash_tree_tests {
    use super::super::hash_tree::{
        CircularDependencyType, CycleSeverity, DependencyHashTree, DependencyReference,
        PackageLocation, PackageNode,
    };
    use crate::context::dependency_source::{DependencySource, GitReference, WorkspaceConstraint};
    use crate::{Graph, Package, Registry};
    use semver::VersionReq;
    use std::path::PathBuf;

    #[test]
    fn test_new_empty_tree() {
        let tree = DependencyHashTree::new();
        assert!(tree.packages.is_empty());
        assert!(tree.dependency_graph.is_empty());
        assert!(tree.dependent_graph.is_empty());
    }

    #[test]
    fn test_add_single_package() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "test-package".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        assert_eq!(tree.packages.len(), 1);
        assert!(tree.packages.contains_key("test-package"));

        let package = &tree.packages["test-package"];
        assert_eq!(package.name, "test-package");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.location, PackageLocation::Internal);
        assert!(package.depends_on.is_empty());
        assert!(package.dependency_of.is_empty());
    }

    #[test]
    fn test_add_package_with_dependencies() {
        let mut tree = DependencyHashTree::new();

        // Add dependency first
        tree.add_package(
            "dependency".to_string(),
            "2.0.0".to_string(),
            PackageLocation::External,
            vec![],
        );

        // Add package that depends on it
        let dep_ref = DependencyReference::new(
            "dependency".to_string(),
            DependencySource::Registry {
                name: "dependency".to_string(),
                version_req: VersionReq::parse("^2.0.0").unwrap(),
            },
        );

        tree.add_package(
            "main-package".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![dep_ref],
        );

        // Verify dependency graph
        let deps = &tree.dependency_graph["main-package"];
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], "dependency");

        // Verify dependent graph
        let dependents = &tree.dependent_graph["dependency"];
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], "main-package");

        // Verify package dependency_of field
        let dep_package = &tree.packages["dependency"];
        assert_eq!(dep_package.dependency_of.len(), 1);
        assert_eq!(dep_package.dependency_of[0], "main-package");
    }

    #[test]
    fn test_find_dependents() {
        let mut tree = DependencyHashTree::new();

        // Create dependency chain: app -> utils, lib -> utils
        tree.add_package(
            "utils".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Registry {
                    name: "utils".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "lib".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Workspace {
                    name: "utils".to_string(),
                    constraint: WorkspaceConstraint::Any,
                },
            )],
        );

        let dependents = tree.find_dependents("utils");
        assert_eq!(dependents.len(), 2);

        let dependent_names: Vec<&str> = dependents.iter().map(|p| p.name.as_str()).collect();
        assert!(dependent_names.contains(&"app"));
        assert!(dependent_names.contains(&"lib"));
    }

    #[test]
    fn test_find_dependents_empty() {
        let mut tree = DependencyHashTree::new();
        tree.add_package(
            "standalone".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        let dependents = tree.find_dependents("standalone");
        assert!(dependents.is_empty());
    }

    #[test]
    fn test_find_dependency_path_direct() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "target".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );
        tree.add_package(
            "source".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "target".to_string(),
                DependencySource::Registry {
                    name: "target".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let path = tree.find_dependency_path("source", "target");
        assert_eq!(path, Some(vec!["source".to_string(), "target".to_string()]));
    }

    #[test]
    fn test_find_dependency_path_transitive() {
        let mut tree = DependencyHashTree::new();

        // Create chain: app -> middleware -> utils
        tree.add_package(
            "utils".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        tree.add_package(
            "middleware".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::File {
                    name: "utils".to_string(),
                    path: PathBuf::from("../utils"),
                },
            )],
        );

        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "middleware".to_string(),
                DependencySource::Registry {
                    name: "middleware".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let path = tree.find_dependency_path("app", "utils");
        assert_eq!(
            path,
            Some(vec!["app".to_string(), "middleware".to_string(), "utils".to_string()])
        );
    }

    #[test]
    fn test_find_dependency_path_not_found() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "isolated-a".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );
        tree.add_package(
            "isolated-b".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        let path = tree.find_dependency_path("isolated-a", "isolated-b");
        assert_eq!(path, None);
    }

    #[test]
    fn test_find_dependency_path_same_package() {
        let mut tree = DependencyHashTree::new();
        tree.add_package(
            "package".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        let path = tree.find_dependency_path("package", "package");
        assert_eq!(path, Some(vec!["package".to_string()]));
    }

    #[test]
    fn test_affected_by_change_single() {
        let mut tree = DependencyHashTree::new();

        // Create chain: app -> utils
        tree.add_package(
            "utils".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );
        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Registry {
                    name: "utils".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let affected = tree.affected_by_change(&["utils".to_string()]);
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&"app".to_string()));
    }

    #[test]
    fn test_affected_by_change_transitive() {
        let mut tree = DependencyHashTree::new();

        // Create chain: frontend -> backend -> database
        tree.add_package(
            "database".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        tree.add_package(
            "backend".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "database".to_string(),
                DependencySource::Registry {
                    name: "database".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "frontend".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "backend".to_string(),
                DependencySource::Registry {
                    name: "backend".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let affected = tree.affected_by_change(&["database".to_string()]);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&"backend".to_string()));
        assert!(affected.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_affected_by_change_multiple_roots() {
        let mut tree = DependencyHashTree::new();

        // Create diamond pattern: app1,app2 -> shared -> core
        tree.add_package(
            "core".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        tree.add_package(
            "shared".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "core".to_string(),
                DependencySource::Registry {
                    name: "core".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "app1".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "shared".to_string(),
                DependencySource::Registry {
                    name: "shared".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "app2".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "shared".to_string(),
                DependencySource::Registry {
                    name: "shared".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let affected = tree.affected_by_change(&["core".to_string()]);
        assert_eq!(affected.len(), 3);
        assert!(affected.contains(&"shared".to_string()));
        assert!(affected.contains(&"app1".to_string()));
        assert!(affected.contains(&"app2".to_string()));
    }

    #[test]
    fn test_detect_circular_deps_simple() {
        let mut tree = DependencyHashTree::new();

        // Create circular dependency: pkg-a -> pkg-b -> pkg-a
        tree.add_package(
            "pkg-a".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "pkg-b".to_string(),
                DependencySource::Registry {
                    name: "pkg-b".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "pkg-b".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "pkg-a".to_string(),
                DependencySource::Registry {
                    name: "pkg-a".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let cycles = tree.detect_circular_deps();
        assert!(!cycles.is_empty());

        // Should detect the cycle
        let cycle = &cycles[0];
        assert_eq!(cycle.cycle_type, CircularDependencyType::ProductionDependencies);
        assert_eq!(cycle.severity, CycleSeverity::Error);
        assert!(cycle.path.contains(&"pkg-a".to_string()));
        assert!(cycle.path.contains(&"pkg-b".to_string()));
    }

    #[test]
    fn test_detect_circular_deps_no_cycles() {
        let mut tree = DependencyHashTree::new();

        // Create linear dependency: app -> lib -> utils
        tree.add_package(
            "utils".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );

        tree.add_package(
            "lib".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Registry {
                    name: "utils".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "lib".to_string(),
                DependencySource::Registry {
                    name: "lib".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let cycles = tree.detect_circular_deps();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_render_ascii_tree() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Registry {
                    name: "utils".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let ascii = tree.render_ascii_tree();
        assert!(ascii.contains("Dependency Tree:"));
        assert!(ascii.contains("app v1.0.0"));
        assert!(ascii.contains("utils"));
    }

    #[test]
    fn test_render_dot_graph() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![DependencyReference::new(
                "utils".to_string(),
                DependencySource::Registry {
                    name: "utils".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            )],
        );

        let dot = tree.render_dot_graph();
        assert!(dot.starts_with("digraph DependencyGraph"));
        assert!(dot.contains("app"));
        assert!(dot.contains("utils"));
        assert!(dot.contains("->"));
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_package_locations_in_dot() {
        let mut tree = DependencyHashTree::new();

        tree.add_package(
            "internal".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![],
        );
        tree.add_package(
            "external".to_string(),
            "2.0.0".to_string(),
            PackageLocation::External,
            vec![],
        );

        let dot = tree.render_dot_graph();
        assert!(dot.contains("lightblue")); // Internal packages
        assert!(dot.contains("lightgray")); // External packages
    }

    #[test]
    fn test_dependency_reference_creation() {
        let dep_ref = DependencyReference::new(
            "test-package".to_string(),
            DependencySource::Git {
                name: "test-package".to_string(),
                repo: "https://github.com/user/test-package.git".to_string(),
                reference: GitReference::Branch("main".to_string()),
            },
        );

        assert_eq!(dep_ref.name, "test-package");
        match dep_ref.source {
            DependencySource::Git { repo, .. } => {
                assert_eq!(repo, "https://github.com/user/test-package.git");
            }
            _ => panic!("Expected Git dependency source"),
        }
    }

    #[test]
    fn test_package_node_creation() {
        let deps = vec![
            DependencyReference::new(
                "dep1".to_string(),
                DependencySource::Registry {
                    name: "dep1".to_string(),
                    version_req: VersionReq::parse("^1.0.0").unwrap(),
                },
            ),
            DependencyReference::new(
                "dep2".to_string(),
                DependencySource::Workspace {
                    name: "dep2".to_string(),
                    constraint: WorkspaceConstraint::Any,
                },
            ),
        ];

        let node = PackageNode::new(
            "test-package".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            deps,
        );

        assert_eq!(node.name, "test-package");
        assert_eq!(node.version, "1.0.0");
        assert_eq!(node.location, PackageLocation::Internal);
        assert_eq!(node.depends_on.len(), 2);
        assert!(node.dependency_of.is_empty());
    }

    #[test]
    fn test_complex_dependency_sources() {
        let mut tree = DependencyHashTree::new();

        // Test various dependency source types
        tree.add_package(
            "app".to_string(),
            "1.0.0".to_string(),
            PackageLocation::Internal,
            vec![
                DependencyReference::new(
                    "registry-dep".to_string(),
                    DependencySource::Registry {
                        name: "registry-dep".to_string(),
                        version_req: VersionReq::parse("^1.0.0").unwrap(),
                    },
                ),
                DependencyReference::new(
                    "workspace-dep".to_string(),
                    DependencySource::Workspace {
                        name: "workspace-dep".to_string(),
                        constraint: WorkspaceConstraint::Compatible,
                    },
                ),
                DependencyReference::new(
                    "file-dep".to_string(),
                    DependencySource::File {
                        name: "file-dep".to_string(),
                        path: PathBuf::from("../local-package"),
                    },
                ),
                DependencyReference::new(
                    "git-dep".to_string(),
                    DependencySource::Git {
                        name: "git-dep".to_string(),
                        repo: "https://github.com/user/repo.git".to_string(),
                        reference: GitReference::Tag("v1.0.0".to_string()),
                    },
                ),
            ],
        );

        assert_eq!(tree.packages.len(), 1);
        let app_package = &tree.packages["app"];
        assert_eq!(app_package.depends_on.len(), 4);

        // Verify dependency graph
        let deps = &tree.dependency_graph["app"];
        assert_eq!(deps.len(), 4);
        assert!(deps.contains(&"registry-dep".to_string()));
        assert!(deps.contains(&"workspace-dep".to_string()));
        assert!(deps.contains(&"file-dep".to_string()));
        assert!(deps.contains(&"git-dep".to_string()));
    }

    #[test]
    fn test_default_trait() {
        let tree = DependencyHashTree::default();
        assert!(tree.packages.is_empty());
        assert!(tree.dependency_graph.is_empty());
        assert!(tree.dependent_graph.is_empty());
    }

    // Integration Tests - Graph to HashTree Conversion
    #[test]
    fn test_graph_to_hash_tree_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry(
                "app",
                "1.0.0",
                Some(vec![("utils", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry("utils", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Verify conversion
        assert_eq!(hash_tree.packages.len(), 2);
        assert!(hash_tree.packages.contains_key("app"));
        assert!(hash_tree.packages.contains_key("utils"));

        // Verify dependency relationships
        let app_deps = &hash_tree.dependency_graph["app"];
        assert_eq!(app_deps.len(), 1);
        assert_eq!(app_deps[0], "utils");

        // Verify dependent relationships
        let utils_dependents = &hash_tree.dependent_graph["utils"];
        assert_eq!(utils_dependents.len(), 1);
        assert_eq!(utils_dependents[0], "app");
    }

    #[test]
    fn test_hash_tree_querying_after_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry(
                "frontend",
                "1.0.0",
                Some(vec![("backend", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry(
                "backend",
                "1.0.0",
                Some(vec![("database", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry("database", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Test querying capabilities
        let dependents = hash_tree.find_dependents("database");
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].name, "backend");

        // Test dependency path finding
        let path = hash_tree.find_dependency_path("frontend", "database");
        assert_eq!(
            path,
            Some(vec!["frontend".to_string(), "backend".to_string(), "database".to_string()])
        );

        // Test affected packages
        let affected = hash_tree.affected_by_change(&["database".to_string()]);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&"backend".to_string()));
        assert!(affected.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection_integration() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry(
                "pkg-a",
                "1.0.0",
                Some(vec![("pkg-b", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry(
                "pkg-b",
                "1.0.0",
                Some(vec![("pkg-a", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Both traditional and hash tree should detect the cycle
        assert!(graph.has_cycles());

        let hash_tree_cycles = hash_tree.detect_circular_deps();
        assert!(!hash_tree_cycles.is_empty());

        let cycle = &hash_tree_cycles[0];
        assert!(cycle.path.contains(&"pkg-a".to_string()));
        assert!(cycle.path.contains(&"pkg-b".to_string()));
    }

    #[test]
    fn test_complex_dependency_graph_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry(
                "web-app",
                "1.0.0",
                Some(vec![("shared-ui", "^1.0.0"), ("api-client", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry(
                "mobile-app",
                "1.0.0",
                Some(vec![("shared-ui", "^1.0.0"), ("api-client", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry(
                "shared-ui",
                "1.0.0",
                Some(vec![("design-tokens", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry(
                "api-client",
                "1.0.0",
                Some(vec![("shared-utils", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry("design-tokens", "1.0.0", Some(vec![]), &mut registry)
                .unwrap(),
            Package::new_with_registry("shared-utils", "1.0.0", Some(vec![]), &mut registry)
                .unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Verify all packages are converted
        assert_eq!(hash_tree.packages.len(), 6);

        // Test complex dependency analysis
        let design_token_dependents = hash_tree.find_dependents("design-tokens");
        assert_eq!(design_token_dependents.len(), 1);
        assert_eq!(design_token_dependents[0].name, "shared-ui");

        // Test transitive affected packages
        let affected_by_design_tokens =
            hash_tree.affected_by_change(&["design-tokens".to_string()]);
        assert_eq!(affected_by_design_tokens.len(), 3); // shared-ui, web-app, mobile-app
        assert!(affected_by_design_tokens.contains(&"shared-ui".to_string()));
        assert!(affected_by_design_tokens.contains(&"web-app".to_string()));
        assert!(affected_by_design_tokens.contains(&"mobile-app".to_string()));

        // Test multiple root changes
        let affected_by_utils = hash_tree.affected_by_change(&["shared-utils".to_string()]);
        assert_eq!(affected_by_utils.len(), 3); // api-client, web-app, mobile-app
        assert!(affected_by_utils.contains(&"api-client".to_string()));
        assert!(affected_by_utils.contains(&"web-app".to_string()));
        assert!(affected_by_utils.contains(&"mobile-app".to_string()));
    }

    #[test]
    fn test_visualization_integration() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry(
                "app",
                "1.0.0",
                Some(vec![("utils", "^1.0.0")]),
                &mut registry,
            )
            .unwrap(),
            Package::new_with_registry("utils", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Test ASCII visualization
        let ascii = hash_tree.render_ascii_tree();
        assert!(ascii.contains("Dependency Tree:"));
        assert!(ascii.contains("app"));
        assert!(ascii.contains("utils"));

        // Test DOT visualization
        let dot = hash_tree.render_dot_graph();
        assert!(dot.starts_with("digraph DependencyGraph"));
        assert!(dot.contains("app"));
        assert!(dot.contains("utils"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_empty_graph_conversion() {
        let packages: Vec<Package> = vec![];
        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Empty graph should result in empty hash tree
        assert!(hash_tree.packages.is_empty());
        assert!(hash_tree.dependency_graph.is_empty());
        assert!(hash_tree.dependent_graph.is_empty());
    }

    #[test]
    fn test_single_package_conversion() {
        let mut registry = Registry::new();
        let packages =
            vec![Package::new_with_registry("standalone", "1.0.0", Some(vec![]), &mut registry)
                .unwrap()];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Single package should be properly converted
        assert_eq!(hash_tree.packages.len(), 1);
        assert!(hash_tree.packages.contains_key("standalone"));

        let package = &hash_tree.packages["standalone"];
        assert_eq!(package.name, "standalone");
        // Note: version is "unknown" because Node trait doesn't expose version info
        assert_eq!(package.version, "unknown");
        assert!(package.depends_on.is_empty());
        assert!(package.dependency_of.is_empty());
    }
}
