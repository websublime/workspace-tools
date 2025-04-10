---
type: Page
title: '`sublime_package_tools` Crate Guide'
description: '`sublime_package_tools` is a comprehensive Rust library for working with Node.js packages, providing robust tools for dependency management, semantic versioning, and package graph analysis.'
icon: 🔧
createdAt: '2025-04-09T23:19:53.377Z'
creationDate: 2025-04-10 00:19
modificationDate: 2025-04-10 00:22
tags: [workspace-tools, rust]
coverImage: null
---

# `sublime_package_tools` Crate Guide

[Workspace Tools](https://app.capacities.io/f8d3f900-cfe2-4c6d-b44c-906946be3e3c/52c8c019-b006-462b-8c5d-6b518afa0d64)

## Core Features

- **Package Management**: Parse, create, and manipulate Node.js packages

- **Dependency Analysis**: Build dependency graphs and detect circular dependencies

- **Version Handling**: Semantic version utilities, compatibility checking

- **Dependency Upgrades**: Find and apply dependency upgrades with various strategies

- **Registry Integration**: Interface with npm and other package registries

## Installation

Add the crate to your `Cargo.toml`:

```text
[dependencies]
sublime_package_tools = "0.1.0"
```

## Usage Examples

### Working with Packages and Dependencies

```rust
use sublime_package_tools::{Package, Dependency, DependencyRegistry};
use std::cell::RefCell;
use std::rc::Rc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a registry to manage dependencies
    let mut registry = DependencyRegistry::new();
    // Create a package with dependencies
    let package = Package::new_with_registry(
        "my-app",
        "1.0.0",
        Some(vec![
            ("react", "^17.0.0"),
            ("lodash", "^4.17.21")
        ]),
        &mut registry
    )?;
    // Access package information
    println!("Package: {} v{}", package.name(), package.version_str());
    // Iterate through dependencies
    for dep in package.dependencies() {
        let borrowed = dep.borrow();
        println!("Dependency: {} {}", borrowed.name(), borrowed.version());
    }
    // Update a dependency version
    package.update_dependency_version("react", "^18.0.0")?;
    
    // Update package version
    package.update_version("1.1.0")?;
    
    Ok(())
}
```

### Building and Analyzing Dependency Graphs

```rust
use sublime_package_tools::{
    Package, DependencyRegistry, build_dependency_graph_from_packages, 
    ValidationOptions, generate_ascii
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = DependencyRegistry::new();
    
    // Create packages with dependencies
    let packages = vec![
        Package::new_with_registry("app", "1.0.0", 
            Some(vec![("lib-a", "^1.0.0"), ("lib-b", "^1.0.0")]), 
            &mut registry)?,
        Package::new_with_registry("lib-a", "1.0.0", 
            Some(vec![("lib-c", "^1.0.0")]),
            &mut registry)?,
        Package::new_with_registry("lib-b", "1.0.0", 
            Some(vec![("lib-c", "^1.0.0")]), 
            &mut registry)?,
        Package::new_with_registry("lib-c", "1.0.0", 
            Some(vec![]), 
            &mut registry)?,
    ];
    
    // Build a dependency graph
    let graph = build_dependency_graph_from_packages(&packages);
    
    // Detect circular dependencies
    let cycles = graph.get_cycles();
    if !cycles.is_empty() {
        println!("Circular dependencies detected:");
        for cycle in graph.get_cycle_strings() {
            println!("  {}", cycle.join(" -> "));
        }
    }
    
    // Validate dependencies with custom options
    let options = ValidationOptions::new()
        .treat_unresolved_as_external(true)
        .with_internal_packages(vec!["lib-a", "lib-b", "lib-c"]);
    
    let validation = graph.validate_with_options(&options)?;
    
    // Check for validation issues
    if validation.has_issues() {
        println!("Validation issues found:");
        for issue in validation.issues() {
            println!("  {}", issue.message());
        }
    }
    
    // External dependencies
    let externals = graph.find_external_dependencies();
    println!("External dependencies: {:?}", externals);
    
    // Visualize the dependency graph
    let ascii_graph = generate_ascii(&graph)?;
    println!("Dependency Graph:\n{}", ascii_graph);
    
    Ok(())
}
```

### Semantic Versioning Operations

```rust
use sublime_package_tools::{Version, VersionRelationship};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a version
    let version = Version::parse("1.2.3")?;
    println!("Version: {}.{}.{}", version.major, version.minor, version.patch);
    
    // Create different version bumps
    let major_bump = Version::bump_major("1.2.3")?;
    let minor_bump = Version::bump_minor("1.2.3")?;
    let patch_bump = Version::bump_patch("1.2.3")?;
    let snapshot = Version::bump_snapshot("1.2.3", "abc123")?;
    
    println!("Major bump: {}", major_bump);       // 2.0.0
    println!("Minor bump: {}", minor_bump);       // 1.3.0
    println!("Patch bump: {}", patch_bump);       // 1.2.4
    println!("Snapshot: {}", snapshot);           // 1.2.3-alpha.abc123
    
    // Compare versions
    let relationship = Version::compare_versions("1.0.0", "2.0.0");
    match relationship {
        VersionRelationship::MajorUpgrade => println!("This is a major upgrade"),
        VersionRelationship::MinorUpgrade => println!("This is a minor upgrade"),
        VersionRelationship::PatchUpgrade => println!("This is a patch upgrade"),
        _ => println!("Other relationship: {}", relationship),
    }
    
    // Check if a change is breaking
    if Version::is_breaking_change("1.0.0", "2.0.0") {
        println!("This is a breaking change!");
    }
    
    Ok(())
}
```

### Finding and Applying Dependency Upgrades

```rust
use sublime_package_tools::{
    Package, DependencyRegistry, Upgrader, UpgradeConfig,
    VersionUpdateStrategy, ExecutionMode, Dependency
};
use std::{cell::RefCell, rc::Rc};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create packages
    let mut registry = DependencyRegistry::new();
    
    // Create package with outdated dependencies
    let pkg = Package::new_with_registry(
        "my-app",
        "1.0.0",
        Some(vec![
            ("react", "^16.0.0"),        // Outdated
            ("lodash", "^4.0.0"),        // Outdated
            ("express", "^4.17.0"),      // Recent
        ]),
        &mut registry
    )?;
    
    // Setup an upgrader that looks for minor and patch updates
    let config = UpgradeConfig {
        update_strategy: VersionUpdateStrategy::MinorAndPatch,
        execution_mode: ExecutionMode::DryRun,
        ..UpgradeConfig::default()
    };
    
    let mut upgrader = Upgrader::with_config(config);
    
    // Check for available upgrades
    let upgrades = upgrader.check_package_upgrades(&pkg)?;
    
    // Generate and print a report
    let report = Upgrader::generate_upgrade_report(&upgrades);
    println!("{}", report);
    
    // Create references for actual updates
    let pkg_ref = Rc::new(RefCell::new(pkg));
    
    // Switch to apply mode to actually update dependencies
    upgrader.set_config(UpgradeConfig {
        execution_mode: ExecutionMode::Apply,
        ..config
    });
    
    // Apply the upgrades
    let applied = upgrader.apply_upgrades(&[Rc::clone(&pkg_ref)], &upgrades)?;
    
    println!("Applied {} upgrades:", applied.len());
    for upgrade in &applied {
        println!("  {} {} -> {}", 
            upgrade.dependency_name, 
            upgrade.current_version,
            upgrade.compatible_version.as_deref().unwrap_or("unknown")
        );
    }
    
    Ok(())
}
```

### Working with Package Registries

```rust
use sublime_package_tools::{
    RegistryManager, RegistryType, RegistryAuth, NpmRegistry, PackageRegistry
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a registry manager
    let mut manager = RegistryManager::new();
    
    // Add GitHub packages registry
    manager.add_registry("https://npm.pkg.github.com", RegistryType::GitHub);
    
    // Associate scopes with registries
    manager.associate_scope("@my-org", "https://npm.pkg.github.com")?;
    
    // Add authentication for private registry
    let auth = RegistryAuth {
        token: "my-token".to_string(),
        token_type: "Bearer".to_string(),
        always: false,
    };
    manager.set_auth("https://npm.pkg.github.com", auth)?;
    
    // Query package information
    let latest_version = manager.get_latest_version("react")?;
    println!("Latest version of React: {:?}", latest_version);
    
    // Get all versions of a package
    let all_versions = manager.get_all_versions("lodash")?;
    println!("Available Lodash versions: {:?}", all_versions);
    
    // Query specific version metadata
    let pkg_info = manager.get_package_info("express", "4.17.1")?;
    println!("Package info for Express 4.17.1: {}", pkg_info.to_string());
    
    // Working directly with a registry
    let npm = NpmRegistry::default();
    let versions = npm.get_all_versions("react")?;
    println!("React versions from npm: {:?}", versions);
    
    Ok(())
}
```

## Key Concepts

### Package and Dependency Model

The core of the library is built around `Package` and `Dependency` types, connected through a registry:

```rust
// Create a dependency registry to manage dependency instances
let mut registry = DependencyRegistry::new();
// Create packages that share dependencies through the registry
let app = Package::new_with_registry("app", "1.0.0", 
    Some(vec![("shared", "^1.0.0")]), 
    &mut registry)?;
    
let lib = Package::new_with_registry("lib", "1.0.0", 
    Some(vec![("shared", "^1.0.0")]), 
    &mut registry)?;
```

This ensures that dependencies with the same name are consistently represented, making dependency resolution and validation possible.

### Dependency Graphs

Dependency graphs model relationships between packages:

```rust
// Build a graph from packages
let graph = build_dependency_graph_from_packages(&packages);
// Find cycles (circular dependencies)
if graph.has_cycles() {
    println!("Warning: Circular dependencies found!");
}
// Validate the graph
let report = graph.validate_package_dependencies()?;
```

### Version Management

The library provides utilities for semantic versioning operations:

```rust
// Bump versions
let next_major = Version::bump_major("1.2.3")?;  // 2.0.0
let next_minor = Version::bump_minor("1.2.3")?;  // 1.3.0
let next_patch = Version::bump_patch("1.2.3")?;  // 1.2.4
// Compare versions
let relationship = Version::compare_versions("1.0.0", "2.0.0");
assert_eq!(relationship, VersionRelationship::MajorUpgrade);
```

### Dependency Upgrading

The `Upgrader` provides configurable dependency upgrade mechanisms:

```rust
// Configure an upgrader with a specific strategy
let config = UpgradeConfig {
    update_strategy: VersionUpdateStrategy::MinorAndPatch,  // Don't allow major version bumps
    ..UpgradeConfig::default()
};
let mut upgrader = Upgrader::with_config(config);
// Find and apply upgrades
let upgrades = upgrader.check_package_upgrades(&package)?;
upgrader.apply_upgrades(&[package_ref], &upgrades)?;
```

## Best Practices

1. **Use the Registry**: Always use `DependencyRegistry` to ensure consistency when working with multiple packages

2. **Validate Dependency Graphs**: Check for cycles before performing operations that rely on topological sorting

3. **Configure Version Strategies**: Set appropriate `VersionUpdateStrategy` to balance freshness with stability

4. **Test Upgrades**: Use `ExecutionMode::DryRun` to preview upgrades before applying them

This crate is ideal for building custom package management tools, monorepo utilities, and automated dependency management systems in Rust.

