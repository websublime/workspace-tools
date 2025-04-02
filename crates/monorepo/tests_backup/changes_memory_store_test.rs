mod test_utils;

use sublime_monorepo_tools::{Change, ChangeStore, ChangeType, Changeset, MemoryChangeStore};

#[cfg(test)]
mod changes_memory_store_tests {
    use super::*;

    fn create_test_changes() -> (Change, Change, Change) {
        (
            Change::new("pkg-a", ChangeType::Feature, "Feature A", false),
            Change::new("pkg-a", ChangeType::Fix, "Fix A", false).with_release_version("1.0.0"),
            Change::new("pkg-b", ChangeType::Refactor, "Refactor B", false),
        )
    }

    #[test]
    fn test_memory_store_basics() {
        let mut store = MemoryChangeStore::new();
        let (change1, change2, change3) = create_test_changes();

        // Create and store changesets
        let changeset1 = Changeset::new::<String>(None, vec![change1.clone()]);
        let changeset2 = Changeset::new::<String>(None, vec![change2.clone(), change3.clone()]);

        store.store_changeset(&changeset1).unwrap();
        store.store_changeset(&changeset2).unwrap();

        // Test retrieval by ID
        let retrieved1 = store.get_changeset(&changeset1.id).unwrap();
        assert!(retrieved1.is_some());
        assert_eq!(retrieved1.unwrap().id.to_string(), changeset1.id.to_string());

        // Test getting all changesets
        let all = store.get_all_changesets().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_memory_store_removing_changesets() {
        let mut store = MemoryChangeStore::new();
        let (change1, _, _) = create_test_changes();

        // Create and store a changeset
        let changeset = Changeset::new::<String>(None, vec![change1]);
        let changeset_id = changeset.id.clone();

        store.store_changeset(&changeset).unwrap();
        assert_eq!(store.get_all_changesets().unwrap().len(), 1);

        // Remove the changeset
        store.remove_changeset(&changeset_id).unwrap();
        assert_eq!(store.get_all_changesets().unwrap().len(), 0);
        assert!(store.get_changeset(&changeset_id).unwrap().is_none());
    }

    #[test]
    fn test_memory_store_unreleased_changes() {
        let mut store = MemoryChangeStore::new();
        let (change1, change2, change3) = create_test_changes();

        // Create and store changesets
        let changeset1 = Changeset::new::<String>(None, vec![change1]); // pkg-a unreleased
        let changeset2 = Changeset::new::<String>(None, vec![change2, change3]); // pkg-a released, pkg-b unreleased

        store.store_changeset(&changeset1).unwrap();
        store.store_changeset(&changeset2).unwrap();

        // Test getting unreleased changes
        let unreleased_a = store.get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased_a.len(), 1);

        let unreleased_b = store.get_unreleased_changes("pkg-b").unwrap();
        assert_eq!(unreleased_b.len(), 1);

        let unreleased_c = store.get_unreleased_changes("pkg-c").unwrap();
        assert_eq!(unreleased_c.len(), 0);
    }

    #[test]
    fn test_memory_store_released_changes() {
        let mut store = MemoryChangeStore::new();
        let (change1, change2, change3) = create_test_changes();

        // Create and store changesets
        let changeset1 = Changeset::new::<String>(None, vec![change1]); // pkg-a unreleased
        let changeset2 = Changeset::new::<String>(None, vec![change2, change3]); // pkg-a released, pkg-b unreleased

        store.store_changeset(&changeset1).unwrap();
        store.store_changeset(&changeset2).unwrap();

        // Test getting released changes
        let released_a = store.get_released_changes("pkg-a").unwrap();
        assert_eq!(released_a.len(), 1);

        let released_b = store.get_released_changes("pkg-b").unwrap();
        assert_eq!(released_b.len(), 0);
    }

