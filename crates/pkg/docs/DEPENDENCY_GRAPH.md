# Dependency Graph Builder Documentation

## Overview

The Dependency Graph Builder is a core component of `sublime_pkg_tools` that provides comprehensive dependency analysis for monorepo structures. It builds detailed dependency graphs, detects circular dependencies, and calculates dependency propagation impacts for version updates.

## What

The dependency graph system provides:

- **DependencyGraph**: Graph representation of package dependencies
- **DependencyGraphBuilder**: Builder for constructing graphs from monorepo structures  
- **DependencyAnalyzer**: Service for dependency analysis and propagation calculations
- **Circular Dependency Detection**: Automatic detection of dependency cycles
- **Propagation Analysis**: Calculation of which packages need updates when dependencies change

## How

The system uses `petgraph` for efficient graph operations and integrates with `sublime_standard_tools` for filesystem access and monorepo detection. It analyzes `package.json` files to understand dependency relationships and provides configurable propagation rules.

## Why

In monorepo environments, understanding dependency relationships is critical for:
- Safe version updates without breaking dependent packages
- Detecting circular dependencies that can cause build failures
- Automated propagation of version changes through dependency chains
- Maintaining consistency across related packages

## Core Components

### DependencyGraph

A graph representation that maintains package relationships:

```rust
use sublime_pkg_tools::dependency::{DependencyGraph, DependencyNode, DependencyEdge, DependencyType};
use sublime_pkg_tools::version::Version;
use std::path::PathBuf;

let mut graph = DependencyGraph::new();

// Add packages as nodes
let version = Version::new(1, 0, 0);
let node = DependencyNode::new("my-package".to_string(), version.into(), PathBuf::from("/path"));
graph.add_node(node);

// Add dependency relationships
let edge = DependencyEdge::new(DependencyType::Runtime, "^1.0.0".to_string());
graph.add_edge("dependent-pkg", "my-package", edge)?;

// Query the graph
let dependencies = graph.get_dependencies("dependent-pkg");
let dependents = graph.get_dependents("my-package");
let cycles = graph.detect_cycles();
```

### DependencyGraphBuilder

Constructs dependency graphs from monorepo structures:

```rust
use sublime_pkg_tools::dependency::DependencyGraphBuilder;
use sublime_pkg_tools::config::DependencyConfig;
use sublime_standard_tools::filesystem::FileSystemManager;

let fs = FileSystemManager::new();
let config = DependencyConfig::default();
let builder = DependencyGraphBuilder::new(fs, config);

// Build from monorepo descriptor
let graph = builder.build_from_monorepo(&monorepo_descriptor).await?;
```

### DependencyAnalyzer

Provides dependency analysis and propagation calculations:

```rust
use sublime_pkg_tools::dependency::DependencyAnalyzer;
use std::collections::HashMap;

let analyzer = DependencyAnalyzer::new(graph, config, filesystem);

// Validate graph for circular dependencies
analyzer.validate_graph()?;

// Analyze propagation when packages change
let mut changed_packages = HashMap::new();
changed_packages.insert("my-package".to_string(), (VersionBump::Minor, new_version));

let propagated_updates = analyzer.analyze_propagation(&changed_packages).await?;
for update in propagated_updates {
    println!("Update {}: {} -> {}", 
        update.package_name, 
        update.current_version, 
        update.next_version
    );
}
```

## Configuration

Dependency analysis behavior is controlled through `DependencyConfig`:

```rust
use sublime_pkg_tools::config::DependencyConfig;

let config = DependencyConfig {
    propagate_updates: true,              // Enable dependency update propagation
    propagate_dev_dependencies: false,    // Include dev dependencies in propagation
    max_propagation_depth: 10,            // Maximum propagation depth (0 = unlimited)
    detect_circular: true,                // Detect circular dependencies
    fail_on_circular: true,               // Fail validation on circular dependencies
    dependency_update_bump: "patch".to_string(), // Default bump type for dependency updates
    include_peer_dependencies: false,     // Include peer dependencies in analysis
    include_optional_dependencies: false, // Include optional dependencies in analysis
};
```

