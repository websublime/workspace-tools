//! Conventional Commits parser implementation.
//!
//! **What**: Provides parsing and categorization of commit messages following the
//! Conventional Commits specification (<https://www.conventionalcommits.org/>).
//!
//! **How**: Uses regular expressions to parse the commit subject line, then processes
//! the body and footers. Detects breaking changes through both the `!` indicator and
//! `BREAKING CHANGE:` footers. All parsing operations return proper Result types for
//! error handling.
//!
//! **Why**: To enable automated changelog generation by categorizing commits into
//! meaningful sections (features, fixes, etc.) and detecting breaking changes.
//!
//! # Conventional Commits Format
//!
//! ```text
//! <type>[optional scope]: <description>
//!
//! [optional body]
//!
//! [optional footer(s)]
//! ```
//!
//! # Configuration Integration
//!
//! This module works with [`crate::config::ConventionalConfig`] for customizing:
//! - Commit type to section title mappings
//! - Breaking changes section title
//! - Exclusion patterns
//!
//! The parser itself is configuration-independent, but section titles can be
//! customized when generating changelogs using the configuration.
//!
//! # Examples
//!
//! Basic feature commit:
//! ```text
//! feat: add user authentication
//! ```
//!
//! With scope:
//! ```text
//! fix(api): resolve timeout issues
//! ```
//!
//! Breaking change with `!`:
//! ```text
//! feat(core)!: redesign public API
//! ```
//!
//! With body and footer:
//! ```text
//! feat(auth): add OAuth2 support
//!
//! This implements OAuth2 authentication flow with support
//! for multiple providers.
//!
//! Refs: #123
//! BREAKING CHANGE: removes basic auth support
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{ChangelogError, ChangelogResult};

// Note: ConventionalConfig from config module provides customization for
// section titles and commit type mappings when generating changelogs.
// The parser itself remains configuration-independent for simplicity.

/// Parsed conventional commit following the Conventional Commits specification.
///
/// A conventional commit has a structured format that enables automated
/// changelog generation and semantic versioning.
///
/// # Format
///
/// ```text
/// <type>[optional scope]: <description>
///
/// [optional body]
///
/// [optional footer(s)]
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::conventional::ConventionalCommit;
///
/// // Parse a simple feature commit
/// let commit = ConventionalCommit::parse("feat: add new API")?;
/// assert_eq!(commit.commit_type(), "feat");
/// assert_eq!(commit.description(), "add new API");
/// assert!(!commit.is_breaking());
///
/// // Parse a breaking change
/// let commit = ConventionalCommit::parse("feat!: redesign API")?;
/// assert!(commit.is_breaking());
///
/// // Parse with scope
/// let commit = ConventionalCommit::parse("fix(core): resolve bug")?;
/// assert_eq!(commit.scope(), Some("core"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConventionalCommit {
    /// Commit type (feat, fix, docs, etc.)
    pub(crate) commit_type: String,

    /// Optional scope indicating the area of change
    pub(crate) scope: Option<String>,

    /// Whether this commit contains breaking changes
    pub(crate) breaking: bool,

    /// Short description from the subject line
    pub(crate) description: String,

    /// Optional detailed body text
    pub(crate) body: Option<String>,

    /// Footer key-value pairs
    pub(crate) footers: Vec<CommitFooter>,
}

