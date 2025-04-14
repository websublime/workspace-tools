mod fixtures;

#[cfg(test)]
mod versioning_suggestions_tests {
    use crate::fixtures::{
        bun_cycle_monorepo, npm_cycle_monorepo, npm_monorepo, pnpm_cycle_monorepo,
        yarn_cycle_monorepo,
    };
    use rstest::*;

    use std::{collections::HashMap, rc::Rc};
    use sublime_monorepo_tools::{
        determine_bump_type_from_change, suggest_version_bumps_with_options, BumpReason, BumpType,
        Change, ChangeTracker, ChangeType, DiscoveryOptions, MemoryChangeStore,
        VersionBumpStrategy, VersionSuggestion, Workspace, WorkspaceConfig,
    };
    use tempfile::TempDir;

    // Helper to create a workspace from fixture
    fn setup_workspace(temp_dir: &TempDir) -> Result<Rc<Workspace>, Box<dyn std::error::Error>> {
        let repo_path = temp_dir.path();

        // Create workspace configuration
        let config = WorkspaceConfig::new(repo_path.to_path_buf());

        // Open Git repo from the fixture
        let git_repo = Some(sublime_git_tools::Repo::open(repo_path.to_str().unwrap())?);

        // Create workspace with the Git repo
        let mut workspace = Workspace::new(repo_path.to_path_buf(), config, git_repo)?;

        // Discover packages
        let options = DiscoveryOptions::new()
            .include_patterns(vec!["packages/*/package.json"])
            .auto_detect_root(true);
        workspace.discover_packages_with_options(&options)?;

        Ok(Rc::new(workspace))
    }

    #[test]
    fn test_version_suggestion() {
        // Create a basic version suggestion
        let suggestion = VersionSuggestion::new(
            "@scope/package-foo".to_string(),
            "1.0.0".to_string(),
            "1.1.0".to_string(),
            BumpType::Minor,
        );

        // Verify fields
        assert_eq!(suggestion.package_name, "@scope/package-foo");
        assert_eq!(suggestion.current_version, "1.0.0");
        assert_eq!(suggestion.suggested_version, "1.1.0");
        assert_eq!(suggestion.bump_type, BumpType::Minor);
        assert_eq!(suggestion.reasons.len(), 0);
        assert!(suggestion.cycle_group.is_none());

        // Add reasons
        let suggestion = suggestion
            .with_reason(BumpReason::Feature("Add button".to_string()))
            .with_reason(BumpReason::Fix("Fix layout".to_string()));

        assert_eq!(suggestion.reasons.len(), 2);

        // Add multiple reasons
        let suggestion = suggestion.with_reasons(vec![
            BumpReason::Breaking("Change API".to_string()),
            BumpReason::Other("Update docs".to_string()),
        ]);

        assert_eq!(suggestion.reasons.len(), 4);

        // Add cycle group
        let cycle_group = vec![
            "@scope/package-foo".to_string(),
            "@scope/package-bar".to_string(),
            "@scope/package-baz".to_string(),
        ];

        let suggestion = suggestion.with_cycle_group(cycle_group.clone());
        assert!(suggestion.cycle_group.is_some());
        assert_eq!(suggestion.cycle_group.unwrap(), cycle_group);
    }

    #[test]
    fn test_determine_bump_type_from_change() {
        // Independent strategy with all options enabled
        let full_strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Breaking change should yield major bump
        let breaking_change =
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true);
        assert_eq!(
            determine_bump_type_from_change(&breaking_change, &full_strategy),
            BumpType::Major
        );

        // Feature change should yield minor bump
        let feature_change =
            Change::new("@scope/package-foo", ChangeType::Feature, "New feature", false);
        assert_eq!(
            determine_bump_type_from_change(&feature_change, &full_strategy),
            BumpType::Minor
        );

        // Fix change should yield patch bump
        let fix_change = Change::new("@scope/package-foo", ChangeType::Fix, "Bug fix", false);
        assert_eq!(determine_bump_type_from_change(&fix_change, &full_strategy), BumpType::Patch);

