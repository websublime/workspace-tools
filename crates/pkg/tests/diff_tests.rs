#[cfg(test)]
mod diff_tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use ws_pkg::types::dependency::Dependency;
    use ws_pkg::types::diff::{ChangeType, DependencyChange, PackageDiff};
    use ws_pkg::types::package::Package;

    // Helper function to create test dependencies
    fn create_test_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    // Helper function to create test packages
    fn create_test_package(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Package {
        Package::new(name, version, dependencies).unwrap()
    }

    #[test]
    fn test_dependency_change() {
        // Test added dependency
        let added = DependencyChange::new("new-dep", None, Some("1.0.0"), ChangeType::Added);
        assert_eq!(added.name, "new-dep");
        assert_eq!(added.previous_version, None);
        assert_eq!(added.current_version, Some("1.0.0".to_string()));
        assert_eq!(added.change_type, ChangeType::Added);
        assert!(!added.breaking); // Added dependency isn't breaking

        // Test removed dependency
        let removed = DependencyChange::new("old-dep", Some("1.0.0"), None, ChangeType::Removed);
        assert_eq!(removed.name, "old-dep");
        assert_eq!(removed.previous_version, Some("1.0.0".to_string()));
        assert_eq!(removed.current_version, None);
        assert_eq!(removed.change_type, ChangeType::Removed);
        assert!(removed.breaking); // Removed dependency is breaking

        // Test updated dependency (non-breaking)
        let updated =
            DependencyChange::new("dep", Some("1.0.0"), Some("1.1.0"), ChangeType::Updated);
        assert_eq!(updated.name, "dep");
        assert_eq!(updated.previous_version, Some("1.0.0".to_string()));
        assert_eq!(updated.current_version, Some("1.1.0".to_string()));
        assert_eq!(updated.change_type, ChangeType::Updated);
        assert!(!updated.breaking); // Minor version update isn't breaking

        // Test updated dependency (breaking)
        let breaking =
            DependencyChange::new("dep", Some("1.0.0"), Some("2.0.0"), ChangeType::Updated);
        assert_eq!(breaking.name, "dep");
        assert!(breaking.breaking); // Major version update is breaking
    }

    #[test]
    fn test_package_diff() {
        // Create two versions of a package
        let deps1 = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
            create_test_dependency("stable", "^1.0.0"),
        ];
        let pkg1 = create_test_package("test-pkg", "1.0.0", Some(deps1));

        let deps2 = vec![
            create_test_dependency("dep1", "^1.1.0"), // Updated
            create_test_dependency("dep3", "^1.0.0"), // Added
            create_test_dependency("stable", "^1.0.0"), // Unchanged
                                                      // dep2 removed
        ];
        let pkg2 = create_test_package("test-pkg", "2.0.0", Some(deps2));

        // Generate diff
        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Verify diff properties
        assert_eq!(diff.package_name, "test-pkg");
        assert_eq!(diff.previous_version, "1.0.0");
        assert_eq!(diff.current_version, "2.0.0");
        assert!(diff.breaking_change); // Major version bump is breaking

        // Verify dependency changes
        assert_eq!(diff.dependency_changes.len(), 3); // stable dep should not be included

        // Verify specific changes
        let dep_changes: HashMap<_, _> =
            diff.dependency_changes.iter().map(|c| (c.name.clone(), &c.change_type)).collect();

        assert_eq!(&dep_changes["dep1"], &&ChangeType::Updated);
        assert_eq!(&dep_changes["dep2"], &&ChangeType::Removed);
        assert_eq!(&dep_changes["dep3"], &&ChangeType::Added);

        // Ensure stable dependency isn't in the changes list
        assert!(!dep_changes.contains_key("stable"));
    }

    #[test]
    fn test_breaking_change_detection() {
        // Create packages with version change
        let pkg1 = create_test_package("test-pkg", "1.0.0", None);
        let pkg2 = create_test_package("test-pkg", "2.0.0", None); // Breaking: major version bump

        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();
        assert!(diff.breaking_change);

        // Non-breaking change
        let pkg3 = create_test_package("test-pkg", "1.1.0", None); // Non-breaking: minor version
        let diff = PackageDiff::between(&pkg1, &pkg3).unwrap();
        assert!(!diff.breaking_change);
    }

    #[test]
    fn test_breaking_dependency_changes() {
        // Create packages with breaking dependency changes
        let deps1 = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
        ];
        let pkg1 = create_test_package("test-pkg", "1.0.0", Some(deps1));

        let deps2 = vec![
            create_test_dependency("dep1", "^2.0.0"), // Breaking: major version bump
            create_test_dependency("dep2", "^2.1.0"), // Non-breaking: minor version
        ];
        let pkg2 = create_test_package("test-pkg", "1.1.0", Some(deps2));

        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Check breaking changes count
        assert_eq!(diff.count_breaking_changes(), 1);

        // Find the breaking change
        let breaking_changes: Vec<_> =
            diff.dependency_changes.iter().filter(|c| c.breaking).collect();

        assert_eq!(breaking_changes.len(), 1);
        assert_eq!(breaking_changes[0].name, "dep1");
    }

    #[test]
    fn test_diff_stats() {
        // Create packages with various changes
        let deps1 = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
            create_test_dependency("dep3", "^1.0.0"),
        ];
        let pkg1 = create_test_package("test-pkg", "1.0.0", Some(deps1));

        let deps2 = vec![
            create_test_dependency("dep1", "^1.1.0"), // Updated
            create_test_dependency("dep2", "^3.0.0"), // Updated (breaking)
            create_test_dependency("dep4", "^1.0.0"), // Added
                                                      // dep3 removed
        ];
        let pkg2 = create_test_package("test-pkg", "2.0.0", Some(deps2));

        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Check change counts
        let counts = diff.count_changes_by_type();
        assert_eq!(&counts[&ChangeType::Added], &1);
        assert_eq!(&counts[&ChangeType::Removed], &1);
        assert_eq!(&counts[&ChangeType::Updated], &2);
        assert!(
            !counts.contains_key(&ChangeType::Unchanged)
                || counts.get(&ChangeType::Unchanged) == Some(&0)
        );
    }

    #[test]
    fn test_diff_string_representation() {
        // Create a simple package diff for testing string representation
        let deps1 = vec![
            create_test_dependency("dep1", "^1.0.0"),
            create_test_dependency("dep2", "^2.0.0"),
        ];
        let pkg1 = create_test_package("test-pkg", "1.0.0", Some(deps1));

        let deps2 = vec![
            create_test_dependency("dep1", "^1.1.0"), // Updated
            create_test_dependency("dep3", "^1.0.0"), // Added
                                                      // dep2 removed
        ];
        let pkg2 = create_test_package("test-pkg", "2.0.0", Some(deps2));

        let diff = PackageDiff::between(&pkg1, &pkg2).unwrap();

        // Convert to string
        let diff_str = diff.to_string();

        // Verify the string contains key information
        assert!(diff_str.contains("test-pkg"));
        assert!(diff_str.contains("1.0.0"));
        assert!(diff_str.contains("2.0.0"));
        assert!(diff_str.contains("Breaking change"));
        assert!(diff_str.contains("dep1"));
        assert!(diff_str.contains("dep2"));
        assert!(diff_str.contains("dep3"));
        assert!(diff_str.contains("added"));
        assert!(diff_str.contains("removed"));
        assert!(diff_str.contains("updated"));
    }

    #[test]
    fn test_change_type_string_conversion() {
        assert_eq!(ChangeType::Added.to_string(), "added");
        assert_eq!(ChangeType::Removed.to_string(), "removed");
        assert_eq!(ChangeType::Updated.to_string(), "updated");
        assert_eq!(ChangeType::Unchanged.to_string(), "unchanged");
    }

    #[test]
    fn test_error_on_different_packages() {
        let pkg1 = create_test_package("pkg1", "1.0.0", None);
        let pkg2 = create_test_package("pkg2", "1.0.0", None);

        let result = PackageDiff::between(&pkg1, &pkg2);
        assert!(result.is_err());
    }
}
