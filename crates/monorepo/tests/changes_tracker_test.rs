use rstest::*;
use std::rc::Rc;
use tempfile::TempDir;

use sublime_git_tools::Repo;
use sublime_monorepo_tools::{
    Change, ChangeScope, ChangeTracker, ChangeType, DiscoveryOptions, FileChangeStore,
    MemoryChangeStore, Workspace, WorkspaceManager,
};

mod fixtures;

// Helper function to create a workspace with a change tracker
fn setup_change_tracker(
    temp_dir: &TempDir,
    use_file_store: bool,
) -> (Rc<Workspace>, ChangeTracker) {
    // Use WorkspaceManager to properly discover the workspace
    let workspace_manager = WorkspaceManager::new();
    let root_path = temp_dir.path();
    println!("Setting up workspace at: {}", root_path.display());

    // First, verify all expected packages exist in the filesystem
    println!("Verifying fixture packages:");
    for pkg_name in &["foo", "bar", "baz", "charlie", "major", "tom"] {
        let pkg_path = root_path.join(format!("packages/package-{}", pkg_name));
        let pkg_json = pkg_path.join("package.json");
        if pkg_path.exists() && pkg_json.exists() {
            println!("  ✅ Package {} exists at {}", pkg_name, pkg_path.display());
        } else {
            println!("  ❌ Package {} NOT FOUND at {}", pkg_name, pkg_path.display());
        }
    }

    // Use more specific discovery options
    let options = DiscoveryOptions::default()
        .auto_detect_root(false) // We know the root path
        .include_patterns(vec!["packages/*/package.json"]) // More specific pattern
        .max_depth(5); // Ensure we go deep enough

    let workspace = workspace_manager
        .discover_workspace(root_path, &options)
        .expect("Failed to discover workspace");

    println!("Discovered workspace with {} packages", workspace.sorted_packages().len());

    // Print all packages
    for pkg_info in workspace.sorted_packages() {
        let pkg = pkg_info.borrow();
        println!("  Package: {} at {}", pkg.package.borrow().name(), pkg.package_path);
    }

    let workspace_rc = Rc::new(workspace);

    // Set up change store
    let store = if use_file_store {
        let changes_dir = temp_dir.path().join(".changes");
        let file_store = FileChangeStore::new(&changes_dir).expect("Failed to create file store");
        Box::new(file_store) as Box<dyn sublime_monorepo_tools::ChangeStore>
    } else {
        Box::new(MemoryChangeStore::new()) as Box<dyn sublime_monorepo_tools::ChangeStore>
    };

    // Create change tracker
    let change_tracker = ChangeTracker::new(Rc::clone(&workspace_rc), store)
        .with_git_user(Some("sublime-bot"), Some("test-bot@websublime.com"));

    (workspace_rc, change_tracker)
}