        // Breaking feature should still be major (breaking flag takes precedence)
        let breaking_feature =
            Change::new("@scope/package-foo", ChangeType::Feature, "Breaking feature", true);
        assert_eq!(
            determine_bump_type_from_change(&breaking_feature, &full_strategy),
            BumpType::Major
        );

        // Test with different strategies

        // Strategy with no major bumps
        let no_major = VersionBumpStrategy::Independent {
            major_if_breaking: false,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Now breaking changes won't yield major bumps
        assert_eq!(
            determine_bump_type_from_change(&breaking_change, &no_major),
            BumpType::Patch // Falls back to patch
        );

        // Test conventional commits strategy
        let conventional = VersionBumpStrategy::ConventionalCommits { from_ref: None };

        // Breaking change should be major
        assert_eq!(
            determine_bump_type_from_change(&breaking_change, &conventional),
            BumpType::Major
        );

        // Feature should be minor
        assert_eq!(
            determine_bump_type_from_change(&feature_change, &conventional),
            BumpType::Minor
        );

        // Fix should be patch
        assert_eq!(determine_bump_type_from_change(&fix_change, &conventional), BumpType::Patch);

        // Synchronized doesn't use individual change types
        let synchronized = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };

        assert_eq!(
            determine_bump_type_from_change(&breaking_change, &synchronized),
            BumpType::None
        );
    }

    #[rstest]
    fn test_suggest_bumps_independent_strategy(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create changes for different packages
        let changes = vec![
            // Breaking change should suggest major bump
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            // Feature should suggest minor bump
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
            // Fix should suggest patch bump
            Change::new("@scope/package-baz", ChangeType::Fix, "Bug fix", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define Independent strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Get suggestions
        let suggestions = suggest_version_bumps_with_options(
            &workspace, &tracker, &strategy, false, // don't harmonize cycles
        )?;

        // Note: The actual number of suggestions may be more than 3 due to dependency propagation
        // Let's just verify that our directly changed packages have the expected bumps

        // Check package-foo (major)
        if let Some(foo_suggestion) = suggestions.get("@scope/package-foo") {
            assert_eq!(foo_suggestion.bump_type, BumpType::Major);
            assert_eq!(foo_suggestion.current_version, "1.0.0");
            assert!(foo_suggestion.suggested_version.starts_with("2.0.0"));
        } else {
            panic!("No suggestion for package-foo");
        }

        // Check package-bar (minor)
        if let Some(bar_suggestion) = suggestions.get("@scope/package-bar") {
            assert_eq!(bar_suggestion.bump_type, BumpType::Minor);
            assert_eq!(bar_suggestion.current_version, "1.0.0");
            assert!(bar_suggestion.suggested_version.starts_with("1.1.0"));
        } else {
            panic!("No suggestion for package-bar");
        }

        // Check package-baz (patch)
        if let Some(baz_suggestion) = suggestions.get("@scope/package-baz") {
            assert_eq!(baz_suggestion.bump_type, BumpType::Patch);
            assert_eq!(baz_suggestion.current_version, "1.0.0");
            assert!(baz_suggestion.suggested_version.starts_with("1.0.1"));
        } else {
            panic!("No suggestion for package-baz");
        }

        Ok(())
    }

    #[rstest]
    fn test_suggest_bumps_synchronized_strategy(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create changes for different packages
        let changes = vec![
            Change::new("@scope/package-foo", ChangeType::Fix, "Breaking fix", true),
            Change::new("@scope/package-bar", ChangeType::Feature, "New feature", false),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define Synchronized strategy
        let strategy = VersionBumpStrategy::Synchronized { version: "2.0.0".to_string() };

        // Get suggestions
        let suggestions =
            suggest_version_bumps_with_options(&workspace, &tracker, &strategy, false)?;

        // Should have suggestions for all packages, not just changed ones
        assert!(suggestions.len() >= 2, "Should have suggestions for at least changed packages");

        // All packages should have the same version
        for (pkg_name, suggestion) in &suggestions {
            assert_eq!(
                suggestion.suggested_version, "2.0.0",
                "All packages should have version 2.0.0, but {} has {}",
                pkg_name, suggestion.suggested_version
            );
        }

        Ok(())
    }

    #[rstest]
    fn test_suggest_bumps_manual_strategy(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Define Manual strategy
        let mut versions = HashMap::new();
        versions.insert("@scope/package-foo".to_string(), "3.0.0".to_string());
        versions.insert("@scope/package-bar".to_string(), "2.5.0".to_string());

        let strategy = VersionBumpStrategy::Manual(versions);

        // Get suggestions
        let suggestions =
            suggest_version_bumps_with_options(&workspace, &tracker, &strategy, false)?;

        // Note: We may get more than 2 suggestions due to dependency propagation
        // Just verify that our manually specified packages have the exact versions we provided

        // Check package-foo version
        if let Some(foo_suggestion) = suggestions.get("@scope/package-foo") {
            assert_eq!(foo_suggestion.suggested_version, "3.0.0");
            assert!(foo_suggestion.reasons.iter().any(|r| matches!(r, BumpReason::Manual)));
        } else {
            panic!("No suggestion for package-foo");
        }

        // Check package-bar version
        if let Some(bar_suggestion) = suggestions.get("@scope/package-bar") {
            assert_eq!(bar_suggestion.suggested_version, "2.5.0");
            assert!(bar_suggestion.reasons.iter().any(|r| matches!(r, BumpReason::Manual)));
        } else {
            panic!("No suggestion for package-bar");
        }

        Ok(())
    }

    #[rstest]
    fn test_dependency_update_propagation(
        npm_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace
        let workspace = setup_workspace(&npm_monorepo)?;

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create a change only for package-baz (which is a dependency of package-bar)
        let changes =
            vec![Change::new("@scope/package-baz", ChangeType::Feature, "New feature", false)];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Get suggestions with dependency updates enabled
        let suggestions = suggest_version_bumps_with_options(
            &workspace, &tracker, &strategy, true, // harmonize cycles
        )?;

        // Should have suggestion for package-baz (direct change)
        assert!(
            suggestions.contains_key("@scope/package-baz"),
            "Should have suggestion for directly changed package"
        );

        // Should also have suggestion for package-bar (depends on package-baz)
        if let Some(bar_suggestion) = suggestions.get("@scope/package-bar") {
            assert!(
                bar_suggestion.reasons.iter().any(|r| matches!(r, BumpReason::DependencyUpdate(_))),
                "package-bar should have dependency update reason"
            );
        } else {
            panic!("No suggestion for package-bar which depends on package-baz");
        }

        Ok(())
    }

    #[rstest]
    #[case::npm(npm_cycle_monorepo())]
    #[case::yarn(yarn_cycle_monorepo())]
    #[case::pnpm(pnpm_cycle_monorepo())]
    #[case::bun(bun_cycle_monorepo())]
    fn test_cycle_harmonization(
        #[case] cycle_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace with cycle dependencies
        let workspace = setup_workspace(&cycle_monorepo)?;

        // Verify cycles exist
        let cycles = workspace.get_circular_dependencies();
        assert!(!cycles.is_empty(), "Test requires circular dependencies");

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create change for just one package in the cycle
        let changed_package = cycles[0][0].clone();
        let changes = vec![
            // Breaking change for the first package in the cycle
            Change::new(changed_package.clone(), ChangeType::Fix, "Breaking fix".to_string(), true),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Get suggestions WITH cycle harmonization
        let harmonized_suggestions = suggest_version_bumps_with_options(
            &workspace, &tracker, &strategy, true, // harmonize cycles
        )?;

        // Get suggestions WITHOUT cycle harmonization
        let unharmonized_suggestions = suggest_version_bumps_with_options(
            &workspace, &tracker, &strategy, false, // don't harmonize cycles
        )?;

        // The key difference should be that harmonized suggestions include cycle information
        // while unharmonized suggestions don't

        // Check for cycle information in harmonized suggestions
        let mut harmonized_with_cycle_info = 0;
        for pkg in &cycles[0] {
            if let Some(suggestion) = harmonized_suggestions.get(pkg) {
                if suggestion.cycle_group.is_some() {
                    harmonized_with_cycle_info += 1;
                }
            }
        }

        // Check for cycle information in unharmonized suggestions
        let mut _unharmonized_with_cycle_info = 0;
        for pkg in &cycles[0] {
            if let Some(suggestion) = unharmonized_suggestions.get(pkg) {
                if suggestion.cycle_group.is_some() {
                    _unharmonized_with_cycle_info += 1;
                }
            }
        }

        // With harmonization, cycle information should be provided
        assert!(
            harmonized_with_cycle_info > 0,
            "With cycle harmonization, at least some packages should have cycle information"
        );

        // The directly changed package should have a Major bump in both cases
        if let Some(harmonized) = harmonized_suggestions.get(&changed_package) {
            assert_eq!(
                harmonized.bump_type,
                BumpType::Major,
                "Changed package should have Major bump with harmonization"
            );
        } else {
            panic!("No suggestion for changed package with harmonization");
        }

        if let Some(unharmonized) = unharmonized_suggestions.get(&changed_package) {
            assert_eq!(
                unharmonized.bump_type,
                BumpType::Major,
                "Changed package should have Major bump without harmonization"
            );
        } else {
            panic!("No suggestion for changed package without harmonization");
        }

        Ok(())
    }

    #[rstest]
    #[case::npm(npm_cycle_monorepo())]
    #[case::yarn(yarn_cycle_monorepo())]
    #[case::pnpm(pnpm_cycle_monorepo())]
    #[case::bun(bun_cycle_monorepo())]
    fn test_cycle_harmonization_disabled(
        #[case] cycle_monorepo: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set up workspace with cycle dependencies
        let workspace = setup_workspace(&cycle_monorepo)?;

        // Verify cycles exist
        let cycles = workspace.get_circular_dependencies();
        assert!(!cycles.is_empty(), "Test requires circular dependencies");

        // Create change tracker
        let store = Box::new(MemoryChangeStore::new());
        let mut tracker = ChangeTracker::new(Rc::clone(&workspace), store);

        // Create change for just one package in the cycle
        let changed_package = cycles[0][0].clone();
        let changes = vec![
            // Breaking change for the first package in the cycle
            Change::new(changed_package.clone(), ChangeType::Fix, "Breaking fix".to_string(), true),
        ];

        // Record changes
        tracker.create_changeset(None, changes)?;

        // Define strategy
        let strategy = VersionBumpStrategy::Independent {
            major_if_breaking: true,
            minor_if_feature: true,
            patch_otherwise: true,
        };

        // Get suggestions WITHOUT cycle harmonization
        let suggestions = suggest_version_bumps_with_options(
            &workspace, &tracker, &strategy, false, // don't harmonize cycles
        )?;

        // Only the directly changed package should have a Major bump
        let mut major_bump_packages = Vec::new();
        for pkg in &cycles[0] {
            if let Some(suggestion) = suggestions.get(pkg) {
                if suggestion.bump_type == BumpType::Major {
                    major_bump_packages.push(pkg.clone());
                }
            }
        }

        assert_eq!(
                    major_bump_packages.len(),
                    1,
                    "Without cycle harmonization, only one package should get Major bump, got: {major_bump_packages:?}",

                );

        assert_eq!(
            major_bump_packages[0], changed_package,
            "Without cycle harmonization, only the directly changed package should get Major bump"
        );

        // No suggestions should have cycle_group information
        for (pkg_name, suggestion) in &suggestions {
            assert!(
                        suggestion.cycle_group.is_none(),
                        "Package {pkg_name} should not have cycle group info when harmonization is disabled",

                    );

            // Other packages in the cycle might get updated via normal dependency propagation
            // but not with BumpReason indicating cycle relationship
            if pkg_name != &changed_package {
                for reason in &suggestion.reasons {
                    if let BumpReason::Other(desc) = reason {
                        assert!(
                            !desc.contains("Part of dependency cycle"),
                            "Reason should not mention cycle: {desc}",
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
