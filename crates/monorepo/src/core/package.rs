//! Package-specific implementations and utilities
//!
//! This module provides the main MonorepoPackageInfo implementation that acts as a facade
//! over the focused components. All complex operations are delegated to specialized components
//! while maintaining backward compatibility through the main struct API.

use super::types::{Changeset, ChangesetStatus, MonorepoPackageInfo, VersionStatus};
use super::components::{
    PackageInfoReader, PackageVersionManager, PackageChangesetManager,
    PackagePersistence
};
use crate::config::{Environment, VersionBumpType};
use crate::error::Result;
use std::collections::HashMap;

impl MonorepoPackageInfo {
    /// Create a new `MonorepoPackageInfo`
    #[must_use]
    pub fn new(
        package_info: sublime_package_tools::PackageInfo,
        workspace_package: sublime_standard_tools::monorepo::WorkspacePackage,
        is_internal: bool,
    ) -> Self {
        Self {
            package_info,
            workspace_package,
            is_internal,
            dependents: Vec::new(),
            dependencies_external: Vec::new(),
            version_status: VersionStatus::Stable,
            changesets: Vec::new(),
        }
    }

    /// Get the package name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.workspace_package.name
    }

    /// Get the package version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.workspace_package.version
    }

    /// Get the package path
    #[must_use]
    pub fn path(&self) -> &std::path::PathBuf {
        &self.workspace_package.absolute_path
    }

    /// Get the relative path from monorepo root
    #[must_use]
    pub fn relative_path(&self) -> &std::path::PathBuf {
        &self.workspace_package.location
    }

    /// Check if this package has pending changesets
    #[must_use]
    pub fn has_pending_changesets(&self) -> bool {
        self.changesets.iter().any(|cs| matches!(cs.status, ChangesetStatus::Pending))
    }

    /// Get pending changesets
    #[must_use]
    pub fn pending_changesets(&self) -> Vec<&Changeset> {
        self.changesets.iter().filter(|cs| matches!(cs.status, ChangesetStatus::Pending)).collect()
    }

    /// Check if package is dirty (has uncommitted changes)
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        matches!(self.version_status, VersionStatus::Dirty)
    }
    /// Update the package version using the version manager component
    pub fn update_version(&mut self, new_version: &str) -> Result<()> {
        let mut version_manager = PackageVersionManager::new(self.clone());
        version_manager.update_version(new_version)?;
        *self = version_manager.into_package();
        Ok(())
    }

    /// Set version as snapshot using the version manager component
    pub fn set_snapshot_version(&mut self, version: &str, sha: &str) -> Result<()> {
        let mut version_manager = PackageVersionManager::new(self.clone());
        version_manager.set_snapshot_version(version, sha)?;
        *self = version_manager.into_package();
        Ok(())
    }

    /// Mark package as having dirty (uncommitted) changes using the version manager component
    pub fn mark_dirty(&mut self) {
        let mut version_manager = PackageVersionManager::new(self.clone());
        version_manager.mark_dirty();
        *self = version_manager.into_package();
    }

    /// Add a changeset to this package using the changeset manager component
    pub fn add_changeset(&mut self, changeset: Changeset) {
        let mut changeset_manager = PackageChangesetManager::new(self.clone());
        changeset_manager.add_changeset(changeset);
        *self = changeset_manager.into_package();
    }

    /// Apply a changeset and bump version accordingly using the changeset manager component
    pub fn apply_changeset(
        &mut self,
        changeset_id: &str,
        final_version: Option<&str>,
    ) -> Result<()> {
        let mut changeset_manager = PackageChangesetManager::new(self.clone());
        changeset_manager.apply_changeset(changeset_id, final_version)?;
        *self = changeset_manager.into_package();
        Ok(())
    }

    /// Deploy changeset to environments using the changeset manager component
    pub fn deploy_changeset(
        &mut self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<()> {
        let mut changeset_manager = PackageChangesetManager::new(self.clone());
        changeset_manager.deploy_changeset(changeset_id, environments)?;
        *self = changeset_manager.into_package();
        Ok(())
    }

    /// Get suggested version bump based on changesets using the version manager component
    #[must_use]
    pub fn suggested_version_bump(&self) -> Option<VersionBumpType> {
        let version_manager = PackageVersionManager::new(self.clone());
        version_manager.suggested_version_bump()
    }

    /// Check if package has been deployed to a specific environment using the info reader component
    #[must_use]
    pub fn is_deployed_to(&self, environment: &Environment) -> bool {
        let info_reader = PackageInfoReader::new(self);
        info_reader.is_deployed_to(environment)
    }

    /// Get deployment status across environments using the info reader component
    #[must_use]
    pub fn deployment_status(&self) -> HashMap<Environment, bool> {
        let info_reader = PackageInfoReader::new(self);
        info_reader.deployment_status()
    }

    /// Write package.json back to disk using the persistence component
    pub fn save(&self) -> Result<()> {
        let persistence = PackagePersistence::new(self.clone());
        persistence.save()
    }
}
