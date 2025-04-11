#[cfg(test)]
mod change_store_tests {
    use sublime_monorepo_tools::{
        Change, ChangeStore, ChangeType, Changeset, FileChangeStore, MemoryChangeStore,
    };
    use tempfile::TempDir;

    // Helper function to create test changes
    fn create_test_changes() -> Vec<Change> {
        vec![
            Change::new("@scope/package-foo", ChangeType::Feature, "Add button component", false),
            Change::new("@scope/package-foo", ChangeType::Fix, "Fix styling", false),
            Change::new("@scope/package-bar", ChangeType::Feature, "Add validation", false),
            Change::new("@scope/package-baz", ChangeType::Performance, "Optimize rendering", true)
                .with_environments(vec!["production"]),
        ]
    }

    #[test]
    fn test_memory_store_basic_operations() {
        let mut store = MemoryChangeStore::new();
        let changes = create_test_changes();

        // Create a changeset
        let changeset =
            Changeset::new(Some("Test changeset"), vec![changes[0].clone(), changes[1].clone()]);
        let changeset_id = changeset.id.clone();

        // Store changeset
        store.store_changeset(&changeset).expect("Failed to store changeset");

        // Retrieve changeset
        let retrieved = store.get_changeset(&changeset_id).expect("Failed to get changeset");
        assert!(retrieved.is_some());

        let retrieved_changeset = retrieved.unwrap();
        assert_eq!(retrieved_changeset.summary, changeset.summary);
        assert_eq!(retrieved_changeset.changes.len(), changeset.changes.len());

        // Get all changesets
        let all_changesets = store.get_all_changesets().expect("Failed to get all changesets");
        assert_eq!(all_changesets.len(), 1);

        // Create another changeset
        let changeset2 =
            Changeset::new(Some("Another changeset"), vec![changes[2].clone(), changes[3].clone()]);

        store.store_changeset(&changeset2).expect("Failed to store second changeset");

        // Verify we have 2 changesets
        let all_changesets = store.get_all_changesets().expect("Failed to get all changesets");
        assert_eq!(all_changesets.len(), 2);

        // Test removing a changeset
        store.remove_changeset(&changeset_id).expect("Failed to remove changeset");

        // Verify one was removed
        let all_changesets = store.get_all_changesets().expect("Failed to get all changesets");
        assert_eq!(all_changesets.len(), 1);
    }

    #[test]
    fn test_memory_store_change_queries() {
        let mut store = MemoryChangeStore::new();
        let changes = create_test_changes();

        // Store changes in multiple changesets
        let changeset1 = Changeset::new::<String>(
            None,
            vec![
                changes[0].clone(), // package-foo, feature
                changes[1].clone(), // package-foo, fix
            ],
        );
        let changeset2 = Changeset::new::<String>(
            None,
            vec![
                changes[2].clone(), // package-bar, feature
                changes[3].clone(), // package-baz, performance
            ],
        );

        store.store_changeset(&changeset1).expect("Failed to store changeset1");
        store.store_changeset(&changeset2).expect("Failed to store changeset2");

        // Test getting unreleased changes by package
        let foo_unreleased = store
            .get_unreleased_changes("@scope/package-foo")
            .expect("Failed to get package-foo unreleased changes");
        assert_eq!(foo_unreleased.len(), 2);

        // Mark one package-foo change as released
        let mut update_changeset = changeset1.clone();
        update_changeset.changes[0].release_version = Some("1.0.0".to_string());
        store.store_changeset(&update_changeset).expect("Failed to update changeset");

        // Verify one unreleased and one released
        let foo_unreleased = store
            .get_unreleased_changes("@scope/package-foo")
            .expect("Failed to get package-foo unreleased changes");
        assert_eq!(foo_unreleased.len(), 1);

        let foo_released = store
            .get_released_changes("@scope/package-foo")
            .expect("Failed to get package-foo released changes");
        assert_eq!(foo_released.len(), 1);

        // Test get_changes_by_version
        let foo_by_version = store
            .get_changes_by_version("@scope/package-foo")
            .expect("Failed to get package-foo changes by version");
        assert_eq!(foo_by_version.len(), 2); // "unreleased" and "1.0.0"
        assert!(foo_by_version.contains_key("unreleased"));
        assert!(foo_by_version.contains_key("1.0.0"));

        // Test get_all_changes_by_package
        let all_by_package =
            store.get_all_changes_by_package().expect("Failed to get all changes by package");
        assert_eq!(all_by_package.len(), 3); // Three packages with changes
        assert_eq!(all_by_package["@scope/package-foo"].len(), 2);
        assert_eq!(all_by_package["@scope/package-bar"].len(), 1);
        assert_eq!(all_by_package["@scope/package-baz"].len(), 1);
    }

