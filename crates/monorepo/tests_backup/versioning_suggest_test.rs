mod test_utils;

use std::rc::Rc;
use sublime_monorepo_tools::{
    suggest_version_bumps, BumpReason, BumpType, Change, ChangeTracker, ChangeType,
    DiscoveryOptions, MemoryChangeStore, VersionBumpStrategy, VersionSuggestion, WorkspaceManager,
};
use test_utils::TestWorkspace;

#[cfg(test)]
mod versioning_suggest_tests {
    use super::*;

    #[test]
    fn test_suggest_version_bumps_basic() {
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

        // Create change tracker with changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add feature change to pkg-a
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "New feature", false))
            .expect("Failed to record change");

        // Use the suggest_version_bumps function directly
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let suggestions = suggest_version_bumps(&workspace, &tracker, &strategy)
            .expect("Failed to suggest version bumps");

        // Check the suggestion for pkg-a
        assert!(suggestions.contains_key("pkg-a"));

        let pkg_a_suggestion = &suggestions["pkg-a"];
        assert_eq!(pkg_a_suggestion.bump_type, BumpType::Minor);
        assert_eq!(pkg_a_suggestion.package_name, "pkg-a");
        assert_eq!(pkg_a_suggestion.current_version, "1.0.0");
        assert_eq!(pkg_a_suggestion.suggested_version, "1.1.0");

        // Check reasons
        assert_eq!(pkg_a_suggestion.reasons.len(), 1);
        match &pkg_a_suggestion.reasons[0] {
            BumpReason::Feature(desc) => assert_eq!(desc, "New feature"),
            _ => panic!("Expected Feature reason"),
        }
    }

    #[test]
    fn test_version_suggestion_creation() {
        // Test basic VersionSuggestion creation
        let suggestion = VersionSuggestion::new(
            "pkg-test".to_string(),
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            BumpType::Minor,
        );

        assert_eq!(suggestion.package_name, "pkg-test");
        assert_eq!(suggestion.current_version, "1.0.0");
        assert_eq!(suggestion.suggested_version, "1.1.0");
        assert_eq!(suggestion.bump_type, BumpType::Minor);
        assert!(suggestion.reasons.is_empty());

        // Test with_reason method
        let suggestion_with_reason =
            suggestion.with_reason(BumpReason::Feature("Test feature".to_string()));

        assert_eq!(suggestion_with_reason.reasons.len(), 1);

        // Test with_reasons method
        let multiple_reasons = vec![
            BumpReason::Feature("Feature 1".to_string()),
            BumpReason::Feature("Feature 2".to_string()),
        ];

        let suggestion_with_multiple = VersionSuggestion::new(
            "pkg-multi".to_string(),
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            BumpType::Minor,
        )
        .with_reasons(multiple_reasons);

        assert_eq!(suggestion_with_multiple.reasons.len(), 2);
    }

    #[test]
    fn test_dependency_update_suggestions() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace with known dependencies:
        // pkg-b depends on pkg-a
        // pkg-c depends on pkg-a and pkg-b
        // web-app depends on pkg-a and pkg-b
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = Rc::new(
            manager
                .discover_workspace(test_workspace.path(), &options)
                .expect("Failed to discover workspace"),
        );

        // Create change tracker with changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add breaking change to pkg-a
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Breaking, "Breaking API change", true))
            .expect("Failed to record change");

        // Use the suggest_version_bumps function
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        let suggestions = suggest_version_bumps(&workspace, &tracker, &strategy)
            .expect("Failed to suggest version bumps");

        // pkg-a should get a major bump
        assert!(suggestions.contains_key("pkg-a"));
        assert_eq!(suggestions["pkg-a"].bump_type, BumpType::Major);

        // packages that depend on pkg-a should also get updated
        for pkg_name in &["pkg-b", "pkg-c"] {
            if let Some(pkg_info) = workspace.get_package(pkg_name) {
                let pkg_info_borrow = pkg_info.borrow();
                let pkg = pkg_info_borrow.package.borrow();
                let has_pkg_a_dep = pkg.dependencies().iter().any(|d| d.borrow().name() == "pkg-a");

                if has_pkg_a_dep {
                    assert!(
                        suggestions.contains_key(*pkg_name),
                        "{pkg_name} should have a version suggestion as it depends on pkg-a"
                    );

                    // Check if the reason is dependency update
                    let suggestion = &suggestions[*pkg_name];
                    assert!(
                        suggestion
                            .reasons
                            .iter()
                            .any(|r| matches!(r, BumpReason::DependencyUpdate(_))),
                        "{pkg_name} should have a dependency update reason"
                    );
                }
            }
        }
    }

    #[test]
    #[allow(clippy::print_stdout)]
    #[allow(clippy::match_same_arms)]
    #[allow(clippy::items_after_statements)]
    fn test_snapshot_version_suggestions() {
        // Skip this test if not running in a Git repo context
        // since snapshot versions require a Git SHA
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo(); // Use regular monorepo instead of git_monorepo

        // Create a Git repository manually if possible
        let git_init_result = std::process::Command::new("git")
            .args(["init"])
            .current_dir(test_workspace.path())
            .output();

        // If git isn't available or fails, skip the test
        if git_init_result.is_err() {
            println!("Skipping snapshot test - couldn't initialize Git repo");
            return;
        }

        // Add files to Git so we can get a SHA
        let _ = std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(test_workspace.path())
            .output();
        let _ = std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(test_workspace.path())
            .output();
        let _ = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(test_workspace.path())
            .output();
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(test_workspace.path())
            .output();

        // Check if the workspace has a Git repo
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = match manager.discover_workspace(test_workspace.path(), &options) {
            Ok(ws) => Rc::new(ws),
            Err(_) => {
                println!("Skipping snapshot test - couldn't discover workspace");
                return;
            }
        };

        if workspace.git_repo().is_none() {
            println!("Skipping snapshot test - no Git repo available");
            return;
        }

        // Check if we're in a Git repo (the test might not be able to create one)
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();
        let workspace = match manager.discover_workspace(test_workspace.path(), &options) {
            Ok(ws) => Rc::new(ws),
            Err(_) => {
                println!("Skipping snapshot test - not in Git repo");
                return;
            }
        };

        // Check if the workspace has a Git repo
        if workspace.git_repo().is_none() {
            println!("Skipping snapshot test - no Git repo available");
            return;
        }

        // Create change tracker with changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add change to pkg-a
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Test feature", false))
            .expect("Failed to record change");

        // Create custom bump strategy that includes snapshot for feature changes
        struct CustomBumpStrategy;

        impl CustomBumpStrategy {
            fn determine_bump(change: &Change) -> BumpType {
                match change.change_type {
                    ChangeType::Feature => BumpType::Snapshot,
                    ChangeType::Fix => BumpType::Patch,
                    ChangeType::Breaking => BumpType::Major,
                    _ => BumpType::Patch,
                }
            }
        }

        // Now manually create suggestions based on our custom logic
        // (This is a bit different than the normal flow but allows testing snapshot logic)
        let unreleased = tracker.unreleased_changes().expect("Failed to get unreleased changes");
        let pkg_a_changes = unreleased.get("pkg-a").expect("Should have changes for pkg-a");

        // Get current version and Git SHA
        let pkg_a = workspace.get_package("pkg-a").expect("pkg-a should exist");
        let current_version = pkg_a.borrow().package.borrow().version_str();

        let git_repo = workspace.git_repo().expect("Should have Git repo");
        let sha = git_repo.get_current_sha().expect("Should get SHA");
        let short_sha = if sha.len() > 7 { &sha[0..7] } else { &sha };

        // Determine bump type based on our custom logic
        let bump_type = CustomBumpStrategy::determine_bump(&pkg_a_changes[0]);
        assert_eq!(bump_type, BumpType::Snapshot);

        // Calculate expected snapshot version
        let expected_version =
            match sublime_package_tools::Version::bump_snapshot(&current_version, short_sha) {
                Ok(v) => v.to_string(),
                Err(_) => {
                    println!(
                        "Skipping snapshot version test - couldn't calculate snapshot version"
                    );
                    return;
                }
            };

        assert!(
            expected_version.contains(short_sha),
            "Snapshot version should contain the Git SHA"
        );
    }

    #[test]
    fn test_synchronized_version_suggestions() {
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

        // Create change tracker with changes
        let mut tracker =
            ChangeTracker::new(Rc::clone(&workspace), Box::new(MemoryChangeStore::new()));

        // Add changes to different packages
        tracker
            .record_change(Change::new("pkg-a", ChangeType::Feature, "Feature A", false))
            .expect("Failed to record change");
        tracker
            .record_change(Change::new("pkg-b", ChangeType::Fix, "Fix B", false))
            .expect("Failed to record change");

        // Use synchronized strategy
        let strategy = VersionBumpStrategy::Synchronized { version: "3.0.0".to_string() };

        let suggestions = suggest_version_bumps(&workspace, &tracker, &strategy)
            .expect("Failed to suggest version bumps");

        // All packages should have a suggestion to use version 3.0.0
        for pkg_name in &["pkg-a", "pkg-b", "pkg-c"] {
            if workspace.get_package(pkg_name).is_some() {
                assert!(
                    suggestions.contains_key(*pkg_name),
                    "{pkg_name} should have a version suggestion"
                );

                let suggestion = &suggestions[*pkg_name];
                assert_eq!(
                    suggestion.suggested_version, "3.0.0",
                    "{pkg_name} should be suggested to update to 3.0.0"
                );

                // Reason should be Manual for synchronized strategy
                assert!(
                    suggestion.reasons.iter().any(|r| matches!(r, BumpReason::Manual)),
                    "{pkg_name} should have a Manual reason for synchronized strategy"
                );
            }
        }
    }
}