impl ConventionalCommit {
    /// Parses a commit message into a `ConventionalCommit`.
    ///
    /// The parser follows the Conventional Commits specification strictly and will
    /// return an error for messages that don't conform to the format.
    ///
    /// # Format
    ///
    /// The subject line must match: `<type>[optional scope][!]: <description>`
    ///
    /// # Breaking Changes
    ///
    /// Breaking changes are detected in two ways:
    /// - The `!` indicator after type/scope: `feat!:` or `feat(scope)!:`
    /// - A `BREAKING CHANGE:` or `BREAKING-CHANGE:` footer
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::conventional::ConventionalCommit;
    ///
    /// // Simple commit
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert_eq!(commit.commit_type(), "feat");
    ///
    /// // With scope
    /// let commit = ConventionalCommit::parse("fix(api): resolve bug")?;
    /// assert_eq!(commit.scope(), Some("api"));
    ///
    /// // Breaking change with !
    /// let commit = ConventionalCommit::parse("feat!: breaking change")?;
    /// assert!(commit.is_breaking());
    ///
    /// // With body and footers
    /// let message = "feat: add feature\n\nDetailed description\n\nRefs: #123";
    /// let commit = ConventionalCommit::parse(message)?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message is empty
    /// - The subject line doesn't match the conventional format
    /// - The regex compilation fails
    pub fn parse(message: &str) -> ChangelogResult<Self> {
        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() || lines[0].trim().is_empty() {
            return Err(ChangelogError::ConventionalCommitParseError {
                commit: message.to_string(),
                reason: "Empty commit message".to_string(),
            });
        }

        let first_line = lines[0];

        // Parse type, scope, breaking, description from subject line
        let (commit_type, scope, breaking_indicator, description) =
            Self::parse_subject(first_line, message)?;

        // Parse body and footers from remaining lines
        let (body, footers) = Self::parse_body_and_footers(&lines[1..]);

        // Check for BREAKING CHANGE in footers
        let breaking = breaking_indicator
            || footers.iter().any(|f| f.key == "BREAKING CHANGE" || f.key == "BREAKING-CHANGE");

        Ok(Self { commit_type, scope, breaking, description, body, footers })
    }

    /// Returns the commit type (feat, fix, docs, etc.).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert_eq!(commit.commit_type(), "feat");
    /// ```
    pub fn commit_type(&self) -> &str {
        &self.commit_type
    }

    /// Returns the optional scope.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let commit = ConventionalCommit::parse("feat(api): add endpoint")?;
    /// assert_eq!(commit.scope(), Some("api"));
    ///
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert_eq!(commit.scope(), None);
    /// ```
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    /// Returns whether this commit contains breaking changes.
    ///
    /// Breaking changes are indicated by either:
    /// - The `!` marker in the subject line
    /// - A `BREAKING CHANGE:` footer
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let commit = ConventionalCommit::parse("feat!: breaking")?;
    /// assert!(commit.is_breaking());
    ///
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert!(!commit.is_breaking());
    /// ```
    pub fn is_breaking(&self) -> bool {
        self.breaking
    }

    /// Returns the commit description.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert_eq!(commit.description(), "add feature");
    /// ```
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the optional body text.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let message = "feat: add feature\n\nDetailed description";
    /// let commit = ConventionalCommit::parse(message)?;
    /// assert_eq!(commit.body(), Some("Detailed description"));
    /// ```
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// Returns the commit footers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let message = "feat: add feature\n\nRefs: #123\nCloses: #456";
    /// let commit = ConventionalCommit::parse(message)?;
    /// assert_eq!(commit.footers().len(), 2);
    /// ```
    pub fn footers(&self) -> &[CommitFooter] {
        &self.footers
    }

    /// Maps this commit to a changelog section type.
    ///
    /// Breaking changes always map to the `Breaking` section regardless of type.
    /// Other types are mapped according to conventional commits conventions.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let commit = ConventionalCommit::parse("feat: add feature")?;
    /// assert_eq!(commit.section_type(), SectionType::Features);
    ///
    /// let commit = ConventionalCommit::parse("feat!: breaking")?;
    /// assert_eq!(commit.section_type(), SectionType::Breaking);
    /// ```
    pub fn section_type(&self) -> SectionType {
        if self.breaking {
            return SectionType::Breaking;
        }

        match self.commit_type.as_str() {
            "feat" => SectionType::Features,
            "fix" => SectionType::Fixes,
            "perf" => SectionType::Performance,
            "docs" => SectionType::Documentation,
            "refactor" => SectionType::Refactoring,
            "build" => SectionType::Build,
            "ci" => SectionType::CI,
            "test" => SectionType::Tests,
            _ => SectionType::Other,
        }
    }

