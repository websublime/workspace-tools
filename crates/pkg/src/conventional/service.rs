//! Conventional commit service for Git integration.
//!
//! This module provides a service layer that integrates conventional commit parsing
//! with Git operations, enabling retrieval and analysis of commit history for
//! version bump calculation and changeset creation.
//!
//! # What
//!
//! Provides the `ConventionalCommitService` which acts as a bridge between:
//! - `sublime_git_tools::Repo` for Git operations
//! - `ConventionalCommitParser` for commit message parsing
//! - Configuration system for custom commit type mappings
//!
//! # How
//!
//! Uses the Git tools to retrieve commit history, then parses each commit
//! message according to conventional commit specification. Supports batch
//! processing of commits and provides aggregated analysis for version
//! bump determination.
//!
//! # Why
//!
//! Enables automatic version bump calculation based on commit history,
//! supporting the changeset workflow by analyzing commits between
//! branches or since specific points in history.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sublime_git_tools::{Repo, RepoCommit};

use crate::{
    config::PackageToolsConfig,
    conventional::{CommitType, ConventionalCommit, ConventionalCommitParser},
    error::{ConventionalCommitError, PackageResult},
    VersionBump,
};

/// Service for integrating conventional commit parsing with Git operations.
///
/// Provides methods to retrieve and parse commits from Git history,
/// calculate version bumps, and analyze commit patterns for changeset creation.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::conventional::ConventionalCommitService;
/// use sublime_git_tools::Repo;
/// use sublime_pkg_tools::config::PackageToolsConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let repo = Repo::open(".")?;
/// let config = PackageToolsConfig::default();
/// let service = ConventionalCommitService::new(repo, config)?;
///
/// // Get commits since last tag
/// let commits = service.get_commits_since_last_tag().await?;
/// println!("Found {} conventional commits", commits.len());
///
/// // Calculate version bump from commits
/// let bump = service.calculate_version_bump(&commits);
/// println!("Suggested bump: {:?}", bump);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ConventionalCommitService {
    /// Git repository for commit operations
    pub(crate) repo: Repo,
    /// Conventional commit parser
    pub(crate) parser: ConventionalCommitParser,
    /// Configuration for commit type mappings
    pub(crate) config: PackageToolsConfig,
}

