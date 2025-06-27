//! Package changeset management component
//!
//! Handles all changeset-related operations including adding, applying,
//! and deploying changesets for packages.

use super::super::types::{Changeset, ChangesetStatus, MonorepoPackageInfo};
use crate::config::{Environment, VersionBumpType};
use crate::error::{Error, Result};
use sublime_package_tools::Version;

/// Component for managing package changeset operations
pub struct PackageChangesetManager {
    package: MonorepoPackageInfo,
}

impl PackageChangesetManager {
    /// Create a new package changeset manager
    #[must_use]
    pub fn new(package: MonorepoPackageInfo) -> Self {
        Self { package }
    }

    /// Get immutable reference to the package
    #[must_use]
    pub fn package(&self) -> &MonorepoPackageInfo {
        &self.package
    }

    /// Get all changesets for this package
    #[must_use]
    pub fn changesets(&self) -> &[Changeset] {
        &self.package.changesets
    }

    /// Get pending changesets
    #[must_use]
    pub fn pending_changesets(&self) -> Vec<&Changeset> {
        self.package
            .changesets
            .iter()
            .filter(|cs| matches!(cs.status, ChangesetStatus::Pending))
            .collect()
    }

    /// Get merged changesets
    #[must_use]
    pub fn merged_changesets(&self) -> Vec<&Changeset> {
        self.package
            .changesets
            .iter()
            .filter(|cs| matches!(cs.status, ChangesetStatus::Merged { .. }))
            .collect()
    }

    /// Get deployed changesets
    #[must_use]
    pub fn deployed_changesets(&self) -> Vec<&Changeset> {
        self.package
            .changesets
            .iter()
            .filter(|cs| {
                matches!(
                    cs.status,
                    ChangesetStatus::FullyDeployed { .. }
                        | ChangesetStatus::PartiallyDeployed { .. }
                )
            })
            .collect()
    }

    /// Add a changeset to this package
    ///
    /// # Arguments
    /// * `changeset` - The changeset to add
    pub fn add_changeset(&mut self, changeset: Changeset) {
        self.package.changesets.push(changeset);
    }

    /// Remove a changeset by ID
    ///
    /// # Arguments
    /// * `changeset_id` - ID of the changeset to remove
    ///
    /// # Returns
    /// True if a changeset was removed, false if not found
    pub fn remove_changeset(&mut self, changeset_id: &str) -> bool {
        let initial_len = self.package.changesets.len();
        self.package.changesets.retain(|cs| cs.id != changeset_id);
        self.package.changesets.len() < initial_len
    }

    /// Find a changeset by ID
    #[must_use]
    pub fn find_changeset(&self, changeset_id: &str) -> Option<&Changeset> {
        self.package.changesets.iter().find(|cs| cs.id == changeset_id)
    }

    /// Find a mutable changeset by ID
    pub fn find_changeset_mut(&mut self, changeset_id: &str) -> Option<&mut Changeset> {
        self.package.changesets.iter_mut().find(|cs| cs.id == changeset_id)
    }

    /// Apply a changeset and bump version accordingly
    ///
    /// # Arguments
    /// * `changeset_id` - ID of the changeset to apply
    /// * `final_version` - Optional specific version to set (if None, auto-bump based on changeset)
    ///
    /// # Errors
    /// Returns an error if the changeset is not found or version update fails
    pub fn apply_changeset(
        &mut self,
        changeset_id: &str,
        final_version: Option<&str>,
    ) -> Result<String> {
        // Find changeset index first
        let changeset_idx = self
            .package
            .changesets
            .iter()
            .position(|cs| cs.id == changeset_id)
            .ok_or_else(|| Error::changeset(format!("Changeset {changeset_id} not found")))?;

        // Determine new version
        let new_version = if let Some(version) = final_version {
            version.to_string()
        } else {
            let current_version = &self.package.workspace_package.version;
            let version_bump = self.package.changesets[changeset_idx].version_bump;
            match version_bump {
                VersionBumpType::Major => Version::bump_major(current_version)?.to_string(),
                VersionBumpType::Minor => Version::bump_minor(current_version)?.to_string(),
                VersionBumpType::Patch => Version::bump_patch(current_version)?.to_string(),
                VersionBumpType::Snapshot => {
                    return Err(Error::changeset("Cannot apply snapshot changeset without SHA"));
                }
            }
        };

        // Update version in both package info and workspace package
        self.package
            .package_info
            .update_version(&new_version)
            .map_err(|e| Error::changeset(format!("Failed to update package version: {e}")))?;
        // Clone once and use efficiently
        let final_version = new_version.clone();

        // Update changeset status (move original new_version)
        self.package.changesets[changeset_idx].status =
            ChangesetStatus::Merged { merged_at: chrono::Utc::now(), final_version: new_version };

        // Use clone_from for efficient assignment
        self.package.workspace_package.version.clone_from(&final_version);

        Ok(final_version)
    }

