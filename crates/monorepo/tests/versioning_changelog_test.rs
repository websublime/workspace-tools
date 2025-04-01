mod test_utils;

use std::fs;
use std::rc::Rc;
use sublime_monorepo_tools::{
    Change, ChangeTracker, ChangeType, ChangelogOptions, DiscoveryOptions, MemoryChangeStore,
    VersionManager, WorkspaceManager,
};
use tempfile::TempDir;
use test_utils::TestWorkspace;

#[cfg(test)]
mod versioning_changelog_tests {
    use super::*;

    fn setup_workspace_with_diverse_changes(
    ) -> (TestWorkspace, Rc<sublime_monorepo_tools::Workspace>, ChangeTracker) {
        // Create a test workspace with packages
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create change tracker with various types of changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add a variety of changes to package A
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Add new API", false))
            .expect("Failed to record change");
        tracker
            .record_change(
                Change::new("pkg-a", ChangeType::Fix, "Fix critical bug", true)
                    .with_release_version("1.0.0"),
            ) // Released change
            .expect("Failed to record change");
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Documentation, "Update README", false))
            .expect("Failed to record change");

        // Add changes to package B
        tracker
            .record_change(Change::new("pkg-b", ChangeType::Feature, "Redesign UI", true))
            .expect("Failed to record change");
        tracker
            .record_change(Change::new(
                "pkg-b",
                ChangeType::Performance,
                "Optimize rendering",
                false,
            ))
            .expect("Failed to record change");

        // Add changes to package C
        tracker
            .record_change(
                Change::new("pkg-c", ChangeType::Refactor, "Clean up code", false)
                    .with_release_version("0.9.0"),
            ) // Released change
            .expect("Failed to record change");

        (test_workspace, workspace, tracker)
    }

    #[test]
    fn test_generate_changelogs_basic() {
        let (_, workspace, tracker) = setup_workspace_with_diverse_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Create default changelog options
        let options = ChangelogOptions::new();

        // Generate changelogs with dry run
        let changelogs = version_manager
            .generate_changelogs(&options, true)
            .expect("Failed to generate changelogs");

        // Check that we got changelogs for the right packages
        assert!(changelogs.contains_key("pkg-a"));
        assert!(changelogs.contains_key("pkg-b"));
        assert!(changelogs.contains_key("pkg-c"));

        // Check content of pkg-a changelog
        let pkg_a_changelog = &changelogs["pkg-a"];

        // Should contain both unreleased and released changes
        assert!(pkg_a_changelog.contains("## Unreleased"));
        assert!(pkg_a_changelog.contains("## Version 1.0.0"));

        // Should contain all change types
        assert!(pkg_a_changelog.contains("### Feature"));
        assert!(pkg_a_changelog.contains("### Fix"));
        assert!(pkg_a_changelog.contains("### Documentation"));

        // Should contain change descriptions
        assert!(pkg_a_changelog.contains("Add new API"));
        assert!(pkg_a_changelog.contains("Fix critical bug"));
        assert!(pkg_a_changelog.contains("Update README"));

        // Breaking changes should have an indicator
        assert!(pkg_a_changelog.contains("⚠️"));

        // Check pkg-b changelog (should only have unreleased changes)
        let pkg_b_changelog = &changelogs["pkg-b"];
        assert!(pkg_b_changelog.contains("## Unreleased"));
        assert!(!pkg_b_changelog.contains("## Version"));
    }

    #[test]
    fn test_generate_changelogs_write_files() {
        // Create a temp directory for this test
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Create package directories in the temp dir
        fs::create_dir_all(temp_path.join("packages/pkg-a")).expect("Failed to create pkg-a dir");

        // Create a minimal package.json
        let pkg_json = r#"{"name": "pkg-a", "version": "1.0.0"}"#;
        fs::write(temp_path.join("packages/pkg-a/package.json"), pkg_json)
            .expect("Failed to write package.json");

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager.discover_workspace(&temp_path, &options).expect("Failed to discover workspace"),
        );

        // Create change tracker with a change
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add a change to pkg-a
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Test feature", false))
            .expect("Failed to record change");

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Generate and write changelogs
        let options = ChangelogOptions::new();
        version_manager
            .generate_changelogs(&options, false)
            .expect("Failed to generate changelogs");

        // Verify the changelog file was written
        let changelog_path = temp_path.join("packages/pkg-a/CHANGELOG.md");
        assert!(changelog_path.exists(), "Changelog file wasn't created");

        // Read the changelog and check its content
        let changelog_content =
            fs::read_to_string(&changelog_path).expect("Failed to read changelog file");

        assert!(changelog_content.contains("# Changelog"));
        assert!(changelog_content.contains("## Unreleased"));
        assert!(changelog_content.contains("Test feature"));
    }

    #[test]
    fn test_changelog_custom_options() {
        let (_, workspace, tracker) = setup_workspace_with_diverse_changes();

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Create custom changelog options
        let options = ChangelogOptions {
            update_existing: true,
            filename: "HISTORY.md".to_string(),
            include_version_details: false,
            include_release_date: false,
            header_template: "# Version History\n\n".to_string(),
            change_template: "* **{type}**: {description} {breaking}\n".to_string(),
        };

        // Generate changelogs with dry run
        let changelogs = version_manager
            .generate_changelogs(&options, true)
            .expect("Failed to generate changelogs");

        // Check content format based on custom options
        let pkg_a_changelog = &changelogs["pkg-a"];

        // Check custom header
        assert!(pkg_a_changelog.contains("# Version History"));

        // Version should be displayed without "Version" prefix
        assert!(pkg_a_changelog.contains("## 1.0.0"));
        assert!(!pkg_a_changelog.contains("## Version 1.0.0"));

        // Check custom change format
        assert!(pkg_a_changelog.contains("* **Feature**:"));
        assert!(pkg_a_changelog.contains("* **Fix**:"));
    }

    #[test]
    fn test_changelog_updating_existing() {
        // Create a temp directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Create package directory
        fs::create_dir_all(temp_path.join("packages/pkg-a")).expect("Failed to create pkg-a dir");

        // Create a minimal package.json
        let pkg_json = r#"{"name": "pkg-a", "version": "1.0.0"}"#;
        fs::write(temp_path.join("packages/pkg-a/package.json"), pkg_json)
            .expect("Failed to write package.json");

        // Create an existing changelog
        let existing_changelog = "# Changelog\n\nOld header content that should be preserved.\n\n## v0.9.0\n\n* Old change entry\n";
        fs::write(temp_path.join("packages/pkg-a/CHANGELOG.md"), existing_changelog)
            .expect("Failed to write existing changelog");

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager.discover_workspace(&temp_path, &options).expect("Failed to discover workspace"),
        );

        // Create change tracker with a change
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add a new change
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "New feature", false))
            .expect("Failed to record change");

        // Create version manager and generate changelog
        let version_manager = VersionManager::new(&workspace, Some(&tracker));
        let changelog_options = ChangelogOptions::new();

        version_manager
            .generate_changelogs(&changelog_options, false)
            .expect("Failed to generate changelog");

        // Read the updated changelog
        let updated_changelog = fs::read_to_string(temp_path.join("packages/pkg-a/CHANGELOG.md"))
            .expect("Failed to read updated changelog");

        // The old header should be preserved
        assert!(updated_changelog.contains("Old header content that should be preserved"));

        // The new content should be added
        assert!(updated_changelog.contains("## Unreleased"));
        assert!(updated_changelog.contains("New feature"));

        // The old content should still be there
        assert!(updated_changelog.contains("## v0.9.0"));
        assert!(updated_changelog.contains("Old change entry"));
    }
}
