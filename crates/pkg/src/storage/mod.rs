//! Data storage services for package management
//!
//! This module provides storage services for dependency data management,
//! implementing thread-safe storage with intelligent version resolution.

pub mod dependency_storage;

pub use dependency_storage::Registry;