//! Package information reader component
//!
//! Provides read-only access to package information including basic metadata,
//! paths, and status queries without modification capabilities.

use super::super::types::{Changeset, ChangesetStatus, MonorepoPackageInfo, VersionStatus};
use crate::config::Environment;
use std::collections::HashMap;

/// Component for read-only access to package information
pub struct PackageInfoReader<'a> {
    package: &'a MonorepoPackageInfo,
}

impl<'a> PackageInfoReader<'a> {
    /// Create a new package info reader
    #[must_use]
    pub fn new(package: &'a MonorepoPackageInfo) -> Self {
        Self { package }
    }

    /// Get the package name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.package.workspace_package.name
    }

    /// Get the package version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.package.workspace_package.version
    }

    /// Get the package path
    #[must_use]
    pub fn path(&self) -> &std::path::PathBuf {
        &self.package.workspace_package.absolute_path
    }

    /// Get the relative path from monorepo root
    #[must_use]
    pub fn relative_path(&self) -> &std::path::PathBuf {
        &self.package.workspace_package.location
    }

    /// Check if this is an internal package (part of the monorepo)
    #[must_use]
    pub fn is_internal(&self) -> bool {
        self.package.is_internal
    }

    /// Get list of packages that depend on this package
    #[must_use]
    pub fn dependents(&self) -> &[String] {
        &self.package.dependents
    }

    /// Get external dependencies (not in the monorepo)
    #[must_use]
    pub fn external_dependencies(&self) -> &[String] {
        &self.package.dependencies_external
    }

    /// Get current version status
    #[must_use]
    pub fn version_status(&self) -> &VersionStatus {
        &self.package.version_status
    }

    /// Check if this package has pending changesets
    #[must_use]
    pub fn has_pending_changesets(&self) -> bool {
        self.package.changesets.iter().any(|cs| matches!(cs.status, ChangesetStatus::Pending))
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

    /// Get all changesets associated with this package
    #[must_use]
    pub fn changesets(&self) -> &[Changeset] {
        &self.package.changesets
    }

    /// Check if package is dirty (has uncommitted changes)
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        matches!(self.package.version_status, VersionStatus::Dirty)
    }

    /// Check if package has been deployed to a specific environment
    #[must_use]
    pub fn is_deployed_to(&self, environment: &Environment) -> bool {
        self.package.changesets.iter().any(|cs| {
            cs.development_environments.contains(environment)
                || (environment == &Environment::Production && cs.production_deployment)
        })
    }

    /// Get deployment status across environments
    #[must_use]
    pub fn deployment_status(&self) -> HashMap<Environment, bool> {
        let mut status = HashMap::new();

        for changeset in &self.package.changesets {
            for env in &changeset.development_environments {
                status.insert(env.clone(), true);
            }
            if changeset.production_deployment {
                status.insert(Environment::Production, true);
            }
        }

        status
    }

    /// Check if package is a snapshot version
    #[must_use]
    pub fn is_snapshot(&self) -> bool {
        matches!(self.package.version_status, VersionStatus::Snapshot { .. })
    }

    /// Check if package is a pre-release version
    #[must_use]
    pub fn is_pre_release(&self) -> bool {
        matches!(self.package.version_status, VersionStatus::PreRelease { .. })
    }

    /// Get snapshot SHA if this is a snapshot version
    #[must_use]
    pub fn snapshot_sha(&self) -> Option<&str> {
        match &self.package.version_status {
            VersionStatus::Snapshot { sha } => Some(sha),
            _ => None,
        }
    }

    /// Get pre-release tag if this is a pre-release version
    #[must_use]
    pub fn pre_release_tag(&self) -> Option<&str> {
        match &self.package.version_status {
            VersionStatus::PreRelease { tag } => Some(tag),
            _ => None,
        }
    }

    /// Get package statistics
    #[must_use]
    pub fn package_stats(&self) -> PackageStats {
        let pending_changesets = self.pending_changesets().len();
        let total_changesets = self.package.changesets.len();
        let dependents_count = self.package.dependents.len();
        let external_deps_count = self.package.dependencies_external.len();

        PackageStats {
            pending_changesets,
            total_changesets,
            dependents_count,
            external_deps_count,
            is_internal: self.package.is_internal,
            is_dirty: self.is_dirty(),
        }
    }
}

/// Package statistics summary
#[derive(Debug, Clone)]
pub struct PackageStats {
    /// Number of pending changesets
    pub pending_changesets: usize,
    /// Total number of changesets
    pub total_changesets: usize,
    /// Number of dependent packages
    pub dependents_count: usize,
    /// Number of external dependencies
    pub external_deps_count: usize,
    /// Whether this is an internal package
    pub is_internal: bool,
    /// Whether the package has uncommitted changes
    pub is_dirty: bool,
}
