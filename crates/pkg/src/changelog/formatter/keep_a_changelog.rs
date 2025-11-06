//! Keep a Changelog formatter implementation.
//!
//! **What**: Implements the Keep a Changelog format specification for changelog generation.
//! This formatter converts internal changelog data structures into the standard format
//! defined at <https://keepachangelog.com>.
//!
//! **How**: Maps internal section types to Keep a Changelog sections (Added, Changed,
//! Deprecated, Removed, Fixed, Security) and formats entries according to the specification.
//! The formatter respects configuration settings for links, authors, and templates.
//!
//! **Why**: Keep a Changelog is a widely adopted standard that provides a consistent,
//! human-readable format for documenting changes. Following this standard makes it easier
//! for users to understand what has changed between versions.
//!
//! # Keep a Changelog Specification
//!
//! The format follows these principles:
//! - Changelogs are for humans, not machines
//! - There should be an entry for every single version
//! - The same types of changes should be grouped
//! - Versions and sections should be linkable
//! - The latest version comes first
//! - The release date of each version is displayed
//!
//! Standard sections (in order):
//! - **Added** for new features
//! - **Changed** for changes in existing functionality
//! - **Deprecated** for soon-to-be removed features
//! - **Removed** for now removed features
//! - **Fixed** for any bug fixes
//! - **Security** for security vulnerability fixes
//!
//! # Section Mapping
//!
//! Internal `SectionType` values are mapped to Keep a Changelog sections as follows:
//! - `Features` → Added
//! - `Fixes` → Fixed
//! - `Deprecations` → Deprecated
//! - `Performance` → Changed
//! - `Refactoring` → Changed
//! - `Documentation` → Changed
//! - `Build` → Changed
//! - `CI` → Changed
//! - `Tests` → Changed
//! - `Breaking` → Changed (with special notation)
//! - `Other` → Changed
//!
//! # Example Output
//!
//! ```markdown
//! ## [1.0.0] - 2024-01-15
//!
//! ### Added
//! - New feature X ([abc123](repo/commit/abc123))
//! - New feature Y ([def456](repo/commit/def456))
//!
//! ### Changed
//! - **BREAKING**: Updated API behavior ([ghi789](repo/commit/ghi789))
//! - Improved performance of Z ([jkl012](repo/commit/jkl012))
//!
//! ### Fixed
//! - Fixed bug in parser ([mno345](repo/commit/mno345)) ([#123](repo/issues/123))
//! ```

use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection, SectionType};
use crate::config::ChangelogConfig;
use std::collections::HashMap;

/// Formatter for Keep a Changelog format.
///
/// This formatter converts `Changelog` structures into markdown following
/// the Keep a Changelog specification.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::formatter::KeepAChangelogFormatter;
/// use sublime_pkg_tools::changelog::Changelog;
/// use sublime_pkg_tools::config::ChangelogConfig;
/// use chrono::Utc;
///
/// let config = ChangelogConfig::default();
/// let formatter = KeepAChangelogFormatter::new(&config);
///
/// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
/// let formatted = formatter.format(&changelog);
/// ```
#[derive(Debug)]
pub struct KeepAChangelogFormatter<'a> {
    /// Configuration for formatting options.
    config: &'a ChangelogConfig,
}

/// Keep a Changelog section type.
///
/// Represents the standard sections defined in the Keep a Changelog specification.
///
/// # Note on Unused Variants
///
/// The `Removed` and `Security` sections are part of the Keep a Changelog specification
/// but are not currently mapped from internal `SectionType` values. They are included
/// for completeness and future extensibility. When custom section support is added in
/// future stories, these sections will be available for use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub(crate) enum KeepAChangelogSection {
    /// New features.
    Added,
    /// Changes in existing functionality.
    Changed,
    /// Soon-to-be removed features.
    Deprecated,
    /// Now removed features.
    Removed,
    /// Bug fixes.
    Fixed,
    /// Security vulnerability fixes.
    Security,
}

impl KeepAChangelogSection {
    /// Returns the section title.
    ///
    /// # Returns
    ///
    /// The standard Keep a Changelog section title.
    pub(crate) fn title(&self) -> &str {
        match self {
            KeepAChangelogSection::Added => "Added",
            KeepAChangelogSection::Changed => "Changed",
            KeepAChangelogSection::Deprecated => "Deprecated",
            KeepAChangelogSection::Removed => "Removed",
            KeepAChangelogSection::Fixed => "Fixed",
            KeepAChangelogSection::Security => "Security",
        }
    }

