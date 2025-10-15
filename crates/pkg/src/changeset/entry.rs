use serde::{Deserialize, Serialize};

use crate::{changeset::ChangeReason, ResolvedVersion, VersionBump};

/// Package-specific changes within a changeset.
///
/// Represents a single package that will be updated as part of a changeset,
/// including its version bump, the reason for the change, and detailed
/// change entries for documentation and changelog generation.
///
/// # Examples
///
/// ## Direct Changes Package
///
/// ```rust
/// use sublime_pkg_tools::changeset::{ChangesetPackage, ChangeEntry, ChangeReason};
/// use sublime_pkg_tools::{Version, VersionBump};
///
/// let package = ChangesetPackage {
///     name: "@myorg/auth-service".to_string(),
///     bump: VersionBump::Minor,
///     current_version: Version::new(1, 2, 3).into(),
///     next_version: Version::new(1, 3, 0).into(),
///     reason: ChangeReason::DirectChanges {
///         commits: vec!["abc123".to_string(), "def456".to_string()],
///     },
///     dependency: None,
///     changes: vec![
///         ChangeEntry {
///             change_type: "feat".to_string(),
///             description: "Add OAuth2 authentication support".to_string(),
///             breaking: false,
///             commit: Some("abc123".to_string()),
///         },
///         ChangeEntry {
///             change_type: "fix".to_string(),
///             description: "Fix memory leak in token validation".to_string(),
///             breaking: false,
///             commit: Some("def456".to_string()),
///         },
///     ],
/// };
///
/// assert_eq!(package.name, "@myorg/auth-service");
/// assert!(package.has_breaking_changes() == false);
/// assert_eq!(package.changes.len(), 2);
/// ```
///
/// ## Dependency Update Package
///
/// ```rust
/// use sublime_pkg_tools::changeset::{ChangesetPackage, ChangeReason};
/// use sublime_pkg_tools::{Version, VersionBump};
///
/// let package = ChangesetPackage {
///     name: "@myorg/user-service".to_string(),
///     bump: VersionBump::Patch,
///     current_version: Version::new(2, 1, 0).into(),
///     next_version: Version::new(2, 1, 1).into(),
///     reason: ChangeReason::DependencyUpdate {
///         dependency: "@myorg/auth-service".to_string(),
///         old_version: "1.2.3".to_string(),
///         new_version: "1.3.0".to_string(),
///     },
///     dependency: Some("@myorg/auth-service".to_string()),
///     changes: vec![], // No direct changes, just dependency update
/// };
///
/// assert!(package.is_dependency_update());
/// assert_eq!(package.dependency, Some("@myorg/auth-service".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangesetPackage {
    /// Package name (e.g., "@myorg/auth-service" or "my-package")
    ///
    /// Should match the name field in the package's package.json file.
    pub name: String,

    /// Type of version bump to apply
    ///
    /// Determines how the version will be incremented:
    /// - Major: Breaking changes (1.0.0 → 2.0.0)
    /// - Minor: New features (1.0.0 → 1.1.0)
    /// - Patch: Bug fixes (1.0.0 → 1.0.1)
    /// - None: No version change (for metadata-only updates)
    pub bump: VersionBump,

    /// Current version of the package
    ///
    /// The version currently in package.json before applying the changeset.
    pub current_version: ResolvedVersion,

    /// Next version after applying the bump
    ///
    /// The version that will be written to package.json after applying
    /// the changeset.
    pub next_version: ResolvedVersion,

    /// Reason for the version bump
    ///
    /// Explains why this package is being updated, whether due to direct
    /// changes or dependency propagation.
    pub reason: ChangeReason,

    /// Optional dependency that triggered this change
    ///
    /// Set when this package is being updated due to a dependency change.
    /// Should match the dependency name in the ChangeReason for consistency.
    pub dependency: Option<String>,

    /// Individual change entries for this package
    ///
    /// List of specific changes made to the package, used for generating
    /// changelogs and providing detailed release notes.
    pub changes: Vec<ChangeEntry>,
}

