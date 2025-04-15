//! # sublime_workspace_cli
//!
//! `sublime_workspace_cli` is a comprehensive CLI tool for managing monorepos.
//! It provides functionality for handling multiple packages, tracking changes,
//! managing versions, and monitoring workspace status.

pub mod common;

// Re-export commonly used types for convenience
pub use common::config::Config;
pub use common::errors::CliError;
