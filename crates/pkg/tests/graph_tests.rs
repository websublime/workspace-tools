#[cfg(test)]
mod graph_tests {
    use sublime_package_tools::{
        build_dependency_graph_from_packages, generate_ascii, generate_dot, Dependency, DotOptions,
        Package, Step, ValidationIssue, ValidationOptions, ValidationReport,
    };

    // Helper function to create test packages
    fn create_test_packages() -> Vec<Package> {
        let mut packages = Vec::new();
        let mut registry = sublime_package_tools::DependencyRegistry::new();

        // Package A depends on B and C
        let pkg_a = Package::new_with_registry(
            "pkg-a",
            "1.0.0",
            Some(vec![("pkg-b", "^1.0.0"), ("pkg-c", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package B depends on C
        let pkg_b = Package::new_with_registry(
            "pkg-b",
            "1.0.0",
            Some(vec![("pkg-c", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package C has no dependencies
        let pkg_c = Package::new_with_registry("pkg-c", "1.0.0", None, &mut registry).unwrap();

        packages.push(pkg_a);
        packages.push(pkg_b);
        packages.push(pkg_c);

        packages
    }

    // Helper function to create packages with circular dependencies
    fn create_circular_packages() -> Vec<Package> {
        let mut packages = Vec::new();
        let mut registry = sublime_package_tools::DependencyRegistry::new();

        // Package X depends on Y
        let pkg_x = Package::new_with_registry(
            "pkg-x",
            "1.0.0",
            Some(vec![("pkg-y", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package Y depends on Z
        let pkg_y = Package::new_with_registry(
            "pkg-y",
            "1.0.0",
            Some(vec![("pkg-z", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package Z depends on X (creating a cycle)
        let pkg_z = Package::new_with_registry(
            "pkg-z",
            "1.0.0",
            Some(vec![("pkg-x", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        packages.push(pkg_x);
        packages.push(pkg_y);
        packages.push(pkg_z);

        packages
    }

    #[test]
    fn test_build_dependency_graph() {
        let packages = create_test_packages();
        let graph = build_dependency_graph_from_packages(&packages);

        // Check graph has correct number of nodes
        assert_eq!(graph.graph.node_count(), 3);

        // Check edges (dependencies)
        assert!(graph.node_indices.contains_key("pkg-a"));
        assert!(graph.node_indices.contains_key("pkg-b"));
        assert!(graph.node_indices.contains_key("pkg-c"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let packages = create_circular_packages();
        let graph = build_dependency_graph_from_packages(&packages);

        // Detect cycles
        let graph = graph.detect_circular_dependencies();

        // Verify cycles detected
        assert!(graph.has_cycles());

        // Check cycle information
        let cycles = graph.get_cycles();
        assert_eq!(cycles.len(), 1);

        // Check cycle content
        let cycle_strings = graph.get_cycle_strings();
        assert_eq!(cycle_strings.len(), 1);

        // The cycle should contain all three packages
        assert_eq!(cycle_strings[0].len(), 3);
        assert!(cycle_strings[0].contains(&"pkg-x".to_string()));
        assert!(cycle_strings[0].contains(&"pkg-y".to_string()));
        assert!(cycle_strings[0].contains(&"pkg-z".to_string()));
    }

    #[test]
    fn test_find_external_dependencies() {
        let mut packages = create_test_packages();

        // Add a package with external dependency
        let mut registry = sublime_package_tools::DependencyRegistry::new();
        let pkg_d = Package::new_with_registry(
            "pkg-d",
            "1.0.0",
            Some(vec![("external-pkg", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();
        packages.push(pkg_d);

        let graph = build_dependency_graph_from_packages(&packages);

        // Find external dependencies
        let externals = graph.find_external_dependencies();

        assert_eq!(externals.len(), 1);
        assert!(externals.contains(&"external-pkg".to_string()));
    }

    #[test]
    fn test_find_version_conflicts() {
        let mut packages = Vec::new();
        let mut registry = sublime_package_tools::DependencyRegistry::new();

        // Package A depends on C v1.0.0
        let pkg_a = Package::new_with_registry(
            "pkg-a",
            "1.0.0",
            Some(vec![("pkg-c", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package B depends on C v2.0.0 (conflict)
        let pkg_b = Package::new_with_registry(
            "pkg-b",
            "1.0.0",
            Some(vec![("pkg-c", "^2.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Package C v1.0.0
        let pkg_c = Package::new_with_registry("pkg-c", "1.0.0", None, &mut registry).unwrap();

        packages.push(pkg_a);
        packages.push(pkg_b);
        packages.push(pkg_c);

        let graph = build_dependency_graph_from_packages(&packages);

        // Find version conflicts
        let conflicts = graph.find_version_conflicts();

        assert!(conflicts.is_some());
        let conflicts = conflicts.unwrap();
        assert!(conflicts.contains_key("pkg-c"));
        assert_eq!(conflicts["pkg-c"].len(), 2);
    }

    #[test]
    fn test_validate_dependencies() {
        // Test with external dependencies
        let mut packages = create_test_packages();

        // Add package with external dependency
        let mut registry = sublime_package_tools::DependencyRegistry::new();
        let pkg_d = Package::new_with_registry(
            "pkg-d",
            "1.0.0",
            Some(vec![("external-pkg", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();
        packages.push(pkg_d);

        let graph = build_dependency_graph_from_packages(&packages);

        // Validate with default options
        let validation = graph.validate_package_dependencies();
        assert!(validation.is_ok());
        let report = validation.unwrap();

        // Should have unresolved dependency issue
        assert!(report.has_issues());

        // With custom options
        let options = ValidationOptions::new().treat_unresolved_as_external(true);

        let validation = graph.validate_with_options(&options);
        assert!(validation.is_ok());
        let report = validation.unwrap();

        // Should not have issues now
        assert!(!report.has_issues());
    }

    #[test]
    fn test_get_dependents() {
        let packages = create_test_packages();
        let mut graph = build_dependency_graph_from_packages(&packages); // Note: mut graph

        // pkg-c has dependents pkg-a and pkg-b
        let c_dependents = graph.get_dependents(&"pkg-c".to_string()).unwrap();
        assert_eq!(c_dependents.len(), 2);
        assert!(c_dependents.contains(&"pkg-a".to_string()));
        assert!(c_dependents.contains(&"pkg-b".to_string()));

        // pkg-b has dependent pkg-a
        let b_dependents = graph.get_dependents(&"pkg-b".to_string()).unwrap();
        assert_eq!(b_dependents.len(), 1);
        assert!(b_dependents.contains(&"pkg-a".to_string()));

        // pkg-a has no dependents
        let a_dependents = graph.get_dependents(&"pkg-a".to_string()).unwrap();
        assert_eq!(a_dependents.len(), 0);
    }

    #[test]
    fn test_visualization() {
        let packages = create_test_packages();
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate DOT format
        let options = DotOptions {
            title: "Test Graph".to_string(),
            show_external: true,
            highlight_cycles: true,
        };

        let dot = generate_dot(&graph, &options);
        assert!(dot.is_ok());
        let dot_string = dot.unwrap();

        // Check DOT content
        assert!(dot_string.contains("digraph"));
        assert!(dot_string.contains("Test Graph"));
        assert!(dot_string.contains("pkg-a"));
        assert!(dot_string.contains("pkg-b"));
        assert!(dot_string.contains("pkg-c"));

        // Generate ASCII
        let ascii = generate_ascii(&graph);
        assert!(ascii.is_ok());
        let ascii_string = ascii.unwrap();

        // Check ASCII content
        assert!(!ascii_string.is_empty());
    }

    #[test]
    #[allow(clippy::match_on_vec_items)]
    fn test_validation_report() {
        // Create a validation report
        let mut report = ValidationReport::new();

        // Initially no issues
        assert!(!report.has_issues());

        // Add circular dependency issue (now known to be a warning)
        report.add_issue(ValidationIssue::CircularDependency {
            path: vec![
                "pkg-x".to_string(),
                "pkg-y".to_string(),
                "pkg-z".to_string(),
                "pkg-x".to_string(),
            ],
        });

        // Add unresolved dependency issue (now known to be critical)
        report.add_issue(ValidationIssue::UnresolvedDependency {
            name: "external-pkg".to_string(),
            version_req: "^1.0.0".to_string(),
        });

        // Add version conflict issue (now known to be a warning)
        report.add_issue(ValidationIssue::VersionConflict {
            name: "pkg-c".to_string(),
            versions: vec!["1.0.0".to_string(), "2.0.0".to_string()],
        });

        // Check if report has issues
        assert!(report.has_issues());
        assert_eq!(report.issues().len(), 3);

        // Check critical issues
        assert!(report.has_critical_issues());
        let critical = report.critical_issues();
        assert_eq!(critical.len(), 1); // Only UnresolvedDependency is critical

        match critical[0] {
            ValidationIssue::UnresolvedDependency { .. } => {} // This is expected
            _ => panic!("Expected UnresolvedDependency to be a critical issue"),
        }

        // Check warnings
        assert!(report.has_warnings());
        let warnings = report.warnings();
        assert_eq!(warnings.len(), 2); // CircularDependency and VersionConflict are warnings

        // Verify warnings contain the expected issues
        let has_circular = warnings
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::CircularDependency { .. }));
        let has_version_conflict =
            warnings.iter().any(|issue| matches!(issue, ValidationIssue::VersionConflict { .. }));

        assert!(has_circular, "CircularDependency should be a warning");
        assert!(has_version_conflict, "VersionConflict should be a warning");
    }

    #[test]
    fn test_dot_visualization() {
        let packages = create_test_packages(); // Using existing helper function
        let graph = build_dependency_graph_from_packages(&packages);

        // Test with default options
        let default_options = DotOptions::default();
        let dot_result = generate_dot(&graph, &default_options);
        assert!(dot_result.is_ok());

        let dot_string = dot_result.unwrap();
        assert!(dot_string.contains("digraph"));
        assert!(dot_string.contains("pkg-a"));
        assert!(dot_string.contains("pkg-b"));
        assert!(dot_string.contains("pkg-c"));

        // Test with custom options
        let custom_options = DotOptions {
            title: "Custom Graph".to_string(),
            show_external: false,
            highlight_cycles: true,
        };

        let custom_dot = generate_dot(&graph, &custom_options);
        assert!(custom_dot.is_ok());
        let custom_string = custom_dot.unwrap();
        assert!(custom_string.contains("Custom Graph"));
    }

    #[test]
    fn test_ascii_visualization() {
        // Test on a graph with no nodes
        let empty_packages: Vec<Package> = Vec::new();
        let empty_graph = build_dependency_graph_from_packages(&empty_packages);

        let ascii_result = generate_ascii(&empty_graph);
        assert!(ascii_result.is_ok());
        let ascii_string = ascii_result.unwrap();
        assert!(ascii_string.contains("(empty)"));

        // Test on a normal graph
        let packages = create_test_packages();
        let graph = build_dependency_graph_from_packages(&packages);

        let ascii_result = generate_ascii(&graph);
        assert!(ascii_result.is_ok());
        let ascii_string = ascii_result.unwrap();

        // Check for expected content
        assert!(ascii_string.contains("pkg-a"));
        assert!(ascii_string.contains("├──") || ascii_string.contains("└──")); // Tree branches
    }

    #[test]
    fn test_validation_issue_messages() {
        // Create different issue types and test their messages
        let circular = ValidationIssue::CircularDependency {
            path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
        };

        let unresolved = ValidationIssue::UnresolvedDependency {
            name: "missing-pkg".to_string(),
            version_req: "^1.0.0".to_string(),
        };

        let conflict = ValidationIssue::VersionConflict {
            name: "shared-pkg".to_string(),
            versions: vec!["1.0.0".to_string(), "2.0.0".to_string()],
        };

        // Check messages
        assert!(circular.message().contains("Circular dependency"));
        assert!(circular.message().contains("a -> b -> a"));

        assert!(unresolved.message().contains("Unresolved dependency"));
        assert!(unresolved.message().contains("missing-pkg"));

        assert!(conflict.message().contains("Version conflict"));
        assert!(conflict.message().contains("shared-pkg"));

        // Check criticality
        assert!(!circular.is_critical()); // Circular dependencies are warnings
        assert!(unresolved.is_critical()); // Unresolved dependencies are critical
        assert!(!conflict.is_critical()); // Version conflicts are warnings
    }

    #[test]
    fn test_graph_node_and_step() {
        // Create a test package
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();

        // Create resolved and unresolved steps with explicit type parameters
        let resolved = Step::<Package>::Resolved(&pkg);
        let unresolved = Step::<Package>::Unresolved(Dependency::new("missing", "^1.0.0").unwrap());

        // Test step methods
        assert!(resolved.is_resolved());
        assert!(!unresolved.is_resolved());

        assert!(resolved.as_resolved().is_some());
        assert!(unresolved.as_resolved().is_none());

        assert!(resolved.as_unresolved().is_none());
        assert!(unresolved.as_unresolved().is_some());

        // Test display
        assert_eq!(format!("{resolved}"), "Resolved");
        assert_eq!(format!("{unresolved}"), "Unresolved");
    }

    #[test]
    fn test_graph_iteration() {
        let packages = create_test_packages(); // Using existing helper function
        let mut graph = build_dependency_graph_from_packages(&packages);

        // Dependency-order iteration (leaf nodes first)
        // pkg-c should come first (no dependencies)
        // Then pkg-b (depends on c)
        // Then pkg-a (depends on b and c)

        let first = graph.next();
        assert!(first.is_some());
        if let Some(Step::Resolved(node)) = first {
            assert_eq!(node.name(), "pkg-c"); // pkg-c has no dependencies
        } else {
            panic!("Expected resolved node for pkg-c");
        }

        let second = graph.next();
        assert!(second.is_some());
        if let Some(Step::Resolved(node)) = second {
            assert_eq!(node.name(), "pkg-b"); // pkg-b depends only on pkg-c
        } else {
            panic!("Expected resolved node for pkg-b");
        }

        let third = graph.next();
        assert!(third.is_some());
        if let Some(Step::Resolved(node)) = third {
            assert_eq!(node.name(), "pkg-a"); // pkg-a depends on pkg-b and pkg-c
        } else {
            panic!("Expected resolved node for pkg-a");
        }

        // No more nodes
        assert!(graph.next().is_none());
    }
}
