#[cfg(test)]
mod version_tests {
    use semver::Version;
    use ws_pkg::{Version as WsVersion, VersionRelationship};

    #[test]
    fn test_bump_major() {
        let version = WsVersion::bump_major("1.0.0");
        assert_eq!(version, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_bump_minor() {
        let version = WsVersion::bump_minor("1.0.0");
        assert_eq!(version, Version::parse("1.1.0").unwrap());
    }

    #[test]
    fn test_bump_patch() {
        let version = WsVersion::bump_patch("1.0.0");
        assert_eq!(version, Version::parse("1.0.1").unwrap());
    }

    #[test]
    fn test_bump_snapshot() {
        let version = WsVersion::bump_snapshot("1.0.0", "sha");
        assert_eq!(version, Version::parse("1.0.0-alpha.sha").unwrap());
    }

    #[test]
    fn test_version_comparison() {
        // Test upgrades
        assert_eq!(
            WsVersion::compare_versions("1.0.0", "2.0.0"),
            VersionRelationship::MajorUpgrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0", "1.1.0"),
            VersionRelationship::MinorUpgrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0", "1.0.1"),
            VersionRelationship::PatchUpgrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0-alpha", "1.0.0"),
            VersionRelationship::PrereleaseToStable
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0-alpha", "1.0.0-beta"),
            VersionRelationship::NewerPrerelease
        );

        // Test identical
        assert_eq!(WsVersion::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);

        // Test downgrades
        assert_eq!(
            WsVersion::compare_versions("2.0.0", "1.0.0"),
            VersionRelationship::MajorDowngrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.1.0", "1.0.0"),
            VersionRelationship::MinorDowngrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.1", "1.0.0"),
            VersionRelationship::PatchDowngrade
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0", "1.0.0-alpha"),
            VersionRelationship::StableToPrerelease
        );
        assert_eq!(
            WsVersion::compare_versions("1.0.0-beta", "1.0.0-alpha"),
            VersionRelationship::OlderPrerelease
        );

        // Test invalid versions
        assert_eq!(
            WsVersion::compare_versions("invalid", "1.0.0"),
            VersionRelationship::Indeterminate
        );
    }

    #[test]
    fn test_breaking_change_detection() {
        // Major version changes are breaking
        assert!(WsVersion::is_breaking_change("1.0.0", "2.0.0"));

        // Minor version changes are not breaking
        assert!(!WsVersion::is_breaking_change("1.0.0", "1.1.0"));

        // Patch version changes are not breaking
        assert!(!WsVersion::is_breaking_change("1.0.0", "1.0.1"));

        // Prerelease changes follow the same rules
        assert!(WsVersion::is_breaking_change("1.0.0-alpha", "2.0.0-alpha"));
        assert!(!WsVersion::is_breaking_change("1.0.0-alpha", "1.1.0-alpha"));

        // Going from stable to alpha of same version is not breaking
        assert!(!WsVersion::is_breaking_change("1.0.0", "1.0.0-alpha"));

        // Invalid versions conservatively return true
        assert!(WsVersion::is_breaking_change("invalid", "1.0.0"));
        assert!(WsVersion::is_breaking_change("1.0.0", "invalid"));
    }
}
