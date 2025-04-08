use rstest::*;
use std::rc::Rc;
use tempfile::TempDir;

use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    Change, ChangeScope, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore, Workspace,
    WorkspaceManager,
};

mod fixtures;

// Helper function to create a workspace with a change tracker
fn setup_change_tracker(temp_dir: &TempDir) -> (Rc<Workspace>, ChangeTracker) {
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();

    // Use very permissive discovery options
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["**/package.json"]) // Any package.json
        .exclude_patterns(vec!["**/node_modules/**"]) // But not in node_modules
        .max_depth(10); // Go deep

    // Try to discover workspace
    match workspace_manager.discover_workspace(root_path, &options) {
        Ok(workspace) => {
            let workspace_rc = Rc::new(workspace);

            // Create very simple store - just in memory
            let store =
                Box::new(MemoryChangeStore::new()) as Box<dyn sublime_monorepo_tools::ChangeStore>;

            // Create change tracker
            let change_tracker = ChangeTracker::new(Rc::clone(&workspace_rc), store)
                .with_git_user(Some("sublime-bot"), Some("test-bot@websublime.com"));

            (workspace_rc, change_tracker)
        }
        Err(err) => {
            panic!("Failed to discover workspace: {err}");
        }
    }
}

#[rstest]
fn test_map_file_to_scope(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_workspace_rc, mut change_tracker) = setup_change_tracker(&temp_dir);

    // Use absolute paths to be sure
    let repo_path = temp_dir.path();
    let foo_file_path = repo_path.join("packages/package-foo/index.mjs");

    // Test mapping files to scopes using absolute paths
    let scope = change_tracker
        .map_file_to_scope(&foo_file_path.to_string_lossy())
        .expect("Failed to map file to scope");

    // Print the scope first, then do the assertion separately
    match &scope {
        ChangeScope::Package(name) => {
            assert_eq!(name, "@scope/package-foo", "Wrong package name detected");
        }
        _ => {
            panic!("Expected Package(@scope/package-foo), got {scope:?}");
        }
    }

    // Test root file
    let root_file = repo_path.join("package.json");
    let scope = change_tracker
        .map_file_to_scope(&root_file.to_string_lossy())
        .expect("Failed to map file to scope");

    assert!(matches!(scope, ChangeScope::Root), "Expected Root scope");

    // Test monorepo infrastructure file
    let scripts_dir = repo_path.join("scripts");
    std::fs::create_dir_all(&scripts_dir).expect("Failed to create scripts directory");
    let monorepo_file = scripts_dir.join("some-script.js");
    std::fs::write(&monorepo_file, "// test script").expect("Failed to create test script");

    let scope = change_tracker
        .map_file_to_scope(&monorepo_file.to_string_lossy())
        .expect("Failed to map file to scope");

    assert!(matches!(scope, ChangeScope::Monorepo), "Expected Monorepo scope");
}

#[rstest]
fn test_record_change(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir);

    // Create a change record
    let change = Change::new("@scope/package-foo", ChangeType::Feature, "Add new feature", false);

    // Record the change
    change_tracker.record_change(change.clone()).expect("Failed to record change");

    // Verify the change was recorded
    let unreleased = change_tracker
        .store()
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");

    assert_eq!(unreleased.len(), 1);
    assert_eq!(unreleased[0].package, "@scope/package-foo");
    assert!(matches!(unreleased[0].change_type, ChangeType::Feature));
    assert_eq!(unreleased[0].description, "Add new feature");
    assert!(!unreleased[0].breaking);
}

#[rstest]
fn test_mark_changes_as_released(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir);

    // Create and record changes
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add new feature", false);

    let change2 = Change::new("@scope/package-foo", ChangeType::Fix, "Fix bug", false);

    change_tracker.record_change(change1).expect("Failed to record change1");
    change_tracker.record_change(change2).expect("Failed to record change2");

    // Verify we have 2 unreleased changes
    let unreleased = change_tracker
        .store()
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");
    assert_eq!(unreleased.len(), 2);

    // Mark changes as released
    let released = change_tracker
        .mark_released("@scope/package-foo", "1.1.0", false)
        .expect("Failed to mark changes as released");
    assert_eq!(released.len(), 2);

    // Verify changes are now marked as released
    let still_unreleased = change_tracker
        .store()
        .get_unreleased_changes("@scope/package-foo")
        .expect("Failed to get unreleased changes");
    assert_eq!(still_unreleased.len(), 0);

    let released_changes = change_tracker
        .store()
        .get_released_changes("@scope/package-foo")
        .expect("Failed to get released changes");
    assert_eq!(released_changes.len(), 2);

    // Verify version is set correctly
    for change in &released_changes {
        assert_eq!(change.release_version.as_deref(), Some("1.1.0"));
    }
}

