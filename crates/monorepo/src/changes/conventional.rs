//! Conventional Commits Parser
//!
//! ## What
//! This module provides parsing and analysis capabilities for conventional commits format.
//! It extracts semantic information from commit messages to suggest appropriate version bumps.
//!
//! ## How
//! Parses commit messages according to the Conventional Commits specification (conventionalcommits.org)
//! and maps commit types to version bump recommendations based on semantic versioning rules.
//!
//! ## Why
//! Provides intelligent fallback when explicit changesets are not available, using standardized
//! commit message format to determine appropriate version increments automatically.

use crate::config::VersionBumpType;
use sublime_git_tools::RepoCommit;

/// Represents the source of a version bump decision
///
/// Defines the priority hierarchy for determining version bumps:
/// 1. Explicit changesets have highest priority
/// 2. Conventional commits provide intelligent fallback
/// 3. Conservative patch fallback when no other information available
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changes::ChangeDecisionSource;
/// use sublime_monorepo_tools::config::VersionBumpType;
///
/// // Changeset takes priority
/// let decision = ChangeDecisionSource::Changeset(VersionBumpType::Major);
/// assert!(decision.is_authoritative());
///
/// // Conventional commit as fallback
/// let decision = ChangeDecisionSource::ConventionalCommit(VersionBumpType::Minor);
/// assert!(!decision.is_authoritative());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeDecisionSource {
    /// Explicit changeset decision (highest priority)
    /// Developer has explicitly declared the intended version bump
    Changeset(VersionBumpType),

    /// Decision derived from conventional commit analysis (fallback)
    /// Parsed from commit message following conventional commits specification
    ConventionalCommit(VersionBumpType),

    /// Conservative fallback when no other information available
    /// Always suggests patch to avoid accidental breaking changes
    Fallback(VersionBumpType),
}

impl ChangeDecisionSource {
    /// Returns the version bump type regardless of source
    ///
    /// # Returns
    ///
    /// The version bump type contained in this decision source
    #[must_use]
    pub fn version_bump(&self) -> VersionBumpType {
        match self {
            Self::Changeset(bump) | Self::ConventionalCommit(bump) | Self::Fallback(bump) => *bump,
        }
    }

    /// Returns true if this decision comes from an authoritative source
    ///
    /// Changesets are considered authoritative as they represent explicit developer decisions.
    /// Conventional commits and fallbacks are suggestions that can be overridden.
    ///
    /// # Returns
    ///
    /// True if the decision source is authoritative (changeset), false otherwise
    #[must_use]
    pub fn is_authoritative(&self) -> bool {
        matches!(self, Self::Changeset(_))
    }

    /// Returns a description of the decision source for logging and debugging
    ///
    /// # Returns
    ///
    /// A human-readable string describing the source of this decision
    #[must_use]
    pub fn source_description(&self) -> &'static str {
        match self {
            Self::Changeset(_) => "explicit changeset",
            Self::ConventionalCommit(_) => "conventional commit analysis",
            Self::Fallback(_) => "conservative fallback",
        }
    }
}

/// Parser for conventional commits according to conventionalcommits.org
///
/// Parses commit messages in the format:
/// ```text
/// <type>[optional scope]: <description>
///
/// [optional body]
///
/// [optional footer(s)]
/// ```
///
/// Where breaking changes are indicated by:
/// - `!` after the type/scope (e.g., `feat!:` or `feat(api)!:`)
/// - `BREAKING CHANGE:` footer
///
/// # Examples
///
/// ```rust
/// use sublime_monorepo_tools::changes::ConventionalCommitParser;
/// use sublime_monorepo_tools::config::VersionBumpType;
///
/// let parser = ConventionalCommitParser::new();
///
/// // Breaking change triggers major bump
/// let bump = parser.parse_commit_message("feat!: add new API endpoint");
/// assert_eq!(bump, Some(VersionBumpType::Major));
///
/// // New feature triggers minor bump
/// let bump = parser.parse_commit_message("feat: add user authentication");
/// assert_eq!(bump, Some(VersionBumpType::Minor));
///
/// // Bug fix triggers patch bump
/// let bump = parser.parse_commit_message("fix: resolve login issue");
/// assert_eq!(bump, Some(VersionBumpType::Patch));
/// ```
#[derive(Debug, Clone)]
pub struct ConventionalCommitParser {
    /// Configuration for mapping commit types to version bumps
    type_mappings: std::collections::HashMap<String, VersionBumpType>,
}

