mod fixtures;

#[cfg(test)]
mod dependency_analysis_tests {
    use crate::fixtures::{
        bun_cycle_monorepo, npm_cycle_monorepo, npm_monorepo, pnpm_cycle_monorepo,
        yarn_cycle_monorepo,
    };
    use rstest::*;
    use std::collections::HashSet;
    use sublime_monorepo_tools::{
        DiscoveryOptions, ValidationOptions, Workspace, WorkspaceConfig, WorkspaceManager,
    };
    use tempfile::TempDir;

    // Helper to setup a workspace
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

    #[allow(clippy::print_stdout)]
    #[rstest]
    fn test_analyze_dependencies(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Analyze dependencies
        let analysis = workspace.analyze_dependencies()?;

        // The fixtures DO have cycles (package-bar ↔ package-baz)
        assert!(analysis.cycles_detected, "Cycles should be detected in the fixture");
        assert!(!analysis.cycles.is_empty(), "Cycle list should not be empty");

        // Verify the cycle includes package-bar and package-baz
        let found_bar_baz_cycle = analysis.cycles.iter().any(|cycle| {
            cycle.contains(&"@scope/package-bar".to_string())
                && cycle.contains(&"@scope/package-baz".to_string())
        });

        assert!(found_bar_baz_cycle, "Should find cycle between package-bar and package-baz");

        // Check for external dependencies
        assert!(!analysis.external_dependencies.is_empty());

        // There should be external dependencies in package-major and package-tom
        let ext_deps = analysis.external_dependencies;

        // These should be in the list
        let expected_ext_deps = [
            "lit",
            "rollup-plugin-postcss-lit",
            "@websublime/pulseio-core",
            "@websublime/pulseio-style",
            "postcss",
            "postcss-cli",
            "open-props",
            "vite",
            "typescript",
            // ... possibly others
        ];

        for dep in expected_ext_deps {
            assert!(
                ext_deps.contains(&dep.to_string()),
                "Expected external dependency {dep} not found"
            );
        }

        // Check for version conflicts
        if !analysis.version_conflicts.is_empty() {
            println!("Version conflicts: {:?}", analysis.version_conflicts);
            // We don't necessarily expect conflicts, but we should be able to get them
        }

        Ok(())
    }

    #[rstest]
    #[case::npm(npm_cycle_monorepo())]
    #[case::yarn(yarn_cycle_monorepo())]
    #[case::pnpm(pnpm_cycle_monorepo())]
    #[case::bun(bun_cycle_monorepo())]
    fn test_circular_dependency_detection(
        #[case] temp_dir: TempDir,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Setup workspace with cycles
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new()
            .include_patterns(vec!["packages/*/package.json"])
            .detect_package_manager(true);

        let workspace = manager.discover_workspace(temp_dir.path(), &options)?;

        // Get circular dependencies
        let cycles = workspace.get_circular_dependencies();

        // Should have found at least one cycle
        assert!(!cycles.is_empty(), "No cycles found in cycle monorepo fixture");

        // The cycle should involve package-foo, package-bar, and package-baz
        let expected_packages = HashSet::from([
            "@scope/package-foo".to_string(),
            "@scope/package-bar".to_string(),
            "@scope/package-baz".to_string(),
        ]);

        // Check that all packages are in the cycle
        let cycle_packages: HashSet<String> = cycles[0].iter().cloned().collect();
        assert_eq!(cycle_packages, expected_packages);

        // Test is_in_cycle
        for package in ["@scope/package-foo", "@scope/package-bar", "@scope/package-baz"] {
            assert!(workspace.is_in_cycle(package), "Package {package} should be in a cycle");
        }

        // Test get_cycle_for_package
        let foo_cycle = workspace.get_cycle_for_package("@scope/package-foo");
        assert!(foo_cycle.is_some());
        assert_eq!(foo_cycle.unwrap().len(), 3);

        // Test get_cycle_membership
        let membership = workspace.get_cycle_membership();
        assert_eq!(membership.len(), 3);
        assert_eq!(membership["@scope/package-foo"], membership["@scope/package-bar"]);
        assert_eq!(membership["@scope/package-bar"], membership["@scope/package-baz"]);

        // Test get_sorted_packages_with_circulars
        let sorted = workspace.get_sorted_packages_with_circulars();

        // There should be no packages in the sorted list
        assert!(sorted.sorted.is_empty(), "Expected no packages in sorted list due to cycle");

        // There should be one cycle group with 3 packages
        assert_eq!(sorted.circular.len(), 1, "Expected one cycle group");
        assert_eq!(sorted.circular[0].len(), 3, "Expected 3 packages in cycle group");

        Ok(())
    }

