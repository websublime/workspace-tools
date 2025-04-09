//! Generate visual representations of dependency graphs in GraphViz DOT format.

use std::collections::HashSet;
use std::fmt::Write;

use petgraph::{graph::NodeIndex, Direction};

use crate::{DependencyGraph, Node, Step};

/// Options for generating DOT graph output
#[derive(Debug, Clone)]
pub struct DotOptions {
    /// Title of the graph
    pub title: String,
    /// Whether to include external (unresolved) dependencies
    pub show_external: bool,
    /// Whether to highlight circular dependencies
    pub highlight_cycles: bool,
}

impl Default for DotOptions {
    fn default() -> Self {
        Self { title: "Dependency Graph".to_string(), show_external: true, highlight_cycles: true }
    }
}

/// Node style definitions for the DOT output
#[derive(Debug, Clone, Copy)]
enum NodeStyle {
    /// Normal package node
    Normal,
    /// Node that is part of a cycle
    Cycle,
    /// External or unresolved dependency
    External,
}

impl NodeStyle {
    fn attributes(self) -> &'static str {
        match self {
            Self::Normal => "[shape=box, style=filled, fillcolor=lightblue]",
            Self::Cycle => "[shape=box, style=filled, fillcolor=lightcoral, penwidth=2]",
            Self::External => "[shape=ellipse, style=filled, fillcolor=lightgrey]",
        }
    }
}

/// Generate DOT format representation of a dependency graph
///
/// # Examples
///
/// ```
/// use ws_pkg::graph::DependencyGraph;
/// use ws_pkg::types::package::Package;
/// use ws_pkg::graph::visualization::{generate_dot, DotOptions};
///
/// // Create packages and build a graph
/// let packages = vec![];  // Add your packages here
/// let graph: DependencyGraph<'_, Package> = DependencyGraph::from(packages.as_slice());
///
/// // Generate DOT representation with default options
/// let options = DotOptions::default();
/// let dot_output = generate_dot(&graph, &options).unwrap();
/// ```
#[allow(clippy::writeln_empty_string)]
pub fn generate_dot<N: Node>(
    graph: &DependencyGraph<N>,
    options: &DotOptions,
) -> Result<String, std::fmt::Error> {
    let mut output = String::new();

    // Start the digraph
    writeln!(output, "digraph \"{}\" {{", options.title)?;
    writeln!(output, "  // Graph options")?;
    writeln!(output, "  rankdir=LR;")?;
    writeln!(output, "  node [fontname=\"Helvetica\"];")?;
    writeln!(output, "  edge [fontname=\"Helvetica\"];")?;
    writeln!(output, "  graph [fontname=\"Helvetica\"];")?;
    writeln!(output, "")?;

    // Find cycles if highlighting is enabled
    let nodes_in_cycles =
        if options.highlight_cycles { find_nodes_in_cycles(graph) } else { HashSet::new() };

    // Add nodes
    writeln!(output, "  // Nodes")?;
    for node_idx in graph.graph.node_indices() {
        let node = &graph.graph[node_idx];

        match node {
            Step::Resolved(package) => {
                let id = package.identifier().to_string();

                // Determine style based on whether the node is part of a cycle
                let style = if nodes_in_cycles.contains(&id) {
                    NodeStyle::Cycle
                } else {
                    NodeStyle::Normal
                };

                writeln!(output, "  \"{}\" {};", id, style.attributes())?;
            }
            Step::Unresolved(_dependency) => {
                if !options.show_external {
                    continue;
                }

                // Use index as part of ID to ensure uniqueness
                let id = format!("unresolved_{}", node_idx.index());

                writeln!(
                    output,
                    "  \"{}\" [label=\"External Dependency\"] {};",
                    id,
                    NodeStyle::External.attributes()
                )?;
            }
        }
    }

    // Add edges
    writeln!(output, "\n  // Edges")?;
    for edge_idx in graph.graph.edge_indices() {
        if let Some((source_idx, target_idx)) = graph.graph.edge_endpoints(edge_idx) {
            // Get source and target identifiers
            let source_id = match &graph.graph[source_idx] {
                Step::Resolved(package) => package.identifier().to_string(),
                Step::Unresolved(_) => {
                    format!("unresolved_{}", source_idx.index())
                }
            };

            let target_id = match &graph.graph[target_idx] {
                Step::Resolved(package) => package.identifier().to_string(),
                Step::Unresolved(_) => {
                    format!("unresolved_{}", target_idx.index())
                }
            };

            writeln!(output, "  \"{source_id}\" -> \"{target_id}\";")?;
        }
    }

    // Close the graph
    writeln!(output, "}}")?;

    Ok(output)
}

/// Check if the graph has any circular dependencies and return the nodes involved
fn find_nodes_in_cycles<N: Node>(graph: &DependencyGraph<N>) -> HashSet<String> {
    let mut nodes_in_cycles = HashSet::new();

    // Quick check - if there are no cycles, return empty set
    if !graph.has_cycles() {
        return nodes_in_cycles;
    }

    // Get all nodes involved in cycles from the graph's cycles field
    for cycle in graph.get_cycles() {
        for node_id in cycle {
            nodes_in_cycles.insert(node_id.to_string());
        }
    }

    nodes_in_cycles
}

