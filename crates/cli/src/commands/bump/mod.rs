//! Bump command implementations module.
//!
//! This module contains all version bump related command execution logic.
//!
//! # What
//!
//! Provides implementations for version bump commands:
//! - `preview` - Preview version bumps without applying changes (default, story 5.1)
//! - `execute` - Apply version bumps and update files (story 5.2)
//!
//! # How
//!
//! The bump command:
//! 1. Loads workspace configuration to determine versioning strategy
//! 2. Loads all active changesets from the changeset directory
//! 3. Uses `VersionResolver` from sublime-package-tools to calculate version bumps
//! 4. Displays preview or applies changes based on command flags
//! 5. Handles both Independent and Unified versioning strategies correctly
//!
//! ## Versioning Strategy Behavior
//!
//! ### Independent Strategy
//! - Only packages listed in `changeset.packages` receive version bumps
//! - Packages not in any active changeset remain at their current version
//! - Dependency propagation: Package A's dependencies are updated, but A's version
//!   only bumps if A is also in a changeset
//!
//! ### Unified Strategy
//! - ALL workspace packages receive the same version bump
//! - When ANY package listed in changesets requires a bump, ALL packages get bumped
//! - The highest bump type from all changesets is applied (major > minor > patch)
//!
//! # Why
//!
//! Centralizing bump command logic provides:
//! - Consistent version management workflow
//! - Clear preview of what will change before applying
//! - Proper handling of both versioning strategies
//! - Support for CI/CD automation via JSON output
//! - Safe default (dry-run/preview mode)
//!
//! # Examples
//!
//! ```rust,no_run
//! use sublime_cli_tools::commands::bump::execute_bump_preview;
//! use sublime_cli_tools::cli::commands::BumpArgs;
//! use sublime_cli_tools::output::{Output, OutputFormat};
//! use std::io;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let args = BumpArgs {
//!     dry_run: true,
//!     execute: false,
//!     snapshot: false,
//!     snapshot_format: None,
//!     prerelease: None,
//!     packages: None,
//!     git_tag: false,
//!     git_push: false,
//!     git_commit: false,
//!     no_changelog: false,
//!     no_archive: false,
//!     force: false,
//! };
//!
//! let output = Output::new(OutputFormat::Human, io::stdout(), false);
//! let root = Path::new(".");
//! execute_bump_preview(&args, &output, root, None).await?;
//! # Ok(())
//! # }
//! ```

pub mod execute;
pub mod git_integration;
pub mod preview;
pub mod snapshot;

#[cfg(test)]
mod tests;

// Re-export command functions for convenience
pub use execute::execute_bump_apply;
pub use preview::execute_bump_preview;