impl ChangesetPackage {
    /// Creates a new changeset package with direct changes.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `bump` - Version bump type
    /// * `current_version` - Current package version
    /// * `next_version` - Version after bump
    /// * `commits` - List of commit hashes that caused changes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// assert_eq!(package.name, "my-package");
    /// assert!(package.is_direct_changes());
    /// ```
    #[must_use]
    pub fn new_direct_changes(
        name: String,
        bump: VersionBump,
        current_version: ResolvedVersion,
        next_version: ResolvedVersion,
        commits: Vec<String>,
    ) -> Self {
        Self {
            name,
            bump,
            current_version,
            next_version,
            reason: ChangeReason::DirectChanges { commits },
            dependency: None,
            changes: Vec::new(),
        }
    }

    /// Creates a new changeset package for dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `bump` - Version bump type
    /// * `current_version` - Current package version
    /// * `next_version` - Version after bump
    /// * `dependency` - Name of the updated dependency
    /// * `old_version` - Previous dependency version
    /// * `new_version` - New dependency version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_dependency_update(
    ///     "dependent-package".to_string(),
    ///     VersionBump::Patch,
    ///     Version::new(2, 0, 0).into(),
    ///     Version::new(2, 0, 1).into(),
    ///     "shared-lib".to_string(),
    ///     "1.0.0".to_string(),
    ///     "1.1.0".to_string(),
    /// );
    ///
    /// assert_eq!(package.name, "dependent-package");
    /// assert!(package.is_dependency_update());
    /// assert_eq!(package.dependency, Some("shared-lib".to_string()));
    /// ```
    #[must_use]
    pub fn new_dependency_update(
        name: String,
        bump: VersionBump,
        current_version: ResolvedVersion,
        next_version: ResolvedVersion,
        dependency: String,
        old_version: String,
        new_version: String,
    ) -> Self {
        Self {
            name,
            bump,
            current_version,
            next_version,
            reason: ChangeReason::DependencyUpdate {
                dependency: dependency.clone(),
                old_version,
                new_version,
            },
            dependency: Some(dependency),
            changes: Vec::new(),
        }
    }

    /// Creates a new changeset package for dev dependency update.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `bump` - Version bump type
    /// * `current_version` - Current package version
    /// * `next_version` - Version after bump
    /// * `dependency` - Name of the updated dev dependency
    /// * `old_version` - Previous dependency version
    /// * `new_version` - New dependency version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_dev_dependency_update(
    ///     "test-package".to_string(),
    ///     VersionBump::Patch,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 0, 1).into(),
    ///     "@types/node".to_string(),
    ///     "18.0.0".to_string(),
    ///     "20.0.0".to_string(),
    /// );
    ///
    /// assert!(package.is_dev_dependency_update());
    /// ```
    #[must_use]
    pub fn new_dev_dependency_update(
        name: String,
        bump: VersionBump,
        current_version: ResolvedVersion,
        next_version: ResolvedVersion,
        dependency: String,
        old_version: String,
        new_version: String,
    ) -> Self {
        Self {
            name,
            bump,
            current_version,
            next_version,
            reason: ChangeReason::DevDependencyUpdate {
                dependency: dependency.clone(),
                old_version,
                new_version,
            },
            dependency: Some(dependency),
            changes: Vec::new(),
        }
    }

    /// Adds a change entry to this package.
    ///
    /// # Arguments
    ///
    /// * `entry` - Change entry to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{ChangesetPackage, ChangeEntry};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// let entry = ChangeEntry {
    ///     change_type: "feat".to_string(),
    ///     description: "Add new feature".to_string(),
    ///     breaking: false,
    ///     commit: Some("abc123".to_string()),
    /// };
    ///
    /// package.add_change(entry);
    /// assert_eq!(package.changes.len(), 1);
    /// ```
    pub fn add_change(&mut self, entry: ChangeEntry) {
        self.changes.push(entry);
    }

    /// Checks if this package has breaking changes.
    ///
    /// # Returns
    ///
    /// `true` if any change entry is marked as breaking, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{ChangesetPackage, ChangeEntry};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Major,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(2, 0, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// let breaking_entry = ChangeEntry {
    ///     change_type: "feat".to_string(),
    ///     description: "Remove deprecated API".to_string(),
    ///     breaking: true,
    ///     commit: Some("abc123".to_string()),
    /// };
    ///
    /// package.add_change(breaking_entry);
    /// assert!(package.has_breaking_changes());
    /// ```
    #[must_use]
    pub fn has_breaking_changes(&self) -> bool {
        self.changes.iter().any(|change| change.breaking)
    }

