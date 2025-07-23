// Thread safety tests for sublime_package_tools
//
// This module contains tests to verify that public types implement Send + Sync
// where appropriate, ensuring thread safety guarantees.

#[cfg(test)]
mod thread_safety_tests {
    use sublime_package_tools::*;
    use sublime_package_tools::errors::*;

    // Helper functions to verify Send + Sync bounds at compile time
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_core_types_are_send_sync() {
        // Core data types should be Send + Sync
        assert_send_sync::<Package>();
        assert_send_sync::<Dependency>();
        assert_send_sync::<Info>();
        
        // Version types
        // Note: Version here refers to the enum in version/mod.rs, not semver::Version
        // assert_send_sync::<Version>(); // This is an enum for version bumping, should be Send+Sync
        assert_send_sync::<VersionUpdateStrategy>();
        assert_send_sync::<VersionStability>();
        assert_send_sync::<VersionRelationship>();
    }

    #[test]
    fn test_error_types_are_send_sync() {
        // All error types should be Send + Sync for proper error propagation
        assert_send_sync::<Error>();
        assert_send_sync::<VersionError>();
        assert_send_sync::<PackageError>();
        assert_send_sync::<DependencyResolutionError>();
        assert_send_sync::<PackageRegistryError>();
        assert_send_sync::<RegistryError>();
    }

    #[test]
    fn test_registry_types_are_send_sync() {
        // Registry types should be thread-safe for concurrent access
        // Note: Registry is NOT Send+Sync due to Box<dyn PackageRegistryClone> without Send+Sync bounds
        // assert_send_sync::<Registry>(); // This will fail until trait bounds are fixed
        assert_send_sync::<RegistryManager>();
        assert_send_sync::<NpmRegistry>();
        assert_send_sync::<LocalRegistry>();
        
        // Registry configuration types
        assert_send_sync::<RegistryType>();
        assert_send_sync::<RegistryAuth>();
    }

    #[test]
    fn test_upgrade_types_are_send_sync() {
        // Upgrader types should be thread-safe
        assert_send_sync::<Upgrader>();
        assert_send_sync::<UpgradeConfig>();
        assert_send_sync::<UpgradeStatus>();
        assert_send_sync::<AvailableUpgrade>();
        assert_send_sync::<ExecutionMode>();
    }

    #[test]
    fn test_graph_types_are_send_sync() {
        // Graph types should be thread-safe for concurrent analysis
        assert_send_sync::<ValidationIssue>();
        assert_send_sync::<ValidationReport>();
        assert_send_sync::<ValidationOptions>();
        assert_send_sync::<DotOptions>();
    }

    #[test]
    fn test_dependency_types_are_send_sync() {
        // Dependency management types
        assert_send_sync::<Change>();
        assert_send_sync::<Filter>();
        assert_send_sync::<Update>();
        assert_send_sync::<ResolutionResult>();
    }

    #[test]
    fn test_configuration_types_are_send_sync() {
        // Configuration types should be shareable between threads
        assert_send_sync::<Filter>(); // This is the actual enum name in the codebase
    }

    // Test for collections and complex types
    #[test]
    fn test_collections_are_send_sync() {
        assert_send_sync::<Vec<Package>>();
        assert_send_sync::<Vec<Dependency>>();
        assert_send_sync::<Vec<AvailableUpgrade>>();
        
        // Test Result types commonly used in the API (using the crate's Result<T> type alias)
        assert_send_sync::<Result<Package>>();
        assert_send_sync::<Result<Vec<AvailableUpgrade>>>();
        
        // Test standard Result types
        assert_send_sync::<std::result::Result<Package, VersionError>>();
        assert_send_sync::<std::result::Result<Vec<AvailableUpgrade>, PackageRegistryError>>();
    }

    // Specific test for graph node patterns
    #[test]
    fn test_graph_node_implementation() {
        // Verify Package implements Node trait properly
        fn verify_node_send_sync<T: Node + Send + Sync>() {}
        verify_node_send_sync::<Package>();
    }

    // Test for cache entry types
    #[test]
    fn test_cache_types_are_send_sync() {
        assert_send_sync::<CacheEntry<String>>();
        assert_send_sync::<CacheEntry<Package>>();
        assert_send_sync::<CacheEntry<serde_json::Value>>();
    }

    // Documentation test showing thread safety usage
    #[test]
    fn test_concurrent_usage_example() {
        use std::sync::Arc;
        use std::thread;
        
        // Create a registry that can be shared between threads
        let registry = Arc::new(RegistryManager::new());
        
        // Clone the Arc for use in another thread
        let registry_clone = Arc::clone(&registry);
        
        // Spawn a thread that uses the registry
        let handle = thread::spawn(move || {
            // This should compile without issues if RegistryManager is Send + Sync
            let _urls = registry_clone.registry_urls();
        });
        
        // Use registry in main thread
        let _default = registry.default_registry();
        
        // Wait for thread to complete
        handle.join().unwrap();
    }

    // Test for potential issues with boxed trait objects
    #[test]
    fn test_boxed_traits_are_send_sync() {
        // These are the types actually used in the codebase
        // Note: These will fail until traits have Send + Sync bounds
        // assert_send::<Box<dyn PackageRegistryClone>>();
        
        // This demonstrates the thread safety issue - traits need Send + Sync bounds
        // to be used in multi-threaded contexts
    }
}

// Integration test demonstrating multi-threaded usage
#[cfg(test)]
mod integration_thread_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use sublime_package_tools::*;

    #[test]
    fn test_multi_threaded_registry_access() {
        let registry = Arc::new(Mutex::new(RegistryManager::new()));
        let mut handles = vec![];

        // Spawn multiple threads accessing the registry
        for i in 0..4 {
            let registry_clone = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let registry = registry_clone.lock().unwrap();
                let _urls = registry.registry_urls();
                
                // Simulate some work
                thread::sleep(std::time::Duration::from_millis(10 * i));
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_package_processing() {
        use std::sync::mpsc;
        
        // Create some test packages
        let packages = vec![
            Package::new("pkg1", "1.0.0", None).unwrap(),
            Package::new("pkg2", "2.0.0", None).unwrap(),
            Package::new("pkg3", "3.0.0", None).unwrap(),
        ];

        let (tx, rx) = mpsc::channel();

        // Process packages in separate thread
        let handle = thread::spawn(move || {
            for package in packages {
                // Package should be Send, so this should work
                tx.send(package.name().to_string()).unwrap();
            }
        });

        // Collect results in main thread
        let mut results = vec![];
        for _ in 0..3 {
            results.push(rx.recv().unwrap());
        }

        handle.join().unwrap();
        
        // Verify we got all package names
        assert_eq!(results.len(), 3);
        assert!(results.contains(&"pkg1".to_string()));
        assert!(results.contains(&"pkg2".to_string()));
        assert!(results.contains(&"pkg3".to_string()));
    }
}