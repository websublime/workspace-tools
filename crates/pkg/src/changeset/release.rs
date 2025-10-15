use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Release information added when changeset is applied.
///
/// This structure tracks the metadata associated with applying a changeset,
/// including when and by whom it was applied, the Git commit where it was
/// applied, and which environments received releases.
///
/// # Lifecycle
///
/// 1. **Changeset Creation**: Initially `None` in the changeset
/// 2. **Changeset Application**: Created and populated when changeset is applied
/// 3. **Archive**: Preserved when changeset is moved to history
///
/// # Examples
///
/// ## Single Environment Release
///
/// ```rust
/// use sublime_pkg_tools::changeset::{ReleaseInfo, EnvironmentRelease};
/// use std::collections::HashMap;
/// use chrono::Utc;
///
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
///     git_commit: "abc123def456".to_string(),
///     environments_released: environments,
/// };
///
/// assert_eq!(release_info.applied_by, "ci-bot");
/// assert!(release_info.has_environment("dev"));
/// assert_eq!(release_info.get_environment_count(), 1);
/// ```
///
/// ## Multi-Environment Release
///
/// ```rust
/// use sublime_pkg_tools::changeset::{ReleaseInfo, EnvironmentRelease};
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let mut release_info = ReleaseInfo::new(
///     "deploy-bot".to_string(),
///     "def456abc789".to_string(),
/// );
///
/// release_info.add_environment_release(
///     "staging".to_string(),
///     "v1.3.0-staging".to_string(),
/// );
///
/// release_info.add_environment_release(
///     "prod".to_string(),
///     "v1.3.0".to_string(),
/// );
///
/// assert_eq!(release_info.get_environment_count(), 2);
/// assert!(release_info.has_environment("staging"));
/// assert!(release_info.has_environment("prod"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseInfo {
    /// When the changeset was applied
    ///
    /// Timestamp of when the changeset was applied and version bumps
    /// were written to package.json files.
    pub applied_at: DateTime<Utc>,

    /// Who applied the changeset
    ///
    /// Username, email, or identifier of the person or system that
    /// applied the changeset. Could be a developer, CI/CD system, or bot.
    pub applied_by: String,

    /// Git commit where it was applied
    ///
    /// Hash of the Git commit where the version bumps were applied.
    /// This is typically the merge commit on the main branch.
    pub git_commit: String,

    /// Environment-specific release information
    ///
    /// Map of environment names to their release details, including
    /// when they were released and what Git tag was created.
    pub environments_released: HashMap<String, EnvironmentRelease>,
}

