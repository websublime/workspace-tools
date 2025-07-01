//! Changeset management module
//!
//! This module provides functionality for managing changesets in a monorepo environment.
//! Changesets track planned changes to packages with support for multiple deployment
//! environments and integration with the development workflow.
//!
//! # Overview
//!
//! The changeset system allows developers to:
//! - Create changesets that describe planned changes to packages
//! - Track version bumps and deployment environments
//! - Validate changesets before applying them
//! - Deploy changesets to specific environments during development
//! - Apply changesets automatically on merge
//!
//! # Architecture
//!
//! The module is organized into several components:
//! - `types` - Core type definitions for changesets
//! - `storage` - Persistent storage using FileSystemManager
//! - `manager` - Main interface for changeset operations
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
//! ## Deploying to environments
//!
//! ```rust
//! use sublime_monorepo_tools::Environment;
//!
//! # async fn example(manager: &ChangesetManager) -> Result<(), Box<dyn std::error::Error>> {
//! let environments = vec![Environment::Development, Environment::Staging];
//! let result = manager.deploy_to_environments("changeset-id", &environments).await?;
//!
//! if result.success {
//!     println!("Successfully deployed to all environments");
//! } else {
//!     for (env, result) in result.environment_results {
//!         if !result.success {
//!             println!("Failed to deploy to {}: {:?}", env, result.error);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod types;
pub mod storage;
pub mod manager;


// Explicit re-exports from types module
pub use types::{
    // Core types
    Changeset, ChangesetStatus, ChangesetSpec, ChangesetApplication,
    ChangesetFilter, ValidationResult, DeploymentResult, EnvironmentDeploymentResult,
    // Implementation
    ChangesetManager, ChangesetStorage,
};