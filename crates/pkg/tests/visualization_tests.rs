#[cfg(test)]
mod visualization_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use ws_pkg::graph::build_dependency_graph_from_packages;
    use ws_pkg::graph::visualization::{generate_dot, save_dot_to_file, DotOptions};
    use ws_pkg::types::dependency::Dependency;
    use ws_pkg::types::package::Package;
    use ws_pkg::visualization::generate_ascii;

    // Helper function to create test dependencies
    fn create_test_dependency(name: &str, version: &str) -> Rc<RefCell<Dependency>> {
        Rc::new(RefCell::new(Dependency::new(name, version).unwrap()))
    }

    // Helper function to create test packages
    fn create_test_package(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Package {
        Package::new(name, version, dependencies).unwrap()
    }

    // Tests for DotOptions
    #[test]
    fn test_dot_options() {
        // Test default options
        let options = DotOptions::default();
        assert_eq!(options.title, "Dependency Graph");
        assert!(options.show_external);
        assert!(options.highlight_cycles);

        // Test custom options
        let custom_options = DotOptions {
            title: "Custom Title".to_string(),
            show_external: false,
            highlight_cycles: false,
        };

        assert_eq!(custom_options.title, "Custom Title");
        assert!(!custom_options.show_external);
        assert!(!custom_options.highlight_cycles);
    }

    // Test basic graph generation
    #[test]
    fn test_generate_dot_basic() {
        // Create packages with dependencies
        let deps1 = vec![create_test_dependency("pkg2", "^1.0.0")];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));

        let deps2 = vec![create_test_dependency("pkg3", "^1.0.0")];
        let pkg2 = create_test_package("pkg2", "1.0.0", Some(deps2));

        let pkg3 = create_test_package("pkg3", "1.0.0", None);

        let packages = vec![pkg1, pkg2, pkg3];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate DOT output
        let options = DotOptions::default();
        let dot = generate_dot(&graph, &options).unwrap();

        // Check that the DOT output has the expected structure
        assert!(dot.contains("digraph \"Dependency Graph\""));
        assert!(dot.contains("rankdir=LR;"));

        // Check that all packages are included
        assert!(dot.contains("\"pkg1\""));
        assert!(dot.contains("\"pkg2\""));
        assert!(dot.contains("\"pkg3\""));

        // Check that all dependencies are included
        assert!(dot.contains("\"pkg1\" -> \"pkg2\";"));
        assert!(dot.contains("\"pkg2\" -> \"pkg3\";"));

        // Check styling
        assert!(dot.contains("[shape=box, style=filled, fillcolor=lightblue]"));
    }

    // Test cycle detection and highlighting
    #[test]
    fn test_generate_dot_with_cycles() {
        // Create packages with circular dependencies
        let deps1 = vec![create_test_dependency("pkg2", "^1.0.0")];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));

        let deps2 = vec![create_test_dependency("pkg1", "^1.0.0")]; // Creates a cycle
        let pkg2 = create_test_package("pkg2", "1.0.0", Some(deps2));

        let packages = vec![pkg1, pkg2];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Test with cycle highlighting enabled
        let options = DotOptions { highlight_cycles: true, ..DotOptions::default() };

        let dot = generate_dot(&graph, &options).unwrap();

        // Both pkg1 and pkg2 should have cycle styling
        assert!(dot.contains("fillcolor=lightcoral"));

        // Check that nodes in cycle have correct styling
        assert!(dot.contains("\"pkg1\"") && dot.contains("fillcolor=lightcoral"));
        assert!(dot.contains("\"pkg2\"") && dot.contains("fillcolor=lightcoral"));

        // Test with cycle highlighting disabled
        let options = DotOptions { highlight_cycles: false, ..DotOptions::default() };

        let dot = generate_dot(&graph, &options).unwrap();

        // Both should have normal styling
        assert!(dot.contains("fillcolor=lightblue"));
        assert!(!dot.contains("fillcolor=lightcoral"));
    }

    // Test external dependency handling
    #[test]
    fn test_generate_dot_with_external_deps() {
        // Create packages with external dependency
        let deps1 = vec![
            create_test_dependency("pkg2", "^1.0.0"),
            create_test_dependency("external", "^1.0.0"), // External dep
        ];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));
        let pkg2 = create_test_package("pkg2", "1.0.0", None);

        let packages = vec![pkg1, pkg2];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Test with external dependencies shown
        let options = DotOptions { show_external: true, ..DotOptions::default() };

        let dot = generate_dot(&graph, &options).unwrap();

        // Should include external dependencies
        assert!(dot.contains("External Dependency"));
        assert!(dot.contains("fillcolor=lightgrey"));

        // Test with external dependencies hidden
        let options = DotOptions { show_external: false, ..DotOptions::default() };

        let dot = generate_dot(&graph, &options).unwrap();

        // Should not include external dependencies
        assert!(!dot.contains("External Dependency"));
    }

    // Test saving DOT output to a file
    #[test]
    fn test_save_dot_to_file() {
        // Create a simple graph
        let pkg1 = create_test_package("pkg1", "1.0.0", None);
        let packages = vec![pkg1];

        let graph = build_dependency_graph_from_packages(&packages);
        let options = DotOptions::default();
        let dot = generate_dot(&graph, &options).unwrap();

        // Create a temporary directory and file
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_graph.dot");
        let file_path_str = file_path.to_str().unwrap();

        // Save the DOT content to the file
        save_dot_to_file(&dot, file_path_str).unwrap();

        // Verify the file exists
        assert!(file_path.exists());

        // Read the file and verify it contains the expected content
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, dot);
        assert!(content.contains("digraph \"Dependency Graph\""));
    }

    // Test custom title
    #[test]
    fn test_custom_title() {
        // Create a simple graph
        let pkg1 = create_test_package("pkg1", "1.0.0", None);
        let packages = vec![pkg1];

        let graph = build_dependency_graph_from_packages(&packages);
        let options = DotOptions { title: "My Custom Graph".to_string(), ..DotOptions::default() };

        let dot = generate_dot(&graph, &options).unwrap();

        // Check that the title was used
        assert!(dot.contains("digraph \"My Custom Graph\""));
    }

    // Test empty graph
    #[test]
    fn test_empty_graph() {
        // Create an empty graph
        let packages: Vec<Package> = Vec::new();
        let graph = build_dependency_graph_from_packages(&packages);

        let options = DotOptions::default();
        let dot = generate_dot(&graph, &options).unwrap();

        // Should still generate valid DOT output
        assert!(dot.contains("digraph \"Dependency Graph\""));
        // But shouldn't have any nodes or edges
        assert!(!dot.contains("->"));
    }

    // Test ASCII visualization
    #[test]
    fn test_generate_ascii() {
        // Create packages with dependencies
        let deps1 = vec![create_test_dependency("pkg2", "^1.0.0")];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));

        let deps2 = vec![create_test_dependency("pkg3", "^1.0.0")];
        let pkg2 = create_test_package("pkg2", "1.0.0", Some(deps2));

        let pkg3 = create_test_package("pkg3", "1.0.0", None);

        let packages = vec![pkg1, pkg2, pkg3];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Check the output structure
        assert!(ascii.contains("Dependency Graph:"));
        assert!(ascii.contains("pkg1"));
        assert!(ascii.contains("└── pkg2"));
        assert!(ascii.contains("    └── pkg3"));
    }

    // Test ASCII visualization with cycles
    #[test]
    #[allow(clippy::print_stdout, clippy::uninlined_format_args)]
    fn test_generate_ascii_with_cycles() {
        // Create packages with circular dependencies
        let deps1 = vec![create_test_dependency("pkg2", "^1.0.0")];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));

        let deps2 = vec![create_test_dependency("pkg1", "^1.0.0")]; // Creates a cycle
        let pkg2 = create_test_package("pkg2", "1.0.0", Some(deps2));

        let packages = vec![pkg1, pkg2];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Print the output for debugging
        println!("ASCII output:\n{}", ascii);

        // Check that the graph is detected and some output is generated
        assert!(ascii.contains("no root nodes found, graph may contain cycles"));

        // Check that pkg1 and pkg2 are included
        assert!(ascii.contains("pkg1"));
        assert!(ascii.contains("pkg2"));

        // Check that cycles are detected
        assert!(ascii.contains("cycle detected"));
    }

    // Test ASCII visualization with external dependencies
    #[test]
    fn test_generate_ascii_with_external_deps() {
        // Create packages with external dependency
        let deps1 = vec![
            create_test_dependency("pkg2", "^1.0.0"),
            create_test_dependency("external", "^1.0.0"), // External dep
        ];
        let pkg1 = create_test_package("pkg1", "1.0.0", Some(deps1));
        let pkg2 = create_test_package("pkg2", "1.0.0", None);

        let packages = vec![pkg1, pkg2];

        // Build graph from packages
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Check that external dependencies are included
        assert!(ascii.contains("External Dependency"));
    }

    // Test ASCII visualization with empty graph
    #[test]
    fn test_generate_ascii_empty_graph() {
        // Create an empty graph
        let packages: Vec<Package> = Vec::new();
        let graph = build_dependency_graph_from_packages(&packages);

        // Generate ASCII representation
        let ascii = generate_ascii(&graph).unwrap();

        // Check that it shows as empty
        assert!(ascii.contains("(empty)"));
    }
}