impl ReleaseInfo {
    /// Creates a new release info.
    ///
    /// # Arguments
    ///
    /// * `applied_by` - Who applied the changeset
    /// * `git_commit` - Git commit hash where applied
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let release_info = ReleaseInfo::new(
    ///     "developer@example.com".to_string(),
    ///     "abc123def456".to_string(),
    /// );
    ///
    /// assert_eq!(release_info.applied_by, "developer@example.com");
    /// assert_eq!(release_info.git_commit, "abc123def456");
    /// assert_eq!(release_info.get_environment_count(), 0);
    /// ```
    #[must_use]
    pub fn new(applied_by: String, git_commit: String) -> Self {
        Self {
            applied_at: Utc::now(),
            applied_by,
            git_commit,
            environments_released: HashMap::new(),
        }
    }

    /// Creates a new release info with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `applied_at` - When the changeset was applied
    /// * `applied_by` - Who applied the changeset
    /// * `git_commit` - Git commit hash where applied
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    /// use chrono::{Utc, TimeZone};
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    /// let release_info = ReleaseInfo::new_with_timestamp(
    ///     timestamp,
    ///     "ci-bot".to_string(),
    ///     "def456abc789".to_string(),
    /// );
    ///
    /// assert_eq!(release_info.applied_at, timestamp);
    /// ```
    #[must_use]
    pub fn new_with_timestamp(
        applied_at: DateTime<Utc>,
        applied_by: String,
        git_commit: String,
    ) -> Self {
        Self { applied_at, applied_by, git_commit, environments_released: HashMap::new() }
    }

    /// Adds an environment release.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    /// * `tag` - Git tag created for this release
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release(
    ///     "production".to_string(),
    ///     "v1.2.3".to_string(),
    /// );
    ///
    /// assert!(release_info.has_environment("production"));
    /// assert_eq!(release_info.get_tag_for_environment("production"), Some("v1.2.3"));
    /// ```
    pub fn add_environment_release(&mut self, environment: String, tag: String) {
        let env_release = EnvironmentRelease { released_at: Utc::now(), tag };
        self.environments_released.insert(environment, env_release);
    }

    /// Adds an environment release with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    /// * `tag` - Git tag created for this release
    /// * `released_at` - When the release happened
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    /// use chrono::{Utc, TimeZone};
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// let release_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
    /// release_info.add_environment_release_with_timestamp(
    ///     "staging".to_string(),
    ///     "v1.2.3-staging".to_string(),
    ///     release_time,
    /// );
    ///
    /// let env_release = release_info.get_environment_release("staging").unwrap();
    /// assert_eq!(env_release.released_at, release_time);
    /// ```
    pub fn add_environment_release_with_timestamp(
        &mut self,
        environment: String,
        tag: String,
        released_at: DateTime<Utc>,
    ) {
        let env_release = EnvironmentRelease { released_at, tag };
        self.environments_released.insert(environment, env_release);
    }

    /// Checks if an environment was released.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    ///
    /// assert!(release_info.has_environment("dev"));
    /// assert!(!release_info.has_environment("prod"));
    /// ```
    #[must_use]
    pub fn has_environment(&self, environment: &str) -> bool {
        self.environments_released.contains_key(environment)
    }

    /// Gets the number of environments that were released.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// assert_eq!(release_info.get_environment_count(), 0);
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    /// release_info.add_environment_release("staging".to_string(), "v1.0.0-staging".to_string());
    ///
    /// assert_eq!(release_info.get_environment_count(), 2);
    /// ```
    #[must_use]
    pub fn get_environment_count(&self) -> usize {
        self.environments_released.len()
    }

    /// Gets the list of environment names that were released.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    /// release_info.add_environment_release("prod".to_string(), "v1.0.0".to_string());
    ///
    /// let environments = release_info.get_environment_names();
    /// assert_eq!(environments.len(), 2);
    /// assert!(environments.contains(&"dev".to_string()));
    /// assert!(environments.contains(&"prod".to_string()));
    /// ```
    #[must_use]
    pub fn get_environment_names(&self) -> Vec<String> {
        self.environments_released.keys().cloned().collect()
    }

    /// Gets the environment release information for a specific environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    ///
    /// # Returns
    ///
    /// Reference to the environment release if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    ///
    /// let env_release = release_info.get_environment_release("dev");
    /// assert!(env_release.is_some());
    /// assert_eq!(env_release.unwrap().tag, "v1.0.0-dev");
    ///
    /// let missing = release_info.get_environment_release("prod");
    /// assert!(missing.is_none());
    /// ```
    #[must_use]
    pub fn get_environment_release(&self, environment: &str) -> Option<&EnvironmentRelease> {
        self.environments_released.get(environment)
    }

    /// Gets the Git tag for a specific environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    ///
    /// # Returns
    ///
    /// Git tag string if environment was released, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("staging".to_string(), "v1.2.3-staging".to_string());
    ///
    /// assert_eq!(release_info.get_tag_for_environment("staging"), Some("v1.2.3-staging"));
    /// assert_eq!(release_info.get_tag_for_environment("prod"), None);
    /// ```
    #[must_use]
    pub fn get_tag_for_environment(&self, environment: &str) -> Option<&str> {
        self.environments_released.get(environment).map(|env| env.tag.as_str())
    }

    /// Gets the release timestamp for a specific environment.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment name
    ///
    /// # Returns
    ///
    /// Release timestamp if environment was released, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    ///
    /// let timestamp = release_info.get_release_timestamp_for_environment("dev");
    /// assert!(timestamp.is_some());
    ///
    /// let missing = release_info.get_release_timestamp_for_environment("prod");
    /// assert!(missing.is_none());
    /// ```
    #[must_use]
    pub fn get_release_timestamp_for_environment(
        &self,
        environment: &str,
    ) -> Option<DateTime<Utc>> {
        self.environments_released.get(environment).map(|env| env.released_at)
    }

    /// Removes an environment release.
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
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    /// assert!(release_info.has_environment("dev"));
    ///
    /// assert!(release_info.remove_environment("dev"));
    /// assert!(!release_info.has_environment("dev"));
    ///
    /// assert!(!release_info.remove_environment("non-existent"));
    /// ```
    pub fn remove_environment(&mut self, environment: &str) -> bool {
        self.environments_released.remove(environment).is_some()
    }

    /// Validates the release info structure.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with validation error if invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let valid_release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123def456".to_string(),
    /// );
    /// assert!(valid_release_info.validate().is_ok());
    ///
    /// let invalid_release_info = ReleaseInfo::new(
    ///     "".to_string(),
    ///     "abc123".to_string(),
    /// );
    /// assert!(invalid_release_info.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.applied_by.trim().is_empty() {
            return Err("Applied by cannot be empty".to_string());
        }

        if self.git_commit.trim().is_empty() {
            return Err("Git commit cannot be empty".to_string());
        }

        // Basic git hash validation
        if !self.git_commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Git commit must be hexadecimal".to_string());
        }

