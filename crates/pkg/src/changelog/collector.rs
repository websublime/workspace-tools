//! Changelog data collection module for gathering commits and metadata.
//!
//! **What**: Provides functionality to collect commit data from Git repositories,
//! parse commit messages, and organize them into changelog sections.
//!
//! **How**: This module integrates with Git to retrieve commits between versions,
//! parses them using conventional commit format (with fallback to plain messages),
//! groups them by section type, and collects all relevant metadata.
//!
//! **Why**: To automate the process of gathering and organizing commit information
//! for changelog generation, ensuring consistent and comprehensive release documentation.

use crate::changelog::{ChangelogEntry, ChangelogSection, ConventionalCommit, SectionType};
use crate::config::ChangelogConfig;
use crate::error::{ChangelogError, ChangelogResult};
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use std::collections::HashMap;
use sublime_git_tools::{Repo, RepoCommit};

/// Collector for gathering changelog data from Git commits.
///
/// The `ChangelogCollector` is responsible for retrieving commits from a Git
/// repository, parsing them into changelog entries, and organizing them by
/// section type (Features, Fixes, Breaking Changes, etc.).
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::changelog::ChangelogCollector;
/// use sublime_pkg_tools::config::ChangelogConfig;
/// use sublime_git_tools::Repo;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let repo = Repo::open(".")?;
/// let config = ChangelogConfig::default();
/// let collector = ChangelogCollector::new(&repo, &config);
///
/// // Collect commits between two versions
/// let sections = collector.collect_between_versions("v1.0.0", "v2.0.0", None).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ChangelogCollector<'a> {
    /// Reference to the Git repository.
    repo: &'a Repo,

    /// Changelog configuration.
    config: &'a ChangelogConfig,

    /// Cached exclude patterns regex.
    exclude_patterns: Vec<Regex>,
}

impl<'a> ChangelogCollector<'a> {
    /// Creates a new `ChangelogCollector`.
    ///
    /// # Arguments
    ///
    /// * `repo` - Reference to the Git repository
    /// * `config` - Reference to changelog configuration
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogCollector;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use sublime_git_tools::Repo;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = ChangelogConfig::default();
    /// let collector = ChangelogCollector::new(&repo, &config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo: &'a Repo, config: &'a ChangelogConfig) -> Self {
        // Compile exclude patterns
        let exclude_patterns =
            config.exclude.patterns.iter().filter_map(|pattern| Regex::new(pattern).ok()).collect();

