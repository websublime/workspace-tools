//! Command implementations module.
//!
//! This module contains all command execution logic for the CLI.
//!
//! # What
//!
//! Provides implementations for all CLI commands:
//! - Configuration commands (`init`, `config`)
//! - Changeset commands (`add`, `list`, `show`, `update`, `edit`, `remove`, `history`)
//! - Version management commands (`bump`, `changes`)
//! - Upgrade commands (`check`, `apply`, `rollback`)
//! - Audit commands (`audit` with various modes)
//!
//! # How
//!
//! Each command is implemented as an async function that:
//! 1. Validates arguments
//! 2. Creates necessary managers/services from internal crates
//! 3. Executes the operation
//! 4. Formats and outputs results
//! 5. Returns appropriate exit codes on errors
//!
//! Commands use the `Output` context for consistent formatting across
//! different output modes (human, JSON, compact JSON).
//!
//! # Why
//!
//! Separating command logic from CLI definition improves testability,
//! maintainability, and allows reuse of command logic in other contexts.
//!
//! ## Module Organization
//!
//! Commands will be organized by epic/feature area:
//! - `config.rs` - Configuration management commands
//! - `changeset.rs` - Changeset workflow commands
//! - `version.rs` - Version management commands
//! - `upgrade.rs` - Dependency upgrade commands
//! - `audit.rs` - Audit and health check commands
//! - `changes.rs` - Change analysis commands

// Module exports
pub mod bump;
pub mod changeset;
pub mod config;
pub mod init;
pub mod version;

#[cfg(test)]
mod tests;

// TODO: will be implemented in subsequent stories
// Story 4.2+ will implement remaining changeset commands (update, list, show, delete, history, check)
// Story 5.2 will implement version bump execution
// Story 6.x will implement upgrade commands
// Story 7.x will implement audit commands