        if self.git_commit.len() < 7 || self.git_commit.len() > 40 {
            return Err("Git commit must be between 7 and 40 characters".to_string());
        }

        // Validate environment releases
        for (env_name, env_release) in &self.environments_released {
            if env_name.trim().is_empty() {
                return Err("Environment name cannot be empty".to_string());
            }

            if let Err(e) = env_release.validate() {
                return Err(format!("Environment '{}': {}", env_name, e));
            }
        }

        Ok(())
    }

    /// Gets a summary of all tags created for this release.
    ///
    /// # Returns
    ///
    /// Vector of (environment, tag) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let mut release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// release_info.add_environment_release("dev".to_string(), "v1.0.0-dev".to_string());
    /// release_info.add_environment_release("prod".to_string(), "v1.0.0".to_string());
    ///
    /// let tags = release_info.get_tags_summary();
    /// assert_eq!(tags.len(), 2);
    /// assert!(tags.contains(&("dev".to_string(), "v1.0.0-dev".to_string())));
    /// assert!(tags.contains(&("prod".to_string(), "v1.0.0".to_string())));
    /// ```
    #[must_use]
    pub fn get_tags_summary(&self) -> Vec<(String, String)> {
        self.environments_released
            .iter()
            .map(|(env, release)| (env.clone(), release.tag.clone()))
            .collect()
    }

    /// Checks if this release was applied today.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::ReleaseInfo;
    ///
    /// let release_info = ReleaseInfo::new(
    ///     "deploy-bot".to_string(),
    ///     "abc123".to_string(),
    /// );
    ///
    /// // Since we just created it, it should be from today
    /// assert!(release_info.was_applied_today());
    /// ```
    #[must_use]
    pub fn was_applied_today(&self) -> bool {
        let now = Utc::now();
        self.applied_at.date_naive() == now.date_naive()
    }
}

/// Environment-specific release information.
///
/// Tracks when a package was released to a specific environment and
/// what Git tag was created for that release.
///
/// # Examples
///
/// ## Production Release
///
/// ```rust
/// use sublime_pkg_tools::changeset::EnvironmentRelease;
/// use chrono::Utc;
///
/// let prod_release = EnvironmentRelease {
///     released_at: Utc::now(),
///     tag: "v1.2.3".to_string(),
/// };
///
/// assert_eq!(prod_release.tag, "v1.2.3");
/// assert!(prod_release.was_released_today());
/// ```
///
/// ## Development Release
///
/// ```rust
/// use sublime_pkg_tools::changeset::EnvironmentRelease;
/// use chrono::Utc;
///
/// let dev_release = EnvironmentRelease::new("v1.2.3-dev".to_string());
/// assert_eq!(dev_release.tag, "v1.2.3-dev");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentRelease {
    /// When released to this environment
    ///
    /// Timestamp of when the package was actually deployed/released
    /// to this specific environment.
    pub released_at: DateTime<Utc>,

    /// Git tag created for this release
    ///
    /// The Git tag that was created and pushed for this environment.
    /// Typically follows patterns like:
    /// - Production: "v1.2.3"
    /// - Staging: "v1.2.3-staging"
    /// - Development: "v1.2.3-dev"
    pub tag: String,
}

impl EnvironmentRelease {
    /// Creates a new environment release with current timestamp.
    ///
    /// # Arguments
    ///
    /// * `tag` - Git tag for this release
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let release = EnvironmentRelease::new("v1.2.3-staging".to_string());
    /// assert_eq!(release.tag, "v1.2.3-staging");
    /// ```
    #[must_use]
    pub fn new(tag: String) -> Self {
        Self { released_at: Utc::now(), tag }
    }

