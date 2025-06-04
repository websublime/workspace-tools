//! Core monorepo project types and implementations
//! 
//! This module provides the central `MonorepoProject` type that aggregates
//! functionality from all base crates, as well as the enhanced `MonorepoPackageInfo`
//! type that provides monorepo-specific package information.

mod types;
mod project;
mod package;

pub use types::{
    Changeset,
    ChangesetStatus,
    MonorepoPackageInfo,
    VersionStatus,
};
pub use project::MonorepoProject;