    /// Checks if this package was updated due to direct changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// assert!(package.is_direct_changes());
    /// assert!(!package.is_dependency_update());
    /// ```
    #[must_use]
    pub fn is_direct_changes(&self) -> bool {
        matches!(self.reason, ChangeReason::DirectChanges { .. })
    }

    /// Checks if this package was updated due to a dependency update.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_dependency_update(
    ///     "dependent-package".to_string(),
    ///     VersionBump::Patch,
    ///     Version::new(2, 0, 0).into(),
    ///     Version::new(2, 0, 1).into(),
    ///     "shared-lib".to_string(),
    ///     "1.0.0".to_string(),
    ///     "1.1.0".to_string(),
    /// );
    ///
    /// assert!(package.is_dependency_update());
    /// assert!(!package.is_direct_changes());
    /// ```
    #[must_use]
    pub fn is_dependency_update(&self) -> bool {
        matches!(self.reason, ChangeReason::DependencyUpdate { .. })
    }

    /// Checks if this package was updated due to a dev dependency update.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_dev_dependency_update(
    ///     "test-package".to_string(),
    ///     VersionBump::Patch,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 0, 1).into(),
    ///     "@types/node".to_string(),
    ///     "18.0.0".to_string(),
    ///     "20.0.0".to_string(),
    /// );
    ///
    /// assert!(package.is_dev_dependency_update());
    /// assert!(!package.is_dependency_update());
    /// ```
    #[must_use]
    pub fn is_dev_dependency_update(&self) -> bool {
        matches!(self.reason, ChangeReason::DevDependencyUpdate { .. })
    }

    /// Gets the commits associated with this package change.
    ///
    /// # Returns
    ///
    /// Vector of commit hashes if this is a direct change, empty vector otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string(), "def456".to_string()],
    /// );
    ///
    /// let commits = package.get_commits();
    /// assert_eq!(commits, vec!["abc123", "def456"]);
    /// ```
    #[must_use]
    pub fn get_commits(&self) -> Vec<&str> {
        match &self.reason {
            ChangeReason::DirectChanges { commits } => commits.iter().map(|s| s.as_str()).collect(),
            _ => Vec::new(),
        }
    }

    /// Gets the dependency information if this is a dependency update.
    ///
    /// # Returns
    ///
    /// Tuple of (dependency_name, old_version, new_version) if this is a dependency
    /// update, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_dependency_update(
    ///     "dependent-package".to_string(),
    ///     VersionBump::Patch,
    ///     Version::new(2, 0, 0).into(),
    ///     Version::new(2, 0, 1).into(),
    ///     "shared-lib".to_string(),
    ///     "1.0.0".to_string(),
    ///     "1.1.0".to_string(),
    /// );
    ///
    /// let dep_info = package.get_dependency_info();
    /// assert_eq!(dep_info, Some(("shared-lib", "1.0.0", "1.1.0")));
    /// ```
    #[must_use]
    pub fn get_dependency_info(&self) -> Option<(&str, &str, &str)> {
        match &self.reason {
            ChangeReason::DependencyUpdate { dependency, old_version, new_version }
            | ChangeReason::DevDependencyUpdate { dependency, old_version, new_version } => {
                Some((dependency, old_version, new_version))
            }
            _ => None,
        }
    }

    /// Gets a summary of change types in this package.
    ///
    /// # Returns
    ///
    /// HashMap mapping change types to their counts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::{ChangesetPackage, ChangeEntry};
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let mut package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// package.add_change(ChangeEntry {
    ///     change_type: "feat".to_string(),
    ///     description: "Add feature 1".to_string(),
    ///     breaking: false,
    ///     commit: None,
    /// });
    ///
    /// package.add_change(ChangeEntry {
    ///     change_type: "feat".to_string(),
    ///     description: "Add feature 2".to_string(),
    ///     breaking: false,
    ///     commit: None,
    /// });
    ///
    /// package.add_change(ChangeEntry {
    ///     change_type: "fix".to_string(),
    ///     description: "Fix bug".to_string(),
    ///     breaking: false,
    ///     commit: None,
    /// });
    ///
    /// let summary = package.get_change_type_summary();
    /// assert_eq!(summary.get("feat"), Some(&2));
    /// assert_eq!(summary.get("fix"), Some(&1));
    /// ```
    #[must_use]
    pub fn get_change_type_summary(&self) -> std::collections::HashMap<String, usize> {
        let mut summary = std::collections::HashMap::new();
        for change in &self.changes {
            *summary.entry(change.change_type.clone()).or_insert(0) += 1;
        }
        summary
    }

