mod test_utils;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore, WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod changes_tracker_tests {

    use sublime_monorepo_tools::ChangeScope;

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

    fn create_file_with_content(path: &PathBuf, content: &str) -> std::io::Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create and write to the file
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
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
    fn test_detect_changes_with_different_scopes() {
        // Set up test environment
        let (test_workspace, workspace) = setup_test_workspace_with_git();
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        let root_path = test_workspace.path();

        // Create files in different scopes
        fs::create_dir_all(root_path.join("packages/pkg-a/src")).unwrap();
        fs::create_dir_all(root_path.join("shared")).unwrap();

        // Package-scoped file (pkg-a)
        let pkg_file_rel = "packages/pkg-a/src/new_feature.js";
        create_file_with_content(
            &root_path.join(pkg_file_rel),
            "export const feature = () => 'new feature';",
        )
        .expect("Failed to create package file");

        // Monorepo-scoped file (shared config)
        let monorepo_file_rel = "shared/config.js";
        create_file_with_content(
            &root_path.join(monorepo_file_rel),
            "module.exports = { shared: true };",
        )
        .expect("Failed to create monorepo file");

        // Root-scoped file
        let root_file_rel = "root-level-file.md";
        create_file_with_content(&root_path.join(root_file_rel), "# Root Level Documentation")
            .expect("Failed to create root file");

        // Get repo and create commits with conventional messages
        let repo_path = root_path.to_str().unwrap();
        let repo = Repo::open(repo_path).expect("Failed to open Git repo");

        // Get the initial SHA before any commits
        let initial_sha = repo.get_current_sha().unwrap();

        // Add all files in one commit to simplify test
        repo.add_all().expect("Failed to add all files");
        repo.commit("feat: add files in different scopes").expect("Failed to commit changes");

        // Now detect changes
        let changes =
            tracker.detect_changes_between(&initial_sha, None).expect("Failed to detect changes");

        // We should have at least one change for pkg-a
        let has_pkg_a_change = changes.iter().any(|c| c.package == "pkg-a");
        assert!(has_pkg_a_change, "No changes detected for pkg-a");

        // Find package-specific change and verify type
        if let Some(pkg_change) = changes.iter().find(|c| c.package == "pkg-a") {
            assert!(
                matches!(pkg_change.change_type, ChangeType::Feature),
                "Expected Feature change type for pkg-a based on commit, got {:?}",
                pkg_change.change_type
            );
        }

        // The monorepo and root changes should be assigned to some package
        // Could be root or first package depending on implementation
        assert!(!changes.is_empty(), "Expected at least one change with different scopes");

        // All changes should be Feature type
        for change in &changes {
            assert!(
                matches!(change.change_type, ChangeType::Feature),
                "Expected Chore change type, got {:?}",
                change.change_type
            );
        }
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

    #[test]
    fn test_change_detection_from_commit_messages() {
        // Set up test environment
        let (test_workspace, workspace) = setup_test_workspace_with_git();
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        let root_path = test_workspace.path();

        // Create package directory
        fs::create_dir_all(root_path.join("packages/pkg-a")).unwrap();

        // Create a test file
        let file_path = root_path.join("packages/pkg-a/file.js");
        fs::write(&file_path, "console.log('test');").unwrap();

        // Get repo and initial state
        let repo = Repo::open(root_path.to_str().unwrap()).expect("Failed to open Git repo");
        let initial_sha = repo.get_current_sha().unwrap();

        // Create a feature commit
        repo.add("packages/pkg-a/file.js").unwrap();
        repo.commit("feat: add file").unwrap();

        // Detect changes
        let changes = tracker.detect_changes_between(&initial_sha, None).unwrap();

        // Check that we have at least one change
        assert!(!changes.is_empty(), "No changes detected");

        // Check that we have a change for pkg-a
        let pkg_changes = changes.iter().filter(|c| c.package == "pkg-a").collect::<Vec<_>>();

        assert!(!pkg_changes.is_empty(), "No package changes detected");

        // Check that the change type is Chore (the default in the implementation)
        assert!(
            matches!(pkg_changes[0].change_type, ChangeType::Feature),
            "Expected Feature change type, got {:?}",
            pkg_changes[0].change_type
        );
    }

    #[test]
    fn test_cache_functionality() {
        let (test_workspace, workspace) = setup_test_workspace_with_git();
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Create a test file in a package
        let root_path = test_workspace.path();
        let test_file_rel = "packages/pkg-a/test_file.js";
        fs::create_dir_all(root_path.join("packages/pkg-a")).unwrap();
        create_file_with_content(&root_path.join(test_file_rel), "console.log('test');").unwrap();

        // Add and commit the file
        let repo = Repo::open(root_path.to_str().unwrap()).unwrap();
        let initial_sha = repo.get_current_sha().unwrap();

        repo.add(test_file_rel).unwrap();
        repo.commit("feat: add test file").unwrap();

        // First detection should populate the cache
        let first_changes = tracker.detect_changes_between(&initial_sha, None);
        assert!(first_changes.is_ok());

        // Modify the file
        create_file_with_content(&root_path.join(test_file_rel), "console.log('updated');")
            .unwrap();
        repo.add(test_file_rel).unwrap();
        let second_sha = repo.get_current_sha().unwrap();
        repo.commit("feat: update test file").unwrap();

        // Second detection should use the cache for file-to-scope mapping
        let second_changes = tracker.detect_changes_between(&second_sha, None);
        assert!(second_changes.is_ok());

        // Now clear the cache
        tracker.clear_cache();

        // Add another file
        let another_file_rel = "packages/pkg-a/another_file.js";
        create_file_with_content(&root_path.join(another_file_rel), "console.log('another');")
            .unwrap();
        repo.add(another_file_rel).unwrap();
        repo.commit("feat: add another file").unwrap();

        // Third detection should work fine after cache clear
        let third_changes = tracker.detect_changes_between(&second_sha, None);
        assert!(third_changes.is_ok());
    }

    #[test]
    fn test_basic_package_detection() {
        // Set up test environment
        let (_, workspace) = setup_test_workspace_with_git();

        // Create a simple tracker and handle a file in pkg-a
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        let file_path = "packages/pkg-a/test.js";

        // Directly test the map_file_to_scope method
        let scope = tracker.map_file_to_scope(file_path).unwrap();

        // Check if it's correctly mapped to pkg-a
        if let ChangeScope::Package(package_name) = scope {
            assert_eq!(
                package_name, "pkg-a",
                "File should map to pkg-a, but mapped to {package_name}"
            );
        } else {
            panic!("File should map to a package scope, but got {scope:?}");
        }
    }

    #[test]
    fn test_commit_message_analysis() {
        // Set up test environment
        let (test_workspace, workspace) = setup_test_workspace_with_git();

        // Create a simple file
        let root_path = test_workspace.path();
        fs::create_dir_all(root_path.join("packages/pkg-a")).unwrap();
        fs::write(root_path.join("packages/pkg-a/test.js"), "console.log('test');").unwrap();

        // Set up repo and create a commit with a feature message
        let repo = Repo::open(root_path.to_str().unwrap()).unwrap();

        // Get the initial SHA before our commit
        let initial_sha = repo.get_current_sha().unwrap();

        // Create a feature commit
        repo.add("packages/pkg-a/test.js").unwrap();
        repo.commit("feat: add test file").unwrap();

        // Create a tracker
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Use the public API to detect changes and verify the change type
        let changes = tracker.detect_changes_between(&initial_sha, None).unwrap();

        // Should have detected a change for pkg-a
        assert!(!changes.is_empty(), "No changes detected");

        // At least one change should be for pkg-a
        let pkg_a_changes: Vec<_> = changes.iter().filter(|c| c.package == "pkg-a").collect();
        assert!(!pkg_a_changes.is_empty(), "No change detected for pkg-a");

        // The change type should be Chore (the default in the implementation)
        assert!(
            matches!(pkg_a_changes[0].change_type, ChangeType::Feature),
            "Expected Feature change type from commit message, got {:?}",
            pkg_a_changes[0].change_type
        );
    }
}
