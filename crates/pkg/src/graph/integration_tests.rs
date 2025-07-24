//! # Integration Tests for Graph to HashTree Conversion
//!
//! This module tests the integration between the traditional Graph and the new
//! DependencyHashTree, ensuring seamless conversion and functionality.

#[cfg(test)]
mod integration_tests {
    use crate::{Graph, Registry, Package};

    #[test]
    fn test_graph_to_hash_tree_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("app", "1.0.0", Some(vec![("utils", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("utils", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Verify conversion
        assert_eq!(hash_tree.packages.len(), 2);
        assert!(hash_tree.packages.contains_key("app"));
        assert!(hash_tree.packages.contains_key("utils"));

        // Verify dependency relationships
        let app_deps = &hash_tree.dependency_graph["app"];
        assert_eq!(app_deps.len(), 1);
        assert_eq!(app_deps[0], "utils");

        // Verify dependent relationships
        let utils_dependents = &hash_tree.dependent_graph["utils"];
        assert_eq!(utils_dependents.len(), 1);
        assert_eq!(utils_dependents[0], "app");
    }

    #[test]
    fn test_hash_tree_querying_after_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("frontend", "1.0.0", Some(vec![("backend", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("backend", "1.0.0", Some(vec![("database", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("database", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Test querying capabilities
        let dependents = hash_tree.find_dependents("database");
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].name, "backend");

        // Test dependency path finding
        let path = hash_tree.find_dependency_path("frontend", "database");
        assert_eq!(path, Some(vec!["frontend".to_string(), "backend".to_string(), "database".to_string()]));

        // Test affected packages
        let affected = hash_tree.affected_by_change(&["database".to_string()]);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&"backend".to_string()));
        assert!(affected.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection_integration() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("pkg-b", "1.0.0", Some(vec![("pkg-a", "^1.0.0")]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Both traditional and hash tree should detect the cycle
        assert!(graph.has_cycles());
        
        let hash_tree_cycles = hash_tree.detect_circular_deps();
        assert!(!hash_tree_cycles.is_empty());
        
        let cycle = &hash_tree_cycles[0];
        assert!(cycle.path.contains(&"pkg-a".to_string()));
        assert!(cycle.path.contains(&"pkg-b".to_string()));
    }

    #[test]
    fn test_complex_dependency_graph_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("web-app", "1.0.0", Some(vec![("shared-ui", "^1.0.0"), ("api-client", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("mobile-app", "1.0.0", Some(vec![("shared-ui", "^1.0.0"), ("api-client", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("shared-ui", "1.0.0", Some(vec![("design-tokens", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("api-client", "1.0.0", Some(vec![("shared-utils", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("design-tokens", "1.0.0", Some(vec![]), &mut registry).unwrap(),
            Package::new_with_registry("shared-utils", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Verify all packages are converted
        assert_eq!(hash_tree.packages.len(), 6);

        // Test complex dependency analysis
        let design_token_dependents = hash_tree.find_dependents("design-tokens");
        assert_eq!(design_token_dependents.len(), 1);
        assert_eq!(design_token_dependents[0].name, "shared-ui");

        // Test transitive affected packages
        let affected_by_design_tokens = hash_tree.affected_by_change(&["design-tokens".to_string()]);
        assert_eq!(affected_by_design_tokens.len(), 3); // shared-ui, web-app, mobile-app
        assert!(affected_by_design_tokens.contains(&"shared-ui".to_string()));
        assert!(affected_by_design_tokens.contains(&"web-app".to_string()));
        assert!(affected_by_design_tokens.contains(&"mobile-app".to_string()));

        // Test multiple root changes
        let affected_by_utils = hash_tree.affected_by_change(&["shared-utils".to_string()]);
        assert_eq!(affected_by_utils.len(), 3); // api-client, web-app, mobile-app
        assert!(affected_by_utils.contains(&"api-client".to_string()));
        assert!(affected_by_utils.contains(&"web-app".to_string()));
        assert!(affected_by_utils.contains(&"mobile-app".to_string()));
    }

    #[test]
    fn test_visualization_integration() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("app", "1.0.0", Some(vec![("utils", "^1.0.0")]), &mut registry).unwrap(),
            Package::new_with_registry("utils", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Test ASCII visualization
        let ascii = hash_tree.render_ascii_tree();
        assert!(ascii.contains("Dependency Tree:"));
        assert!(ascii.contains("app"));
        assert!(ascii.contains("utils"));

        // Test DOT visualization
        let dot = hash_tree.render_dot_graph();
        assert!(dot.starts_with("digraph DependencyGraph"));
        assert!(dot.contains("app"));
        assert!(dot.contains("utils"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_empty_graph_conversion() {
        let packages: Vec<Package> = vec![];
        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Empty graph should result in empty hash tree
        assert!(hash_tree.packages.is_empty());
        assert!(hash_tree.dependency_graph.is_empty());
        assert!(hash_tree.dependent_graph.is_empty());
    }

    #[test]
    fn test_single_package_conversion() {
        let mut registry = Registry::new();
        let packages = vec![
            Package::new_with_registry("standalone", "1.0.0", Some(vec![]), &mut registry).unwrap(),
        ];

        let graph = Graph::from(packages.as_slice());
        let hash_tree = graph.to_hash_tree();

        // Single package should be properly converted
        assert_eq!(hash_tree.packages.len(), 1);
        assert!(hash_tree.packages.contains_key("standalone"));
        
        let package = &hash_tree.packages["standalone"];
        assert_eq!(package.name, "standalone");
        // Note: version is "unknown" because Node trait doesn't expose version info
        assert_eq!(package.version, "unknown");
        assert!(package.depends_on.is_empty());
        assert!(package.dependency_of.is_empty());
    }
}