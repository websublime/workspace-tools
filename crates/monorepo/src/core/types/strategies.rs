//! Versioning strategy implementations

use crate::config::VersionBumpType;
use crate::analysis::PackageChange;
use crate::changes::ChangeSignificance;

/// Strategy for determining version bumps and propagation
pub trait VersioningStrategy: Send + Sync {
    /// Determine the bump type for a package based on changes
    fn determine_bump_type(&self, changes: &PackageChange) -> VersionBumpType;

    /// Determine if a change should propagate to dependents
    fn should_propagate(&self, bump_type: VersionBumpType) -> bool;

    /// Determine bump type for a dependent package
    fn determine_bump_type_for_dependent(
        &self,
        _changed_package: &str,
        _dependent_package: &str,
    ) -> Option<VersionBumpType>;
}

/// Default versioning strategy implementation
#[derive(Debug, Clone)]
pub struct DefaultVersioningStrategy;

/// Conservative versioning strategy - minimal propagation
#[derive(Debug, Clone)]
pub struct ConservativeVersioningStrategy;

/// Aggressive versioning strategy - propagates all changes
#[derive(Debug, Clone)]
pub struct AggressiveVersioningStrategy;

impl VersioningStrategy for DefaultVersioningStrategy {
    fn determine_bump_type(&self, changes: &PackageChange) -> VersionBumpType {
        changes.suggested_version_bump
    }

    fn should_propagate(&self, bump_type: VersionBumpType) -> bool {
        matches!(bump_type, VersionBumpType::Major | VersionBumpType::Minor)
    }

    fn determine_bump_type_for_dependent(
        &self,
        _changed_package: &str,
        _dependent_package: &str,
    ) -> Option<VersionBumpType> {
        // Conservative strategy: only bump patch versions for dependents
        Some(VersionBumpType::Patch)
    }
}

impl VersioningStrategy for ConservativeVersioningStrategy {
    fn determine_bump_type(&self, changes: &PackageChange) -> VersionBumpType {
        match changes.significance {
            ChangeSignificance::High => VersionBumpType::Major,
            ChangeSignificance::Medium => VersionBumpType::Minor,
            ChangeSignificance::Low => VersionBumpType::Patch,
        }
    }

    fn should_propagate(&self, bump_type: VersionBumpType) -> bool {
        matches!(bump_type, VersionBumpType::Major)
    }

    fn determine_bump_type_for_dependent(
        &self,
        _changed_package: &str,
        _dependent_package: &str,
    ) -> Option<VersionBumpType> {
        None // No automatic propagation
    }
}

impl VersioningStrategy for AggressiveVersioningStrategy {
    fn determine_bump_type(&self, changes: &PackageChange) -> VersionBumpType {
        changes.suggested_version_bump
    }

    fn should_propagate(&self, _bump_type: VersionBumpType) -> bool {
        true // Always propagate
    }

    fn determine_bump_type_for_dependent(
        &self,
        _changed_package: &str,
        _dependent_package: &str,
    ) -> Option<VersionBumpType> {
        Some(VersionBumpType::Patch) // Always bump dependents
    }
}