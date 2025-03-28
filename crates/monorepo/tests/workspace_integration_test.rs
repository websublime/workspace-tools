mod test_utils;

use sublime_monorepo_tools::{DiscoveryOptions, WorkspaceManager};
use test_utils::TestWorkspace;

#[cfg(test)]
mod workspace_integration_tests {
    use super::*;

    #[test]
    fn test_workspace_end_to_end() {
        // Create a test workspace
        let test_workspace = TestWorkspace::new();

        // Create a more complex workspace
        test_workspace.create_package_json("", "root-workspace", "1.0.0", &[]);

        // Create a few packages with dependencies between them
        test_workspace.create_package_json("packages/a", "pkg-a", "1.0.0", &[]);
        test_workspace.create_package_json("packages/b", "pkg-b", "1.0.0", &[("pkg-a", "^1.0.0")]);
        test_workspace.create_package_json("packages/c", "pkg-c", "1.0.0", &[("pkg-b", "^1.0.0")]);
        test_workspace.create_package_json(
            "packages/d",
            "pkg-d",
            "1.0.0",
            &[("pkg-c", "^1.0.0"), ("pkg-a", "^1.0.0")],
        );

        // Discover and analyze the workspace
        let manager = WorkspaceManager::new();
        let workspace = manager
            .discover_workspace(test_workspace.path(), &DiscoveryOptions::new())
            .expect("Failed to discover workspace");

        // 1. Verify all packages were discovered
        assert!(workspace.get_package("pkg-a").is_some());
        assert!(workspace.get_package("pkg-b").is_some());
        assert!(workspace.get_package("pkg-c").is_some());
        assert!(workspace.get_package("pkg-d").is_some());

        // 2. Verify dependency relationships
        let deps_of_d = workspace.dependencies_of("pkg-d");
        assert_eq!(deps_of_d.len(), 2);
        assert!(deps_of_d.iter().any(|p| p.borrow().package.borrow().name() == "pkg-c"));
        assert!(deps_of_d.iter().any(|p| p.borrow().package.borrow().name() == "pkg-a"));

        // 3. Verify dependency analysis
        let analysis = workspace.analyze_dependencies().expect("Analysis failed");
        assert!(!analysis.cycles_detected);
        assert!(analysis.missing_dependencies.is_empty());
        assert!(analysis.version_conflicts.is_empty());

        // 4. Verify topological sorting
        let sorted = workspace.sorted_packages();
        let names: Vec<String> =
            sorted.iter().map(|p| p.borrow().package.borrow().name().to_string()).collect();

        let idx_a = names.iter().position(|n| n == "pkg-a").unwrap();
        let idx_b = names.iter().position(|n| n == "pkg-b").unwrap();
        let idx_c = names.iter().position(|n| n == "pkg-c").unwrap();
        let idx_d = names.iter().position(|n| n == "pkg-d").unwrap();

        assert!(idx_a < idx_b);
        assert!(idx_b < idx_c);
        assert!(idx_c < idx_d);
        assert!(idx_a < idx_d);

        // 5. Check the workspace analysis via the manager
        let workspace_analysis =
            manager.analyze_workspace(&workspace).expect("Workspace analysis failed");
        assert!(!workspace_analysis.cycle_detected);
        assert!(workspace_analysis.missing_dependencies.is_empty());
        assert!(workspace_analysis.version_conflicts.is_empty());

        // The validation should pass without critical issues
        assert!(!workspace_analysis.validation_issues);
    }
}
