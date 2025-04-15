//! # sublime_workspace_cli
//!
//! `sublime_workspace_cli` is a comprehensive CLI tool for managing monorepos.
//! It provides functionality for handling multiple packages, tracking changes,
//! managing versions, and monitoring workspace status.

pub mod common;
pub mod daemon;

// Re-export commonly used types for convenience
pub use common::config::Config;
pub use common::errors::CliError;

/// Get a list of all available commands
pub fn available_commands() -> anyhow::Result<Vec<common::commands::DiscoveredCommand>> {
    common::commands::get_available_commands()
}

/// Discover commands dynamically
pub fn discover_commands(
) -> anyhow::Result<std::collections::HashMap<String, common::commands::DiscoveredCommand>> {
    common::commands::discover_commands()
}
