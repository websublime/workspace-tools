//! Dependency Version Tracking Example
//!
//! This example demonstrates how the dependency graph builder tracks actual
//! dependency versions and uses them in propagation analysis instead of "unknown".

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use sublime_pkg_tools::{
    config::DependencyConfig,
    dependency::{
        DependencyAnalyzer, DependencyEdge, DependencyGraph, DependencyNode, DependencyType,
        PropagationReason,
    },
    version::Version,
    ResolvedVersion, VersionBump,
};
use sublime_standard_tools::filesystem::FileSystemManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Dependency Version Tracking Example");
    println!("======================================\n");

    // 1. Create a realistic dependency graph with actual version requirements
    let mut graph = DependencyGraph::new();

    // Create packages with realistic versions
    let shared_version = Version::from_str("2.0.0")?;
    let api_version = Version::from_str("1.5.2")?;
    let web_version = Version::from_str("1.2.8")?;

    // Shared library (no dependencies)
    let shared_node = DependencyNode::new(
        "@myorg/shared".to_string(),
        shared_version.into(),
        PathBuf::from("/monorepo/packages/shared"),
    );

    // API service that depends on shared
    let mut api_node = DependencyNode::new(
        "@myorg/api".to_string(),
        api_version.into(),
        PathBuf::from("/monorepo/packages/api"),
    );
    api_node.add_dependency("@myorg/shared".to_string(), "^2.0.0".to_string());
    api_node.add_dev_dependency("jest".to_string(), "^29.0.0".to_string());
    api_node.add_dev_dependency("@types/node".to_string(), "^18.0.0".to_string());

    // Web app that depends on both shared and api
    let mut web_node = DependencyNode::new(
        "@myorg/web".to_string(),
        web_version.into(),
        PathBuf::from("/monorepo/packages/web"),
    );
    web_node.add_dependency("@myorg/shared".to_string(), "^2.0.0".to_string());
    web_node.add_dependency("@myorg/api".to_string(), "^1.5.0".to_string());
    web_node.add_dev_dependency("@types/react".to_string(), "^18.0.0".to_string());

    // Add nodes to graph
    graph.add_node(shared_node);
    graph.add_node(api_node);
    graph.add_node(web_node);

    // Add dependency edges with version requirements
    let shared_edge = DependencyEdge::new(DependencyType::Runtime, "^2.0.0".to_string());
    let api_edge = DependencyEdge::new(DependencyType::Runtime, "^1.5.0".to_string());

    graph.add_edge("@myorg/api", "@myorg/shared", shared_edge.clone())?;
    graph.add_edge("@myorg/web", "@myorg/shared", shared_edge)?;
    graph.add_edge("@myorg/web", "@myorg/api", api_edge)?;

    println!("🏗️  Created dependency graph:");
    println!("  📦 @myorg/shared (v2.0.0) - no dependencies");
    println!("  📦 @myorg/api (v1.5.2) - depends on @myorg/shared@^2.0.0");
    println!("  📦 @myorg/web (v1.2.8) - depends on @myorg/shared@^2.0.0, @myorg/api@^1.5.0");
    println!();

    // 2. Analyze current dependency relationships
    println!("📊 Current Dependency Relationships:");
    for package in ["@myorg/shared", "@myorg/api", "@myorg/web"] {
        let dependencies = graph.get_dependencies(package);
        let dependents = graph.get_dependents(package);

        println!("  📦 {}:", package);
        if !dependencies.is_empty() {
            println!("    Dependencies: {}", dependencies.join(", "));
        }
        if !dependents.is_empty() {
            println!("    Dependents: {}", dependents.join(", "));
        }
        if dependencies.is_empty() && dependents.is_empty() {
            println!("    No workspace dependencies");
        }
    }
    println!();

    // 3. Setup analyzer with configuration
    let config = DependencyConfig {
        propagate_updates: true,
        propagate_dev_dependencies: false,
        max_propagation_depth: 5,
        detect_circular: true,
        fail_on_circular: true,
        dependency_update_bump: "patch".to_string(),
        include_peer_dependencies: false,
        include_optional_dependencies: false,
    };

    let filesystem = FileSystemManager::new();
    let analyzer = DependencyAnalyzer::new(graph, config.clone(), filesystem.clone());

    // 4. Simulate updating the shared library
    println!("📈 Simulating @myorg/shared update from v2.0.0 to v2.1.0");
    let mut changed_packages = HashMap::new();
    let new_shared_version = Version::from_str("2.1.0")?;
    changed_packages.insert(
        "@myorg/shared".to_string(),
        (VersionBump::Minor, ResolvedVersion::Release(new_shared_version)),
    );

    match analyzer.analyze_propagation(&changed_packages).await {
        Ok(propagated_updates) => {
            if propagated_updates.is_empty() {
                println!("📭 No propagated updates needed");
            } else {
                println!("📮 Found {} propagated updates:", propagated_updates.len());
                println!();

                for (i, update) in propagated_updates.iter().enumerate() {
                    println!("  Update {}: {}", i + 1, update.package_name);
                    println!("    Current: {}", update.current_version);
                    println!("    Next: {}", update.next_version);
                    println!("    Bump: {}", update.suggested_bump.as_str());

                    // Show detailed version information from the propagation reason
                    match &update.reason {
                        PropagationReason::DependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!("    Reason: Runtime dependency update");
                            println!("      Dependency: {}", dependency);
                            println!("      Old requirement: {}", old_version);
                            println!("      New version: {}", new_version);
                        }
                        PropagationReason::DevDependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!("    Reason: Dev dependency update");
                            println!("      Dependency: {}", dependency);
                            println!("      Old requirement: {}", old_version);
                            println!("      New version: {}", new_version);
                        }
                        PropagationReason::DirectChanges { commits } => {
                            println!("    Reason: Direct changes");
                            println!("      Commits: {:?}", commits);
                        }
                        PropagationReason::OptionalDependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!("    Reason: Optional dependency update");
                            println!("      Dependency: {}", dependency);
                            println!("      Old requirement: {}", old_version);
                            println!("      New version: {}", new_version);
                        }
                        PropagationReason::PeerDependencyUpdate {
                            dependency,
                            old_version,
                            new_version,
                        } => {
                            println!("    Reason: Peer dependency update");
                            println!("      Dependency: {}", dependency);
                            println!("      Old requirement: {}", old_version);
                            println!("      New version: {}", new_version);
                        }
                    }
                    println!();
                }

                // 5. Demonstrate version requirement tracking
                println!("🔍 Version Requirement Analysis:");
                println!("  The system correctly tracks that:");
                for update in &propagated_updates {
                    if let PropagationReason::DependencyUpdate {
                        dependency,
                        old_version,
                        new_version,
                    } = &update.reason
                    {
                        println!(
                            "  • {} had requirement '{}' for {} before update to {}",
                            update.package_name, old_version, dependency, new_version
                        );
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Propagation analysis failed: {}", e);
        }
    }

    // 6. Simulate updating with dev dependencies included
    println!("\n🔧 Testing with dev dependencies enabled:");
    let config_with_dev = DependencyConfig { propagate_dev_dependencies: true, ..config };

    let analyzer_with_dev =
        DependencyAnalyzer::new(analyzer.graph().clone(), config_with_dev, filesystem);

    // Simulate updating a dev dependency
    println!("  Simulating jest update from ^29.0.0 to ^29.5.0");
    let mut dev_changed = HashMap::new();
    let jest_version = Version::from_str("29.5.0")?;
    dev_changed
        .insert("jest".to_string(), (VersionBump::Minor, ResolvedVersion::Release(jest_version)));

    // Note: This won't show propagation in our example since jest is not a workspace package,
    // but demonstrates the configuration difference
    match analyzer_with_dev.analyze_propagation(&dev_changed).await {
        Ok(updates) => {
            if updates.is_empty() {
                println!("  📭 No workspace propagation (jest is external dependency)");
            } else {
                println!("  📮 Dev dependency propagation detected: {} updates", updates.len());
            }
        }
        Err(e) => {
            println!("  ❌ Dev dependency analysis failed: {}", e);
        }
    }

    println!("\n✅ Version tracking demonstrates:");
    println!("  • Real dependency version requirements are captured (not 'unknown')");
    println!("  • Propagation reasons include specific version information");
    println!("  • Different dependency types are handled appropriately");
    println!("  • Configuration controls which dependencies are analyzed");

    println!("\n🎉 Version tracking example completed successfully!");
    Ok(())
}
