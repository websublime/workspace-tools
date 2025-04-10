#[cfg(test)]
mod dependency_tests {
    use std::{cmp::Ordering, collections::HashMap, rc::Rc};

    use sublime_package_tools::{
        build_dependency_graph_from_packages, ChangeType, Dependency, DependencyChange,
        DependencyFilter, DependencyRegistry, DependencyResolutionError, DependencyUpdate, Package,
        ResolutionResult, ValidationOptions,
    };

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new("react", "^17.0.0");
        assert!(dep.is_ok());

        let invalid_dep = Dependency::new("react", "not-a-version");
        assert!(invalid_dep.is_err());
    }

    #[test]
    fn test_dependency_getters() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep.name(), "react");
        assert_eq!(dep.version().to_string(), "^17.0.0");
    }

    #[test]
    fn test_dependency_to_string() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep.to_string(), "react@^17.0.0");
    }

    #[test]
    fn test_fixed_version() {
        // Without operators
        let dep = Dependency::new("react", "17.0.0").unwrap();
        assert_eq!(dep.fixed_version().unwrap().to_string(), "17.0.0");

        // With caret
        let dep_caret = Dependency::new("react", "^17.0.0").unwrap();
        assert_eq!(dep_caret.fixed_version().unwrap().to_string(), "17.0.0");

        // With tilde
        let dep_tilde = Dependency::new("react", "~17.0.0").unwrap();
        assert_eq!(dep_tilde.fixed_version().unwrap().to_string(), "17.0.0");
    }

    #[test]
    fn test_compare_versions() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();

        // Equal
        assert_eq!(dep.compare_versions("17.0.0").unwrap(), Ordering::Equal);

        // Later version
        assert_eq!(dep.compare_versions("17.0.1").unwrap(), Ordering::Less);

        // Earlier version
        assert_eq!(dep.compare_versions("16.0.0").unwrap(), Ordering::Greater);

        // Invalid version
        assert!(dep.compare_versions("not-a-version").is_err());
    }

    #[test]
    fn test_update_version() {
        let dep = Dependency::new("react", "^17.0.0").unwrap();
        assert!(dep.update_version("^18.0.0").is_ok());
        assert_eq!(dep.version().to_string(), "^18.0.0");

        assert!(dep.update_version("not-a-version").is_err());
    }

    #[test]
    fn test_version_matching() {
        // Caret ranges
        let dep_caret = Dependency::new("react", "^17.0.0").unwrap();
        assert!(dep_caret.matches("17.0.0").unwrap());
        assert!(dep_caret.matches("17.0.1").unwrap());
        assert!(dep_caret.matches("17.1.0").unwrap());
        assert!(!dep_caret.matches("18.0.0").unwrap());

        // Tilde ranges
        let dep_tilde = Dependency::new("react", "~17.0.0").unwrap();
        assert!(dep_tilde.matches("17.0.0").unwrap());
        assert!(dep_tilde.matches("17.0.9").unwrap());
        assert!(!dep_tilde.matches("17.1.0").unwrap());

        // Range with greater than
        let dep_gt = Dependency::new("react", ">17.0.0").unwrap();
        assert!(!dep_gt.matches("17.0.0").unwrap());
        assert!(dep_gt.matches("17.0.1").unwrap());

        // Range with greater than or equal
        let dep_gte = Dependency::new("react", ">=17.0.0").unwrap();
        assert!(dep_gte.matches("17.0.0").unwrap());
        assert!(dep_gte.matches("17.0.1").unwrap());

        // Invalid version to match
        assert!(dep_caret.matches("not-a-version").is_err());
    }

    #[test]
    fn test_dependency_change() {
        // Test adding a dependency
        let add_change = DependencyChange::new("react", None, Some("^17.0.0"), ChangeType::Added);
        assert_eq!(add_change.name, "react");
        assert_eq!(add_change.previous_version, None);
        assert_eq!(add_change.current_version, Some("^17.0.0".to_string()));
        assert!(!add_change.breaking); // Adding is not breaking

        // Test removing a dependency
        let remove_change =
            DependencyChange::new("react", Some("^17.0.0"), None, ChangeType::Removed);
        assert_eq!(remove_change.previous_version, Some("^17.0.0".to_string()));
        assert_eq!(remove_change.current_version, None);
        assert!(remove_change.breaking); // Removing is breaking

        // Test updating a dependency (non-breaking)
        let minor_update =
            DependencyChange::new("react", Some("^17.0.0"), Some("^17.1.0"), ChangeType::Updated);
        assert!(!minor_update.breaking);

        // Test updating a dependency (breaking)
        let major_update =
            DependencyChange::new("react", Some("^17.0.0"), Some("^18.0.0"), ChangeType::Updated);
        assert!(major_update.breaking);
    }

    #[allow(clippy::no_effect_underscore_binding)]
    #[test]
    fn test_dependency_filter() {
        // Test default filter
        let default_filter = DependencyFilter::default();
        assert!(matches!(default_filter, DependencyFilter::WithDevelopment));

        // Test other filter types
        let _prod_only = DependencyFilter::ProductionOnly;
        let _all_deps = DependencyFilter::AllDependencies;

        // These are enums without methods, so we're just verifying they exist and can be created
    }

    #[test]
    fn test_dependency_registry_operations() {
        let mut registry = DependencyRegistry::new();

        // Create initial dependency
        let dep1 = registry.get_or_create("react", "^17.0.0").unwrap();
        assert_eq!(dep1.borrow().name(), "react");

        // Get the same dependency (should return same instance)
        let dep1_again = registry.get_or_create("react", "^17.0.0").unwrap();
        assert!(Rc::ptr_eq(&dep1, &dep1_again));

        // Request same dependency with higher version (should update)
        let dep1_upgraded = registry.get_or_create("react", "^18.0.0").unwrap();
        assert!(Rc::ptr_eq(&dep1, &dep1_upgraded)); // Same instance
        assert_eq!(dep1.borrow().version().to_string(), "^18.0.0"); // But updated version

        // Request with lower version (should not downgrade)
        let _ = registry.get_or_create("react", "^17.0.0").unwrap();
        assert_eq!(dep1.borrow().version().to_string(), "^18.0.0"); // Version remains higher

        // Get non-existent dependency
        assert!(registry.get("nonexistent").is_none());

        // Test find_highest_compatible_version
        let req1 = semver::VersionReq::parse("^18.0.0").unwrap();
        let req2 = semver::VersionReq::parse("^18.0.1").unwrap();
        let highest = registry.find_highest_compatible_version("react", &[&req1, &req2]);
        assert!(!highest.is_empty());
    }

    #[test]
    fn test_dependency_resolution_result() {
        // Create a resolution result
        let mut resolved_versions = HashMap::new();
        resolved_versions.insert("react".to_string(), "18.0.0".to_string());
        resolved_versions.insert("lodash".to_string(), "4.17.21".to_string());

        let updates = vec![DependencyUpdate {
            package_name: "my-app".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^17.0.0".to_string(),
            new_version: "^18.0.0".to_string(),
        }];

        let result = ResolutionResult { resolved_versions, updates_required: updates };

        // Verify contents
        assert_eq!(result.resolved_versions.len(), 2);
        assert_eq!(result.updates_required.len(), 1);
        assert_eq!(result.updates_required[0].dependency_name, "react");

        // Apply resolution to registry
        let mut registry = DependencyRegistry::new();
        let react_dep = registry.get_or_create("react", "^17.0.0").unwrap();

        assert!(registry.apply_resolution_result(&result).is_ok());
        assert_eq!(react_dep.borrow().version().to_string(), "^18.0.0");
    }

    #[test]
    fn test_dependency_graph_integration() {
        // Create packages with dependencies
        let mut registry = DependencyRegistry::new();

        let pkg_a = Package::new_with_registry(
            "pkg-a",
            "1.0.0",
            Some(vec![("pkg-b", "^1.0.0"), ("pkg-c", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        let pkg_b = Package::new_with_registry(
            "pkg-b",
            "1.0.0",
            Some(vec![("pkg-c", "^1.0.0")]),
            &mut registry,
        )
        .unwrap();

        let pkg_c =
            Package::new_with_registry("pkg-c", "1.0.0", Some(vec![]), &mut registry).unwrap();

        let packages = vec![pkg_a.clone(), pkg_b.clone(), pkg_c.clone()];

        // Build dependency graph
        let graph = build_dependency_graph_from_packages(&packages);

        // Test graph contents
        assert_eq!(graph.graph.node_count(), 3);
        assert!(graph.node_indices.contains_key("pkg-a"));
        assert!(graph.node_indices.contains_key("pkg-b"));
        assert!(graph.node_indices.contains_key("pkg-c"));

        // Test dependency relationships
        // pkg-a depends on pkg-b and pkg-c
        let a_idx = graph.get_node_index(&"pkg-a".to_string()).unwrap();
        let a_deps: Vec<_> =
            graph.graph.neighbors_directed(a_idx, petgraph::Direction::Outgoing).collect();
        assert_eq!(a_deps.len(), 2);

        // pkg-b depends on pkg-c
        let b_idx = graph.get_node_index(&"pkg-b".to_string()).unwrap();
        let b_deps: Vec<_> =
            graph.graph.neighbors_directed(b_idx, petgraph::Direction::Outgoing).collect();
        assert_eq!(b_deps.len(), 1);

        // pkg-c has no dependencies
        let c_idx = graph.get_node_index(&"pkg-c".to_string()).unwrap();
        let c_deps: Vec<_> =
            graph.graph.neighbors_directed(c_idx, petgraph::Direction::Outgoing).collect();
        assert_eq!(c_deps.len(), 0);

        // Test validation options
        let options = ValidationOptions::new()
            .treat_unresolved_as_external(true)
            .with_internal_packages(vec!["@internal/core"]);

        assert!(options.treat_unresolved_as_external);
        assert_eq!(options.internal_packages.len(), 1);
        assert!(options.is_internal_dependency("@internal/core"));
        assert!(!options.is_internal_dependency("external-pkg"));

        // Test graph validation
        let validation_result = graph.validate_with_options(&options);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_resolution_error_handling() {
        let mut registry = DependencyRegistry::new();

        // Create a package with a dependency
        let pkg = Package::new_with_registry(
            "my-app",
            "1.0.0",
            Some(vec![("react", "^17.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Try updating a non-existent dependency
        let result = pkg.update_dependency_version("nonexistent", "^1.0.0");

        assert!(result.is_err());
        match result {
            Err(DependencyResolutionError::DependencyNotFound { name, package }) => {
                assert_eq!(name, "nonexistent");
                assert_eq!(package, "my-app");
            }
            _ => panic!("Expected DependencyNotFound error"),
        }
    }
}
