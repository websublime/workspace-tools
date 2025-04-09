//! Change tracking system for monorepo packages.
//!
//! This module provides functionality for tracking changes to packages in a monorepo,
//! including storing, retrieving, and managing these changes for versioning and release
//! management purposes.
//!
//! ## Key Components
//!
//! - **Change**: Represents a single atomic change to a package
//! - **Changeset**: Groups related changes together
//! - **ChangeStore**: Interface for storing and retrieving changes
//! - **ChangeTracker**: High-level API for tracking changes
//!
//! ## Change Storage Implementations
//!
//! - **FileChangeStore**: Persists changes to the filesystem as JSON files
//! - **MemoryChangeStore**: Stores changes in memory (useful for testing)
//!
//! ## Example
//!
//! ```no_run
//! use sublime_monorepo_tools::{
//!     Change, ChangeType, Changeset, FileChangeStore, ChangeStore, ChangeTracker, Workspace
//! };
//! use std::rc::Rc;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a workspace
//! let workspace = Workspace::new(Path::new(".").to_path_buf(), Default::default(), None)?;
//!
//! // Create a change store and tracker
//! let store = FileChangeStore::new(".changes")?;
//! let mut tracker = ChangeTracker::new(Rc::new(workspace), Box::new(store));
//!
//! // Create a change
//! let change = Change::new(
//!     "ui-components",
//!     ChangeType::Feature,
//!     "Add new button component",
//!     false
//! );
//!
//! // Record the change
//! tracker.record_change(change)?;
//!
//! // Get unreleased changes for release planning
//! let unreleased = tracker.unreleased_changes()?;
//! # Ok(())
//! # }
//! ```

pub mod change;
pub mod changeset;
pub mod error;
pub mod file;
pub mod memory;
pub mod store;
pub mod tracker;
