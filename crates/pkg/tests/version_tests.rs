#[cfg(test)]
mod version_tests {
    use ws_pkg::types::version::{
        Version, VersionRelationship, VersionStability, VersionUpdateStrategy,
    };

    #[test]
    fn test_version_enum() {
        let major = Version::Major;
        let minor = Version::Minor;
        let patch = Version::Patch;
        let snapshot = Version::Snapshot;

        assert_eq!(major.to_string(), "major");
        assert_eq!(minor.to_string(), "minor");
        assert_eq!(patch.to_string(), "patch");
        assert_eq!(snapshot.to_string(), "snapshot");

        // Test From<&str> implementation
        assert!(matches!(Version::from("major"), Version::Major));
        assert!(matches!(Version::from("minor"), Version::Minor));
        assert!(matches!(Version::from("patch"), Version::Patch));
        assert!(matches!(Version::from("snapshot"), Version::Snapshot));
        assert!(matches!(Version::from("unknown"), Version::Patch)); // Default to patch
    }

    #[test]
    fn test_version_comparison() {
        // Test various version comparisons
        assert_eq!(Version::compare_versions("1.0.0", "1.0.1"), VersionRelationship::PatchUpgrade);
        assert_eq!(Version::compare_versions("1.0.0", "1.1.0"), VersionRelationship::MinorUpgrade);
        assert_eq!(Version::compare_versions("1.0.0", "2.0.0"), VersionRelationship::MajorUpgrade);
        assert_eq!(Version::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);
        assert_eq!(
            Version::compare_versions("1.0.1", "1.0.0"),
            VersionRelationship::PatchDowngrade
        );
        assert_eq!(
            Version::compare_versions("1.1.0", "1.0.0"),
            VersionRelationship::MinorDowngrade
        );
        assert_eq!(
            Version::compare_versions("2.0.0", "1.0.0"),
            VersionRelationship::MajorDowngrade
        );
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0"),
            VersionRelationship::PrereleaseToStable
        );
        assert_eq!(
            Version::compare_versions("1.0.0", "1.0.0-alpha"),
            VersionRelationship::StableToPrerelease
        );
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0-beta"),
            VersionRelationship::NewerPrerelease
        );
        assert_eq!(
            Version::compare_versions("1.0.0-beta", "1.0.0-alpha"),
            VersionRelationship::OlderPrerelease
        );
        assert_eq!(
            Version::compare_versions("invalid", "1.0.0"),
            VersionRelationship::Indeterminate
        );
    }

    #[test]
    fn test_version_bumping() {
        // Test major bump
        let bumped = Version::bump_major("1.2.3");
        assert_eq!(bumped.to_string(), "2.0.0");

        // Test minor bump
        let bumped = Version::bump_minor("1.2.3");
        assert_eq!(bumped.to_string(), "1.3.0");

        // Test patch bump
        let bumped = Version::bump_patch("1.2.3");
        assert_eq!(bumped.to_string(), "1.2.4");

        // Test snapshot bump
        let bumped = Version::bump_snapshot("1.2.3", "abc123");
        assert_eq!(bumped.to_string(), "1.2.3-alpha.abc123");
    }

    #[test]
    fn test_breaking_change_detection() {
        assert!(Version::is_breaking_change("1.0.0", "2.0.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.0.1"));
        assert!(Version::is_breaking_change("invalid", "1.0.0")); // Should conservatively return true
    }

    #[test]
    fn test_version_update_strategy() {
        // Test all variants can be created
        let patch_only = VersionUpdateStrategy::PatchOnly;
        let minor_and_patch = VersionUpdateStrategy::MinorAndPatch;
        let all_updates = VersionUpdateStrategy::AllUpdates;

        // Verify correct variants
        assert!(matches!(patch_only, VersionUpdateStrategy::PatchOnly));
        assert!(matches!(minor_and_patch, VersionUpdateStrategy::MinorAndPatch));
        assert!(matches!(all_updates, VersionUpdateStrategy::AllUpdates));

        // Test default is MinorAndPatch
        let default = VersionUpdateStrategy::default();
        assert!(matches!(default, VersionUpdateStrategy::MinorAndPatch));
    }

    #[test]
    fn test_version_stability() {
        // Test all variants can be created
        let stable_only = VersionStability::StableOnly;
        let include_prerelease = VersionStability::IncludePrerelease;

        // Verify correct variants
        assert!(matches!(stable_only, VersionStability::StableOnly));
        assert!(matches!(include_prerelease, VersionStability::IncludePrerelease));

        // Test default is StableOnly
        let default = VersionStability::default();
        assert!(matches!(default, VersionStability::StableOnly));
    }
}
