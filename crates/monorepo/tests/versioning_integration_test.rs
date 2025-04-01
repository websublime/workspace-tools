mod test_utils;

use std::fs;
use std::rc::Rc;
use sublime_monorepo_tools::{
    BumpType, Change, ChangeTracker, ChangeType, ChangelogOptions, DiscoveryOptions,
    FileChangeStore, MemoryChangeStore, VersionBumpStrategy, VersionManager, WorkspaceManager,
};
use tempfile::TempDir;
use test_utils::TestWorkspace;

#[cfg(test)]
mod versioning_integration_tests {
    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_versioning_end_to_end() {
        // Create a test workspace with temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_path_buf();

        // Create package directories and files
        fs::create_dir_all(temp_path.join("packages/pkg-a")).expect("Failed to create dir");
        fs::create_dir_all(temp_path.join("packages/pkg-b")).expect("Failed to create dir");
        fs::create_dir_all(temp_path.join("packages/pkg-c")).expect("Failed to create dir");

        // Create package.json files
        fs::write(
            temp_path.join("packages/pkg-a/package.json"),
            r#"{"name": "pkg-a", "version": "1.0.0"}"#,
        )
        .expect("Failed to write package.json");

        fs::write(
            temp_path.join("packages/pkg-b/package.json"),
            r#"{"name": "pkg-b", "version": "1.0.0", "dependencies": {"pkg-a": "^1.0.0"}}"#,
        )
        .expect("Failed to write package.json");

        fs::write(
            temp_path.join("packages/pkg-c/package.json"),
            r#"{"name": "pkg-c", "version": "1.0.0", "dependencies": {"pkg-b": "^1.0.0"}}"#,
        )
        .expect("Failed to write package.json");

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager.discover_workspace(&temp_path, &options).expect("Failed to discover workspace"),
        );

        // Create changeset directory for tracking changes
        let changeset_dir = temp_path.join(".changeset");
        fs::create_dir_all(&changeset_dir).expect("Failed to create changeset dir");

        // Create change tracker with persistent storage
        let store = Box::new(
            FileChangeStore::new(changeset_dir).expect("Failed to create file change store"),
        );
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Record changes for different packages
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Add new API", false))
            .expect("Failed to record change");

        tracker
            .record_change(Change::new("pkg-b", ChangeType::Fix, "Fix critical bug", true))
            .expect("Failed to record change");

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Step 1: Preview version bumps with independent strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let preview = version_manager.preview_bumps(&strategy).expect("Failed to preview bumps");

        // Verify preview contents (pkg-a: minor, pkg-b: major, pkg-c: should get patch update due to dependency)
        assert_eq!(preview.changes.len(), 3);

        // Find changes by package name
        let pkg_a_change = preview
            .changes
            .iter()
            .find(|c| c.package_name == "pkg-a")
            .expect("pkg-a change not found in preview");
        let pkg_b_change = preview
            .changes
            .iter()
            .find(|c| c.package_name == "pkg-b")
            .expect("pkg-b change not found in preview");
        let pkg_c_change = preview
            .changes
            .iter()
            .find(|c| c.package_name == "pkg-c")
            .expect("pkg-c change not found in preview");

        assert_eq!(pkg_a_change.bump_type, BumpType::Minor);
        assert_eq!(pkg_b_change.bump_type, BumpType::Major);
        assert_eq!(pkg_c_change.bump_type, BumpType::Patch); // Dependency update

        // Step 2: Apply version bumps
        let changes = {
            // Create version manager in a new scope so it's dropped after use
            let version_manager = VersionManager::new(&workspace, Some(&tracker));

            let changes =
                version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

            assert_eq!(changes.len(), 3);
            changes
        }; // version_manager is dropped here, releasing the immutable borrow of tracker

        // Step 3: Mark changes as released
        VersionManager::mark_changes_as_released(&mut tracker, &changes, false)
            .expect("Failed to mark changes as released");

        // Step 4: Generate changelogs (create a new version manager)
        let changelog_options = ChangelogOptions::new();
        let _changelogs = {
            let version_manager = VersionManager::new(&workspace, Some(&tracker));
            version_manager
                .generate_changelogs(&changelog_options, false)
                .expect("Failed to generate changelogs")
        };