### Configuration Options

| Option | Description | Default | Values |
|--------|-------------|---------|---------|
| `propagate_updates` | Enable dependency update propagation | `true` | `true`/`false` |
| `propagate_dev_dependencies` | Include dev dependencies | `false` | `true`/`false` |  
| `max_propagation_depth` | Maximum propagation depth | `10` | `0` (unlimited) or positive integer |
| `detect_circular` | Enable circular dependency detection | `true` | `true`/`false` |
| `fail_on_circular` | Fail validation on circular deps | `true` | `true`/`false` |
| `dependency_update_bump` | Default bump for dependency updates | `"patch"` | `"major"`, `"minor"`, `"patch"` |
| `include_peer_dependencies` | Include peer deps in analysis | `false` | `true`/`false` |
| `include_optional_dependencies` | Include optional deps | `false` | `true`/`false` |

## Dependency Types

The system recognizes four types of dependencies:

```rust
use sublime_pkg_tools::dependency::DependencyType;

match dependency_type {
    DependencyType::Runtime => {
        // Regular dependencies from "dependencies" field
    }
    DependencyType::Development => {
        // Dev dependencies from "devDependencies" field  
    }
    DependencyType::Optional => {
        // Optional dependencies from "optionalDependencies" field
    }
    DependencyType::Peer => {
        // Peer dependencies from "peerDependencies" field
    }
}
```

## Propagation Analysis

When packages are updated, the analyzer calculates which dependent packages need version updates:

### Propagation Reasons

```rust
use sublime_pkg_tools::dependency::PropagationReason;

match update.reason {
    PropagationReason::DirectChanges { commits } => {
        // Package has direct code changes
    }
    PropagationReason::DependencyUpdate { dependency, old_version, new_version } => {
        // Runtime dependency was updated
    }
    PropagationReason::DevDependencyUpdate { dependency, old_version, new_version } => {
        // Development dependency was updated
    }
    PropagationReason::OptionalDependencyUpdate { dependency, old_version, new_version } => {
        // Optional dependency was updated  
    }
    PropagationReason::PeerDependencyUpdate { dependency, old_version, new_version } => {
        // Peer dependency was updated
    }
}
```

### Version Bump Calculation

The analyzer calculates appropriate version bumps based on:

1. **Configuration**: Default bump type from `dependency_update_bump`
2. **Dependency Type**: Different treatment for runtime vs dev dependencies
3. **Change Impact**: Considers the original change that triggered propagation

## Circular Dependency Detection

The system uses Tarjan's strongly connected components algorithm to detect cycles:

```rust
let cycles = graph.detect_cycles();

if !cycles.is_empty() {
    for cycle in cycles {
        println!("Circular dependency: {}", cycle.join(" -> "));
    }
}
```

### Handling Circular Dependencies

When circular dependencies are detected:

1. **Detection**: All cycles are identified and reported
2. **Validation**: Graph validation fails if `fail_on_circular` is true
3. **Resolution**: Manual intervention required to break cycles

Common cycle resolution strategies:
- Extract shared functionality into a separate package
- Use dependency injection patterns
- Restructure package boundaries

## Integration with Standard Tools

The dependency graph builder integrates seamlessly with existing tools:

### MonorepoDescriptor Integration

```rust
use sublime_standard_tools::monorepo::MonorepoDescriptor;

// Detect monorepo structure
let monorepo = monorepo_detector.detect_monorepo(&root_path).await?;

// Build dependency graph from structure
let graph = builder.build_from_monorepo(&monorepo).await?;
```

### FileSystem Integration

```rust
use sublime_standard_tools::filesystem::FileSystemManager;

// Uses AsyncFileSystem trait for reading package.json files
let fs = FileSystemManager::new();
let builder = DependencyGraphBuilder::new(fs, config);
```

