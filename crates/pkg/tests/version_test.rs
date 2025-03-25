#[cfg(test)]
mod version_tests {
    use sublime_package_tools::{
        Version, VersionRelationship, VersionStability, VersionUpdateStrategy,
    };

    #[test]
    fn test_version_from_str() {
        assert_eq!(Version::from("major"), Version::Major);
        assert_eq!(Version::from("minor"), Version::Minor);
        assert_eq!(Version::from("patch"), Version::Patch);
        assert_eq!(Version::from("snapshot"), Version::Snapshot);
        assert_eq!(Version::from("unknown"), Version::Patch); // Default case
    }

    #[test]
    fn test_version_display() {
        assert_eq!(Version::Major.to_string(), "major");
        assert_eq!(Version::Minor.to_string(), "minor");
        assert_eq!(Version::Patch.to_string(), "patch");
        assert_eq!(Version::Snapshot.to_string(), "snapshot");
    }

    #[test]
    fn test_bump_major() {
        let cases = [
            ("1.2.3", "2.0.0"),
            ("0.1.0", "1.0.0"),
            ("1.0.0-alpha", "2.0.0"),
            ("1.2.3+build", "2.0.0"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_major(input).unwrap();
            assert_eq!(bumped.to_string(), expected);
        }
    }

    #[test]
    fn test_bump_minor() {
        let cases = [
            ("1.2.3", "1.3.0"),
            ("0.1.0", "0.2.0"),
            ("1.2.3-alpha", "1.3.0"),
            ("1.2.3+build", "1.3.0"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_minor(input).unwrap();
            assert_eq!(bumped.to_string(), expected);
        }
    }

    #[test]
    fn test_bump_patch() {
        let cases = [
            ("1.2.3", "1.2.4"),
            ("0.1.0", "0.1.1"),
            ("1.2.3-alpha", "1.2.4"),
            ("1.2.3+build", "1.2.4"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_patch(input).unwrap();
            assert_eq!(bumped.to_string(), expected);
        }
    }

    #[test]
    fn test_bump_snapshot() {
        let cases = [
            ("1.2.3", "abc123", "1.2.3-alpha.abc123"),
            ("0.1.0", "def456", "0.1.0-alpha.def456"),
            ("1.2.3-beta", "ghi789", "1.2.3-alpha.ghi789"),
            ("1.2.3+build", "jkl012", "1.2.3-alpha.jkl012"),
        ];

        for (input, sha, expected) in cases {
            let bumped = Version::bump_snapshot(input, sha).unwrap();
            assert_eq!(bumped.to_string(), expected);
        }
    }

    #[test]
    fn test_version_parse_error() {
        let invalid = "not.a.version";
        assert!(Version::bump_major(invalid).is_err());
        assert!(Version::bump_minor(invalid).is_err());
        assert!(Version::bump_patch(invalid).is_err());
        assert!(Version::bump_snapshot(invalid, "abc123").is_err());
    }

    #[test]
    fn test_compare_versions() {
        // Test major upgrades
        assert_eq!(Version::compare_versions("1.0.0", "2.0.0"), VersionRelationship::MajorUpgrade);
        assert_eq!(Version::compare_versions("1.2.3", "2.0.0"), VersionRelationship::MajorUpgrade);

        // Test minor upgrades
        assert_eq!(Version::compare_versions("1.0.0", "1.1.0"), VersionRelationship::MinorUpgrade);
        assert_eq!(Version::compare_versions("1.2.0", "1.3.0"), VersionRelationship::MinorUpgrade);

        // Test patch upgrades
        assert_eq!(Version::compare_versions("1.0.0", "1.0.1"), VersionRelationship::PatchUpgrade);
        assert_eq!(Version::compare_versions("1.1.1", "1.1.2"), VersionRelationship::PatchUpgrade);

        // Test prerelease to stable
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0"),
            VersionRelationship::PrereleaseToStable
        );
        assert_eq!(
            Version::compare_versions("2.0.0-beta.1", "2.0.0"),
            VersionRelationship::PrereleaseToStable
        );

        // Test newer prerelease
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0-beta"),
            VersionRelationship::NewerPrerelease
        );
        assert_eq!(
            Version::compare_versions("1.0.0-alpha.1", "1.0.0-alpha.2"),
            VersionRelationship::NewerPrerelease
        );

        // Test identical versions
        assert_eq!(Version::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0-alpha"),
            VersionRelationship::Identical
        );

        // Test major downgrades
        assert_eq!(
            Version::compare_versions("2.0.0", "1.0.0"),
            VersionRelationship::MajorDowngrade
        );
        assert_eq!(
            Version::compare_versions("3.2.1", "2.0.0"),
            VersionRelationship::MajorDowngrade
        );

        // Test minor downgrades
        assert_eq!(
            Version::compare_versions("1.2.0", "1.1.0"),
            VersionRelationship::MinorDowngrade
        );
        assert_eq!(
            Version::compare_versions("1.3.0", "1.2.0"),
            VersionRelationship::MinorDowngrade
        );

        // Test patch downgrades
        assert_eq!(
            Version::compare_versions("1.0.2", "1.0.1"),
            VersionRelationship::PatchDowngrade
        );
        assert_eq!(
            Version::compare_versions("1.1.3", "1.1.2"),
            VersionRelationship::PatchDowngrade
        );

        // Test stable to prerelease
        assert_eq!(
            Version::compare_versions("1.0.0", "1.0.0-alpha"),
            VersionRelationship::StableToPrerelease
        );
        assert_eq!(
            Version::compare_versions("2.0.0", "2.0.0-beta.1"),
            VersionRelationship::StableToPrerelease
        );

        // Test older prerelease
        assert_eq!(
            Version::compare_versions("1.0.0-beta", "1.0.0-alpha"),
            VersionRelationship::OlderPrerelease
        );
        assert_eq!(
            Version::compare_versions("1.0.0-alpha.2", "1.0.0-alpha.1"),
            VersionRelationship::OlderPrerelease
        );

        // Test indeterminate
        assert_eq!(
            Version::compare_versions("not-a-version", "1.0.0"),
            VersionRelationship::Indeterminate
        );
        assert_eq!(
            Version::compare_versions("1.0.0", "also-not-a-version"),
            VersionRelationship::Indeterminate
        );
    }

    #[test]
    fn test_is_breaking_change() {
        // True cases (breaking changes)
        assert!(Version::is_breaking_change("1.0.0", "2.0.0"));
        assert!(Version::is_breaking_change("0.9.9", "1.0.0"));
        assert!(Version::is_breaking_change("1.2.3", "2.0.0-alpha"));

        // False cases (non-breaking changes)
        assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.0.1"));
        assert!(!Version::is_breaking_change("1.0.0-alpha", "1.0.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.0.0-beta"));

        // Invalid version cases
        assert!(Version::is_breaking_change("not-a-version", "1.0.0"));
        assert!(Version::is_breaking_change("1.0.0", "also-not-a-version"));
    }

    #[test]
    fn test_version_update_strategy_default() {
        assert_eq!(VersionUpdateStrategy::default(), VersionUpdateStrategy::MinorAndPatch);
    }

    #[test]
    fn test_version_stability_default() {
        assert_eq!(VersionStability::default(), VersionStability::StableOnly);
    }
}
