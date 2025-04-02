mod test_utils;
use sublime_monorepo_tools::{DiscoveryOptions, ValidationOptions, WorkspaceManager};
use test_utils::TestWorkspace;

#[cfg(test)]
mod workspace_analysis_tests {
    use super::*;

    #[test]
    fn test_basic_dependency_analysis() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let result = manager.discover_workspace(test_workspace.path(), &options);
        assert!(result.is_ok(), "Workspace discovery failed: {:?}", result.err());

        let workspace = result.unwrap();

        // Analyze dependencies
        let analysis_result = workspace.analyze_dependencies();
        assert!(analysis_result.is_ok(), "Dependency analysis failed: {:?}", analysis_result.err());

        let analysis = analysis_result.unwrap();

        // The monorepo we created should not have cycles
        assert!(!analysis.cycles_detected, "Unexpected cycle detected");

        // Expect one missing dependency: lodash (external)
        assert_eq!(analysis.missing_dependencies.len(), 1);
        assert!(analysis.missing_dependencies.contains(&"lodash".to_string()));

        // Should be no version conflicts
        assert!(analysis.version_conflicts.is_empty(), "Unexpected version conflicts");
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        // Create a test workspace with circular dependencies
        let test_workspace = TestWorkspace::new();
        test_workspace.create_circular_deps_workspace();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let result = manager.discover_workspace(test_workspace.path(), &options);
        assert!(result.is_ok());

        let workspace = result.unwrap();

        // Analyze dependencies
        let analysis_result = workspace.analyze_dependencies();
        assert!(analysis_result.is_ok());

        let analysis = analysis_result.unwrap();

        // Should detect the cycles we created
        assert!(analysis.cycles_detected, "Failed to detect circular dependency");
    }

    #[test]
    fn test_version_conflict_detection() {
        // Create a test workspace with version conflicts
        let test_workspace = TestWorkspace::new();
        test_workspace.create_version_conflicts_workspace();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let result = manager.discover_workspace(test_workspace.path(), &options);
        assert!(result.is_ok());

        let workspace = result.unwrap();

        // Analyze dependencies
        let analysis_result = workspace.analyze_dependencies();
        assert!(analysis_result.is_ok());

        let analysis = analysis_result.unwrap();

        // Should detect version conflicts
        assert!(!analysis.version_conflicts.is_empty(), "Failed to detect version conflicts");
        assert!(
            analysis.version_conflicts.contains_key("shared-dep"),
            "Should detect conflict for 'shared-dep'"
        );
    }

    #[test]
    fn test_workspace_validation() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // First validate with default options - should have issues
        let default_validation = workspace.validate().unwrap();
        assert!(
            default_validation.has_issues(),
            "Expected validation to have issues with default options"
        );

        // Now validate with custom options that treat unresolved deps as external
        let validation_options = ValidationOptions::new().treat_unresolved_as_external(true);
        let custom_validation = workspace.validate_with_options(&validation_options).unwrap();

        // Should not have critical issues with custom options
        assert!(
            !custom_validation.has_critical_issues(),
            "Did not expect critical issues with custom options"
        );
    }

    #[test]
    fn test_manager_workspace_analysis() {
        // Create a test workspace with a monorepo
        let test_workspace = TestWorkspace::new();
        test_workspace.create_monorepo();

        // Create workspace manager and discover workspace
        let manager = WorkspaceManager::new();
        let options = DiscoveryOptions::new();

        let workspace = manager.discover_workspace(test_workspace.path(), &options).unwrap();

        // Analyze workspace
        let analysis_result = manager.analyze_workspace(&workspace);
        assert!(analysis_result.is_ok(), "Workspace analysis failed: {:?}", analysis_result.err());

        let analysis = analysis_result.unwrap();

        // Check analysis results
        assert!(!analysis.cycle_detected);
        assert!(!analysis.missing_dependencies.is_empty()); // Should include lodash

        // With default options, we should have validation issues
        assert!(analysis.validation_issues, "Expected validation issues in analysis");
    }
}
