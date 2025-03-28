mod test_utils;

use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore, WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod changes_tracker_tests {
    use super::*;

    fn setup_test_workspace_with_git() -> (TestWorkspace, Rc<sublime_monorepo_tools::Workspace>) {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Initialize Git repository
        let root = test_workspace.path();
        let repo = Repo::create(root.to_str().unwrap()).expect("Failed to create Git repo");
        repo.config("Test User", "test@example.com").expect("Failed to configure Git");

        // Add and commit the files
        repo.add_all().expect("Failed to add files");
        repo.commit("Initial commit").expect("Failed to commit");

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace =
            manager.discover_workspace(&root, &options).expect("Failed to discover workspace");

        (test_workspace, Rc::new(workspace))
    }

    #[test]
    fn test_change_tracker_creation() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());

        // Basic creation
        let tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // With Git user
        let tracker_with_git_user =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()))
                .with_git_user(Some("John Doe"), Some("john@example.com"));

        // We can't directly test the private workspace field, but we can test that
        // the trackers were created successfully
        assert!(tracker.store().get_all_changesets().is_ok());
        assert!(tracker_with_git_user.store().get_all_changesets().is_ok());
    }

    #[test]
    fn test_manual_change_recording() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create a change for a package that exists in the workspace
        let existing_pkg = "pkg-a";
        let change = Change::new(existing_pkg, ChangeType::Feature, "New feature", false);

        // Record the change
        let result = tracker.record_change(change);
        assert!(result.is_ok(), "Failed to record change: {:?}", result.err());

        // Verify the change was stored
        let unreleased = tracker.store().get_unreleased_changes(existing_pkg).unwrap();
        assert_eq!(unreleased.len(), 1);
        assert_eq!(unreleased[0].package, existing_pkg);
        assert!(matches!(unreleased[0].change_type, ChangeType::Feature));

        // Try recording a change for a non-existent package
        let non_existent_pkg = "non-existent-pkg";
        let invalid_change = Change::new(non_existent_pkg, ChangeType::Feature, "Feature", false);

        let result = tracker.record_change(invalid_change);
        assert!(result.is_err(), "Expected error when recording change for non-existent package");
    }

    #[test]
    fn test_creating_changeset() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create changes for existing packages
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 = Change::new("pkg-b", ChangeType::Fix, "Fix B", true);

        // Create a changeset with these changes
        let result =
            tracker.create_changeset(Some("Test changeset".to_string()), vec![change1, change2]);
        assert!(result.is_ok(), "Failed to create changeset: {:?}", result.err());

        let changeset = result.unwrap();
        assert_eq!(changeset.summary, Some("Test changeset".to_string()));
        assert_eq!(changeset.changes.len(), 2);

        // Verify the changeset was stored
        let all_changesets = tracker.store().get_all_changesets().unwrap();
        assert_eq!(all_changesets.len(), 1);

        // Changes should be retrievable by package
        let pkg_a_changes = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(pkg_a_changes.len(), 1);

        let pkg_b_changes = tracker.store().get_unreleased_changes("pkg-b").unwrap();
        assert_eq!(pkg_b_changes.len(), 1);
        assert!(pkg_b_changes[0].breaking);
    }

    #[test]
    fn test_unreleased_changes() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create and record changes with mixed release status
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 =
            Change::new("pkg-a", ChangeType::Fix, "Fix A", false).with_release_version("1.0.0");
        let change3 = Change::new("pkg-b", ChangeType::Feature, "Feature B", false);

        tracker.create_changeset(None, vec![change1, change2]).unwrap();
        tracker.create_changeset(None, vec![change3]).unwrap();

        // Get unreleased changes
        let unreleased = tracker.unreleased_changes().unwrap();

        // Should have unreleased changes for both packages
        assert_eq!(unreleased.len(), 2);
        assert!(unreleased.contains_key("pkg-a"));
        assert_eq!(unreleased["pkg-a"].len(), 1); // Only the unreleased change
        assert!(unreleased.contains_key("pkg-b"));
        assert_eq!(unreleased["pkg-b"].len(), 1);
    }

    #[test]
    fn test_mark_released() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create and record changes
        let change1 = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        let change2 = Change::new("pkg-a", ChangeType::Fix, "Fix A", false);

        tracker.create_changeset(None, vec![change1, change2]).unwrap();

        // Verify we have 2 unreleased changes for pkg-a
        let unreleased_before = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased_before.len(), 2);

        // Mark changes as released
        let marked = tracker.mark_released("pkg-a", "1.0.0", false).unwrap();
        assert_eq!(marked.len(), 2);

        // Verify changes are now released
        let unreleased_after = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased_after.len(), 0);

        let released = tracker.store().get_released_changes("pkg-a").unwrap();
        assert_eq!(released.len(), 2);
        assert!(released.iter().all(|c| c.release_version == Some("1.0.0".to_string())));
    }

    #[test]
    fn test_mark_released_dry_run() {
        let (_, workspace) = setup_test_workspace_with_git();
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create and record a change
        let change = Change::new("pkg-a", ChangeType::Feature, "Feature A", false);
        tracker.record_change(change).unwrap();

        // Do a dry run of mark_released
        let marked = tracker.mark_released("pkg-a", "1.0.0", true).unwrap();
        assert_eq!(marked.len(), 1);

        // Change should still be unreleased
        let unreleased = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(unreleased.len(), 1);
        assert!(unreleased[0].release_version.is_none());
    }
}
