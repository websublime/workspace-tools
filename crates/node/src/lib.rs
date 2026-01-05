//! # `sublime_node_tools`
//!
//! Node.js bindings for Sublime Workspace CLI Tools via napi-rs.
//!
//! ## What
//!
//! This crate provides Node.js bindings via napi-rs for the workspace-tools CLI,
//! exposing 23 execute functions as native JavaScript/TypeScript functions. It enables
//! Node.js and TypeScript applications to use workspace-tools functionality
//! programmatically without spawning CLI processes.
//!
//! ## How
//!
//! The crate uses `#[napi]` macros to:
//! - Define async functions callable from Node.js
//! - Convert Rust types to JavaScript objects via `#[napi(object)]`
//! - Generate TypeScript definitions automatically
//!
//! ### Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         JavaScript/TypeScript                        │
//! │                                                                      │
//! │   const result = await status({ root: "/path/to/project" });        │
//! │                                                                      │
//! │   if (result.success) {                                             │
//! │     console.log(result.data.packages);  // ← Native JS objects      │
//! │   } else {                                                          │
//! │     console.error(result.error.code);   // ← "EVALIDATION"          │
//! │   }                                                                 │
//! └────────────────────────────────┬────────────────────────────────────┘
//!                                  │
//!                                  ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      NAPI Layer (crates/node/)                       │
//! │                                                                      │
//! │   1. Parameter validation (validation.rs)                           │
//! │   2. Conversion JS → Rust structs                                   │
//! │   3. Call execute_* functions from CLI                              │
//! │   4. Capture JSON output                                            │
//! │   5. Parse and convert to NAPI structs                              │
//! │   6. Return ApiResponse<T>                                          │
//! └────────────────────────────────┬────────────────────────────────────┘
//!                                  │
//!                                  ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      CLI Layer (crates/cli/)                         │
//! │                                                                      │
//! │   execute_status(), execute_init(), execute_changeset_add(), ...    │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ### Module Structure
//!
//! - **`error`**: Error handling with Node.js-style error codes (EVALIDATION, EGIT, etc.)
//! - **`validation`**: Parameter validation before CLI execution
//! - **`response`**: `ApiResponse<T>` wrapper for consistent success/error responses
//! - **`types`**: NAPI-compatible structs for parameters and responses
//! - **`commands`**: Implementation of all 23 NAPI functions
//!
//! ## Why
//!
//! Enables Node.js/TypeScript applications to use workspace-tools functionality
//! programmatically without spawning CLI processes, with:
//!
//! - **Full type safety**: TypeScript definitions are auto-generated
//! - **Native performance**: Direct function calls instead of process spawning
//! - **Structured responses**: `ApiResponse<T>` with success/error handling
//! - **Node.js-style errors**: Familiar error codes like ENOENT, EVALIDATION
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```typescript
//! import { status, changesetAdd, bumpPreview } from '@websublime/workspace-tools';
//!
//! // Get workspace status
//! const result = await status({ root: '.' });
//! if (result.success) {
//!   console.log(result.data.packages);
//! } else {
//!   console.error(`Error [${result.error.code}]: ${result.error.message}`);
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```typescript
//! import { changesetAdd } from '@websublime/workspace-tools';
//!
//! const result = await changesetAdd({
//!   root: '.',
//!   packages: ['@scope/pkg1'],
//!   bumpType: 'minor',
//!   message: 'Add new feature'
//! });
//!
//! if (!result.success) {
//!   switch (result.error.code) {
//!     case 'EVALIDATION':
//!       console.error('Invalid parameters:', result.error.message);
//!       break;
//!     case 'EGIT':
//!       console.error('Git error:', result.error.message);
//!       break;
//!     default:
//!       console.error('Unexpected error:', result.error.message);
//!   }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

#[macro_use]
extern crate napi_derive;

pub(crate) mod commands;
pub(crate) mod error;
pub(crate) mod response;
pub(crate) mod types;
pub(crate) mod validation;

// Re-export NAPI functions from commands module
// Status command (Story 3.2)
pub use commands::status;

// Init command (Story 3.4)
pub use commands::init;

// Changeset commands (Story 4.2-4.8)
// Story 4.2: changesetAdd
pub use commands::changeset_add;
// Story 4.3: changesetUpdate
pub use commands::changeset_update;
// Story 4.4: changesetList
pub use commands::changeset_list;
// Story 4.5: changesetShow
pub use commands::changeset_show;
// Story 4.6: changesetRemove
pub use commands::changeset_remove;
// Story 4.7: changesetHistory
pub use commands::changeset_history;
// Story 4.8: changesetCheck
pub use commands::changeset_check;

// Bump commands (Story 5.2-5.4)
// Story 5.2: bumpPreview
pub use commands::bump_preview;
// Story 5.3: bumpApply
pub use commands::bump_apply;
// Story 5.4: bumpSnapshot
pub use commands::bump_snapshot;

// Execute command (Story 6.3)
pub use commands::execute;

// Config commands (Story 7.2-7.3)
// Story 7.2: configShow
pub use commands::config_show;
// TODO: will be implemented on story 8.2-8.4 (upgrade commands)
// TODO: will be implemented on story 9.1-9.3 (remaining commands)

/// Version of the crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of the crate.
///
/// This function is exposed to Node.js and returns the current version
/// of the `sublime_node_tools` crate.
///
/// # Returns
///
/// A string containing the semantic version of the crate.
///
/// # Examples
///
/// ```typescript
/// import { getVersion } from '@websublime/workspace-tools';
///
/// console.log(`Using workspace-tools version: ${getVersion()}`);
/// ```
#[napi]
#[must_use]
pub fn get_version() -> String {
    VERSION.to_string()
}

#[cfg(test)]
mod tests;
