#[cfg(test)]
mod version_tests {
    use sublime_package_tools::{Version, VersionRelationship};

    #[test]
    fn test_version_bumps() {
        // Test major bump
        let major_bump = Version::bump_major("1.2.3").unwrap();
        assert_eq!(major_bump, semver::Version::parse("2.0.0").unwrap());

        // Test minor bump
        let minor_bump = Version::bump_minor("1.2.3").unwrap();
        assert_eq!(minor_bump, semver::Version::parse("1.3.0").unwrap());

        // Test patch bump
        let patch_bump = Version::bump_patch("1.2.3").unwrap();
        assert_eq!(patch_bump, semver::Version::parse("1.2.4").unwrap());

        // Test snapshot bump
        let snapshot = Version::bump_snapshot("1.2.3", "abc123").unwrap();
        assert_eq!(snapshot.major, 1);
        assert_eq!(snapshot.minor, 2);
        assert_eq!(snapshot.patch, 3);
        assert!(!snapshot.pre.is_empty());
        assert!(snapshot.pre.to_string().contains("alpha"));
        assert!(snapshot.pre.to_string().contains("abc123"));
    }

    #[test]
    fn test_version_comparisons() {
        // Test major upgrade
        let rel1 = Version::compare_versions("1.0.0", "2.0.0");
        assert_eq!(rel1, VersionRelationship::MajorUpgrade);

        // Test minor upgrade
        let rel2 = Version::compare_versions("1.0.0", "1.1.0");
        assert_eq!(rel2, VersionRelationship::MinorUpgrade);

        // Test patch upgrade
        let rel3 = Version::compare_versions("1.0.0", "1.0.1");
        assert_eq!(rel3, VersionRelationship::PatchUpgrade);

        // Test prerelease to stable
        let rel4 = Version::compare_versions("1.0.0-alpha", "1.0.0");
        assert_eq!(rel4, VersionRelationship::PrereleaseToStable);

        // Test newer prerelease
        let rel5 = Version::compare_versions("1.0.0-alpha", "1.0.0-beta");
        assert_eq!(rel5, VersionRelationship::NewerPrerelease);
    }

    #[test]
    fn test_is_breaking_change() {
        // Major version change is breaking
        assert!(Version::is_breaking_change("1.0.0", "2.0.0"));

        // Minor version change is not breaking
        assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));

        // Patch version change is not breaking
        assert!(!Version::is_breaking_change("1.0.0", "1.0.1"));

        // Prerelease to stable of same version is not breaking
        assert!(!Version::is_breaking_change("1.0.0-alpha", "1.0.0"));
    }

    #[test]
    fn test_version_parsing() {
        // Valid version
        let valid = Version::parse("1.2.3");
        assert!(valid.is_ok());
        let v = valid.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        // Valid version with prerelease
        let pre = Version::parse("1.2.3-beta.1");
        assert!(pre.is_ok());
        let v = pre.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(!v.pre.is_empty());

        // Invalid version
        let invalid = Version::parse("not-a-version");
        assert!(invalid.is_err());
    }
}
