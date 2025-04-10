#[cfg(test)]
mod visualization_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use sublime_package_tools::{
        generate_ascii, generate_dot, save_dot_to_file, Dependency, DependencyGraph, DotOptions,
        Package,
    };

    // Helper to create a dependency
    fn make_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    // Helper to create a package with dependencies
    fn make_package(name: &str, version: &str, dependencies: Vec<(&str, &str)>) -> Package {
        let deps = dependencies
            .into_iter()
            .map(|(name, version)| make_dependency(name, version))
            .collect();

        Package::new(name, version, Some(deps)).unwrap()
    }

    // Modified to use static packages for testing
    fn create_test_packages() -> Vec<Package> {
        vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("pkg-c", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-d", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![("pkg-d", "^1.0.0")]),
            make_package("pkg-d", "1.0.0", vec![]),
        ]
    }

    fn create_cyclic_packages() -> Vec<Package> {
        vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![("pkg-c", "^1.0.0")]),
            make_package("pkg-c", "1.0.0", vec![("pkg-a", "^1.0.0")]), // Creates cycle
        ]
    }

    fn create_packages_with_external() -> Vec<Package> {
        vec![
            make_package("pkg-a", "1.0.0", vec![("pkg-b", "^1.0.0"), ("external", "^1.0.0")]),
            make_package("pkg-b", "1.0.0", vec![]),
        ]
    }

    #[test]
    fn test_dot_generation_basic() {
        // Create packages and convert to graph during the test
        let packages = create_test_packages();

        // We need to keep the packages variable alive during the test
        let graph = DependencyGraph::from(packages.as_slice());

        let options = DotOptions::default();

        // Generate DOT representation
        let dot_result = generate_dot(&graph, &options);
        assert!(dot_result.is_ok());

        let dot = dot_result.unwrap();

        // Check basic DOT structure
        assert!(dot.starts_with("digraph"));
        assert!(dot.contains("rankdir=LR"));

        // Check nodes
        assert!(dot.contains("\"pkg-a\""));
        assert!(dot.contains("\"pkg-b\""));
        assert!(dot.contains("\"pkg-c\""));
        assert!(dot.contains("\"pkg-d\""));

        // Check edges
        assert!(dot.contains("\"pkg-a\" -> \"pkg-b\""));
        assert!(dot.contains("\"pkg-a\" -> \"pkg-c\""));
        assert!(dot.contains("\"pkg-b\" -> \"pkg-d\""));
        assert!(dot.contains("\"pkg-c\" -> \"pkg-d\""));
    }

    #[test]
    fn test_dot_generation_with_options() {
        let packages = create_test_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        // Custom options
        let options = DotOptions {
            title: "Custom Graph Title".to_string(),
            show_external: true,
            highlight_cycles: true,
        };

        // Generate DOT representation
        let dot = generate_dot(&graph, &options).unwrap();

        // Check custom title
        assert!(dot.contains("digraph \"Custom Graph Title\""));
    }

    #[test]
    fn test_dot_generation_with_cycle() {
        let packages = create_cyclic_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        // Options with cycle highlighting
        let options = DotOptions {
            title: "Cyclic Graph".to_string(),
            show_external: true,
            highlight_cycles: true,
        };

        // Generate DOT representation
        let dot = generate_dot(&graph, &options).unwrap();

        // Check for cycle highlighting (nodes in cycles should have fillcolor=lightcoral)
        assert!(dot.contains("fillcolor=lightcoral"));
    }

    #[test]
    fn test_dot_generation_with_external_deps() {
        let packages = create_packages_with_external();
        let graph = DependencyGraph::from(packages.as_slice());

        // Default options (show external)
        let options_with_external = DotOptions { show_external: true, ..DotOptions::default() };

        // Generate DOT representation
        let dot_with_external = generate_dot(&graph, &options_with_external).unwrap();

        // Should include external dependency
        assert!(dot_with_external.contains("External Dependency"));

        // Options without external
        let options_without_external = DotOptions { show_external: false, ..DotOptions::default() };

        // Generate DOT representation
        let dot_without_external = generate_dot(&graph, &options_without_external).unwrap();

        // Should not include external dependency
        assert!(!dot_without_external.contains("External Dependency"));
    }

    #[test]
    fn test_ascii_generation_basic() {
        let packages = create_test_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        // Generate ASCII representation
        let ascii_result = generate_ascii(&graph);
        assert!(ascii_result.is_ok());

        let ascii = ascii_result.unwrap();

        // Check ASCII structure
        assert!(ascii.contains("Dependency Graph:"));

        // Check package names appear in the output
        assert!(ascii.contains("pkg-a"));
        assert!(ascii.contains("pkg-b"));
        assert!(ascii.contains("pkg-c"));
        assert!(ascii.contains("pkg-d"));

        // Check tree structure characters
        assert!(ascii.contains("└──"));
        assert!(ascii.contains("├──"));
    }

    #[test]
    fn test_ascii_generation_with_cycle() {
        let packages = create_cyclic_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Check for cycle indication
        assert!(ascii.contains("cycle detected"));
    }

    #[test]
    fn test_ascii_generation_empty_graph() {
        // Create empty graph
        let empty_packages: Vec<Package> = vec![];
        let graph = DependencyGraph::from(empty_packages.as_slice());

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Should indicate empty graph
        assert!(ascii.contains("(empty)"));
    }

    #[test]
    fn test_save_dot_to_file() {
        use std::fs;
        use tempfile::NamedTempFile;

        let packages = create_test_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        let options = DotOptions::default();

        // Generate DOT representation
        let dot = generate_dot(&graph, &options).unwrap();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Save the DOT content
        let result = save_dot_to_file(&dot, &file_path);
        assert!(result.is_ok());

        // Read back the file
        let content = fs::read_to_string(&file_path).unwrap();

        // Content should match the generated DOT
        assert_eq!(content, dot);
    }

    #[test]
    fn test_save_dot_to_file_error() {
        // Try to save to an invalid path
        let packages = create_test_packages();
        let graph = DependencyGraph::from(packages.as_slice());

        let options = DotOptions::default();
        let dot = generate_dot(&graph, &options).unwrap();

        // Use a path that should not be writable
        let invalid_path = "/path/that/should/not/exist/graph.dot";

        let result = save_dot_to_file(&dot, invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_dot_options_default() {
        let options = DotOptions::default();

        assert_eq!(options.title, "Dependency Graph");
        assert!(options.show_external);
        assert!(options.highlight_cycles);
    }
}
