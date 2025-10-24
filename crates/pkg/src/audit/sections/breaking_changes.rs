//! Breaking changes audit section for detecting and reporting breaking changes.
//!
//! **What**: Provides functionality to audit packages for breaking changes by analyzing
//! conventional commits, changesets, and changelogs. Identifies packages that will introduce
//! breaking changes in their next release.
//!
//! **How**: Uses the `ChangesAnalyzer` to analyze commits and detect breaking changes through:
//! - Conventional commit messages with `!` indicator (e.g., `feat!:`, `fix(api)!:`)
//! - `BREAKING CHANGE:` footers in commit messages
//! - Changeset data that indicates major version bumps
//! - Changelog entries marked as breaking changes
//!
//! **Why**: To provide visibility into breaking changes before they are released, helping teams
//! make informed decisions about version updates and ensuring proper communication of breaking
//! changes to users.
//!
//! # Detection Methods
//!
//! Breaking changes are detected from multiple sources:
//!
//! ## 1. Conventional Commits
//!
//! - Commits with `!` after type: `feat!:`, `fix(scope)!:`
//! - Commits with `BREAKING CHANGE:` or `BREAKING-CHANGE:` footers
//!
//! ## 2. Changesets
//!
//! - Changesets that specify a major version bump
//! - Explicit breaking change notes in changeset data
//!
//! ## 3. Changelogs
//!
//! - Changelog sections marked as breaking changes
//! - Version transitions that indicate breaking changes

use crate::audit::issue::{AuditIssue, IssueCategory, IssueSeverity};
use crate::changelog::ConventionalCommit;
use crate::changes::{ChangesAnalyzer, CommitInfo};
use crate::config::BreakingChangesAuditConfig;
use crate::error::{AuditError, AuditResult};
use crate::types::Changeset;
use crate::types::{Version, VersionBump};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sublime_standard_tools::filesystem::AsyncFileSystem;

/// Audit section containing breaking changes analysis results.
///
/// Contains detailed information about packages with breaking changes,
/// including the source of each breaking change (commit, changelog, or changeset).
///
/// # Examples
///
/// ## Accessing breaking changes statistics
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
///
/// # fn example(section: BreakingChangesAuditSection) {
/// println!("Packages with breaking changes: {}", section.packages_with_breaking.len());
/// println!("Total breaking changes: {}", section.total_breaking_changes);
/// println!("Issues found: {}", section.issues.len());
/// # }
/// ```
///
/// ## Checking for critical issues
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
///
/// # fn example(section: BreakingChangesAuditSection) {
/// let critical_issues: Vec<_> = section.issues.iter()
///     .filter(|issue| issue.is_critical())
///     .collect();
///
/// if !critical_issues.is_empty() {
///     println!("Found {} critical breaking change issues", critical_issues.len());
///     for issue in critical_issues {
///         println!("  - {}: {}", issue.title, issue.description);
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangesAuditSection {
    /// Packages with detected breaking changes.
    ///
    /// Each entry represents a package that has one or more breaking changes.
    pub packages_with_breaking: Vec<PackageBreakingChanges>,

    /// Total number of breaking changes found across all packages.
    ///
    /// Sum of all breaking changes in all packages.
    pub total_breaking_changes: usize,

    /// List of audit issues generated from the breaking changes analysis.
    ///
    /// Issues are created based on breaking change severity:
    /// - Breaking changes with major version bump → Critical issues
    /// - Breaking changes without version bump → Warning issues
    pub issues: Vec<AuditIssue>,
}

impl BreakingChangesAuditSection {
    /// Creates an empty breaking changes audit section.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let section = BreakingChangesAuditSection::empty();
    /// assert_eq!(section.total_breaking_changes, 0);
    /// assert!(section.issues.is_empty());
    /// ```
    #[must_use]
    pub fn empty() -> Self {
        Self { packages_with_breaking: Vec::new(), total_breaking_changes: 0, issues: Vec::new() }
    }