    /// Extracts issue references from the commit message.
    ///
    /// Searches for references in the format `#123` in:
    /// - The description
    /// - The body
    /// - Footer values (especially Refs, Closes, Fixes footers)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let message = "feat: add feature #123\n\nCloses: #456\nRefs: #789";
    /// let commit = ConventionalCommit::parse(message)?;
    /// let refs = commit.extract_references();
    /// assert!(refs.contains(&"#123".to_string()));
    /// assert!(refs.contains(&"#456".to_string()));
    /// assert!(refs.contains(&"#789".to_string()));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the regex compilation fails.
    pub fn extract_references(&self) -> ChangelogResult<Vec<String>> {
        let mut refs = Vec::new();

        // Check description
        refs.extend(Self::find_refs_in_text(&self.description)?);

        // Check body
        if let Some(body) = &self.body {
            refs.extend(Self::find_refs_in_text(body)?);
        }

        // Check footers (especially those related to issues)
        for footer in &self.footers {
            let key_lower = footer.key.to_lowercase();
            if key_lower.contains("ref")
                || key_lower.contains("close")
                || key_lower.contains("fix")
                || key_lower.contains("resolve")
            {
                refs.extend(Self::find_refs_in_text(&footer.value)?);
            }
        }

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        Ok(refs.into_iter().filter(|r| seen.insert(r.clone())).collect())
    }

    /// Parses the subject line into components.
    ///
    /// Extracts: type, optional scope, breaking indicator, description.
    ///
    /// # Format
    ///
    /// ```text
    /// <type>(<scope>)!: <description>
    /// ```
    ///
    /// All parts except type and description are optional.
    fn parse_subject(
        line: &str,
        full_message: &str,
    ) -> ChangelogResult<(String, Option<String>, bool, String)> {
        // Regex: ^(\w+)(\(([^)]+)\))?(!)?:\s*(.+)$
        // Groups: 1=type, 3=scope, 4=breaking(!), 5=description
        let re = Regex::new(r"^(\w+)(\(([^)]+)\))?(!)?:\s*(.+)$").map_err(|e| {
            ChangelogError::ConventionalCommitParseError {
                commit: full_message.to_string(),
                reason: format!("Regex compilation failed: {}", e),
            }
        })?;

        let caps = re.captures(line).ok_or_else(|| {
            ChangelogError::ConventionalCommitParseError {
                commit: full_message.to_string(),
                reason: format!(
                    "Subject line '{}' does not match conventional format: <type>[scope]: <description>",
                    line
                ),
            }
        })?;

        let commit_type = caps.get(1).map(|m| m.as_str().to_string()).ok_or_else(|| {
            ChangelogError::ConventionalCommitParseError {
                commit: full_message.to_string(),
                reason: "Failed to extract commit type".to_string(),
            }
        })?;

        let scope = caps.get(3).map(|m| m.as_str().to_string());
        let breaking = caps.get(4).is_some();
        let description = caps.get(5).map(|m| m.as_str().trim().to_string()).ok_or_else(|| {
            ChangelogError::ConventionalCommitParseError {
                commit: full_message.to_string(),
                reason: "Failed to extract description".to_string(),
            }
        })?;

        Ok((commit_type, scope, breaking, description))
    }

    /// Parses body and footers from commit message lines.
    ///
    /// The body is all content before the first footer line.
    /// Footers are key-value pairs in the format `Key: value` or `Key #value`.
    fn parse_body_and_footers(lines: &[&str]) -> (Option<String>, Vec<CommitFooter>) {
        // Skip initial empty lines
        let lines: Vec<&str> = lines.iter().skip_while(|l| l.trim().is_empty()).copied().collect();

        if lines.is_empty() {
            return (None, vec![]);
        }

        // Find where footers start (first line matching footer format)
        let footer_start =
            lines.iter().position(|l| Self::is_footer_line(l)).unwrap_or(lines.len());

        let body = if footer_start > 0 {
            let body_text = lines[..footer_start].join("\n").trim().to_string();
            if body_text.is_empty() {
                None
            } else {
                Some(body_text)
            }
        } else {
            None
        };

        let footers = if footer_start < lines.len() {
            Self::parse_footers(&lines[footer_start..])
        } else {
            vec![]
        };

        (body, footers)
    }

