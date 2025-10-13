use std::{fmt, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use sublime_git_tools::Repo;
use sublime_standard_tools::{
    filesystem::AsyncFileSystem,
    monorepo::{MonorepoDetector, MonorepoDetectorTrait},
};

use crate::{
    config::PackageToolsConfig,
    error::{PackageResult, VersionError},
    version::{
        snapshot::SnapshotVersion,
        versioning::{Version, VersionComparison},
    },
};

/// Resolved version union type.
///
/// Represents either a release version (from package.json) or
/// a snapshot version (calculated for development branches).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolvedVersion {
    /// Release version from package.json
    Release(Version),
    /// Snapshot version for development
    Snapshot(SnapshotVersion),
}

impl fmt::Display for ResolvedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Release(version) => write!(f, "{}", version),
            Self::Snapshot(snapshot) => write!(f, "{}", snapshot),
        }
    }
}

impl From<Version> for ResolvedVersion {
    fn from(version: Version) -> Self {
        Self::Release(version)
    }
}

impl From<SnapshotVersion> for ResolvedVersion {
    fn from(snapshot: SnapshotVersion) -> Self {
        Self::Snapshot(snapshot)
    }
}

impl ResolvedVersion {
    /// Checks if this is a release version.
    #[must_use]
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release(_))
    }

    /// Checks if this is a snapshot version.
    #[must_use]
    pub fn is_snapshot(&self) -> bool {
        matches!(self, Self::Snapshot(_))
    }

    /// Gets the base version (release version or snapshot base).
    #[must_use]
    pub fn base_version(&self) -> &Version {
        match self {
            Self::Release(version) => version,
            Self::Snapshot(snapshot) => &snapshot.base_version,
        }
    }

    /// Converts to release version if possible.
    #[must_use]
    pub fn as_release(&self) -> Option<&Version> {
        match self {
            Self::Release(version) => Some(version),
            Self::Snapshot(_) => None,
        }
    }

    /// Converts to snapshot version if possible.
    #[must_use]
    pub fn as_snapshot(&self) -> Option<&SnapshotVersion> {
        match self {
            Self::Release(_) => None,
            Self::Snapshot(snapshot) => Some(snapshot),
        }
    }

    /// Compares this resolved version with another.
    ///
    /// # Arguments
    ///
    /// * `other` - The resolved version to compare against
    #[must_use]
    pub fn compare(&self, other: &Self) -> VersionComparison {
        match (self, other) {
            (Self::Release(a), Self::Release(b)) => a.compare(b),
            (Self::Snapshot(a), Self::Snapshot(b)) => a.compare(b),
            (Self::Release(release), Self::Snapshot(snapshot))
            | (Self::Snapshot(snapshot), Self::Release(release)) => {
                match release.compare(&snapshot.base_version) {
                    VersionComparison::Equal => VersionComparison::Incomparable,
                    other => other,
                }
            }
        }
    }
}

/// Version resolver service for determining package versions.
///
/// The VersionResolver is responsible for determining the current version
/// of packages, whether they should use snapshot versions (on development
/// branches) or release versions (from package.json on main branch).
///
/// # Examples
///
/// ```ignore
/// use sublime_pkg_tools::version::VersionResolver;
/// use sublime_standard_tools::filesystem::FileSystemManager;
/// use sublime_git_tools::Repo;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fs = FileSystemManager::new();
/// let repo = Repo::open(".")?;
/// let config = Default::default();
///
/// let resolver = VersionResolver::new(fs, repo, config);
/// let version = resolver.resolve_current_version(Path::new("packages/my-pkg")).await?;
///
/// println!("Current version: {}", version);
/// # Ok(())
/// # }
/// ```
pub struct VersionResolver<F>
where
    F: AsyncFileSystem + Send + Sync + Clone,
{
    filesystem: F,
    repo: Repo,
    config: PackageToolsConfig,
    monorepo_detector: MonorepoDetector<F>,
}

