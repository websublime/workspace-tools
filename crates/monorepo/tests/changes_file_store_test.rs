use rstest::*;
use std::fs;
use tempfile::TempDir;

use sublime_monorepo_tools::{Change, ChangeStore, ChangeType, Changeset, FileChangeStore};

mod fixtures;

// Helper function to create a file-based change store
fn create_file_store(temp_dir: &TempDir) -> FileChangeStore {
    let changes_dir = temp_dir.path().join(".changes");
    fs::create_dir_all(&changes_dir).expect("Failed to create changes directory");
    FileChangeStore::new(&changes_dir).expect("Failed to create file change store")
}

#[rstest]
fn test_store_and_get_changeset(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let mut store = create_file_store(&temp_dir);

    // Create a change
    let change = Change::new("@scope/package-foo", ChangeType::Feature, "New feature", false);

    // Create a changeset with the change
    let changeset = Changeset::new(Some("Test changeset".to_string()), vec![change]);

    // Store the changeset
    store.store_changeset(&changeset).expect("Failed to store changeset");

    // Retrieve the changeset
    let retrieved = store
        .get_changeset(&changeset.id)
        .expect("Failed to get changeset")
        .expect("Changeset not found");

    // Verify contents
    assert_eq!(retrieved.id, changeset.id);
    assert_eq!(retrieved.summary, Some("Test changeset".to_string()));
    assert_eq!(retrieved.changes.len(), 1);
    assert_eq!(retrieved.changes[0].package, "@scope/package-foo");
}

#[rstest]
fn test_get_all_changesets(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let mut store = create_file_store(&temp_dir);

    // Create and store multiple changesets
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Feature 1", false);

    let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix 1", false);

    let changeset1 = Changeset::new(Some("Changeset 1".to_string()), vec![change1]);
    let changeset2 = Changeset::new(Some("Changeset 2".to_string()), vec![change2]);

    store.store_changeset(&changeset1).expect("Failed to store changeset1");
    store.store_changeset(&changeset2).expect("Failed to store changeset2");

    // Get all changesets
    let all_changesets = store.get_all_changesets().expect("Failed to get all changesets");

    // Verify count
    assert_eq!(all_changesets.len(), 2);
}

#[rstest]
fn test_remove_changeset(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let mut store = create_file_store(&temp_dir);

    // Create and store a changeset
    let change = Change::new("@scope/package-foo", ChangeType::Feature, "New feature", false);

    let changeset = Changeset::new::<std::string::String>(None, vec![change]);

    store.store_changeset(&changeset).expect("Failed to store changeset");

    // Verify it exists
    let before = store.get_changeset(&changeset.id).expect("Failed to get changeset");
    assert!(before.is_some());

    // Remove the changeset
    store.remove_changeset(&changeset.id).expect("Failed to remove changeset");

    // Verify it no longer exists
    let after = store.get_changeset(&changeset.id).expect("Failed to get changeset");
    assert!(after.is_none());
}

#[rstest]
fn test_get_changes_by_version(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let mut store = create_file_store(&temp_dir);

    // Create unreleased changes
    let unreleased1 =
        Change::new("@scope/package-foo", ChangeType::Feature, "Unreleased feature", false);

    let unreleased2 = Change::new("@scope/package-foo", ChangeType::Fix, "Unreleased fix", false);

    // Create released changes
    let mut released1 =
        Change::new("@scope/package-foo", ChangeType::Feature, "Released feature", false);
    released1.release_version = Some("1.0.0".to_string());

    let mut released2 = Change::new("@scope/package-foo", ChangeType::Fix, "Released fix", false);
    released2.release_version = Some("1.1.0".to_string());

    // Store changes in changesets
    let changeset1 = Changeset::new::<std::string::String>(None, vec![unreleased1, unreleased2]);
    let changeset2 = Changeset::new::<std::string::String>(None, vec![released1, released2]);

    store.store_changeset(&changeset1).expect("Failed to store changeset1");
    store.store_changeset(&changeset2).expect("Failed to store changeset2");

    // Get changes by version
    let changes_by_version = store
        .get_changes_by_version("@scope/package-foo")
        .expect("Failed to get changes by version");

    // Verify structure
    assert_eq!(changes_by_version.len(), 3); // unreleased, 1.0.0, 1.1.0
    assert!(changes_by_version.contains_key("unreleased"));
    assert!(changes_by_version.contains_key("1.0.0"));
    assert!(changes_by_version.contains_key("1.1.0"));

    // Verify counts
    assert_eq!(changes_by_version["unreleased"].len(), 2);
    assert_eq!(changes_by_version["1.0.0"].len(), 1);
    assert_eq!(changes_by_version["1.1.0"].len(), 1);
}

#[rstest]
fn test_mark_changes_as_released(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let mut store = create_file_store(&temp_dir);

    // Create unreleased changes
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Feature 1", false);

    let change2 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix 1", false);

    // Store changes
    let changeset = Changeset::new::<std::string::String>(None, vec![change1, change2]);
    store.store_changeset(&changeset).expect("Failed to store changeset");

    // Verify they are unreleased
    let unreleased = store
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");
    assert_eq!(unreleased.len(), 2);

    // Mark as released
    let released_changes = store
        .mark_changes_as_released("@scope/package-foo", "1.0.0", false)
        .expect("Failed to mark changes as released");

    // Verify the result
    assert_eq!(released_changes.len(), 2);

    // Verify they are now released
    let still_unreleased = store
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");
    assert_eq!(still_unreleased.len(), 0);

    let released =
        store.get_released_changes("@scope/package-foo").expect("Failed to get released changes");
    assert_eq!(released.len(), 2);

    // Verify version
    for change in &released {
        assert_eq!(change.release_version.as_deref(), Some("1.0.0"));
    }
}