impl ConventionalCommitParser {
    /// Creates a new conventional commit parser with default type mappings
    ///
    /// Default mappings follow semantic versioning conventions:
    /// - `feat`: Minor version bump (new functionality)
    /// - `fix`: Patch version bump (bug fixes)
    /// - `perf`: Patch version bump (performance improvements)
    /// - `refactor`: Patch version bump (code refactoring)
    /// - `style`: Patch version bump (formatting, style changes)
    /// - `test`: Patch version bump (adding tests)
    /// - `docs`: Patch version bump (documentation changes)
    /// - `chore`: Patch version bump (maintenance tasks)
    /// - `ci`: Patch version bump (CI/CD changes)
    /// - `build`: Patch version bump (build system changes)
    /// - `revert`: Patch version bump (reverting changes)
    ///
    /// Any type with `!` suffix or `BREAKING CHANGE:` footer triggers major bump.
    ///
    /// # Returns
    ///
    /// A new parser instance with default conventional commit type mappings
    #[must_use]
    pub fn new() -> Self {
        let mut type_mappings = std::collections::HashMap::new();

        // Features introduce new functionality (minor bump)
        type_mappings.insert("feat".to_string(), VersionBumpType::Minor);

        // All other standard types trigger patch bumps
        let patch_types =
            ["fix", "perf", "refactor", "style", "test", "docs", "chore", "ci", "build", "revert"];

        for commit_type in &patch_types {
            type_mappings.insert((*commit_type).to_string(), VersionBumpType::Patch);
        }

        Self { type_mappings }
    }

    /// Creates a parser with custom type mappings
    ///
    /// Allows customization of how commit types map to version bumps.
    /// Useful for organizations with specific conventions or additional commit types.
    ///
    /// # Arguments
    ///
    /// * `type_mappings` - HashMap mapping commit types to version bump types
    ///
    /// # Returns
    ///
    /// A new parser instance with the provided type mappings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use sublime_monorepo_tools::changes::ConventionalCommitParser;
    /// use sublime_monorepo_tools::config::VersionBumpType;
    ///
    /// let mut mappings = HashMap::new();
    /// mappings.insert("hotfix".to_string(), VersionBumpType::Patch);
    /// mappings.insert("breaking".to_string(), VersionBumpType::Major);
    ///
    /// let parser = ConventionalCommitParser::with_mappings(mappings);
    /// ```
    #[must_use]
    pub fn with_mappings(
        type_mappings: std::collections::HashMap<String, VersionBumpType>,
    ) -> Self {
        Self { type_mappings }
    }

    /// Parses a single commit message and returns suggested version bump
    ///
    /// Analyzes the commit message format and content to determine appropriate
    /// version bump according to conventional commits specification.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message to parse
    ///
    /// # Returns
    ///
    /// `Some(VersionBumpType)` if the message follows conventional commits format,
    /// `None` if the message doesn't match the expected format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::changes::ConventionalCommitParser;
    /// use sublime_monorepo_tools::config::VersionBumpType;
    ///
    /// let parser = ConventionalCommitParser::new();
    ///
    /// assert_eq!(parser.parse_commit_message("feat!: breaking API change"), Some(VersionBumpType::Major));
    /// assert_eq!(parser.parse_commit_message("feat: new feature"), Some(VersionBumpType::Minor));
    /// assert_eq!(parser.parse_commit_message("fix: bug fix"), Some(VersionBumpType::Patch));
    /// assert_eq!(parser.parse_commit_message("invalid message"), None);
    /// ```
    pub fn parse_commit_message(&self, message: &str) -> Option<VersionBumpType> {
        let first_line = message.lines().next()?.trim();

        // Check for breaking change footer
        let has_breaking_footer = message.contains("BREAKING CHANGE:");

        // Parse the conventional commit format: type[scope]: description
        let commit_info = Self::parse_commit_header(first_line)?;

        // Breaking changes always trigger major version bump
        if commit_info.is_breaking || has_breaking_footer {
            return Some(VersionBumpType::Major);
        }

        // Look up version bump for this commit type
        self.type_mappings.get(&commit_info.commit_type).copied()
    }

