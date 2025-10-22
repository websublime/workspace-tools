//! Merge commit message generation for releases.
//!
//! **What**: Provides functionality to generate merge commit messages for releases using
//! configurable templates with variable replacement.
//!
//! **How**: This module takes changelog data and metadata (version, package name, etc.) and
//! generates formatted commit messages by replacing template variables with actual values.
//! It supports both single-package and monorepo templates, and can include breaking change
//! warnings when configured.
//!
//! **Why**: To ensure consistent, informative merge commit messages across releases that
//! clearly communicate what changed and provide context about the release.
//!
//! # Features
//!
//! - **Template-based Generation**: Use customizable templates with variable placeholders
//! - **Variable Replacement**: Support for version, package name, date, counts, and summaries
//! - **Breaking Change Warnings**: Automatically include warnings for breaking changes
//! - **Monorepo Support**: Different templates for single-package and monorepo releases
//! - **Changelog Integration**: Extract summaries and statistics from changelog data
//!
//! # Example
//!
//! ```rust,ignore
//! use sublime_pkg_tools::changelog::{generate_merge_commit_message, MergeMessageContext, Changelog};
//! use sublime_pkg_tools::config::GitConfig;
//! use chrono::Utc;
//!
//! // Create a context for the merge message
//! let context = MergeMessageContext::new(
//!     Some("my-package"),
//!     "1.0.0",
//!     Some("0.9.0"),
//!     "Minor",
//!     Utc::now(),
//! )
//! .with_author(Some("John Doe".to_string()))
//! .with_changelog(Some(changelog));
//!
//! // Generate the message using the default configuration
//! let config = GitConfig::default();
//! let message = generate_merge_commit_message(&context, &config);
//! println!("{}", message);
//! // Output:
//! // chore(release): my-package@1.0.0
//! //
//! // Release my-package version 1.0.0
//! //
//! // - 3 new features
//! // - 2 bug fixes
//! // - 1 breaking change
//! //
//! // ⚠️  BREAKING CHANGES: 1
//! ```

use crate::changelog::{Changelog, SectionType};
use crate::config::GitConfig;
use chrono::{DateTime, Utc};

/// Context information for generating a merge commit message.
///
/// Contains all the data needed to replace variables in the merge commit template,
/// including version information, package details, and changelog data.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::MergeMessageContext;
/// use chrono::Utc;
///
/// let context = MergeMessageContext {
///     package_name: Some("my-package".to_string()),
///     version: "1.0.0".to_string(),
///     previous_version: Some("0.9.0".to_string()),
///     bump_type: "Minor".to_string(),
///     date: Utc::now(),
///     author: Some("John Doe".to_string()),
///     changelog: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MergeMessageContext {
    /// Package name (None for single-package projects, Some for monorepo).
    pub package_name: Option<String>,

    /// The version being released.
    pub version: String,

    /// The previous version (if known).
    pub previous_version: Option<String>,

    /// The bump type (Major, Minor, Patch, or None).
    pub bump_type: String,

    /// The release date.
    pub date: DateTime<Utc>,

    /// The author name (current git user).
    pub author: Option<String>,

    /// Optional changelog for extracting summaries and statistics.
    pub changelog: Option<Changelog>,
}