        Self { repo, config, exclude_patterns }
    }

    /// Collects commits between two Git references and organizes them into sections.
    ///
    /// This is the main entry point for data collection. It retrieves commits
    /// between the specified Git references, parses each commit, groups them
    /// by section type, and sorts them appropriately.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting Git reference (exclusive)
    /// * `to_ref` - Ending Git reference (inclusive)
    /// * `relative_path` - Optional path filter for commits (for monorepo packages)
    ///
    /// # Returns
    ///
    /// A vector of `ChangelogSection` instances, sorted by priority.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Git operations fail
    /// - Commit retrieval fails
    /// - References are invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::changelog::ChangelogCollector;
    /// use sublime_pkg_tools::config::ChangelogConfig;
    /// use sublime_git_tools::Repo;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = ChangelogConfig::default();
    /// let collector = ChangelogCollector::new(&repo, &config);
    ///
    /// // Collect for entire repository
    /// let sections = collector.collect_between_versions("v1.0.0", "v2.0.0", None).await?;
    ///
    /// // Collect for specific package in monorepo
    /// let sections = collector.collect_between_versions(
    ///     "pkg-a@1.0.0",
    ///     "pkg-a@2.0.0",
    ///     Some("packages/pkg-a")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collect_between_versions(
        &self,
        from_ref: &str,
        to_ref: &str,
        relative_path: Option<&str>,
    ) -> ChangelogResult<Vec<ChangelogSection>> {
        // Get commits from Git
        let commits = self.get_commits_between(from_ref, to_ref, relative_path)?;

        // Parse and group commits
        self.process_commits(commits)
    }

    /// Retrieves commits between two Git references.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting Git reference
    /// * `to_ref` - Ending Git reference
    /// * `relative_path` - Optional path filter
    ///
    /// # Returns
    ///
    /// A vector of `RepoCommit` instances.
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail.
    fn get_commits_between(
        &self,
        from_ref: &str,
        to_ref: &str,
        relative_path: Option<&str>,
    ) -> ChangelogResult<Vec<RepoCommit>> {
        let path_option = relative_path.map(String::from);

        self.repo.get_commits_between(from_ref, to_ref, &path_option).map_err(|e| {
            ChangelogError::GitError {
                operation: format!("get commits between {} and {}", from_ref, to_ref),
                reason: e.as_ref().to_string(),
            }
        })
    }

    /// Processes a list of commits into organized changelog sections.
    ///
    /// This method:
    /// 1. Filters commits based on exclusion rules
    /// 2. Parses each commit into a changelog entry
    /// 3. Groups entries by section type
    /// 4. Sorts entries within sections
    /// 5. Sorts sections by priority
    ///
    /// # Arguments
    ///
    /// * `commits` - Vector of Git commits to process
    ///
    /// # Returns
    ///
    /// A vector of `ChangelogSection` instances.
    fn process_commits(&self, commits: Vec<RepoCommit>) -> ChangelogResult<Vec<ChangelogSection>> {
        // Filter commits
        let filtered_commits: Vec<&RepoCommit> =
            commits.iter().filter(|commit| self.should_include_commit(commit)).collect();

        // Parse commits into entries
        let mut entries: Vec<ChangelogEntry> =
            filtered_commits.iter().map(|commit| self.parse_commit(commit)).collect();

        // Sort entries by date (newest first)
        entries.sort_by(|a, b| b.date.cmp(&a.date));

        // Group entries by section
        let sections = self.group_entries_by_section(entries);

        Ok(sections)
    }

    /// Checks if a commit should be included in the changelog.
    ///
    /// Applies exclusion rules based on:
    /// - Commit message patterns
    /// - Author names
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit to check
    ///
    /// # Returns
    ///
    /// `true` if the commit should be included, `false` otherwise.
    pub(crate) fn should_include_commit(&self, commit: &RepoCommit) -> bool {
        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if pattern.is_match(&commit.message) {
                return false;
            }
        }

        // Check exclude authors
        if self.config.exclude.authors.contains(&commit.author_name) {
            return false;
        }

        true
    }

    /// Parses a Git commit into a changelog entry.
    ///
    /// This method attempts to parse the commit using conventional commit format.
    /// If that fails, it falls back to treating the commit as a plain message.
    ///
    /// # Arguments
    ///
    /// * `commit` - The Git commit to parse
    ///
    /// # Returns
    ///
    /// A `ChangelogEntry` representing the commit.
    pub(crate) fn parse_commit(&self, commit: &RepoCommit) -> ChangelogEntry {
        let short_hash =
            if commit.hash.len() >= 7 { commit.hash[..7].to_string() } else { commit.hash.clone() };

        // Parse date
        let date = self.parse_commit_date(&commit.author_date);

        // Try parsing as conventional commit
        if self.config.conventional.enabled {
            if let Ok(conventional) = ConventionalCommit::parse(&commit.message) {
                return ChangelogEntry {
                    description: conventional.description().to_string(),
                    commit_hash: commit.hash.clone(),
                    short_hash,
                    commit_type: Some(conventional.commit_type().to_string()),
                    scope: conventional.scope().map(String::from),
                    breaking: conventional.is_breaking(),
                    author: commit.author_name.clone(),
                    references: conventional.extract_references().unwrap_or_default(),
                    date,
                };
            }
        }

        // Fallback to plain message
        let description = self.extract_first_line(&commit.message);
        let references = self.extract_references_from_text(&commit.message);

        ChangelogEntry {
            description,
            commit_hash: commit.hash.clone(),
            short_hash,
            commit_type: None,
            scope: None,
            breaking: false,
            author: commit.author_name.clone(),
            references,
            date,
        }
    }

    /// Parses a commit date string into a DateTime.
    ///
    /// # Arguments
    ///
    /// * `date_str` - Date string from Git commit
    ///
    /// # Returns
    ///
    /// A `DateTime<Utc>` representing the commit date.
    fn parse_commit_date(&self, date_str: &str) -> DateTime<Utc> {
        // Try parsing as RFC 2822 format (Git default)
        if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
            return dt.with_timezone(&Utc);
        }

        // Try parsing as RFC 3339 format
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return dt.with_timezone(&Utc);
        }

        // Try parsing as Unix timestamp
        if let Ok(timestamp) = date_str.parse::<i64>() {
            if let Some(dt) = Utc.timestamp_opt(timestamp, 0).single() {
                return dt;
            }
        }

        // Fallback to current time if parsing fails
        Utc::now()
    }

    /// Extracts the first line from a commit message.
    ///
    /// # Arguments
    ///
    /// * `message` - The full commit message
    ///
    /// # Returns
    ///
    /// The first line of the message, trimmed.
    pub(crate) fn extract_first_line(&self, message: &str) -> String {
        message.lines().next().unwrap_or(message).trim().to_string()
    }

    /// Extracts issue/PR references from text.
    ///
    /// Searches for patterns like `#123`, `fixes #456`, `closes #789`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to search
    ///
    /// # Returns
    ///
    /// A vector of reference strings (e.g., `["#123", "#456"]`).
    pub(crate) fn extract_references_from_text(&self, text: &str) -> Vec<String> {
        // Use a static regex pattern for issue references
        // Pattern matches: #123, fixes #456, closes #789, resolves #101, etc.
        let Ok(re) = Regex::new(r"(?i)(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)?[:\s]*#(\d+)")
        else {
            // If regex compilation fails (which shouldn't happen with a valid pattern),
            // return empty vec rather than panicking
            return Vec::new();
        };

        let mut refs = Vec::new();
        for cap in re.captures_iter(text) {
            if let Some(num) = cap.get(1) {
                refs.push(format!("#{}", num.as_str()));
            }
        }

        refs.sort();
        refs.dedup();
        refs
    }

    /// Groups changelog entries by section type.
    ///
    /// This method:
    /// 1. Determines the section type for each entry
    /// 2. Groups entries into sections
    /// 3. Sorts sections by priority (Breaking Changes first, etc.)
    ///
    /// # Arguments
    ///
    /// * `entries` - Vector of changelog entries
    ///
    /// # Returns
    ///
    /// A vector of `ChangelogSection` instances, sorted by priority.
    pub(crate) fn group_entries_by_section(
        &self,
        entries: Vec<ChangelogEntry>,
    ) -> Vec<ChangelogSection> {
        // Group entries by section type
        let mut section_map: HashMap<SectionType, Vec<ChangelogEntry>> = HashMap::new();

        for entry in entries {
            let section_type = self.determine_section_type(&entry);
            section_map.entry(section_type).or_default().push(entry);
        }

        // Convert to sorted vector of sections
        let mut sections: Vec<ChangelogSection> = section_map
            .into_iter()
            .map(|(section_type, entries)| {
                let mut section = ChangelogSection::new(section_type);
                for entry in entries {
                    section.add_entry(entry);
                }
                section
            })
            .collect();

        // Sort sections by priority (Breaking Changes first, etc.)
        sections.sort_by(|a, b| a.section_type.cmp(&b.section_type));

        sections
    }

    /// Determines the section type for a changelog entry.
    ///
    /// Uses the commit type (if available) or defaults to "Other" for
    /// non-conventional commits.
    ///
    /// # Arguments
    ///
    /// * `entry` - The changelog entry
    ///
    /// # Returns
    ///
    /// The appropriate `SectionType` for the entry.
    pub(crate) fn determine_section_type(&self, entry: &ChangelogEntry) -> SectionType {
        // Breaking changes always go to Breaking section
        if entry.breaking {
            return SectionType::Breaking;
        }

        // Use conventional commit type if available
        if let Some(ref commit_type) = entry.commit_type {
            return match commit_type.as_str() {
                "feat" => SectionType::Features,
                "fix" => SectionType::Fixes,
                "perf" => SectionType::Performance,
                "deprecate" => SectionType::Deprecations,
                "docs" => SectionType::Documentation,
                "refactor" => SectionType::Refactoring,
                "build" => SectionType::Build,
                "ci" => SectionType::CI,
                "test" => SectionType::Tests,
                _ => SectionType::Other,
            };
        }

        // Default to Other for non-conventional commits
        SectionType::Other
    }
}