impl<F> VersionResolver<F>
where
    F: AsyncFileSystem + Send + Sync + Clone,
{
    /// Creates a new version resolver.
    ///
    /// # Arguments
    ///
    /// * `filesystem` - Filesystem implementation for reading package.json files
    /// * `repo` - Git repository for branch and commit information
    /// * `config` - Package tools configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use sublime_pkg_tools::version::VersionResolver;
    /// use sublime_standard_tools::filesystem::FileSystemManager;
    /// use sublime_git_tools::Repo;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs = FileSystemManager::new();
    /// let repo = Repo::open(".")?;
    /// let config = Default::default();
    ///
    /// let resolver = VersionResolver::new(fs, repo, config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(filesystem: F, repo: Repo, config: PackageToolsConfig) -> Self {
        let monorepo_detector = MonorepoDetector::with_filesystem(filesystem.clone());
        Self { filesystem, repo, config, monorepo_detector }
    }

    /// Resolves the current version for a package.
    ///
    /// This method determines whether to return a release version (from package.json)
    /// or a snapshot version (calculated for development branches). The decision
    /// is based on the current Git branch and configuration settings.
    ///
    /// # Arguments
    ///
    /// * `package_path` - Path to the package directory
    ///
    /// # Returns
    ///
    /// The resolved version (either Release or Snapshot)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - package.json cannot be read
    /// - Version in package.json is invalid
    /// - Git operations fail
    /// - Snapshot version calculation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # use std::path::Path;
    /// # async fn example(resolver: VersionResolver<impl AsyncFileSystem + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> {
    /// let version = resolver.resolve_current_version(Path::new("packages/auth")).await?;
    ///
    /// match version {
    ///     ResolvedVersion::Release(v) => println!("Release version: {}", v),
    ///     ResolvedVersion::Snapshot(s) => println!("Snapshot version: {}", s),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_current_version(
        &self,
        package_path: &Path,
    ) -> PackageResult<ResolvedVersion> {
        // Read current version from package.json
        let package_version = self.read_package_version(package_path).await?;

        // Check if we're on main branch or should use snapshots
        if self.should_use_snapshot().await? {
            self.create_snapshot_version(package_version).await
        } else {
            Ok(ResolvedVersion::Release(package_version))
        }
    }

    /// Resolves the version for a specific package by name.
    ///
    /// This method searches for the package in both single repositories and monorepos.
    /// It automatically detects the repository type and uses the appropriate search strategy:
    ///
    /// - **Single Repository**: Checks if the package.json at the root matches the requested name
    /// - **Monorepo**: Uses MonorepoDetector to find packages in the workspace structure
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to resolve
    /// * `workspace_root` - Root path of the repository (single repo or monorepo root)
    ///
    /// # Returns
    ///
    /// The resolved version for the named package
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package is not found in single repo or monorepo
    /// - Repository type cannot be determined
    /// - Version resolution fails
    /// - Package.json cannot be read or parsed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # use std::path::Path;
    /// # async fn example(resolver: VersionResolver<impl AsyncFileSystem + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> {
    /// // Works in both single repo and monorepo
    /// let version = resolver.resolve_package_version(
    ///     "@myorg/auth-service",
    ///     Path::new(".")
    /// ).await?;
    ///
    /// println!("Package version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve_package_version(
        &self,
        package_name: &str,
        workspace_root: &Path,
    ) -> PackageResult<ResolvedVersion> {
        let package_path = self.find_package_path(package_name, workspace_root).await?;
        self.resolve_current_version(&package_path).await
    }

    /// Creates a snapshot version for development branches.
    ///
    /// Snapshot versions include the base version from package.json plus
    /// the current commit hash, providing unique identifiers for each
    /// commit on development branches.
    ///
    /// # Arguments
    ///
    /// * `base_version` - The base version from package.json
    ///
    /// # Returns
    ///
    /// A resolved snapshot version
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail or commit hash cannot be determined
    pub async fn create_snapshot_version(
        &self,
        base_version: Version,
    ) -> PackageResult<ResolvedVersion> {
        let commit_hash = self.get_current_commit_hash().await?;
        let shortened_hash = self.shorten_commit_hash(&commit_hash);

        let snapshot = SnapshotVersion::new(base_version, shortened_hash);
        Ok(ResolvedVersion::Snapshot(snapshot))
    }

    /// Checks if the resolver is configured to use snapshots in the current context.
    ///
    /// # Returns
    ///
    /// True if snapshots should be used based on current branch and configuration
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: VersionResolver<impl AsyncFileSystem + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> {
    /// if resolver.should_use_snapshot().await? {
    ///     println!("Using snapshot versions");
    /// } else {
    ///     println!("Using release versions");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn should_use_snapshot(&self) -> PackageResult<bool> {
        let current_branch =
            self.repo.get_current_branch().map_err(|e| VersionError::SnapshotResolutionFailed {
                package: "repository".to_string(),
                reason: format!("Failed to get current branch: {}", e),
            })?;

        // Check if we're on main branch
        let is_main_branch = matches!(current_branch.as_str(), "main" | "master");

        if is_main_branch {
            // Check configuration to see if snapshots are allowed on main
            Ok(self.config.version.allow_snapshot_on_main)
        } else {
            // Always use snapshots on non-main branches
            Ok(true)
        }
    }

    /// Gets the current Git branch name.
    ///
    /// # Returns
    ///
    /// The name of the current Git branch
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: VersionResolver<impl AsyncFileSystem + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> {
    /// let branch = resolver.get_current_branch().await?;
    /// println!("Current branch: {}", branch);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_branch(&self) -> PackageResult<String> {
        self.repo.get_current_branch().map_err(|e| {
            VersionError::SnapshotResolutionFailed {
                package: "repository".to_string(),
                reason: format!("Failed to get current branch: {}", e),
            }
            .into()
        })
    }

    /// Gets the current commit hash.
    ///
    /// # Returns
    ///
    /// The full commit hash of the current HEAD
    ///
    /// # Errors
    ///
    /// Returns an error if Git operations fail
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use sublime_pkg_tools::version::VersionResolver;
    /// # async fn example(resolver: VersionResolver<impl AsyncFileSystem + Send + Sync>) -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = resolver.get_current_commit_hash().await?;
    /// println!("Current commit: {}", hash);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_commit_hash(&self) -> PackageResult<String> {
        self.repo.get_current_sha().map_err(|e| {
            VersionError::SnapshotResolutionFailed {
                package: "repository".to_string(),
                reason: format!("Failed to get current commit hash: {}", e),
            }
            .into()
        })
    }

    /// Reads the version from a package.json file.
    ///
    /// # Arguments
    ///
    /// * `package_path` - Path to the package directory
    ///
    /// # Returns
    ///
    /// The version parsed from package.json
    ///
    /// # Errors
    ///
    /// Returns an error if package.json cannot be read or version is invalid
    async fn read_package_version(&self, package_path: &Path) -> PackageResult<Version> {
        let package_json_path = package_path.join("package.json");

        let content = self.filesystem.read_file_string(&package_json_path).await.map_err(|e| {
            VersionError::SnapshotResolutionFailed {
                package: package_path.display().to_string(),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let version_str = package_json["version"].as_str().ok_or_else(|| {
            VersionError::SnapshotResolutionFailed {
                package: package_path.display().to_string(),
                reason: "No version field found in package.json".to_string(),
            }
        })?;

        Version::from_str(version_str).map_err(|e| {
            VersionError::SnapshotResolutionFailed {
                package: package_path.display().to_string(),
                reason: format!("Invalid version in package.json: {}", e),
            }
            .into()
        })
    }

    /// Shortens a commit hash according to configuration.
    ///
    /// # Arguments
    ///
    /// * `full_hash` - The full commit hash
    ///
    /// # Returns
    ///
    /// The shortened commit hash
    fn shorten_commit_hash(&self, full_hash: &str) -> String {
        let length = self.config.version.commit_hash_length as usize;
        if full_hash.len() <= length {
            full_hash.to_string()
        } else {
            full_hash.chars().take(length).collect()
        }
    }

    /// Finds the path for a package by name in a workspace or single repository.
    ///
    /// Handles both single repository and monorepo scenarios:
    /// - Single repo: Searches for package.json in the workspace root
    /// - Monorepo: Uses MonorepoDetector to analyze workspace structure
    ///
    /// # Arguments
    ///
    /// * `package_name` - Name of the package to find
    /// * `workspace_root` - Root path of the workspace
    ///
    /// # Returns
    ///
    /// The path to the package directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Package is not found in single repo or monorepo
    /// - Monorepo detection or analysis fails
    /// - Package.json cannot be read or parsed
    async fn find_package_path(
        &self,
        package_name: &str,
        workspace_root: &Path,
    ) -> PackageResult<std::path::PathBuf> {
        // First, check if this is a monorepo
        let monorepo_kind =
            self.monorepo_detector.is_monorepo_root(workspace_root).await.map_err(|e| {
                VersionError::SnapshotResolutionFailed {
                    package: package_name.to_string(),
                    reason: format!("Failed to detect repository type: {}", e),
                }
            })?;

        match monorepo_kind {
            Some(_) => {
                // Handle monorepo case
                self.find_package_in_monorepo(package_name, workspace_root).await
            }
            None => {
                // Handle single repository case
                self.find_package_in_single_repo(package_name, workspace_root).await
            }
        }
    }

    /// Finds a package in a monorepo structure.
    async fn find_package_in_monorepo(
        &self,
        package_name: &str,
        workspace_root: &Path,
    ) -> PackageResult<std::path::PathBuf> {
        // Detect and analyze the monorepo structure
        let monorepo =
            self.monorepo_detector.detect_monorepo(workspace_root).await.map_err(|e| {
                VersionError::SnapshotResolutionFailed {
                    package: package_name.to_string(),
                    reason: format!("Failed to analyze monorepo: {}", e),
                }
            })?;

        // Find the package by name using the analyzed structure
        if let Some(workspace_package) = monorepo.get_package(package_name) {
            Ok(workspace_package.absolute_path.clone())
        } else {
            Err(VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: format!(
                    "Package '{}' not found in {} monorepo. Available packages: {}",
                    package_name,
                    monorepo.kind().name(),
                    monorepo
                        .packages()
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
            .into())
        }
    }

    /// Finds a package in a single repository structure.
    async fn find_package_in_single_repo(
        &self,
        package_name: &str,
        workspace_root: &Path,
    ) -> PackageResult<std::path::PathBuf> {
        // In a single repo, check if the package.json at root matches the requested name
        let package_json_path = workspace_root.join("package.json");

        if !self.filesystem.exists(&package_json_path).await {
            return Err(VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: format!(
                    "No package.json found at repository root: {}",
                    workspace_root.display()
                ),
            }
            .into());
        }

        // Read and parse package.json
        let content = self.filesystem.read_file_string(&package_json_path).await.map_err(|e| {
            VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: format!("Failed to read package.json: {}", e),
            }
        })?;

        let package_json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: format!("Failed to parse package.json: {}", e),
            })?;

        let actual_name = package_json["name"].as_str().ok_or_else(|| {
            VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: "No name field found in package.json".to_string(),
            }
        })?;

        if actual_name == package_name {
            Ok(workspace_root.to_path_buf())
        } else {
            Err(VersionError::SnapshotResolutionFailed {
                package: package_name.to_string(),
                reason: format!(
                    "Package name mismatch in single repository. Expected: '{}', Found: '{}'",
                    package_name, actual_name
                ),
            }
            .into())
        }
    }
}
