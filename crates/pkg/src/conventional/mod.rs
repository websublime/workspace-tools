//! Conventional commit parsing module for sublime_pkg_tools.
//!
//! This module handles parsing and interpretation of conventional commits
//! according to the conventional commits specification. It extracts semantic
//! information from commit messages to determine appropriate version bumps
//! and changelog entries.
//!
//! # What
//!
//! Provides conventional commit functionality:
//! - `ConventionalCommit`: Parsed commit representation
//! - `CommitType`: Enumeration of standard commit types
//! - `ConventionalCommitParser`: Service for parsing commit messages
//! - `BreakingChangeDetector`: Detection of breaking changes
//!
//! # How
//!
//! Uses regular expressions to parse commit messages according to the
//! conventional commits specification. Extracts type, scope, description,
//! body, and footer information while detecting breaking changes.
//!
//! # Why
//!
//! Enables automatic version bump calculation and changelog generation
//! based on semantic commit messages, reducing manual effort and
//! ensuring consistent release practices.
mod commit;
mod parser;

#[cfg(test)]
mod tests;

pub use commit::{BreakingChange, CommitType, CommitTypeConfig, ConventionalCommit};
pub use parser::ConventionalCommitParser;
