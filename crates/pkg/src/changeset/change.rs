use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    changeset::{entry::ChangesetPackage, release::ReleaseInfo},
    error::{ChangesetError, ChangesetResult},
};

/// Reason for a package change.
///
/// Describes why a package is being updated in a changeset. This helps with
/// tracking dependencies and understanding the impact of changes across
/// the monorepo.
///
/// # Examples
///
/// ```rust
/// use sublime_pkg_tools::changeset::ChangeReason;
///
/// // Direct changes from commits
/// let direct = ChangeReason::DirectChanges {
///     commits: vec!["abc123".to_string(), "def456".to_string()],
/// };
///
/// // Dependency update propagation
/// let dep_update = ChangeReason::DependencyUpdate {
///     dependency: "@myorg/shared-lib".to_string(),
///     old_version: "1.0.0".to_string(),
///     new_version: "1.1.0".to_string(),
/// };
///
/// // Dev dependency update
/// let dev_dep_update = ChangeReason::DevDependencyUpdate {
///     dependency: "@types/node".to_string(),
///     old_version: "18.0.0".to_string(),
///     new_version: "20.0.0".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeReason {
    /// Direct changes to the package from commits
    DirectChanges {
        /// List of commit hashes that caused the change
        commits: Vec<String>,
    },
    /// Dependency update propagation
    DependencyUpdate {
        /// Name of the updated dependency
        dependency: String,
        /// Previous version of the dependency
        old_version: String,
        /// New version of the dependency
        new_version: String,
    },
    /// Dev dependency update propagation
    DevDependencyUpdate {
        /// Name of the updated dev dependency
        dependency: String,
        /// Previous version of the dev dependency
        old_version: String,
        /// New version of the dev dependency
        new_version: String,
    },
}

/// Core changeset data structure.
///
/// Represents a set of changes to be applied across multiple packages,
/// including version bumps and release targets. Changesets are created
/// on feature branches and applied when merging to main, providing a
/// controlled and reviewable way to manage version bumps.
///
/// # Lifecycle
///
/// 1. **Creation**: Created on feature branch with intended changes
/// 2. **Review**: Validated and reviewed as part of PR process
/// 3. **Application**: Applied when merging to main, updating package.json files
/// 4. **Archive**: Moved to history with release metadata for audit trail
///
/// # Filename Convention
///
/// Changesets use filename-based identity following the pattern:
/// `{sanitized_branch}-{datetime}.json`
///
/// Examples:
/// - `feat-user-auth-20240115T103000Z.json`
/// - `bugfix-memory-leak-20240115T144530Z.json`
///
/// # Examples
///
/// ## Basic Changeset
///
/// ```rust
/// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeEntry, ChangeReason};
/// use sublime_pkg_tools::{Version, VersionBump};
/// use chrono::Utc;
///
/// let mut changeset = Changeset::new(
///     "feat/user-auth".to_string(),
///     "developer@example.com".to_string(),
/// );
///
/// changeset.add_target_environment("dev".to_string());
/// changeset.add_target_environment("qa".to_string());
///
/// let package = ChangesetPackage {
///     name: "@myorg/auth-service".to_string(),
///     bump: VersionBump::Minor,
///     current_version: Version::new(1, 2, 3).into(),
///     next_version: Version::new(1, 3, 0).into(),
///     reason: ChangeReason::DirectChanges {
///         commits: vec!["abc123".to_string()],
///     },
///     dependency: None,
///     changes: vec![
///         ChangeEntry {
///             change_type: "feat".to_string(),
///             description: "Add OAuth2 support".to_string(),
///             breaking: false,
///             commit: Some("abc123".to_string()),
///         }
///     ],
/// };
///
/// changeset.add_package(package);
/// ```
///
/// ## Applied Changeset with Release Info
///
/// ```rust
/// use sublime_pkg_tools::changeset::{Changeset, ReleaseInfo, EnvironmentRelease};
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let mut changeset = Changeset::new(
///     "feat/user-auth".to_string(),
///     "developer@example.com".to_string(),
/// );
///
/// // After application, add release info
/// let mut environments = HashMap::new();
/// environments.insert(
///     "dev".to_string(),
///     EnvironmentRelease {
///         released_at: Utc::now(),
///         tag: "v1.3.0-dev".to_string(),
///     },
/// );
///
/// let release_info = ReleaseInfo {
///     applied_at: Utc::now(),
///     applied_by: "ci-bot".to_string(),
///     git_commit: "def456".to_string(),
///     environments_released: environments,
/// };
///
/// changeset.apply_release_info(release_info);
/// assert!(changeset.is_applied());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Branch where changes originated
    ///
    /// This is the Git branch name where the changes were made.
    /// Used for identification and filename generation.
    pub branch: String,

    /// When changeset was created
    ///
    /// Timestamp of changeset creation, used for sorting and
    /// filename generation.
    pub created_at: DateTime<Utc>,

    /// Author of the changeset
    ///
    /// Email or username of the person who created the changeset.
    pub author: String,

    /// Target environments for release
    ///
    /// List of environment names where this changeset should be released.
    /// Must match configured available environments.
    pub releases: Vec<String>,

    /// Package changes included in this changeset
    ///
    /// List of packages to be updated with their version bumps and reasons.
    pub packages: Vec<ChangesetPackage>,

    /// Release information (populated when applied)
    ///
    /// This field is `None` when the changeset is pending and populated
    /// with release metadata when the changeset is applied.
    pub release_info: Option<ReleaseInfo>,
}

