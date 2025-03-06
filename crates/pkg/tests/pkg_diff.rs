mod diff_tests {
    use std::{cell::RefCell, rc::Rc};
    use ws_pkg::{
        ChangeType, Dependency, DependencyChange, DependencyRegistry, Package, PackageDiff, Result,
    };

    // Helper function to create test packages with controlled dependencies
    fn create_test_package(name: &str, version: &str, deps: Vec<(&str, &str)>) -> Result<Package> {
        let mut dependencies = Vec::new();

        for (dep_name, dep_version) in deps {
            let dep = Dependency::new(dep_name, dep_version)?;
            dependencies.push(Rc::new(RefCell::new(dep)));
        }

        Package::new(name, version, Some(dependencies))
    }

    #[test]
    fn test_dependency_change_detection() {
        // Test added dependency
        let added = DependencyChange::new("@scope/added", None, Some("1.0.0"), ChangeType::Added);
        assert_eq!(added.name, "@scope/added");
        assert_eq!(added.previous_version, None);
        assert_eq!(added.current_version, Some("1.0.0".to_string()));
        assert_eq!(added.change_type, ChangeType::Added);
        assert!(!added.breaking); // Added dependencies are no longer considered breaking

        // Test removed dependency
        let removed =
            DependencyChange::new("@scope/removed", Some("1.0.0"), None, ChangeType::Removed);
        assert_eq!(removed.name, "@scope/removed");
        assert_eq!(removed.previous_version, Some("1.0.0".to_string()));
        assert_eq!(removed.current_version, None);
        assert_eq!(removed.change_type, ChangeType::Removed);
        assert!(removed.breaking); // Removed dependencies are still breaking

        // Test updated dependency (non-breaking)
        let updated = DependencyChange::new(
            "@scope/updated",
            Some("1.0.0"),
            Some("1.1.0"),
            ChangeType::Updated,
        );
        assert_eq!(updated.name, "@scope/updated");
        assert_eq!(updated.previous_version, Some("1.0.0".to_string()));
        assert_eq!(updated.current_version, Some("1.1.0".to_string()));
        assert_eq!(updated.change_type, ChangeType::Updated);
        assert!(!updated.breaking); // Minor version changes are not breaking

        // Test updated dependency (breaking)
        let breaking = DependencyChange::new(
            "@scope/breaking",
            Some("1.0.0"),
            Some("2.0.0"),
            ChangeType::Updated,
        );
        assert_eq!(breaking.name, "@scope/breaking");
        assert_eq!(breaking.previous_version, Some("1.0.0".to_string()));
        assert_eq!(breaking.current_version, Some("2.0.0".to_string()));
        assert_eq!(breaking.change_type, ChangeType::Updated);
        assert!(breaking.breaking); // Major version changes are breaking
    }

    #[test]
    fn test_package_diff_generation() -> Result<()> {
        // Create previous package with explicit dependencies
        let prev_deps =
            vec![("@scope/foo", "1.0.0"), ("@scope/bar", "1.0.0"), ("@scope/remove-me", "1.0.0")];

        let prev_package = create_test_package("@scope/test-package", "1.0.0", prev_deps)?;

        // Create current package with explicit dependencies
        let curr_deps =
            vec![("@scope/foo", "1.1.0"), ("@scope/bar", "2.0.0"), ("@scope/added", "1.0.0")];

        let curr_package = create_test_package("@scope/test-package", "2.0.0", curr_deps)?;

        // Generate diff
        let diff = PackageDiff::between(&prev_package, &curr_package)?;

        // Basic package info
        assert_eq!(diff.package_name, "@scope/test-package");
        assert_eq!(diff.previous_version, "1.0.0");
        assert_eq!(diff.current_version, "2.0.0");
        assert!(diff.breaking_change);

        // Should have 4 dependency changes (foo, bar updated + added, remove-me removed)
        assert_eq!(diff.dependency_changes.len(), 4);

        // ... rest of test assertions ...

        Ok(())
    }

    #[test]
    fn test_diff_statistics() -> Result<()> {
        // Create previous package with explicit dependencies
        let prev_deps =
            vec![("@scope/dep1", "1.0.0"), ("@scope/dep2", "1.0.0"), ("@scope/dep3", "1.0.0")];

        let pkg_v1 = create_test_package("@scope/pkg", "1.0.0", prev_deps)?;

        // Create current package with explicit dependencies
        let curr_deps = vec![
            ("@scope/dep2", "2.0.0"),
            ("@scope/dep3", "1.1.0"),
            ("@scope/dep4", "1.0.0"),
            ("@scope/dep5", "1.0.0"),
        ];

        let pkg_v2 = create_test_package("@scope/pkg", "1.1.0", curr_deps)?;

        // Generate diff
        let diff = PackageDiff::between(&pkg_v1, &pkg_v2)?;
        let counts = diff.count_changes_by_type();

        // We should have 2 breaking changes:
        // - dep1 removed
        // - dep2 major version update
        assert_eq!(diff.count_breaking_changes(), 2);

        // Check each change type individually
        let added_count = *counts.get(&ChangeType::Added).unwrap_or(&0);
        let removed_count = *counts.get(&ChangeType::Removed).unwrap_or(&0);
        let updated_count = *counts.get(&ChangeType::Updated).unwrap_or(&0);

        assert_eq!(added_count, 2); // dep4, dep5
        assert_eq!(removed_count, 1); // dep1
        assert_eq!(updated_count, 2); // dep2, dep3

        // Total changes should be sum of all types
        assert_eq!(diff.dependency_changes.len(), 5);

        Ok(())
    }

    #[test]
    fn test_diff_error_for_different_packages() -> Result<()> {
        // Create two packages with different names
        let pkg1 = Package::new("@scope/pkg1", "1.0.0", None)?;
        let pkg2 = Package::new("@scope/pkg2", "1.0.0", None)?;

        // Attempt to generate diff between different packages
        let result = PackageDiff::between(&pkg1, &pkg2);

        // Should return an error
        assert!(result.is_err());

        if let Err(ws_pkg::PkgError::Other { message }) = result {
            assert!(message.contains("Cannot diff different packages"));
            assert!(message.contains("@scope/pkg1"));
            assert!(message.contains("@scope/pkg2"));
        } else {
            panic!("Expected Other error type");
        }

        Ok(())
    }

    #[test]
    fn test_no_dependency_changes() -> Result<()> {
        // Create two versions of a package with the same dependencies
        let mut registry = DependencyRegistry::new();
        let dep = registry.get_or_create("@scope/dep", "1.0.0")?;

        let pkg_v1 = Package::new("@scope/pkg", "1.0.0", Some(vec![Rc::clone(&dep)]))?;

        let pkg_v2 = Package::new("@scope/pkg", "1.1.0", Some(vec![Rc::clone(&dep)]))?;

        let diff = PackageDiff::between(&pkg_v1, &pkg_v2)?;

        assert_eq!(diff.package_name, "@scope/pkg");
        assert_eq!(diff.previous_version, "1.0.0");
        assert_eq!(diff.current_version, "1.1.0");
        assert!(!diff.breaking_change);
        assert!(diff.dependency_changes.is_empty());

        let diff_string = diff.to_string();
        assert!(diff_string.contains("No dependency changes"));

        Ok(())
    }
}