    #[test]
    fn test_memory_store_release_operations() {
        let mut store = MemoryChangeStore::new();
        let changes = create_test_changes();

        // Store all changes in a changeset
        let changeset = Changeset::new::<String>(None, changes);
        store.store_changeset(&changeset).expect("Failed to store changeset");

        // Verify initial state
        let unreleased_foo = store
            .get_unreleased_changes("@scope/package-foo")
            .expect("Failed to get unreleased foo");
        assert_eq!(unreleased_foo.len(), 2);

        // Test dry run release
        let to_be_released = store
            .mark_changes_as_released("@scope/package-foo", "1.0.0", true)
            .expect("Dry run release failed");
        assert_eq!(to_be_released.len(), 2);

        // Verify nothing actually changed
        let still_unreleased = store
            .get_unreleased_changes("@scope/package-foo")
            .expect("Failed to get unreleased foo");
        assert_eq!(still_unreleased.len(), 2);

        // Do real release
        let released = store
            .mark_changes_as_released("@scope/package-foo", "1.0.0", false)
            .expect("Release failed");
        assert_eq!(released.len(), 2);

        // Verify changes are marked as released
        let now_unreleased = store
            .get_unreleased_changes("@scope/package-foo")
            .expect("Failed to get unreleased foo");
        assert_eq!(now_unreleased.len(), 0);

        let now_released =
            store.get_released_changes("@scope/package-foo").expect("Failed to get released foo");
        assert_eq!(now_released.len(), 2);
        assert_eq!(now_released[0].release_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_memory_store_environment_filters() {
        let mut store = MemoryChangeStore::new();

        // Create changes with environment specifics
        let prod_change =
            Change::new("@scope/package-foo", ChangeType::Feature, "Feature A", false)
                .with_environments(vec!["production"]);

        let staging_change =
            Change::new("@scope/package-foo", ChangeType::Feature, "Feature B", false)
                .with_environments(vec!["staging"]);

        let all_env_change = Change::new("@scope/package-foo", ChangeType::Fix, "Fix C", false);

        // Store changes
        let changeset =
            Changeset::new::<String>(None, vec![prod_change, staging_change, all_env_change]);
        store.store_changeset(&changeset).expect("Failed to store changeset");

        // Test environment-specific queries
        let prod_changes = store
            .get_changes_for_environment("@scope/package-foo", "production")
            .expect("Failed to get production changes");
        assert_eq!(prod_changes.len(), 2); // prod_change + all_env_change

        let staging_changes = store
            .get_changes_for_environment("@scope/package-foo", "staging")
            .expect("Failed to get staging changes");
        assert_eq!(staging_changes.len(), 2); // staging_change + all_env_change

        let dev_changes = store
            .get_changes_for_environment("@scope/package-foo", "development")
            .expect("Failed to get development changes");
        assert_eq!(dev_changes.len(), 1); // only all_env_change

        // Test get_changes_by_environment
        let all_prod_changes = store
            .get_changes_by_environment("production")
            .expect("Failed to get all production changes");
        assert_eq!(all_prod_changes.len(), 1); // Only one package has changes
        assert_eq!(all_prod_changes["@scope/package-foo"].len(), 2);
    }

    #[test]
    fn test_file_store_basic_operations() {
        // Create temporary directory for file store
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let store_path = temp_dir.path().join("changes");

        // Create file store
        let mut store = FileChangeStore::new(&store_path).expect("Failed to create file store");

        // Create some changes
        let changes = create_test_changes();
        let changeset =
            Changeset::new(Some("File store test"), vec![changes[0].clone(), changes[1].clone()]);
        let changeset_id = changeset.id.clone();

        // Store the changeset
        store.store_changeset(&changeset).expect("Failed to store changeset in file store");

        // Retrieve the changeset
        let retrieved =
            store.get_changeset(&changeset_id).expect("Failed to get changeset from file store");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().summary, changeset.summary);

        // Create a new file store instance to test persistence
        let mut new_store =
            FileChangeStore::new(&store_path).expect("Failed to create second file store");

        // The changeset should still be there
        let retrieved2 = new_store
            .get_changeset(&changeset_id)
            .expect("Failed to get changeset from new file store");
        assert!(retrieved2.is_some());

        // Test removing a changeset
        new_store.remove_changeset(&changeset_id).expect("Failed to remove changeset");
        let after_remove = new_store.get_changeset(&changeset_id).expect("Get operation failed");
        assert!(after_remove.is_none());
    }
}