    /// Returns whether any breaking changes were found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let empty = BreakingChangesAuditSection::empty();
    /// assert!(!empty.has_breaking_changes());
    /// ```
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.total_breaking_changes > 0
    }

    /// Returns the number of packages with breaking changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let section = BreakingChangesAuditSection::empty();
    /// assert_eq!(section.affected_package_count(), 0);
    /// ```
    #[must_use]
    pub fn affected_package_count(&self) -> usize {
        self.packages_with_breaking.len()
    }

    /// Returns the number of critical issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let section = BreakingChangesAuditSection::empty();
    /// assert_eq!(section.critical_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn critical_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_critical()).count()
    }

    /// Returns the number of warning issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let section = BreakingChangesAuditSection::empty();
    /// assert_eq!(section.warning_issue_count(), 0);
    /// ```
    #[must_use]
    pub fn warning_issue_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.is_warning()).count()
    }

    /// Returns breaking changes for a specific package.
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to get breaking changes for
    ///
    /// # Returns
    ///
    /// A reference to the package's breaking changes, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::BreakingChangesAuditSection;
    ///
    /// let section = BreakingChangesAuditSection::empty();
    /// let breaking = section.breaking_changes_for_package("my-app");
    /// assert!(breaking.is_none());
    /// ```
    #[must_use]
    pub fn breaking_changes_for_package(
        &self,
        package_name: &str,
    ) -> Option<&PackageBreakingChanges> {
        self.packages_with_breaking.iter().find(|p| p.package_name == package_name)
    }
}

/// Breaking changes detected for a specific package.
///
/// Contains information about a package's breaking changes, including current and
/// next versions, and a list of all detected breaking changes.
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::PackageBreakingChanges;
/// use sublime_pkg_tools::types::Version;
///
/// let breaking = PackageBreakingChanges {
///     package_name: "@myorg/core".to_string(),
///     current_version: Some(Version::parse("1.2.3")?),
///     next_version: Some(Version::parse("2.0.0")?),
///     breaking_changes: vec![],
/// };
///
/// assert!(breaking.is_major_bump());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageBreakingChanges {
    /// Name of the package with breaking changes.
    pub package_name: String,

    /// Current version of the package.
    ///
    /// May be `None` if the package is new or version cannot be determined.
    pub current_version: Option<Version>,

    /// Next version of the package after breaking changes.
    ///
    /// May be `None` if version cannot be calculated.
    pub next_version: Option<Version>,

    /// List of breaking changes detected for this package.
    pub breaking_changes: Vec<BreakingChange>,
}

impl PackageBreakingChanges {
    /// Returns whether this represents a major version bump.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use sublime_pkg_tools::audit::PackageBreakingChanges;
    /// use sublime_pkg_tools::types::Version;
    ///
    /// let breaking = PackageBreakingChanges {
    ///     package_name: "@myorg/core".to_string(),
    ///     current_version: Some(Version::parse("1.2.3")?),
    ///     next_version: Some(Version::parse("2.0.0")?),
    ///     breaking_changes: vec![],
    /// };
    ///
    /// assert!(breaking.is_major_bump());
    /// ```
    #[must_use]
    pub fn is_major_bump(&self) -> bool {
        match (&self.current_version, &self.next_version) {
            (Some(current), Some(next)) => next.major() > current.major(),
            _ => false,
        }
    }

    /// Returns the number of breaking changes for this package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::PackageBreakingChanges;
    ///
    /// let breaking = PackageBreakingChanges {
    ///     package_name: "@myorg/core".to_string(),
    ///     current_version: None,
    ///     next_version: None,
    ///     breaking_changes: vec![],
    /// };
    ///
    /// assert_eq!(breaking.breaking_change_count(), 0);
    /// ```
    #[must_use]
    pub fn breaking_change_count(&self) -> usize {
        self.breaking_changes.len()
    }
}

/// A single breaking change detected in a package.
///
/// Represents an individual breaking change with its description, source commit,
/// and the method used to detect it.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::{BreakingChange, BreakingChangeSource};
///
/// let change = BreakingChange {
///     description: "Removed deprecated API".to_string(),
///     commit_hash: Some("abc123def".to_string()),
///     source: BreakingChangeSource::ConventionalCommit,
/// };
///
/// assert!(change.has_commit());
/// assert!(change.is_from_conventional_commit());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreakingChange {
    /// Description of the breaking change.
    ///
    /// This is typically extracted from:
    /// - Commit message description
    /// - `BREAKING CHANGE:` footer content
    /// - Changeset notes
    /// - Changelog entry
    pub description: String,

    /// Git commit hash associated with this breaking change.
    ///
    /// May be `None` if the change comes from a changeset or changelog
    /// that isn't associated with a specific commit.
    pub commit_hash: Option<String>,

    /// Source where this breaking change was detected.
    pub source: BreakingChangeSource,
}

