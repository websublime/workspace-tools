//! Data storage services for package management
//!
//! This module provides storage services for dependency data management,
//! implementing thread-safe storage with intelligent version resolution.

pub mod dependency_storage;

// TODO: Re-enable when refactored to use standard crate patterns
// pub use dependency_storage::Registry;