        // Verify changelogs were created for all packages
        let changelog_files = vec![
            temp_path.join("packages/pkg-a/CHANGELOG.md"),
            temp_path.join("packages/pkg-b/CHANGELOG.md"),
            temp_path.join("packages/pkg-c/CHANGELOG.md"),
        ];

        for path in changelog_files {
            assert!(path.exists(), "Changelog not created at {}", path.display());

            // Read the changelog content
            let content = fs::read_to_string(&path).expect("Failed to read changelog");

            // Verify basic structure
            assert!(content.contains("# Changelog"), "Missing header in {}", path.display());

            // All changelogs should have a version section since we marked changes as released
            assert!(
                !content.contains("## Unreleased"),
                "Should not contain unreleased section in {}",
                path.display()
            );

            // The version section should contain the new version
            if path.to_string_lossy().contains("pkg-a") {
                assert!(content.contains("## Version 1.1.0"), "Wrong version in pkg-a changelog");
                assert!(content.contains("Add new API"), "Missing change in pkg-a changelog");
            } else if path.to_string_lossy().contains("pkg-b") {
                assert!(content.contains("## Version 2.0.0"), "Wrong version in pkg-b changelog");
                assert!(content.contains("Fix critical bug"), "Missing change in pkg-b changelog");
                assert!(
                    content.contains("⚠️"),
                    "Missing breaking change indicator in pkg-b changelog"
                );
            } else if path.to_string_lossy().contains("pkg-c") {
                assert!(content.contains("## Version 1.0.1"), "Wrong version in pkg-c changelog");
                assert!(
                    content.contains("DependencyUpdate"),
                    "Missing dependency update reason in pkg-c changelog"
                );
            }
        }

        // Step 5: Verify package.json files were updated with new versions
        let pkg_a_json = fs::read_to_string(temp_path.join("packages/pkg-a/package.json"))
            .expect("Failed to read pkg-a package.json");
        let pkg_b_json = fs::read_to_string(temp_path.join("packages/pkg-b/package.json"))
            .expect("Failed to read pkg-b package.json");
        let pkg_c_json = fs::read_to_string(temp_path.join("packages/pkg-c/package.json"))
            .expect("Failed to read pkg-c package.json");

        assert!(
            pkg_a_json.contains("\"version\": \"1.1.0\""),
            "pkg-a version not updated correctly"
        );
        assert!(
            pkg_b_json.contains("\"version\": \"2.0.0\""),
            "pkg-b version not updated correctly"
        );
        assert!(
            pkg_c_json.contains("\"version\": \"1.0.1\""),
            "pkg-c version not updated correctly"
        );

        // Step 6: Verify dependencies were updated
        assert!(
            pkg_b_json.contains("\"pkg-a\": \"^1.1.0\""),
            "pkg-b dependency on pkg-a not updated"
        );
        assert!(
            pkg_c_json.contains("\"pkg-b\": \"^2.0.0\""),
            "pkg-c dependency on pkg-b not updated"
        );

        // Step 7: Verify no unreleased changes remain
        let unreleased_changes =
            tracker.unreleased_changes().expect("Failed to get unreleased changes");