impl BreakingChange {
    /// Returns whether this breaking change has an associated commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{BreakingChange, BreakingChangeSource};
    ///
    /// let change = BreakingChange {
    ///     description: "API change".to_string(),
    ///     commit_hash: Some("abc123".to_string()),
    ///     source: BreakingChangeSource::ConventionalCommit,
    /// };
    ///
    /// assert!(change.has_commit());
    /// ```
    #[must_use]
    pub fn has_commit(&self) -> bool {
        self.commit_hash.is_some()
    }

    /// Returns whether this change was detected from a conventional commit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{BreakingChange, BreakingChangeSource};
    ///
    /// let change = BreakingChange {
    ///     description: "API change".to_string(),
    ///     commit_hash: None,
    ///     source: BreakingChangeSource::ConventionalCommit,
    /// };
    ///
    /// assert!(change.is_from_conventional_commit());
    /// ```
    #[must_use]
    pub fn is_from_conventional_commit(&self) -> bool {
        matches!(self.source, BreakingChangeSource::ConventionalCommit)
    }

    /// Returns whether this change was detected from a changeset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{BreakingChange, BreakingChangeSource};
    ///
    /// let change = BreakingChange {
    ///     description: "API change".to_string(),
    ///     commit_hash: None,
    ///     source: BreakingChangeSource::Changeset,
    /// };
    ///
    /// assert!(change.is_from_changeset());
    /// ```
    #[must_use]
    pub fn is_from_changeset(&self) -> bool {
        matches!(self.source, BreakingChangeSource::Changeset)
    }

    /// Returns whether this change was detected from a changelog.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::audit::{BreakingChange, BreakingChangeSource};
    ///
    /// let change = BreakingChange {
    ///     description: "API change".to_string(),
    ///     commit_hash: None,
    ///     source: BreakingChangeSource::Changelog,
    /// };
    ///
    /// assert!(change.is_from_changelog());
    /// ```
    #[must_use]
    pub fn is_from_changelog(&self) -> bool {
        matches!(self.source, BreakingChangeSource::Changelog)
    }
}

/// Source where a breaking change was detected.
///
/// Indicates the method used to detect the breaking change, which affects
/// how the change is presented and what additional information is available.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::audit::BreakingChangeSource;
///
/// let source = BreakingChangeSource::ConventionalCommit;
/// assert_eq!(format!("{:?}", source), "ConventionalCommit");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingChangeSource {
    /// Detected from a conventional commit message.
    ///
    /// Commits with `!` indicator (e.g., `feat!:`, `fix(api)!:`) or
    /// `BREAKING CHANGE:` footers.
    ConventionalCommit,

    /// Detected from a changelog entry.
    ///
    /// Changelog sections marked as breaking changes or major version releases.
    Changelog,

    /// Detected from a changeset.
    ///
    /// Changesets that specify major version bumps or breaking change notes.
    Changeset,
}