    /// Analyzes multiple commits and returns the highest version bump suggested
    ///
    /// When multiple commits are analyzed, the most significant version bump wins:
    /// Major > Minor > Patch
    ///
    /// # Arguments
    ///
    /// * `commits` - Iterator of commits to analyze
    ///
    /// # Returns
    ///
    /// `Some(VersionBumpType)` representing the highest version bump suggested,
    /// `None` if no commits follow conventional commits format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_monorepo_tools::changes::ConventionalCommitParser;
    /// use sublime_monorepo_tools::config::VersionBumpType;
    /// use sublime_git_tools::RepoCommit;
    ///
    /// let parser = ConventionalCommitParser::new();
    /// let commits = vec![
    ///     RepoCommit { message: "fix: bug fix".to_string(), /* other fields */ },
    ///     RepoCommit { message: "feat: new feature".to_string(), /* other fields */ },
    /// ];
    ///
    /// let bump = parser.analyze_commits(&commits);
    /// assert_eq!(bump, Some(VersionBumpType::Minor)); // Highest bump wins
    /// ```
    pub fn analyze_commits<I>(&self, commits: I) -> Option<VersionBumpType>
    where
        I: IntoIterator<Item = RepoCommit>,
    {
        let mut highest_bump: Option<VersionBumpType> = None;

        for commit in commits {
            if let Some(bump) = self.parse_commit_message(&commit.message) {
                highest_bump = Some(match (highest_bump, bump) {
                    (None, new_bump) => new_bump,
                    (Some(VersionBumpType::Major), _) | (Some(_), VersionBumpType::Major) => {
                        VersionBumpType::Major
                    }
                    (Some(VersionBumpType::Minor), VersionBumpType::Patch)
                    | (Some(VersionBumpType::Patch), VersionBumpType::Minor) => {
                        VersionBumpType::Minor
                    }
                    (Some(existing), _) => existing,
                });

                // Early exit if we find a major bump (highest possible)
                if matches!(highest_bump, Some(VersionBumpType::Major)) {
                    break;
                }
            }
        }

        highest_bump
    }

    /// Parses the header line of a conventional commit
    ///
    /// Internal method to extract commit type, scope, and breaking change indicator
    /// from the first line of a commit message.
    ///
    /// # Arguments
    ///
    /// * `header` - The first line of the commit message
    ///
    /// # Returns
    ///
    /// `Some(CommitInfo)` if the header follows conventional format, `None` otherwise
    fn parse_commit_header(header: &str) -> Option<CommitInfo> {
        // Split on the first colon to separate type/scope from description
        let mut parts = header.splitn(2, ':');
        let type_scope = parts.next()?.trim();
        let _description = parts.next()?.trim();

        // Check for breaking change indicator (!)
        let (type_scope, is_breaking) = if let Some(stripped) = type_scope.strip_suffix('!') {
            (stripped, true)
        } else {
            (type_scope, false)
        };

        // Extract type and optional scope
        let commit_type = if type_scope.contains('(') && type_scope.ends_with(')') {
            // Format: type(scope)
            let open_paren = type_scope.find('(')?;
            type_scope[..open_paren].trim().to_string()
        } else {
            // Format: type
            type_scope.trim().to_string()
        };

        Some(CommitInfo { commit_type, is_breaking })
    }
}

impl Default for ConventionalCommitParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Information extracted from a conventional commit header
///
/// Internal structure used by the parser to hold parsed commit information.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CommitInfo {
    /// The commit type (feat, fix, docs, etc.)
    commit_type: String,
    /// Whether this commit introduces breaking changes
    is_breaking: bool,
}
