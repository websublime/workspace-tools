//! Version management and bump strategies for monorepo packages.
//!
//! This module provides functionality for managing package versions across a monorepo,
//! including version bump suggestions based on conventional commits, dependency-aware
//! version updates, and changelog generation.
//!
//! # Key Components
//!
//! - **VersionManager**: Central component for coordinating version operations
//! - **VersionBumpStrategy**: Strategies for determining how versions should be bumped
//! - **ChangelogOptions**: Configuration for generating changelogs
//!
//! # Cycle Handling
//!
//! The versioning system includes special handling for packages in circular dependency
//! relationships, ensuring consistent versioning within cycles.
//!
//! # Examples
//!
//! ```no_run
//! use sublime_monorepo_tools::{
//!     ChangeTracker, VersionBumpStrategy, VersionManager, Workspace
//! };
//!
//! # fn example(workspace: &Workspace, tracker: &ChangeTracker) -> Result<(), Box<dyn std::error::Error>> {
//! // Create version manager
//! let manager = VersionManager::new(workspace, Some(tracker));
//!
//! // Define a bump strategy
//! let strategy = VersionBumpStrategy::Independent {
//!     major_if_breaking: true,
//!     minor_if_feature: true,
//!     patch_otherwise: true,
//! };
//!
//! // Preview version bumps
//! let preview = manager.preview_bumps(&strategy)?;
//! println!("Would bump {} packages", preview.changes.len());
//!
//! // Apply version bumps
//! let changes = manager.apply_bumps(&strategy, false)?;
//! println!("Applied {} version bumps", changes.len());
//! # Ok(())
//! # }
//! ```

pub mod bump;
pub mod changelog;
pub mod error;
pub mod strategy;
pub mod suggest;