/// Performs a breaking changes audit using the provided changes analyzer.
///
/// This function:
/// 1. Analyzes commits in the specified range for breaking changes
/// 2. Parses conventional commit messages to detect breaking indicators
/// 3. Checks changeset data for major version bumps
/// 4. Generates audit issues based on severity:
///    - Breaking changes with major bump → Critical
///    - Breaking changes without version info → Warning
///
/// # Arguments
///
/// * `changes_analyzer` - The changes analyzer to use for detection
/// * `commit_from` - Starting commit ref (e.g., "main", commit hash)
/// * `commit_to` - Ending commit ref (e.g., "HEAD", commit hash)
/// * `changeset` - Optional changeset data to include in analysis
/// * `config` - Configuration specifying which detection methods to use
///
/// # Returns
///
/// A `BreakingChangesAuditSection` containing all breaking changes analysis results.
///
/// # Errors
///
/// Returns `AuditError` if:
/// - The breaking changes audit section is disabled in configuration
/// - Commit analysis fails (invalid refs, Git errors, etc.)
/// - Conventional commit parsing fails
/// - Package analysis fails
///
/// # Examples
///
/// ```rust,ignore
/// use sublime_pkg_tools::audit::audit_breaking_changes;
/// use sublime_pkg_tools::changes::ChangesAnalyzer;
/// use sublime_pkg_tools::config::PackageToolsConfig;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = PackageToolsConfig::default();
/// // TODO: will be implemented on story 7.1
/// // let analyzer = ChangesAnalyzer::new(...).await?;
/// //
/// // let section = audit_breaking_changes(
/// //     &analyzer,
/// //     "main",
/// //     "HEAD",
/// //     None,
/// //     &config.audit.breaking_changes,
/// // ).await?;
/// //
/// // println!("Found {} breaking changes", section.total_breaking_changes);
/// # Ok(())
/// # }
/// ```
pub async fn audit_breaking_changes<FS: AsyncFileSystem + Clone + Send + Sync>(
    changes_analyzer: &ChangesAnalyzer<FS>,
    commit_from: &str,
    commit_to: &str,
    changeset: Option<&Changeset>,
    config: &BreakingChangesAuditConfig,
) -> AuditResult<BreakingChangesAuditSection> {
    // Check if both detection methods are disabled
    if !config.check_conventional_commits && !config.check_changelog {
        return Ok(BreakingChangesAuditSection::empty());
    }

    // Analyze changes in the commit range
    let changes_report = changes_analyzer
        .analyze_commit_range(commit_from, commit_to)
        .await
        .map_err(|e| AuditError::BreakingChangesDetectionFailed {
            reason: format!("Failed to analyze commit range: {}", e),
        })?;

    let mut packages_map: HashMap<String, PackageBreakingChanges> = HashMap::new();

    // Analyze each package for breaking changes
    for package_changes in &changes_report.packages {
        if !package_changes.has_changes {
            continue;
        }

        let mut breaking_changes_list = Vec::new();

        // Check conventional commits if enabled
        if config.check_conventional_commits {
            breaking_changes_list
                .extend(detect_breaking_from_commits(&package_changes.commits).await?);
        }

        // Check changeset if provided and enabled
        if let Some(cs) = changeset {
            breaking_changes_list
                .extend(detect_breaking_from_changeset(package_changes.package_name(), cs)?);
        }

        // Only add package if it has breaking changes
        if !breaking_changes_list.is_empty() {
            packages_map.insert(
                package_changes.package_name().to_string(),
                PackageBreakingChanges {
                    package_name: package_changes.package_name().to_string(),
                    current_version: package_changes.current_version.clone(),
                    next_version: package_changes.next_version.clone(),
                    breaking_changes: breaking_changes_list,
                },
            );
        }
    }

    // Calculate total breaking changes
    let total_breaking_changes: usize =
        packages_map.values().map(|p| p.breaking_changes.len()).sum();

    // Generate audit issues
    let issues = generate_breaking_change_issues(&packages_map);

    Ok(BreakingChangesAuditSection {
        packages_with_breaking: packages_map.into_values().collect(),
        total_breaking_changes,
        issues,
    })
}

/// Detects breaking changes from commit messages using conventional commit parsing.
///
/// Analyzes commit messages to detect:
/// - Commits with `!` indicator (e.g., `feat!:`, `fix(api)!:`)
/// - Commits with `BREAKING CHANGE:` or `BREAKING-CHANGE:` footers
///
/// # Arguments
///
/// * `commits` - List of commits to analyze
///
/// # Returns
///
/// A list of detected breaking changes.
///
/// # Errors
///
/// Returns `AuditError` if conventional commit parsing fails unexpectedly.
async fn detect_breaking_from_commits(commits: &[CommitInfo]) -> AuditResult<Vec<BreakingChange>> {
    let mut breaking_changes = Vec::new();

    for commit in commits {
        // Try to parse as conventional commit
        match ConventionalCommit::parse(&commit.full_message) {
            Ok(conventional) => {
                if conventional.is_breaking() {
                    // Extract description from the commit
                    let description = if conventional.body().is_some() {
                        // Look for BREAKING CHANGE footer content
                        let breaking_footer = conventional
                            .footers()
                            .iter()
                            .find(|f| f.key == "BREAKING CHANGE" || f.key == "BREAKING-CHANGE");

                        if let Some(footer) = breaking_footer {
                            footer.value.clone()
                        } else {
                            format!(
                                "{}: {}",
                                conventional.commit_type(),
                                conventional.description()
                            )
                        }
                    } else {
                        format!("{}: {}", conventional.commit_type(), conventional.description())
                    };

                    breaking_changes.push(BreakingChange {
                        description,
                        commit_hash: Some(commit.short_hash.clone()),
                        source: BreakingChangeSource::ConventionalCommit,
                    });
                }
            }
            Err(_) => {
                // Not a conventional commit or failed to parse, skip
                // This is expected for non-conventional commits
                continue;
            }
        }
    }

    Ok(breaking_changes)
}