## Error Handling

The system provides detailed error types for different failure scenarios:

```rust
use sublime_pkg_tools::error::DependencyError;

match error {
    DependencyError::CircularDependency { cycle } => {
        // Handle circular dependency detection
    }
    DependencyError::ResolutionFailed { package, reason } => {
        // Handle dependency resolution failures
    }
    DependencyError::MissingDependency { package, dependency } => {
        // Handle missing dependencies
    }
    DependencyError::GraphConstructionFailed { reason } => {
        // Handle graph construction failures
    }
    DependencyError::PropagationFailed { reason } => {
        // Handle propagation calculation failures
    }
    DependencyError::MaxDepthExceeded { max_depth } => {
        // Handle depth limit violations
    }
}
```

## Best Practices

### Graph Construction

1. **Validate Early**: Always validate graphs after construction
2. **Handle Cycles**: Implement proper error handling for circular dependencies
3. **Limit Depth**: Set appropriate `max_propagation_depth` to prevent infinite propagation

### Propagation Analysis

1. **Incremental Updates**: Only analyze changed packages, not entire graph
2. **Batch Processing**: Group related changes for efficient analysis
3. **Dry Run**: Use dry-run mode to preview propagation effects

### Configuration

1. **Environment-Specific**: Use different configs for dev vs production
2. **Team Alignment**: Ensure team agrees on propagation rules
3. **Documentation**: Document any custom configuration decisions

## Performance Considerations

### Graph Size

- **Memory Usage**: O(V + E) where V = packages, E = dependencies
- **Query Performance**: O(1) for direct relationships, O(V) for transitive queries
- **Cycle Detection**: O(V + E) using Tarjan's algorithm

### Optimization Tips

1. **Selective Analysis**: Only include necessary dependency types
2. **Depth Limits**: Use reasonable `max_propagation_depth`
3. **Caching**: Cache graph construction results when possible

## Examples

See `examples/dependency_graph_example.rs` for a complete working example demonstrating:

- Graph construction from monorepo structure
- Dependency relationship analysis
- Circular dependency detection
- Propagation analysis simulation

## Testing

The dependency graph system includes comprehensive test coverage:

```bash
# Run dependency-specific tests
cargo test dependency_tests

# Run all tests with output
cargo test dependency_tests -- --nocapture

# Test specific functionality
cargo test test_dependency_graph_cycle_detection
```

## Integration Points

### With Changeset System

The dependency graph integrates with the changeset system to:
- Calculate which packages need updates in a changeset
- Determine appropriate version bumps for dependent packages
- Validate changeset consistency across dependency boundaries

### With Release Management

During releases, the dependency graph:
- Ensures all dependent packages are updated consistently
- Validates that dependency versions are compatible
- Calculates the correct order for package releases

### With Version Resolution

The graph works with version resolution to:
- Resolve snapshot versions for development
- Calculate next versions during propagation
- Maintain version consistency across workspace packages

## Troubleshooting

### Common Issues

1. **Circular Dependencies**
   - **Symptom**: Graph validation fails with cycle detection
   - **Solution**: Restructure packages to break cycles

2. **Missing Dependencies**
   - **Symptom**: Packages not found in graph
   - **Solution**: Ensure package.json files are readable and valid

3. **Propagation Depth Exceeded**
   - **Symptom**: Max depth error during propagation
   - **Solution**: Increase `max_propagation_depth` or check for cycles

4. **Performance Issues**
   - **Symptom**: Slow graph operations
   - **Solution**: Optimize dependency inclusion and use depth limits

### Debug Information

Enable detailed logging for troubleshooting:

```rust
// Add debug prints in development
println!("Graph has {} nodes", graph.package_index.len());
println!("Dependencies for {}: {:?}", package, graph.get_dependencies(package));
```

This comprehensive dependency graph system provides the foundation for safe and automated dependency management in complex monorepo structures.