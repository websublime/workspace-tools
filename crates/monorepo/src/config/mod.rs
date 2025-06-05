//! Configuration management for monorepo tools
//! 
//! This module provides types and utilities for managing monorepo configurations,
//! including versioning, tasks, changelogs, hooks, changesets, and plugins.

pub mod types;
mod manager;
mod defaults;

#[cfg(test)]
mod tests;

// Re-export all types from the types module
pub use types::*;
pub use manager::ConfigManager;