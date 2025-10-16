//! Changelog configuration for generation and formatting settings.
//!
//! **What**: Defines configuration for changelog generation, including format selection,
//! conventional commits parsing, and template customization.
//!
//! **How**: This module provides the `ChangelogConfig` structure that controls how changelogs
//! are generated, formatted, and what information they include.
//!
//! **Why**: To enable flexible changelog generation that supports multiple formats and
//! conventions while maintaining consistency and clarity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sublime_standard_tools::config::{ConfigResult, Configurable};

/// Configuration for changelog generation.
///
/// This structure controls all aspects of changelog generation, including format,
/// content, conventional commits parsing, and exclusion rules.
///
/// # Example
///
/// ```rust
/// use sublime_pkg_tools::config::{ChangelogConfig, ChangelogFormat};
///
/// let config = ChangelogConfig::default();
/// assert!(config.enabled);
/// assert_eq!(config.format, ChangelogFormat::KeepAChangelog);
/// assert_eq!(config.filename, "CHANGELOG.md");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangelogConfig {
    /// Whether changelog generation is enabled.
    ///
    /// # Default: `true`
    pub enabled: bool,

    /// The changelog format to use.
    ///
    /// # Default: `ChangelogFormat::KeepAChangelog`
    pub format: ChangelogFormat,

    /// The filename for the changelog.
    ///
    /// # Default: `"CHANGELOG.md"`
    pub filename: String,

    /// Whether to include links to commits in the repository.
    ///
    /// # Default: `true`
    pub include_commit_links: bool,

    /// Whether to include links to issues (e.g., #123).
    ///
    /// # Default: `true`
    pub include_issue_links: bool,

    /// Whether to include author attribution in changelog entries.
    ///
    /// # Default: `false`
    pub include_authors: bool,

    /// Repository URL for generating links.
    ///
    /// If not set, will attempt to auto-detect from git remote.
    ///
    /// # Default: `None`
    pub repository_url: Option<String>,

    /// Monorepo mode for changelog generation.
    ///
    /// # Default: `MonorepoMode::PerPackage`
    pub monorepo_mode: MonorepoMode,

    /// Format for version tags in monorepo packages.
    ///
    /// Supports placeholders: {name}, {version}
    ///
    /// # Default: `"{name}@{version}"`
    pub version_tag_format: String,

    /// Format for root version tags.
    ///
    /// Supports placeholder: {version}
    ///
    /// # Default: `"v{version}"`
    pub root_tag_format: String,

    /// Conventional commits configuration.
    pub conventional: ConventionalConfig,

    /// Exclusion rules for commits.
    pub exclude: ExcludeConfig,

    /// Custom template configuration.
    pub template: TemplateConfig,
}

/// Changelog format type.
///
/// Defines the structure and style of generated changelogs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChangelogFormat {
    /// Keep a Changelog format.
    ///
    /// Follows the structure defined at <https://keepachangelog.com>
    KeepAChangelog,

    /// Conventional Commits format.
    ///
    /// Groups changes by commit type (feat, fix, etc.)
    Conventional,

    /// Custom format using templates.
    Custom,
}

/// Monorepo mode for changelog generation.
///
/// Determines where and how changelogs are generated in a monorepo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MonorepoMode {
    /// Generate one changelog per package in its directory.
    PerPackage,

    /// Generate a single changelog at the repository root.
    Root,

    /// Generate both per-package and root changelogs.
    Both,
}

/// Configuration for conventional commits parsing.
///
/// Controls how conventional commit messages are parsed and categorized.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConventionalConfig {
    /// Whether conventional commits parsing is enabled.
    ///
    /// # Default: `true`
    pub enabled: bool,

    /// Map of commit types to their display titles.
    ///
    /// # Default: Common types like feat, fix, perf, etc.
    pub types: HashMap<String, String>,

    /// Title for the breaking changes section.
    ///
    /// # Default: `"Breaking Changes"`
    pub breaking_section: String,
}

/// Exclusion rules for changelog generation.
///
/// Defines which commits should be excluded from changelogs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExcludeConfig {
    /// Regex patterns for commit messages to exclude.
    ///
    /// # Default: Release commits and merge commits
    pub patterns: Vec<String>,

    /// Authors whose commits should be excluded.
    ///
    /// # Default: empty
    pub authors: Vec<String>,
}