        assert!(
            unreleased_changes.is_empty(),
            "There should be no unreleased changes after version bump"
        );
    }

    #[test]
    fn test_synchronized_versioning() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create change tracker
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add changes to different packages
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Feature in A", false))
            .expect("Failed to record change");

        tracker
            .record_change(Change::new("pkg-b", ChangeType::Fix, "Fix in B", false))
            .expect("Failed to record change");

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Use synchronized strategy with a specific version
        let strategy = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };

        // Apply version bumps
        let _changes =
            version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

        // All packages should get the same version
        for pkg_name in &["pkg-a", "pkg-b", "pkg-c"] {
            if let Some(pkg) = workspace.get_package(pkg_name) {
                let version = pkg.borrow().package.borrow().version_str();
                assert_eq!(version, "2.0.0", "Package {pkg_name} should have version 2.0.0");
            }
        }

        // Dependencies should reflect the new version
        for pkg_info in workspace.sorted_packages() {
            let info = pkg_info.borrow();
            let pkg = info.package.borrow();

            for dep in pkg.dependencies() {
                let dep_borrow = dep.borrow();
                let dep_name = dep_borrow.name();
                if workspace.get_package(dep_name).is_some() {
                    // If this is an internal dependency, it should point to version 2.0.0
                    if let Ok(fixed_version) = dep.borrow().fixed_version() {
                        assert_eq!(
                            fixed_version.to_string(),
                            "2.0.0",
                            "Dependency {dep_name} should use version 2.0.0"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_manual_versioning() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create change tracker
        let tracker = ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Create version manager
        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // Use manual strategy with specific versions
        let mut versions = std::collections::HashMap::new();
        versions.insert("pkg-a".to_string(), "3.0.0".to_string());
        versions.insert("pkg-b".to_string(), "4.0.0".to_string());

        let strategy = VersionBumpStrategy::Manual(versions);

        // Apply version bumps
        let changes = version_manager.apply_bumps(&strategy, false).expect("Failed to apply bumps");

        // Only pkg-a and pkg-b should be updated
        assert_eq!(changes.len(), 2);

        // Verify versions were correctly applied
        let pkg_a_version =
            workspace.get_package("pkg-a").unwrap().borrow().package.borrow().version_str();
        let pkg_b_version =
            workspace.get_package("pkg-b").unwrap().borrow().package.borrow().version_str();
        let pkg_c_version =
            workspace.get_package("pkg-c").unwrap().borrow().package.borrow().version_str();

        assert_eq!(pkg_a_version, "3.0.0");
        assert_eq!(pkg_b_version, "4.0.0");
        assert_eq!(pkg_c_version, "1.0.0", "pkg-c version should be unchanged");

        // Need to keep all borrows alive while we use pkg_a_dep
        let workspace_package = workspace.get_package("pkg-b").unwrap();
        let pkg_b_info = workspace_package.borrow();
        let pkg_b = pkg_b_info.package.borrow();

        // Find the dependency on pkg-a
        let pkg_a_dep_index = pkg_b
            .dependencies()
            .iter()
            .position(|d| d.borrow().name() == "pkg-a")
            .expect("pkg-b should have a dependency on pkg-a");

        // Get the actual dependency reference
        let pkg_a_dep = &pkg_b.dependencies()[pkg_a_dep_index];

        // Now check the version
        let dep_version_string = if let Ok(fixed_version) = pkg_a_dep.borrow().fixed_version() {
            fixed_version.to_string()
        } else {
            panic!("Failed to get fixed version for pkg-a dependency");
        };

        assert_eq!(
            dep_version_string, "3.0.0",
            "pkg-b's dependency on pkg-a should use version 3.0.0"
        );
    }

    #[test]
    fn test_version_validation() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create tracker and version manager
        let tracker = ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        let version_manager = VersionManager::new(&workspace, Some(&tracker));

        // First, validate versions (should pass with our clean test workspace)
        let validation = version_manager.validate_versions().expect("Version validation failed");

        // Should not have cycles or inconsistencies
        assert!(!validation.has_cycles);
        assert!(validation.inconsistencies.is_empty());

        // Now, manually create an inconsistency by updating pkg-b's version
        // but not updating pkg-c's dependency on pkg-b
        let pkg_b = workspace.get_package("pkg-b").unwrap();
        pkg_b.borrow().update_version("3.0.0").expect("Failed to update pkg-b version");

        // Validate again
        let validation_after =
            version_manager.validate_versions().expect("Version validation failed");

        // Should have an inconsistency now
        assert!(!validation_after.inconsistencies.is_empty());

        let inconsistency = &validation_after.inconsistencies[0];
        assert_eq!(inconsistency.package_name, "pkg-c");
        assert_eq!(inconsistency.dependency_name, "pkg-b");
        assert_eq!(inconsistency.actual_version, "3.0.0");

        // The required version might be different depending on how the test workspace sets it up,
        // but it should not be "3.0.0"
        assert_ne!(inconsistency.required_version, "3.0.0");
    }
}
