//! Plugin system for extending monorepo functionality
//!
//! This module provides a comprehensive plugin system that allows extending
//! monorepo functionality through custom plugins. Plugins can implement
//! custom analyzers, generators, validators, and other components.
//!
//! # What
//! A flexible plugin system that supports loading, managing, and executing
//! custom plugins for monorepo operations. Supports both built-in and external
//! plugins with a standard interface.
//!
//! # How
//! Uses trait objects and dependency injection to provide a standardized
//! plugin interface. Plugins are loaded dynamically and managed through
//! a central PluginManager that handles lifecycle and execution.
//!
//! # Why
//! Essential for extensibility and customization of monorepo workflows.
//! Allows teams to implement custom logic specific to their needs without
//! modifying core functionality.
//!
//! # Examples
//!
//! ```rust
//! use sublime_monorepo_tools::plugins::{PluginManager, MonorepoPlugin};
//! use sublime_monorepo_tools::core::MonorepoProject;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create plugin manager from project
//! let project = Arc::new(MonorepoProject::new(".")?);
//! let mut manager = PluginManager::from_project(project)?;
//!
//! // Load built-in plugins
//! manager.load_builtin_plugins()?;
//!
//! // Execute plugin command
//! let result = manager.execute_plugin_command("analyzer", "custom-analysis", &[])?;
//! println!("Plugin result: {:?}", result);
//! # Ok(())
//! # }
//! ```

pub mod builtin;
mod manager;
mod registry;
mod types;

#[cfg(test)]
mod tests;

// Re-export main types for public API
pub use types::{
    MonorepoPlugin, PluginCapabilities, PluginCommand, PluginContext, PluginError, PluginInfo,
    PluginLifecycle, PluginResult,
};

pub use manager::PluginManager;
pub use registry::PluginRegistry;
