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

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![allow(clippy::unimplemented)] // Temporarily allow during phased implementation
#![deny(clippy::panic)]

// Public modules
pub mod analysis;
pub mod config;
pub mod core;
pub mod error;

// Re-exports
pub use crate::analysis::{MonorepoAnalysisResult, MonorepoAnalyzer};
pub use crate::config::{ConfigManager, Environment, MonorepoConfig, VersionBumpType};
pub use crate::core::{
    Changeset, ChangesetStatus, MonorepoPackageInfo, MonorepoProject, VersionStatus,
};
pub use crate::error::{Error, Result};

// Main entry point - will be implemented in later phases
/// The main orchestrator for monorepo tools functionality
pub struct MonorepoTools {
    project: std::sync::Arc<MonorepoProject>,
}

impl MonorepoTools {
    /// Initialize MonorepoTools by detecting and opening a monorepo at the given path
    pub fn initialize(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::initialize will be implemented in Phase 6")
    }

    /// Get a reference to the monorepo analyzer
    pub fn analyzer(&self) -> &MonorepoAnalyzer {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::analyzer will be implemented in Phase 6")
    }

    /// Run the development workflow
    #[allow(clippy::unused_async)]
    pub async fn development_workflow(&self, _since: Option<&str>) -> Result<DevelopmentResult> {
        // Placeholder - will be implemented in Phase 6
        unimplemented!("MonorepoTools::development_workflow will be implemented in Phase 6")
    }
}

/// Result of a development workflow execution
pub struct DevelopmentResult {
    /// Tasks that were executed
    pub affected_tasks: Vec<String>,
}
