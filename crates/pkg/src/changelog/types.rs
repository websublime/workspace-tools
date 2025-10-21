//! Changelog data structures and types.
//!
//! **What**: Defines the core data structures for representing changelogs, including
//! sections, entries, and metadata.
//!
//! **How**: This module provides strongly-typed structures that represent a changelog's
//! content, organized by version, sections (Features, Fixes, etc.), and individual entries.
//! Each structure includes methods for rendering to markdown and accessing data.
//!
//! **Why**: To provide a clear, type-safe representation of changelog data that can be
//! easily manipulated, rendered in different formats, and serialized for storage or API use.

use crate::changelog::SectionType;
use crate::config::ChangelogConfig;
use crate::types::VersionBump;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete changelog for a specific version.
///
/// Represents all changes for a particular version release, organized into
/// sections (Features, Fixes, Breaking Changes, etc.) with metadata about
/// the release.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::{Changelog, ChangelogSection, ChangelogEntry};
/// use chrono::Utc;
///
/// let changelog = Changelog {
///     package_name: Some("my-package".to_string()),
///     version: "1.0.0".to_string(),
///     previous_version: Some("0.9.0".to_string()),
///     date: Utc::now(),
///     sections: vec![],
///     metadata: ChangelogMetadata::default(),
/// };
///
/// println!("Version: {}", changelog.version);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    /// Package name (None for root changelog in monorepo).
    pub package_name: Option<String>,

    /// Version this changelog is for.
    pub version: String,

    /// Previous version (for comparison).
    pub previous_version: Option<String>,

    /// Release date.
    pub date: DateTime<Utc>,

    /// Changelog sections (Features, Fixes, Breaking Changes, etc.).
    pub sections: Vec<ChangelogSection>,

    /// Metadata about this changelog.
    pub metadata: ChangelogMetadata,
}

impl Changelog {
    /// Creates a new changelog instance.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name for monorepo
    /// * `version` - Version string
    /// * `previous_version` - Optional previous version
    /// * `date` - Release date
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use chrono::Utc;
    ///
    /// let changelog = Changelog::new(
    ///     Some("my-package"),
    ///     "1.0.0",
    ///     Some("0.9.0"),
    ///     Utc::now(),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        package_name: Option<&str>,
        version: &str,
        previous_version: Option<&str>,
        date: DateTime<Utc>,
    ) -> Self {
        Self {
            package_name: package_name.map(String::from),
            version: version.to_string(),
            previous_version: previous_version.map(String::from),
            date,
            sections: Vec::new(),
            metadata: ChangelogMetadata::default(),
        }
    }

    /// Adds a section to the changelog.
    ///
    /// # Arguments
    ///
    /// * `section` - The section to add
    pub fn add_section(&mut self, section: ChangelogSection) {
        self.sections.push(section);
    }

    /// Renders the changelog to markdown format.
    ///
    /// # Arguments
    ///
    /// * `config` - Changelog configuration for formatting
    ///
    /// # Returns
    ///
    /// A markdown string representation of the changelog.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
    /// let config = ChangelogConfig::default();
    /// let markdown = changelog.to_markdown(&config);
    /// println!("{}", markdown);
    /// ```
    #[must_use]
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String {
        let mut output = String::new();

        // Version header
        let date_str = self.date.format("%Y-%m-%d").to_string();
        let version_header = config
            .template
            .version_header
            .replace("{version}", &self.version)
            .replace("{date}", &date_str);

        output.push_str(&version_header);
        output.push_str("\n\n");

        // Render each section
        for section in &self.sections {
            if !section.is_empty() {
                output.push_str(&section.to_markdown(config));
                output.push('\n');
            }
        }

        output
    }

    /// Gets all breaking changes from the changelog.
    ///
    /// # Returns
    ///
    /// A vector of references to breaking change entries.
    #[must_use]
    pub fn breaking_changes(&self) -> Vec<&ChangelogEntry> {
        self.sections
            .iter()
            .filter(|s| s.section_type == SectionType::Breaking)
            .flat_map(|s| &s.entries)
            .collect()
    }

    /// Checks if the changelog has any entries.
    ///
    /// # Returns
    ///
    /// `true` if there are no entries in any section, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(|s| s.is_empty())
    }

    /// Gets the total number of entries across all sections.
    ///
    /// # Returns
    ///
    /// The total count of changelog entries.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.sections.iter().map(|s| s.entries.len()).sum()
    }

    /// Checks if this changelog contains breaking changes.
    ///
    /// # Returns
    ///
    /// `true` if there are any breaking changes, `false` otherwise.
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.sections.iter().any(|s| s.section_type == SectionType::Breaking && !s.is_empty())
    }
}

