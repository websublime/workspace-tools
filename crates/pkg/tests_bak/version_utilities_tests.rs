#[cfg(test)]
mod version_utilities_tests {
    use sublime_package_tools::{
        Version, VersionError, VersionRelationship, VersionStability, VersionUpdateStrategy,
    };

    #[test]
    fn test_version_enum_methods() {
        // Test the Version enum conversion to and from strings
        assert_eq!(Version::from("major"), Version::Major);
        assert_eq!(Version::from("minor"), Version::Minor);
        assert_eq!(Version::from("patch"), Version::Patch);
        assert_eq!(Version::from("snapshot"), Version::Snapshot);
        assert_eq!(Version::from("unknown"), Version::Patch); // Default

        // Test to_string implementations
        assert_eq!(Version::Major.to_string(), "major");
        assert_eq!(Version::Minor.to_string(), "minor");
        assert_eq!(Version::Patch.to_string(), "patch");
        assert_eq!(Version::Snapshot.to_string(), "snapshot");
    }

    #[test]
    fn test_version_bump_major() {
        // Test bumping major version
        let cases = [
            ("1.0.0", "2.0.0"),
            ("1.2.3", "2.0.0"),
            ("0.1.0", "1.0.0"),
            ("1.0.0-alpha", "2.0.0"),
            ("1.0.0-beta.2", "2.0.0"),
            ("1.0.0+build.42", "2.0.0"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_major(input).expect("Failed to bump version");
            assert_eq!(bumped.to_string(), expected);
        }

        // Test with invalid version
        let result = Version::bump_major("not-a-version");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
    }

    #[test]
    fn test_version_bump_minor() {
        // Test bumping minor version
        let cases = [
            ("1.0.0", "1.1.0"),
            ("1.2.3", "1.3.0"),
            ("0.1.0", "0.2.0"),
            ("1.0.0-alpha", "1.1.0"),
            ("1.0.0-beta.2", "1.1.0"),
            ("1.0.0+build.42", "1.1.0"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_minor(input).expect("Failed to bump version");
            assert_eq!(bumped.to_string(), expected);
        }

        // Test with invalid version
        let result = Version::bump_minor("not-a-version");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
    }

    #[test]
    fn test_version_bump_patch() {
        // Test bumping patch version
        let cases = [
            ("1.0.0", "1.0.1"),
            ("1.2.3", "1.2.4"),
            ("0.1.0", "0.1.1"),
            ("1.0.0-alpha", "1.0.1"),
            ("1.0.0-beta.2", "1.0.1"),
            ("1.0.0+build.42", "1.0.1"),
        ];

        for (input, expected) in cases {
            let bumped = Version::bump_patch(input).expect("Failed to bump version");
            assert_eq!(bumped.to_string(), expected);
        }

        // Test with invalid version
        let result = Version::bump_patch("not-a-version");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
    }

    #[test]
    fn test_version_bump_snapshot() {
        // Test creating snapshot versions
        let cases = [
            ("1.0.0", "abc123", "1.0.0-alpha.abc123"),
            ("1.2.3", "def456", "1.2.3-alpha.def456"),
            ("0.1.0", "abcdef", "0.1.0-alpha.abcdef"),
            ("1.0.0-beta", "123456", "1.0.0-alpha.123456"), // Replaces existing prerelease
            ("1.0.0+build.42", "789abc", "1.0.0-alpha.789abc"), // Removes build metadata
        ];

        for (input, sha, expected) in cases {
            let bumped = Version::bump_snapshot(input, sha).expect("Failed to bump to snapshot");
            assert_eq!(bumped.to_string(), expected);
        }

        // Test with invalid version
        let result = Version::bump_snapshot("not-a-version", "abc123");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
    }

    #[test]
    fn test_version_parse() {
        // Test parsing version strings
        let valid_cases = [
            "1.0.0",
            "0.1.0",
            "1.2.3",
            "1.0.0-alpha",
            "1.0.0-beta.2",
            "1.0.0+build.42",
            "1.2.3-alpha.4+build.567",
        ];

        for version_str in valid_cases {
            let result = Version::parse(version_str);
            assert!(result.is_ok(), "Failed to parse valid version: {version_str}");
        }

        // Test with invalid versions
        let invalid_cases = ["not-a-version", "1.0", "v1.0.0", "1.0.0.0", "", " "];

        for version_str in invalid_cases {
            let result = Version::parse(version_str);
            assert!(result.is_err(), "Should not parse invalid version: {version_str}");
            assert!(matches!(result.unwrap_err(), VersionError::Parse { .. }));
        }
    }

    #[test]
    fn test_compare_versions() {
        // Test compare_versions function with various scenarios

        // Major version changes
        assert_eq!(Version::compare_versions("1.0.0", "2.0.0"), VersionRelationship::MajorUpgrade);
        assert_eq!(
            Version::compare_versions("2.0.0", "1.0.0"),
            VersionRelationship::MajorDowngrade
        );

        // Minor version changes
        assert_eq!(Version::compare_versions("1.0.0", "1.1.0"), VersionRelationship::MinorUpgrade);
        assert_eq!(
            Version::compare_versions("1.1.0", "1.0.0"),
            VersionRelationship::MinorDowngrade
        );

        // Patch version changes
        assert_eq!(Version::compare_versions("1.0.0", "1.0.1"), VersionRelationship::PatchUpgrade);
        assert_eq!(
            Version::compare_versions("1.0.1", "1.0.0"),
            VersionRelationship::PatchDowngrade
        );

        // Prerelease changes
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

        // Identical versions
        assert_eq!(Version::compare_versions("1.0.0", "1.0.0"), VersionRelationship::Identical);
        assert_eq!(
            Version::compare_versions("1.0.0-alpha", "1.0.0-alpha"),
            VersionRelationship::Identical
        );

        // Invalid versions
        assert_eq!(
            Version::compare_versions("not-a-version", "1.0.0"),
            VersionRelationship::Indeterminate
        );
        assert_eq!(
            Version::compare_versions("1.0.0", "not-a-version"),
            VersionRelationship::Indeterminate
        );
    }

    #[test]
    fn test_version_relationship_display() {
        // Test string representation of version relationships
        assert_eq!(VersionRelationship::MajorUpgrade.to_string(), "MajorUpgrade");
        assert_eq!(VersionRelationship::MinorUpgrade.to_string(), "MinorUpgrade");
        assert_eq!(VersionRelationship::PatchUpgrade.to_string(), "PatchUpgrade");
        assert_eq!(VersionRelationship::PrereleaseToStable.to_string(), "PrereleaseToStable");
        assert_eq!(VersionRelationship::NewerPrerelease.to_string(), "NewerPrerelease");
        assert_eq!(VersionRelationship::Identical.to_string(), "Identical");
        assert_eq!(VersionRelationship::MajorDowngrade.to_string(), "MajorDowngrade");
        assert_eq!(VersionRelationship::MinorDowngrade.to_string(), "MinorDowngrade");
        assert_eq!(VersionRelationship::PatchDowngrade.to_string(), "PatchDowngrade");
        assert_eq!(VersionRelationship::StableToPrerelease.to_string(), "StableToPrerelease");
        assert_eq!(VersionRelationship::OlderPrerelease.to_string(), "OlderPrerelease");
        assert_eq!(VersionRelationship::Indeterminate.to_string(), "Indeterminate");
    }

    #[test]
    fn test_is_breaking_change() {
        // Test detection of breaking changes

        // Major version changes are breaking
        assert!(Version::is_breaking_change("1.0.0", "2.0.0"));
        assert!(Version::is_breaking_change("0.9.9", "1.0.0"));
        assert!(Version::is_breaking_change("1.2.3", "2.0.0-alpha"));

        // Minor and patch changes are not breaking
        assert!(!Version::is_breaking_change("1.0.0", "1.1.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.0.1"));
        assert!(!Version::is_breaking_change("1.1.0", "1.2.0"));

        // Prerelease changes depend on the major.minor.patch parts
        assert!(!Version::is_breaking_change("1.0.0-alpha", "1.0.0"));
        assert!(!Version::is_breaking_change("1.0.0", "1.0.0-beta"));
        assert!(!Version::is_breaking_change("1.0.0-alpha", "1.0.0-beta"));

        // Invalid versions are conservatively treated as breaking
        assert!(Version::is_breaking_change("not-a-version", "1.0.0"));
        assert!(Version::is_breaking_change("1.0.0", "not-a-version"));
    }

    #[test]
    fn test_update_strategy_default() {
        // Test default update strategy
        assert_eq!(VersionUpdateStrategy::default(), VersionUpdateStrategy::MinorAndPatch);
    }

    #[test]
    fn test_stability_default() {
        // Test default stability preference
        assert_eq!(VersionStability::default(), VersionStability::StableOnly);
    }

    #[test]
    fn test_version_error_handling() {
        // Test version error handling

        // Test Parse error
        let parse_err = Version::parse("not-a-version").unwrap_err();
        assert!(matches!(parse_err, VersionError::Parse { .. }));

        // Convert semver error to VersionError
        let semver_err = semver::Version::parse("not-a-version").unwrap_err();
        let version_err = VersionError::from(semver_err);
        assert!(matches!(version_err, VersionError::Parse { .. }));

        // Test InvalidVersion error (This would normally come from specific validation rules)
        let invalid_err = VersionError::InvalidVersion("Test invalid version".to_string());
        assert!(matches!(invalid_err, VersionError::InvalidVersion(_)));

        // Test as_ref implementation
        assert_eq!(parse_err.as_ref(), "VersionErrorParse");
        assert_eq!(invalid_err.as_ref(), "VersionErrorInvalidVersion");
    }
}
