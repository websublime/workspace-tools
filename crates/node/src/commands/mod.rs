//! Command implementations module for Node.js bindings.
//!
//! # What
//!
//! This module contains the implementations of all NAPI functions that are
//! exposed to JavaScript/TypeScript. Each submodule corresponds to a command
//! group and contains the actual async functions that execute CLI operations.
//!
//! # How
//!
//! The module is organized by command group:
//!
//! - **`status`**: Workspace status command
//! - **`init`**: Workspace initialization command
//! - **`config`**: Configuration commands (show, validate)
//! - **`changeset`**: Changeset workflow commands (add, update, list, show, remove, history, check)
//! - **`bump`**: Version bump commands (preview, apply, snapshot)
//! - **`upgrade`**: Dependency upgrade commands (check, apply, backup)
//! - **`audit`**: Workspace audit command
//! - **`changes`**: Change analysis command
//! - **`clone`**: Repository clone command
//! - **`execute`**: Command execution across packages
//!
//! Each command function:
//! 1. Validates input parameters using the `validation` module
//! 2. Calls the appropriate `execute_*` function from `sublime_cli_tools`
//! 3. Captures and parses the JSON output
//! 4. Returns an `ApiResponse<T>` with the result
//!
//! # Why
//!
//! Separating command implementations by group provides:
//! - Clear organization matching the CLI structure
//! - Easier maintenance and testing
//! - Logical grouping for related functionality
//!
//! # Examples
//!
//! ```typescript
//! import { status, changesetAdd, bumpPreview } from '@websublime/workspace-tools';
//!
//! // All functions return ApiResponse<T>
//! const statusResult = await status({ root: '.' });
//! const changesetResult = await changesetAdd({
//!   root: '.',
//!   packages: ['@scope/pkg'],
//!   bumpType: 'minor',
//!   message: 'Add feature'
//! });
//! const previewResult = await bumpPreview({ root: '.', showDiff: true });
//! ```

// Status command - Story 3.2
pub(crate) mod status;

// Re-export the status function for lib.rs
pub use status::status;

// Tests module for all command implementations
#[cfg(test)]
mod tests;

// TODO: will be implemented on story 3.4 (init command)
pub(crate) mod init;

// TODO: will be implemented on story 7.2-7.3 (config commands)
pub(crate) mod config;

// TODO: will be implemented on story 4.2-4.8 (changeset commands)
pub(crate) mod changeset;

// TODO: will be implemented on story 5.2-5.4 (bump commands)
pub(crate) mod bump;

// TODO: will be implemented on story 8.2-8.4 (upgrade commands)
pub(crate) mod upgrade;

// TODO: will be implemented on story 9.1 (audit command)
pub(crate) mod audit;

// TODO: will be implemented on story 9.2 (changes command)
pub(crate) mod changes;

// TODO: will be implemented on story 9.3 (clone command)
pub(crate) mod clone;

// TODO: will be implemented on story 6.3 (execute command)
pub(crate) mod execute;
