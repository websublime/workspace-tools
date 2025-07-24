//! Comprehensive tests for the dependency module
//!
//! This module contains all tests for dependency-related functionality including
//! conflict resolution, storage, and dependency management.

#![allow(clippy::unwrap_used)] // Tests may use unwrap for test failures per CLAUDE.md rules
#![allow(clippy::expect_used)] // Tests may use expect for test failures per CLAUDE.md rules
#![allow(clippy::panic)] // Tests may use panic for test failures per CLAUDE.md rules

mod conflict_resolver_tests {
    use crate::dependency::conflict_resolver::ConflictResolver;
    use crate::dependency::resolution::ResolutionResult;
    use crate::dependency::update::Update as DependencyUpdate;
    use crate::dependency::storage::DependencyStorage;
    use crate::external::package_registry_client::PackageRegistryClient;
    use semver::VersionReq;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_resolve_no_conflicts() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add single dependency with no conflicts
        storage.get_or_insert("react", "^17.0.0").expect("Test operation should succeed");

        let result = resolver.resolve_version_conflicts().await.expect("Test operation should succeed");
        
        assert_eq!(result.resolved_versions.len(), 1);
        assert!(result.resolved_versions.contains_key("react"));
        assert!(result.updates_required.is_empty()); // No updates needed
    }

    #[tokio::test]
    async fn test_resolve_simple_conflict() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add conflicting versions - storage should resolve to higher
        storage.get_or_insert("react", "^16.0.0").expect("Test operation should succeed");
        storage.get_or_insert("react", "^17.0.0").expect("Test operation should succeed"); // Higher version

        let result = resolver.resolve_version_conflicts().await.expect("Test operation should succeed");
        
        assert_eq!(result.resolved_versions.len(), 1);
        // The resolver uses fixed_version() which removes prefixes
        assert_eq!(result.resolved_versions.get("react").expect("Test operation should succeed"), "17.0.0");
    }

    #[tokio::test]
    async fn test_resolve_empty_storage() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        let result = resolver.resolve_version_conflicts().await.expect("Test operation should succeed");
        
        assert!(result.resolved_versions.is_empty());
        assert!(result.updates_required.is_empty());
    }

    #[tokio::test]
    async fn test_find_highest_compatible_version_fallback() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new(); // No registry configured
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add a dependency to storage
        storage.get_or_insert("lodash", "^4.17.0").expect("Test operation should succeed");

        let req = VersionReq::parse("^4.0.0").expect("Test operation should succeed");
        let result = resolver.find_highest_compatible_version("lodash", &[&req]).await.expect("Test operation should succeed");
        
        // Should use local storage version
        assert_eq!(result, "4.17.0");
    }

    #[tokio::test]
    async fn test_apply_resolution_empty() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        // Empty resolution result
        let result = ResolutionResult {
            resolved_versions: HashMap::new(),
            updates_required: Vec::new(),
        };

        // Should succeed with no updates
        resolver.apply_resolution(&result).await.expect("Test operation should succeed");
    }

    #[tokio::test]
    async fn test_apply_resolution_with_updates() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage.clone(), client);

        // Add initial dependency
        storage.get_or_insert("react", "^16.0.0").expect("Test operation should succeed");

        // Create resolution with update
        let mut resolved_versions = HashMap::new();
        resolved_versions.insert("react".to_string(), "^17.0.0".to_string());

        let updates = vec![DependencyUpdate {
            package_name: "test-app".to_string(),
            dependency_name: "react".to_string(),
            current_version: "^16.0.0".to_string(),
            new_version: "^17.0.0".to_string(),
        }];

        let result = ResolutionResult {
            resolved_versions,
            updates_required: updates,
        };

        // Apply updates
        resolver.apply_resolution(&result).await.expect("Test operation should succeed");

        // Verify update was applied
        let updated_dep = storage.get("react").expect("Test operation should succeed");
        assert_eq!(updated_dep.version().to_string(), "^17.0.0");
    }

    #[test]
    fn test_has_registry() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);

        assert!(!resolver.has_registry()); // No registry configured
    }

    #[test]
    fn test_clone_resolver() {
        let storage = DependencyStorage::new();
        let client = PackageRegistryClient::new();
        let resolver = ConflictResolver::new(storage, client);
        
        let cloned = resolver.clone_resolver();
        assert_eq!(resolver.has_registry(), cloned.has_registry());
    }

    // Note: clean_version_string is private method, removing test

    // Note: generate_fallback_version is private method, removing test
}

