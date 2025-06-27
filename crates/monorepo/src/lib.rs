#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

//! # Sublime Monorepo Tools
//!
//! A comprehensive library that unifies functionality from base crates to provide complete
//! Node.js monorepo workflows including versioning, diff analysis, task management, and more.
//!
//! ## Features
//!
//! - **Versioning**: Major, Minor, Patch, Snapshot with automatic propagation to dependents
//! - **Diffs**: Recognize differences between branches and affected packages
//! - **Tasks**: Package.json scripts organized as tasks executed based on changes
//! - **Monorepo Analysis**: Package manager detection, dependency graph, internal/external packages
//! - **Monorepo as Project**: Aggregate complete monorepo information
//! - **Changelogs**: Based on conventional commits with customizable templates
//! - **Hooks**: Git hooks (pre-commit, pre-push) with validations
//! - **Changesets**: Change management with deployment environments
//! - **Plugins**: Extensible system for customization
//!
//! ## Architecture
//!
//! This crate is built on top of three foundational crates:
//! - `sublime_git_tools`: Git operations and repository management
//! - `sublime_standard_tools`: File system, command execution, and monorepo detection
//! - `sublime_package_tools`: Package management, dependencies, and version handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::MonorepoTools;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize monorepo tools
//! let tools = MonorepoTools::initialize(Path::new("."))?;
//!
//! // Analyze the monorepo
//! let analysis = tools.analyzer().detect_monorepo_info(Path::new("."))?;
//! println!("Found {} packages", analysis.packages.internal_packages.len());
//!
//! // Run development workflow
//! let result = tools.development_workflow(None).await?;
//! println!("Affected tasks: {}", result.affected_tasks.len());
//! # Ok(())
//! # }
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]
#![allow(clippy::unimplemented)] // Temporarily allow during phased implementation
#![deny(clippy::panic)]

// Public modules
pub mod analysis;
pub mod changes;
pub mod changelog;
pub mod changesets;
pub mod config;
pub mod core;
pub mod error;
pub mod events;
pub mod hooks;
pub mod logging;
pub mod plugins;
pub mod tasks;
pub mod validation;
pub mod workflows;

// Essential API surface - core re-exports for common usage
// Advanced types available through module paths (e.g., crate::changesets::ChangesetApplication)

// Core entry point and project management (2 types)
pub use crate::core::{MonorepoTools, MonorepoProject};

// Essential result and error types (2 types)
pub use crate::error::{Error, Result};

// Main workflow types - primary user interfaces (4 types)
pub use crate::workflows::{DevelopmentWorkflow, ReleaseWorkflow, ReleaseResult, ReleaseOptions};

// Core analysis types (3 types)
pub use crate::analysis::{MonorepoAnalyzer, ChangeAnalysis, AffectedPackagesAnalysis};

// Essential configuration (3 types)
pub use crate::config::{MonorepoConfig, Environment, VersionBumpType};

// Core change detection (2 types)
pub use crate::changes::{PackageChange, ChangeDetector};

// Core version management (2 types)
pub use crate::core::{VersionManager, VersioningResult};

// Plugin system - main interface (2 types)
pub use crate::plugins::{PluginManager, MonorepoPlugin};

// Total: 20 essential types - advanced functionality available via module paths

// Main entry point struct re-exported from core module
// Implementation moved to core/tools.rs for better separation

// Workflow result types moved to workflows/types/lib_results.rs
// for better organization and separation of concerns
