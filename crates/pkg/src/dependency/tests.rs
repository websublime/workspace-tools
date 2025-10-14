#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
mod dependency_tests {
    use crate::config::DependencyConfig;
    use crate::dependency::{
        DependencyAnalyzer, DependencyEdge, DependencyGraph, DependencyGraphBuilder,
        DependencyNode, DependencyType, PropagationReason,
    };
    use crate::version::Version;
    use crate::{ResolvedVersion, VersionBump};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;
    use sublime_standard_tools::filesystem::FileSystemManager;

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
        let config = DependencyConfig::default();
        let fs = FileSystemManager::new();
        let analyzer = DependencyAnalyzer::new(graph, config.clone(), fs);

        assert_eq!(analyzer.config().max_propagation_depth, config.max_propagation_depth);
        assert_eq!(analyzer.config().propagate_dev_dependencies, config.propagate_dev_dependencies);
        assert_eq!(
            analyzer.config().include_optional_dependencies,
            config.include_optional_dependencies
        );
        assert_eq!(analyzer.config().include_peer_dependencies, config.include_peer_dependencies);
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
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Add two packages
        let node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b = DependencyNode::new("pkg-b".to_string(), version.into(), PathBuf::from("/b"));

        graph.add_node(node_a);
        graph.add_node(node_b);

        // Add edge from B to A (B depends on A)
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        let result = graph.add_edge("pkg-b", "pkg-a", edge);