impl Changeset {
    /// Creates a new changeset.
    ///
    /// # Arguments
    ///
    /// * `branch` - Git branch name where changes originated
    /// * `author` - Email or username of changeset author
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let changeset = Changeset::new(
    ///     "feat/user-authentication".to_string(),
    ///     "developer@example.com".to_string(),
    /// );
    ///
    /// assert_eq!(changeset.branch, "feat/user-authentication");
    /// assert_eq!(changeset.author, "developer@example.com");
    /// assert!(!changeset.is_applied());
    /// ```
    #[must_use]
    pub fn new(branch: String, author: String) -> Self {
        Self {
            branch,
            created_at: Utc::now(),
            author,
            releases: vec!["dev".to_string()], // Default to dev environment
            packages: Vec::new(),
            release_info: None,
        }
    }

    /// Validates the changeset structure and content.
    ///
    /// Performs comprehensive validation including:
    /// - Non-empty branch name
    /// - Valid author format
    /// - At least one target environment
    /// - At least one package change
    /// - Valid package structures
    /// - No duplicate packages
    /// - Environment names validation (if provided)
    ///
    /// # Arguments
    ///
    /// * `available_environments` - Optional list of valid environment names
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, `Err(ChangesetError)` with details if validation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeEntry, ChangeReason};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// // Empty changeset should fail validation
    /// assert!(changeset.validate(None).is_err());
    ///
    /// // Add a package to make it valid
    /// let package = ChangesetPackage {
    ///     name: "test-pkg".to_string(),
    ///     bump: VersionBump::Patch,
    ///     current_version: Version::new(1, 0, 0).into(),
    ///     next_version: Version::new(1, 0, 1).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["abc".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    /// changeset.add_package(package);
    ///
    /// // Now validation should pass
    /// assert!(changeset.validate(None).is_ok());
    /// ```
    pub fn validate(&self, available_environments: Option<&[String]>) -> ChangesetResult<()> {
        let mut errors = Vec::new();

        // Validate branch name
        if self.branch.trim().is_empty() {
            errors.push("Branch name cannot be empty".to_string());
        }

        // Validate author
        if self.author.trim().is_empty() {
            errors.push("Author cannot be empty".to_string());
        } else if !self.author.contains('@')
            && !self.author.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            errors.push("Author should be an email address or valid username".to_string());
        }

