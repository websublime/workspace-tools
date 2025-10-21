//! Changelog formatters for different output formats.
//!
//! **What**: Provides formatters that convert changelog data structures into
//! various standard formats like Keep a Changelog, Conventional Commits, and custom templates.
//!
//! **How**: Each formatter implements the formatting logic for a specific changelog
//! standard, taking `Changelog` structures and producing formatted markdown strings.
//!
//! **Why**: To support multiple changelog formats and conventions while maintaining
//! a unified internal representation of changelog data.
//!
//! # Available Formatters
//!
//! - **Keep a Changelog**: Standard format following https://keepachangelog.com
//! - **Conventional Commits**: Automatic grouping by commit type (coming in story 8.6)
//! - **Custom Template**: User-defined templates (coming in story 8.7)
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changelog::formatter::KeepAChangelogFormatter;
//! use sublime_pkg_tools::changelog::{Changelog, ChangelogSection, ChangelogEntry, SectionType};
//! use sublime_pkg_tools::config::ChangelogConfig;
//! use chrono::Utc;
//!
//! // Create a changelog with sections and entries
//! let mut changelog = Changelog::new(Some("my-package"), "1.0.0", Some("0.9.0"), Utc::now());
//!
//! // Add features section
//! let mut features = ChangelogSection::new(SectionType::Features);
//! features.add_entry(ChangelogEntry {
//!     description: "Add new API endpoint".to_string(),
//!     commit_hash: "abc123".to_string(),
//!     short_hash: "abc123".to_string(),
//!     commit_type: Some("feat".to_string()),
//!     scope: None,
//!     breaking: false,
//!     author: "John Doe".to_string(),
//!     references: vec!["#123".to_string()],
//!     date: Utc::now(),
//! });
//! changelog.add_section(features);
//!
//! // Format as Keep a Changelog
//! let config = ChangelogConfig::default();
//! let formatter = KeepAChangelogFormatter::new(&config);
//! let formatted = formatter.format(&changelog);
//!
//! // Output:
//! // ## [1.0.0] - 2024-01-15
//! //
//! // ### Added
//! // - Add new API endpoint (abc123) (#123)
//! ```

mod keep_a_changelog;

// Public exports
pub use keep_a_changelog::KeepAChangelogFormatter;

// TODO: will be implemented on story 8.6
// pub use conventional_commits::ConventionalCommitsFormatter;

// TODO: will be implemented on story 8.7
// pub use custom::CustomTemplateFormatter;

// Tests module - located in tests.rs
#[cfg(test)]
mod tests;