    /// Validates this package's structure and data.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with validation error message if invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangesetPackage;
    /// use sublime_pkg_tools::{Version, VersionBump};
    ///
    /// let package = ChangesetPackage::new_direct_changes(
    ///     "my-package".to_string(),
    ///     VersionBump::Minor,
    ///     Version::new(1, 0, 0).into(),
    ///     Version::new(1, 1, 0).into(),
    ///     vec!["abc123".to_string()],
    /// );
    ///
    /// assert!(package.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Package name cannot be empty".to_string());
        }

        if self.current_version == self.next_version && self.bump != VersionBump::None {
            return Err("Current and next versions must differ for non-None bump".to_string());
        }

        // Validate reason consistency
        match &self.reason {
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
                    return Err("Dependency old and new versions must differ".to_string());
                }

                // Check consistency with dependency field
                if let Some(dep_field) = &self.dependency {
                    if dep_field != dependency {
                        return Err("Dependency field must match reason dependency".to_string());
                    }
                } else {
                    return Err("Dependency field must be set for dependency updates".to_string());
                }
            }
        }

        // Validate changes
        for (index, change) in self.changes.iter().enumerate() {
            if let Err(e) = change.validate() {
                return Err(format!("Change {}: {}", index, e));
            }
        }

        Ok(())
    }
}

/// Individual change entry within a package.
///
/// Represents a specific change made to a package, typically derived from
/// a conventional commit. Used for generating changelogs and providing
/// detailed release documentation.
///
/// # Conventional Commit Types
///
/// Common change types include:
/// - `feat`: New features
/// - `fix`: Bug fixes
/// - `docs`: Documentation changes
/// - `style`: Code style changes
/// - `refactor`: Code refactoring
/// - `perf`: Performance improvements
/// - `test`: Test changes
/// - `build`: Build system changes
/// - `ci`: CI configuration changes
/// - `chore`: Maintenance tasks
/// - `revert`: Revert previous changes
///
/// # Examples
///
/// ## Feature Addition
///
/// ```rust
/// use sublime_pkg_tools::changeset::ChangeEntry;
///
/// let feature = ChangeEntry {
///     change_type: "feat".to_string(),
///     description: "Add user authentication with OAuth2".to_string(),
///     breaking: false,
///     commit: Some("abc123def456".to_string()),
/// };
///
/// assert_eq!(feature.change_type, "feat");
/// assert!(!feature.is_breaking());
/// ```
///
/// ## Breaking Change
///
/// ```rust
/// use sublime_pkg_tools::changeset::ChangeEntry;
///
/// let breaking_change = ChangeEntry {
///     change_type: "feat".to_string(),
///     description: "Remove deprecated authentication API".to_string(),
///     breaking: true,
///     commit: Some("def456abc789".to_string()),
/// };
///
/// assert!(breaking_change.is_breaking());
/// assert!(breaking_change.is_feature());
/// ```
///
/// ## Bug Fix
///
/// ```rust
/// use sublime_pkg_tools::changeset::ChangeEntry;
///
/// let bug_fix = ChangeEntry {
///     change_type: "fix".to_string(),
///     description: "Fix memory leak in token validation".to_string(),
///     breaking: false,
///     commit: Some("ghi789jkl012".to_string()),
/// };
///
/// assert!(bug_fix.is_fix());
/// assert!(!bug_fix.is_breaking());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeEntry {
    /// Type of change (feat, fix, docs, etc.)
    ///
    /// Should follow conventional commit conventions. Common types:
    /// - feat: New features
    /// - fix: Bug fixes
    /// - docs: Documentation
    /// - style: Formatting
    /// - refactor: Code restructuring
    /// - perf: Performance improvements
    /// - test: Testing
    /// - build: Build system
    /// - ci: CI/CD
    /// - chore: Maintenance
    pub change_type: String,

    /// Human-readable description of the change
    ///
    /// Should be concise but descriptive enough for changelog inclusion.
    /// Typically the commit message subject line.
    pub description: String,

    /// Whether this change breaks backward compatibility
    ///
    /// Breaking changes typically require a major version bump and
    /// special attention in changelogs and release notes.
    pub breaking: bool,

    /// Associated commit hash (optional)
    ///
    /// Git commit hash that introduced this change. Used for linking
    /// back to the original commit in changelogs and documentation.
    pub commit: Option<String>,
}

