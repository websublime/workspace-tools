//! Changelog generation module for sublime_pkg_tools.
//!
//! This module handles automatic changelog generation from conventional commits
//! and changeset information. It provides flexible formatting options and
//! template support for creating human-readable release documentation.
//!
//! # What
//!
//! Provides changelog generation functionality:
//! - `ChangelogGenerator`: Service for generating changelog content
//! - `ChangelogEntry`: Individual changelog entry representation
//! - `ChangelogSection`: Grouped changelog sections by type
//! - `ChangelogTemplate`: Template system for custom formatting
//!
//! # How
//!
//! Analyzes conventional commits and changeset data to generate structured
//! changelog entries. Supports grouping by commit type, custom templates,
//! and various output formats including Markdown and plain text.
//!
//! # Why
//!
//! Automates changelog creation to ensure consistent documentation of
//! changes between releases, improving communication with users and
//! maintainers about what changed in each version.
mod entry;
mod generator;
mod template;

#[cfg(test)]
mod tests;

pub use entry::ChangelogEntry;
pub use generator::{Changelog, ChangelogGenerator};
pub use template::{ChangelogSection, TemplateContext};
