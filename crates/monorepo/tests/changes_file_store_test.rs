mod test_utils;

use std::fs;
use sublime_monorepo_tools::{Change, ChangeStore, ChangeType, Changeset, FileChangeStore};
use tempfile::TempDir;

#[cfg(test)]
mod changes_file_store_tests {
    use super::*;

    fn setup_test_store() -> (TempDir, FileChangeStore) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let store_path = temp_dir.path().to_path_buf();

        // Ensure the directory exists
        fs::create_dir_all(&store_path).expect("Failed to create directory structure");

        let store = FileChangeStore::new(&store_path).expect("Failed to create FileChangeStore");
        (temp_dir, store)
    }

    #[test]
    fn test_file_store_creation() {
        let (temp_dir, _) = setup_test_store();

        // Directory should exist
        assert!(temp_dir.path().exists());

        // Try to create store in a path that requires creation of multiple directories
        let nested_path = temp_dir.path().join("nested/dirs/for/changesets");
        // Create the directories first
        fs::create_dir_all(&nested_path).expect("Failed to create nested directories");

        let store = FileChangeStore::new(&nested_path);
        assert!(store.is_ok());

        // Nested directory should exist
        assert!(nested_path.exists());
    }

    #[test]
    fn test_file_store_persistence() {
        let (temp_dir, mut store) = setup_test_store();

        // Create and store a changeset
        let change = Change::new("pkg-a", ChangeType::Feature, "Test feature", false);
        let changeset = Changeset::new::<String>(None, vec![change]);
        let changeset_id = changeset.id.clone();

        store.store_changeset(&changeset).expect("Failed to store changeset");

        // A file should exist for the changeset
        let expected_file = temp_dir.path().join(format!("{changeset_id}.json"));
        assert!(expected_file.exists(), "Changeset file wasn't created");

        // Create a new store instance to test loading from files
        let new_store =
            FileChangeStore::new(temp_dir.path()).expect("Failed to create new FileChangeStore");

        // The previously stored changeset should be loaded
        let retrieved = new_store.get_changeset(&changeset_id).expect("Failed to get changeset");
        assert!(retrieved.is_some(), "Failed to retrieve changeset");
        assert_eq!(retrieved.unwrap().id.to_string(), changeset_id.to_string());
    }

    #[test]
    fn test_file_store_removing_changesets() {
        let (temp_dir, mut store) = setup_test_store();

        // Create and store a changeset
        let change = Change::new("pkg-a", ChangeType::Feature, "Test feature", false);
        let changeset = Changeset::new::<String>(None, vec![change]);
        let changeset_id = changeset.id.clone();
        let file_path = temp_dir.path().join(format!("{changeset_id}.json"));

        store.store_changeset(&changeset).expect("Failed to store changeset");
        assert!(file_path.exists(), "Changeset file wasn't created");

        // Remove the changeset
        store.remove_changeset(&changeset_id).expect("Failed to remove changeset");

        // File should be gone
        assert!(!file_path.exists(), "Changeset file wasn't removed");

        // Changeset should no longer be retrievable
        let retrieved = store.get_changeset(&changeset_id).expect("Failed to query store");
        assert!(retrieved.is_none(), "Changeset wasn't removed from store");
    }

    #[test]
    fn test_file_store_mark_as_released() {
        let (temp_dir, mut store) = setup_test_store();

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

        // Create a new store to test if the updates were persisted
        let new_store =
            FileChangeStore::new(temp_dir.path()).expect("Failed to create new FileChangeStore");

        // Store changesets in the new store for testing - No need to do this
        // The new store should load from the files in the temp dir

        // Verify the changes are now released
        let released_a = new_store.get_released_changes("pkg-a").unwrap();
        assert_eq!(released_a.len(), 2);
        assert!(released_a.iter().all(|c| c.release_version == Some("1.0.0".to_string())));

        // pkg-b changes should still be unreleased
        let unreleased_b = new_store.get_unreleased_changes("pkg-b").unwrap();
        assert_eq!(unreleased_b.len(), 1);
    }

    #[test]
    fn test_file_store_loads_only_json_files() {
        let (temp_dir, _) = setup_test_store();

        // Create a non-JSON file that should be ignored
        let text_file = temp_dir.path().join("not-a-changeset.txt");
        fs::write(&text_file, "This is not a changeset").expect("Failed to write test file");

        // Let's try creating the store without the broken JSON first
        let result1 = FileChangeStore::new(temp_dir.path());
        assert!(result1.is_ok(), "Store should be created successfully with non-JSON files");

        // Now create a broken JSON file
        let broken_json = temp_dir.path().join("broken.json");
        fs::write(&broken_json, "{not valid json}").expect("Failed to write broken JSON");

        // Creating a new store should fail with a parse error for the broken JSON
        let result2 = FileChangeStore::new(temp_dir.path());
        assert!(result2.is_err(), "Store creation should fail with broken JSON");

        // Verify it's the expected error type
        if let Err(err) = result2 {
            match err {
                sublime_monorepo_tools::ChangeError::ParseError { path, .. } => {
                    assert_eq!(path, broken_json, "Error should reference the broken JSON file");
                }
                _ => panic!("Expected ParseError, got: {err:?}"),
            }
        }
    }

    #[test]
    fn test_file_store_all_operations() {
        // This test exercises most of the FileChangeStore functionality
        let (_temp_dir, mut store) = setup_test_store();

        // Create changes for different packages with different versions
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 =
            Change::new("pkg-a", ChangeType::Fix, "Fix A", false).with_release_version("1.0.0");
        let change3 = Change::new("pkg-b", ChangeType::Feature, "Feature B", false);

        // Store the changes in changesets
        let cs1 = Changeset::new::<String>(None, vec![change1]);
        let cs2 =
            Changeset::new::<String>(Some(String::from("Release 1.0.0")), vec![change2, change3]);

        store.store_changeset(&cs1).unwrap();
        store.store_changeset(&cs2).unwrap();

        // Test get_all_changesets
        let all_changesets = store.get_all_changesets().unwrap();
        assert_eq!(all_changesets.len(), 2);

        // Test get_unreleased_changes
        let unreleased_a = store.get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased_a.len(), 1);

        let unreleased_b = store.get_unreleased_changes("pkg-b").unwrap();
        assert_eq!(unreleased_b.len(), 1);

        // Test get_released_changes
        let released_a = store.get_released_changes("pkg-a").unwrap();
        assert_eq!(released_a.len(), 1);
        assert_eq!(released_a[0].release_version, Some("1.0.0".to_string()));

        // Test get_changes_by_version
        let changes_by_version = store.get_changes_by_version("pkg-a").unwrap();
        assert_eq!(changes_by_version.len(), 2); // "unreleased" and "1.0.0"
        assert!(changes_by_version.contains_key("unreleased"));
        assert!(changes_by_version.contains_key("1.0.0"));

        // Test get_all_changes_by_package
        let all_by_package = store.get_all_changes_by_package().unwrap();
        assert_eq!(all_by_package.len(), 2); // "pkg-a" and "pkg-b"
        assert_eq!(all_by_package["pkg-a"].len(), 2);
        assert_eq!(all_by_package["pkg-b"].len(), 1);
    }
}
