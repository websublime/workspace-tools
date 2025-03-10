#[cfg(test)]
mod registry_tests {
    use std::collections::HashMap;
    use ws_pkg::registry::{
        DependencyRegistry, LocalRegistry, RegistryAuth, RegistryManager, RegistryType,
    };
    use ws_pkg::PackageRegistry;

    #[test]
    fn test_dependency_registry() {
        let mut registry = DependencyRegistry::new();

        // Test get_or_create
        let dep1 = registry.get_or_create("test-dep", "^1.0.0").unwrap();
        assert_eq!(dep1.borrow().name(), "test-dep");
        assert_eq!(dep1.borrow().version_str(), "^1.0.0");

        // Test get existing dependency with higher version - should update to higher version
        let dep2 = registry.get_or_create("test-dep", "^2.0.0").unwrap();
        assert_eq!(dep1.borrow().name(), dep2.borrow().name());
        assert_eq!(dep1.borrow().version_str(), "^2.0.0"); // Updated assertion

        // Test get by name
        let dep3 = registry.get("test-dep").unwrap();
        assert_eq!(dep3.borrow().name(), "test-dep");
        assert_eq!(dep3.borrow().version_str(), "^2.0.0"); // Updated assertion

        // Test get nonexistent dependency
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_dependency_resolution() {
        let mut registry = DependencyRegistry::new();

        // Create dependencies with different version requirements
        // First create with lower version
        let dep1 = registry.get_or_create("shared-dep", "^1.0.0").unwrap();
        // Then update with higher version - should update
        let dep2 = registry.get_or_create("shared-dep", "^1.1.0").unwrap();

        // Verify the update worked
        assert_eq!(dep1.borrow().version_str(), "^1.1.0");
        assert!(std::ptr::eq(&*dep1, &*dep2)); // Should be the same Rc

        // Resolve version conflicts
        let result = registry.resolve_version_conflicts().unwrap();

        // Verify the resolution picks the highest compatible version
        assert!(result.resolved_versions.contains_key("shared-dep"));
        assert_eq!(&result.resolved_versions["shared-dep"], "1.1.0");

        // Since we've already updated to the highest version,
        // no update should be required
        assert!(!result.updates_required.iter().any(|u| u.dependency_name == "shared-dep"));
    }

    #[test]
    fn test_local_registry() {
        let registry = LocalRegistry::new();

        // Add packages to the registry
        registry.add_package("test-pkg", vec!["1.0.0", "1.1.0", "2.0.0"]).unwrap();

        // Test get latest version
        let latest = registry.get_latest_version("test-pkg").unwrap();
        assert_eq!(latest, Some("2.0.0".to_string()));

        // Test get all versions
        let versions = registry.get_all_versions("test-pkg").unwrap();
        assert_eq!(versions.len(), 3);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"1.1.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));

        // Test nonexistent package
        let latest = registry.get_latest_version("nonexistent").unwrap();
        assert_eq!(latest, None);

        let versions = registry.get_all_versions("nonexistent").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_local_registry_dependencies() {
        let registry = LocalRegistry::new();

        // Add package with dependencies
        registry.add_package("test-pkg", vec!["1.0.0"]).unwrap();

        let mut deps = HashMap::new();
        deps.insert("dep1".to_string(), "^1.0.0".to_string());
        deps.insert("dep2".to_string(), "^2.0.0".to_string());

        registry.set_dependencies("test-pkg", "1.0.0", &deps).unwrap();

        // Get package info with dependencies
        let pkg_info = registry.get_package_info("test-pkg", "1.0.0").unwrap();

        // Check the dependencies
        assert_eq!(pkg_info["dependencies"]["dep1"], "^1.0.0");
        assert_eq!(pkg_info["dependencies"]["dep2"], "^2.0.0");
    }

    #[test]
    fn test_registry_manager() {
        let mut manager = RegistryManager::new();

        // Default registry should be npm
        assert_eq!(manager.default_registry(), "https://registry.npmjs.org");

        // Add a custom registry
        manager
            .add_registry(
                "https://custom-registry.com",
                RegistryType::Custom("test-client".to_string()),
            )
            .unwrap();

        // Associate a scope with the registry
        manager.associate_scope("@test", "https://custom-registry.com").unwrap();

        // Verify scope association
        assert!(manager.has_scope("@test"));
        assert_eq!(manager.get_registry_for_scope("@test").unwrap(), "https://custom-registry.com");

        // Set a default registry
        manager.set_default_registry("https://custom-registry.com").unwrap();
        assert_eq!(manager.default_registry(), "https://custom-registry.com");

        // Add authentication
        let auth = RegistryAuth {
            token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
            always: false,
        };
        manager.set_auth("https://custom-registry.com", auth).unwrap();
    }

    #[test]
    fn test_registry_type() {
        let npm = RegistryType::Npm;
        let github = RegistryType::GitHub;
        let custom = RegistryType::Custom("test-client".to_string());

        assert_ne!(npm, github);
        assert_ne!(npm, custom);
        assert_ne!(github, custom);
    }

    #[test]
    fn test_dependency_registry_lower_version() {
        let mut registry = DependencyRegistry::new();

        // Start with higher version
        let dep1 = registry.get_or_create("test-dep", "^2.0.0").unwrap();
        assert_eq!(dep1.borrow().version_str(), "^2.0.0");

        // Try to update with lower version - should keep higher version
        let dep2 = registry.get_or_create("test-dep", "^1.0.0").unwrap();
        assert_eq!(dep2.borrow().version_str(), "^2.0.0"); // Should still be the higher version

        // They should be the same object
        assert!(std::ptr::eq(&*dep1, &*dep2));
    }
}
