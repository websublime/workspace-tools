//! Version resolution and dependency propagation module.
//!
//! **What**: Provides comprehensive version management for Node.js packages, including version
//! resolution, dependency propagation, snapshot version generation, and version application with
//! support for both independent and unified versioning strategies.
//!
//! **How**: This module analyzes package dependencies, builds a dependency graph, resolves version
//! conflicts, propagates version changes through dependent packages, and applies versions to
//! package.json files. It supports dry-run mode for previewing changes before applying them.
//!
//! **Why**: To automate complex version management in monorepos and single-package projects,
//! ensuring that version changes are correctly propagated through the dependency graph while
//! avoiding circular dependencies and version conflicts.
//!
//! # Features
//!
//! - **Version Resolution**: Calculate next versions for packages based on changesets
//! - **Dependency Propagation**: Automatically update dependent packages when dependencies change
//! - **Versioning Strategies**: Support independent and unified versioning approaches
//! - **Circular Dependency Detection**: Detect and report circular dependencies
//! - **Snapshot Versions**: Generate snapshot versions for pre-release testing
//! - **Dry-Run Mode**: Preview version changes without modifying files
//! - **Version Spec Management**: Handle workspace:, file:, link:, and portal: protocols
//! - **Monorepo Support**: Handle both monorepo and single-package configurations
//!
//! # Versioning Strategies
//!
//! ## Independent Strategy
//!
//! Each package maintains its own version, incremented only when that specific package changes.
//! This is the default and most flexible strategy.
//!
//! ## Unified Strategy
//!
//! All packages share the same version number and are incremented together, even if only
//! one package changes. This is simpler but may result in more version bumps.
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::{VersionResolver, VersioningStrategy};
//! use sublime_pkg_tools::types::{Changeset, VersionBump};
//! use sublime_pkg_tools::config::PackageToolsConfig;
//! use sublime_standard_tools::filesystem::FileSystemManager;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = PathBuf::from(".");
//! let fs = FileSystemManager::new();
//! let config = PackageToolsConfig::default();
//!
//! // TODO: will be implemented on story 5.1
//! // let resolver = VersionResolver::new(
//! //     workspace_root,
//! //     VersioningStrategy::Independent,
//! //     fs,
//! //     config,
//! // ).await?;
//! //
//! // // Create a changeset
//! // let mut changeset = Changeset::new("main", VersionBump::Minor, vec!["production".to_string()]);
//! // changeset.add_package("my-package");
//! //
//! // // Resolve versions (dry run)
//! // let resolution = resolver.resolve_versions(&changeset).await?;
//! // for update in &resolution.updates {
//! //     println!("{}: {} -> {}", update.name, update.current_version, update.next_version);
//! // }
//! //
//! // // Apply versions for real
//! // let result = resolver.apply_versions(&changeset, false).await?;
//! // println!("Updated {} packages", result.resolution.updates.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Dependency Propagation
//!
//! When a package's version changes, all packages that depend on it are also updated:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::VersionResolver;
//! use sublime_pkg_tools::types::Changeset;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let resolver: VersionResolver = todo!();
//! # let changeset: Changeset = todo!();
//! // TODO: will be implemented on story 5.5
//! // // Package A depends on Package B
//! // // If Package B version changes from 1.0.0 to 1.1.0,
//! // // Package A's dependency on B will be updated to 1.1.0
//! //
//! // let resolution = resolver.resolve_versions(&changeset).await?;
//! //
//! // for update in &resolution.updates {
//! //     println!("Package: {}", update.name);
//! //     println!("  Version: {} -> {}", update.current_version, update.next_version);
//! //
//! //     if !update.dependency_updates.is_empty() {
//! //         println!("  Dependency updates:");
//! //         for dep in &update.dependency_updates {
//! //             println!("    {}: {} -> {}",
//! //                 dep.dependency_name,
//! //                 dep.old_version_spec,
//! //                 dep.new_version_spec
//! //             );
//! //         }
//! //     }
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Snapshot Versions
//!
//! Generate snapshot versions for testing:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::VersionResolver;
//! use sublime_pkg_tools::types::Changeset;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let resolver: VersionResolver = todo!();
//! # let changeset: Changeset = todo!();
//! // TODO: will be implemented on story 5.6
//! // // Generate snapshot versions like "1.2.3-feature.20240101120000"
//! // let resolution = resolver.resolve_snapshot_versions(&changeset).await?;
//! //
//! // for update in &resolution.updates {
//! //     println!("{}: {}", update.name, update.next_version);
//! //     // Output: my-package: 1.2.3-feature.20240101120000
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Circular Dependency Detection
//!
//! Detect circular dependencies in the dependency graph:
//!
//! ```rust,ignore
//! use sublime_pkg_tools::version::VersionResolver;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let resolver: VersionResolver = todo!();
//! // TODO: will be implemented on story 5.3
//! // let resolution = resolver.resolve_versions(&changeset).await?;
//! //
//! // if !resolution.circular_dependencies.is_empty() {
//! //     println!("Warning: Circular dependencies detected!");
//! //     for circular in &resolution.circular_dependencies {
//! //         println!("  Cycle: {:?}", circular.cycle);
//! //     }
//! // }
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! Configure version resolution behavior:
//!
//! ```toml
//! [package_tools.version]
//! strategy = "independent"
//! default_bump = "patch"
//! snapshot_format = "{version}-{branch}.{timestamp}"
//!
//! [package_tools.dependency]
//! propagation_bump = "patch"
//! propagate_dependencies = true
//! propagate_dev_dependencies = false
//! propagate_peer_dependencies = true
//! max_depth = 10
//! fail_on_circular = true
//! skip_workspace_protocol = true
//! skip_file_protocol = true
//! skip_link_protocol = true
//! skip_portal_protocol = true
//! ```
//!
//! # Module Structure
//!
//! This module will contain:
//! - `resolver`: The main `VersionResolver` for orchestrating version operations
//! - `strategy`: Versioning strategy implementations (independent, unified)
//! - `graph`: Dependency graph construction and analysis
//! - `propagation`: Dependency propagation logic
//! - `resolution`: Version resolution results and types
//! - `snapshot`: Snapshot version generation
//! - `application`: Version application to package.json files

#![allow(clippy::todo)]

mod resolver;

#[cfg(test)]
mod tests;

pub use resolver::VersionResolver;