#[rstest]
fn test_detect_changes_between_refs(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let repo_path = temp_dir.path();
    let repo = Repo::open(repo_path.to_str().unwrap()).expect("Failed to open git repo");

    // Make sure we're on main branch
    repo.checkout("main").expect("Failed to checkout main branch");

    // First ensure that package-foo exists
    let foo_dir = repo_path.join("packages/package-foo");
    let foo_pkg_json = foo_dir.join("package.json");
    let foo_index_path = foo_dir.join("index.mjs");

    if !foo_dir.exists() || !foo_pkg_json.exists() || !foo_index_path.exists() {
        // Create the package-foo directory and files ourselves
        std::fs::create_dir_all(&foo_dir).expect("Failed to create package-foo dir");

        // Create package.json
        std::fs::write(
            &foo_pkg_json,
            r#"{
            "name": "@scope/package-foo",
            "version": "1.0.0",
            "description": "Awesome package foo",
            "main": "index.mjs"
        }"#,
        )
        .expect("Failed to write package.json");

        // Create index.mjs
        std::fs::write(&foo_index_path, r#"export const foo = "hello foo";"#)
            .expect("Failed to write index.mjs");

        // Add to git
        repo.add_all().expect("Failed to add files");
        repo.commit("feat: Add package-foo").expect("Failed to commit package-foo");
    }

    // Create a distinct branch for our changes
    let test_branch = "test-changes-branch";
    repo.create_branch(test_branch).expect("Failed to create branch");
    repo.checkout(test_branch).expect("Failed to checkout test branch");

    // Ensure the file exists
    assert!(foo_index_path.exists(), "File to modify doesn't exist");

    // Read and modify content
    let original_content =
        std::fs::read_to_string(&foo_index_path).expect("Failed to read original file");

    let modified_content =
        original_content + "\n// Modified for test\nexport const modified = true;\n";
    std::fs::write(&foo_index_path, modified_content).expect("Failed to modify file");

    // Add and commit the change
    repo.add_all().expect("Failed to add files");

    // Create workspace & change tracker
    let workspace_manager = WorkspaceManager::new();
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["packages/*/package.json"])
        .max_depth(5); // Ensure we search deep enough

    let workspace = workspace_manager
        .discover_workspace(repo_path, &options)
        .expect("Failed to discover workspace");

    let mut change_tracker = ChangeTracker::new(
        Rc::new(workspace),
        Box::new(MemoryChangeStore::new()) as Box<dyn sublime_monorepo_tools::ChangeStore>,
    )
    .with_git_user(Some("sublime-bot"), Some("test-bot@websublime.com"));

    // Get the detected changes
    let changes = change_tracker
        .detect_changes_between("main", Some(test_branch))
        .expect("Failed to detect changes");

    // Verify changes were detected
    assert!(!changes.is_empty(), "Should have detected at least one change");

    // Verify the change is attributed to package-foo, not monorepo
    let has_package_foo_change = changes.iter().any(|c| c.package == "@scope/package-foo");
    assert!(has_package_foo_change, "Should have detected change in package-foo specifically");

    // Make sure there are no monorepo changes incorrectly attributed
    let has_monorepo_change =
        changes.iter().any(|c| c.package == "monorepo" && c.description.contains("package-foo"));
    assert!(!has_monorepo_change, "Changes to package-foo should not be attributed to monorepo");
}

#[rstest]
fn test_create_changeset(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir);

    // Create changes
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add new feature", false);

    let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix bug", false);

    // Create a changeset with these changes
    let changeset = change_tracker
        .create_changeset(Some("Test changeset".to_string()), vec![change1, change2])
        .expect("Failed to create changeset");

    // Verify changeset content
    assert_eq!(changeset.changes.len(), 2);
    assert_eq!(changeset.changes[0].package, "@scope/package-foo");
    assert_eq!(changeset.changes[1].package, "@scope/package-bar");
    assert_eq!(changeset.summary.as_deref(), Some("Test changeset"));

    // Verify we can retrieve the changeset
    let stored_changeset = change_tracker
        .store()
        .get_changeset(&changeset.id)
        .expect("Failed to get changeset")
        .expect("Changeset not found");

    assert_eq!(stored_changeset.id, changeset.id);
    assert_eq!(stored_changeset.changes.len(), 2);
}

#[rstest]
fn test_unreleased_changes(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir);

    // Create and record changes for multiple packages
    let change1 = Change::new("@scope/package-foo", ChangeType::Feature, "Add new feature", false);

    let change2 = Change::new("@scope/package-bar", ChangeType::Fix, "Fix bug", false);

    let change3 = Change::new("@scope/package-foo", ChangeType::Refactor, "Refactor code", false);

    change_tracker.record_change(change1).expect("Failed to record change1");
    change_tracker.record_change(change2).expect("Failed to record change2");
    change_tracker.record_change(change3).expect("Failed to record change3");

    // Get all unreleased changes
    let unreleased = change_tracker.unreleased_changes().expect("Failed to get unreleased changes");

    // Verify we have changes for both packages
    assert!(unreleased.contains_key("@scope/package-foo"));
    assert!(unreleased.contains_key("@scope/package-bar"));

    // Verify counts
    assert_eq!(unreleased["@scope/package-foo"].len(), 2);
    assert_eq!(unreleased["@scope/package-bar"].len(), 1);
}
