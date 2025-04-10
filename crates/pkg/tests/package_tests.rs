#[cfg(test)]
mod package_tests {
    use serde_json::json;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sublime_package_tools::{
        ChangeType, Dependency, DependencyChange, DependencyRegistry, Package, PackageDiff,
        PackageInfo,
    };

    #[test]
    fn test_package_creation() {
        // Create package without dependencies
        let pkg = Package::new("my-app", "1.0.0", None);
        assert!(pkg.is_ok());

        // Create package with dependencies
        let dep1 = Rc::new(RefCell::new(Dependency::new("react", "^17.0.0").unwrap()));
        let dep2 = Rc::new(RefCell::new(Dependency::new("lodash", "^4.17.21").unwrap()));

        let pkg_with_deps =
            Package::new("my-app", "1.0.0", Some(vec![Rc::clone(&dep1), Rc::clone(&dep2)]));
        assert!(pkg_with_deps.is_ok());

        // Invalid version
        let invalid_pkg = Package::new("my-app", "not-a-version", None);
        assert!(invalid_pkg.is_err());
    }

    #[test]
    fn test_package_with_registry() {
        let mut registry = DependencyRegistry::new();

        // Create package with registry
        let pkg = Package::new_with_registry(
            "my-app",
            "1.0.0",
            Some(vec![("react", "^17.0.0"), ("lodash", "^4.17.21")]),
            &mut registry,
        );
        assert!(pkg.is_ok());

        // Verify package
        let pkg = pkg.unwrap();
        assert_eq!(pkg.name(), "my-app");
        assert_eq!(pkg.version_str(), "1.0.0");
        assert_eq!(pkg.dependencies().len(), 2);

        // Check if registry properly managed dependencies
        assert!(registry.get("react").is_some());
        assert!(registry.get("lodash").is_some());
    }

    #[test]
    fn test_package_getters() {
        let pkg = Package::new("test-pkg", "2.0.0", None).unwrap();

        assert_eq!(pkg.name(), "test-pkg");
        assert_eq!(pkg.version_str(), "2.0.0");
        assert_eq!(pkg.dependencies().len(), 0);
    }

    #[test]
    fn test_update_package_version() {
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();

        // Valid update
        assert!(pkg.update_version("2.0.0").is_ok());
        assert_eq!(pkg.version_str(), "2.0.0");

        // Invalid update
        assert!(pkg.update_version("not-a-version").is_err());
    }

    #[test]
    fn test_dependency_operations() {
        let mut registry = DependencyRegistry::new();

        // Create package with registry
        let pkg = Package::new_with_registry(
            "my-app",
            "1.0.0",
            Some(vec![("react", "^17.0.0")]),
            &mut registry,
        )
        .unwrap();

        // Add dependency
        let new_dep = Rc::new(RefCell::new(Dependency::new("express", "^4.17.1").unwrap()));
        let mut pkg_mut = pkg.clone();
        pkg_mut.add_dependency(new_dep);

        assert_eq!(pkg_mut.dependencies().len(), 2);

        // Update dependency
        assert!(pkg.update_dependency_version("react", "^18.0.0").is_ok());

        // Verify update
        let react_dep = pkg.dependencies()[0].borrow();
        assert_eq!(react_dep.version().to_string(), "^18.0.0");

        // Try updating non-existent dependency
        assert!(pkg.update_dependency_version("nonexistent", "^1.0.0").is_err());
    }