        assert!(result.is_ok());
    }

    #[test]
    fn test_dependency_graph_get_dependencies() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Create three packages
        let node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b =
            DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
        let node_c = DependencyNode::new("pkg-c".to_string(), version.into(), PathBuf::from("/c"));

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // Create dependency chain: C -> B -> A
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        graph.add_edge("pkg-b", "pkg-a", edge.clone()).unwrap();
        graph.add_edge("pkg-c", "pkg-b", edge).unwrap();

        // Test getting dependencies
        let c_deps = graph.get_dependencies("pkg-c");
        let b_deps = graph.get_dependencies("pkg-b");
        let a_deps = graph.get_dependencies("pkg-a");

        assert_eq!(c_deps, vec!["pkg-b"]);
        assert_eq!(b_deps, vec!["pkg-a"]);
        assert!(a_deps.is_empty());
    }

    #[test]
    fn test_dependency_graph_get_dependents() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Create three packages
        let node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b =
            DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
        let node_c = DependencyNode::new("pkg-c".to_string(), version.into(), PathBuf::from("/c"));

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // Create dependency chain: C -> B -> A
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        graph.add_edge("pkg-b", "pkg-a", edge.clone()).unwrap();
        graph.add_edge("pkg-c", "pkg-b", edge).unwrap();

        // Test getting dependents
        let a_dependents = graph.get_dependents("pkg-a");
        let b_dependents = graph.get_dependents("pkg-b");
        let c_dependents = graph.get_dependents("pkg-c");

        assert_eq!(a_dependents, vec!["pkg-b"]);
        assert_eq!(b_dependents, vec!["pkg-c"]);
        assert!(c_dependents.is_empty());
    }

    #[test]
    fn test_dependency_graph_cycle_detection() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Create packages that form a cycle
        let node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b = DependencyNode::new("pkg-b".to_string(), version.into(), PathBuf::from("/b"));

        graph.add_node(node_a);
        graph.add_node(node_b);

        // Create cycle: A -> B -> A
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        graph.add_edge("pkg-a", "pkg-b", edge.clone()).unwrap();
        graph.add_edge("pkg-b", "pkg-a", edge).unwrap();

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());

        // Should detect the cycle containing both packages
        let first_cycle = &cycles[0];
        assert!(first_cycle.contains(&"pkg-a".to_string()));
        assert!(first_cycle.contains(&"pkg-b".to_string()));
    }

    #[test]
    fn test_dependency_graph_no_cycles() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Create packages without cycles
        let node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b =
            DependencyNode::new("pkg-b".to_string(), version.clone().into(), PathBuf::from("/b"));
        let node_c = DependencyNode::new("pkg-c".to_string(), version.into(), PathBuf::from("/c"));

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);

        // Create linear dependency: C -> B -> A
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        graph.add_edge("pkg-b", "pkg-a", edge.clone()).unwrap();
        graph.add_edge("pkg-c", "pkg-b", edge).unwrap();

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[tokio::test]
    async fn test_dependency_analyzer_propagation() {
        let mut graph = DependencyGraph::new();
        let version = Version::from_str("1.0.0").unwrap();

        // Create packages
        let mut node_a =
            DependencyNode::new("pkg-a".to_string(), version.clone().into(), PathBuf::from("/a"));
        let node_b = DependencyNode::new("pkg-b".to_string(), version.into(), PathBuf::from("/b"));

        // B depends on A
        node_a.add_dependency("pkg-b".to_string(), "^1.0.0".to_string());

        graph.add_node(node_a);
        graph.add_node(node_b);

        // Add edge
        let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
        graph.add_edge("pkg-a", "pkg-b", edge).unwrap();

        let config = DependencyConfig::default();
        let fs = FileSystemManager::new();
        let analyzer = DependencyAnalyzer::new(graph, config, fs);

        // Test propagation when pkg-b is updated
        let mut changed_packages = HashMap::new();
        let new_version = Version::from_str("1.1.0").unwrap();
        changed_packages.insert(
            "pkg-b".to_string(),
            (VersionBump::Minor, ResolvedVersion::Release(new_version)),
        );

        let propagated = analyzer.analyze_propagation(&changed_packages).await.unwrap();
        assert!(!propagated.is_empty());

        let update = &propagated[0];
        assert_eq!(update.package_name, "pkg-a");
        assert_eq!(update.suggested_bump, VersionBump::Patch); // Default config uses patch
    }

    #[test]
    fn test_dependency_graph_builder_creation() {
        let fs = FileSystemManager::new();
        let config = DependencyConfig::default();
        let _builder = DependencyGraphBuilder::new(fs, config);
    }

    #[test]
    fn test_propagation_reason_variants() {
        let reason = PropagationReason::DependencyUpdate {
            dependency: "lodash".to_string(),
            old_version: "4.0.0".to_string(),
            new_version: "4.1.0".to_string(),
        };

        match reason {
            PropagationReason::DependencyUpdate { dependency, .. } => {
                assert_eq!(dependency, "lodash");
            }
            _ => panic!("Expected DependencyUpdate variant"),
        }
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

    #[test]
    fn test_dependency_config_defaults() {
        let config = DependencyConfig::default();
        assert!(config.propagate_updates);
        assert!(!config.propagate_dev_dependencies);
        assert_eq!(config.max_propagation_depth, 10);
        assert!(config.detect_circular);
        assert!(config.fail_on_circular);
        assert_eq!(config.dependency_update_bump, "patch");
        assert!(!config.include_peer_dependencies);
        assert!(!config.include_optional_dependencies);
    }

    #[tokio::test]
    async fn test_dependency_version_tracking_in_propagation() {
        let mut graph = DependencyGraph::new();
        let version_a = Version::from_str("1.0.0").unwrap();
        let version_b = Version::from_str("2.0.0").unwrap();

        // Create packages with specific dependency versions
        let mut node_a =
            DependencyNode::new("pkg-a".to_string(), version_a.into(), PathBuf::from("/a"));
        let node_b =
            DependencyNode::new("pkg-b".to_string(), version_b.clone().into(), PathBuf::from("/b"));

        // A depends on B with specific version requirement
        node_a.add_dependency("pkg-b".to_string(), "^2.0.0".to_string());

        graph.add_node(node_a);
        graph.add_node(node_b);

        // Add edge for dependency relationship
        let edge = DependencyEdge::new(DependencyType::Runtime, "^2.0.0".to_string());
        graph.add_edge("pkg-a", "pkg-b", edge).unwrap();

        let config = DependencyConfig::default();
        let fs = FileSystemManager::new();
        let analyzer = DependencyAnalyzer::new(graph, config, fs);

        // Simulate pkg-b being updated
        let mut changed_packages = HashMap::new();
        let new_version_b = Version::from_str("2.1.0").unwrap();
        changed_packages.insert(
            "pkg-b".to_string(),
            (VersionBump::Minor, ResolvedVersion::Release(new_version_b)),
        );

        let propagated = analyzer.analyze_propagation(&changed_packages).await.unwrap();

        // Should have propagated update for pkg-a
        assert!(!propagated.is_empty());
        let update = &propagated[0];
        assert_eq!(update.package_name, "pkg-a");

        // Check that the propagation reason contains actual version info (not "unknown")
        match &update.reason {
            PropagationReason::DependencyUpdate { dependency, old_version, new_version } => {
                assert_eq!(dependency, "pkg-b");
                // Should have actual version requirement, not "unknown"
                assert_eq!(old_version, "^2.0.0");
                assert_eq!(new_version, "2.1.0");
            }
            _ => panic!("Expected DependencyUpdate reason"),
        }
    }
}