    /// Deploy changeset to environments
    ///
    /// # Arguments
    /// * `changeset_id` - ID of the changeset to deploy
    /// * `environments` - List of environments to deploy to
    ///
    /// # Errors
    /// Returns an error if the changeset is not found or already deployed
    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn deploy_changeset(
        &mut self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<()> {
        let changeset = self
            .find_changeset_mut(changeset_id)
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
            ChangesetStatus::Merged { .. } => {
                // Allow deployment of merged changesets
                changeset.status =
                    ChangesetStatus::PartiallyDeployed { environments: environments.to_vec() };
            }
            _ => {
                return Err(Error::changeset("Changeset is already fully deployed"));
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

    /// Rollback a changeset deployment from specific environments
    ///
    /// # Arguments
    /// * `changeset_id` - ID of the changeset to rollback
    /// * `environments` - List of environments to rollback from
    ///
    /// # Errors
    /// Returns an error if the changeset is not found
    pub fn rollback_changeset(
        &mut self,
        changeset_id: &str,
        environments: &[Environment],
    ) -> Result<()> {
        let changeset = self
            .find_changeset_mut(changeset_id)
            .ok_or_else(|| Error::changeset(format!("Changeset {changeset_id} not found")))?;

        // Remove from development environments
        changeset.development_environments.retain(|env| !environments.contains(env));

        // Handle production rollback
        if environments.contains(&Environment::Production) {
            changeset.production_deployment = false;
        }

        // Update status based on remaining deployments
        if changeset.development_environments.is_empty() && !changeset.production_deployment {
            changeset.status = ChangesetStatus::Pending;
        } else if !changeset.production_deployment {
            changeset.status = ChangesetStatus::PartiallyDeployed {
                environments: changeset.development_environments.clone(),
            };
        }

        Ok(())
    }

    /// Get changeset deployment summary
    #[must_use]
    pub fn deployment_summary(&self) -> ChangesetDeploymentSummary {
        let total = self.package.changesets.len();
        let pending = self.pending_changesets().len();
        let merged = self.merged_changesets().len();
        let deployed = self.deployed_changesets().len();

        let production_deployments =
            self.package.changesets.iter().filter(|cs| cs.production_deployment).count();

        ChangesetDeploymentSummary {
            total_changesets: total,
            pending_changesets: pending,
            merged_changesets: merged,
            deployed_changesets: deployed,
            production_deployments,
        }
    }

    /// Validate all changesets in the package
    #[must_use]
    pub fn validate_changesets(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for changeset in &self.package.changesets {
            // Check for empty descriptions
            if changeset.description.trim().is_empty() {
                errors.push(format!("Changeset {} has empty description", changeset.id));
            }

            // Check for invalid environments
            for env in &changeset.development_environments {
                if matches!(env, Environment::Production) && !changeset.production_deployment {
                    errors.push(format!(
                        "Changeset {} has inconsistent production environment",
                        changeset.id
                    ));
                }
            }

            // Check for valid version bump
            if matches!(changeset.version_bump, VersionBumpType::Snapshot) {
                errors.push(format!("Changeset {} uses snapshot version bump", changeset.id));
            }
        }

        errors
    }

    /// Consume the manager and return the updated package
    #[must_use]
    pub fn into_package(self) -> MonorepoPackageInfo {
        self.package
    }
}

/// Summary of changeset deployment status
#[derive(Debug, Clone)]
pub struct ChangesetDeploymentSummary {
    /// Total number of changesets
    pub total_changesets: usize,
    /// Number of pending changesets
    pub pending_changesets: usize,
    /// Number of merged changesets
    pub merged_changesets: usize,
    /// Number of deployed changesets
    pub deployed_changesets: usize,
    /// Number of production deployments
    pub production_deployments: usize,
}