    #[test]
    fn test_memory_store_changes_by_version() {
        let mut store = MemoryChangeStore::new();

        // Create changes with different versions
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature 1", false);
        let change2 =
            Change::new("pkg-a", ChangeType::Fix, "Fix 1", false).with_release_version("1.0.0");
        let change3 =
            Change::new("pkg-a", ChangeType::Fix, "Fix 2", false).with_release_version("1.0.0");
        let change4 = Change::new("pkg-a", ChangeType::Feature, "Feature 2", false)
            .with_release_version("1.1.0");

        // Store the changes in changesets
        store.store_changeset(&Changeset::new::<String>(None, vec![change1])).unwrap();
        store.store_changeset(&Changeset::new::<String>(None, vec![change2, change3])).unwrap();
        store.store_changeset(&Changeset::new::<String>(None, vec![change4])).unwrap();

        // Get changes by version
        let changes_by_version = store.get_changes_by_version("pkg-a").unwrap();

        // Should have 3 versions: unreleased, 1.0.0, and 1.1.0
        assert_eq!(changes_by_version.len(), 3);
        assert_eq!(changes_by_version["unreleased"].len(), 1);
        assert_eq!(changes_by_version["1.0.0"].len(), 2);
        assert_eq!(changes_by_version["1.1.0"].len(), 1);
    }

    #[test]
    fn test_memory_store_mark_released() {
        let mut store = MemoryChangeStore::new();

        // Create unreleased changes
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature 1", false);
        let change2 = Change::new("pkg-a", ChangeType::Fix, "Fix 1", false);
        let change3 = Change::new("pkg-b", ChangeType::Feature, "Feature B", false);

        // Store the changes in changesets
        store.store_changeset(&Changeset::new::<String>(None, vec![change1, change2])).unwrap();
        store.store_changeset(&Changeset::new::<String>(None, vec![change3])).unwrap();

        // Mark pkg-a changes as released
        let updated = store.mark_changes_as_released("pkg-a", "1.0.0", false).unwrap();

        // Should have updated 2 changes
        assert_eq!(updated.len(), 2);

        // Verify the changes are now released
        let released_a = store.get_released_changes("pkg-a").unwrap();
        assert_eq!(released_a.len(), 2);
        assert!(released_a.iter().all(|c| c.release_version == Some("1.0.0".to_string())));

        // pkg-b changes should still be unreleased
        let unreleased_b = store.get_unreleased_changes("pkg-b").unwrap();
        assert_eq!(unreleased_b.len(), 1);
    }

    #[test]
    fn test_memory_store_dry_run_mark_released() {
        let mut store = MemoryChangeStore::new();

        // Create unreleased changes
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature 1", false);

        // Store the changes in changesets
        store.store_changeset(&Changeset::new::<String>(None, vec![change1])).unwrap();

        // Dry run marking pkg-a changes as released
        let updated = store.mark_changes_as_released("pkg-a", "1.0.0", true).unwrap();

        // Should indicate 1 change would be updated
        assert_eq!(updated.len(), 1);

        // But the change should still be unreleased
        let unreleased_a = store.get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased_a.len(), 1);
        assert!(unreleased_a[0].release_version.is_none());
    }

    #[test]
    fn test_memory_store_all_changes_by_package() {
        let mut store = MemoryChangeStore::new();

        // Create changes for different packages
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 = Change::new("pkg-a", ChangeType::Fix, "Fix A", false);
        let change3 = Change::new("pkg-b", ChangeType::Feature, "Feature B", false);
        let change4 = Change::new("pkg-c", ChangeType::Feature, "Feature C", false);

        // Store the changes in changesets
        store.store_changeset(&Changeset::new::<String>(None, vec![change1, change3])).unwrap();
        store.store_changeset(&Changeset::new::<String>(None, vec![change2, change4])).unwrap();

        // Get all changes by package
        let all_changes = store.get_all_changes_by_package().unwrap();

        // Should have entries for all three packages
        assert_eq!(all_changes.len(), 3);
        assert_eq!(all_changes["pkg-a"].len(), 2);
        assert_eq!(all_changes["pkg-b"].len(), 1);
        assert_eq!(all_changes["pkg-c"].len(), 1);
    }
}
