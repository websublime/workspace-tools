//! Data storage services for package management
//!
//! This module provides storage services for dependency data management,
//! implementing thread-safe storage with intelligent version resolution.

pub mod dependency_storage;

// Phase 4.2 integration pending - Storage API re-export for full integration
#[allow(unused_imports)]
pub use dependency_storage::Registry;