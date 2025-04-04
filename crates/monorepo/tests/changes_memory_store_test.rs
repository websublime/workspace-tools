use rstest::*;

use sublime_monorepo_tools::{Change, ChangeStore, ChangeType, Changeset, MemoryChangeStore};

mod fixtures;

#[rstest]
fn test_memory_store_basic_operations() {
    let mut store = MemoryChangeStore::new();

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

    // Remove the changeset
    store.remove_changeset(&changeset.id).expect("Failed to remove changeset");

    // Verify it's gone
    let after_remove = store.get_changeset(&changeset.id).expect("Failed to check for changeset");
    assert!(after_remove.is_none());
}

#[rstest]
fn test_memory_store_multiple_packages() {
    let mut store = MemoryChangeStore::new();

    // Create changes for different packages
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Foo feature", false);

    let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Bar fix", false);

    let change3 = Change::new("@scope/package-foo", ChangeType::Refactor, "Foo refactor", false);

    // Store in changesets
    let changeset1 = Changeset::new::<std::string::String>(None, vec![change1, change2]);
    let changeset2 = Changeset::new::<std::string::String>(None, vec![change3]);

    store.store_changeset(&changeset1).expect("Failed to store changeset1");
    store.store_changeset(&changeset2).expect("Failed to store changeset2");

    // Get all changes by package
    let changes_by_package =
        store.get_all_changes_by_package().expect("Failed to get all changes by package");

    // Verify structure
    assert!(changes_by_package.contains_key("@scope/package-foo"));
    assert!(changes_by_package.contains_key("@scope/package-bar"));

    // Verify counts
    assert_eq!(changes_by_package["@scope/package-foo"].len(), 2);
    assert_eq!(changes_by_package["@scope/package-bar"].len(), 1);
}

#[rstest]
fn test_memory_store_changes_for_environment() {
    let mut store = MemoryChangeStore::new();

    // Create changes with different environments
    let mut change1 =
        Change::new("@scope/package-foo", ChangeType::Feature, "Feature for production", false);
    change1.environments = vec!["production".to_string()];

    let mut change2 =
        Change::new("@scope/package-foo", ChangeType::Feature, "Feature for staging", false);
    change2.environments = vec!["staging".to_string()];

    let change3 = Change::new(
        "@scope/package-foo",
        ChangeType::Feature,
        "Feature for all environments",
        false,
    );
    // Empty environments list means it applies to all environments

    // Store in a changeset
    let changeset = Changeset::new::<std::string::String>(None, vec![change1, change2, change3]);
    store.store_changeset(&changeset).expect("Failed to store changeset");

    // Get changes for production environment
    let prod_changes = store
        .get_changes_for_environment("@scope/package-foo", "production")
        .expect("Failed to get changes for production");

    // Verify - should include production-specific and all-environment changes
    assert_eq!(prod_changes.len(), 2);

    // Get changes for development environment
    let dev_changes = store
        .get_changes_for_environment("@scope/package-foo", "development")
        .expect("Failed to get changes for development");

    // Verify - should include only all-environment changes
    assert_eq!(dev_changes.len(), 1);
    assert_eq!(dev_changes[0].description, "Feature for all environments");
}

#[rstest]
fn test_memory_store_dry_run() {
    let mut store = MemoryChangeStore::new();

    // Create changes
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Feature 1", false);

    let change2 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix 1", false);

    // Store changes
    let changeset = Changeset::new::<std::string::String>(None, vec![change1, change2]);
    store.store_changeset(&changeset).expect("Failed to store changeset");

    // Mark as released with dry-run
    let released_changes = store
        .mark_changes_as_released("@scope/package-foo", "1.0.0", true)
        .expect("Failed to perform dry run");

    // Verify changes were identified but not actually modified
    assert_eq!(released_changes.len(), 2);

    // Verify changes are still unreleased
    let still_unreleased = store
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");
    assert_eq!(still_unreleased.len(), 2);

    // No released changes yet
    let released =
        store.get_released_changes("@scope/package-foo").expect("Failed to get released changes");
    assert_eq!(released.len(), 0);
}