/// Save DOT output to a file
///
/// This is a simple utility function to write the DOT output to a file.
///
/// # Examples
///
/// ```no_run
/// use ws_pkg::graph::DependencyGraph;
/// use ws_pkg::types::package::Package;
/// use ws_pkg::graph::visualization::{generate_dot, save_dot_to_file, DotOptions};
///
/// // Create packages and build a graph
/// let packages = vec![];  // Add your packages here
/// let graph: DependencyGraph<'_, Package> = DependencyGraph::from(packages.as_slice());
///
/// // Generate and save DOT representation
/// let options = DotOptions::default();
/// let dot_output = generate_dot(&graph, &options).unwrap();
/// save_dot_to_file(&dot_output, "dependency_graph.dot").unwrap();
/// ```
pub fn save_dot_to_file(dot_content: &str, file_path: &str) -> std::io::Result<()> {
    std::fs::write(file_path, dot_content)
}

/// Generate an ASCII representation of the dependency graph
///
/// This function creates a text-based visualization of the dependency graph
/// that can be displayed directly in the terminal.
///
/// # Examples
///
/// ```
/// use ws_pkg::graph::DependencyGraph;
/// use ws_pkg::types::package::Package;
/// use ws_pkg::graph::visualization::generate_ascii;
///
/// // Create packages and build a graph
/// let packages = vec![];  // Add your packages here
/// let graph: DependencyGraph<'_, Package> = DependencyGraph::from(packages.as_slice());
///
/// // Generate ASCII representation
/// let ascii = generate_ascii(&graph).unwrap();
/// println!("{}", ascii);
/// ```
pub fn generate_ascii<N: Node>(graph: &DependencyGraph<N>) -> Result<String, std::fmt::Error> {
    let mut output = String::new();

    writeln!(output, "Dependency Graph:")?;

    // If the graph is empty, indicate that
    if graph.graph.node_count() == 0 {
        writeln!(output, "(empty)")?;
        return Ok(output);
    }

    // Find root nodes (nodes with no incoming edges)
    let mut root_nodes = Vec::new();

    for node_idx in graph.graph.node_indices() {
        let incoming = graph.graph.neighbors_directed(node_idx, Direction::Incoming).count();
        if incoming == 0 {
            if let Step::Resolved(node) = &graph.graph[node_idx] {
                root_nodes.push((node_idx, node.identifier().to_string()));
            }
        }
    }

    // If no root nodes found (e.g., in a cycle), use all nodes as starting points
    if root_nodes.is_empty() {
        writeln!(output, "(no root nodes found, graph may contain cycles)")?;

        // Instead, use all nodes as potential starting points
        for node_idx in graph.graph.node_indices() {
            if let Step::Resolved(node) = &graph.graph[node_idx] {
                root_nodes.push((node_idx, node.identifier().to_string()));
            }
        }
    }

    // Sort roots for deterministic output
    root_nodes.sort_by(|a, b| a.1.cmp(&b.1));

    // No nodes in the graph (should never happen after our earlier check)
    if root_nodes.is_empty() {
        return Ok(output);
    }

    // Process each root node
    for (i, (root_idx, root_name)) in root_nodes.iter().enumerate() {
        if i > 0 {
            writeln!(output)?; // Add blank line between trees
        }

        writeln!(output, "{root_name}")?;

        // Process children recursively
        add_ascii_children(
            &mut output,
            graph,
            *root_idx,
            "",
            true, // Last item at this level
            &mut std::collections::HashSet::new(),
        )?;
    }

    Ok(output)
}

// Helper for the ASCII tree generation
fn add_ascii_children<N: Node>(
    output: &mut String,
    graph: &DependencyGraph<N>,
    node_idx: NodeIndex,
    prefix: &str,
    _is_last: bool,
    visited: &mut HashSet<NodeIndex>,
) -> Result<(), std::fmt::Error> {
    // Check for cycles - if this node is already being processed
    if !visited.insert(node_idx) {
        // Mark cycles in the output
        writeln!(output, "{prefix}└── ... (cycle detected)")?;
        return Ok(());
    }

    // Get all outgoing edges (dependencies)
    let mut children = Vec::new();

    for child_idx in graph.graph.neighbors_directed(node_idx, Direction::Outgoing) {
        match &graph.graph[child_idx] {
            Step::Resolved(node) => {
                children.push((child_idx, node.identifier().to_string()));
            }
            Step::Unresolved(_) => {
                // For unresolved dependencies, use a special name
                children.push((child_idx, "External Dependency".to_string()));
            }
        }
    }

    // Sort for deterministic output
    children.sort_by(|a, b| a.1.cmp(&b.1));

    // Process each child
    for (i, (child_idx, child_name)) in children.iter().enumerate() {
        let is_last_child = i == children.len() - 1;

        // Choose the right branch symbol
        let branch = if is_last_child { "└── " } else { "├── " };

        writeln!(output, "{prefix}{branch}{child_name}")?;

        // Choose prefix for the next level
        let child_prefix = if is_last_child {
            format!("{prefix}    ") // Just space for the last item
        } else {
            format!("{prefix}│   ") // Vertical line for non-last items
        };

        // Process this child's children
        add_ascii_children(
            output,
            graph,
            *child_idx,
            &child_prefix,
            is_last_child,
            visited, // Pass the same visited set to track cycles across the entire tree
        )?;
    }

    // Important: Remove the current node from visited when we're done processing its branch
    // This allows the same node to appear in different parts of the tree (not a cycle)
    visited.remove(&node_idx);

    Ok(())
}
