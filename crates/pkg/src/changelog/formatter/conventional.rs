//! Conventional Commits formatter implementation.
//!
//! **What**: Implements the Conventional Commits format specification for changelog generation.
//! This formatter converts internal changelog data structures into a format that groups
//! changes by commit type (feat, fix, perf, etc.) following the Conventional Commits standard.
//!
//! **How**: Groups changelog entries by their commit type (derived from SectionType) and
//! formats them with configurable section titles. Breaking changes are always displayed
//! first, followed by other sections in priority order. The formatter respects configuration
//! settings for links, authors, and custom section titles.
//!
//! **Why**: Conventional Commits is a widely adopted specification for commit messages that
//! provides an easy set of rules for creating an explicit commit history. This formatter
//! makes it simple to generate changelogs that follow this convention, making it easier
//! for users to understand the nature of changes at a glance.
//!
//! # Conventional Commits Specification
//!
//! The Conventional Commits specification is based on the Angular commit message format:
//! - **feat**: A new feature
//! - **fix**: A bug fix
//! - **perf**: Performance improvements
//! - **refactor**: Code changes that neither fix a bug nor add a feature
//! - **docs**: Documentation only changes
//! - **build**: Changes to the build system or dependencies
//! - **ci**: Changes to CI configuration files and scripts
//! - **test**: Adding or correcting tests
//! - **chore**: Other changes that don't modify src or test files
//!
//! Breaking changes are indicated by:
//! - An exclamation mark before the colon (e.g., `feat!:`)
//! - A `BREAKING CHANGE:` footer in the commit message
//!
//! # Section Ordering
//!
//! Sections are ordered by priority:
//! 1. **Breaking Changes** (always first)
//! 2. **Features**
//! 3. **Bug Fixes**
//! 4. **Performance Improvements**
//! 5. **Deprecations**
//! 6. **Documentation**
//! 7. **Code Refactoring**
//! 8. **Build System**
//! 9. **Continuous Integration**
//! 10. **Tests**
//! 11. **Other Changes**
//!
//! # Example Output
//!
//! ```markdown
//! ## [1.0.0] - 2024-01-15
//!
//! ### Breaking Changes
//! - Change API signature ([abc123](repo/commit/abc123))
//!
//! ### Features
//! - Add new feature X ([def456](repo/commit/def456))
//! - Add new feature Y ([ghi789](repo/commit/ghi789))
//!
//! ### Bug Fixes
//! - Fix critical bug in parser ([jkl012](repo/commit/jkl012)) ([#123](repo/issues/123))
//!
//! ### Performance Improvements
//! - Optimize rendering algorithm ([mno345](repo/commit/mno345))
//! ```

use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection, SectionType};
use crate::config::ChangelogConfig;
use std::collections::HashMap;

/// Formatter for Conventional Commits format.
///
/// This formatter converts `Changelog` structures into markdown following
/// the Conventional Commits specification, grouping changes by commit type.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::formatter::ConventionalCommitsFormatter;
/// use sublime_pkg_tools::changelog::Changelog;
/// use sublime_pkg_tools::config::ChangelogConfig;
/// use chrono::Utc;
///
/// let config = ChangelogConfig::default();
/// let formatter = ConventionalCommitsFormatter::new(&config);
///
/// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
/// let formatted = formatter.format(&changelog);
/// ```
#[derive(Debug)]
pub struct ConventionalCommitsFormatter<'a> {
    /// Configuration for formatting options.
    config: &'a ChangelogConfig,
}

impl<'a> ConventionalCommitsFormatter<'a> {
    /// Creates a new Conventional Commits formatter.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for formatting options
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::ConventionalCommitsFormatter;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = ConventionalCommitsFormatter::new(&config);
    /// ```
    #[must_use]
    pub fn new(config: &'a ChangelogConfig) -> Self {
        Self { config }
    }

    /// Formats a changelog into Conventional Commits format.
    ///
    /// Groups entries by their section type (corresponding to conventional commit types)
    /// and formats them with appropriate headers. Breaking changes are always shown first,
    /// followed by other sections in priority order.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog to format
    ///
    /// # Returns
    ///
    /// A markdown string in Conventional Commits format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::ConventionalCommitsFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = ConventionalCommitsFormatter::new(&config);
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

        // Group sections by their type
        let grouped_sections = self.group_sections(&changelog.sections);

        // Format each section in priority order
        let mut sections: Vec<_> = grouped_sections.into_iter().collect();
        sections.sort_by_key(|(section_type, _)| section_type.priority());

