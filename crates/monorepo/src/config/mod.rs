//! Configuration management for monorepo tools
//! 
//! This module provides types and utilities for managing monorepo configurations,
//! including versioning, tasks, changelogs, hooks, changesets, and plugins.

mod types;
mod manager;
mod defaults;

pub use types::*;
pub use manager::ConfigManager;