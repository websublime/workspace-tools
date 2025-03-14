//! JavaScript bindings for version-related types from ws_pkg.

use napi_derive::napi;
use semver::Version as SemVersion;
use ws_pkg::types::version::{Version as WsVersion, VersionRelationship};

/// JavaScript binding for ws_pkg::types::version::Version enum
#[napi]
#[derive(Clone)]
pub enum Version {
    /// Major version bump
    Major,
    /// Minor version bump
    Minor,
    /// Patch version bump
    Patch,
    /// Snapshot version
    Snapshot,
}

impl From<WsVersion> for Version {
    fn from(version: WsVersion) -> Self {
        match version {
            WsVersion::Major => Version::Major,
            WsVersion::Minor => Version::Minor,
            WsVersion::Patch => Version::Patch,
            WsVersion::Snapshot => Version::Snapshot,
        }
    }
}

impl From<Version> for WsVersion {
    fn from(version: Version) -> Self {
        match version {
            Version::Major => WsVersion::Major,
            Version::Minor => WsVersion::Minor,
            Version::Patch => WsVersion::Patch,
            Version::Snapshot => WsVersion::Snapshot,
        }
    }
}

/// JavaScript binding for version relationship comparisons
#[napi]
pub enum VersionComparisonResult {
    /// Second version is a major upgrade (1.0.0 -> 2.0.0)
    MajorUpgrade,
    /// Second version is a minor upgrade (1.0.0 -> 1.1.0)
    MinorUpgrade,
    /// Second version is a patch upgrade (1.0.0 -> 1.0.1)
    PatchUpgrade,
    /// Moved from prerelease to stable (1.0.0-alpha -> 1.0.0)
    PrereleaseToStable,
    /// Newer prerelease version (1.0.0-alpha -> 1.0.0-beta)
    NewerPrerelease,
    /// Versions are identical (1.0.0 == 1.0.0)
    Identical,
    /// Second version is a major downgrade (2.0.0 -> 1.0.0)
    MajorDowngrade,
    /// Second version is a minor downgrade (1.1.0 -> 1.0.0)
    MinorDowngrade,
    /// Second version is a patch downgrade (1.0.1 -> 1.0.0)
    PatchDowngrade,
    /// Moved from stable to prerelease (1.0.0 -> 1.0.0-alpha)
    StableToPrerelease,
    /// Older prerelease version (1.0.0-beta -> 1.0.0-alpha)
    OlderPrerelease,
    /// Version comparison couldn't be determined (invalid versions)
    Indeterminate,
}

impl From<VersionRelationship> for VersionComparisonResult {
    fn from(rel: VersionRelationship) -> Self {
        match rel {
            VersionRelationship::MajorUpgrade => VersionComparisonResult::MajorUpgrade,
            VersionRelationship::MinorUpgrade => VersionComparisonResult::MinorUpgrade,
            VersionRelationship::PatchUpgrade => VersionComparisonResult::PatchUpgrade,
            VersionRelationship::PrereleaseToStable => VersionComparisonResult::PrereleaseToStable,
            VersionRelationship::NewerPrerelease => VersionComparisonResult::NewerPrerelease,
            VersionRelationship::Identical => VersionComparisonResult::Identical,
            VersionRelationship::MajorDowngrade => VersionComparisonResult::MajorDowngrade,
            VersionRelationship::MinorDowngrade => VersionComparisonResult::MinorDowngrade,
            VersionRelationship::PatchDowngrade => VersionComparisonResult::PatchDowngrade,
            VersionRelationship::StableToPrerelease => VersionComparisonResult::StableToPrerelease,
            VersionRelationship::OlderPrerelease => VersionComparisonResult::OlderPrerelease,
            VersionRelationship::Indeterminate => VersionComparisonResult::Indeterminate,
        }
    }
}

/// JavaScript binding for version utilities
#[napi]
pub struct VersionUtils;