impl ConventionalCommitService {
    /// Creates a new conventional commit service.
    ///
    /// # Arguments
    ///
    /// * `repo` - Git repository instance
    /// * `config` - Package tools configuration
    ///
    /// # Errors
    ///
    /// Returns error if parser initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// use sublime_git_tools::Repo;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo: Repo, config: PackageToolsConfig) -> PackageResult<Self> {
        let parser = ConventionalCommitParser::with_config(config.conventional.clone())?;
        Ok(Self { repo, parser, config })
    }

    /// Gets all conventional commits since the last Git tag.
    ///
    /// Retrieves commits from the last tag (or repository start if no tags exist)
    /// and parses them according to conventional commit specification.
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail or commit parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// let commits = service.get_commits_since_last_tag().await?;
    /// for commit in commits {
    ///     println!("{}: {}", commit.commit_type, commit.description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_commits_since_last_tag(&self) -> PackageResult<Vec<ConventionalCommit>> {
        let last_tag = self.repo.get_last_tag().ok();
        let since_ref = last_tag.clone();

        self.get_commits_since(since_ref.as_ref()).await
    }

    /// Gets all conventional commits since a specific Git reference.
    ///
    /// Retrieves commits from the specified reference to HEAD and parses them
    /// according to conventional commit specification.
    ///
    /// # Arguments
    ///
    /// * `since_ref` - Git reference to start from (commit hash, branch, tag)
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail or commit parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// let commits = service.get_commits_since("v1.2.0").await?;
    /// println!("Found {} commits since v1.2.0", commits.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_commits_since(
        &self,
        since_ref: Option<&String>,
    ) -> PackageResult<Vec<ConventionalCommit>> {
        let git_commits = self.repo.get_commits_since(since_ref.cloned(), &None)?;
        self.parse_git_commits(git_commits).await
    }

    /// Gets all conventional commits between two Git references.
    ///
    /// Retrieves commits between the specified references and parses them
    /// according to conventional commit specification.
    ///
    /// # Arguments
    ///
    /// * `from_ref` - Starting Git reference
    /// * `to_ref` - Ending Git reference
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail or commit parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// let commits = service.get_commits_between("v1.0.0", "v1.1.0").await?;
    /// println!("Found {} commits between versions", commits.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_commits_between(
        &self,
        from_ref: &str,
        to_ref: &str,
    ) -> PackageResult<Vec<ConventionalCommit>> {
        let git_commits = self.repo.get_commits_between(from_ref, to_ref, &None)?;
        self.parse_git_commits(git_commits).await
    }

    /// Gets conventional commits for the current branch since it diverged from main.
    ///
    /// Finds the divergence point between the current branch and main, then
    /// retrieves and parses all commits since that point.
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail or commit parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// let commits = service.get_branch_commits().await?;
    /// println!("Found {} commits on current branch", commits.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_branch_commits(&self) -> PackageResult<Vec<ConventionalCommit>> {
        let current_branch = self.repo.get_current_branch()?;
        let main_branch = "main";

        // Get merge base between current branch and main
        let merge_base = self.repo.get_merge_base(&current_branch, main_branch)?;
        self.get_commits_since(Some(&merge_base)).await
    }

    /// Parses a single commit message into a ConventionalCommit.
    ///
    /// This is a convenience method for parsing individual commit messages
    /// when you already have the commit metadata.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message to parse
    /// * `hash` - The commit hash
    /// * `author` - The commit author name
    /// * `date` - The commit date
    ///
    /// # Returns
    ///
    /// A parsed ConventionalCommit
    ///
    /// # Errors
    ///
    /// Returns error if the message doesn't follow conventional commit format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// use sublime_git_tools::Repo;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use chrono::Utc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    ///
    /// let commit = service.parse_commit_message(
    ///     "feat: add new feature",
    ///     "abc123".to_string(),
    ///     "John Doe".to_string(),
    ///     Utc::now(),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_commit_message(
        &self,
        message: &str,
        hash: String,
        author: String,
        date: DateTime<Utc>,
    ) -> PackageResult<ConventionalCommit> {
        self.parser.parse(message, hash, author, date)
    }

    /// Calculates the suggested version bump from a collection of commits.
    ///
    /// Analyzes all commits and returns the highest version bump suggested
    /// by any commit. Breaking changes always result in major bumps.
    ///
    /// # Arguments
    ///
    /// * `commits` - Collection of conventional commits to analyze
    ///
    /// # Returns
    ///
    /// The highest version bump suggested by the commits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::{ConventionalCommitService, ConventionalCommit, CommitType};
    /// use sublime_pkg_tools::{VersionBump, config::PackageToolsConfig};
    /// use sublime_git_tools::Repo;
    /// use chrono::Utc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    ///
    /// let commits = vec![
    ///     ConventionalCommit {
    ///         commit_type: CommitType::Fix,
    ///         scope: None,
    ///         breaking: false,
    ///         description: "fix bug".to_string(),
    ///         body: None,
    ///         footer: None,
    ///         hash: "abc123".to_string(),
    ///         author: "dev".to_string(),
    ///         date: Utc::now(),
    ///     },
    ///     ConventionalCommit {
    ///         commit_type: CommitType::Feat,
    ///         scope: None,
    ///         breaking: false,
    ///         description: "add feature".to_string(),
    ///         body: None,
    ///         footer: None,
    ///         hash: "def456".to_string(),
    ///         author: "dev".to_string(),
    ///         date: Utc::now(),
    ///     },
    /// ];
    ///
    /// let bump = service.calculate_version_bump(&commits);
    /// assert_eq!(bump, VersionBump::Minor); // feat wins over fix
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn calculate_version_bump(&self, commits: &[ConventionalCommit]) -> VersionBump {
        let mut highest_bump = VersionBump::None;

        for commit in commits {
            let bump = self.parser.get_version_bump(&commit.commit_type, commit.breaking);

            // Update highest bump based on priority: Major > Minor > Patch > None
            highest_bump = match (highest_bump, bump) {
                (_, VersionBump::Major) => VersionBump::Major,
                (VersionBump::Major, _) => VersionBump::Major,
                (_, VersionBump::Minor) => VersionBump::Minor,
                (VersionBump::Minor, _) => VersionBump::Minor,
                (_, VersionBump::Patch) => VersionBump::Patch,
                (VersionBump::Patch, _) => VersionBump::Patch,
                (VersionBump::None, VersionBump::None) => VersionBump::None,
            };
        }

        highest_bump
    }

    /// Groups commits by their type for changelog generation.
    ///
    /// Organizes commits into groups based on their type, filtering out
    /// types that shouldn't be included in changelogs.
    ///
    /// # Arguments
    ///
    /// * `commits` - Collection of conventional commits to group
    ///
    /// # Returns
    ///
    /// HashMap mapping commit types to vectors of commits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::{ConventionalCommitService, ConventionalCommit, CommitType};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use chrono::Utc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    ///
    /// let commits = vec![
    ///     ConventionalCommit {
    ///         commit_type: CommitType::Feat,
    ///         scope: None,
    ///         breaking: false,
    ///         description: "add feature".to_string(),
    ///         body: None,
    ///         footer: None,
    ///         hash: "abc123".to_string(),
    ///         author: "dev".to_string(),
    ///         date: Utc::now(),
    ///     },
    /// ];
    ///
    /// let grouped = service.group_commits_by_type(&commits);
    /// assert!(grouped.contains_key(&CommitType::Feat));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn group_commits_by_type<'a>(
        &self,
        commits: &'a [ConventionalCommit],
    ) -> HashMap<CommitType, Vec<&'a ConventionalCommit>> {
        let mut groups: HashMap<CommitType, Vec<&ConventionalCommit>> = HashMap::new();

        for commit in commits {
            if self.parser.should_include_in_changelog(&commit.commit_type) {
                groups.entry(commit.commit_type.clone()).or_default().push(commit);
            }
        }

        groups
    }

    /// Filters commits to only include those that affect specific paths.
    ///
    /// Uses Git to determine which files were changed in each commit and
    /// filters to only include commits that affect the specified paths.
    ///
    /// # Arguments
    ///
    /// * `commits` - Collection of conventional commits to filter
    /// * `paths` - Paths to filter by (package directories, etc.)
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// # let commits = vec![];
    /// let paths = vec!["packages/auth".to_string(), "packages/ui".to_string()];
    /// let filtered = service.filter_commits_by_paths(&commits, &paths).await?;
    /// println!("Found {} commits affecting specified paths", filtered.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn filter_commits_by_paths(
        &self,
        commits: &[ConventionalCommit],
        paths: &[String],
    ) -> PackageResult<Vec<ConventionalCommit>> {
        let mut filtered_commits = Vec::new();

        for commit in commits {
            let changed_files =
                self.repo.get_all_files_changed_since_sha(&format!("{}~1", commit.hash))?;

            // Check if any changed file is in one of the specified paths
            let affects_paths =
                changed_files.iter().any(|file| paths.iter().any(|path| file.starts_with(path)));

            if affects_paths {
                filtered_commits.push(commit.clone());
            }
        }

        Ok(filtered_commits)
    }

    /// Gets the current Git commit information.
    ///
    /// Retrieves information about the current HEAD commit for use in
    /// snapshot version generation and changeset metadata.
    ///
    /// # Errors
    ///
    /// Returns error if Git operations fail.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// # use sublime_git_tools::Repo;
    /// # use sublime_pkg_tools::config::PackageToolsConfig;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let repo = Repo::open(".")?;
    /// # let config = PackageToolsConfig::default();
    /// # let service = ConventionalCommitService::new(repo, config)?;
    /// let commit_info = service.get_current_commit_info().await?;
    /// println!("Current commit: {} by {}", commit_info.hash, commit_info.author_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_commit_info(&self) -> PackageResult<RepoCommit> {
        let current_sha = self.repo.get_current_sha()?;
        let commits = self.repo.get_commits_since(None, &None)?;

        commits.into_iter().next().ok_or_else(|| {
            ConventionalCommitError::ParseFailed {
                commit: current_sha.clone(),
                reason: "Could not retrieve current commit information".to_string(),
            }
            .into()
        })
    }

    /// Parses a collection of Git commits into conventional commits.
    ///
    /// Attempts to parse each Git commit according to conventional commit
    /// specification. Non-conforming commits are logged but do not cause failures.
    ///
    /// # Arguments
    ///
    /// * `git_commits` - Collection of Git commits to parse
    ///
    /// # Errors
    ///
    /// Returns error if critical parsing failures occur.
    async fn parse_git_commits(
        &self,
        git_commits: Vec<RepoCommit>,
    ) -> PackageResult<Vec<ConventionalCommit>> {
        let mut conventional_commits = Vec::new();

        for git_commit in git_commits {
            // Parse the date string to DateTime<Utc>
            let date = chrono::DateTime::parse_from_rfc3339(&git_commit.author_date)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let commit_hash = git_commit.hash.clone(); // Clone for logging

            match self.parser.parse(
                &git_commit.message,
                git_commit.hash,
                git_commit.author_name,
                date,
            ) {
                Ok(conventional_commit) => {
                    conventional_commits.push(conventional_commit);
                }
                Err(err) => {
                    // Log parsing errors but continue processing other commits
                    log::debug!(
                        "Failed to parse commit {} as conventional commit: {}",
                        commit_hash,
                        err
                    );
                }
            }
        }

        Ok(conventional_commits)
    }
}

