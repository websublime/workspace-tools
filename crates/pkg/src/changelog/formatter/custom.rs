//! Custom template formatter implementation.
//!
//! **What**: Implements a flexible template-based formatter for changelog generation.
//! This formatter allows users to define custom templates with variable substitution
//! to match their organization's specific changelog style.
//!
//! **How**: Uses template strings from configuration with variable placeholders like
//! {version}, {date}, {title}, {description}, etc. The formatter replaces these
//! variables with actual values from the changelog data structures, providing complete
//! control over the output format.
//!
//! **Why**: While Keep a Changelog and Conventional Commits are popular standards,
//! many organizations have their own changelog conventions. Custom templates enable
//! users to maintain their existing style while benefiting from automated changelog
//! generation.
//!
//! # Template Variables
//!
//! ## Version Header Variables
//! - `{version}`: The version number (e.g., "1.0.0")
//! - `{date}`: The release date in YYYY-MM-DD format
//! - `{package}`: The package name (if applicable)
//!
//! ## Section Header Variables
//! - `{title}`: The section title (e.g., "Features", "Bug Fixes")
//! - `{section}`: Alias for {title}
//!
//! ## Entry Variables
//! - `{description}`: The change description
//! - `{hash}`: Full commit hash
//! - `{short_hash}`: Abbreviated commit hash (typically 7 characters)
//! - `{author}`: Commit author name
//! - `{type}`: Commit type (feat, fix, etc.)
//! - `{scope}`: Commit scope (if present)
//! - `{date}`: Commit date in YYYY-MM-DD format
//! - `{references}`: Issue/PR references (e.g., "#123, #456")
//! - `{breaking}`: "BREAKING" marker if this is a breaking change, empty otherwise
//!
//! # Example Templates
//!
//! ## Simple Format
//! ```text
//! version_header = "## Version {version} ({date})"
//! section_header = "### {title}"
//! entry_format = "* {description}"
//! ```
//!
//! ## Detailed Format
//! ```text
//! version_header = "# Release {version} - {date}"
//! section_header = "## {title}"
//! entry_format = "- {breaking}{description} - {author} ({short_hash})"
//! ```
//!
//! ## With Links
//! ```text
//! version_header = "## [{version}] - {date}"
//! section_header = "### {title}"
//! entry_format = "- {description} [{short_hash}] {references}"
//! ```
//!
//! # Example Output
//!
//! Using the default templates:
//!
//! ```markdown
//! ## [1.0.0] - 2024-01-15
//!
//! ### Features
//! - Add new API endpoint (abc123)
//! - Implement user authentication (def456)
//!
//! ### Bug Fixes
//! - Fix memory leak in parser (ghi789)
//! ```

use crate::changelog::{Changelog, ChangelogEntry, ChangelogSection};
use crate::config::ChangelogConfig;

/// Formatter for custom template-based changelog format.
///
/// This formatter provides maximum flexibility by allowing users to define
/// their own templates with variable substitution. Templates can include
/// any combination of supported variables and literal text.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::formatter::CustomTemplateFormatter;
/// use sublime_pkg_tools::changelog::Changelog;
/// use sublime_pkg_tools::config::ChangelogConfig;
/// use chrono::Utc;
///
/// let config = ChangelogConfig::default();
/// let formatter = CustomTemplateFormatter::new(&config);
///
/// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
/// let formatted = formatter.format(&changelog);
/// ```
#[derive(Debug)]
pub struct CustomTemplateFormatter<'a> {
    /// Configuration containing the templates to use.
    config: &'a ChangelogConfig,
}

impl<'a> CustomTemplateFormatter<'a> {
    /// Creates a new custom template formatter.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration containing template definitions
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::CustomTemplateFormatter;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = CustomTemplateFormatter::new(&config);
    /// ```
    #[must_use]
    pub fn new(config: &'a ChangelogConfig) -> Self {
        Self { config }
    }