    /// Checks if a line is a footer line.
    ///
    /// A footer line contains a colon and follows the format: `Key: value`
    ///
    /// Special handling for "BREAKING CHANGE" which is allowed to have a space
    /// per the Conventional Commits specification.
    fn is_footer_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Must contain colon and have content before it
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();

            if key.is_empty() {
                return false;
            }

            // Special case: "BREAKING CHANGE" and "BREAKING-CHANGE" are valid footer keys
            if key == "BREAKING CHANGE" || key == "BREAKING-CHANGE" {
                return true;
            }

            // General case: no spaces allowed, must have letters, can have hyphens/underscores
            !key.contains(' ') && key.chars().any(|c| c.is_alphabetic())
        } else {
            false
        }
    }

    /// Parses footer lines into `CommitFooter` structs.
    ///
    /// Footers can span multiple lines. Continuation lines are joined to the
    /// previous footer's value.
    fn parse_footers(lines: &[&str]) -> Vec<CommitFooter> {
        let mut footers = Vec::new();
        let mut current: Option<CommitFooter> = None;

        for line in lines {
            let trimmed = line.trim();

            // Skip completely empty lines when no current footer
            if trimmed.is_empty() && current.is_none() {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();

                // Check if this looks like a new footer
                let is_valid_footer_key = if key == "BREAKING CHANGE" || key == "BREAKING-CHANGE" {
                    // Special case for breaking change footers
                    true
                } else {
                    // General case: not empty, no spaces in key, has letters
                    !key.is_empty() && !key.contains(' ') && key.chars().any(|c| c.is_alphabetic())
                };

                if is_valid_footer_key {
                    // Save previous footer if exists
                    if let Some(footer) = current.take() {
                        footers.push(footer);
                    }

                    // Start new footer
                    let value = trimmed[colon_pos + 1..].trim().to_string();
                    current = Some(CommitFooter { key: key.to_string(), value });
                    continue;
                }
            }

            // This is a continuation line or non-footer content
            if let Some(ref mut footer) = current
                && !trimmed.is_empty() {
                    if !footer.value.is_empty() {
                        footer.value.push(' ');
                    }
                    footer.value.push_str(trimmed);
                }
        }

        // Don't forget the last footer
        if let Some(footer) = current {
            footers.push(footer);
        }

        footers
    }

    /// Finds issue references in text (e.g., #123).
    ///
    /// Searches for the pattern `#<digits>` and returns all matches.
    fn find_refs_in_text(text: &str) -> ChangelogResult<Vec<String>> {
        let re =
            Regex::new(r"#(\d+)").map_err(|e| ChangelogError::ConventionalCommitParseError {
                commit: text.to_string(),
                reason: format!("Failed to compile reference regex: {}", e),
            })?;

        Ok(re
            .captures_iter(text)
            .filter_map(|cap| cap.get(0).map(|m| m.as_str().to_string()))
            .collect())
    }
}

/// A commit footer containing a key-value pair.
///
/// Footers provide additional metadata about a commit, such as:
/// - Issue references (`Refs: #123`)
/// - Breaking change descriptions (`BREAKING CHANGE: removed API`)
/// - Authorship information (`Co-authored-by: Name <email>`)
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::conventional::CommitFooter;
///
/// let footer = CommitFooter {
///     key: "Refs".to_string(),
///     value: "#123, #456".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitFooter {
    /// Footer key (e.g., "Refs", "BREAKING CHANGE")
    pub(crate) key: String,

    /// Footer value
    pub(crate) value: String,
}