impl MergeMessageContext {
    /// Creates a new merge message context.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Optional package name for monorepo
    /// * `version` - The version being released
    /// * `previous_version` - Optional previous version
    /// * `bump_type` - The version bump type
    /// * `date` - Release date
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::MergeMessageContext;
    /// use chrono::Utc;
    ///
    /// let context = MergeMessageContext::new(
    ///     Some("my-package"),
    ///     "1.0.0",
    ///     Some("0.9.0"),
    ///     "Minor",
    ///     Utc::now(),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        package_name: Option<&str>,
        version: &str,
        previous_version: Option<&str>,
        bump_type: &str,
        date: DateTime<Utc>,
    ) -> Self {
        Self {
            package_name: package_name.map(String::from),
            version: version.to_string(),
            previous_version: previous_version.map(String::from),
            bump_type: bump_type.to_string(),
            date,
            author: None,
            changelog: None,
        }
    }

    /// Sets the author for the context.
    ///
    /// # Arguments
    ///
    /// * `author` - The author name
    #[must_use]
    pub fn with_author(mut self, author: Option<String>) -> Self {
        self.author = author;
        self
    }

    /// Sets the changelog for the context.
    ///
    /// # Arguments
    ///
    /// * `changelog` - The changelog data
    #[must_use]
    pub fn with_changelog(mut self, changelog: Option<Changelog>) -> Self {
        self.changelog = changelog;
        self
    }

    /// Gets the number of breaking changes from the changelog.
    ///
    /// # Returns
    ///
    /// The count of breaking changes, or 0 if no changelog is available.
    #[must_use]
    pub fn breaking_changes_count(&self) -> usize {
        self.changelog.as_ref().map(|cl| cl.breaking_changes().len()).unwrap_or(0)
    }

    /// Gets the number of features from the changelog.
    ///
    /// # Returns
    ///
    /// The count of feature entries, or 0 if no changelog is available.
    #[must_use]
    pub fn features_count(&self) -> usize {
        self.changelog
            .as_ref()
            .map(|cl| {
                cl.sections
                    .iter()
                    .filter(|s| s.section_type == SectionType::Features)
                    .map(|s| s.entries.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Gets the number of bug fixes from the changelog.
    ///
    /// # Returns
    ///
    /// The count of fix entries, or 0 if no changelog is available.
    #[must_use]
    pub fn fixes_count(&self) -> usize {
        self.changelog
            .as_ref()
            .map(|cl| {
                cl.sections
                    .iter()
                    .filter(|s| s.section_type == SectionType::Fixes)
                    .map(|s| s.entries.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Generates a brief summary from the changelog.
    ///
    /// Creates a concise summary of the changelog contents by listing the number
    /// of changes in each category (features, fixes, breaking changes, etc.).
    ///
    /// # Returns
    ///
    /// A formatted summary string, or a default message if no changelog is available.
    ///
    /// # Examples
    ///
    /// The summary might look like:
    /// ```text
    /// - 5 new features
    /// - 3 bug fixes
    /// - 1 breaking change
    /// ```
    #[must_use]
    pub fn changelog_summary(&self) -> String {
        let Some(ref changelog) = self.changelog else {
            return String::from("No changelog available");
        };

        if changelog.is_empty() {
            return String::from("No changes recorded");
        }

        let mut parts = Vec::new();

        let features = self.features_count();
        if features > 0 {
            parts.push(format!(
                "- {} {}",
                features,
                if features == 1 { "new feature" } else { "new features" }
            ));
        }

        let fixes = self.fixes_count();
        if fixes > 0 {
            parts.push(format!("- {} bug {}", fixes, if fixes == 1 { "fix" } else { "fixes" }));
        }

        let breaking = self.breaking_changes_count();
        if breaking > 0 {
            parts.push(format!(
                "- {} breaking {}",
                breaking,
                if breaking == 1 { "change" } else { "changes" }
            ));
        }

        // Count other sections
        for section in &changelog.sections {
            if section.section_type != SectionType::Features
                && section.section_type != SectionType::Fixes
                && section.section_type != SectionType::Breaking
                && !section.is_empty()
            {
                let count = section.entries.len();
                parts.push(format!("- {} {}", count, section.title().to_lowercase()));
            }
        }

        if parts.is_empty() {
            String::from("- Minor updates and improvements")
        } else {
            parts.join("\n")
        }
    }

    /// Checks if this is a monorepo release.
    ///
    /// # Returns
    ///
    /// `true` if a package name is specified, `false` otherwise.
    #[must_use]
    pub fn is_monorepo(&self) -> bool {
        self.package_name.is_some()
    }
}

/// Generates a merge commit message for a release.
///
/// Takes the provided context and configuration to generate a formatted merge commit
/// message with all template variables replaced. Automatically uses the appropriate
/// template (single-package or monorepo) based on the context.
///
/// # Arguments
///
/// * `context` - The merge message context containing all variable values
/// * `config` - Git configuration with templates
///
/// # Returns
///
/// A formatted merge commit message string ready to use.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::{generate_merge_commit_message, MergeMessageContext};
/// use sublime_pkg_tools::config::GitConfig;
/// use chrono::Utc;
///
/// let context = MergeMessageContext::new(
///     Some("my-package"),
///     "1.0.0",
///     Some("0.9.0"),
///     "Minor",
///     Utc::now(),
/// );
///
/// let config = GitConfig::default();
/// let message = generate_merge_commit_message(&context, &config);
///
/// assert!(message.contains("1.0.0"));
/// assert!(message.contains("my-package"));
/// ```
#[must_use]
pub fn generate_merge_commit_message(context: &MergeMessageContext, config: &GitConfig) -> String {
    // Choose the appropriate template
    let template = if context.is_monorepo() {
        &config.monorepo_merge_commit_template
    } else {
        &config.merge_commit_template
    };

    // Replace all variables in the template
    let mut message = replace_variables(template, context);

    // Add breaking changes warning if needed
    if config.include_breaking_warning && context.breaking_changes_count() > 0 {
        let warning = replace_variables(&config.breaking_warning_template, context);
        message.push_str(&warning);
    }

    message
}

/// Replaces template variables with actual values from the context.
///
/// Supports the following variables:
/// - `{version}`: The new version
/// - `{previous_version}`: The previous version (or "N/A" if not available)
/// - `{package_name}`: The package name (or "N/A" if not available)
/// - `{bump_type}`: The version bump type
/// - `{date}`: Release date in YYYY-MM-DD format
/// - `{breaking_changes_count}`: Number of breaking changes
/// - `{features_count}`: Number of new features
/// - `{fixes_count}`: Number of bug fixes
/// - `{changelog_summary}`: Brief summary from changelog
/// - `{author}`: Current git user (or "Unknown" if not available)
///
/// # Arguments
///
/// * `template` - The template string with variable placeholders
/// * `context` - The context containing variable values
///
/// # Returns
///
/// The template string with all variables replaced.
fn replace_variables(template: &str, context: &MergeMessageContext) -> String {
    let date_str = context.date.format("%Y-%m-%d").to_string();
    let previous_version = context.previous_version.as_deref().unwrap_or("N/A");
    let package_name = context.package_name.as_deref().unwrap_or("N/A");
    let author = context.author.as_deref().unwrap_or("Unknown");
    let changelog_summary = context.changelog_summary();

    template
        .replace("{version}", &context.version)
        .replace("{previous_version}", previous_version)
        .replace("{package_name}", package_name)
        .replace("{bump_type}", &context.bump_type)
        .replace("{date}", &date_str)
        .replace("{breaking_changes_count}", &context.breaking_changes_count().to_string())
        .replace("{features_count}", &context.features_count().to_string())
        .replace("{fixes_count}", &context.fixes_count().to_string())
        .replace("{changelog_summary}", &changelog_summary)
        .replace("{author}", author)
}