#[napi]
impl VersionUtils {
    /// Bump a version to the next major version
    #[napi]
    pub fn bump_major(version: String) -> Option<String> {
        if let Ok(_semver) = SemVersion::parse(&version) {
            Some(WsVersion::bump_major(&version).to_string())
        } else {
            None
        }
    }

    /// Bump a version to the next minor version
    #[napi]
    pub fn bump_minor(version: String) -> Option<String> {
        if let Ok(_semver) = SemVersion::parse(&version) {
            Some(WsVersion::bump_minor(&version).to_string())
        } else {
            None
        }
    }

    /// Bump a version to the next patch version
    #[napi]
    pub fn bump_patch(version: String) -> Option<String> {
        if let Ok(_semver) = SemVersion::parse(&version) {
            Some(WsVersion::bump_patch(&version).to_string())
        } else {
            None
        }
    }

    /// Bump a version to a snapshot version with the given SHA
    #[napi]
    pub fn bump_snapshot(version: String, sha: String) -> Option<String> {
        if let Ok(_semver) = SemVersion::parse(&version) {
            Some(WsVersion::bump_snapshot(&version, &sha).to_string())
        } else {
            None
        }
    }

    /// Compare two version strings and return their relationship
    #[napi]
    pub fn compare_versions(v1: String, v2: String) -> VersionComparisonResult {
        WsVersion::compare_versions(&v1, &v2).into()
    }

    /// Check if moving from v1 to v2 is a breaking change
    #[napi]
    pub fn is_breaking_change(v1: String, v2: String) -> bool {
        WsVersion::is_breaking_change(&v1, &v2)
    }
}

/// JavaScript binding for ws_pkg::types::version::VersionUpdateStrategy
#[napi]
#[derive(Clone)]
pub enum VersionUpdateStrategy {
    /// Only upgrade patch versions (0.0.x)
    PatchOnly,
    /// Upgrade patch and minor versions (0.x.y)
    MinorAndPatch,
    /// Upgrade all versions including major ones (x.y.z)
    AllUpdates,
}

impl From<VersionUpdateStrategy> for ws_pkg::types::version::VersionUpdateStrategy {
    fn from(strategy: VersionUpdateStrategy) -> Self {
        match strategy {
            VersionUpdateStrategy::PatchOnly => Self::PatchOnly,
            VersionUpdateStrategy::MinorAndPatch => Self::MinorAndPatch,
            VersionUpdateStrategy::AllUpdates => Self::AllUpdates,
        }
    }
}

impl From<ws_pkg::types::version::VersionUpdateStrategy> for VersionUpdateStrategy {
    fn from(strategy: ws_pkg::types::version::VersionUpdateStrategy) -> Self {
        match strategy {
            ws_pkg::types::version::VersionUpdateStrategy::PatchOnly => Self::PatchOnly,
            ws_pkg::types::version::VersionUpdateStrategy::MinorAndPatch => Self::MinorAndPatch,
            ws_pkg::types::version::VersionUpdateStrategy::AllUpdates => Self::AllUpdates,
        }
    }
}

/// JavaScript binding for ws_pkg::types::version::VersionStability
#[napi]
#[derive(Clone)]
pub enum VersionStability {
    /// Only include stable versions
    StableOnly,
    /// Include prereleases and stable versions
    IncludePrerelease,
}

impl From<VersionStability> for ws_pkg::types::version::VersionStability {
    fn from(stability: VersionStability) -> Self {
        match stability {
            VersionStability::StableOnly => Self::StableOnly,
            VersionStability::IncludePrerelease => Self::IncludePrerelease,
        }
    }
}

impl From<ws_pkg::types::version::VersionStability> for VersionStability {
    fn from(stability: ws_pkg::types::version::VersionStability) -> Self {
        match stability {
            ws_pkg::types::version::VersionStability::StableOnly => Self::StableOnly,
            ws_pkg::types::version::VersionStability::IncludePrerelease => Self::IncludePrerelease,
        }
    }
}

#[cfg(test)]
mod version_types_tests {
    use super::*;

