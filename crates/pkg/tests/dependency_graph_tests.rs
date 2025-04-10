#[cfg(test)]
mod dependency_graph_tests {
    use std::rc::Rc;
    use std::{cell::RefCell, collections::HashMap};
    use sublime_package_tools::{
        build_dependency_graph_from_packages, Dependency, Package, Step, ValidationIssue,
        ValidationOptions,
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

        // Use the helper function to create graph
        let graph = build_dependency_graph_from_packages(&packages);

        // Verify nodes were created
        assert_eq!(graph.graph.node_count(), 3);

        // Verify edges were created
        assert_eq!(graph.graph.edge_count(), 2);

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

        let graph = build_dependency_graph_from_packages(&packages);

        // All dependencies should be resolved
        assert!(graph.is_internally_resolvable());
        assert_eq!(graph.unresolved_dependencies().count(), 0);

        // Test with an unresolvable dependency
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("pkg-c", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ];

        let graph = build_dependency_graph_from_packages(&packages);

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

        let graph = build_dependency_graph_from_packages(&packages);
        assert!(!graph.has_cycles());

        // With a cycle
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![("pkg-a", "^1.0.0")]), // Creates cycle
        ];

        let graph = build_dependency_graph_from_packages(&packages);
        let result = graph.detect_circular_dependencies();
        assert!(result.has_cycles());

        // Check cycle information
        let cycles = result.get_cycles();
        assert!(!cycles.is_empty());

        // Check cycle strings format
        let cycle_strings = result.get_cycle_strings();
        assert!(!cycle_strings.is_empty());
        let cycle = &cycle_strings[0];
        assert!(
            cycle.contains(&"pkg-a".to_string())
                && cycle.contains(&"pkg-b".to_string())
                && cycle.contains(&"pkg-c".to_string())
        );
    }

    #[test]
    fn test_missing_dependencies() {
        // Create packages with one missing dependency
        let packages = vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("missing", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ];

        let graph = build_dependency_graph_from_packages(&packages);

        // Check for external dependencies
        let missing = graph.find_external_dependencies();
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

        let graph = build_dependency_graph_from_packages(&packages);

        // Check for version conflicts
        let conflicts = graph.find_version_conflicts().unwrap();
        assert_eq!(conflicts.len(), 1);

        let versions = &conflicts["pkg-c"];
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"^1.0.0".to_string()));
        assert!(versions.contains(&"^2.0.0".to_string()));
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

        let graph = build_dependency_graph_from_packages(&packages);

        // Validate the graph
        let validation = graph.validate_package_dependencies().unwrap();

        // Should have detected issues
        assert!(validation.has_issues());
        assert!(validation.has_critical_issues());

        // Check specific issue types
        let circular = validation
            .issues()
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::CircularDependency { .. }));
        assert!(circular, "Should detect circular dependency");

        let unresolved = validation
            .issues()
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::UnresolvedDependency { .. }));
        assert!(unresolved, "Should detect unresolved dependency");

        // For version conflict, make more explicit check
        let version_conflicts = validation
            .issues()
            .iter()
            .filter_map(|issue| {
                if let ValidationIssue::VersionConflict { name, versions } = issue {
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
    fn test_validation_with_options() {
        // Create packages with external dependencies
        let packages = vec![
            make_package(
                "pkg-a",
                "1.0.0",
                vec![
                    ("pkg-b", "^1.0.0"),
                    ("external1", "^1.0.0"), // External dependency
                ],
            ),
            make_package(
                "pkg-b",
                "1.0.0",
                vec![
                    ("external2", "^1.0.0"), // Another external dependency
                ],
            ),
        ];

        let graph = build_dependency_graph_from_packages(&packages);

        // First validate without special options
        let default_validation = graph.validate_package_dependencies().unwrap();

        // Should have two unresolved dependency issues
        let unresolved_deps = default_validation
            .issues()
            .iter()
            .filter(|issue| matches!(issue, ValidationIssue::UnresolvedDependency { .. }))
            .count();
        assert_eq!(unresolved_deps, 2);

        // Now validate with custom options
        let options = ValidationOptions::new()
            .treat_unresolved_as_external(true)
            .with_internal_packages(vec!["pkg-a", "pkg-b"]);

        let custom_validation = graph.validate_with_options(&options).unwrap();

        // With treat_unresolved_as_external=true, there should be no unresolved dependency issues
        let unresolved_deps = custom_validation
            .issues()
            .iter()
            .filter(|issue| matches!(issue, ValidationIssue::UnresolvedDependency { .. }))
            .count();
        assert_eq!(unresolved_deps, 0);
    }

    #[test]
    fn test_graph_navigation() {
        // Create a simple graph
        let packages = vec![
            make_package("app", "1.0.0", vec![("lib-a", "^1.0.0"), ("lib-b", "^1.0.0")]),
            make_package("lib-a", "1.0.0", vec![("lib-c", "^1.0.0")]),
            make_package("lib-b", "1.0.0", vec![("lib-c", "^1.0.0")]),
            make_package("lib-c", "1.0.0", vec![]),
        ];

        let graph = build_dependency_graph_from_packages(&packages);

        // Test forward dependency navigation
        let app_node = graph.get_node(&"app".to_string()).unwrap();
        let app_deps = graph.resolved_dependencies()
        assert_eq!(app_deps.count(), 2);

        // Test reverse dependency lookup
        let lib_c_dependents = graph.get_dependents(&"lib-c".to_string()).unwrap();
        assert_eq!(lib_c_dependents.len(), 2);
        assert!(lib_c_dependents.contains(&"lib-a".to_string()));
        assert!(lib_c_dependents.contains(&"lib-b".to_string()));
    }
}
