//! Changeset management module for sublime_pkg_tools.
//!
//! This module handles changeset creation, validation, storage, and application
//! for package release management. Changesets track changes between releases
//! and coordinate version bumps across multiple packages.
//!
//! # What
//!
//! Provides changeset management functionality:
//! - `Changeset`: Core changeset data structure
//! - `ChangesetManager`: Service for changeset operations
//! - `ChangesetValidator`: Validation logic for changesets
//! - `ChangesetStorage`: File-based changeset persistence
//!
//! # How
//!
//! Changesets are stored as JSON files in the `.changesets/` directory.
//! When applied, they are moved to `.changesets/history/` with release
//! metadata for audit trails.
//!
//! # Why
//!
//! Changesets provide a controlled, reviewable way to manage version
//! bumps across multiple packages while maintaining complete audit
//! trails of all releases.
mod change;
mod entry;
mod manager;
mod release;
mod storage;

#[cfg(test)]
mod tests;

pub use change::{ChangeReason, Changeset};
pub use entry::{ChangeEntry, ChangesetPackage};
pub use manager::{ChangesetManager, ChangesetSummary};
pub use release::{EnvironmentRelease, ReleaseInfo};
pub use storage::{ChangesetStorage, FileBasedChangesetStorage};
