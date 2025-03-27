#[cfg(test)]
mod dependency_graph_tests {
    use std::rc::Rc;
    use std::{cell::RefCell, collections::HashMap};
    use sublime_package_tools::{
        Dependency, DependencyGraph, DependencyResolutionError, Package, Step,
    };

    // Helper function to create a simple dependency
    fn make_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    // Helper function to create a package with dependencies
    fn make_package(name: &str, version: &str, dependencies: Vec<(&str, &str)>) -> Package {
        let deps = dependencies
            .into_iter()
            .map(|(name, version)| make_dependency(name, version))
            .collect();

        Package::new(name, version, Some(deps)).unwrap()
    }

    #[test]
    fn test_graph_creation() {
        // Create packages with dependencies
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![]),
        ];

        // Create graph from packages
        let graph = DependencyGraph::from(packages.as_slice());

        // Verify nodes were created
        assert_eq!(graph.graph.node_count(), 3);

        // Verify edges were created
        assert_eq!(graph.graph.edge_count(), 2);

        // Verify node indices
        assert!(graph.get_node_index(&"pkg-a".to_string()).is_some());
        assert!(graph.get_node_index(&"pkg-b".to_string()).is_some());
        assert!(graph.get_node_index(&"pkg-c".to_string()).is_some());

        // Verify node access
        let node_a = graph.get_node(&"pkg-a".to_string()).unwrap();
        match node_a {
            Step::Resolved(pkg) => {
                assert_eq!(pkg.name(), "pkg-a");
                assert_eq!(pkg.version_str(), "1.0.0");
            }
            Step::Unresolved(_) => panic!("Expected resolved node"),
        }
    }

    #[test]
    fn test_internal_resolution() {
        // All dependencies resolvable within the packages
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ];

        let graph = DependencyGraph::from(packages.as_slice());

        // All dependencies should be resolved
        assert!(graph.is_internally_resolvable());
        assert_eq!(graph.unresolved_dependencies().count(), 0);

        // Test with an unresolvable dependency
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("pkg-c", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ];

        let graph = DependencyGraph::from(packages.as_slice());

        // Should have unresolved dependency
        assert!(!graph.is_internally_resolvable());
        assert_eq!(graph.unresolved_dependencies().count(), 1);

        // Verify the unresolved dependency
        let unresolved = graph.unresolved_dependencies().next().unwrap();
        assert_eq!(unresolved.name(), "pkg-c");
    }

    #[test]
    fn test_cycle_detection() {
        // No cycles
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![]),
        ];

        let graph = DependencyGraph::from(packages.as_slice());
        assert!(graph.detect_circular_dependencies().is_ok());

        // With a cycle
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![("pkg-a", "^1.0.0")]), // Creates cycle
        ];

        let graph = DependencyGraph::from(packages.as_slice());
        let result = graph.detect_circular_dependencies();
        assert!(result.is_err());

        match result {
            Err(DependencyResolutionError::CircularDependency { path }) => {
                assert!(path.contains(&"pkg-a".to_string()));
            }
            _ => panic!("Expected CircularDependency error"),
        }
    }

    #[test]
    fn test_missing_dependencies() {
        // Create packages with one missing dependency
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("missing", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ];

        let graph = DependencyGraph::from(packages.as_slice());

        // Check for missing dependencies
        let missing = graph.find_missing_dependencies();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "missing");
    }

    #[test]
    fn test_version_conflicts() {
        // Create packages with conflicting version requirements
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^2.0.0")]), // Conflict!
            make_package("pkg-c", "1.5.0", vec![]),
        ];

        let graph = DependencyGraph::from(packages.as_slice());

        // Check for version conflicts
        let conflicts = graph.find_version_conflicts().unwrap();
        assert_eq!(conflicts.len(), 1);

        let versions = &conflicts["pkg-c"];
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
    }

    #[test]
    fn test_dependency_validation() {
        // Create packages with various issues
        let packages = vec![
            make_package(
                "pkg-a",
                "1.0.0",
                vec![
                    ("pkg-b", "^1.0.0"),
                    ("pkg-c", "^1.0.0"), // This requires pkg-c at ^1.0.0
                    ("missing", "^1.0.0"),
                ],
            ),
            make_package(
                "pkg-b",
                "1.0.0",
                vec![
                    ("pkg-a", "^1.0.0"), // Creates cycle
                    ("pkg-c", "^2.0.0"), // This requires pkg-c at ^2.0.0, creating a conflict
                ],
            ),
            make_package("pkg-c", "1.5.0", vec![]), // This version satisfies ^1.0.0 but not ^2.0.0
        ];

        let graph = DependencyGraph::from(packages.as_slice());

        // Validate the graph
        let validation = graph.validate_package_dependencies().unwrap();

        // Should have detected issues
        assert!(validation.has_issues());
        assert!(validation.has_critical_issues());

        // Check specific issue types
        let circular = validation.issues().iter().any(|issue| {
            matches!(issue, sublime_package_tools::ValidationIssue::CircularDependency { .. })
        });
        assert!(circular, "Should detect circular dependency");

        let unresolved = validation.issues().iter().any(|issue| {
            matches!(issue, sublime_package_tools::ValidationIssue::UnresolvedDependency { .. })
        });
        assert!(unresolved, "Should detect unresolved dependency");

        // For version conflict, make more explicit check
        let version_conflicts = validation
            .issues()
            .iter()
            .filter_map(|issue| {
                if let sublime_package_tools::ValidationIssue::VersionConflict { name, versions } =
                    issue
                {
                    Some((name.as_str(), versions.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<&str, Vec<String>>>();

        assert!(!version_conflicts.is_empty(), "Should detect at least one version conflict");
        assert!(version_conflicts.contains_key("pkg-c"), "Should detect conflict for pkg-c");

        if let Some(versions) = version_conflicts.get("pkg-c") {
            assert_eq!(versions.len(), 2, "Should have two conflicting versions for pkg-c");
        }
    }

    #[test]
    fn test_graph_iteration() {
        // Create a simple dependency graph
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![]),
        ];

        // Graph iteration should follow dependency order (leaf nodes first)
        let mut graph = DependencyGraph::from(packages.as_slice());

        // First should be pkg-c (no dependencies)
        let step = graph.next().unwrap();
        match step {
            Step::Resolved(pkg) => assert_eq!(pkg.name(), "pkg-c"),
            Step::Unresolved(_) => panic!("Expected resolved node"),
        }

        // Next should be pkg-b (depends on pkg-c)
        let step = graph.next().unwrap();
        match step {
            Step::Resolved(pkg) => assert_eq!(pkg.name(), "pkg-b"),
            Step::Unresolved(_) => panic!("Expected resolved node"),
        }

        // Last should be pkg-a (depends on pkg-b)
        let step = graph.next().unwrap();
        match step {
            Step::Resolved(pkg) => assert_eq!(pkg.name(), "pkg-a"),
            Step::Unresolved(_) => panic!("Expected resolved node"),
        }

        // Should be no more nodes
        assert!(graph.next().is_none());
    }
}
