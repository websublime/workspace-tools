#[cfg(test)]
mod graph_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use sublime_package_tools::{
        build_dependency_graph_from_packages, generate_ascii, generate_dot, Dependency, DotOptions,
        Package, ValidationIssue, ValidationOptions, ValidationReport,
    };

    // Helper function to create test packages
    fn create_test_packages() -> Vec<Package> {
        let mut packages = Vec::new();

        // Package A depends on B and C
        let pkg_a = Package::new(
            "pkg-a",
            "1.0.0",
            Some(vec![
                Rc::new(RefCell::new(Dependency::new("pkg-b", "^1.0.0").unwrap())),
                Rc::new(RefCell::new(Dependency::new("pkg-c", "^1.0.0").unwrap())),
            ]),
        )
        .unwrap();

        // Package B depends on C
        let pkg_b = Package::new(
            "pkg-b",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-c", "^1.0.0").unwrap()))]),
        )
        .unwrap();

        // Package C has no dependencies
        let pkg_c = Package::new("pkg-c", "1.0.0", None).unwrap();

        packages.push(pkg_a);
        packages.push(pkg_b);
        packages.push(pkg_c);

        packages
    }

    // Helper function to create packages with circular dependencies
    fn create_circular_packages() -> Vec<Package> {
        let mut packages = Vec::new();

        // Package X depends on Y
        let pkg_x = Package::new(
            "pkg-x",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-y", "^1.0.0").unwrap()))]),
        )
        .unwrap();

        // Package Y depends on Z
        let pkg_y = Package::new(
            "pkg-y",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-z", "^1.0.0").unwrap()))]),
        )
        .unwrap();

        // Package Z depends on X (creating a cycle)
        let pkg_z = Package::new(
            "pkg-z",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-x", "^1.0.0").unwrap()))]),
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
        let pkg_d = Package::new(
            "pkg-d",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("external-pkg", "^1.0.0").unwrap()))]),
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

        // Package A depends on C v1.0.0
        let pkg_a = Package::new(
            "pkg-a",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-c", "^1.0.0").unwrap()))]),
        )
        .unwrap();

        // Package B depends on C v2.0.0 (conflict)
        let pkg_b = Package::new(
            "pkg-b",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("pkg-c", "^2.0.0").unwrap()))]),
        )
        .unwrap();

        // Package C v1.0.0
        let pkg_c = Package::new("pkg-c", "1.0.0", None).unwrap();

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
        let pkg_d = Package::new(
            "pkg-d",
            "1.0.0",
            Some(vec![Rc::new(RefCell::new(Dependency::new("external-pkg", "^1.0.0").unwrap()))]),
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
}
