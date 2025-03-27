#[cfg(test)]
mod package_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use sublime_package_tools::{
        Dependency, DependencyRegistry, DependencyResolutionError, Node, Package, ResolutionResult,
        VersionError,
    };

    // Helper function to create a dependency
    fn make_dependency(name: &str, version: &str) -> Result<Rc<RefCell<Dependency>>, VersionError> {
        Ok(Rc::new(RefCell::new(Dependency::new(name, version)?)))
    }

    #[test]
    fn test_package_creation() {
        // Test basic package creation
        let pkg = Package::new("test-pkg", "1.0.0", None);
        assert!(pkg.is_ok());

        let pkg = pkg.unwrap();
        assert_eq!(pkg.name(), "test-pkg");
        assert_eq!(pkg.version_str(), "1.0.0");
        assert!(pkg.dependencies().is_empty());

        // Test with dependencies
        let deps = vec![
            make_dependency("dep1", "^1.0.0").unwrap(),
            make_dependency("dep2", "~2.0.0").unwrap(),
        ];

        let pkg = Package::new("test-pkg", "1.0.0", Some(deps));
        assert!(pkg.is_ok());

        let pkg = pkg.unwrap();
        assert_eq!(pkg.dependencies().len(), 2);
        assert_eq!(pkg.dependencies()[0].borrow().name(), "dep1");
        assert_eq!(pkg.dependencies()[1].borrow().name(), "dep2");

        // Test with invalid version
        let pkg = Package::new("test-pkg", "invalid", None);
        assert!(pkg.is_err());
        assert!(matches!(pkg.unwrap_err(), VersionError::Parse { .. }));
    }

    #[test]
    fn test_package_with_registry() {
        let mut registry = DependencyRegistry::new();

        // Create package using registry
        let pkg = Package::new_with_registry(
            "test-pkg",
            "1.0.0",
            Some(vec![("dep1", "^1.0.0"), ("dep2", "^2.0.0")]),
            &mut registry,
        );

        assert!(pkg.is_ok());
        let pkg = pkg.unwrap();

        assert_eq!(pkg.name(), "test-pkg");
        assert_eq!(pkg.version_str(), "1.0.0");
        assert_eq!(pkg.dependencies().len(), 2);

        // Verify registry has the dependencies
        let dep1 = registry.get("dep1");
        assert!(dep1.is_some());
        assert_eq!(dep1.unwrap().borrow().name(), "dep1");

        // Create another package with a shared dependency
        let pkg2 = Package::new_with_registry(
            "pkg2",
            "1.0.0",
            Some(vec![
                ("dep1", "^1.0.0"), // Same dependency as first package
                ("dep3", "^3.0.0"), // New dependency
            ]),
            &mut registry,
        );

        assert!(pkg2.is_ok());
        pkg2.unwrap();

        // Check that registry now has three dependencies
        assert!(registry.get("dep1").is_some());
        assert!(registry.get("dep2").is_some());
        assert!(registry.get("dep3").is_some());
    }

    #[test]
    fn test_package_version_update() {
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();

        // Update to new version
        let result = pkg.update_version("2.0.0");
        assert!(result.is_ok());
        assert_eq!(pkg.version_str(), "2.0.0");

        // Try invalid version
        let result = pkg.update_version("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));

        // Version should remain unchanged after failed update
        assert_eq!(pkg.version_str(), "2.0.0");
    }

    #[test]
    fn test_update_dependency_version() {
        // Create package with dependencies
        let deps = vec![
            make_dependency("dep1", "^1.0.0").unwrap(),
            make_dependency("dep2", "^2.0.0").unwrap(),
        ];

        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();

        // Update an existing dependency
        let result = pkg.update_dependency_version("dep1", "^1.5.0");
        assert!(result.is_ok());
        assert_eq!(pkg.dependencies()[0].borrow().version().to_string(), "^1.5.0");

        // Try to update non-existent dependency
        let result = pkg.update_dependency_version("dep3", "^3.0.0");
        assert!(result.is_err());
        assert!(matches!(result, Err(DependencyResolutionError::DependencyNotFound { .. })));
    }

    #[test]
    fn test_add_dependency() {
        let mut pkg = Package::new("test-pkg", "1.0.0", None).unwrap();
        assert!(pkg.dependencies().is_empty());

        // Add a dependency
        let dep = make_dependency("dep1", "^1.0.0").unwrap();
        pkg.add_dependency(dep);

        // Verify dependency was added
        assert_eq!(pkg.dependencies().len(), 1);
        assert_eq!(pkg.dependencies()[0].borrow().name(), "dep1");
    }

    #[test]
    fn test_update_from_resolution() {
        // Create package with dependencies
        let deps = vec![
            make_dependency("dep1", "^1.0.0").unwrap(),
            make_dependency("dep2", "^2.0.0").unwrap(),
        ];

        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();

        // Create resolution result
        let mut resolved_versions = std::collections::HashMap::new();
        resolved_versions.insert("dep1".to_string(), "1.5.0".to_string());
        resolved_versions.insert("dep2".to_string(), "2.3.0".to_string());
        resolved_versions.insert("dep3".to_string(), "3.0.0".to_string()); // Not in package

        let resolution = ResolutionResult {
            resolved_versions,
            updates_required: Vec::new(), // Not testing this part
        };

        // Apply resolution
        let updated = pkg.update_dependencies_from_resolution(&resolution);
        assert!(updated.is_ok());

        let updated = updated.unwrap();
        assert_eq!(updated.len(), 2); // Only two updates, not three

        // Verify updates
        assert_eq!(pkg.dependencies()[0].borrow().version().to_string(), "^1.5.0");
        assert_eq!(pkg.dependencies()[1].borrow().version().to_string(), "^2.3.0");
    }

    #[test]
    fn test_node_implementation() {
        // Test that Package correctly implements Node trait
        let deps = vec![
            make_dependency("dep1", "^1.0.0").unwrap(),
            make_dependency("dep2", "^2.0.0").unwrap(),
        ];

        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();

        // Test identifier
        assert_eq!(pkg.identifier(), "test-pkg");

        // Test dependencies_vec
        let dep_vec = pkg.dependencies_vec();
        assert_eq!(dep_vec.len(), 2);
        assert_eq!(dep_vec[0].name(), "dep1");
        assert_eq!(dep_vec[1].name(), "dep2");

        // Test matches method with matching dependency
        let matching_dep = Dependency::new("test-pkg", "1.0.0").unwrap();
        assert!(pkg.matches(&matching_dep));

        // Test with non-matching name
        let non_matching_name = Dependency::new("other-pkg", "1.0.0").unwrap();
        assert!(!pkg.matches(&non_matching_name));

        // Test with non-matching version
        let non_matching_version = Dependency::new("test-pkg", "2.0.0").unwrap();
        assert!(!pkg.matches(&non_matching_version));

        // Test with compatible version range
        let compatible_version = Dependency::new("test-pkg", "^1.0.0").unwrap();
        assert!(pkg.matches(&compatible_version));
    }

    #[test]
    fn test_package_cloning() {
        // Test that package cloning works correctly
        let deps = vec![make_dependency("dep1", "^1.0.0").unwrap()];

        let pkg = Package::new("test-pkg", "1.0.0", Some(deps)).unwrap();
        let cloned = pkg.clone();

        // Both should have same values
        assert_eq!(pkg.name(), cloned.name());
        assert_eq!(pkg.version_str(), cloned.version_str());
        assert_eq!(pkg.dependencies().len(), cloned.dependencies().len());

        // Update original package version
        pkg.update_version("2.0.0").unwrap();

        // Cloned should also be updated due to Rc<RefCell>
        assert_eq!(cloned.version_str(), "2.0.0");

        // Same for dependencies
        pkg.dependencies()[0].borrow_mut().update_version("^1.5.0").unwrap();
        assert_eq!(cloned.dependencies()[0].borrow().version().to_string(), "^1.5.0");
    }
}
