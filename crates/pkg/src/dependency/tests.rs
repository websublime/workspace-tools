#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod dependency_tests {
    use crate::dependency::{
        DependencyAnalyzer, DependencyEdge, DependencyGraph, DependencyNode, DependencyType,
    };
    use crate::version::Version;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn test_dependency_graph_creation() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.package_index.len(), 0);
    }

    #[test]
    fn test_dependency_node_creation() {
        let version = Version::from_str("1.0.0").unwrap();
        let node = DependencyNode::new(
            "test-package".to_string(),
            version.into(),
            PathBuf::from("/path/to/package"),
        );

        assert_eq!(node.name, "test-package");
        assert_eq!(node.path, PathBuf::from("/path/to/package"));
        assert!(node.dependencies.is_empty());
    }

    #[test]
    fn test_dependency_node_add_dependencies() {
        let version = Version::from_str("1.0.0").unwrap();
        let mut node = DependencyNode::new(
            "test-package".to_string(),
            version.into(),
            PathBuf::from("/path/to/package"),
        );

        node.add_dependency("lodash".to_string(), "^4.0.0".to_string());
        node.add_dev_dependency("jest".to_string(), "^29.0.0".to_string());

        assert_eq!(node.dependencies.get("lodash"), Some(&"^4.0.0".to_string()));
        assert_eq!(node.dev_dependencies.get("jest"), Some(&"^29.0.0".to_string()));
    }

    #[test]
    fn test_dependency_edge_creation() {
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        assert_eq!(edge.dependency_type, DependencyType::Runtime);
        assert_eq!(edge.version_requirement, "^1.0.0");
    }

    #[test]
    fn test_dependency_analyzer_creation() {
        let graph = DependencyGraph::new();
        let analyzer = DependencyAnalyzer::new(graph, 10, true, false, false);

        assert_eq!(analyzer.max_depth, 10);
        assert!(analyzer.include_dev_dependencies);
        assert!(!analyzer.include_optional_dependencies);
        assert!(!analyzer.include_peer_dependencies);
    }

    #[test]
    fn test_get_dependencies_by_type() {
        let version = Version::from_str("1.0.0").unwrap();
        let mut node = DependencyNode::new(
            "test-package".to_string(),
            version.into(),
            PathBuf::from("/path/to/package"),
        );

        node.add_dependency("lodash".to_string(), "^4.0.0".to_string());
        node.add_dev_dependency("jest".to_string(), "^29.0.0".to_string());

        let runtime_deps = node.get_dependencies(DependencyType::Runtime);
        let dev_deps = node.get_dependencies(DependencyType::Development);

        assert_eq!(runtime_deps.len(), 1);
        assert_eq!(dev_deps.len(), 1);
        assert!(runtime_deps.contains_key("lodash"));
        assert!(dev_deps.contains_key("jest"));
    }

    #[test]
    fn test_dependency_type_variants() {
        assert_eq!(format!("{:?}", DependencyType::Runtime), "Runtime");
        assert_eq!(format!("{:?}", DependencyType::Development), "Development");
        assert_eq!(format!("{:?}", DependencyType::Optional), "Optional");
        assert_eq!(format!("{:?}", DependencyType::Peer), "Peer");
    }

    #[test]
    fn test_dependency_node_optional_dependencies() {
        let version = Version::from_str("1.0.0").unwrap();
        let mut node = DependencyNode::new(
            "test-package".to_string(),
            version.into(),
            PathBuf::from("/path/to/package"),
        );

        node.add_optional_dependency("optional-dep".to_string(), "^1.0.0".to_string());
        node.add_peer_dependency("peer-dep".to_string(), "^2.0.0".to_string());

        assert_eq!(node.optional_dependencies.get("optional-dep"), Some(&"^1.0.0".to_string()));
        assert_eq!(node.peer_dependencies.get("peer-dep"), Some(&"^2.0.0".to_string()));

        let optional_deps = node.get_dependencies(DependencyType::Optional);
        let peer_deps = node.get_dependencies(DependencyType::Peer);

        assert_eq!(optional_deps.len(), 1);
        assert_eq!(peer_deps.len(), 1);
        assert!(optional_deps.contains_key("optional-dep"));
        assert!(peer_deps.contains_key("peer-dep"));
    }

    #[test]
    fn test_dependency_graph_add_node() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();
        let node = DependencyNode::new(
            "test-package".to_string(),
            version.into(),
            PathBuf::from("/path/to/package"),
        );

        let node_index = graph.add_node(node);
        assert_eq!(graph.package_index.len(), 1);
        assert!(graph.package_index.contains_key("test-package"));
        assert_eq!(graph.package_index["test-package"], node_index);
    }

    #[test]
    fn test_dependency_analyzer_settings() {
        let graph = DependencyGraph::new();
        let analyzer = DependencyAnalyzer::new(graph, 20, false, true, true);

        assert_eq!(analyzer.max_depth, 20);
        assert!(!analyzer.include_dev_dependencies);
        assert!(analyzer.include_optional_dependencies);
        assert!(analyzer.include_peer_dependencies);
    }

    #[test]
    fn test_dependency_node_serialization() {
        let version = Version::from_str("1.2.3").unwrap();
        let mut node = DependencyNode::new(
            "serialize-test".to_string(),
            version.into(),
            PathBuf::from("/test/path"),
        );

        node.add_dependency("dep1".to_string(), "^1.0.0".to_string());
        node.add_dev_dependency("dev-dep".to_string(), "^2.0.0".to_string());

        // Test JSON serialization
        let json_result = serde_json::to_string(&node);
        assert!(json_result.is_ok());

        // Test JSON deserialization
        let json_str = json_result.unwrap();
        let deserialized: Result<DependencyNode, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_node = deserialized.unwrap();
        assert_eq!(deserialized_node.name, "serialize-test");
        assert_eq!(deserialized_node.path, PathBuf::from("/test/path"));
        assert_eq!(deserialized_node.dependencies.len(), 1);
        assert_eq!(deserialized_node.dev_dependencies.len(), 1);
    }
}