        // Validate release environments
        if self.releases.is_empty() {
            errors.push("At least one target environment is required".to_string());
        } else {
            // Check for duplicates
            let unique_releases: HashSet<_> = self.releases.iter().collect();
            if unique_releases.len() != self.releases.len() {
                errors.push("Duplicate environments in releases list".to_string());
            }

            // Validate against available environments if provided
            if let Some(available) = available_environments {
                let available_set: HashSet<_> = available.iter().collect();
                for release in &self.releases {
                    if !available_set.contains(release) {
                        errors.push(format!(
                            "Environment '{}' is not in available environments",
                            release
                        ));
                    }
                }
            }

            // Check for empty environment names
            for release in &self.releases {
                if release.trim().is_empty() {
                    errors.push("Environment names cannot be empty".to_string());
                    break;
                }
            }
        }

        // Validate packages
        if self.packages.is_empty() {
            errors.push("At least one package change is required".to_string());
        } else {
            // Check for duplicate packages
            let package_names: Vec<_> = self.packages.iter().map(|p| &p.name).collect();
            let unique_names: HashSet<_> = package_names.iter().collect();
            if unique_names.len() != package_names.len() {
                errors.push("Duplicate packages in changeset".to_string());
            }

            // Validate each package
            for (index, package) in self.packages.iter().enumerate() {
                if let Err(e) = self.validate_package(package) {
                    errors.push(format!("Package {} ({}): {}", index, package.name, e));
                }
            }
        }

        // Validate release info consistency if present
        if let Some(release_info) = &self.release_info {
            if release_info.applied_by.trim().is_empty() {
                errors.push("Release info applied_by cannot be empty".to_string());
            }
            if release_info.git_commit.trim().is_empty() {
                errors.push("Release info git_commit cannot be empty".to_string());
            }

            // Validate that released environments are subset of target environments
            let target_set: HashSet<_> = self.releases.iter().collect();
            for env in release_info.environments_released.keys() {
                if !target_set.contains(env) {
                    errors
                        .push(format!("Released environment '{}' not in target environments", env));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ChangesetError::validation_failed(self.generate_id(), errors))
        }
    }

    /// Validates a single package within the changeset.
    fn validate_package(&self, package: &ChangesetPackage) -> Result<(), String> {
        if package.name.trim().is_empty() {
            return Err("Package name cannot be empty".to_string());
        }

        if package.current_version == package.next_version {
            return Err("Current and next versions cannot be the same".to_string());
        }

        // Validate change entries
        for (index, change) in package.changes.iter().enumerate() {
            if change.change_type.trim().is_empty() {
                return Err(format!("Change {} has empty change_type", index));
            }
            if change.description.trim().is_empty() {
                return Err(format!("Change {} has empty description", index));
            }
        }

        // Validate reason consistency
        match &package.reason {
            ChangeReason::DirectChanges { commits } => {
                if commits.is_empty() {
                    return Err("DirectChanges reason must have at least one commit".to_string());
                }
                if commits.iter().any(|c| c.trim().is_empty()) {
                    return Err("DirectChanges commits cannot be empty".to_string());
                }
            }
            ChangeReason::DependencyUpdate { dependency, old_version, new_version }
            | ChangeReason::DevDependencyUpdate { dependency, old_version, new_version } => {
                if dependency.trim().is_empty() {
                    return Err("Dependency name cannot be empty".to_string());
                }
                if old_version.trim().is_empty() || new_version.trim().is_empty() {
                    return Err("Dependency versions cannot be empty".to_string());
                }
                if old_version == new_version {
                    return Err("Dependency old and new versions cannot be the same".to_string());
                }
            }
        }

        Ok(())
    }

