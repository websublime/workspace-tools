//! # `sublime_package_tools`
//!
//! A robust library for managing Node.js packages, dependency graphs, and version handling in Rust.
//!
//! ## Overview
//!
//! `sublime_package_tools` provides advanced utilities for working with Node.js package ecosystems:
//!
//! - **Dependency Management**: Parse, validate, and manipulate package dependencies
//! - **Version Handling**: Semantic versioning utilities, compatibility checking, and upgrade strategies
//! - **Dependency Graph**: Build and visualize dependency graphs with cycle detection
//! - **Package Registry**: Interface with npm and other package registries
//! - **Upgrader**: Find and apply dependency upgrades with various strategies
//!
//! This library is designed for Rust applications that interact with Node.js package ecosystems such as
//! package managers, monorepo tools, or dependency analysis utilities.
//!
//! ## Main Features
//!
//! ### Package and Dependency Management
//!
//! ```rust
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use sublime_package_tools::{Dependency, DependencyRegistry, Package};
//!
//! // Create dependencies using a registry (ensures consistent instances)
//! let mut registry = DependencyRegistry::new();
//! let react_dep = registry.get_or_create("react", "^17.0.2")?;
//! let router_dep = registry.get_or_create("react-router", "^6.0.0")?;
//!
//! // Create a package with dependencies
//! let package = Package::new_with_registry(
//!     "my-app",
//!     "1.0.0",
//!     Some(vec![("react", "^17.0.2"), ("react-router", "^6.0.0")]),
//!     &mut registry
//! )?;
//!
//! // Access package information
//! println!("Package: {} v{}", package.name(), package.version_str());
//!
//! // Work with dependencies
//! for dep in package.dependencies() {
//!     println!("  Depends on: {} {}", dep.borrow().name(), dep.borrow().version());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Dependency Graph Analysis
//!
//! ```rust
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # use sublime_package_tools::{Package, Dependency, DependencyRegistry};
//! # let mut registry = DependencyRegistry::new();
//! # let packages = vec![
//! #     Package::new_with_registry("pkg-a", "1.0.0", Some(vec![("pkg-b", "^1.0.0")]), &mut registry)?,
//! #     Package::new_with_registry("pkg-b", "1.0.0", Some(vec![]), &mut registry)?
//! # ];
//! use sublime_package_tools::{
//!     build_dependency_graph_from_packages,
//!     generate_ascii,
//!     ValidationOptions
//! };
//!
//! // Build a dependency graph
//! let graph = build_dependency_graph_from_packages(&packages);
//!
//! // Validate dependencies with custom options
//! let options = ValidationOptions::new()
//!     .treat_unresolved_as_external(true)
//!     .with_internal_packages(vec!["@internal/ui", "@internal/core"]);
//!
//! let validation = graph.validate_with_options(&options)?;
//!
//! // Check for validation issues
//! if validation.has_critical_issues() {
//!     for issue in validation.critical_issues() {
//!         println!("Critical: {}", issue.message());
//!     }
//! }
//!
//! // Check for version conflicts
//! if let Some(conflicts) = graph.find_version_conflicts() {
//!     for (name, versions) in conflicts {
//!         println!("Package {} has conflicting versions: {}", name, versions.join(", "));
//!     }
//! }
//!
//! // Generate a visualization
//! let ascii = generate_ascii(&graph)?;
//! println!("{}", ascii);
//! # Ok(())
//! # }
//! ```
//!
//! ### Version Management
//!
//! ```rust
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use sublime_package_tools::{Version, VersionRelationship};
//!
//! // Parse versions
//! let version = Version::parse("1.2.3")?;
//!
//! // Bump versions
//! let next_major = Version::bump_major("1.2.3")?;  // -> 2.0.0
//! let next_minor = Version::bump_minor("1.2.3")?;  // -> 1.3.0
//! let next_patch = Version::bump_patch("1.2.3")?;  // -> 1.2.4
//!
//! // Create snapshot version
//! let snapshot = Version::bump_snapshot("1.2.3", "abc123")?;  // -> 1.2.3-alpha.abc123
//!
//! // Compare versions
//! let relationship = Version::compare_versions("1.0.0", "2.0.0");
//! if relationship == VersionRelationship::MajorUpgrade {
//!     println!("This is a major upgrade!");
//! }
//!
//! // Check if a change is breaking
//! if Version::is_breaking_change("1.0.0", "2.0.0") {
//!     println!("Breaking change detected!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Finding Upgrades
//!
//! ```no_run
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # use sublime_package_tools::{Package};
//! # let packages = vec![];
//! # let rc_packages = vec![];
//! use sublime_package_tools::{
//!     Upgrader,
//!     UpgradeConfig,
//!     VersionUpdateStrategy,
//!     ExecutionMode
//! };
//!
//! // Configure an upgrader
//! let config = UpgradeConfig {
//!     update_strategy: VersionUpdateStrategy::MinorAndPatch,
//!     execution_mode: ExecutionMode::DryRun,
//!     ..UpgradeConfig::default()
//! };
//!
//! let mut upgrader = Upgrader::with_config(config);
//!
//! // Check for available upgrades
//! let available_upgrades = upgrader.check_all_upgrades(&packages)?;
//!
//! // Generate report
//! let report = Upgrader::generate_upgrade_report(&available_upgrades);
//! println!("{}", report);
//!
//! // Apply upgrades
//! if !available_upgrades.is_empty() {
//!     let config = UpgradeConfig {
//!         execution_mode: ExecutionMode::Apply,
//!         ..config
//!     };
//!     upgrader.set_config(config);
//!
//!     let applied = upgrader.apply_upgrades(&rc_packages, &available_upgrades)?;
//!     println!("Applied {} upgrades", applied.len());
//! }
//! # Ok(())
//! # }
//! ```

mod dependency;
pub mod errors;
mod graph;
mod package;
mod registry;
mod upgrader;
mod version;

pub use package::{
    cache::CacheEntry,
    change::ChangeType,
    diff::PackageDiff,
    info::PackageInfo,
    package::Package,
    registry::{NpmRegistry, PackageRegistry, PackageRegistryClone},
    scope::{package_scope_name_version, PackageScopeMetadata},
};

pub use dependency::{
    change::DependencyChange, dependency::Dependency, filter::DependencyFilter,
    graph::DependencyGraph, registry::DependencyRegistry, resolution::ResolutionResult,
    update::DependencyUpdate,
};

pub use errors::{Error, Result};

pub use registry::{
    local::LocalRegistry,
    manager::{RegistryAuth, RegistryManager, RegistryType},
};

pub use version::version::{Version, VersionRelationship, VersionStability, VersionUpdateStrategy};

pub use graph::{
    builder::{build_dependency_graph_from_package_infos, build_dependency_graph_from_packages},
    node::{Node, Step},
    validation::{ValidationIssue, ValidationOptions, ValidationReport},
    visualization::{generate_ascii, generate_dot, save_dot_to_file, DotOptions},
};

pub use upgrader::{
    builder::Upgrader,
    config::{ExecutionMode, UpgradeConfig},
    status::{AvailableUpgrade, UpgradeStatus},
};
