#[cfg(test)]
mod package_diff_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use sublime_package_tools::{
        ChangeType, Dependency, DependencyChange, Package, PackageDiff, PackageError,
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
    fn test_dependency_change_creation() {
        // Create various types of dependency changes

        // Added dependency
        let added = DependencyChange::new("new-dep", None, Some("^1.0.0"), ChangeType::Added);

        assert_eq!(added.name, "new-dep");
        assert_eq!(added.previous_version, None);
        assert_eq!(added.current_version, Some("^1.0.0".to_string()));
        assert_eq!(added.change_type, ChangeType::Added);
        assert!(!added.breaking); // Adding deps is not breaking

        // Removed dependency
        let removed = DependencyChange::new("old-dep", Some("^1.0.0"), None, ChangeType::Removed);

        assert_eq!(removed.name, "old-dep");
        assert_eq!(removed.previous_version, Some("^1.0.0".to_string()));
        assert_eq!(removed.current_version, None);
        assert_eq!(removed.change_type, ChangeType::Removed);
        assert!(removed.breaking); // Removing deps is breaking

        // Updated dependency (non-breaking)
        let updated_minor = DependencyChange::new(
            "updated-dep",
            Some("^1.0.0"),
            Some("^1.1.0"),
            ChangeType::Updated,
        );

        assert_eq!(updated_minor.name, "updated-dep");
        assert_eq!(updated_minor.previous_version, Some("^1.0.0".to_string()));
        assert_eq!(updated_minor.current_version, Some("^1.1.0".to_string()));
        assert_eq!(updated_minor.change_type, ChangeType::Updated);
        assert!(!updated_minor.breaking); // Minor update is not breaking

        // Updated dependency (breaking)
        let updated_major = DependencyChange::new(
            "breaking-dep",
            Some("^1.0.0"),
            Some("^2.0.0"),
            ChangeType::Updated,
        );

        assert_eq!(updated_major.name, "breaking-dep");
        assert_eq!(updated_major.previous_version, Some("^1.0.0".to_string()));
        assert_eq!(updated_major.current_version, Some("^2.0.0".to_string()));
        assert_eq!(updated_major.change_type, ChangeType::Updated);
        assert!(updated_major.breaking); // Major update is breaking
    }

    #[test]
    fn test_dependency_change_with_non_semver() {
        // Test with version strings that are not strictly semver

        // With prefixed versions
        let change_caret =
            DependencyChange::new("caret-dep", Some("^1.0.0"), Some("^2.0.0"), ChangeType::Updated);

        assert!(change_caret.breaking); // Should detect major version change

        // With tilde versions
        let change_tilde =
            DependencyChange::new("tilde-dep", Some("~1.0.0"), Some("~1.1.0"), ChangeType::Updated);

        assert!(!change_tilde.breaking); // Should not be breaking

        // With non-parseable versions
        let change_complex = DependencyChange::new(
            "complex-dep",
            Some("latest"),
            Some("1.0.0"),
            ChangeType::Updated,
        );

        assert!(change_complex.breaking); // Conservatively assumes breaking
    }

    #[test]
    fn test_package_diff_between() {
        // Create two versions of a package
        let pkg_v1 = make_package(
            "test-pkg",
            "1.0.0",
            vec![("dep1", "^1.0.0"), ("dep2", "^2.0.0"), ("dep-to-remove", "^1.0.0")],
        );

        let pkg_v2 = make_package(
            "test-pkg",
            "1.1.0",
            vec![
                ("dep1", "^1.0.0"),      // unchanged
                ("dep2", "^2.1.0"),      // updated
                ("dep-added", "^1.0.0"), // added
            ],
        );

        // Generate diff
        let diff = PackageDiff::between(&pkg_v1, &pkg_v2).unwrap();

        // Verify basic properties
        assert_eq!(diff.package_name, "test-pkg");
        assert_eq!(diff.previous_version, "1.0.0");
        assert_eq!(diff.current_version, "1.1.0");
        assert!(!diff.breaking_change); // Minor version bump is not breaking

        // Verify dependency changes
        assert_eq!(diff.dependency_changes.len(), 3); // 1 updated, 1 added, 1 removed

        // Count changes by type
        let changes_by_type = diff.count_changes_by_type();
        assert_eq!(changes_by_type.get(&ChangeType::Added).unwrap_or(&0), &1);
        assert_eq!(changes_by_type.get(&ChangeType::Removed).unwrap_or(&0), &1);
        assert_eq!(changes_by_type.get(&ChangeType::Updated).unwrap_or(&0), &1);
        assert_eq!(changes_by_type.get(&ChangeType::Unchanged).unwrap_or(&0), &0);

        // Find specific changes
        let has_removed = diff
            .dependency_changes
            .iter()
            .any(|c| c.name == "dep-to-remove" && c.change_type == ChangeType::Removed);
        assert!(has_removed);

        let has_added = diff
            .dependency_changes
            .iter()
            .any(|c| c.name == "dep-added" && c.change_type == ChangeType::Added);
        assert!(has_added);

        let has_updated = diff
            .dependency_changes
            .iter()
            .any(|c| c.name == "dep2" && c.change_type == ChangeType::Updated);
        assert!(has_updated);
    }