impl ChangeEntry {
    /// Creates a new change entry.
    ///
    /// # Arguments
    ///
    /// * `change_type` - Type of change (feat, fix, etc.)
    /// * `description` - Description of the change
    /// * `breaking` - Whether this is a breaking change
    /// * `commit` - Optional commit hash
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let entry = ChangeEntry::new(
    ///     "feat",
    ///     "Add OAuth2 authentication",
    ///     false,
    ///     Some("abc123"),
    /// );
    ///
    /// assert_eq!(entry.change_type, "feat");
    /// assert_eq!(entry.description, "Add OAuth2 authentication");
    /// assert!(!entry.breaking);
    /// assert_eq!(entry.commit, Some("abc123".to_string()));
    /// ```
    #[must_use]
    pub fn new(
        change_type: impl Into<String>,
        description: impl Into<String>,
        breaking: bool,
        commit: Option<impl Into<String>>,
    ) -> Self {
        Self {
            change_type: change_type.into(),
            description: description.into(),
            breaking,
            commit: commit.map(Into::into),
        }
    }

    /// Creates a new feature change entry.
    ///
    /// # Arguments
    ///
    /// * `description` - Description of the feature
    /// * `breaking` - Whether this is a breaking change
    /// * `commit` - Optional commit hash
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let feature = ChangeEntry::feature("Add user profiles", false, Some("abc123"));
    /// assert_eq!(feature.change_type, "feat");
    /// assert!(feature.is_feature());
    /// ```
    #[must_use]
    pub fn feature(
        description: impl Into<String>,
        breaking: bool,
        commit: Option<impl Into<String>>,
    ) -> Self {
        Self::new("feat", description, breaking, commit)
    }

    /// Creates a new bug fix change entry.
    ///
    /// # Arguments
    ///
    /// * `description` - Description of the fix
    /// * `commit` - Optional commit hash
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let fix = ChangeEntry::fix("Fix memory leak", Some("def456"));
    /// assert_eq!(fix.change_type, "fix");
    /// assert!(fix.is_fix());
    /// assert!(!fix.is_breaking());
    /// ```
    #[must_use]
    pub fn fix(description: impl Into<String>, commit: Option<impl Into<String>>) -> Self {
        Self::new("fix", description, false, commit)
    }

    /// Creates a new breaking change entry.
    ///
    /// # Arguments
    ///
    /// * `change_type` - Type of change
    /// * `description` - Description of the change
    /// * `commit` - Optional commit hash
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let breaking = ChangeEntry::breaking("feat", "Remove old API", Some("ghi789"));
    /// assert!(breaking.is_breaking());
    /// assert!(breaking.is_feature());
    /// ```
    #[must_use]
    pub fn breaking(
        change_type: impl Into<String>,
        description: impl Into<String>,
        commit: Option<impl Into<String>>,
    ) -> Self {
        Self::new(change_type, description, true, commit)
    }

    /// Checks if this is a breaking change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let breaking = ChangeEntry::breaking("feat", "Remove API", None::<String>);
    /// assert!(breaking.is_breaking());
    ///
    /// let normal = ChangeEntry::feature("Add feature", false, None::<String>);
    /// assert!(!normal.is_breaking());
    /// ```
    #[must_use]
    pub fn is_breaking(&self) -> bool {
        self.breaking
    }

    /// Checks if this is a feature change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let feature = ChangeEntry::feature("Add feature", false, None::<String>);
    /// assert!(feature.is_feature());
    ///
    /// let fix = ChangeEntry::fix("Fix bug", None::<String>);
    /// assert!(!fix.is_feature());
    /// ```
    #[must_use]
    pub fn is_feature(&self) -> bool {
        self.change_type == "feat"
    }

    /// Checks if this is a bug fix change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let fix = ChangeEntry::fix("Fix bug", None::<String>);
    /// assert!(fix.is_fix());
    ///
    /// let feature = ChangeEntry::feature("Add feature", false, None::<String>);
    /// assert!(!feature.is_fix());
    /// ```
    #[must_use]
    pub fn is_fix(&self) -> bool {
        self.change_type == "fix"
    }

    /// Checks if this is a documentation change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let docs = ChangeEntry::new("docs", "Update README", false, None::<String>);
    /// assert!(docs.is_docs());
    /// ```
    #[must_use]
    pub fn is_docs(&self) -> bool {
        self.change_type == "docs"
    }

    /// Checks if this is a refactoring change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let refactor = ChangeEntry::new("refactor", "Extract utility function", false, None::<String>);
    /// assert!(refactor.is_refactor());
    /// ```
    #[must_use]
    pub fn is_refactor(&self) -> bool {
        self.change_type == "refactor"
    }

    /// Checks if this is a performance improvement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let perf = ChangeEntry::new("perf", "Optimize database queries", false, None::<String>);
    /// assert!(perf.is_performance());
    /// ```
    #[must_use]
    pub fn is_performance(&self) -> bool {
        self.change_type == "perf"
    }

    /// Checks if this is a test-related change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let test = ChangeEntry::new("test", "Add integration tests", false, None::<String>);
    /// assert!(test.is_test());
    /// ```
    #[must_use]
    pub fn is_test(&self) -> bool {
        self.change_type == "test"
    }

    /// Checks if this is a build system change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let build = ChangeEntry::new("build", "Update webpack config", false, None::<String>);
    /// assert!(build.is_build());
    /// ```
    #[must_use]
    pub fn is_build(&self) -> bool {
        self.change_type == "build"
    }

    /// Checks if this is a CI/CD change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let ci = ChangeEntry::new("ci", "Add GitHub Actions workflow", false, None::<String>);
    /// assert!(ci.is_ci());
    /// ```
    #[must_use]
    pub fn is_ci(&self) -> bool {
        self.change_type == "ci"
    }

    /// Checks if this is a maintenance/chore change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let chore = ChangeEntry::new("chore", "Update dependencies", false, None::<String>);
    /// assert!(chore.is_chore());
    /// ```
    #[must_use]
    pub fn is_chore(&self) -> bool {
        self.change_type == "chore"
    }

    /// Validates this change entry.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with validation error if invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let valid = ChangeEntry::feature("Add feature", false, None::<String>);
    /// assert!(valid.validate().is_ok());
    ///
    /// let invalid = ChangeEntry::new("", "", false, None::<String>);
    /// assert!(invalid.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.change_type.trim().is_empty() {
            return Err("Change type cannot be empty".to_string());
        }