/// Metadata about a changelog.
///
/// Contains additional information about the changelog generation,
/// including Git references, commit counts, and repository URLs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangelogMetadata {
    /// Git tag for this version (if any).
    pub tag: Option<String>,

    /// Commit range used to generate this changelog (e.g., "v1.0.0..v1.1.0").
    pub commit_range: Option<String>,

    /// Total number of commits in this version.
    pub total_commits: usize,

    /// Repository URL for generating links.
    pub repository_url: Option<String>,

    /// Version bump type that was applied.
    pub bump_type: Option<VersionBump>,
}

/// A section within a changelog.
///
/// Groups related changelog entries together (e.g., all Features, all Bug Fixes).
/// Sections are typically rendered with a header and a list of entries.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::{ChangelogSection, SectionType, ChangelogEntry};
///
/// let mut section = ChangelogSection::new(SectionType::Features);
/// section.add_entry(ChangelogEntry {
///     description: "Add new feature".to_string(),
///     commit_hash: "abc123".to_string(),
///     short_hash: "abc123".to_string(),
///     commit_type: Some("feat".to_string()),
///     scope: None,
///     breaking: false,
///     author: "John Doe".to_string(),
///     references: vec![],
///     date: chrono::Utc::now(),
/// });
///
/// println!("Section has {} entries", section.entries.len());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSection {
    /// The type of this section.
    pub section_type: SectionType,

    /// Entries in this section.
    pub entries: Vec<ChangelogEntry>,
}

impl ChangelogSection {
    /// Creates a new changelog section.
    ///
    /// # Arguments
    ///
    /// * `section_type` - The type of section to create
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::{ChangelogSection, SectionType};
    ///
    /// let section = ChangelogSection::new(SectionType::Features);
    /// assert_eq!(section.section_type, SectionType::Features);
    /// ```
    #[must_use]
    pub fn new(section_type: SectionType) -> Self {
        Self { section_type, entries: Vec::new() }
    }

    /// Adds an entry to the section.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to add
    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        self.entries.push(entry);
    }

    /// Renders the section to markdown format.
    ///
    /// # Arguments
    ///
    /// * `config` - Changelog configuration for formatting
    ///
    /// # Returns
    ///
    /// A markdown string representation of the section.
    #[must_use]
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Section header
        let title = self.section_type.title();
        let section_header = config.template.section_header.replace("{section}", title);
        output.push_str(&section_header);
        output.push_str("\n\n");

        // Render each entry
        for entry in &self.entries {
            output.push_str(&entry.to_markdown(config));
            output.push('\n');
        }

        output.push('\n');
        output
    }

    /// Checks if the section has no entries.
    ///
    /// # Returns
    ///
    /// `true` if the section is empty, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets the title for this section.
    ///
    /// # Returns
    ///
    /// The display title for the section type.
    #[must_use]
    pub fn title(&self) -> &str {
        self.section_type.title()
    }
}