#[rstest]
fn test_map_file_to_scope(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_workspace_rc, mut change_tracker) = setup_change_tracker(&temp_dir, false);

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
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir, false);

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
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir, false);

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

    println!("Working with repo at: {}", repo_path.display());

    // Print initial git status and branches
    let initial_status = repo.status_porcelain().expect("Failed to get initial status");
    println!("Initial git status: {:?}", initial_status);

    let branches = repo.list_branches().expect("Failed to list branches");
    println!("Available branches: {:?}", branches);

    // Make sure we're on main branch
    repo.checkout("main").expect("Failed to checkout main branch");

    // First ensure that package-foo exists
    let foo_dir = repo_path.join("packages/package-foo");
    let foo_pkg_json = foo_dir.join("package.json");
    let foo_index_path = foo_dir.join("index.mjs");

    if !foo_dir.exists() || !foo_pkg_json.exists() || !foo_index_path.exists() {
        println!("package-foo missing! Attempting to create it from scratch");

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
        repo.commit("feat: recreate package-foo").expect("Failed to commit");
    }

    // Create a distinct branch for our changes
    let test_branch = "test-changes-branch";
    repo.create_branch(test_branch).expect("Failed to create branch");
    repo.checkout(test_branch).expect("Failed to checkout test branch");

    // CRITICAL: Make changes to an existing file
    println!("Modifying existing file: {}", foo_index_path.display());

    // Ensure the file exists
    assert!(foo_index_path.exists(), "File to modify doesn't exist");

    // Read and modify content
    let original_content =
        std::fs::read_to_string(&foo_index_path).expect("Failed to read original file");
    println!("Original content: {}", original_content);

    let modified_content =
        original_content + "\n// Modified for test\nexport const modified = true;\n";
    std::fs::write(&foo_index_path, modified_content).expect("Failed to modify file");

    // Verify modification
    let new_content =
        std::fs::read_to_string(&foo_index_path).expect("Failed to read modified file");
    println!("Modified content: {}", new_content);

    // Add and commit the change
    repo.add_all().expect("Failed to add files");
    let commit_sha = repo.commit("feat: modify file in package-foo").expect("Failed to commit");
    println!("Created commit: {}", commit_sha);

    // Get status after commit to verify
    let post_commit_status = repo.status_porcelain().expect("Failed to get post-commit status");
    println!("Status after commit: {:?}", post_commit_status);

    // Create workspace with all packages
    // Use WorkspaceManager directly with custom options
    let workspace_manager = WorkspaceManager::new();

    // Use a more specific pattern to ensure we find all packages
    let options = DiscoveryOptions::default()
        .auto_detect_root(false)
        .include_patterns(vec!["packages/*/package.json"]);

    let workspace = workspace_manager
        .discover_workspace(repo_path, &options)
        .expect("Failed to discover workspace");

    println!("Discovered {} packages", workspace.sorted_packages().len());

    for pkg_info in workspace.sorted_packages() {
        let pkg = pkg_info.borrow();
        println!("  - {} at {}", pkg.package.borrow().name(), pkg.package_path);
    }

    // Create a change tracker with this workspace
    let mut change_tracker = ChangeTracker::new(
        Rc::new(workspace),
        Box::new(MemoryChangeStore::new()) as Box<dyn sublime_monorepo_tools::ChangeStore>,
    )
    .with_git_user(Some("sublime-bot"), Some("test-bot@websublime.com"));

    // Test file mapping directly
    println!("\nDirect file mapping test:");
    let foo_index_rel_path = "packages/package-foo/index.mjs";
    let mapping = change_tracker.map_file_to_scope(foo_index_rel_path);
    println!("Mapping result for {}: {:?}", foo_index_rel_path, mapping);

    // Detect changes from main to test branch
    println!("\nDetecting changes between main and {}", test_branch);

    // First check if Git can detect the changes
    let git_diff = repo.get_all_files_changed_since_sha("main").expect("Failed to get git diff");
    println!("Git detects these changed files: {:?}", git_diff);

    if git_diff.is_empty() {
        panic!("Git couldn't detect any changes between main and {}", test_branch);
    }

    // Now try our change detection
    let changes = match change_tracker.detect_changes_between("main", Some(test_branch)) {
        Ok(changes) => changes,
        Err(err) => {
            panic!("Failed to detect changes: {:?}", err);
        }
    };

    // Print all detected changes
    println!("\nDetected changes:");
    for change in &changes {
        println!(
            "  Package: {}, Type: {:?}, Description: {}",
            change.package, change.change_type, change.description
        );
    }

    // Verify changes were detected
    assert!(!changes.is_empty(), "Should have detected at least one change");

    // Verify package assignment
    let has_foo_change = changes.iter().any(|c| c.package == "@scope/package-foo");
    assert!(has_foo_change, "Should have detected change in package-foo");
}

#[rstest]
fn test_create_changeset(#[from(fixtures::npm_monorepo)] temp_dir: TempDir) {
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir, false);

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
    let (_, mut change_tracker) = setup_change_tracker(&temp_dir, false);

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