    #[allow(clippy::print_stdout)]
    #[rstest]
    fn test_workspace_validation(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Validate the workspace with default options
        let validation = workspace.validate()?;

        // Check if there are issues
        if validation.has_issues() {
            assert!(!validation.issues().is_empty());

            println!("Validation issues found: {}", validation.issues().len());
            for issue in validation.issues() {
                println!("- {issue:?}");
            }
        }

        // Test with custom validation options
        let custom_options = ValidationOptions::new()
            .treat_unresolved_as_external(true)
            .with_internal_dependencies(vec!["@scope/package-foo", "@scope/package-bar"]);

        let custom_validation = workspace.validate_with_options(&custom_options)?;

        // With treat_unresolved_as_external=true, there should be fewer issues
        if validation.has_issues() && custom_validation.has_issues() {
            assert!(
                custom_validation.issues().len() <= validation.issues().len(),
                "Custom validation should have fewer or equal issues"
            );
        }

        Ok(())
    }

    #[allow(clippy::print_stdout)]
    #[rstest]
    fn test_package_operations(npm_monorepo: TempDir) -> Result<(), Box<dyn std::error::Error>> {
        let workspace = setup_workspace(&npm_monorepo)?;

        // Test sorted_packages
        let all_packages = workspace.sorted_packages();
        assert!(!all_packages.is_empty());

        // Test get_package
        let foo = workspace.get_package("@scope/package-foo");
        assert!(foo.is_some());

        // Test dependencies_of
        let foo_deps = workspace.dependencies_of("@scope/package-foo");
        assert!(!foo_deps.is_empty());
        assert_eq!(foo_deps.len(), 1); // package-foo depends on package-bar

        let foo_dep_names: Vec<String> =
            foo_deps.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect();
        assert!(foo_dep_names.contains(&"@scope/package-bar".to_string()));

        // Test dependents_of
        let bar_dependents = workspace.dependents_of("@scope/package-bar");
        assert!(!bar_dependents.is_empty());

        let bar_dependent_names: Vec<String> =
            bar_dependents.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect();

        // Both package-foo and package-tom depend on package-bar
        assert!(bar_dependent_names.contains(&"@scope/package-foo".to_string()));
        assert!(bar_dependent_names.contains(&"@scope/package-tom".to_string()));

        // Test affected_packages - EXPLICITLY DISABLE CYCLE CHECK
        let affected = workspace.affected_packages(&["@scope/package-bar"], Some(false));

        // With cycle checking disabled, we should get results
        assert!(!affected.is_empty(), "Should get affected packages with cycle check disabled");

        // Now check if the expected packages are affected
        let affected_names: Vec<String> =
            affected.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect();

        println!("Affected packages: {affected_names:?}");

        // package-bar should be included in the affected packages
        assert!(
            affected_names.contains(&"@scope/package-bar".to_string()),
            "package-bar should be in affected packages"
        );

        // package-foo depends on package-bar, so it should be affected
        assert!(
            affected_names.contains(&"@scope/package-foo".to_string()),
            "package-foo should be affected by changes to package-bar"
        );

        // package-tom depends on package-bar, so it should be affected
        assert!(
            affected_names.contains(&"@scope/package-tom".to_string()),
            "package-tom should be affected by changes to package-bar"
        );

        Ok(())
    }
}