        if self.description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        // Validate commit hash format if present
        if let Some(commit) = &self.commit {
            if commit.trim().is_empty() {
                return Err("Commit hash cannot be empty if specified".to_string());
            }
            // Basic git hash validation (hexadecimal, reasonable length)
            if !commit.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err("Commit hash must be hexadecimal".to_string());
            }
            if commit.len() < 7 || commit.len() > 40 {
                return Err("Commit hash must be between 7 and 40 characters".to_string());
            }
        }

        Ok(())
    }

    /// Gets a formatted string representation suitable for changelogs.
    ///
    /// # Arguments
    ///
    /// * `include_type` - Whether to include the change type
    /// * `include_commit` - Whether to include commit hash if available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ChangeEntry;
    ///
    /// let entry = ChangeEntry::feature("Add OAuth2 support", false, Some("abc123"));
    ///
    /// assert_eq!(entry.format_for_changelog(true, false), "feat: Add OAuth2 support");
    /// assert_eq!(entry.format_for_changelog(false, false), "Add OAuth2 support");
    /// assert_eq!(entry.format_for_changelog(true, true), "feat: Add OAuth2 support (abc123)");
    /// ```
    #[must_use]
    pub fn format_for_changelog(&self, include_type: bool, include_commit: bool) -> String {
        let mut result = String::new();

        if include_type {
            result.push_str(&self.change_type);
            result.push_str(": ");
        }

        result.push_str(&self.description);

        if include_commit {
            if let Some(commit) = &self.commit {
                result.push_str(&format!(" ({})", commit));
            }
        }

        if self.breaking {
            result.push_str(" **BREAKING CHANGE**");
        }

        result
    }
}
