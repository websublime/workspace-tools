//! Dependency Graph Builder Example
//!
//! This example demonstrates how to use the dependency graph builder
//! to analyze package dependencies in a monorepo structure.

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use sublime_pkg_tools::{
    config::DependencyConfig,
    dependency::{DependencyAnalyzer, DependencyGraphBuilder},
    version::Version,
    ResolvedVersion, VersionBump,
};
use sublime_standard_tools::{
    filesystem::FileSystemManager,
    monorepo::{MonorepoDescriptor, MonorepoKind, WorkspacePackage},
    project::ProjectValidationStatus,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Dependency Graph Builder Example");
    println!("=====================================\n");

    // 1. Setup configuration
    let config = DependencyConfig {
        propagate_updates: true,
        propagate_dev_dependencies: true,
        max_propagation_depth: 5,
        detect_circular: true,
        fail_on_circular: true,
        dependency_update_bump: "patch".to_string(),
        include_peer_dependencies: false,
        include_optional_dependencies: false,
    };

    println!("📋 Configuration:");
    println!("  - Propagate updates: {}", config.propagate_updates);
    println!("  - Include dev deps: {}", config.propagate_dev_dependencies);
    println!("  - Max depth: {}", config.max_propagation_depth);
    println!("  - Default bump: {}\n", config.dependency_update_bump);

    // 2. Create a sample monorepo structure
    let monorepo = create_sample_monorepo();
    println!("🏗️  Created sample monorepo with {} packages:", monorepo.packages().len());
    for package in monorepo.packages() {
        println!("  - {} (v{})", package.name, package.version);
    }
    println!();

    // 3. Build dependency graph
    let filesystem = FileSystemManager::new();
    let builder = DependencyGraphBuilder::new(filesystem.clone(), config.clone());

    println!("🔨 Building dependency graph...");
    let graph = match builder.build_from_monorepo(&monorepo).await {
        Ok(graph) => {
            println!("✅ Successfully built dependency graph");
            graph
        }
        Err(e) => {
            println!("❌ Failed to build graph: {}", e);
            return Ok(());
        }
    };

    // 4. Analyze the graph structure
    println!("\n📊 Dependency Analysis:");

    // Show dependencies for each package
    for package in monorepo.packages() {
        let dependencies = graph.get_dependencies(&package.name);
        let dependents = graph.get_dependents(&package.name);

        println!("  📦 {}:", package.name);
        if !dependencies.is_empty() {
            println!("    Depends on: {}", dependencies.join(", "));
        }
        if !dependents.is_empty() {
            println!("    Depended by: {}", dependents.join(", "));
        }
        if dependencies.is_empty() && dependents.is_empty() {
            println!("    No workspace dependencies");
        }
    }

    // 5. Check for circular dependencies
    println!("\n🔄 Circular Dependency Check:");
    let cycles = graph.detect_cycles();
    if cycles.is_empty() {
        println!("✅ No circular dependencies found");
    } else {
        println!("⚠️  Found {} cycle(s):", cycles.len());
        for (i, cycle) in cycles.iter().enumerate() {
            println!("  Cycle {}: {}", i + 1, cycle.join(" -> "));
        }
    }

    // 6. Create analyzer and validate graph
    let analyzer = DependencyAnalyzer::new(graph, config, filesystem);

    println!("\n🔍 Graph Validation:");
    match analyzer.validate_graph() {
        Ok(()) => println!("✅ Graph is valid"),
        Err(e) => println!("❌ Graph validation failed: {}", e),
    }

    // 7. Simulate dependency propagation
    println!("\n📈 Dependency Propagation Analysis:");
    let mut changed_packages = HashMap::new();

    // Simulate updating the shared library
    let new_version = Version::from_str("2.1.0")?;
    changed_packages.insert(
        "@myorg/shared".to_string(),
        (VersionBump::Minor, ResolvedVersion::Release(new_version)),
    );

    println!("Simulating update: @myorg/shared -> 2.1.0 (minor)");

    match analyzer.analyze_propagation(&changed_packages).await {
        Ok(propagated_updates) => {
            if propagated_updates.is_empty() {
                println!("📭 No propagated updates needed");
            } else {
                println!("📮 Found {} propagated updates:", propagated_updates.len());
                for update in propagated_updates {
                    println!(
                        "  - {}: {} -> {} ({})",
                        update.package_name,
                        update.current_version,
                        update.next_version,
                        update.suggested_bump.as_str()
                    );
                    match update.reason {
                        sublime_pkg_tools::dependency::PropagationReason::DependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!(
                                "    Reason: dependency '{}' updated from {} to {}",
                                dependency, old_version, new_version
                            );
                        }
                        sublime_pkg_tools::dependency::PropagationReason::DevDependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!(
                                "    Reason: dev dependency '{}' updated from {} to {}",
                                dependency, old_version, new_version
                            );
                        }
                        _ => {
                            println!("    Reason: dependency update");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Propagation analysis failed: {}", e);
        }
    }

    println!("\n🎉 Example completed successfully!");
    Ok(())
}

/// Creates a sample monorepo structure for demonstration purposes.
fn create_sample_monorepo() -> MonorepoDescriptor {
    let root = PathBuf::from("/example/monorepo");

    let packages = vec![
        // Shared library that other packages depend on
        WorkspacePackage {
            name: "@myorg/shared".to_string(),
            version: "2.0.0".to_string(),
            location: PathBuf::from("packages/shared"),
            absolute_path: root.join("packages/shared"),
            workspace_dependencies: vec![], // No internal dependencies
            workspace_dev_dependencies: vec![],
        },
        // API service that depends on shared
        WorkspacePackage {
            name: "@myorg/api".to_string(),
            version: "1.5.2".to_string(),
            location: PathBuf::from("packages/api"),
            absolute_path: root.join("packages/api"),
            workspace_dependencies: vec!["@myorg/shared".to_string()],
            workspace_dev_dependencies: vec![],
        },
        // Web app that depends on shared
        WorkspacePackage {
            name: "@myorg/web".to_string(),
            version: "1.2.8".to_string(),
            location: PathBuf::from("packages/web"),
            absolute_path: root.join("packages/web"),
            workspace_dependencies: vec!["@myorg/shared".to_string()],
            workspace_dev_dependencies: vec![],
        },
        // CLI tool that depends on both shared and api
        WorkspacePackage {
            name: "@myorg/cli".to_string(),
            version: "0.8.1".to_string(),
            location: PathBuf::from("packages/cli"),
            absolute_path: root.join("packages/cli"),
            workspace_dependencies: vec!["@myorg/shared".to_string(), "@myorg/api".to_string()],
            workspace_dev_dependencies: vec![],
        },
        // Testing utilities (dev dependency for others)
        WorkspacePackage {
            name: "@myorg/test-utils".to_string(),
            version: "1.0.5".to_string(),
            location: PathBuf::from("packages/test-utils"),
            absolute_path: root.join("packages/test-utils"),
            workspace_dependencies: vec!["@myorg/shared".to_string()],
            workspace_dev_dependencies: vec![],
        },
    ];

    MonorepoDescriptor::new(
        MonorepoKind::YarnWorkspaces,
        root,
        packages,
        None, // package_manager
        None, // package_json
        ProjectValidationStatus::Valid,
    )
}
