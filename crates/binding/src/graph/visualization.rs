//! JavaScript bindings for graph visualization utilities.

use crate::errors::handle_pkg_result;
use crate::graph::builder::DependencyGraph;
use napi::Result as NapiResult;
use napi_derive::napi;
use ws_pkg::graph::visualization::{
    generate_ascii as ws_generate_ascii, generate_dot as ws_generate_dot,
    save_dot_to_file as ws_save_dot_to_file, DotOptions as WsDotOptions,
};

/// JavaScript binding for ws_pkg::graph::visualization::DotOptions
#[napi(object)]
pub struct DotOptions {
    /// Title of the graph
    pub title: String,
    /// Whether to include external (unresolved) dependencies
    pub show_external: bool,
    /// Whether to highlight circular dependencies
    pub highlight_cycles: bool,
}

impl From<DotOptions> for WsDotOptions {
    fn from(options: DotOptions) -> Self {
        Self {
            title: options.title,
            show_external: options.show_external,
            highlight_cycles: options.highlight_cycles,
        }
    }
}

/// Generate DOT format representation of a dependency graph
///
/// @param {DependencyGraph} graph - The dependency graph to visualize
/// @param {DotOptions} options - Options for generating the DOT output
/// @returns {string} DOT format graph representation
#[napi]
pub fn generate_dot(graph: &DependencyGraph, options: DotOptions) -> NapiResult<String> {
    let ws_options = WsDotOptions::from(options);
    handle_pkg_result(ws_generate_dot(&graph.inner, &ws_options))
}

/// Generate an ASCII representation of the dependency graph
///
/// @param {DependencyGraph} graph - The dependency graph to visualize
/// @returns {string} ASCII representation of the graph
#[napi]
pub fn generate_ascii(graph: &DependencyGraph) -> NapiResult<String> {
    handle_pkg_result(ws_generate_ascii(&graph.inner))
}

/// Save DOT output to a file
///
/// @param {string} dotContent - The DOT content to save
/// @param {string} filePath - Path to save the file
/// @returns {void}
#[napi]
pub fn save_dot_to_file(dot_content: String, file_path: String) -> NapiResult<()> {
    handle_pkg_result(ws_save_dot_to_file(&dot_content, &file_path))
}

#[cfg(test)]
mod visualization_binding_tests {
    use super::*;
    use crate::graph::builder::build_dependency_graph_from_packages;
    use crate::types::dependency::Dependency;
    use crate::types::package::Package;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_generate_dot() {
        // Create packages
        let mut pkg1 = Package::new("pkg1".to_string(), "1.0.0".to_string());
        let pkg2 = Package::new("pkg2".to_string(), "1.0.0".to_string());

        // Add dependency
        let dep = Dependency::new("pkg2".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep);

        // Build graph
        let graph = build_dependency_graph_from_packages(vec![&pkg1, &pkg2]);

        // Create options
        let options = DotOptions {
            title: "Test Graph".to_string(),
            show_external: true,
            highlight_cycles: true,
        };

        // Generate DOT output
        let dot = generate_dot(&graph, options).unwrap();

        // Verify output contains key elements
        assert!(dot.contains("digraph \"Test Graph\""));
        assert!(dot.contains("\"pkg1\""));
        assert!(dot.contains("\"pkg2\""));
        assert!(dot.contains("\"pkg1\" -> \"pkg2\""));
    }

    #[test]
    fn test_generate_ascii() {
        // Create packages
        let mut pkg1 = Package::new("pkg1".to_string(), "1.0.0".to_string());
        let pkg2 = Package::new("pkg2".to_string(), "1.0.0".to_string());

        // Add dependency
        let dep = Dependency::new("pkg2".to_string(), "^1.0.0".to_string());
        pkg1.add_dependency(&dep);

        // Build graph
        let graph = build_dependency_graph_from_packages(vec![&pkg1, &pkg2]);

        // Generate ASCII output
        let ascii = generate_ascii(&graph).unwrap();

        // Verify output
        assert!(ascii.contains("pkg1"));
        assert!(ascii.contains("pkg2"));
    }

    #[test]
    fn test_save_dot_to_file() {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.dot");
        let file_path_str = file_path.to_str().unwrap().to_string();

        // Save a simple DOT content
        let dot_content = "digraph { A -> B }".to_string();
        save_dot_to_file(dot_content.clone(), file_path_str.clone()).unwrap();

        // Check the file was created and contains the content
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, dot_content);
    }
}