        for (section_type, entries) in sections {
            if !entries.is_empty() {
                output.push_str(&self.format_section(&section_type, &entries));
                output.push('\n');
            }
        }

        output
    }

    /// Formats the version header.
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

        // Use template if provided, otherwise use standard format
        if self.config.template.version_header.contains("{version}")
            && self.config.template.version_header.contains("{date}")
        {
            self.config
                .template
                .version_header
                .replace("{version}", &changelog.version)
                .replace("{date}", &date_str)
        } else {
            // Standard Conventional Commits format
            format!("## [{}] - {}", changelog.version, date_str)
        }
    }

    /// Groups changelog sections by their section type.
    ///
    /// Each section type maintains its own list of entries. This allows
    /// proper grouping of all entries of the same type (e.g., all features together).
    ///
    /// # Arguments
    ///
    /// * `sections` - The internal changelog sections to group
    ///
    /// # Returns
    ///
    /// A map of section types to their entries.
    pub(crate) fn group_sections<'b>(
        &self,
        sections: &'b [ChangelogSection],
    ) -> HashMap<SectionType, Vec<&'b ChangelogEntry>> {
        let mut grouped: HashMap<SectionType, Vec<&ChangelogEntry>> = HashMap::new();

        for section in sections {
            for entry in &section.entries {
                grouped.entry(section.section_type).or_default().push(entry);
            }
        }

        grouped
    }

    /// Formats a section with its entries.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The section type
    /// * `entries` - The entries for this section
    ///
    /// # Returns
    ///
    /// The formatted section string.
    pub(crate) fn format_section(
        &self,
        section_type: &SectionType,
        entries: &[&ChangelogEntry],
    ) -> String {
        let mut output = String::new();

        // Section header - use configured title or default
        let title = self.get_section_title(section_type);
        let section_header = self.config.template.section_header.replace("{section}", &title);
        output.push_str(&section_header);
        output.push_str("\n\n");

        // Format each entry
        for entry in entries {
            output.push_str(&self.format_entry(entry));
            output.push('\n');
        }

        output
    }

    /// Gets the title for a section type.
    ///
    /// Uses configured section titles from the conventional config if available,
    /// otherwise falls back to the default title.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The section type
    ///
    /// # Returns
    ///
    /// The section title string.
    pub(crate) fn get_section_title(&self, section_type: &SectionType) -> String {
        // For breaking changes, use the configured breaking_section title
        if *section_type == SectionType::Breaking {
            return self.config.conventional.breaking_section.clone();
        }

        // Try to get custom title from configuration
        let commit_type = self.section_type_to_commit_type(section_type);

        if let Some(custom_title) = self.config.conventional.types.get(&commit_type) {
            return custom_title.clone();
        }

        // Fall back to default title
        section_type.title().to_string()
    }

    /// Maps a section type to its corresponding conventional commit type.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The section type
    ///
    /// # Returns
    ///
    /// The conventional commit type string (e.g., "feat", "fix").
    pub(crate) fn section_type_to_commit_type(&self, section_type: &SectionType) -> String {
        match section_type {
            SectionType::Breaking => "breaking".to_string(),
            SectionType::Features => "feat".to_string(),
            SectionType::Fixes => "fix".to_string(),
            SectionType::Performance => "perf".to_string(),
            SectionType::Deprecations => "deprecate".to_string(),
            SectionType::Documentation => "docs".to_string(),
            SectionType::Refactoring => "refactor".to_string(),
            SectionType::Build => "build".to_string(),
            SectionType::CI => "ci".to_string(),
            SectionType::Tests => "test".to_string(),
            SectionType::Other => "chore".to_string(),
        }
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

        // Add scope if present
        if let Some(ref scope) = entry.scope {
            output.push_str(&format!("**{}**: ", scope));
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
    /// This includes a standard header that explains the Conventional Commits
    /// format and semantic versioning adherence.
    ///
    /// # Returns
    ///
    /// The formatted header string.
    #[must_use]
    pub fn format_header(&self) -> String {
        if self.config.template.header.contains("Conventional Commits")
            || self.config.template.header.contains("conventional")
        {
            // Use custom header if it references Conventional Commits
            self.config.template.header.clone()
        } else {
            // Use standard Conventional Commits header
            String::from(
                "# Changelog\n\n\
                 All notable changes to this project will be documented in this file.\n\n\
                 The format is based on [Conventional Commits](https://www.conventionalcommits.org/),\n\
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
    /// use sublime_pkg_tools::changelog::formatter::ConventionalCommitsFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = ConventionalCommitsFormatter::new(&config);
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
