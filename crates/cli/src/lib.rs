//! # Workspace Node Tools CLI
//!
//! A comprehensive command-line interface for managing Node.js workspaces and monorepos
//! with changeset-based version management.
//!
//! ## What
//!
//! This crate provides the `wnt` CLI tool that offers:
//! - Configuration management (`init`, `config`)
//! - Changeset workflow (`changeset add`, `list`, `show`, etc.)
//! - Version bumping (`bump`)
//! - Dependency upgrades (`upgrade`)
//! - Audit and health checks (`audit`)
//! - Change analysis (`changes`)
//!
//! ## How
//!
//! The CLI is built using:
//! - `clap` for argument parsing and command dispatch
//! - `tokio` for async runtime
//! - Internal crates (`sublime-package-tools`, `sublime-standard-tools`, `sublime-git-tools`)
//! - Multiple output formats (human-readable, JSON, compact JSON)
//! - Comprehensive error handling with user-friendly messages
//!
//! ## Why
//!
//! This CLI provides a unified interface for managing complex Node.js workspaces,
//! supporting both single-package repositories and monorepos with multiple versioning
//! strategies. It follows a changeset-based workflow for better change tracking and
//! version management.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools;
//!
//! // The CLI is primarily used as a binary, but the library exports
//! // command execution functions for testing and integration purposes.
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//! - `cli`: CLI definition, argument parsing, and command dispatch
//! - `commands`: Command implementations
//! - `error`: Error types and user-friendly error messages
//! - `output`: Output formatting (human, JSON, tables, progress bars)
//! - `interactive`: Interactive prompts and user input (future)
//! - `utils`: Shared utilities (future)

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// TODO: Re-enable after all commands are implemented
// #![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::panic)]

// Module declarations
// Note: Modules are created as stubs and will be implemented in subsequent stories

/// CLI framework and argument parsing
pub mod cli;

/// Command implementations
pub mod commands;

/// Error types and handling
pub mod error;

/// Output formatting and logging
pub mod output;

/// Interactive prompts and user input
pub mod interactive;

/// Utility modules
pub(crate) mod utils;

// Re-exports for convenience
pub use cli::{Cli, Commands, LogLevel, OutputFormatArg};
pub use error::{CliError, Result};
pub use interactive::prompts;
pub use output::{JsonResponse, MultiProgress, OutputFormat, ProgressBar, Spinner};