    /// Returns the section priority for ordering.
    ///
    /// Lower numbers appear first. This follows the Keep a Changelog
    /// standard section ordering.
    ///
    /// # Returns
    ///
    /// Priority value (0-5).
    pub(crate) fn priority(&self) -> u8 {
        match self {
            KeepAChangelogSection::Added => 0,
            KeepAChangelogSection::Changed => 1,
            KeepAChangelogSection::Deprecated => 2,
            KeepAChangelogSection::Removed => 3,
            KeepAChangelogSection::Fixed => 4,
            KeepAChangelogSection::Security => 5,
        }
    }
}

impl<'a> KeepAChangelogFormatter<'a> {
    /// Creates a new Keep a Changelog formatter.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for formatting options
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::KeepAChangelogFormatter;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = KeepAChangelogFormatter::new(&config);
    /// ```
    #[must_use]
    pub fn new(config: &'a ChangelogConfig) -> Self {
        Self { config }
    }

    /// Formats a changelog into Keep a Changelog format.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog to format
    ///
    /// # Returns
    ///
    /// A markdown string in Keep a Changelog format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::KeepAChangelogFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = KeepAChangelogFormatter::new(&config);
    /// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
    ///
    /// let formatted = formatter.format(&changelog);
    /// println!("{}", formatted);
    /// ```
    #[must_use]
    pub fn format(&self, changelog: &Changelog) -> String {
        let mut output = String::new();

        // Version header
        output.push_str(&self.format_version_header(changelog));
        output.push_str("\n\n");

        // Group sections by Keep a Changelog categories
        let grouped_sections = self.group_sections(&changelog.sections);

        // Format each Keep a Changelog section in priority order
        let mut sections: Vec<_> = grouped_sections.into_iter().collect();
        sections.sort_by_key(|(section, _)| section.priority());

        for (keep_section, entries) in sections {
            if !entries.is_empty() {
                output.push_str(&self.format_section(&keep_section, &entries));
                output.push('\n');
            }
        }

        output
    }

    /// Formats the version header according to Keep a Changelog format.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog to format the header for
    ///
    /// # Returns
    ///
    /// The formatted version header string.
    pub(crate) fn format_version_header(&self, changelog: &Changelog) -> String {
        let date_str = changelog.date.format("%Y-%m-%d").to_string();

        // Use template if provided, otherwise use Keep a Changelog standard format
        if self.config.template.version_header.contains("{version}")
            && self.config.template.version_header.contains("{date}")
        {
            self.config
                .template
                .version_header
                .replace("{version}", &changelog.version)
                .replace("{date}", &date_str)
        } else {
            // Standard Keep a Changelog format
            format!("## [{}] - {}", changelog.version, date_str)
        }
    }