    /// Creates a new environment release with specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `released_at` - When the release happened
    /// * `tag` - Git tag for this release
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    /// use chrono::{Utc, TimeZone};
    ///
    /// let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    /// let release = EnvironmentRelease::new_with_timestamp(
    ///     timestamp,
    ///     "v1.2.3".to_string(),
    /// );
    ///
    /// assert_eq!(release.released_at, timestamp);
    /// assert_eq!(release.tag, "v1.2.3");
    /// ```
    #[must_use]
    pub fn new_with_timestamp(released_at: DateTime<Utc>, tag: String) -> Self {
        Self { released_at, tag }
    }

    /// Validates the environment release.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(String)` with validation error if invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let valid_release = EnvironmentRelease::new("v1.2.3".to_string());
    /// assert!(valid_release.validate().is_ok());
    ///
    /// let invalid_release = EnvironmentRelease::new("".to_string());
    /// assert!(invalid_release.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if self.tag.trim().is_empty() {
            return Err("Tag cannot be empty".to_string());
        }

        // Basic tag format validation (should start with 'v' for semantic versions)
        if !self.tag.starts_with('v') {
            return Err("Tag should start with 'v' (e.g., v1.2.3)".to_string());
        }

        Ok(())
    }

    /// Checks if this release was made today.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let release = EnvironmentRelease::new("v1.2.3".to_string());
    /// // Since we just created it, it should be from today
    /// assert!(release.was_released_today());
    /// ```
    #[must_use]
    pub fn was_released_today(&self) -> bool {
        let now = Utc::now();
        self.released_at.date_naive() == now.date_naive()
    }

    /// Gets the semantic version from the tag.
    ///
    /// Extracts the version part from tags like "v1.2.3" or "v1.2.3-staging".
    ///
    /// # Returns
    ///
    /// The version string without the 'v' prefix and environment suffix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let prod_release = EnvironmentRelease::new("v1.2.3".to_string());
    /// assert_eq!(prod_release.get_version(), "1.2.3");
    ///
    /// let staging_release = EnvironmentRelease::new("v1.2.3-staging".to_string());
    /// assert_eq!(staging_release.get_version(), "1.2.3-staging");
    /// ```
    #[must_use]
    pub fn get_version(&self) -> &str {
        self.tag.strip_prefix('v').unwrap_or(&self.tag)
    }

    /// Checks if this is a prerelease tag.
    ///
    /// Determines if the tag contains a prerelease identifier (e.g., -dev, -staging, -alpha).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let prod_release = EnvironmentRelease::new("v1.2.3".to_string());
    /// assert!(!prod_release.is_prerelease());
    ///
    /// let staging_release = EnvironmentRelease::new("v1.2.3-staging".to_string());
    /// assert!(staging_release.is_prerelease());
    ///
    /// let dev_release = EnvironmentRelease::new("v1.2.3-dev".to_string());
    /// assert!(dev_release.is_prerelease());
    /// ```
    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        let version = self.get_version();
        version.contains('-')
    }

    /// Gets the environment identifier from the tag.
    ///
    /// Extracts the environment part from tags like "v1.2.3-staging" or "v1.2.3-dev".
    ///
    /// # Returns
    ///
    /// The environment identifier if present, `None` for production releases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sublime_pkg_tools::changeset::EnvironmentRelease;
    ///
    /// let prod_release = EnvironmentRelease::new("v1.2.3".to_string());
    /// assert_eq!(prod_release.get_environment_identifier(), None);
    ///
    /// let staging_release = EnvironmentRelease::new("v1.2.3-staging".to_string());
    /// assert_eq!(staging_release.get_environment_identifier(), Some("staging"));
    ///
    /// let dev_release = EnvironmentRelease::new("v1.2.3-dev".to_string());
    /// assert_eq!(dev_release.get_environment_identifier(), Some("dev"));
    /// ```
    #[must_use]
    pub fn get_environment_identifier(&self) -> Option<&str> {
        let version = self.get_version();
        if let Some(dash_pos) = version.find('-') {
            Some(&version[dash_pos + 1..])
        } else {
            None
        }
    }
}
