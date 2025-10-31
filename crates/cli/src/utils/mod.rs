//! Utility modules for CLI operations.
//!
//! This module provides utility functions and helpers used across the CLI.
//!
//! # What
//!
//! Contains utility modules for:
//! - `editor` - Editor detection and file opening functionality
//!
//! # How
//!
//! Each utility module provides focused functionality that can be imported
//! and used by command implementations. These utilities are implementation
//! details not exposed in the public API.
//!
//! # Why
//!
//! Centralizing utilities provides:
//! - Reusable functionality across commands
//! - Consistent behavior for common operations
//! - Easier testing and maintenance
//! - Clean separation of concerns
//!
//! # Examples
//!
//! ```rust,ignore
//! use sublime_cli_tools::utils::editor::open_in_editor;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let file = Path::new("changeset.json");
//! open_in_editor(file)?;
//! # Ok(())
//! # }
//! ```

pub(crate) mod editor;

#[cfg(test)]
mod tests;