    #[test]
    fn test_version_update_strategy_conversion() {
        // Test conversion from VersionUpdateStrategy to ws_pkg::types::version::VersionUpdateStrategy
        let strategy = VersionUpdateStrategy::MinorAndPatch;
        let ws_strategy = ws_pkg::types::version::VersionUpdateStrategy::from(strategy);
        assert!(matches!(
            ws_strategy,
            ws_pkg::types::version::VersionUpdateStrategy::MinorAndPatch
        ));

        // Test conversion from ws_pkg::types::version::VersionUpdateStrategy to VersionUpdateStrategy
        let strategy_back = VersionUpdateStrategy::from(ws_strategy);
        assert!(matches!(strategy_back, VersionUpdateStrategy::MinorAndPatch));
    }

    #[test]
    fn test_version_stability_conversion() {
        // Test conversion from VersionStability to ws_pkg::types::version::VersionStability
        let stability = VersionStability::IncludePrerelease;
        let ws_stability = ws_pkg::types::version::VersionStability::from(stability);
        assert!(matches!(
            ws_stability,
            ws_pkg::types::version::VersionStability::IncludePrerelease
        ));

        // Test conversion from ws_pkg::types::version::VersionStability to VersionStability
        let stability_back = VersionStability::from(ws_stability);
        assert!(matches!(stability_back, VersionStability::IncludePrerelease));
    }
}

#[cfg(test)]
mod version_binding_tests {
    use super::*;

    #[test]
    fn test_version_bump_major() {
        assert_eq!(VersionUtils::bump_major("1.2.3".to_string()), Some("2.0.0".to_string()));
        assert_eq!(VersionUtils::bump_major("invalid".to_string()), None);
    }

    #[test]
    fn test_version_bump_minor() {
        assert_eq!(VersionUtils::bump_minor("1.2.3".to_string()), Some("1.3.0".to_string()));
        assert_eq!(VersionUtils::bump_minor("invalid".to_string()), None);
    }

    #[test]
    fn test_version_bump_patch() {
        assert_eq!(VersionUtils::bump_patch("1.2.3".to_string()), Some("1.2.4".to_string()));
        assert_eq!(VersionUtils::bump_patch("invalid".to_string()), None);
    }

    #[test]
    fn test_version_bump_snapshot() {
        assert_eq!(
            VersionUtils::bump_snapshot("1.2.3".to_string(), "abc123".to_string()),
            Some("1.2.3-alpha.abc123".to_string())
        );
        assert_eq!(VersionUtils::bump_snapshot("invalid".to_string(), "abc123".to_string()), None);
    }

    #[test]
    fn test_compare_versions() {
        assert!(matches!(
            VersionUtils::compare_versions("1.0.0".to_string(), "2.0.0".to_string()),
            VersionComparisonResult::MajorUpgrade
        ));
        assert!(matches!(
            VersionUtils::compare_versions("1.0.0".to_string(), "1.1.0".to_string()),
            VersionComparisonResult::MinorUpgrade
        ));
        assert!(matches!(
            VersionUtils::compare_versions("1.0.0".to_string(), "1.0.0".to_string()),
            VersionComparisonResult::Identical
        ));
        assert!(matches!(
            VersionUtils::compare_versions("2.0.0".to_string(), "1.0.0".to_string()),
            VersionComparisonResult::MajorDowngrade
        ));
        assert!(matches!(
            VersionUtils::compare_versions("invalid".to_string(), "1.0.0".to_string()),
            VersionComparisonResult::Indeterminate
        ));
    }

    #[test]
    fn test_is_breaking_change() {
        assert!(VersionUtils::is_breaking_change("1.0.0".to_string(), "2.0.0".to_string()));
        assert!(!VersionUtils::is_breaking_change("1.0.0".to_string(), "1.1.0".to_string()));
        assert!(!VersionUtils::is_breaking_change("1.0.0".to_string(), "1.0.1".to_string()));
        assert!(VersionUtils::is_breaking_change("invalid".to_string(), "1.0.0".to_string()));
        // Should conservatively return true
    }
}