    #[test]
    fn test_breaking_change_detection() {
        // Create two versions of a package with breaking change
        let pkg_v1 = make_package("test-pkg", "1.0.0", vec![("dep1", "^1.0.0")]);
        let pkg_v2 = make_package("test-pkg", "2.0.0", vec![("dep1", "^1.0.0")]);

        // Generate diff
        let diff = PackageDiff::between(&pkg_v1, &pkg_v2).unwrap();

        // Verify breaking change detection
        assert!(diff.breaking_change); // Major version bump is breaking

        // No breaking dependency changes
        assert_eq!(diff.count_breaking_changes(), 0);
    }

    #[test]
    fn test_dependency_breaking_changes() {
        // Create two versions of a package with breaking dependency changes
        let pkg_v1 = make_package(
            "test-pkg",
            "1.0.0",
            vec![("dep1", "^1.0.0"), ("dep2", "^2.0.0"), ("dep3", "^3.0.0")],
        );

        let pkg_v2 = make_package(
            "test-pkg",
            "1.1.0", // Minor version bump only
            vec![
                ("dep1", "^2.0.0"), // Breaking: major bump
                ("dep2", "^2.1.0"), // Non-breaking: minor bump
                ("dep4", "^1.0.0"), // Added: non-breaking
            ],
        );

        // Generate diff
        let diff = PackageDiff::between(&pkg_v1, &pkg_v2).unwrap();

        // Overall package isn't marked breaking (only minor bump)
        assert!(!diff.breaking_change);

        // But it has breaking dependency changes
        assert_eq!(diff.count_breaking_changes(), 2); // dep1 major bump and dep3 removed
    }

    #[test]
    fn test_diff_error_conditions() {
        // Try to diff different packages
        let pkg1 = make_package("pkg1", "1.0.0", vec![]);
        let pkg2 = make_package("pkg2", "1.0.0", vec![]);

        let result = PackageDiff::between(&pkg1, &pkg2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PackageError::PackageBetweenFailure(_)));
    }

    #[test]
    fn test_display_formatting() {
        // Create a diff with various changes for display testing
        let pkg_v1 = make_package(
            "display-test",
            "1.0.0",
            vec![("unchanged", "^1.0.0"), ("updated", "^1.0.0"), ("removed", "^1.0.0")],
        );

        let pkg_v2 = make_package(
            "display-test",
            "1.1.0",
            vec![("unchanged", "^1.0.0"), ("updated", "^1.2.0"), ("added", "^2.0.0")],
        );

        // Generate diff
        let diff = PackageDiff::between(&pkg_v1, &pkg_v2).unwrap();

        // Test display output
        let display_str = diff.to_string();

        // Should contain package info
        assert!(display_str.contains("display-test"));
        assert!(display_str.contains("1.0.0→1.1.0"));

        // Should contain dependency changes
        assert!(display_str.contains("added"));
        assert!(display_str.contains("removed"));
        assert!(display_str.contains("updated"));

        // Should have the right symbols
        assert!(display_str.contains("+ added"));
        assert!(display_str.contains("- removed"));
        assert!(display_str.contains("↑ updated"));
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(ChangeType::Added.to_string(), "added");
        assert_eq!(ChangeType::Removed.to_string(), "removed");
        assert_eq!(ChangeType::Updated.to_string(), "updated");
        assert_eq!(ChangeType::Unchanged.to_string(), "unchanged");
    }
}
