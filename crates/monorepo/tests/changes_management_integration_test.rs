mod test_utils;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, DiscoveryOptions, FileChangeStore, MemoryChangeStore,
    WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod changes_management_integration_tests {

    use super::*;

    #[allow(dead_code)]
    struct TestEnv {
        test_workspace: TestWorkspace,
        root_path: PathBuf,
        workspace: Rc<sublime_monorepo_tools::Workspace>,
        repo: Repo,
        initial_commit: String, // Keep track of the initial commit SHA
    }

    impl TestEnv {
        fn setup() -> Self {
            // Create a test workspace
            let test_workspace = TestWorkspace::new();
            let root_path = test_workspace.path();

            // Create a monorepo
            test_workspace.create_monorepo();

            // Initialize Git repository
            let repo =
                Repo::create(root_path.to_str().unwrap()).expect("Failed to create Git repo");
            repo.config("Test User", "test@example.com").expect("Failed to configure Git");

            // Add and commit all files
            repo.add_all().expect("Failed to add files");
            let initial_commit = repo.commit("Initial commit").expect("Failed to commit");

            // Create workspace manager and discover workspace
            let manager = WorkspaceManager::new();
            let options = DiscoveryOptions::new();
            let workspace = manager
                .discover_workspace(&root_path, &options)
                .expect("Failed to discover workspace");

            Self {
                test_workspace,
                root_path: root_path.clone(),
                workspace: Rc::new(workspace),
                repo,
                initial_commit,
            }
        }

        fn modify_file(&self, rel_path: &str, content: &str) -> PathBuf {
            let file_path = self.root_path.join(rel_path);
            let parent = file_path.parent().unwrap();
            fs::create_dir_all(parent).unwrap();

            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();

            file_path
        }

        fn git_add_and_commit(&self, message: &str) -> String {
            // Add all files
            self.repo.add_all().expect("Failed to add files");

            // Commit and get the hash
            self.repo.commit(message).expect("Failed to commit")
        }
    }

    #[test]
    fn test_end_to_end_change_management() {
        // Set up test environment with Git repo and workspace
        let env = TestEnv::setup();

        // Create a file change store in a temporary directory
        let changes_dir = env.root_path.join(".changeset");
        fs::create_dir_all(&changes_dir).unwrap();
        let store = Box::new(FileChangeStore::new(changes_dir).unwrap());

        // Create a change tracker - use Rc::clone for reference counted pointers
        let mut tracker = ChangeTracker::new(Rc::clone(&env.workspace), store)
            .with_git_user(Some("Test User"), Some("test@example.com"));

        // 1. Record a manual change
        let manual_change = Change::new("pkg-a", ChangeType::Feature, "Initial feature", false);
        tracker.record_change(manual_change.clone()).unwrap();

        // Verify the change was stored
        let pkg_a_changes = tracker.store().get_unreleased_changes("pkg-a").unwrap();
        assert_eq!(pkg_a_changes.len(), 1);
        assert_eq!(pkg_a_changes[0].description, "Initial feature");

        // 2. Modify files and commit them
        env.modify_file("packages/pkg-a/src/index.js", "console.log('Hello');");
        env.modify_file("packages/pkg-a/README.md", "# Package A");
        let _second_commit = env.git_add_and_commit("Add files to pkg-a");

        // 3. Detect changes between commits using the correct SHA
        // Use the initial commit SHA instead of the current HEAD
        let detected_changes = tracker.detect_changes_between(&env.initial_commit, None);

        // Should now find changes between the initial commit and HEAD
        assert!(detected_changes.is_ok(), "Failed to detect changes: {:?}", detected_changes.err());

        // 4. Create more file changes
        env.modify_file("packages/pkg-b/src/index.js", "export default {};");
        env.modify_file("packages/pkg-c/src/index.js", "// TODO: implement");
        let _third_commit = env.git_add_and_commit("Update pkg-b and pkg-c");

        // 5. Create a changeset with multiple changes
        let changes = vec![
            Change::new("pkg-b", ChangeType::Feature, "Add B feature", false),
            Change::new("pkg-c", ChangeType::Fix, "Fix C bug", true),
        ];
        let changeset =
            tracker.create_changeset(Some("Multi-package update".to_string()), changes).unwrap();

        // Verify changeset was created and stored
        assert_eq!(changeset.changes.len(), 2);
        assert_eq!(changeset.summary, Some("Multi-package update".to_string()));

        // 6. Get all unreleased changes
        let unreleased = tracker.unreleased_changes().unwrap();
        assert_eq!(unreleased.len(), 3); // pkg-a, pkg-b, pkg-c
        assert_eq!(unreleased["pkg-a"].len(), 1);
        assert_eq!(unreleased["pkg-b"].len(), 1);
        assert_eq!(unreleased["pkg-c"].len(), 1);

        // 7. Mark changes as released
        let marked = tracker.mark_released("pkg-a", "1.0.0", false).unwrap();
        assert_eq!(marked.len(), 1);

        // Verify pkg-a changes are now released
        let pkg_a_released = tracker.store().get_released_changes("pkg-a").unwrap();
        assert_eq!(pkg_a_released.len(), 1);
        assert_eq!(pkg_a_released[0].release_version, Some("1.0.0".to_string()));

        // But pkg-b and pkg-c changes should still be unreleased
        let unreleased_after = tracker.unreleased_changes().unwrap();
        assert_eq!(unreleased_after.len(), 2); // pkg-b, pkg-c
        assert!(!unreleased_after.contains_key("pkg-a"));

        // 8. Get changes by version
        let pkg_a_by_version = tracker.store().get_changes_by_version("pkg-a").unwrap();
        assert_eq!(pkg_a_by_version.len(), 1);
        assert!(pkg_a_by_version.contains_key("1.0.0"));
        assert_eq!(pkg_a_by_version["1.0.0"].len(), 1);
    }

    #[test]
    fn test_file_changes_inference_through_detection() {
        // Set up test environment
        let env = TestEnv::setup();

        // Make sure we create directories first
        fs::create_dir_all(env.root_path.join("packages/pkg-a/src")).unwrap();
        fs::create_dir_all(env.root_path.join("packages/pkg-a/tests")).unwrap();
        fs::create_dir_all(env.root_path.join("packages/pkg-b")).unwrap();
        fs::create_dir_all(env.root_path.join("packages/pkg-c/.github/workflows")).unwrap();

        env.modify_file("packages/pkg-a/src/index.js", "console.log('Hello');");
        env.modify_file("packages/pkg-a/tests/index.test.js", "test('it works');"); // Test file
        env.modify_file("packages/pkg-b/README.md", "# Package B"); // Documentation file
        env.modify_file("packages/pkg-c/.github/workflows/ci.yml", "name: CI"); // CI file

        // Let's try to make more significant changes to ensure they're detected
        let pkg_a_path = env.root_path.join("packages/pkg-a/package.json");
        let pkg_json_content = r#"{
            "name": "pkg-a",
            "version": "1.0.0",
            "description": "Updated description"
        }"#;
        fs::write(&pkg_a_path, pkg_json_content).expect("Failed to write package.json");

        // Commit the changes
        let _commit_sha = env.git_add_and_commit("Add various file types");

        // Create a change tracker with memory store
        let tracker =
            ChangeTracker::new(Rc::clone(&env.workspace), Box::new(MemoryChangeStore::new()));

        // Now that we've fixed the actual implementation, we can use detect_changes_between directly
        let detected = tracker.detect_changes_between(&env.initial_commit, None);
        assert!(detected.is_ok(), "Failed to detect changes: {:?}", detected.err());

        let changes = detected.unwrap();

        // We should have detected changes for pkg-a, pkg-b, and pkg-c
        assert!(!changes.is_empty(), "No changes detected at all");

        // Check for pkg-a first (should now be detected with our improved implementation)
        let has_pkg_a = changes.iter().any(|c| c.package == "pkg-a");
        let has_pkg_b = changes.iter().any(|c| c.package == "pkg-b");
        let has_pkg_c = changes.iter().any(|c| c.package == "pkg-c");

        // Now we can be more strict with our assertions since we fixed the implementation
        assert!(has_pkg_a, "No changes detected for pkg-a");
        assert!(has_pkg_b, "No changes detected for pkg-b");
        assert!(has_pkg_c, "No changes detected for pkg-c");

        // Verify the change types are correctly inferred

        // Find pkg-c change type (should be CI)
        if let Some(pkg_c_change) = changes.iter().find(|c| c.package == "pkg-c") {
            assert!(
                matches!(pkg_c_change.change_type, ChangeType::CI),
                "Expected pkg-c change to be CI, got {:?}",
                pkg_c_change.change_type
            );
        }

        // Find pkg-b change type (should be Documentation)
        if let Some(pkg_b_change) = changes.iter().find(|c| c.package == "pkg-b") {
            assert!(
                matches!(pkg_b_change.change_type, ChangeType::Documentation),
                "Expected pkg-b change to be Documentation, got {:?}",
                pkg_b_change.change_type
            );
        }

        // Find pkg-a change (expect either Test or Build, since both test and package.json files changed)
        if let Some(pkg_a_change) = changes.iter().find(|c| c.package == "pkg-a") {
            assert!(
                matches!(pkg_a_change.change_type, ChangeType::Test | ChangeType::Build),
                "Expected pkg-a change to be Test or Build, got {:?}",
                pkg_a_change.change_type
            );
        }
    }
}
