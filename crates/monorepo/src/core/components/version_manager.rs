//! Package version management component
//!
//! Handles all version-related operations including version updates,
//! snapshot versions, and version bump suggestions.

use super::super::types::{MonorepoPackageInfo, VersionStatus};
use crate::config::VersionBumpType;
use crate::error::{Error, Result};
use sublime_package_tools::Version;

/// Component for managing package version operations
#[allow(dead_code)]
pub(crate) struct PackageVersionManager {
    package: MonorepoPackageInfo,
}

#[allow(dead_code)]
impl PackageVersionManager {
    /// Create a new package version manager
    #[must_use]
    pub fn new(package: MonorepoPackageInfo) -> Self {
        Self { package }
    }

    /// Get immutable reference to the package
    #[must_use]
    pub fn package(&self) -> &MonorepoPackageInfo {
        &self.package
    }

    /// Get current version
    #[must_use]
    pub fn current_version(&self) -> &str {
        &self.package.workspace_package.version
    }

    /// Get current version status
    #[must_use]
    pub fn version_status(&self) -> &VersionStatus {
        &self.package.version_status
    }

    /// Update the package version
    ///
    /// # Arguments
    /// * `new_version` - The new version string to set
    ///
    /// # Errors
    /// Returns an error if the version format is invalid
    pub fn update_version(&mut self, new_version: &str) -> Result<()> {
        // Validate version format
        if new_version.trim().is_empty() {
            return Err(Error::package("Version cannot be empty"));
        }

        // Update in PackageInfo
        self.package
            .package_info
            .update_version(new_version)
            .map_err(|e| Error::package(format!("Failed to update package info version: {e}")))?;

        // Update in WorkspacePackage
        self.package.workspace_package.version = new_version.to_string();

        // Update version status to stable
        self.package.version_status = VersionStatus::Stable;

        Ok(())
    }

    /// Set version as snapshot
    ///
    /// # Arguments
    /// * `base_version` - The base version to use for the snapshot
    /// * `sha` - Git commit SHA for the snapshot
    ///
    /// # Errors
    /// Returns an error if the version update fails
    pub fn set_snapshot_version(&mut self, base_version: &str, sha: &str) -> Result<()> {
        if sha.len() < 7 {
            return Err(Error::package("SHA must be at least 7 characters long"));
        }

        let snapshot_version =
            format!("{base_version}-snapshot.{sha_short}", sha_short = &sha[..7]);
        self.update_version(&snapshot_version)?;
        self.package.version_status = VersionStatus::Snapshot { sha: sha.to_string() };
        Ok(())
    }

    /// Set version as pre-release
    ///
    /// # Arguments
    /// * `base_version` - The base version to use for the pre-release
    /// * `tag` - Pre-release tag (e.g., "alpha", "beta", "rc.1")
    ///
    /// # Errors
    /// Returns an error if the version update fails
    pub fn set_pre_release_version(&mut self, base_version: &str, tag: &str) -> Result<()> {
        if tag.trim().is_empty() {
            return Err(Error::package("Pre-release tag cannot be empty"));
        }

        let pre_release_version = format!("{base_version}-{tag}");
        self.update_version(&pre_release_version)?;
        self.package.version_status = VersionStatus::PreRelease { tag: tag.to_string() };
        Ok(())
    }

    /// Mark package as having dirty (uncommitted) changes
    pub fn mark_dirty(&mut self) {
        self.package.version_status = VersionStatus::Dirty;
    }

    /// Mark package as clean (committed changes)
    pub fn mark_clean(&mut self) {
        self.package.version_status = VersionStatus::Stable;
    }

    /// Bump version according to the specified bump type
    ///
    /// # Arguments
    /// * `bump_type` - Type of version bump to perform
    ///
    /// # Returns
    /// The new version string after bumping
    ///
    /// # Errors
    /// Returns an error if the version bump fails
    pub fn bump_version(&mut self, bump_type: VersionBumpType) -> Result<String> {
        let current_version = self.current_version();

        let new_version = match bump_type {
            VersionBumpType::Major => Version::bump_major(current_version)?.to_string(),
            VersionBumpType::Minor => Version::bump_minor(current_version)?.to_string(),
            VersionBumpType::Patch => Version::bump_patch(current_version)?.to_string(),
            VersionBumpType::Snapshot => {
                return Err(Error::package("Cannot bump to snapshot without SHA"));
            }
        };

        self.update_version(&new_version)?;
        Ok(new_version)
    }

    /// Bump version to snapshot
    ///
    /// # Arguments
    /// * `bump_type` - Base version bump type before adding snapshot suffix
    /// * `sha` - Git commit SHA for the snapshot
    ///
    /// # Returns
    /// The new snapshot version string
    ///
    /// # Errors
    /// Returns an error if the version bump fails
    pub fn bump_to_snapshot(&mut self, bump_type: VersionBumpType, sha: &str) -> Result<String> {
        let base_version = match bump_type {
            VersionBumpType::Major => Version::bump_major(self.current_version())?.to_string(),
            VersionBumpType::Minor => Version::bump_minor(self.current_version())?.to_string(),
            VersionBumpType::Patch => Version::bump_patch(self.current_version())?.to_string(),
            VersionBumpType::Snapshot => self.current_version().to_string(),
        };

        self.set_snapshot_version(&base_version, sha)?;
        Ok(self.current_version().to_string())
    }

    /// Get suggested version bump based on pending changesets
    #[must_use]
    pub fn suggested_version_bump(&self) -> Option<VersionBumpType> {
        let pending_changesets: Vec<_> = self
            .package
            .changesets
            .iter()
            .filter(|cs| matches!(cs.status, super::super::types::ChangesetStatus::Pending))
            .collect();

        if pending_changesets.is_empty() {
            return None;
        }

        // Find the highest priority bump
        let mut bump = VersionBumpType::Patch;
        for changeset in pending_changesets {
            match changeset.version_bump {
                VersionBumpType::Major => return Some(VersionBumpType::Major),
                VersionBumpType::Minor => bump = VersionBumpType::Minor,
                VersionBumpType::Patch | VersionBumpType::Snapshot => {}
            }
        }

        Some(bump)
    }

    /// Get the next version that would be created with the given bump type
    ///
    /// # Arguments
    /// * `bump_type` - Type of version bump to simulate
    ///
    /// # Returns
    /// The version string that would result from the bump
    ///
    /// # Errors
    /// Returns an error if the version format is invalid
    pub fn preview_version_bump(&self, bump_type: VersionBumpType) -> Result<String> {
        let current_version = self.current_version();

        match bump_type {
            VersionBumpType::Major => Ok(Version::bump_major(current_version)?.to_string()),
            VersionBumpType::Minor => Ok(Version::bump_minor(current_version)?.to_string()),
            VersionBumpType::Patch => Ok(Version::bump_patch(current_version)?.to_string()),
            VersionBumpType::Snapshot => {
                Err(Error::package("Cannot preview snapshot version without SHA"))
            }
        }
    }

    /// Check if the package version is valid semver
    #[must_use]
    pub fn is_valid_semver(&self) -> bool {
        Version::parse(self.current_version()).is_ok()
    }

    /// Get version components (major, minor, patch) if valid semver
    #[must_use]
    pub fn version_components(&self) -> Option<(u64, u64, u64)> {
        Version::parse(self.current_version()).ok().map(|v| (v.major, v.minor, v.patch))
    }

    /// Consume the manager and return the updated package
    #[must_use]
    pub fn into_package(self) -> MonorepoPackageInfo {
        self.package
    }
}
