//! Changeset management module
//!
//! This module provides functionality for managing changesets in a monorepo environment.
//! Changesets track planned changes to packages for CI/CD integration and version bump
//! indicators. Designed for CLI and daemon consumption.
//!
//! # Overview
//!
//! The changeset system allows developers to:
//! - Create changesets that describe planned changes to packages
//! - Track version bumps and environment targeting for CLI hooks
//! - Validate changesets before applying them
//! - Apply changesets automatically on merge
//! - Provide bump indicators for CI/CD systems
//!
//! # Architecture
//!
//! The module is organized into several components:
//! - `types` - Core type definitions for changesets
//! - `storage` - JSON-based persistent storage for CLI consumption
//! - `manager` - Main interface for CRUD operations
//! - `tests` - Unit tests for changeset functionality
//!
//! # Examples
//!
//! ## Creating a changeset
//!
//! ```rust
//! use std::sync::Arc;
//! use sublime_monorepo_tools::{
//!     ChangesetManager, ChangesetSpec, MonorepoProject,
//!     VersionBumpType, Environment
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let project = Arc::new(MonorepoProject::new("/path/to/monorepo")?);
//! let manager = ChangesetManager::from_project(project)?;
//!
//! let spec = ChangesetSpec {
//!     package: "@myapp/core".to_string(),
//!     version_bump: VersionBumpType::Minor,
//!     description: "Add new API endpoint for user management".to_string(),
//!     development_environments: vec![Environment::Development, Environment::Staging],
//!     production_deployment: false,
//!     author: None, // Will be inferred from Git config
//! };
//!
//! let changeset = manager.create_changeset(spec).await?;
//! println!("Created changeset: {}", changeset.id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Listing changesets for a package
//!
//! ```rust
//! use sublime_monorepo_tools::{ChangesetManager, ChangesetFilter};
//!
//! # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
//! let filter = ChangesetFilter {
//!     package: Some("@myapp/core".to_string()),
//!     ..Default::default()
//! };
//!
//! let changesets = manager.list_changesets(filter).await?;
//! for changeset in changesets {
//!     println!("{}: {}", changeset.id, changeset.description);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Applying changesets on merge
//!
//! ```rust
//! # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
//! let applications = manager.apply_changesets_on_merge("feature/new-api")?;
//! for app in applications {
//!     println!("Applied changeset {} to {}: {} -> {}",
//!         app.changeset_id, app.package, app.old_version, app.new_version);
//! }
//! # Ok(())
//! # }
//! ```

pub mod manager;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod types;

// Explicit re-exports from types module
pub use types::{
    // Core types
    Changeset,
    ChangesetApplication,
    ChangesetFilter,
    // Implementation
    ChangesetManager,
    ChangesetSpec,
    ChangesetStatus,
    ChangesetStorage,
    ValidationResult,
};