mod storage_tests {
    use crate::dependency::storage::DependencyStorage;
    use std::thread;
    use std::sync::{Arc as StdArc, Barrier};

    #[test]
    fn test_new_storage_is_empty() {
        let storage = DependencyStorage::new();
        let (count, memory) = storage.stats();
        assert_eq!(count, 0);
        assert_eq!(memory, 0);
    }

    #[test]
    fn test_get_or_insert_new_dependency() {
        let storage = DependencyStorage::new();
        let dep = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep.name(), "react");
        assert_eq!(dep.version().to_string(), "^17.0.0");
        
        let (count, _) = storage.stats();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_or_insert_existing_dependency() {
        let storage = DependencyStorage::new();
        
        // Insert initial version
        let dep1 = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep1.version().to_string(), "^17.0.0");
        
        // Get same version - should return same
        let dep2 = storage.get_or_insert("react", "^17.0.0").unwrap();
        assert_eq!(dep2.version().to_string(), "^17.0.0");
        
        let (count, _) = storage.stats();
        assert_eq!(count, 1); // Still only one dependency
    }

    #[test]
    fn test_version_update_logic() {
        let storage = DependencyStorage::new();
        
        // Insert initial version
        storage.get_or_insert("react", "^17.0.0").unwrap();
        
        // Try to insert higher version - should update
        let dep = storage.get_or_insert("react", "^18.0.0").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
        
        // Try to insert lower version - should keep higher
        let dep = storage.get_or_insert("react", "^16.0.0").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
        
        // Verify using get method
        let dep = storage.get("react").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
    }

    #[test]
    fn test_get_nonexistent_dependency() {
        let storage = DependencyStorage::new();
        assert!(storage.get("missing").is_none());
    }

    #[test]
    fn test_update_existing_dependency() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        // Update existing dependency
        storage.update("lodash", "^4.17.21").unwrap();
        let dep = storage.get("lodash").unwrap();
        assert_eq!(dep.version().to_string(), "^4.17.21");
    }

    #[test]
    fn test_update_nonexistent_dependency() {
        let storage = DependencyStorage::new();
        let result = storage.update("missing", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_dependencies() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^17.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        storage.get_or_insert("vue", "^3.0.0").unwrap();
        
        let all_deps = storage.all_dependencies();
        assert_eq!(all_deps.len(), 3);
        
        // Verify all dependencies are present
        let names: Vec<String> = all_deps.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"react".to_string()));
        assert!(names.contains(&"lodash".to_string()));
        assert!(names.contains(&"vue".to_string()));
    }

    #[test]
    fn test_batch_updates_success() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^16.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        let updates = vec![
            ("react".to_string(), "^17.0.0".to_string()),
            ("lodash".to_string(), "^4.17.21".to_string()),
        ];
        
        storage.apply_batch_updates(&updates).unwrap();
        
        // Verify all updates applied
        let react = storage.get("react").unwrap();
        assert_eq!(react.version().to_string(), "^17.0.0");
        let lodash = storage.get("lodash").unwrap();
        assert_eq!(lodash.version().to_string(), "^4.17.21");
    }

    #[test]
    fn test_batch_updates_atomicity() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^16.0.0").unwrap();
        
        // Include one invalid update - should fail and not apply any
        let updates = vec![
            ("react".to_string(), "^17.0.0".to_string()),
            ("missing".to_string(), "^1.0.0".to_string()), // This will fail
        ];
        
        let result = storage.apply_batch_updates(&updates);
        assert!(result.is_err());
        
        // Verify no updates were applied
        let react = storage.get("react").unwrap();
        assert_eq!(react.version().to_string(), "^16.0.0"); // Unchanged
    }

    #[test]
    fn test_clear() {
        let storage = DependencyStorage::new();
        storage.get_or_insert("react", "^17.0.0").unwrap();
        storage.get_or_insert("lodash", "^4.17.0").unwrap();
        
        let (count, _) = storage.stats();
        assert_eq!(count, 2);
        
        storage.clear().unwrap();
        let (count, _) = storage.stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_concurrent_access() {
        let storage = StdArc::new(DependencyStorage::new());
        let barrier = StdArc::new(Barrier::new(3));
        let mut handles = vec![];
        
        // Spawn 3 threads that will access storage concurrently
        for i in 0..3 {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            
            let handle = thread::spawn(move || {
                barrier_clone.wait(); // Synchronize start
                
                // Each thread inserts different dependencies
                for j in 0..10 {
                    let dep_name = format!("dep-{}-{}", i, j);
                    storage_clone.get_or_insert(&dep_name, "^1.0.0").unwrap();
                }
                
                // Read operations
                for j in 0..10 {
                    let dep_name = format!("dep-{}-{}", i, j);
                    assert!(storage_clone.get(&dep_name).is_some());
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all dependencies were inserted correctly
        let (count, _) = storage.stats();
        assert_eq!(count, 30); // 3 threads * 10 dependencies each
    }

    #[test]
    fn test_concurrent_version_updates() {
        let storage = StdArc::new(DependencyStorage::new());
        
        // Insert initial dependency
        storage.get_or_insert("react", "^16.0.0").unwrap();
        
        let barrier = StdArc::new(Barrier::new(2));
        let mut handles = vec![];
        
        // Thread 1: Update to 17.0.0
        {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                storage_clone.get_or_insert("react", "^17.0.0").unwrap();
            });
            handles.push(handle);
        }
        
        // Thread 2: Update to 18.0.0 (higher version)
        {
            let storage_clone = StdArc::clone(&storage);
            let barrier_clone = StdArc::clone(&barrier);
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                storage_clone.get_or_insert("react", "^18.0.0").unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for both threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // The final version should be the highest one (^18.0.0)
        let dep = storage.get("react").unwrap();
        assert_eq!(dep.version().to_string(), "^18.0.0");
    }

    #[test]
    fn test_version_comparison_edge_cases() {
        let storage = DependencyStorage::new();
        
        // Test with versions without prefix (semver interprets as caret by default)
        storage.get_or_insert("test", "1.0.0").unwrap();
        storage.get_or_insert("test", "1.0.1").unwrap(); // Higher
        let dep = storage.get("test").unwrap();
        assert_eq!(dep.version().to_string(), "^1.0.1"); // semver default behavior
        
        // Test with exact versions using = prefix
        storage.get_or_insert("test_exact", "=1.0.0").unwrap();
        storage.get_or_insert("test_exact", "=1.0.1").unwrap(); // Higher
        let dep = storage.get("test_exact").unwrap();
        assert_eq!(dep.version().to_string(), "=1.0.1");
        
        // Test with mixed prefixes
        storage.get_or_insert("test2", "^1.0.0").unwrap();
        storage.get_or_insert("test2", "~1.0.5").unwrap(); // Higher patch version
        let dep = storage.get("test2").unwrap();
        assert_eq!(dep.version().to_string(), "~1.0.5");
        
        // Test with major version differences
        storage.get_or_insert("test3", "^1.5.0").unwrap();
        storage.get_or_insert("test3", "^2.0.0").unwrap(); // Higher major
        let dep = storage.get("test3").unwrap();
        assert_eq!(dep.version().to_string(), "^2.0.0");
    }

    #[test]
    fn test_invalid_version_handling() {
        let storage = DependencyStorage::new();
        
        // Test invalid version format
        let result = storage.get_or_insert("test", "invalid-version");
        assert!(result.is_err());
        
        // Verify storage wasn't corrupted
        let (count, _) = storage.stats();
        assert_eq!(count, 0);
    }
}