/// A single entry in a changelog.
///
/// Represents one change (typically corresponding to a commit) with all
/// relevant metadata like the author, date, commit hash, and any issue
/// references.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::ChangelogEntry;
/// use chrono::Utc;
///
/// let entry = ChangelogEntry {
///     description: "Fix critical bug in parser".to_string(),
///     commit_hash: "abc123def456".to_string(),
///     short_hash: "abc123d".to_string(),
///     commit_type: Some("fix".to_string()),
///     scope: Some("parser".to_string()),
///     breaking: false,
///     author: "Jane Smith".to_string(),
///     references: vec!["#123".to_string()],
///     date: Utc::now(),
/// };
///
/// println!("Entry: {}", entry.description);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Description of the change.
    pub description: String,

    /// Full commit hash.
    pub commit_hash: String,

    /// Short commit hash (typically first 7 characters).
    pub short_hash: String,

    /// Commit type from conventional commits (feat, fix, etc.).
    pub commit_type: Option<String>,

    /// Scope from conventional commits.
    pub scope: Option<String>,

    /// Whether this is a breaking change.
    pub breaking: bool,

    /// Author name.
    pub author: String,

    /// Related issues/PRs (e.g., `["#123", "#456"]`).
    pub references: Vec<String>,

    /// Commit date.
    pub date: DateTime<Utc>,
}

impl ChangelogEntry {
    /// Renders the entry to markdown format.
    ///
    /// # Arguments
    ///
    /// * `config` - Changelog configuration for formatting
    ///
    /// # Returns
    ///
    /// A markdown string representation of the entry.
    #[must_use]
    pub fn to_markdown(&self, config: &ChangelogConfig) -> String {
        let mut output = config.template.entry_format.replace("{description}", &self.description);

        // Add commit hash
        if config.include_commit_links {
            if let Some(ref repo_url) = config.repository_url {
                let commit_link = self.commit_link(repo_url);
                output = output.replace("{hash}", &commit_link);
            } else {
                output = output.replace("{hash}", &self.short_hash);
            }
        } else {
            output = output.replace("{hash}", &self.short_hash);
        }

        // Add issue links if configured
        if config.include_issue_links && !self.references.is_empty() {
            let refs = if let Some(ref repo_url) = config.repository_url {
                self.issue_links(repo_url).join(", ")
            } else {
                self.references.join(", ")
            };
            if !refs.is_empty() {
                output.push_str(&format!(" ({})", refs));
            }
        }

        // Add author if configured
        if config.include_authors && !self.author.is_empty() {
            output.push_str(&format!(" by {}", self.author));
        }

        output
    }

    /// Generates a commit link for the repository.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base repository URL
    ///
    /// # Returns
    ///
    /// A markdown link to the commit.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogEntry;
    /// use chrono::Utc;
    ///
    /// let entry = ChangelogEntry {
    ///     commit_hash: "abc123".to_string(),
    ///     short_hash: "abc123".to_string(),
    ///     // ... other fields
    /// #   description: "test".to_string(),
    /// #   commit_type: None,
    /// #   scope: None,
    /// #   breaking: false,
    /// #   author: "test".to_string(),
    /// #   references: vec![],
    /// #   date: Utc::now(),
    /// };
    ///
    /// let link = entry.commit_link("https://github.com/user/repo");
    /// assert!(link.contains("abc123"));
    /// ```
    #[must_use]
    pub fn commit_link(&self, base_url: &str) -> String {
        let url = base_url.trim_end_matches('/');
        format!("[{}]({}/commit/{})", self.short_hash, url, self.commit_hash)
    }

    /// Generates issue links for the repository.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base repository URL
    ///
    /// # Returns
    ///
    /// A vector of markdown links to issues/PRs.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogEntry;
    /// use chrono::Utc;
    ///
    /// let entry = ChangelogEntry {
    ///     references: vec!["#123".to_string(), "#456".to_string()],
    ///     // ... other fields
    /// #   description: "test".to_string(),
    /// #   commit_hash: "abc123".to_string(),
    /// #   short_hash: "abc123".to_string(),
    /// #   commit_type: None,
    /// #   scope: None,
    /// #   breaking: false,
    /// #   author: "test".to_string(),
    /// #   date: Utc::now(),
    /// };
    ///
    /// let links = entry.issue_links("https://github.com/user/repo");
    /// assert_eq!(links.len(), 2);
    /// ```
    #[must_use]
    pub fn issue_links(&self, base_url: &str) -> Vec<String> {
        let url = base_url.trim_end_matches('/');
        self.references
            .iter()
            .map(|ref_| {
                let issue_num = ref_.trim_start_matches('#');
                format!("[{}]({}/issues/{})", ref_, url, issue_num)
            })
            .collect()
    }
}
