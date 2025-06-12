//! Changeset type definitions
//!
//! This module contains all type definitions for the changeset system,
//! organized by functional area. Types are separated from their implementations
//! to maintain clean architecture and allow for easier testing.

pub mod core;

// Implementation structs (moved from main modules)
pub mod manager;
pub mod storage;

// Explicit exports to avoid wildcard re-exports
pub use core::{
    Changeset, ChangesetStatus, ChangesetSpec, ChangesetApplication,
    ChangesetFilter, ValidationResult, DeploymentResult, EnvironmentDeploymentResult
};
pub use manager::ChangesetManager;
pub use storage::ChangesetStorage;