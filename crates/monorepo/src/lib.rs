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
//! A streamlined library for CLI and daemon consumption that provides essential
//! Node.js monorepo functionality including project management, analysis, versioning,
//! change detection, and more.
//!
//! ## Features
//!
//! - **Project Management**: Direct MonorepoProject integration with base crates
//! - **Analysis**: Dependency graph analysis and change detection with < 1s performance
//! - **Versioning**: Major, Minor, Patch with automatic propagation to dependents
//! - **Change Detection**: Efficient package change analysis for CLI operations
//! - **Configuration**: Streamlined monorepo configuration management
//! - **Changelogs**: Conventional commits parsing with customizable templates
//! - **Tasks**: Synchronous task execution optimized for CLI responsiveness
//!
//! ## Architecture
//!
//! This crate provides direct integration with foundational base crates:
//! - `sublime_git_tools`: Git operations and repository management
//! - `sublime_standard_tools`: File system, command execution, and monorepo detection  
//! - `sublime_package_tools`: Package management, dependencies, and version handling
//!
//! ## Core API (12 Types)
//!
//! The public API is intentionally minimal for CLI/daemon usage:
//!
//! ```rust,no_run
//! use sublime_monorepo_tools::{MonorepoProject, MonorepoAnalyzer, Result};
//! use std::path::Path;
//!
//! fn example() -> Result<()> {
//!     // Initialize project with direct base crate integration
//!     let project = MonorepoProject::new(".")?;
//!     
//!     // Perform analysis with sub-second performance
//!     let analyzer = MonorepoAnalyzer::new(&project);
//!     let changes = analyzer.detect_changes_since("HEAD~1", None)?;
//!     
//!     println!("Found {} affected packages", changes.package_changes.len());
//!     Ok(())
//! }
//! ```
//!
//! Advanced functionality is available through module paths (e.g., `crate::changelog::ChangelogManager`).

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]
#![deny(clippy::panic)]

// Public modules
pub mod analysis;
pub mod changelog;
pub mod changes;
pub mod changesets;
pub mod config;
pub mod core;
pub mod error;
pub mod tasks;

// Essential API surface - core re-exports for CLI/daemon consumption
// Target: Exactly 12 public types for focused API
// Advanced types available through module paths (e.g., crate::changesets::ChangesetApplication)

// Core project management (1 type)
pub use crate::core::MonorepoProject;

// Core analysis (2 types)
pub use crate::analysis::{ChangeAnalysis, MonorepoAnalyzer};

// Configuration (3 types)
pub use crate::config::{Environment, MonorepoConfig, VersionBumpType};

// Version management (2 types)
pub use crate::core::{VersionManager, VersioningResult};

// Change detection (2 types)  
pub use crate::changes::{ChangeDetector, PackageChange};

// Error handling (2 types)
pub use crate::error::{Error, Result};

// Total: 12 essential types - focused API for CLI/daemon consumption

// Main entry point struct re-exported from core module
// Implementation moved to core/tools.rs for better separation

// Workflow result types moved to workflows/types/lib_results.rs
// for better organization and separation of concerns