    /// Groups internal sections into Keep a Changelog sections.
    ///
    /// # Arguments
    ///
    /// * `sections` - The internal changelog sections to group
    ///
    /// # Returns
    ///
    /// A map of Keep a Changelog sections to entries.
    pub(crate) fn group_sections<'b>(
        &self,
        sections: &'b [ChangelogSection],
    ) -> HashMap<KeepAChangelogSection, Vec<&'b ChangelogEntry>> {
        let mut grouped: HashMap<KeepAChangelogSection, Vec<&ChangelogEntry>> = HashMap::new();

        for section in sections {
            let keep_section = self.map_section_type(&section.section_type);

            for entry in &section.entries {
                grouped.entry(keep_section).or_default().push(entry);
            }
        }

        grouped
    }

    /// Maps internal `SectionType` to Keep a Changelog section.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The internal section type
    ///
    /// # Returns
    ///
    /// The corresponding Keep a Changelog section.
    pub(crate) fn map_section_type(&self, section_type: &SectionType) -> KeepAChangelogSection {
        match section_type {
            SectionType::Features => KeepAChangelogSection::Added,
            SectionType::Fixes => KeepAChangelogSection::Fixed,
            SectionType::Deprecations => KeepAChangelogSection::Deprecated,
            SectionType::Breaking => KeepAChangelogSection::Changed,
            SectionType::Performance => KeepAChangelogSection::Changed,
            SectionType::Refactoring => KeepAChangelogSection::Changed,
            SectionType::Documentation => KeepAChangelogSection::Changed,
            SectionType::Build => KeepAChangelogSection::Changed,
            SectionType::CI => KeepAChangelogSection::Changed,
            SectionType::Tests => KeepAChangelogSection::Changed,
            SectionType::Other => KeepAChangelogSection::Changed,
        }
    }

    /// Formats a Keep a Changelog section with its entries.
    ///
    /// # Arguments
    ///
    /// * `section` - The Keep a Changelog section type
    /// * `entries` - The entries for this section
    ///
    /// # Returns
    ///
    /// The formatted section string.
    pub(crate) fn format_section(
        &self,
        section: &KeepAChangelogSection,
        entries: &[&ChangelogEntry],
    ) -> String {
        let mut output = String::new();

        // Section header
        output.push_str(&format!("### {}\n\n", section.title()));

        // Format each entry
        for entry in entries {
            output.push_str(&self.format_entry(entry));
            output.push('\n');
        }

        output
    }

    /// Formats a single changelog entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to format
    ///
    /// # Returns
    ///
    /// The formatted entry string.
    pub(crate) fn format_entry(&self, entry: &ChangelogEntry) -> String {
        let mut output = String::from("- ");

        // Add breaking change marker if applicable
        if entry.breaking {
            output.push_str("**BREAKING**: ");
        }

        // Add description
        output.push_str(&entry.description);

        // Add commit link
        if self.config.include_commit_links {
            output.push(' ');
            if let Some(ref repo_url) = self.config.repository_url {
                let commit_link = self.format_commit_link(entry, repo_url);
                output.push_str(&commit_link);
            } else {
                output.push_str(&format!("({})", entry.short_hash));
            }
        }

        // Add issue links
        if self.config.include_issue_links && !entry.references.is_empty() {
            output.push(' ');
            if let Some(ref repo_url) = self.config.repository_url {
                let issue_links = self.format_issue_links(entry, repo_url);
                output.push_str(&format!("({})", issue_links.join(", ")));
            } else {
                let refs = entry.references.join(", ");
                output.push_str(&format!("({})", refs));
            }
        }

        // Add author
        if self.config.include_authors && !entry.author.is_empty() {
            output.push_str(&format!(" by {}", entry.author));
        }

        output
    }

    /// Formats a commit link for the repository.
    ///
    /// # Arguments
    ///
    /// * `entry` - The changelog entry
    /// * `base_url` - Base repository URL
    ///
    /// # Returns
    ///
    /// A markdown link to the commit.
    pub(crate) fn format_commit_link(&self, entry: &ChangelogEntry, base_url: &str) -> String {
        let url = base_url.trim_end_matches('/');
        format!("[{}]({}/commit/{})", entry.short_hash, url, entry.commit_hash)
    }

    /// Formats issue links for the repository.
    ///
    /// # Arguments
    ///
    /// * `entry` - The changelog entry
    /// * `base_url` - Base repository URL
    ///
    /// # Returns
    ///
    /// A vector of markdown links to issues/PRs.
    pub(crate) fn format_issue_links(&self, entry: &ChangelogEntry, base_url: &str) -> Vec<String> {
        let url = base_url.trim_end_matches('/');
        entry
            .references
            .iter()
            .map(|ref_| {
                let issue_num = ref_.trim_start_matches('#');
                format!("[{}]({}/issues/{})", ref_, url, issue_num)
            })
            .collect()
    }

    /// Formats the complete changelog header with description.
    ///
    /// This includes the standard Keep a Changelog header and description
    /// that explains the format and semantic versioning adherence.
    ///
    /// # Returns
    ///
    /// The formatted header string.
    #[must_use]
    pub fn format_header(&self) -> String {
        if self.config.template.header.contains("Keep a Changelog")
            || self.config.template.header.contains("keepachangelog")
        {
            // Use custom header if it references Keep a Changelog
            self.config.template.header.clone()
        } else {
            // Use standard Keep a Changelog header
            String::from(
                "# Changelog\n\n\
                 All notable changes to this project will be documented in this file.\n\n\
                 The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),\n\
                 and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n",
            )
        }
    }

    /// Formats multiple changelog versions into a complete changelog file.
    ///
    /// # Arguments
    ///
    /// * `changelogs` - Vector of changelogs to format (should be in reverse chronological order)
    ///
    /// # Returns
    ///
    /// A complete changelog file content.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::KeepAChangelogFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = KeepAChangelogFormatter::new(&config);
    ///
    /// let changelogs = vec![
    ///     Changelog::new(Some("pkg"), "1.1.0", Some("1.0.0"), Utc::now()),
    ///     Changelog::new(Some("pkg"), "1.0.0", None, Utc::now()),
    /// ];
    ///
    /// let complete = formatter.format_complete(&changelogs);
    /// ```
    #[must_use]
    pub fn format_complete(&self, changelogs: &[Changelog]) -> String {
        let mut output = self.format_header();

        // Add unreleased section if configured
        output.push_str("## [Unreleased]\n\n");

        // Format each version
        for changelog in changelogs {
            output.push_str(&self.format(changelog));
        }

        output
    }
}
