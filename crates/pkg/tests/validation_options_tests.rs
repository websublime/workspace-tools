#[cfg(test)]
mod validation_options_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use sublime_package_tools::{
        build_dependency_graph_from_packages, Dependency, Package, ValidationIssue,
        ValidationOptions,
    };

    // Helper to create a dependency
    fn make_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    // Helper to create a package with dependencies
    fn make_package(name: &str, version: &str, dependencies: Vec<(&str, &str)>) -> Package {
        let deps = dependencies
            .into_iter()
            .map(|(name, version)| make_dependency(name, version))
            .collect();

        Package::new(name, version, Some(deps)).unwrap()
    }

    #[test]
    fn test_external_dependency_handling() {
        // Create packages with both internal and external dependencies
        let packages = vec![
            make_package(
                "my-app",
                "1.0.0",
                vec![
                    ("react", "^17.0.0"),           // External npm package
                    ("internal-lib", "^1.0.0"),     // Internal package
                    ("missing-internal", "^1.0.0"), // Missing internal package
                ],
            ),
            make_package(
                "internal-lib",
                "1.0.0",
                vec![
                    ("lodash", "^4.17.0"), // External npm package
                ],
            ),
        ];

        // Create the dependency graph
        let graph = build_dependency_graph_from_packages(&packages);

        // First, validate with default options (all unresolved are errors)
        let default_validation =
            graph.validate_with_options(&ValidationOptions::default()).unwrap();

        // With default options, all unresolved dependencies should be flagged
        let unresolved_count_default = default_validation
            .issues()
            .iter()
            .filter(|issue| matches!(issue, ValidationIssue::UnresolvedDependency { .. }))
            .count();

        // Should have 3 unresolved: react, lodash, and missing-internal
        assert_eq!(unresolved_count_default, 3);

        // Now, validate with option to treat unresolved as external
        let options = ValidationOptions::new()
            .treat_unresolved_as_external(true)
            .with_internal_packages(vec!["missing-internal", "internal-lib"]);

        let custom_validation = graph.validate_with_options(&options).unwrap();

        // With these options, only missing-internal should be flagged
        let unresolved_issues = custom_validation
            .issues()
            .iter()
            .filter_map(|issue| {
                if let ValidationIssue::UnresolvedDependency { name, .. } = issue {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(unresolved_issues.len(), 1);
        assert!(unresolved_issues.contains(&"missing-internal"));
    }
}
