mod fixtures;

#[cfg(test)]
mod tests {
    use crate::fixtures::npm_monorepo;
    use rstest::*;
    use std::fs;
    use std::path::Path;
    use std::rc::Rc;
    use sublime_monorepo_tools::{
        Change, ChangeTracker, ChangeType, ChangelogOptions, DiscoveryOptions, MemoryChangeStore,
        VersionManager, Workspace, WorkspaceConfig,
    };
    use tempfile::TempDir;

    // Helper to create a workspace
    fn setup_workspace(temp_dir: &TempDir) -> Result<Workspace, Box<dyn std::error::Error>> {
        let repo_path = temp_dir.path();

        // Create workspace configuration
        let config = WorkspaceConfig::new(repo_path.to_path_buf());

        // Open Git repo from the fixture
        let git_repo = Some(sublime_git_tools::Repo::open(repo_path.to_str().unwrap())?);

        // Create workspace with the Git repo
        let mut workspace = Workspace::new(repo_path.to_path_buf(), config, git_repo)?;

        // Discover packages
        let options = DiscoveryOptions::new().include_patterns(vec!["packages/*/package.json"]);
        workspace.discover_packages_with_options(&options)?;

        Ok(workspace)
    }

    #[rstest]
    fn test_changelog_generation(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for different packages with different types
        let changes = vec![
            // Breaking change
            Change::new("@scope/package-foo", ChangeType::Feature, "Add new API", true),
            // Feature
            Change::new("@scope/package-foo", ChangeType::Feature, "Add button component", false),
            // Fix
            Change::new("@scope/package-foo", ChangeType::Fix, "Fix layout bug", false),
            // Other packages
            Change::new("@scope/package-bar", ChangeType::Fix, "Fix security issue", false),
            Change::new("@scope/package-bar", ChangeType::Documentation, "Update docs", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Generate changelogs (dry run first)
        let default_options = ChangelogOptions::default();
        let changelogs = manager.generate_changelogs(&default_options, true)?;

        // Verify changelogs content
        assert!(
            changelogs.contains_key("@scope/package-foo"),
            "Should have changelog for package-foo"
        );
        assert!(
            changelogs.contains_key("@scope/package-bar"),
            "Should have changelog for package-bar"
        );

        // Check content of foo's changelog
        let foo_changelog = &changelogs["@scope/package-foo"];
        assert!(foo_changelog.contains("## Unreleased"));
        assert!(foo_changelog.contains("### Feature"));
        assert!(foo_changelog.contains("Add new API"));
        assert!(foo_changelog.contains("Add button component"));
        assert!(foo_changelog.contains("⚠️")); // Breaking change indicator
        assert!(foo_changelog.contains("### Fix"));
        assert!(foo_changelog.contains("Fix layout bug"));

        // Check content of bar's changelog
        let bar_changelog = &changelogs["@scope/package-bar"];
        assert!(bar_changelog.contains("## Unreleased"));
        assert!(bar_changelog.contains("### Fix"));
        assert!(bar_changelog.contains("Fix security issue"));
        assert!(bar_changelog.contains("### Docs"));
        assert!(bar_changelog.contains("Update docs"));

        // Generate with custom options
        let custom_options = ChangelogOptions {
            update_existing: true,
            filename: "CHANGES.md".to_string(),
            include_version_details: false,
            include_release_date: false,
            header_template: "# Release History\n\n".to_string(),
            change_template: "* {type}: {description} {breaking}\n".to_string(),
        };

        let custom_changelogs = manager.generate_changelogs(&custom_options, true)?;

        // Verify custom changelogs
        assert!(custom_changelogs.contains_key("@scope/package-foo"));
        let custom_foo = &custom_changelogs["@scope/package-foo"];
        assert!(custom_foo.contains("# Release History"));
        assert!(custom_foo.contains("* feature: Add new API ⚠️"));
        assert!(!custom_foo.contains("Released:"), "Release date should not be included");

        // Actually write changelogs to disk
        manager.generate_changelogs(&default_options, false)?;

        // Check that files were created on disk
        for pkg_name in ["@scope/package-foo", "@scope/package-bar"] {
            let pkg = workspace.get_package(pkg_name).expect("Package should exist");
            let package_borrow = pkg.borrow();
            let pkg_path = Path::new(&package_borrow.package_path);
            let changelog_path = pkg_path.join("CHANGELOG.md");

            assert!(changelog_path.exists(), "Changelog file should exist for {pkg_name}");

            let content = fs::read_to_string(&changelog_path)?;
            assert!(!content.is_empty(), "Changelog should not be empty");
            assert!(content.contains("## Unreleased"), "Changelog should have Unreleased section");
        }

        // Test updating existing changelogs

        // First create a file with existing content
        let foo_pkg = workspace.get_package("@scope/package-foo").expect("Package should exist");
        let foo_borrow = foo_pkg.borrow();
        let foo_path = Path::new(&foo_borrow.package_path);
        let foo_changelog_path = foo_path.join("CHANGELOG.md");

        // Write custom existing content
        let existing_content =
            "# Existing Changelog\n\nSome existing content\n\n## v1.0.0\n\n* Initial release\n";
        fs::write(&foo_changelog_path, existing_content)?;

        // Now generate changelogs with update_existing=true
        let update_options =
            ChangelogOptions { update_existing: true, ..ChangelogOptions::default() };

        manager.generate_changelogs(&update_options, false)?;

        // Read the merged content
        let merged_content = fs::read_to_string(&foo_changelog_path)?;

        // Should preserve existing header
        assert!(merged_content.contains("# Existing Changelog"));
        // And include both old and new content
        assert!(merged_content.contains("Some existing content"));
        assert!(merged_content.contains("## Unreleased"));

        Ok(())
    }

    #[rstest]
    fn test_markdown_content_generation(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker with changes
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::new(workspace.clone()), store);

        // Create changes for multiple versions - some released, some unreleased
        let unreleased_changes = vec![Change::new(
            "@scope/package-foo",
            ChangeType::Feature,
            "Unreleased feature",
            false,
        )];

        let v1_changes = vec![
            Change::new("@scope/package-foo", ChangeType::Feature, "Initial feature", false),
            Change::new("@scope/package-foo", ChangeType::Fix, "Fix bug in v1", false),
        ];

        // Record unreleased changes
        tracker.create_changeset(None, unreleased_changes)?;

        // Record released changes and mark as released
        tracker.create_changeset(None, v1_changes)?;
        tracker.mark_released("@scope/package-foo", "1.0.0", false)?;

        // Create version manager
        let manager = VersionManager::new(&workspace, Some(&tracker));

        // Generate changelogs with version information
        let options = ChangelogOptions {
            include_version_details: true,
            include_release_date: true,
            ..ChangelogOptions::default()
        };

        let changelogs = manager.generate_changelogs(&options, true)?;

        // Check content of foo's changelog
        let foo_changelog = &changelogs["@scope/package-foo"];

        // Should have Unreleased section first
        assert!(foo_changelog.contains("## Unreleased"));
        assert!(foo_changelog.contains("Unreleased feature"));

        // Then should have version 1.0.0 section
        assert!(foo_changelog.contains("## Version 1.0.0"));
        assert!(foo_changelog.contains("Initial feature"));
        assert!(foo_changelog.contains("Fix bug in v1"));

        // Should include date for released version
        assert!(foo_changelog.contains("*Released: "));

        // Version sections should be in correct order (unreleased first, then newest to oldest)
        let unreleased_pos = foo_changelog.find("## Unreleased").unwrap_or(0);
        let v1_pos = foo_changelog.find("## Version 1.0.0").unwrap_or(usize::MAX);

        assert!(unreleased_pos < v1_pos, "Unreleased section should come before version sections");

        Ok(())
    }
}