    #[test]
    fn test_package_diff() {
        // Create two separate registries to avoid shared references
        let mut registry_before = DependencyRegistry::new();
        let mut registry_after = DependencyRegistry::new();

        // Create "before" package
        let before_pkg = Package::new_with_registry(
            "my-app",
            "1.0.0",
            Some(vec![("react", "^17.0.0"), ("lodash", "^4.17.20")]),
            &mut registry_before,
        )
        .unwrap();

        // Create "after" package with changes
        let after_pkg = Package::new_with_registry(
            "my-app",
            "1.1.0",
            Some(vec![
                ("react", "^18.0.0"), // Updated
                ("express", "^4.17.1"), // Added
                                      // lodash removed
            ]),
            &mut registry_after,
        )
        .unwrap();

        // Generate diff
        let diff = PackageDiff::between(&before_pkg, &after_pkg);
        assert!(diff.is_ok());

        let diff = diff.unwrap();
        assert_eq!(diff.package_name, "my-app");
        assert_eq!(diff.previous_version, "1.0.0");
        assert_eq!(diff.current_version, "1.1.0");

        // Check dependency changes
        assert_eq!(diff.dependency_changes.len(), 3);

        // Count by type
        let changes_by_type = diff.count_changes_by_type();
        assert_eq!(*changes_by_type.get(&ChangeType::Added).unwrap_or(&0), 1);
        assert_eq!(*changes_by_type.get(&ChangeType::Updated).unwrap_or(&0), 1);
        assert_eq!(*changes_by_type.get(&ChangeType::Removed).unwrap_or(&0), 1);

        // Check for breaking changes
        assert!(diff.count_breaking_changes() > 0);

        // Verify specific changes
        for change in &diff.dependency_changes {
            match change.name.as_str() {
                "react" => {
                    assert_eq!(change.change_type, ChangeType::Updated);
                    assert_eq!(change.previous_version, Some("17.0.0".to_string()));
                    assert_eq!(change.current_version, Some("18.0.0".to_string()));
                    assert!(change.breaking); // Major version change should be breaking
                }
                "express" => {
                    assert_eq!(change.change_type, ChangeType::Added);
                    assert_eq!(change.previous_version, None);
                    assert_eq!(change.current_version, Some("4.17.1".to_string()));
                }
                "lodash" => {
                    assert_eq!(change.change_type, ChangeType::Removed);
                    assert_eq!(change.previous_version, Some("4.17.20".to_string()));
                    assert_eq!(change.current_version, None);
                }
                _ => panic!("Unexpected dependency: {}", change.name),
            }
        }
    }

    #[test]
    fn test_package_info() {
        let pkg = Package::new("test-pkg", "1.0.0", None).unwrap();

        // Create package info
        let pkg_info = PackageInfo::new(
            pkg,
            "/path/to/package.json".to_string(),
            "/path/to".to_string(),
            "relative/path".to_string(),
            json!({
                "name": "test-pkg",
                "version": "1.0.0",
                "dependencies": {}
            }),
        );

        // Check fields
        assert_eq!(pkg_info.package_json_path, "/path/to/package.json");
        assert_eq!(pkg_info.package_path, "/path/to");
        assert_eq!(pkg_info.package_relative_path, "relative/path");

        // Test update version
        assert!(pkg_info.update_version("1.1.0").is_ok());
        assert_eq!(pkg_info.package.borrow().version_str(), "1.1.0");

        // Test dependency update
        let mut registry = DependencyRegistry::new();
        let pkg_with_deps = Package::new_with_registry(
            "test-pkg",
            "1.0.0",
            Some(vec![("react", "^17.0.0")]),
            &mut registry,
        )
        .unwrap();

        let pkg_info_with_deps = PackageInfo::new(
            pkg_with_deps,
            "/path/to/package.json".to_string(),
            "/path/to".to_string(),
            "relative/path".to_string(),
            json!({
                "name": "test-pkg",
                "version": "1.0.0",
                "dependencies": {
                    "react": "^17.0.0"
                }
            }),
        );

        assert!(pkg_info_with_deps.update_dependency_version("react", "^18.0.0").is_ok());
    }

    #[test]
    fn test_dependency_change() {
        // Added dependency
        let added = DependencyChange::new("express", None, Some("^4.17.1"), ChangeType::Added);
        assert_eq!(added.name, "express");
        assert_eq!(added.previous_version, None);
        assert_eq!(added.current_version, Some("^4.17.1".to_string()));
        assert_eq!(added.change_type, ChangeType::Added);
        assert!(!added.breaking);

        // Updated dependency (non-breaking)
        let updated = DependencyChange::new(
            "lodash",
            Some("^4.17.20"),
            Some("^4.17.21"),
            ChangeType::Updated,
        );
        assert_eq!(updated.change_type, ChangeType::Updated);
        assert!(!updated.breaking);

        // Breaking update
        let breaking =
            DependencyChange::new("react", Some("^17.0.0"), Some("^18.0.0"), ChangeType::Updated);
        assert!(breaking.breaking);

        // Removed dependency
        let removed = DependencyChange::new("moment", Some("^2.29.1"), None, ChangeType::Removed);
        assert_eq!(removed.change_type, ChangeType::Removed);
    }
}