    /// Generates a unique ID for this changeset based on filename convention.
    ///
    /// Uses the pattern: `{sanitized_branch}-{datetime}`
    /// - Branch name is sanitized by replacing invalid characters with hyphens
    /// - DateTime is formatted as ISO 8601 compact format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let changeset = Changeset::new(
    ///     "feat/user-auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let id = changeset.generate_id();
    /// assert!(id.starts_with("feat-user-auth-"));
    /// assert!(id.ends_with("Z"));
    /// ```
    #[must_use]
    pub fn generate_id(&self) -> String {
        let sanitized_branch = self
            .branch
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | ' ' => '-',
                c if c.is_alphanumeric() || c == '-' || c == '_' => c,
                _ => '-',
            })
            .collect::<String>()
            .trim_start_matches('-')
            .trim_end_matches('-')
            .to_string();

        let datetime = self.created_at.format("%Y%m%dT%H%M%SZ");
        format!("{}-{}", sanitized_branch, datetime)
    }

    /// Generates the filename for this changeset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let changeset = Changeset::new(
    ///     "feat/oauth2".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let filename = changeset.generate_filename();
    /// assert!(filename.starts_with("feat-oauth2-"));
    /// assert!(filename.ends_with(".json"));
    /// ```
    #[must_use]
    pub fn generate_filename(&self) -> String {
        format!("{}.json", self.generate_id())
    }

    /// Adds a package to this changeset.
    ///
    /// # Arguments
    ///
    /// * `package` - Package change to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeReason};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let package = ChangesetPackage {
    ///     name: "auth-service".to_string(),
    ///     bump: VersionBump::Minor,
    ///     current_version: Version::new(1, 0, 0).into(),
    ///     next_version: Version::new(1, 1, 0).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    ///
    /// changeset.add_package(package);
    /// assert_eq!(changeset.packages.len(), 1);
    /// ```
    pub fn add_package(&mut self, package: ChangesetPackage) {
        self.packages.push(package);
    }

    /// Adds a target environment for release.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// changeset.add_target_environment("staging".to_string());
    /// changeset.add_target_environment("prod".to_string());
    ///
    /// assert!(changeset.releases.contains(&"staging".to_string()));
    /// assert!(changeset.releases.contains(&"prod".to_string()));
    /// ```
    pub fn add_target_environment(&mut self, environment: String) {
        if !self.releases.contains(&environment) {
            self.releases.push(environment);
        }
    }

    /// Removes a target environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name to remove
    ///
    /// # Returns
    ///
    /// `true` if the environment was removed, `false` if it wasn't found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// changeset.add_target_environment("staging".to_string());
    /// assert!(changeset.remove_target_environment("staging"));
    /// assert!(!changeset.remove_target_environment("staging")); // Already removed
    /// ```
    pub fn remove_target_environment(&mut self, environment: &str) -> bool {
        if let Some(pos) = self.releases.iter().position(|e| e == environment) {
            self.releases.remove(pos);
            true
        } else {
            false
        }
    }

    /// Applies release information to mark this changeset as applied.
    ///
    /// # Arguments
    ///
    /// * `release_info` - Release metadata to attach
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ReleaseInfo, EnvironmentRelease};
    /// use std::collections::HashMap;
    /// use chrono::Utc;
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let mut environments = HashMap::new();
    /// environments.insert(
    ///     "dev".to_string(),
    ///     EnvironmentRelease {
    ///         released_at: Utc::now(),
    ///         tag: "v1.1.0-dev".to_string(),
    ///     },
    /// );
    ///
    /// let release_info = ReleaseInfo {
    ///     applied_at: Utc::now(),
    ///     applied_by: "ci-bot".to_string(),
    ///     git_commit: "def456".to_string(),
    ///     environments_released: environments,
    /// };
    ///
    /// changeset.apply_release_info(release_info);
    /// assert!(changeset.is_applied());
    /// ```
    pub fn apply_release_info(&mut self, release_info: ReleaseInfo) {
        self.release_info = Some(release_info);
    }

    /// Checks if this changeset has been applied (has release info).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// assert!(!changeset.is_applied());
    /// ```
    #[must_use]
    pub fn is_applied(&self) -> bool {
        self.release_info.is_some()
    }

    /// Checks if this changeset is pending (not yet applied).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::Changeset;
    ///
    /// let changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// assert!(changeset.is_pending());
    /// ```
    #[must_use]
    pub fn is_pending(&self) -> bool {
        !self.is_applied()
    }

    /// Gets the list of packages affected by this changeset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeReason};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let package = ChangesetPackage {
    ///     name: "auth-service".to_string(),
    ///     bump: VersionBump::Minor,
    ///     current_version: Version::new(1, 0, 0).into(),
    ///     next_version: Version::new(1, 1, 0).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    ///
    /// changeset.add_package(package);
    ///
    /// let package_names = changeset.get_package_names();
    /// assert_eq!(package_names, vec!["auth-service"]);
    /// ```
    #[must_use]
    pub fn get_package_names(&self) -> Vec<&str> {
        self.packages.iter().map(|p| p.name.as_str()).collect()
    }

    /// Finds a package by name in this changeset.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to find
    ///
    /// # Returns
    ///
    /// Reference to the package if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeReason};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let package = ChangesetPackage {
    ///     name: "auth-service".to_string(),
    ///     bump: VersionBump::Minor,
    ///     current_version: Version::new(1, 0, 0).into(),
    ///     next_version: Version::new(1, 1, 0).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["abc123".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    ///
    /// changeset.add_package(package);
    ///
    /// let found = changeset.find_package("auth-service");
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().name, "auth-service");
    /// ```
    #[must_use]
    pub fn find_package(&self, name: &str) -> Option<&ChangesetPackage> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Finds a mutable package by name in this changeset.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to find
    ///
    /// # Returns
    ///
    /// Mutable reference to the package if found, `None` otherwise.
    pub fn find_package_mut(&mut self, name: &str) -> Option<&mut ChangesetPackage> {
        self.packages.iter_mut().find(|p| p.name == name)
    }

    /// Gets a summary of version bumps in this changeset.
    ///
    /// # Returns
    ///
    /// HashMap mapping bump types to counts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{Changeset, ChangesetPackage, ChangeReason};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut changeset = Changeset::new(
    ///     "feat/auth".to_string(),
    ///     "dev@example.com".to_string(),
    /// );
    ///
    /// let package1 = ChangesetPackage {
    ///     name: "pkg1".to_string(),
    ///     bump: VersionBump::Minor,
    ///     current_version: Version::new(1, 0, 0).into(),
    ///     next_version: Version::new(1, 1, 0).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["abc".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    ///
    /// let package2 = ChangesetPackage {
    ///     name: "pkg2".to_string(),
    ///     bump: VersionBump::Patch,
    ///     current_version: Version::new(2, 0, 0).into(),
    ///     next_version: Version::new(2, 0, 1).into(),
    ///     reason: ChangeReason::DirectChanges { commits: vec!["def".to_string()] },
    ///     dependency: None,
    ///     changes: vec![],
    /// };
    ///
    /// changeset.add_package(package1);
    /// changeset.add_package(package2);
    ///
    /// let summary = changeset.get_bump_summary();
    /// assert_eq!(summary.get(&VersionBump::Minor), Some(&1));
    /// assert_eq!(summary.get(&VersionBump::Patch), Some(&1));
    /// ```
    #[must_use]
    pub fn get_bump_summary(&self) -> std::collections::HashMap<crate::VersionBump, usize> {
        let mut summary = std::collections::HashMap::new();
        for package in &self.packages {
            *summary.entry(package.bump).or_insert(0) += 1;
        }
        summary
    }
}

impl Default for Changeset {
    fn default() -> Self {
        Self {
            branch: String::new(),
            created_at: Utc::now(),
            author: String::new(),
            releases: vec!["dev".to_string()],
            packages: Vec::new(),
            release_info: None,
        }
    }
}

impl PartialEq for Changeset {
    fn eq(&self, other: &Self) -> bool {
        self.branch == other.branch
            && self.author == other.author
            && self.releases == other.releases
            && self.packages == other.packages
            && self.release_info == other.release_info
        // Note: We don't compare created_at for equality as it may vary slightly
    }
}

impl Eq for Changeset {}
