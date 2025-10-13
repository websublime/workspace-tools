use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ResolvedVersion;

/// Plan for executing a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePlan {
    /// Changeset ID being released
    pub changeset_id: String,
    /// Target environment for release
    pub environment: String,
    /// Packages to be released
    pub packages: Vec<PackageRelease>,
    /// Git tag to be created
    pub version_tag: Option<String>,
    /// Whether to create Git tags
    pub create_tags: bool,
    /// Whether to push tags to remote
    pub push_tags: bool,
    /// Whether to create changelog
    pub create_changelog: bool,
    /// Release strategy to use
    pub strategy: ReleaseStrategy,
    /// Estimated release duration
    pub estimated_duration: Option<u64>,
}

/// Individual package release information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRelease {
    /// Package name
    pub name: String,
    /// Current version
    pub current_version: ResolvedVersion,
    /// Next version after release
    pub next_version: ResolvedVersion,
    /// Package directory path
    pub path: PathBuf,
    /// Whether to publish to registry
    pub publish: bool,
    /// Reason for release
    pub reason: String,
    /// Registry access level
    pub access: String,
    /// Dist tag for publishing
    pub tag: String,
}

/// Release strategy enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseStrategy {
    /// Independent versioning - each package maintains its own version
    Independent,
    /// Unified versioning - all packages share the same version
    Unified { version: String },
}

/// Result of a dry run execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunResult {
    /// Packages that would be updated
    pub packages: Vec<PackageRelease>,
    /// Files that would be modified
    pub files_to_modify: Vec<PathBuf>,
    /// Git tags that would be created
    pub tags_to_create: Vec<String>,
    /// Commands that would be executed
    pub commands: Vec<String>,
    /// Summary of the dry run
    pub summary: String,
    /// Estimated total duration
    pub estimated_duration: u64,
    /// Any warnings detected
    pub warnings: Vec<String>,
}

/// Release execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResult {
    /// Whether the release was successful
    pub success: bool,
    /// Packages that were successfully released
    pub released_packages: Vec<String>,
    /// Packages that failed to release
    pub failed_packages: Vec<String>,
    /// Git tags that were created
    pub created_tags: Vec<String>,
    /// When the release started
    pub started_at: DateTime<Utc>,
    /// When the release completed
    pub completed_at: DateTime<Utc>,
    /// Total duration in seconds
    pub duration: u64,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

impl Default for ReleaseStrategy {
    fn default() -> Self {
        Self::Independent
    }
}

impl ReleasePlan {
    /// Creates a new release plan.
    ///
    /// # Arguments
    ///
    /// * `changeset_id` - Changeset being released
    /// * `environment` - Target environment
    /// * `strategy` - Release strategy
    #[must_use]
    pub fn new(changeset_id: String, environment: String, strategy: ReleaseStrategy) -> Self {
        Self {
            changeset_id,
            environment,
            packages: Vec::new(),
            version_tag: None,
            create_tags: true,
            push_tags: true,
            create_changelog: true,
            strategy,
            estimated_duration: None,
        }
    }

    /// Adds a package to the release plan.
    ///
    /// # Arguments
    ///
    /// * `package` - Package release information
    pub fn add_package(&mut self, package: PackageRelease) {
        self.packages.push(package);
    }

    /// Gets the total number of packages in the plan.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Estimates the total release duration.
    #[must_use]
    pub fn estimate_duration(&self) -> u64 {
        // Simple estimation: 30 seconds per package + 60 seconds overhead
        (self.packages.len() as u64 * 30) + 60
    }
}

impl PackageRelease {
    /// Creates a new package release.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `current_version` - Current version
    /// * `next_version` - Next version
    /// * `path` - Package directory path
    #[must_use]
    pub fn new(
        name: String,
        current_version: ResolvedVersion,
        next_version: ResolvedVersion,
        path: PathBuf,
    ) -> Self {
        Self {
            name,
            current_version,
            next_version,
            path,
            publish: true,
            reason: "Version update".to_string(),
            access: "public".to_string(),
            tag: "latest".to_string(),
        }
    }

    /// Sets whether to publish this package.
    ///
    /// # Arguments
    ///
    /// * `publish` - Whether to publish
    pub fn with_publish(mut self, publish: bool) -> Self {
        self.publish = publish;
        self
    }

    /// Sets the reason for this release.
    ///
    /// # Arguments
    ///
    /// * `reason` - Release reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = reason;
        self
    }

    /// Sets the registry access level.
    ///
    /// # Arguments
    ///
    /// * `access` - Access level (public/restricted)
    pub fn with_access(mut self, access: String) -> Self {
        self.access = access;
        self
    }

    /// Sets the dist tag for publishing.
    ///
    /// # Arguments
    ///
    /// * `tag` - Dist tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag = tag;
        self
    }
}
