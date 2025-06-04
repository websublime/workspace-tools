//! Package-specific implementations and utilities

use super::types::{Changeset, ChangesetStatus, MonorepoPackageInfo, VersionStatus};
use crate::config::{Environment, VersionBumpType};
use crate::error::{Error, Result};
use std::collections::HashMap;
use sublime_package_tools::Version;

impl MonorepoPackageInfo {
    /// Update the package version
    pub fn update_version(&mut self, new_version: &str) -> Result<()> {
        // Update in PackageInfo
        self.package_info.update_version(new_version)?;

        // Update in WorkspacePackage
        self.workspace_package.version = new_version.to_string();

        // Update version status
        self.version_status = VersionStatus::Stable;

        Ok(())
    }

    /// Set version as snapshot
    pub fn set_snapshot_version(&mut self, version: &str, sha: &str) -> Result<()> {
        let snapshot_version = format!("{}-snapshot.{}", version, &sha[..7]);
        self.update_version(&snapshot_version)?;
        self.version_status = VersionStatus::Snapshot { sha: sha.to_string() };
        Ok(())
    }

    /// Mark package as having dirty (uncommitted) changes
    pub fn mark_dirty(&mut self) {
        self.version_status = VersionStatus::Dirty;
    }

    /// Add a changeset to this package
    pub fn add_changeset(&mut self, changeset: Changeset) {
        self.changesets.push(changeset);
    }

    /// Apply a changeset and bump version accordingly
    pub fn apply_changeset(
        &mut self,
        changeset_id: &str,
        final_version: Option<&str>,
    ) -> Result<()> {
        // Find changeset index first
        let changeset_idx = self
            .changesets
            .iter()
            .position(|cs| cs.id == changeset_id)
            .ok_or_else(|| Error::changeset(format!("Changeset {changeset_id} not found")))?;

        // Determine new version
        let new_version = if let Some(version) = final_version {
            version.to_string()
        } else {
            let current_version = self.version();
            let version_bump = self.changesets[changeset_idx].version_bump;
            match version_bump {
                VersionBumpType::Major => Version::bump_major(current_version)?.to_string(),
                VersionBumpType::Minor => Version::bump_minor(current_version)?.to_string(),
                VersionBumpType::Patch => Version::bump_patch(current_version)?.to_string(),
                VersionBumpType::Snapshot => {
                    return Err(Error::changeset("Cannot apply snapshot changeset without SHA"));
                }
            }
        };

        // Update version
        self.update_version(&new_version)?;

        // Update changeset status
        self.changesets[changeset_idx].status =
            ChangesetStatus::Merged { merged_at: chrono::Utc::now(), final_version: new_version };

        Ok(())
    }

    /// Deploy changeset to environments
    pub fn deploy_changeset(
        &mut self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<()> {
        let changeset = self
            .changesets
            .iter_mut()
            .find(|cs| cs.id == changeset_id)
            .ok_or_else(|| Error::changeset(format!("Changeset {changeset_id} not found")))?;

        // Update deployment status
        match &mut changeset.status {
            ChangesetStatus::Pending => {
                changeset.status =
                    ChangesetStatus::PartiallyDeployed { environments: environments.to_vec() };
            }
            ChangesetStatus::PartiallyDeployed { environments: deployed } => {
                for env in environments {
                    if !deployed.contains(env) {
                        deployed.push(env.clone());
                    }
                }
            }
            _ => {
                return Err(Error::changeset("Changeset is already merged or fully deployed"));
            }
        }

        // Update development environments
        for env in environments {
            if !changeset.development_environments.contains(env) {
                changeset.development_environments.push(env.clone());
            }
        }

        // Check if fully deployed
        if environments.contains(&Environment::Production) {
            changeset.production_deployment = true;
            changeset.status = ChangesetStatus::FullyDeployed { deployed_at: chrono::Utc::now() };
        }

        Ok(())
    }

    /// Get suggested version bump based on changesets
    pub fn suggested_version_bump(&self) -> Option<VersionBumpType> {
        let pending = self.pending_changesets();
        if pending.is_empty() {
            return None;
        }

        // Find the highest priority bump
        let mut bump = VersionBumpType::Patch;
        for changeset in pending {
            match changeset.version_bump {
                VersionBumpType::Major => return Some(VersionBumpType::Major),
                VersionBumpType::Minor => bump = VersionBumpType::Minor,
                VersionBumpType::Patch | VersionBumpType::Snapshot => {}
            }
        }

        Some(bump)
    }

    /// Check if package has been deployed to a specific environment
    pub fn is_deployed_to(&self, environment: &Environment) -> bool {
        self.changesets.iter().any(|cs| {
            cs.development_environments.contains(environment)
                || (environment == &Environment::Production && cs.production_deployment)
        })
    }

    /// Get deployment status across environments
    pub fn deployment_status(&self) -> HashMap<Environment, bool> {
        let mut status = HashMap::new();

        for changeset in &self.changesets {
            for env in &changeset.development_environments {
                status.insert(env.clone(), true);
            }
            if changeset.production_deployment {
                status.insert(Environment::Production, true);
            }
        }

        status
    }

    /// Write package.json back to disk
    pub fn save(&self) -> Result<()> {
        self.package_info.write_package_json().map_err(|e| Error::Package(e.to_string()))
    }
}