/// Custom template configuration for changelog generation.
///
/// Defines templates for various parts of the changelog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateConfig {
    /// Template for the changelog header.
    ///
    /// # Default: Standard changelog header with description
    pub header: String,

    /// Template for version headers.
    ///
    /// Supports placeholders: {version}, {date}
    ///
    /// # Default: `"## [{version}] - {date}"`
    pub version_header: String,

    /// Template for section headers.
    ///
    /// Supports placeholder: {section}
    ///
    /// # Default: `"### {section}"`
    pub section_header: String,

    /// Template for individual entries.
    ///
    /// Supports placeholders: {description}, {hash}
    ///
    /// # Default: `"- {description} ({hash})"`
    pub entry_format: String,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: ChangelogFormat::KeepAChangelog,
            filename: "CHANGELOG.md".to_string(),
            include_commit_links: true,
            include_issue_links: true,
            include_authors: false,
            repository_url: None,
            monorepo_mode: MonorepoMode::PerPackage,
            version_tag_format: "{name}@{version}".to_string(),
            root_tag_format: "v{version}".to_string(),
            conventional: ConventionalConfig::default(),
            exclude: ExcludeConfig::default(),
            template: TemplateConfig::default(),
        }
    }
}

impl Default for ConventionalConfig {
    fn default() -> Self {
        let mut types = HashMap::new();
        types.insert("feat".to_string(), "Features".to_string());
        types.insert("fix".to_string(), "Bug Fixes".to_string());
        types.insert("perf".to_string(), "Performance Improvements".to_string());
        types.insert("refactor".to_string(), "Code Refactoring".to_string());
        types.insert("docs".to_string(), "Documentation".to_string());
        types.insert("build".to_string(), "Build System".to_string());
        types.insert("ci".to_string(), "Continuous Integration".to_string());
        types.insert("test".to_string(), "Tests".to_string());
        types.insert("chore".to_string(), "Chores".to_string());

        Self { enabled: true, types, breaking_section: "Breaking Changes".to_string() }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            header: "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n".to_string(),
            version_header: "## [{version}] - {date}".to_string(),
            section_header: "### {section}".to_string(),
            entry_format: "- {description} ({hash})".to_string(),
        }
    }
}

impl Configurable for ChangelogConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.filename.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.filename: Filename cannot be empty".to_string(),
            });
        }

        if self.version_tag_format.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.version_tag_format: Version tag format cannot be empty"
                    .to_string(),
            });
        }

        if self.root_tag_format.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.root_tag_format: Root tag format cannot be empty".to_string(),
            });
        }

        self.conventional.validate()?;
        self.exclude.validate()?;
        self.template.validate()?;

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enabled = other.enabled;
        self.format = other.format;
        self.filename = other.filename;
        self.include_commit_links = other.include_commit_links;
        self.include_issue_links = other.include_issue_links;
        self.include_authors = other.include_authors;
        self.repository_url = other.repository_url;
        self.monorepo_mode = other.monorepo_mode;
        self.version_tag_format = other.version_tag_format;
        self.root_tag_format = other.root_tag_format;
        self.conventional.merge_with(other.conventional)?;
        self.exclude.merge_with(other.exclude)?;
        self.template.merge_with(other.template)?;
        Ok(())
    }
}

impl Configurable for ConventionalConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.breaking_section.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.conventional.breaking_section: Breaking section title cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.enabled = other.enabled;
        self.types = other.types;
        self.breaking_section = other.breaking_section;
        Ok(())
    }
}

impl Configurable for ExcludeConfig {
    fn validate(&self) -> ConfigResult<()> {
        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.patterns = other.patterns;
        self.authors = other.authors;
        Ok(())
    }
}

impl Configurable for TemplateConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.header.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.template.header: Header template cannot be empty".to_string(),
            });
        }

        if self.version_header.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message:
                    "changelog.template.version_header: Version header template cannot be empty"
                        .to_string(),
            });
        }

        if self.section_header.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message:
                    "changelog.template.section_header: Section header template cannot be empty"
                        .to_string(),
            });
        }

        if self.entry_format.is_empty() {
            return Err(sublime_standard_tools::config::ConfigError::ValidationError {
                message: "changelog.template.entry_format: Entry format template cannot be empty"
                    .to_string(),
            });
        }

        Ok(())
    }

    fn merge_with(&mut self, other: Self) -> ConfigResult<()> {
        self.header = other.header;
        self.version_header = other.version_header;
        self.section_header = other.section_header;
        self.entry_format = other.entry_format;
        Ok(())
    }
}

