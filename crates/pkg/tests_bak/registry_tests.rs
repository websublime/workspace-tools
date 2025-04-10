#[cfg(test)]
mod registry_tests {
    use std::sync::Arc;

    use sublime_package_tools::{
        LocalRegistry, NpmRegistry, PackageRegistry, PackageRegistryError, RegistryAuth,
        RegistryError, RegistryManager, RegistryType,
    };

    #[test]
    fn test_local_registry() {
        // Create a local registry for testing
        let registry = LocalRegistry::default();

        // Initially should be empty
        let versions = registry.get_all_versions("test-package");
        assert!(versions.is_ok());
        assert_eq!(versions.unwrap().len(), 0);

        let latest = registry.get_latest_version("test-package");
        assert!(latest.is_ok());
        assert_eq!(latest.unwrap(), None);

        // Package info for non-existent should fail
        let info = registry.get_package_info("test-package", "1.0.0");
        assert!(info.is_err());
        assert!(matches!(info.unwrap_err(), PackageRegistryError::NotFound { .. }));

        // Note: We cannot easily modify the LocalRegistry since its state is private
        // In a real test, you would use reflection or add methods to insert test data
    }

    #[test]
    fn test_registry_manager_basics() {
        // Create a new registry manager
        let mut manager = RegistryManager::new();

        // Should have default npm registry
        assert_eq!(manager.default_registry(), "https://registry.npmjs.org");

        // Add a custom registry
        manager.add_registry(
            "https://custom-registry.example.com",
            RegistryType::Custom("TestClient".to_string()),
        );

        // Get registry URLs
        let urls = manager.registry_urls();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://registry.npmjs.org"));
        assert!(urls.contains(&"https://custom-registry.example.com"));

        // Set default registry
        let result = manager.set_default_registry("https://custom-registry.example.com");
        assert!(result.is_ok());
        assert_eq!(manager.default_registry(), "https://custom-registry.example.com");

        // Try to set default to non-existent registry
        let result = manager.set_default_registry("https://non-existent.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RegistryError::UrlNotFound(_)));
    }

    #[test]
    fn test_registry_scopes() {
        let mut manager = RegistryManager::new();

        // Add a registry for a scope
        manager.add_registry("https://scope-registry.example.com", RegistryType::Npm);

        // Associate scope with registry
        let result = manager.associate_scope("@test-scope", "https://scope-registry.example.com");
        assert!(result.is_ok());

        // Verify scope association
        assert!(manager.has_scope("@test-scope"));
        assert_eq!(
            manager.get_registry_for_scope("@test-scope"),
            Some("https://scope-registry.example.com")
        );

        // Get registry for scoped package
        // Note: This is hard to test fully because the registry objects are wrapped in Arc
    }

    #[test]
    fn test_registry_auth() {
        let mut manager = RegistryManager::new();

        // Add a registry
        manager.add_registry("https://auth-registry.example.com", RegistryType::Npm);

        // Set authentication
        let auth = RegistryAuth {
            token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
            always: true,
        };

        let result = manager.set_auth("https://auth-registry.example.com", auth);
        assert!(result.is_ok());

        // Try to set auth for non-existent registry
        let auth2 = RegistryAuth {
            token: "another-token".to_string(),
            token_type: "Basic".to_string(),
            always: false,
        };

        let result = manager.set_auth("https://non-existent.com", auth2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RegistryError::UrlNotFound(_)));
    }

    #[test]
    fn test_registry_add_instance() {
        let mut manager = RegistryManager::new();

        // Create a local registry
        let local_registry =
            Arc::new(LocalRegistry::default()) as Arc<dyn PackageRegistry + Send + Sync>;

        // Add it to the manager
        manager.add_registry_instance("local://registry", local_registry);

        // Verify it was added
        let urls = manager.registry_urls();
        assert!(urls.contains(&"local://registry"));
    }

    #[test]
    fn test_npm_registry_creation() {
        // Basic creation
        let _registry = NpmRegistry::new("https://registry.npmjs.org");

        // Configure options
        let mut configured = NpmRegistry::new("https://registry.npmjs.org");
        configured.set_user_agent("Test Agent/1.0");
        configured.set_auth("token123", "bearer");

        // Test default registry
        let _default = NpmRegistry::default();
        // Not much we can assert here due to the networking aspects
    }

    // This simulates what loading from .npmrc might do
    #[test]
    fn test_registry_manager_configuration() {
        let mut manager = RegistryManager::new();

        // Add registries
        manager.add_registry("https://registry1.example.com", RegistryType::Npm);
        manager.add_registry("https://registry2.example.com", RegistryType::GitHub);

        // Configure scopes
        manager.associate_scope("@scope1", "https://registry1.example.com").unwrap();
        manager.associate_scope("@scope2", "https://registry2.example.com").unwrap();

        // Set default
        manager.set_default_registry("https://registry1.example.com").unwrap();

        // Verify configuration
        assert_eq!(manager.default_registry(), "https://registry1.example.com");
        assert!(manager.has_scope("@scope1"));
        assert!(manager.has_scope("@scope2"));

        // Verify URL detection for packages
        assert_eq!(
            manager.get_registry_for_scope("@scope1"),
            Some("https://registry1.example.com")
        );
        assert_eq!(
            manager.get_registry_for_scope("@scope2"),
            Some("https://registry2.example.com")
        );
    }
}