impl CommitFooter {
    /// Returns the footer key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the footer value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Categorizes commits into changelog sections.
///
/// Each section type corresponds to a standard category in changelog formats.
/// Breaking changes are always given their own section regardless of commit type.
///
/// # Configuration
///
/// Section titles can be customized via [`crate::config::ConventionalConfig`].
/// This enum provides default titles, but the configuration allows overriding them.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::conventional::{ConventionalCommit, SectionType};
///
/// let commit = ConventionalCommit::parse("feat: add feature")?;
/// assert_eq!(commit.section_type(), SectionType::Features);
///
/// let commit = ConventionalCommit::parse("fix: resolve bug")?;
/// assert_eq!(commit.section_type(), SectionType::Fixes);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SectionType {
    /// Breaking changes (highest priority).
    ///
    /// These are changes that break backward compatibility and require
    /// major version bumps.
    Breaking,

    /// New features.
    ///
    /// Corresponds to `feat` commits and typically trigger minor version bumps.
    Features,

    /// Bug fixes.
    ///
    /// Corresponds to `fix` commits and typically trigger patch version bumps.
    Fixes,

    /// Performance improvements.
    ///
    /// Corresponds to `perf` commits.
    Performance,

    /// Deprecation notices.
    ///
    /// Indicates features that are deprecated and will be removed in future versions.
    Deprecations,

    /// Documentation changes.
    ///
    /// Corresponds to `docs` commits.
    Documentation,

    /// Code refactoring.
    ///
    /// Corresponds to `refactor` commits that restructure code without changing behavior.
    Refactoring,

    /// Build system changes.
    ///
    /// Corresponds to `build` commits affecting the build system or dependencies.
    Build,

    /// CI/CD changes.
    ///
    /// Corresponds to `ci` commits affecting continuous integration or deployment.
    CI,

    /// Test changes.
    ///
    /// Corresponds to `test` commits adding or modifying tests.
    Tests,

    /// Other changes.
    ///
    /// Catch-all for commits that don't fit other categories (e.g., `chore`).
    Other,
}

impl SectionType {
    /// Returns the human-readable title for this section.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::conventional::SectionType;
    ///
    /// assert_eq!(SectionType::Features.title(), "Features");
    /// assert_eq!(SectionType::Fixes.title(), "Bug Fixes");
    /// assert_eq!(SectionType::Breaking.title(), "Breaking Changes");
    /// ```
    pub fn title(&self) -> &str {
        match self {
            SectionType::Breaking => "Breaking Changes",
            SectionType::Features => "Features",
            SectionType::Fixes => "Bug Fixes",
            SectionType::Performance => "Performance Improvements",
            SectionType::Deprecations => "Deprecations",
            SectionType::Documentation => "Documentation",
            SectionType::Refactoring => "Code Refactoring",
            SectionType::Build => "Build System",
            SectionType::CI => "Continuous Integration",
            SectionType::Tests => "Tests",
            SectionType::Other => "Other Changes",
        }
    }

    /// Returns the sort priority for this section type.
    ///
    /// Lower numbers appear first in the changelog. Breaking changes have
    /// the highest priority (0) and always appear first.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::conventional::SectionType;
    ///
    /// assert_eq!(SectionType::Breaking.priority(), 0);
    /// assert_eq!(SectionType::Features.priority(), 1);
    /// ```
    pub fn priority(&self) -> u8 {
        match self {
            SectionType::Breaking => 0,
            SectionType::Features => 1,
            SectionType::Fixes => 2,
            SectionType::Performance => 3,
            SectionType::Deprecations => 4,
            SectionType::Documentation => 5,
            SectionType::Refactoring => 6,
            SectionType::Build => 7,
            SectionType::CI => 8,
            SectionType::Tests => 9,
            SectionType::Other => 10,
        }
    }
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title())
    }
}

impl PartialOrd for SectionType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SectionType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}