    /// Formats a changelog using custom templates.
    ///
    /// This method processes the changelog through the configured templates,
    /// replacing all variable placeholders with actual values from the
    /// changelog data.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog to format
    ///
    /// # Returns
    ///
    /// A string with all template variables replaced by actual values.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::CustomTemplateFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = CustomTemplateFormatter::new(&config);
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

        // Format each section
        for section in &changelog.sections {
            if !section.is_empty() {
                output.push_str(&self.format_section(section));
                output.push('\n');
            }
        }

        output
    }

    /// Formats the version header using the configured template.
    ///
    /// Replaces variables in the `version_header` template:
    /// - `{version}`: Version number
    /// - `{date}`: Release date in YYYY-MM-DD format
    /// - `{package}`: Package name (if present)
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog containing version and date information
    ///
    /// # Returns
    ///
    /// The formatted version header string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // With template: "## [{version}] - {date}"
    /// // Output: "## [1.0.0] - 2024-01-15"
    /// ```
    pub(crate) fn format_version_header(&self, changelog: &Changelog) -> String {
        let date_str = changelog.date.format("%Y-%m-%d").to_string();
        let package_name = changelog.package_name.as_deref().unwrap_or("");

        self.config
            .template
            .version_header
            .replace("{version}", &changelog.version)
            .replace("{date}", &date_str)
            .replace("{package}", package_name)
    }

    /// Formats a changelog section using the configured templates.
    ///
    /// Formats the section header and all entries within the section.
    ///
    /// # Arguments
    ///
    /// * `section` - The section to format
    ///
    /// # Returns
    ///
    /// The formatted section as a string.
    pub(crate) fn format_section(&self, section: &ChangelogSection) -> String {
        if section.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Section header
        output.push_str(&self.format_section_header(section));
        output.push_str("\n\n");

        // Format each entry
        for entry in &section.entries {
            output.push_str(&self.format_entry(entry));
            output.push('\n');
        }

        output.push('\n');
        output
    }

    /// Formats the section header using the configured template.
    ///
    /// Replaces variables in the `section_header` template:
    /// - `{title}`: Section title (e.g., "Features", "Bug Fixes")
    /// - `{section}`: Alias for {title}
    ///
    /// # Arguments
    ///
    /// * `section` - The section to format the header for
    ///
    /// # Returns
    ///
    /// The formatted section header string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // With template: "### {title}"
    /// // Output: "### Features"
    /// ```
    pub(crate) fn format_section_header(&self, section: &ChangelogSection) -> String {
        let title = section.title();

        self.config.template.section_header.replace("{title}", title).replace("{section}", title)
    }

    /// Formats a changelog entry using the configured template.
    ///
    /// Replaces variables in the `entry_format` template:
    /// - `{description}`: The change description
    /// - `{hash}`: Full commit hash
    /// - `{short_hash}`: Abbreviated commit hash
    /// - `{author}`: Commit author name
    /// - `{type}`: Commit type (feat, fix, etc.)
    /// - `{scope}`: Commit scope
    /// - `{date}`: Commit date in YYYY-MM-DD format
    /// - `{references}`: Issue/PR references
    /// - `{breaking}`: "BREAKING: " marker for breaking changes
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry to format
    ///
    /// # Returns
    ///
    /// The formatted entry string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // With template: "- {description} ({short_hash})"
    /// // Output: "- Add new feature (abc123d)"
    /// ```
    pub(crate) fn format_entry(&self, entry: &ChangelogEntry) -> String {
        let date_str = entry.date.format("%Y-%m-%d").to_string();
        let commit_type = entry.commit_type.as_deref().unwrap_or("");
        let scope = entry.scope.as_deref().unwrap_or("");
        let breaking_marker = if entry.breaking { "BREAKING: " } else { "" };

        // Format references as a comma-separated list
        let references = if entry.references.is_empty() {
            String::new()
        } else if self.config.include_issue_links {
            self.format_issue_links(&entry.references)
        } else {
            entry.references.join(", ")
        };

        // Build author string if configured
        let author = if self.config.include_authors { entry.author.clone() } else { String::new() };

        // Format hash with optional link
        let hash = if self.config.include_commit_links {
            self.format_commit_link(&entry.commit_hash, &entry.short_hash)
        } else {
            entry.short_hash.clone()
        };

        let short_hash = if self.config.include_commit_links {
            self.format_commit_link(&entry.commit_hash, &entry.short_hash)
        } else {
            entry.short_hash.clone()
        };

        self.config
            .template
            .entry_format
            .replace("{description}", &entry.description)
            .replace("{hash}", &hash)
            .replace("{short_hash}", &short_hash)
            .replace("{author}", &author)
            .replace("{type}", commit_type)
            .replace("{scope}", scope)
            .replace("{date}", &date_str)
            .replace("{references}", &references)
            .replace("{breaking}", breaking_marker)
    }

    /// Formats a commit hash as a link if a repository URL is configured.
    ///
    /// # Arguments
    ///
    /// * `full_hash` - The full commit hash
    /// * `short_hash` - The abbreviated commit hash for display
    ///
    /// # Returns
    ///
    /// A markdown link if repository URL is set, otherwise just the short hash.
    pub(crate) fn format_commit_link(&self, full_hash: &str, short_hash: &str) -> String {
        if let Some(repo_url) = &self.config.repository_url {
            let commit_url = format!("{}/commit/{}", repo_url.trim_end_matches('/'), full_hash);
            format!("[{}]({})", short_hash, commit_url)
        } else {
            short_hash.to_string()
        }
    }

    /// Formats issue references as markdown links if a repository URL is configured.
    ///
    /// # Arguments
    ///
    /// * `references` - Issue/PR references (e.g., `["#123", "#456"]`)
    ///
    /// # Returns
    ///
    /// A space-separated list of markdown links, or plain references if no URL is set.
    pub(crate) fn format_issue_links(&self, references: &[String]) -> String {
        if let Some(repo_url) = &self.config.repository_url {
            references
                .iter()
                .map(|ref_str| {
                    if let Some(issue_num) = ref_str.strip_prefix('#') {
                        let issue_url =
                            format!("{}/issues/{}", repo_url.trim_end_matches('/'), issue_num);
                        format!("[{}]({})", ref_str, issue_url)
                    } else {
                        ref_str.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            references.join(" ")
        }
    }

    /// Formats the complete changelog with header.
    ///
    /// Includes the configured header template followed by all changelog versions.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog to format
    ///
    /// # Returns
    ///
    /// The complete formatted changelog including header.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::CustomTemplateFormatter;
    /// use sublime_pkg_tools::changelog::Changelog;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use chrono::Utc;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = CustomTemplateFormatter::new(&config);
    /// let changelog = Changelog::new(Some("my-package"), "1.0.0", None, Utc::now());
    ///
    /// let complete = formatter.format_complete(&changelog);
    /// // Includes header from config.template.header
    /// ```
    #[must_use]
    pub fn format_complete(&self, changelog: &Changelog) -> String {
        let mut output = String::new();

        // Add header if not empty
        if !self.config.template.header.is_empty() {
            output.push_str(&self.config.template.header);
            if !self.config.template.header.ends_with('\n') {
                output.push('\n');
            }
            output.push('\n');
        }

        // Add changelog content
        output.push_str(&self.format(changelog));

        output
    }

    /// Formats just the header from the template.
    ///
    /// # Returns
    ///
    /// The configured header template.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::formatter::CustomTemplateFormatter;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    ///
    /// let config = ChangelogConfig::default();
    /// let formatter = CustomTemplateFormatter::new(&config);
    ///
    /// let header = formatter.format_header();
    /// println!("{}", header);
    /// ```
    #[must_use]
    pub fn format_header(&self) -> String {
        self.config.template.header.clone()
    }
}