/// Analysis results from conventional commit processing.
///
/// Provides aggregated information about a collection of conventional commits
/// including suggested version bump, commit type distribution, and breaking changes.
#[derive(Debug, Clone)]
pub struct CommitAnalysis {
    /// Total number of commits analyzed
    pub total_commits: usize,
    /// Number of commits successfully parsed as conventional commits
    pub conventional_commits: usize,
    /// Suggested version bump based on all commits
    pub suggested_bump: VersionBump,
    /// Whether any breaking changes were found
    pub has_breaking_changes: bool,
    /// Distribution of commit types
    pub type_distribution: HashMap<CommitType, usize>,
    /// Commits that include breaking changes
    pub breaking_changes: Vec<ConventionalCommit>,
}

impl ConventionalCommitService {
    /// Analyzes a collection of conventional commits and provides summary statistics.
    ///
    /// Processes commits to extract version bump suggestions, breaking change detection,
    /// and type distribution for reporting and decision making.
    ///
    /// # Arguments
    ///
    /// * `commits` - Collection of conventional commits to analyze
    ///
    /// # Returns
    ///
    /// Analysis results including suggested version bump and statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::{ConventionalCommitService, ConventionalCommit, CommitType};
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    /// use sublime_git_tools::Repo;
    /// use chrono::Utc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    ///
    /// let commits = vec![
    ///     ConventionalCommit {
    ///         commit_type: CommitType::Feat,
    ///         scope: None,
    ///         breaking: true,
    ///         description: "breaking feature".to_string(),
    ///         body: None,
    ///         footer: None,
    ///         hash: "abc123".to_string(),
    ///         author: "dev".to_string(),
    ///         date: Utc::now(),
    ///     },
    /// ];
    ///
    /// let analysis = service.analyze_commits(&commits);
    /// assert_eq!(analysis.total_commits, 1);
    /// assert!(analysis.has_breaking_changes);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn analyze_commits(&self, commits: &[ConventionalCommit]) -> CommitAnalysis {
        let total_commits = commits.len();
        let conventional_commits = commits.len(); // All provided commits are already conventional
        let suggested_bump = self.calculate_version_bump(commits);

        let breaking_changes: Vec<ConventionalCommit> =
            commits.iter().filter(|c| c.breaking).cloned().collect();
        let has_breaking_changes = !breaking_changes.is_empty();

        let mut type_distribution: HashMap<CommitType, usize> = HashMap::new();
        for commit in commits {
            *type_distribution.entry(commit.commit_type.clone()).or_insert(0) += 1;
        }

        CommitAnalysis {
            total_commits,
            conventional_commits,
            suggested_bump,
            has_breaking_changes,
            type_distribution,
            breaking_changes,
        }
    }

    /// Gets the configuration used by this service.
    ///
    /// # Returns
    ///
    /// Reference to the package tools configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::conventional::ConventionalCommitService;
    /// use sublime_git_tools::Repo;
    /// use sublime_pkg_tools::config::PackageToolsConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let repo = Repo::open(".")?;
    /// let config = PackageToolsConfig::default();
    /// let service = ConventionalCommitService::new(repo, config)?;
    ///
    /// let config_ref = service.config();
    /// println!("Required conventional commits: {}",
    ///          config_ref.conventional.require_conventional_commits);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &PackageToolsConfig {
        &self.config
    }
}