/// Detects breaking changes from changeset data.
///
/// Checks if the changeset specifies a major version bump for the package,
/// which indicates breaking changes.
///
/// # Arguments
///
/// * `package_name` - Name of the package to check
/// * `changeset` - Changeset data to analyze
///
/// # Returns
///
/// A list of detected breaking changes (empty if no major bump).
///
/// # Errors
///
/// Returns `AuditError` if changeset analysis fails.
fn detect_breaking_from_changeset(
    package_name: &str,
    changeset: &Changeset,
) -> AuditResult<Vec<BreakingChange>> {
    let mut breaking_changes = Vec::new();

    // Check if this package is in the changeset
    if changeset.packages.iter().any(|p| p == package_name) {
        // Check if the bump type indicates a breaking change (major version)
        if matches!(changeset.bump, VersionBump::Major) {
            // Create a breaking change entry from the changeset
            let description =
                if changeset.changes.is_empty() {
                    format!("Major version bump planned for {}", package_name)
                } else {
                    // Use the first change description as the breaking change description
                    changeset.changes.first().map(|c| c.to_string()).unwrap_or_else(|| {
                        format!("Major version bump planned for {}", package_name)
                    })
                };

            breaking_changes.push(BreakingChange {
                description,
                commit_hash: None,
                source: BreakingChangeSource::Changeset,
            });
        }
    }

    Ok(breaking_changes)
}

/// Generates audit issues from detected breaking changes.
///
/// Creates issues based on breaking change severity:
/// - Breaking changes with major version bump → Critical
/// - Breaking changes without version information → Warning
///
/// # Arguments
///
/// * `packages_map` - Map of package names to their breaking changes
///
/// # Returns
///
/// A list of audit issues.
fn generate_breaking_change_issues(
    packages_map: &HashMap<String, PackageBreakingChanges>,
) -> Vec<AuditIssue> {
    let mut issues = Vec::new();

    for package_breaking in packages_map.values() {
        let severity = if package_breaking.is_major_bump() {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };

        let version_info = match (&package_breaking.current_version, &package_breaking.next_version)
        {
            (Some(current), Some(next)) => format!(" ({} → {})", current, next),
            (Some(current), None) => format!(" (current: {})", current),
            (None, Some(next)) => format!(" (next: {})", next),
            (None, None) => String::new(),
        };

        let title = format!(
            "Breaking changes detected in {}{}",
            package_breaking.package_name, version_info
        );

        let description = if package_breaking.breaking_changes.len() == 1 {
            format!(
                "1 breaking change detected in {}. {}",
                package_breaking.package_name, package_breaking.breaking_changes[0].description
            )
        } else {
            let changes_list = package_breaking
                .breaking_changes
                .iter()
                .map(|bc| format!("  - {}", bc.description))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "{} breaking changes detected in {}:\n{}",
                package_breaking.breaking_changes.len(),
                package_breaking.package_name,
                changes_list
            )
        };

        let suggestion = if package_breaking.is_major_bump() {
            Some(format!(
                "Review breaking changes for {} and update documentation. Ensure major version bump is intentional.",
                package_breaking.package_name
            ))
        } else {
            Some(format!(
                "Review breaking changes for {} and ensure appropriate version bump (major) is planned.",
                package_breaking.package_name
            ))
        };

        let mut metadata = HashMap::new();
        metadata.insert("package".to_string(), package_breaking.package_name.clone());
        metadata.insert(
            "breaking_change_count".to_string(),
            package_breaking.breaking_changes.len().to_string(),
        );

        if let Some(current) = &package_breaking.current_version {
            metadata.insert("current_version".to_string(), current.to_string());
        }
        if let Some(next) = &package_breaking.next_version {
            metadata.insert("next_version".to_string(), next.to_string());
        }

        issues.push(AuditIssue {
            severity,
            category: IssueCategory::BreakingChanges,
            title,
            description,
            affected_packages: vec![package_breaking.package_name.clone()],
            suggestion,
            metadata,
        });
    }

    issues
}
